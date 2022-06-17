use serenity::{client::{Context, EventHandler}, model::{channel::Message, interactions::{Interaction, application_command::ApplicationCommand}, gateway::Ready}};

use crate::{config::Config, emojicache::EmojiCache};

pub struct Handler {
    pub config: Config,
    pub emoji_cache: EmojiCache,
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let command = match interaction {
            Interaction::ApplicationCommand(command) => command,
            _ => return, // not a something we know how to handle
        };
        let guild_id = match command.guild_id {
            Some(guild_id) => guild_id,
            None => return, // bail from the whole interaction
        };
        let guild_config = match self.config.guilds.get(&guild_id.0) {
            Some(guild_config) => guild_config,
            None => return, // not a guild we have config for, skip
        };

        if let Some(c) = guild_config.commands.iter().find(|c| c.alias == command.data.name.as_str()) {
            c.handle(&command, ctx).await;
        }
    }

    async fn message(&self, context: Context, message: Message) {
        let guild_id = match message.guild_id {
            Some(guild_id) => guild_id,
            None => return, // bail from the whole thing
        };
        let guild_config = match self.config.guilds.get(&guild_id.0) {
            Some(guild_config) => guild_config,
            None => return, // not a guild we have config for, skip
        };

        for autoresponder in &guild_config.autoresponders {
            autoresponder.handle(&self.emoji_cache, &context, &message, &guild_id).await;
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        log::info!("{} is connected!", ready.user.name);

        let _ = ApplicationCommand::set_global_application_commands(&ctx.http, |commands| commands)
            .await;

        let guilds = ready.guilds.iter().map(|offline_guild| offline_guild.id);

        for guild in guilds {
            let guild_id = guild.0;
            if let Some(guild_config) = self.config.guilds.get(&guild_id) {
                log::info!("Setting application commands on Guild ID {}", guild_id);
                let r = guild
                    .set_application_commands(&ctx.http, |mut commands| {
                        for command_config in &guild_config.commands {
                            log::info!(
                                "Adding command {} to guild {}",
                                command_config.alias,
                                guild_id
                            );
                            commands = commands.create_application_command(|command| {
                                command
                                    .name(&command_config.alias)
                                    .description(&command_config.description)
                            });
                        }

                        commands
                    })
                    .await;

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
