use std::{
    path::{Path, PathBuf},
    sync::{atomic::AtomicU64, atomic::Ordering, Arc},
};

use anyhow::{bail, Result};
use async_trait::async_trait;
use tracing::{debug, info, trace, warn};

use crate::{
    //application::background::SyncPipes,
    repo::{
        filesystem::FileWalker,
        iterator::{FileSource, RepoDirectoryEntry},
        types::{RepoMetadata, RepoRef, Repository},
    },
    state::schema_version::get_schema_version,
};

use super::{
    caching::{SnippetCache, SnippetCacheKeys, SnippetCacheSnapshot},
    indexer::{get_text_field, get_u64_field, Indexable},
    schema::Snippet,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SnippetDocument {
    pub relative_path: String,
    pub repo_name: String,
    pub repo_ref: String,
    pub content: String,
    pub line_start: u64,
    pub line_end: u64,
    pub score: f32,
}

pub struct SnippetReader;

impl SnippetReader {
    pub fn read_document(schema: &Snippet) -> SnippetDocument {
        let path = get_text_field(&doc, schema.relative_path);
        let repo_ref = get_text_field(&doc, schema.repo_ref);
        let content = get_text_field(&doc, schema.content);
        let line_start = get_u64_field(&doc, schema.start_line);
        let line_end = get_u64_field(&doc, schema.end_line);

        SnippetDocument {
            relative_path: path,
            repo_name: "".to_owned(),
            repo_ref,
            content,
            line_start,
            line_end,
            score: 0.0,
        }
    }

    pub fn read_document_with_score(
        schema: &Snippet,
        score: f32,
    ) -> SnippetDocument {
        let mut code_snippet_doc = Self::read_document(schema, doc);
        code_snippet_doc.score = score;
        code_snippet_doc
    }
}

pub struct Workload<'a> {
    cache: &'a SnippetCacheSnapshot<'a>,
    repo_disk_path: &'a Path,
    repo_name: &'a str,
    repo_metadata: &'a RepoMetadata,
    repo_ref: String,
    relative_path: PathBuf,
    normalized_path: PathBuf,
    commit_hash: String,
}

impl<'a> Workload<'a> {
    pub fn new(
        cache: &'a SnippetCacheSnapshot<'a>,
        repo_disk_path: &'a Path,
        repo_name: &'a str,
        repo_metadata: &'a RepoMetadata,
        repo_ref: String,
        relative_path: PathBuf,
        normalized_path: PathBuf,
        commit_hash: String,
    ) -> Self {
        Self {
            cache,
            repo_disk_path,
            repo_name,
            repo_metadata,
            repo_ref,
            relative_path,
            normalized_path,
            commit_hash,
        }
    }
}

impl<'a> Workload<'a> {
    // These cache keys are important as they also encode information about the
    // the file path in the cache, which implies that for each file we will have
    // a unique cache key.
    fn cache_keys(&self, dir_entry: &RepoDirectoryEntry) -> SnippetCacheKeys {
        let semantic_hash = {
            let mut hash = blake3::Hasher::new();
            hash.update(get_schema_version().as_bytes());
            hash.update(self.relative_path.to_string_lossy().as_ref().as_ref());
            hash.update(self.repo_ref.as_bytes());
            hash.update(dir_entry.buffer().unwrap_or_default().as_bytes());
            hash.finalize().to_hex().to_string()
        };

        let file_content_hash = match dir_entry.buffer() {
            Some(content) => {
                let mut hash = blake3::Hasher::new();
                hash.update(content.as_bytes())
                    .finalize()
                    .to_hex()
                    .to_string()
            }
            None => "no_content_hash".to_owned(),
        };

        let file_path = dir_entry.path();

        debug!(
            ?semantic_hash,
            ?file_content_hash,
            ?file_path,
            "cache keys"
        );

        SnippetCacheKeys::new(
            self.commit_hash.to_owned(),
            self.normalized_path
                .to_str()
                .map_or("mangled_path".to_owned(), |path| path.to_owned()),
            file_content_hash,
        )
    }
}

