Exploring web search APIs in Rust

CLI program that searches the web using either the Brave Search API or the Exa API.

CLI arguments:
==============

--provider - exa or brave, defaults to exa

--query - the query

--limit - how many results to fetch. Defaults to 3

--search-type - for exa, this can be one of keyword, neural, or auto. Defaults to auto.

--summary-query - a custom summary query to power exa's summarizer feature. Defaults to null.

Brave also advertises functionality for summarizing contents, but it doesn't support custom queries, and in my very limiting testing didn't seem to appear in the results for the types of queries I was using.

--full-text

Defaults to false.

For exa, includes the full text of the page in the results.


Caching
=======

The program saves results to disk and initializes a thread-safe in-memory cache from this file at startup.

The in-memory cache is pointless for a CLI program, but would matter if this were integrated elsewhere.


Usage:
======

cargo run -- --provider exa
cargo run -- --provider brave