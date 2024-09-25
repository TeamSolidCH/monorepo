use std::env;

/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */
use anyhow::{Error, Result};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use poise::serenity_prelude as serenity;

pub struct Data {
    pub application_id: serenity::UserId,
    pub client_id: serenity::UserId,
    pub bot_start_time: std::time::Instant,
    pub db: Pool<ConnectionManager<PgConnection>>,
}

impl Data {
    pub fn new(db_connection: Pool<ConnectionManager<PgConnection>>) -> Result<Data> {
        Ok(Self {
            application_id: env::var("APPLICATION_ID")
                .expect("APPLICATION_ID not found")
                .parse::<u64>()?
                .into(),
            client_id: env::var("CLIENT_ID")
                .expect("CLIENT_ID not found")
                .parse::<u64>()?
                .into(),
            bot_start_time: std::time::Instant::now(),
            db: db_connection,
        })
    }
}

pub type Context<'a> = poise::Context<'a, Data, Error>;

pub const EMBED_COLOR: (u8, u8, u8) = (0xb7, 0x47, 0x00);
