use crate::models::GuildCalendar;
use crate::schema::guilds_calendars::dsl as guilds_calendars;
use crate::types::TimezoneChoices;
use crate::ApplicationContext;
use anyhow::Result;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use log::trace;

async fn update_settings(
    db: &mut AsyncPgConnection,
    channel_id: u64,
    timezone: Option<String>,
    nb_displayed_days: Option<i32>,
    skip_weekends: Option<bool>,
    skip_empty_days: Option<bool>,
) -> Result<()> {
    let default_values = guilds_calendars::guilds_calendars
        .filter(guilds_calendars::channelId.eq(channel_id.to_string()))
        .first::<GuildCalendar>(db)
        .await;

    if let Err(e) = default_values {
        return Err(e.into());
    }

    let mut values = default_values.unwrap();

    if let Some(timezone) = timezone {
        values.timezone = timezone;
    }

    if let Some(nb_displayed_days) = nb_displayed_days {
        values.nbDisplayedDays = nb_displayed_days;
    }

    if let Some(skip_weekends) = skip_weekends {
        values.skipWeekend = skip_weekends;
    }

    if let Some(skip_empty_days) = skip_empty_days {
        values.skipEmptyDays = skip_empty_days;
    }

    let res = diesel::update(
        guilds_calendars::guilds_calendars
            .filter(guilds_calendars::channelId.eq(channel_id.to_string())),
    )
    .set((
        guilds_calendars::timezone.eq(&values.timezone),
        guilds_calendars::nbDisplayedDays.eq(values.nbDisplayedDays),
        guilds_calendars::skipWeekend.eq(values.skipWeekend),
        guilds_calendars::skipEmptyDays.eq(values.skipEmptyDays),
        guilds_calendars::forceUpdate.eq(true),
    ))
    .execute(db)
    .await;

    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

#[poise::command(
    slash_command,
    guild_only,
    category = "Google calendar",
    subcommands("timezone"),
    subcommand_required
)]
pub async fn set(_: ApplicationContext<'_>) -> Result<()> {
    Ok(())
}

#[poise::command(slash_command, guild_only, category = "Google calendar")]
pub async fn timezone(
    ctx: ApplicationContext<'_>,
    #[description = "Timezone (defaults to UTC)"] timezone: TimezoneChoices,
) -> Result<()> {
    let channel = ctx.guild_channel().await.unwrap();
    let timezone = timezone.to_normalized_string();
    let mut db = ctx.data().db.get().await.unwrap();

    let res = guilds_calendars::guilds_calendars
        .filter(guilds_calendars::channelId.eq(channel.id.get().to_string()))
        .select(guilds_calendars::timezone)
        .first::<String>(&mut db)
        .await;

    if let Err(e) = res {
        match e {
            diesel::result::Error::NotFound => {
                let _ = ctx.reply("This channel doesn't have a calendar").await?;
                return Ok(());
            }
            _ => return Err(e.into()),
        }
    }
    let old_timezone = res.unwrap();

    if old_timezone == timezone {
        let _ = ctx.reply("Timezone already set to this value").await?;
        return Ok(());
    }

    trace!(
        "Changing timezone from {:?} to {:?} for channel {:?}",
        old_timezone,
        timezone,
        channel.id.get()
    );

    let res = update_settings(&mut db, channel.id.get(), Some(timezone), None, None, None).await;

    match res {
        Ok(_) => {
            let _ = ctx.reply("Timezone updated").await?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}