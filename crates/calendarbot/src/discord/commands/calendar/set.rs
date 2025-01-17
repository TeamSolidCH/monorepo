use crate::models::GuildCalendar;
use crate::schema::guilds_calendars as guilds_calendars_all;
use crate::schema::guilds_calendars::dsl as guilds_calendars;
use crate::types::TimezoneChoices;
use crate::ApplicationContext;
use anyhow::{anyhow, Result};
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
        .await?;

    let mut values = default_values;

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

    match diesel::update(
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
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

async fn get_settings<CSelect, T>(
    connection: &mut diesel_async::AsyncPgConnection,
    ctx: &ApplicationContext<'_>,
    channel_id: String,
    column: CSelect,
) -> Result<T>
where
    CSelect: diesel::Expression
        + diesel::SelectableExpression<guilds_calendars_all::table>
        + diesel::AppearsOnTable<guilds_calendars_all::table>
        + diesel::expression::ValidGrouping<()>
        + Send
        + 'static
        + diesel::query_builder::QueryId
        + diesel::query_builder::QueryFragment<diesel::pg::Pg>,
    diesel::pg::Pg: diesel::sql_types::HasSqlType<CSelect::SqlType>,
    CSelect::SqlType: diesel::sql_types::SingleValue, // Ensures column returns a single value
    T: diesel::Queryable<CSelect::SqlType, diesel::pg::Pg> + Send + 'static, // Ensure T matches the SQL type
{
    // Perform the query
    let result = guilds_calendars_all::table
        .filter(guilds_calendars::channelId.eq(channel_id))
        .select(column)
        .first::<T>(connection)
        .await;

    // Handle the result and reply to the context if needed
    match result {
        Ok(value) => Ok(value),
        Err(diesel::result::Error::NotFound) => {
            ctx.reply("This channel doesn't have a calendar")
                .await
                .map_err(|e| anyhow!(e))?; // Handle the reply error
            Err(diesel::result::Error::NotFound.into())
        }
        Err(e) => Err(e.into()),
    }
}

#[poise::command(
    slash_command,
    guild_only,
    category = "Google calendar",
    subcommands("timezone", "nb_displayed_days"),
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
    let mut db = ctx.data().db.get().await?;

    let old_timezone: String = get_settings(
        &mut db,
        &ctx,
        channel.id.get().to_string(),
        guilds_calendars::timezone,
    )
    .await?;
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

#[poise::command(slash_command, guild_only, category = "Google calendar")]
pub async fn nb_displayed_days(
    ctx: ApplicationContext<'_>,
    #[description = "Days (defaults to UTC)"] days: u8,
) -> Result<()> {
    let channel = ctx.guild_channel().await;
    let channel = channel.ok_or_else(|| anyhow!("Channel not found"))?;
    let mut db = ctx.data().db.get().await?;

    let res: i32 = get_settings(
        &mut db,
        &ctx,
        channel.id.get().to_string(),
        guilds_calendars::nbDisplayedDays,
    )
    .await?;

    let old_nb_displayed_days =
        u8::try_from(res).map_err(|_| anyhow!("Number of displayed days is too big"))?;

    if old_nb_displayed_days == days {
        let _ = ctx
            .reply("Number of displayed days already set to this value")
            .await?;
        return Ok(());
    }

    trace!(
        "Changing number of displayed days from {:?} to {:?} for channel {:?}",
        old_nb_displayed_days,
        days,
        channel.id.get()
    );

    update_settings(
        &mut db,
        channel.id.get(),
        None,
        Some(days as i32),
        None,
        None,
    )
    .await
    .map_err(|e| anyhow!(e))?;
    let _ = ctx.reply("Number of displayed days updated").await?;
    Ok(())
}
