use anyhow::Result;
use axum::Extension;
use std::net::SocketAddr;
use tokio::signal;
use tokio::sync::oneshot;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::trace::{self, TraceLayer, OnRequest, OnResponse};
use tracing::{debug, error, info, Level};
use axum_test::application::application::Application;
use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post}
};
use http;
use http_body_util::BodyExt;


pub type Router<S = Application> = axum::Router<S>;

#[tokio::main]
async fn main() -> Result<()> {
    info!("hello from info");
    println!("hello from println");

    // We get the logging setup first
    debug!("installing logging to local file");
    Application::install_logging();

    // Create a oneshot channel
    let (tx, rx) = oneshot::channel();

    // Spawn a task to listen for signals
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("failed to listen for event");
        let _ = tx.send(());
    });

    // We initialize the logging here
    let application = Application::initialize().await?;
    println!("initialized application");
    debug!("initialized application");

    // Main logic
    tokio::select! {
        // Start the webserver
        _ = run(application) => {
            // Your server logic
        }
        _ = rx => {
            // Signal received, this block will be executed.
            // Drop happens automatically when variables go out of scope.
            debug!("Signal received, cleaning up...");
        }
    }

    Ok(())
}

pub async fn run(application: Application) -> Result<()> {
    let mut joins = tokio::task::JoinSet::new();

    joins.spawn(start(application));

    while let Some(result) = joins.join_next().await {
        if let Ok(Err(err)) = result {
            error!(?err, "failure");
            return Err(err);
        }
    }

    Ok(())
}

#[derive(Clone, Debug)]
struct MyOnRequest {}

impl<B> OnRequest<B> for MyOnRequest {
    fn on_request(&mut self, request: &http::Request<B>, _span: &tracing::Span) {
        tracing::info!(
            "request: {} {}",
            request.method(),
            request.uri().path()
        );
    }
}

#[derive(Clone, Debug)]
struct MyOnResponse {}

impl<B> OnResponse<B> for MyOnResponse {
    fn on_response(self, response: &http::Response<B>, _latency: core::time::Duration, _: &tracing::Span) {
        tracing::info!("hello");
    }
}

async fn printing_middleware(request: axum::extract::Request, next: axum::middleware::Next) -> Result<impl IntoResponse, Response> {
    let request = buffer_request_body(request).await?;

    Ok(next.run(request).await)
}

async fn buffer_request_body(request: axum::extract::Request) -> Result<axum::extract::Request, Response> {
    let (parts, body) = request.into_parts();

    let bytes = body
        .collect()
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?
        .to_bytes();

    tracing::debug!(body = ?bytes);

    Ok(axum::http::Request::from_parts(parts, Body::from(bytes)))
}

pub async fn start(app: Application) -> anyhow::Result<()> {
    const PORT: u16 = 4000;

    println!("Port: {}", PORT);
    let bind = SocketAddr::new("127.0.0.1".parse()?, PORT);

    let routes = Router::new()
        .route("/version", get(axum_test::route_handlers::version))
        .route("/postme", post(axum_test::route_handlers::handle_poster));

    // both protected and public merged into api
    let mut api = Router::new().merge(routes);

    api = api.route("/health", get(axum_test::route_handlers::health));

    let api = api
        .layer(Extension(app.clone()))
        .with_state(app.clone())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_request(MyOnRequest{})
                .on_response(MyOnResponse{})
                //.on_request(trace::DefaultOnRequest::new())
                //.on_response(trace::DefaultOnResponse::new().level(Level::INFO))
            )
        .layer(CatchPanicLayer::new())
        .layer(axum::middleware::from_fn(printing_middleware));

    let router = Router::new().nest("/api", api);

    let listener = tokio::net::TcpListener::bind(&bind).await.unwrap();
    axum::serve(listener, router.into_make_service()).await.unwrap();

    Ok(())
}
