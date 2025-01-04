/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */

pub mod discord;
pub mod events;
pub mod gcalendar;
pub mod models;
pub mod schema;
pub mod types;

use crate::events::UpdateCalendarEvent;
use crate::gcalendar::GCalendar;
use anyhow::Error;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use events::CalendarCommands;
use poise::serenity_prelude as serenity;
use std::env;

use dotenvy::dotenv;

type Context<'a> = poise::Context<'a, types::GlobalData, Error>;
type ApplicationContext<'a> = poise::ApplicationContext<'a, types::GlobalData, Error>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub async fn get_connection_pool(database_url: String) -> Pool<diesel_async::AsyncPgConnection> {
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
    Pool::builder(config)
        .build()
        .expect("Failed to create pool.")
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = get_connection_pool(database_url).await;

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found");
    let intents = serenity::GatewayIntents::non_privileged();

    let (update_calendar_tx, update_calendar_rx) =
        tokio::sync::mpsc::channel::<UpdateCalendarEvent>(200);

    let (gcalendar_tx, worker_thread_rx) = tokio::sync::mpsc::channel::<CalendarCommands>(200);

    GCalendar::new(pool.clone(), update_calendar_tx)
        .await
        .expect("Unable to connect to google calendar")
        .init_threads(worker_thread_rx);

    let data = types::GlobalData::new(pool.clone(), gcalendar_tx).expect("Unable to load config!");

    let mut client = discord::Discord::new(token, intents)
        .init(update_calendar_rx, data)
        .await;

    if let Err(why) = client.start().await {
        println!("An error occured while running the client: {:?}", why);
    }
}
