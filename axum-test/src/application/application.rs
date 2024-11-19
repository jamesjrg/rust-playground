// This is where we will define the core application and all the related things
// on how to startup the application

use tracing::{debug, warn};
use super::logging::tracing::tracing_subscribe;

#[derive(Clone)]
pub struct Application {
}

impl Application {
    pub fn install_logging() {
        if !tracing_subscribe() {
            warn!("Failed to install tracing_subscriber.");
        };

        if color_eyre::install().is_err() {
            warn!("Failed to install color-eyre.");
        };

        println!("hello from println");
        debug!("hello from debug");
    }

    pub async fn initialize() -> anyhow::Result<Self> {
        Ok(Self {
        })
    }
}
