use crate::infrastructure::config::Config;
use std::sync::Arc;

pub struct ServiceContainer {
    pub config: Config,
}

impl ServiceContainer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
} 