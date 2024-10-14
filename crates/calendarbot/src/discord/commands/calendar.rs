/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */
use crate::ApplicationContext;
use anyhow::Result;
use log::info;
use poise::serenity_prelude::{self as serenity};

#[derive(Debug, poise::Modal)]
struct NewCalendarModal {
    google_calendar_id: String,
    channel_id: String,
}

#[poise::command(slash_command, guild_only, category = "Google calendar")]
pub async fn new(
    ctx: ApplicationContext<'_>,
    #[description = "Google Calendar ID"] calendar_id: String,
    #[channel_types("Text")]
    #[description = "Channel (defaults to the current channel)"]
    channel: Option<serenity::Channel>,
) -> Result<()> {
    Ok(())
}
