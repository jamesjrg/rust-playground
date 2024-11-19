use tracing_subscriber::{fmt, prelude::*, EnvFilter};


pub fn tracing_subscribe() -> bool {
    let env_filter_stdout_layer = fmt::layer()
        // Disable the hyper logs or else its a lot of log spam
        .with_filter(
            EnvFilter::from_default_env()
                .add_directive("hyper=off".parse().unwrap())
                .add_directive("tantivy=off".parse().unwrap()), // .add_directive("error".parse().unwrap()),
        );

    tracing_subscriber::registry()
        .with(env_filter_stdout_layer)
        .try_init()
        .is_ok()
}

