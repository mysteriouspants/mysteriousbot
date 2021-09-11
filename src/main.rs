use configparser::Config;
use dotenv::dotenv;
use emojicache::EmojiCache;
use serenity::client::Client;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::guild::GuildStatus;
use serenity::model::interactions::application_command::ApplicationCommand;
use serenity::model::interactions::{Interaction, InteractionResponseType};
use serenity::prelude::{Context, EventHandler};
use std::env;
use std::fs::read_to_string;

mod configparser;
mod emojicache;

struct Handler {
    config: Config,
    emoji_cache: EmojiCache,
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "ping" => "Hey, I'm alive!".to_string(),
                "bus" => {
                    ":musical_score::musical_note: Another one rides the bus! :notes:".to_string()
                }
                "wednesday" => "https://www.idevgames.com/its_wednesday_my_doods.mp4".to_string(),
                "friday" => "https://www.idevgames.com/its_friday_then.mp4".to_string(),
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }
    async fn message(&self, ctx: Context, message: Message) {
        // P33ky 261559920485335040
        // Necro 249051133987913728
        if message.mentions_user_id(249051133987913728) {
            // P33ky
            let twemojis = ["swedishfish"];
            for twemoji in twemojis {
                if let Some(guild_id) = message.guild_id {
                    if let Ok(Some(emoji)) =
                        self.emoji_cache.get_emoji(&ctx, &guild_id, twemoji).await
                    {
                        message.react(&ctx, emoji).await;
                    }
                }
            }
        }
    }
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        for guild in ready.guilds {
            match guild {
                GuildStatus::OnlineGuild(g) => {
                    
                },
                GuildStatus::OnlinePartialGuild(pg) => {

                },
                GuildStatus::Offline(gu) => {

                },
                _ => { /* do nothing */ }
            }
        }

        let commands = ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command.name("ping").description("A ping command")
                })
                .create_application_command(|command| {
                    command.name("bus").description("another one rides the bus")
                })
                .create_application_command(|command| {
                    command
                        .name("wednesday")
                        .description("it is wednesday my doods")
                })
                .create_application_command(|command| {
                    command
                        .name("friday")
                        .description("mufasa, it's friday then")
                })
        })
        .await;

        println!(
            "I now have the following global slash commands: {:#?}",
            commands
        );
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok(); // enable use of .env files
    let application_id: u64 = env::var("DISCORD_APPLICATION_ID")
        .expect("DISCORD_APPLICATION_ID environment variable is unset, exiting")
        .parse()
        .expect("DISCORD_APPLICATION_ID is not an integer, exiting");
    let token =
        &env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN environment variable is unset, exiting");
    let config_file =
        &env::var("MYSTERIOUSBOT_CONFIG").unwrap_or("./config/mysteriousbot.toml".to_owned());
    let handler = Handler {
        config: toml::from_str(
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
