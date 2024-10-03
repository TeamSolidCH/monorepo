/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */
mod calendar;
mod discord;
pub mod models;
pub mod schema;
pub mod types;

use anyhow::Error;
use calendar::UpdateCalendarEvent;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use log::info;
use poise::serenity_prelude as serenity;
use std::time::Duration;
use std::{env, thread};

use dotenvy::dotenv;

type Context<'a> = poise::Context<'a, types::Data, Error>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub fn get_connection_pool(database_url: String) -> Pool<ConnectionManager<PgConnection>> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Failed to create pool.")
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = get_connection_pool(database_url);

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found");
    let intents = serenity::GatewayIntents::non_privileged();

    let data = types::Data::new(pool.clone()).expect("Unable to load config!");

    let (tx, rx) = tokio::sync::mpsc::channel::<UpdateCalendarEvent>(200);

    tokio::spawn(async move {
        let g_client = calendar::GCalendar::new(pool.clone())
            .await
            .expect("Unable to connect to google calendar");

        loop {
            info!("Updating calendars");
            g_client.update_calendars(tx.clone()).await;
            thread::sleep(Duration::from_secs(10));
        }
    });

    let mut client = discord::Discord::new(token, intents).init(rx, data).await;

    if let Err(why) = client.start().await {
        println!("An error occured while running the client: {:?}", why);
    }
}
