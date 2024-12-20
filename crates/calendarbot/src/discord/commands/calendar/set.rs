use crate::schema::guilds_calendars::dsl as guilds_calendars;
use crate::types::TimezoneChoices;
use crate::ApplicationContext;
use anyhow::Result;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use log::trace;

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

    trace!(
        "Changing timezone from {:?} to {:?} for channel {:?}",
        res.unwrap(),
        timezone,
        channel.id.get()
    );
    let res = diesel::update(
        guilds_calendars::guilds_calendars
            .filter(guilds_calendars::channelId.eq(channel.id.get().to_string())),
    )
    .set((
        guilds_calendars::timezone.eq(timezone),
        guilds_calendars::forceUpdate.eq(true),
    ))
    .execute(&mut db)
    .await;

    match res {
        Ok(_) => {
            let _ = ctx.reply("Timezone updated").await?;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