#[async_trait]
impl Indexable for Snippet {
    async fn index_repository(
        &self,
        reporef: &RepoRef,
        repo: &Repository,
        repo_metadata: &RepoMetadata,
        writer: &IndexWriter,
        pipes: &SyncPipes,
    ) -> Result<()> {
        let code_snippet_cache = Arc::new(SnippetCache::for_repo(&self.sql, reporef));
        let cache = code_snippet_cache.retrieve().await;
        let repo_name = reporef.indexed_name();
        let processed = &AtomicU64::new(0);

        let file_worker = |count: usize| {
            let cache = &cache;
            move |dir_entry: RepoDirectoryEntry| {
                let completed = processed.fetch_add(1, Ordering::Relaxed);

                let entry_disk_path = dir_entry.path().unwrap().to_owned();
                debug!(
                    entry_disk_path,
                    "processing entry for indexing code snippet"
                );
                let relative_path = {
                    let entry_srcpath = PathBuf::from(&entry_disk_path);
                    entry_srcpath
                        .strip_prefix(&repo.disk_path)
                        .map(ToOwned::to_owned)
                        .unwrap_or(entry_srcpath)
                };
                debug!(?relative_path, "relative path for indexing code snippets");
                let normalized_path = repo.disk_path.join(&relative_path);

                let workload = Workload::new(
                    cache,
                    &repo.disk_path,
                    &repo_name,
                    repo_metadata,
                    reporef.to_string(),
                    relative_path,
                    normalized_path,
                    repo_metadata.commit_hash.clone(),
                );

                trace!(entry_disk_path, "queueing entry for code snippet indexing");
                if let Err(err) = self.worker(dir_entry, workload, writer) {
                    warn!(%err, entry_disk_path, "indexing failed code snippet; finished");
                }
                debug!(entry_disk_path, "finished indexing code snippet");
                pipes.index_percent(((completed as f32 / count as f32) * 100f32) as u8);
            }
        };

        let start = std::time::Instant::now();

        let walker = FileWalker::index_directory(&repo.disk_path);
        let count = walker.len();
        walker.for_each(pipes, file_worker(count));

        if pipes.is_cancelled() {
            bail!("cancelled indexing");
        }

        info!(?repo.disk_path, "indexing finished, took {:?}", start.elapsed());

        code_snippet_cache
            .synchronize(cache, |key| {
                writer.delete_term(Term::from_field_text(self.unique_hash, key));
            })
            .await?;

        pipes.index_percent(100);
        Ok(())
    }

    fn delete_by_repo(&self, writer: &IndexWriter, repo: &Repository) {
        writer.delete_term(Term::from_field_text(
            self.repo_disk_path,
            &repo.disk_path.to_string_lossy(),
        ));
    }

    /// Return the tantivy `Schema` of the current index
    fn schema(&self) -> Schema {
        self.schema.clone()
    }
}

impl Snippet {
    fn worker(
        &self,
        dir_entry: RepoDirectoryEntry,
        workload: Workload<'_>,
        writer: &IndexWriter,
    ) -> Result<()> {
        let cache_keys = workload.cache_keys(&dir_entry);
        let last_commit = workload
            .repo_metadata
            .last_commit_unix_secs
            .unwrap_or_default();
        trace!("processing file for code snippets");
        match dir_entry {
            _ if workload.cache.is_fresh(&cache_keys) => {
                info!(?cache_keys, "code snippet cache is fresh");
                return Ok(());
            }
            RepoDirectoryEntry::Dir(dir) => {
                debug!("not indexing snippets from the directory {:?}", dir);
            }
            RepoDirectoryEntry::File(file) => {
                // Here we get back a list of documents all of which we have to write
                // to the index
                let documents = file.build_documents(
                    self,
                    &workload,
                    &cache_keys,
                    last_commit,
                    workload.cache.parent(),
                );
                // add all the generated code snippets to the index
                documents.into_iter().for_each(|document| {
                    // TODO(codestory): This kind of expect is bad, but we need
                    // it for now while we are testing
                    let _ = writer
                        .add_document(document)
                        .expect("writer adding code snippet should always work");
                });
            }
            RepoDirectoryEntry::Other => {
                bail!("found an entry which is neither a file or a document");
            }
        }
        Ok(())
    }
}
