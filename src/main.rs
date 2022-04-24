use configparser::{AutoTrigger, ChannelFilter, Config, GuildConfig};
use dotenv::dotenv;
use emojicache::EmojiCache;
use serenity::{
    client::Client,
    model::{
        channel::Message,
        gateway::{GatewayIntents, Ready},
        id::GuildId,
        interactions::{
            application_command::ApplicationCommand, Interaction, InteractionResponseType,
        },
    },
    prelude::{Context, EventHandler},
};
use std::{env, fs::read_to_string, str::FromStr};

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
            if !matches_channel_filter(&autoemojiconfig.channel_filter, message)
                || !matches_autotrigger(&autoemojiconfig.on, ctx, message)
            {
                continue; // we shouldn't even bother
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

    async fn handle_autoresponder(
        &self,
        ctx: &Context,
        message: &Message,
        _guild_id: &GuildId,
        guild_config: &GuildConfig,
    ) {
        for autoresponder in &guild_config.autoresponders {
            if !matches_channel_filter(&autoresponder.channel_filter, message)
                || !matches_autotrigger(&autoresponder.on, ctx, message)
            {
                continue; // we shouldn't even bother
            }

            if let Err(why) = message.reply(ctx, &autoresponder.message).await {
                log::error!("Failed to autoreply to message with reason {:?}", why);
            }
        }
    }
}

fn matches_autotrigger(trigger: &AutoTrigger, ctx: &Context, message: &Message) -> bool {
    match &trigger {
        AutoTrigger::Mention(user_id) => message.mentions_user_id(*user_id),
        AutoTrigger::Message(user_id) => message.author.id == *user_id,
        AutoTrigger::Match(regex) => regex.is_match(&message.content_safe(ctx)),
    }
}

fn matches_channel_filter(channel_filter: &ChannelFilter, message: &Message) -> bool {
    if let Some(ignore_channels) = &channel_filter.ignore_channels {
        if ignore_channels.contains(&message.channel_id.0) {
            return false; // we should ignore this triggering
        }
    }

    if let Some(only_in_channels) = &channel_filter.only_in_channels {
        if !only_in_channels.contains(&message.channel_id.0) {
            return false; // we should ignore this triggering
        }
    }

    true
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

        self.handle_autoemoji(&ctx, &message, &guild_id, &guild_config)
            .await;
        self.handle_autoresponder(&ctx, &message, &guild_id, &guild_config)
            .await;
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
    let mut client = Client::builder(
        token,
        GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::GUILD_MESSAGE_REACTIONS
            | GatewayIntents::MESSAGE_CONTENT,
    )
    .application_id(application_id)
    .event_handler(handler)
    .await
    .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
