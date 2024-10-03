pub mod init;

mod calendar_event;
mod commands;
mod local_cache;

use local_cache::LocalCache;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::serenity;

pub struct Discord {
    token: String,
    intents: serenity::GatewayIntents,
    cache: Arc<Mutex<Option<LocalCache>>>,
}

impl Discord {
    pub fn new(token: String, gateway_intents: serenity::GatewayIntents) -> Self {
        Self {
            token,
            intents: gateway_intents,
            cache: Arc::new(Mutex::new(None)),
        }
    }
}
