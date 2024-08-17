use crate::calendar::GCalendar;
use crate::calendar::UpdateCalendarEvent;
use crate::commands;
use crate::types;
use anyhow::{Error, Result};
use google_calendar3::api::Event;
use google_calendar3::chrono::{Datelike, NaiveDate, Utc};
use log::{debug, error, info};
use poise::serenity_prelude as serenity;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

async fn on_error(error: poise::FrameworkError<'_, types::Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,)
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

#[derive(Clone)]
struct LocalCache {
    cache: Arc<serenity::Cache>,
    client: Arc<serenity::Http>,
}
impl LocalCache {
    fn new(client: Arc<serenity::Http>) -> Self {
        Self {
            cache: Arc::new(serenity::Cache::default()),
            client,
        }
    }
}

impl serenity::CacheHttp for LocalCache {
    fn http(&self) -> &serenity::Http {
        &self.client
    }

    fn cache(&self) -> Option<&Arc<serenity::Cache>> {
        Some(&self.cache)
    }
}

pub struct Discord {
    token: String,
    intents: serenity::GatewayIntents,
    cache: Arc<Mutex<Option<LocalCache>>>,
}

impl Discord {
    pub fn new(token: String, gateway_intents: serenity::GatewayIntents) -> Self {
        Self {
            token,
            intents: gateway_intents,
            cache: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn init(
        &mut self,
        calendar_rx: mpsc::Receiver<UpdateCalendarEvent>,
        data: types::Data,
    ) -> serenity::Client {
        let cache_clone = self.cache.clone();

        let framework = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                commands: vec![
                    commands::utilities::help(),
                    commands::utilities::uptime(),
                    commands::utilities::age(),
                ],
                on_error: |error| Box::pin(async move { on_error(error).await }),
                pre_command: |ctx| {
                    Box::pin(async move {
                        let channel_name = &ctx
                            .channel_id()
                            .name(&ctx)
                            .await
                            .unwrap_or_else(|_| "<unknown>".to_owned());
                        let author = &ctx.author().name;

                        info!(
                            "{} in {} used slash command '{}'",
                            author,
                            channel_name,
                            &ctx.invoked_command_name()
                        );
                    })
                },
                post_command: |ctx| {
                    Box::pin(async move {
                        debug!(
                            "{} executed command \"{}\"",
                            ctx.author().tag(),
                            ctx.command().qualified_name
                        );
                    })
                },
                ..Default::default()
            })
            .setup(move |ctx, ready, framework| {
                Box::pin(async move {
                    cache_clone
                        .lock()
                        .await
                        .replace(LocalCache::new(ctx.http.clone()));

                    Discord::new_data_thread(calendar_rx, cache_clone.clone());

                    debug!("Registering commands..");
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                    debug!("Setting activity text...");
                    ctx.set_activity(Some(serenity::ActivityData::listening("/help")));

                    info!("{} is ready !", ready.user.name);

                    Ok(data)
                })
            })
            .build();

        let it = self.intents.clone();
        let client = serenity::Client::builder(self.token.clone(), it)
            .framework(framework)
            .await
            .expect("Failed to create client");

        return client;
    }

    fn new_data_thread(
        mut calendar_rx: mpsc::Receiver<UpdateCalendarEvent>,
        cache: Arc<Mutex<Option<LocalCache>>>,
    ) {
        tokio::spawn(async move {
            let mut events_cache: BTreeMap<String, Vec<Event>> = BTreeMap::new();
            let mut message_cache: BTreeMap<String, u64> = BTreeMap::new();
            let channel = serenity::ChannelId::new(1102198299093647470);

            while let Some(event) = calendar_rx.recv().await {
                debug!("Received event for calendar {}", event.calendar_id);

                let events = events_cache
                    .entry(event.calendar_id.clone())
                    .or_insert(Vec::new());
                let matching = events
                    .iter()
                    .zip(event.new_events.iter())
                    .filter(|&(a, b)| GCalendar::compare_event(a, b))
                    .count();

                let do_match = matching == event.new_events.len() && matching == events.len();
                debug!(
                    "matching: {} == {} && {} == {}",
                    matching,
                    event.new_events.len(),
                    matching,
                    events.len()
                );

                if do_match && !events.is_empty() {
                    debug!("No new events");
                    continue;
                }

                let cache = cache.as_ref().lock().await.clone().unwrap();
                let embed = match Discord::event_to_embed(event.new_events.clone()) {
                    Ok(v) => v,
                    Err(_) => serenity::CreateEmbed::new().title("Events").field(
                        "Error",
                        "Error getting events",
                        true,
                    ),
                };

                let mut create_new_message = false;
                if let Entry::Occupied(o) = message_cache.entry(event.calendar_id.clone()) {
                    let message_id = serenity::MessageId::new(o.get().clone());

                    let err = cache
                        .client
                        .edit_message(
                            channel,
                            message_id,
                            &serenity::EditMessage::new().add_embed(embed.clone()),
                            Vec::new(),
                        )
                        .await;

                    match err {
                        Err(e) => {
                            error!("Failed to edit message ({}): {}", message_id.clone(), e);
                            create_new_message = true;
                        }
                        _ => (),
                    };
                } else {
                    create_new_message = true;
                }

                if create_new_message {
                    let res = channel
                        .send_message(cache, serenity::CreateMessage::new().add_embed(embed))
                        .await
                        .unwrap();

                    message_cache
                        .entry(event.calendar_id.clone())
                        .or_insert(res.id.get());
                }

                if !events.is_empty() {
                    events.clear();
                }
                events.extend(event.new_events.clone());
            }
        });
    }

    fn event_to_embed(events: Vec<Event>) -> Result<serenity::CreateEmbed> {
        let mut sorted: BTreeMap<(NaiveDate, NaiveDate), Vec<Event>> = BTreeMap::new();
        let mut fields: Vec<(String, String, bool)> = vec![];
        for ele in events {
            let ele_clone = ele.clone();
            let start_date = ele_clone.start.unwrap().date_time.unwrap();
            let end_date = ele.clone().end.unwrap().date_time.unwrap();
            sorted
                .entry((start_date.date_naive(), end_date.date_naive()))
                .or_insert(Vec::new())
                .push(ele);
        }

        for ((start_date, end_date), events) in sorted.iter() {
            let mut field = String::new();
            for event in events {
                field.push_str(&format!(
                    "```{} - {} | {}```\n",
                    event
                        .start
                        .clone()
                        .unwrap()
                        .date_time
                        .unwrap()
                        .format("%H:%M"),
                    event
                        .end
                        .clone()
                        .unwrap()
                        .date_time
                        .unwrap()
                        .format("%H:%M"),
                    event.summary.clone().unwrap()
                ));
            }
            let mut format = String::from("**%A** - %e %B");
            if start_date.year() != end_date.year() || start_date.year() != Utc::now().year() {
                format = String::from("%F");
            }

            let mut key = start_date.format(&format.clone()).to_string();

            if start_date.format("%F").to_string() != end_date.format("%F").to_string() {
                key.push_str(
                    &end_date
                        .format(format!(" // {}", format.clone()).as_str())
                        .to_string(),
                );
            }

            fields.push((key, field, false));
        }

        Ok(serenity::CreateEmbed::new().title("Events").fields(fields))
    }
}
