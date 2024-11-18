use anyhow::Result;
use axum::routing::get;
use axum::Extension;
use std::net::SocketAddr;
use tokio::signal;
use tokio::sync::oneshot;
use tower_http::{catch_panic::CatchPanicLayer};
use tracing::{debug, error, info};
use axum_test::application::application::Application;

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

pub async fn start(app: Application) -> anyhow::Result<()> {
    const PORT: u16 = 4000;

    println!("Port: {}", PORT);
    let bind = SocketAddr::new("127.0.0.1".parse()?, PORT);

    let routes = Router::new()
        .route("/version", get(axum_test::route_handlers::version));

    // both protected and public merged into api
    let mut api = Router::new().merge(routes);

    api = api.route("/health", get(axum_test::route_handlers::health));

    let api = api
        .layer(Extension(app.clone()))
        .with_state(app.clone())
        .layer(CatchPanicLayer::new());

    let router = Router::new().nest("/api", api);

    axum::Server::bind(&bind)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
