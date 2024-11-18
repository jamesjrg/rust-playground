// This is where we will define the core application and all the related things
// on how to startup the application

use tracing::{debug, warn};
use once_cell::sync::OnceCell;
static LOGGER_INSTALLED: OnceCell<bool> = OnceCell::new();
use super::logging::tracing::tracing_subscribe;

#[derive(Clone)]
pub struct Application {
}

impl Application {
    pub fn install_logging() {
        if let Some(true) = LOGGER_INSTALLED.get() {
            return;
        }

        if !tracing_subscribe() {
            warn!("Failed to install tracing_subscriber.");
        };

        if color_eyre::install().is_err() {
            warn!("Failed to install color-eyre.");
        };

        println!("hello from println");
        debug!("hello from debug");

        LOGGER_INSTALLED.set(true).unwrap();
    }

    pub async fn initialize() -> anyhow::Result<Self> {
        Ok(Self {
        })
    }
}
