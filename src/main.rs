use configparser::{AutoEmojiTrigger, Config, GuildConfig};
use dotenv::dotenv;
use emojicache::EmojiCache;
use serenity::client::Client;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::guild::GuildStatus;
use serenity::model::id::GuildId;
use serenity::model::interactions::application_command::ApplicationCommand;
use serenity::model::interactions::{Interaction, InteractionResponseType};
use serenity::prelude::{Context, EventHandler};
use std::env;
use std::fs::read_to_string;
use std::str::FromStr;

mod configparser;
mod emojicache;

struct Handler {
    config: Config,
    emoji_cache: EmojiCache,
}

impl Handler {
    async fn handle_autoemoji(
        &self,
        ctx: &Context,
        message: &Message,
        guild_id: &GuildId,
        guild_config: &GuildConfig,
    ) {
        for autoemojiconfig in &guild_config.autoemojis {
            let should_react = match autoemojiconfig.on {
                AutoEmojiTrigger::Mention(user_id) => message.mentions_user_id(user_id),
                AutoEmojiTrigger::Message(user_id) => message.author.id == user_id,
            };

            if !should_react {
                continue; // we shouldn't even bother
            }

            if let Some(ignore_channels) = &autoemojiconfig.ignore_channels {
                if ignore_channels.contains(&message.channel_id.0) {
                    continue; // we should ignore this triggering
                }
            }

            if let Some(only_in_channels) = &autoemojiconfig.only_in_channels {
                if !only_in_channels.contains(&message.channel_id.0) {
                    continue; // we should ignore this triggering
                }
            }

            for twemoji in &autoemojiconfig.twemojis {
                if let Ok(Some(emoji)) = self.emoji_cache.get_emoji(&ctx, &guild_id, twemoji).await
                {
                    if let Err(why) = message.react(&ctx, emoji).await {
                        log::error!("Failed to react to message with reason {:?}", why);
                    }
                } else {
                    log::error!("Unknown twemoji {}", twemoji);
                }
            }
        }
    }
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

        // slash command handling
        let slash_responder = guild_config
            .slash_responders
            .iter()
            .find(|slash_responder| slash_responder.command == command.data.name.as_str());

        if let Some(slash_responder) = slash_responder {
            let content = &slash_responder.response;

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                log::error!("Cannot respond to slash command: {:?}", why);
            }
        }
    }

    async fn message(&self, ctx: Context, message: Message) {
        let guild_id = match message.guild_id {
            Some(guild_id) => guild_id,
            None => return, // bail from the whole thing
        };
        let guild_config = match self.config.guilds.get(&guild_id.0) {
            Some(guild_config) => guild_config,
            None => return, // not a guild we have config for, skip
        };

        // autoemojis
        self.handle_autoemoji(&ctx, &message, &guild_id, &guild_config)
            .await;
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        log::info!("{} is connected!", ready.user.name);

        let _ = ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
            commands
        }).await;

        let guilds = ready
            .guilds
            .iter()
            .filter_map(|guild| match guild {
                GuildStatus::OnlineGuild(g) => Some(g.id),
                GuildStatus::OnlinePartialGuild(pg) => Some(pg.id),
                GuildStatus::Offline(gu) => Some(gu.id),
                _ => None,
            });

        for guild in guilds {
            let guild_id = guild.0;
            if let Some(guild_config) = self.config.guilds.get(&guild_id) {
                log::info!("Setting application commands on Guild ID {}", guild_id);
                let r = guild
                    .set_application_commands(&ctx.http, |mut commands| {
                        for slash_responder in &guild_config.slash_responders {
                            log::info!(
                                "Adding command {} to guild {}",
                                slash_responder.command,
                                guild_id
                            );
                            commands = commands.create_application_command(|command| {
                                command
                                    .name(&slash_responder.command)
                                    .description(&slash_responder.description)
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

#[tokio::main]
async fn main() {
    dotenv().ok(); // enable use of .env files
    env_logger::init();
    let application_id: u64 = env::var("DISCORD_APPLICATION_ID")
        .expect("DISCORD_APPLICATION_ID environment variable is unset, exiting")
        .parse()
        .expect("DISCORD_APPLICATION_ID is not an integer, exiting");
    let token =
        &env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN environment variable is unset, exiting");
    let config_file =
        &env::var("MYSTERIOUSBOT_CONFIG").unwrap_or("./config/mysteriousbot.toml".to_owned());
    let handler = Handler {
        config: Config::from_str(
            &read_to_string(config_file)
                .expect(&format!("Config file at {} could not be read", config_file)),
        )
        .expect("Config could not be parsed"),
        emoji_cache: EmojiCache::new(),
    };
    let mut client = Client::builder(token)
        .application_id(application_id)
        .event_handler(handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
