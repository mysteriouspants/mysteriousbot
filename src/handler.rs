use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serenity::{
    all::CreateCommand,
    client::{Context, EventHandler},
    model::{
        application::{Command, Interaction},
        channel::Message,
        gateway::Ready,
    },
};

use crate::{config::Config, counter::CounterFactory, emojicache::EmojiCache};

pub struct Handler {
    pub config: Config,
    pub emoji_cache: EmojiCache,
    pub pool: Pool<SqliteConnectionManager>,
    pub counter_factory: CounterFactory,
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let command = match interaction {
            Interaction::Command(command) => command,
            _ => return, // not a something we know how to handle
        };
        let guild_id = match command.guild_id {
            Some(guild_id) => guild_id,
            None => return, // bail from the whole interaction
        };
        let guild_config = match self.config.guilds.get(&guild_id.get()) {
            Some(guild_config) => guild_config,
            None => return, // not a guild we have config for, skip
        };

        if let Some(c) = guild_config
            .commands
            .iter()
            .find(|c| c.alias == command.data.name.as_str())
        {
            c.handle(&command, ctx, &self.counter_factory).await;
        }
    }

    async fn message(&self, context: Context, message: Message) {
        let guild_id = match message.guild_id {
            Some(guild_id) => guild_id,
            None => return, // bail from the whole thing
        };
        let guild_config = match self.config.guilds.get(&guild_id.get()) {
            Some(guild_config) => guild_config,
            None => return, // not a guild we have config for, skip
        };

        for autoresponder in &guild_config.autoresponders {
            autoresponder
                .handle(
                    &self.emoji_cache,
                    &self.counter_factory,
                    &context,
                    &message,
                    &guild_id,
                )
                .await;
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        log::info!("{} is connected!", ready.user.name);

        let _ = Command::set_global_commands(&ctx.http, vec![]).await;

        let guilds = ready.guilds.iter().map(|offline_guild| offline_guild.id);

        for guild in guilds {
            let guild_id = guild.get();
            if let Some(guild_config) = self.config.guilds.get(&guild_id) {
                log::info!("Setting application commands on Guild ID {}", guild_id);

                let commands = guild_config
                    .commands
                    .iter()
                    .map(|command_config| {
                        CreateCommand::new(&command_config.alias)
                            .description(&command_config.description)
                    })
                    .collect::<Vec<_>>();

                let r = guild.set_commands(&ctx.http, commands).await;

                match r {
                    Ok(_) => log::info!("Application commands for guild {} set", guild_id),
                    Err(e) => log::error!(
                        "Failed setting commands for guild {} with error {:?}",
                        guild_id,
                        e
                    ),
                }
            } else {
                log::info!(
                    "Connected to guild {} which has no associated config",
                    guild_id
                );
            }
        }
    }
}
