/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */

use crate::events::CalendarCommands;
use crate::schema::calendars::dsl as calendars;
use crate::schema::guilds::dsl as guilds;
use crate::schema::guilds_calendars::dsl as guilds_calendars;
use crate::types::TimezoneChoices;
use crate::ApplicationContext;
use anyhow::Result;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use poise::serenity_prelude as serenity;
use tokio::sync::oneshot;

#[poise::command(slash_command, guild_only, category = "Google calendar")]
pub async fn new(
    ctx: ApplicationContext<'_>,
    #[description = "Google Calendar ID"] calendar_id: String,
    #[channel_types("Text")]
    #[description = "Channel (defaults to the current channel)"]
    channel: Option<serenity::GuildChannel>,
    #[description = "Timezone (defaults to UTC)"] timezone: TimezoneChoices,
    #[description = "Number of days to display (defaults to 8)"] num_displayed_days: Option<u8>,
    #[description = "Skip weekends (default to false)"] skip_weekend: Option<bool>,
    #[description = "Show days if there are no events (defaults to false)"]
    show_if_no_events: Option<bool>,
) -> Result<()> {
    let channel = match channel {
        Some(c) => c,
        None => ctx.guild_channel().await.unwrap(),
    };

    let mut db = ctx.data().db.get().await?;

    // Checking if the channel as calendar
    let res = guilds_calendars::guilds_calendars
        .filter(guilds_calendars::channelId.eq(channel.id.get().to_string()))
        .select(guilds_calendars::channelId)
        .first::<String>(&mut db)
        .await;

    if res.is_ok() {
        let _ = ctx.reply("This channel already has a calendar").await?;
        return Ok(());
    }

    // Checking if the calendar is already present in db
    let res = calendars::calendars
        .filter(calendars::googleId.eq(&calendar_id))
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

        ctx.defer().await?;

        ctx.data().gcalendar_tx.clone().send(cmd).await?;

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
        .filter(guilds::discordId.eq(guild_id.clone()))
        .select(guilds::id)
        .first::<i32>(&mut db)
        .await;

    let guild_id = match res {
        Ok(id) => id,
        Err(_) => {
            diesel::insert_into(guilds::guilds)
                .values(guilds::discordId.eq(guild_id))
                .returning(guilds::id)
                .get_result::<i32>(&mut db)
                .await?
        }
    };

    // Inserting calendar into db
    let db_cal_id = match db_cal_id {
        None => {
            diesel::insert_into(calendars::calendars)
                .values(calendars::googleId.eq(&calendar_id))
                .returning(calendars::id)
                .get_result::<i32>(&mut db)
                .await?
        }
        Some(id) => id,
    };

    // Inserting guild_calendar into db
    diesel::insert_into(guilds_calendars::guilds_calendars)
        .values((
            guilds_calendars::guild_id.eq(guild_id),
            guilds_calendars::calendar_id.eq(db_cal_id),
            guilds_calendars::channelId.eq(channel.id.get().to_string()),
            guilds_calendars::timezone.eq(timezone.to_normalized_string()),
            guilds_calendars::nbDisplayedDays.eq(num_displayed_days.unwrap_or(7) as i32),
            guilds_calendars::skipWeekend.eq(skip_weekend.unwrap_or(false)),
            guilds_calendars::skipEmptyDays.eq(!show_if_no_events.unwrap_or(true)),
        ))
        .execute(&mut db)
        .await?;

    ctx.send(
        poise::CreateReply::default()
            .content("Successfully added")
            .reply(true)
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
