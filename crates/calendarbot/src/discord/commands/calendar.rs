/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */
use crate::events::CalendarCommands;
use crate::schema::calendars::dsl as calendars;
use crate::schema::guilds::dsl as guilds;
use crate::schema::guilds_calendars::dsl as guilds_calendars;
use crate::ApplicationContext;
use anyhow::Result;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use log::{error, warn};
use poise::serenity_prelude as serenity;
use tokio::sync::oneshot;

#[poise::command(slash_command, guild_only, category = "Google calendar")]
pub async fn new(
    ctx: ApplicationContext<'_>,
    #[description = "Google Calendar ID"] calendar_id: String,
    #[channel_types("Text")]
    #[description = "Channel (defaults to the current channel)"]
    channel: Option<serenity::GuildChannel>,
) -> Result<()> {
    let channel = match channel {
        Some(c) => c,
        None => ctx.guild_channel().await.unwrap(),
    };

    let mut db = ctx.data().db.get().await.unwrap();

    // Checking if the channel as calendar
    let res = guilds_calendars::guilds_calendars
        .filter(guilds_calendars::channelid.eq(channel.id.get().to_string()))
        .select(guilds_calendars::channelid)
        .first::<String>(&mut db)
        .await;

    if res.is_ok() {
        let _ = ctx.reply("This channel already has a calendar").await?;
        return Ok(());
    }

    // Checking if the calendar is already present in db
    let res = calendars::calendars
        .filter(calendars::googleid.eq(&calendar_id))
        .select(calendars::id)
        .first::<i32>(&mut db)
        .await;

    let check_if_valid = res.is_err();

    let db_cal_id = match res {
        Ok(id) => Some(id),
        Err(_) => None,
    };

    // Checking if the calendar ID is valid and accessible from gcalendar
    if check_if_valid {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = CalendarCommands::VerifyCalendarId {
            calendar_id: calendar_id.clone(),
            resp: resp_tx,
        };

        ctx.defer().await.unwrap();

        ctx.data().gcalendar_tx.clone().send(cmd).await.unwrap();

        let is_valid = resp_rx.await.unwrap_or(Ok(false)).unwrap_or(false);

        if !is_valid {
            let _ = ctx.reply("Invalid calendar ID").await?;
            return Ok(());
        }
    }

    // Checking if the guild is already present in db
    // If not present, add it
    let guild_id = ctx.guild_id().unwrap().get().to_string();

    let res = guilds::guilds
        .filter(guilds::discordid.eq(guild_id.clone()))
        .select(guilds::id)
        .first::<i32>(&mut db)
        .await;

    let guild_id = match res {
        Ok(id) => id,
        Err(_) => diesel::insert_into(guilds::guilds)
            .values(guilds::discordid.eq(guild_id))
            .returning(guilds::id)
            .get_result::<i32>(&mut db)
            .await
            .expect("Unable to insert guild into database"),
    };

    // Inserting calendar into db
    let db_cal_id = match db_cal_id {
        None => diesel::insert_into(calendars::calendars)
            .values(calendars::googleid.eq(&calendar_id))
            .returning(calendars::id)
            .get_result::<i32>(&mut db)
            .await
            .expect("Unable to insert calendar into database"),
        Some(id) => id,
    };

    // Inserting guild_calendar into db
    diesel::insert_into(guilds_calendars::guilds_calendars)
        .values((
            guilds_calendars::guild_id.eq(guild_id),
            guilds_calendars::calendar_id.eq(db_cal_id),
            guilds_calendars::channelid.eq(channel.id.get().to_string()),
        ))
        .execute(&mut db)
        .await
        .expect("Unable to insert guild_calendar into database");

    ctx.send(
        poise::CreateReply::default()
            .content("Successfully added")
            .reply(true)
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

#[poise::command(slash_command, guild_only, category = "Google calendar")]
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

    let mut db = ctx.data().db.get().await.unwrap();

    let res = guilds_calendars::guilds_calendars
        .filter(guilds_calendars::channelid.eq(channel.id.get().to_string()))
        .select((guilds_calendars::calendar_id, guilds_calendars::messageid))
        .first::<(i32, Option<String>)>(&mut db)
        .await;

    if res.is_err() {
        let _ = ctx.reply("This channel doesn't have a calendar").await?;
        return Ok(());
    }
    let res = res.unwrap();

    let calendar_id = res.0;

    // Deleting the calendar message
    if let Some(message_id) = res.1 {
        let message_id = serenity::MessageId::new(message_id.parse::<u64>().unwrap());

        let res = channel.delete_messages(&ctx.http(), vec![message_id]).await;

        if res.is_err() {
            warn!("Unable to delete message (maybe the bot is missing the MANAGE_MESSAGE permission?): {:?}", res);
        }
    }

    // Remove the calendar from the database
    let del = diesel::delete(
        guilds_calendars::guilds_calendars
            .filter(guilds_calendars::channelid.eq(channel.id.get().to_string())),
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
        .filter(guilds_calendars::calendar_id.eq(calendar_id.clone()))
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
    .await
    .unwrap();

    Ok(())
}
