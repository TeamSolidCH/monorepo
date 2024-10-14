use anyhow::Error;
use log::{debug, error, info};
use poise::serenity_prelude as serenity;
use tokio::sync::mpsc;

use crate::discord::LocalCache;
use crate::{discord::commands, discord::Discord, events::UpdateCalendarEvent, types};

async fn on_error(error: poise::FrameworkError<'_, types::Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!("Error in command `{}`: {:?}", ctx.command().name, error,)
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {}", e)
            }
        }
    }
}

impl Discord {
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
                    commands::calendar::new(),
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

                    Discord::calendar_events_thread(
                        calendar_rx,
                        cache_clone.clone(),
                        data.db.clone(),
                    );

                    debug!("Registering commands..");
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                    debug!("Setting activity text...");
                    ctx.set_activity(Some(serenity::ActivityData::listening("/help")));

                    info!("{} is ready !", ready.user.name);

                    Ok(data)
                })
            })
            .build();

        let it = self.intents;
        serenity::Client::builder(self.token.clone(), it)
            .framework(framework)
            .await
            .expect("Failed to create client")
    }
}
