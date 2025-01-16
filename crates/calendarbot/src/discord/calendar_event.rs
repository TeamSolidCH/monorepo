/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */

use crate::discord::{Discord, LocalCache};
use crate::UpdateCalendarEvent;

use crate::schema::calendars::dsl as calendars;
use crate::schema::guilds_calendars::dsl as guilds_calendars;
use crate::types::CalendarEvent;
use anyhow::Result;
use diesel::prelude::*;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;
use log::{debug, error};
use poise::serenity_prelude as serenity;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

impl Discord {
    /// Send or edit a message in a channel
    async fn send_or_edit_message(
        channel_id: u64,
        message_id: Option<u64>,
        embed: serenity::CreateEmbed,
        cache: LocalCache,
    ) -> Result<serenity::MessageId> {
        let channel = serenity::ChannelId::new(channel_id);
        if let Some(message_id) = message_id {
            let msg_id = serenity::MessageId::new(message_id);
            debug!("Trying to edit message ({})", msg_id.get());
            let result = cache
                .client
                .edit_message(
                    channel,
                    msg_id,
                    &serenity::EditMessage::new().add_embed(embed.clone()),
                    Vec::new(),
                )
                .await;

            if let Err(e) = result {
                error!("Failed to edit message ({}): {}", message_id.clone(), e);
            } else {
                return Ok(msg_id);
            }
        }

        debug!("Send new message");
        match channel
            .send_message(cache, serenity::CreateMessage::new().add_embed(embed))
            .await
        {
            Err(e) => Err(e.into()),
            Ok(res) => Ok(res.id),
        }
    }

    pub(crate) fn calendar_events_thread(
        mut calendar_rx: mpsc::Receiver<UpdateCalendarEvent>,
        cache: Arc<Mutex<Option<LocalCache>>>,
        db: Pool<AsyncPgConnection>,
    ) {
        //TODO: Find a better way to handle new events (maybe a threadpool)
        tokio::spawn(async move {
            let db = db.get().await;
            if let Err(e) = db {
                error!("Unable to get db connection from poolmanager: {:?}", e);
                return;
            }

            let mut db = db.unwrap();

            while let Some(event) = calendar_rx.recv().await {
                debug!("Received event for calendar {}", event.calendar_id);

                let cache = cache.as_ref().lock().await.clone().unwrap();

                let embed = CalendarEvent::to_embed(
                    event.new_events.clone(),
                    event.calendar_options.clone(),
                );

                for (channel_id, message_id) in event.discord_channel_and_message_ids {
                    debug!(target: &channel_id.to_string(), "Handling new_events with message_id: {:?}", message_id);
                    let result = Discord::send_or_edit_message(
                        channel_id,
                        message_id,
                        embed.clone(),
                        cache.clone(),
                    )
                    .await;

                    match result {
                        Err(e) => {
                            error!("Failed to send or edit message: {}, calendar: {}, channel_id: {:?}, message_id: {:?}", e, event.calendar_id, channel_id, message_id);
                        }
                        Ok(msg_id) => if Some(msg_id.get()) != message_id {
                            let db_cal_id = calendars::calendars
                                .filter(calendars::googleId.eq(event.calendar_id.clone()))
                                .select(calendars::id)
                                .first::<i32>(&mut db)
                                .await
                                .expect("Unable to get calendar id from database");

                            diesel::update(guilds_calendars::guilds_calendars)
                                .filter(
                                    guilds_calendars::channelId
                                        .eq(channel_id.to_string())
                                        .and(guilds_calendars::calendar_id.eq(db_cal_id)),
                                )
                                .set(guilds_calendars::messageId.eq(msg_id.get().to_string()))
                                .execute(&mut db)
                                .await
                                .expect("Unable to update message id in database");
                        },
                    };
                }
            }
        });
    }
}
