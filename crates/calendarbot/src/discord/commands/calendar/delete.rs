/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */

use crate::schema::calendars::dsl as calendars;
use crate::schema::guilds_calendars::dsl as guilds_calendars;
use crate::ApplicationContext;
use anyhow::Result;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use log::{error, warn};
use poise::serenity_prelude as serenity;

#[poise::command(slash_command, guild_only, category = "Google Calendar")]
pub async fn delete(
    ctx: ApplicationContext<'_>,
    #[channel_types("Text")]
    #[description = "Channel (defaults to the current channel)"]
    channel: Option<serenity::GuildChannel>,
) -> Result<()> {
    let channel = match channel {
        Some(c) => c,
        None => ctx.guild_channel().await.unwrap(),
    };

    let mut db = ctx.data().db.get().await?;

    let res = guilds_calendars::guilds_calendars
        .filter(guilds_calendars::channelId.eq(channel.id.get().to_string()))
        .select((guilds_calendars::calendar_id, guilds_calendars::messageId))
        .first::<(i32, Option<String>)>(&mut db)
        .await;

    if res.is_err() {
        let _ = ctx.reply("This channel doesn't have a calendar").await?;
        return Ok(());
    }
    let res = res?;

    let calendar_id = res.0;

    // Deleting the calendar message
    if let Some(message_id) = res.1 {
        let message_id = serenity::MessageId::new(message_id.parse::<u64>()?);

        let res = channel.delete_messages(&ctx.http(), vec![message_id]).await;

        if res.is_err() {
            warn!("Unable to delete message (maybe the bot is missing the MANAGE_MESSAGE permission?): {:?}", res);
        }
    }

    // Remove the calendar from the database
    let del = diesel::delete(
        guilds_calendars::guilds_calendars
            .filter(guilds_calendars::channelId.eq(channel.id.get().to_string())),
    )
    .execute(&mut db)
    .await;

    if del.is_err() {
        let _ = ctx.reply("Unable to delete calendar").await?;
        error!("Unable to delete calendar: {:?}", del);
        return Ok(());
    }

    // Remove the calendar from the database if it's not used anymore
    // (no other guild or channel is using it)
    let res = guilds_calendars::guilds_calendars
        .filter(guilds_calendars::calendar_id.eq(calendar_id))
        .select(guilds_calendars::calendar_id)
        .first::<i32>(&mut db)
        .await;

    if res.is_err() {
        // The calendar is not used anymore
        // Remove it from the database
        let del = diesel::delete(calendars::calendars.filter(calendars::id.eq(calendar_id)))
            .execute(&mut db)
            .await;
        if del.is_err() {
            error!("Failed to delete calendar from Calendar table: {:?}", del);
            ctx.reply("Unable to delete calendar").await?;
        }
    }

    ctx.send(
        poise::CreateReply::default()
            .content("Successfully deleted")
            .reply(true)
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
