/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */

use poise::serenity_prelude as serenity;
use std::sync::Arc;

#[derive(Clone)]
pub struct LocalCache {
    pub(crate) cache: Arc<serenity::Cache>,
    pub(crate) client: Arc<serenity::Http>,
}
impl LocalCache {
    pub(crate) fn new(client: Arc<serenity::Http>) -> Self {
        Self {
            cache: Arc::new(serenity::Cache::default()),
            client,
        }
    }
}

impl serenity::CacheHttp for LocalCache {
    fn http(&self) -> &serenity::Http {
        &self.client
    }

    fn cache(&self) -> Option<&Arc<serenity::Cache>> {
        Some(&self.cache)
    }
}
