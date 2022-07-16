use crate::handler::Handler;
use counter::CounterFactory;
use dotenv::dotenv;
use emojicache::EmojiCache;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serenity::{client::Client, model::gateway::GatewayIntents};
use std::{env, fs::read_to_string};

mod autoresponder;
mod command;
mod config;
mod counter;
mod emojicache;
mod handler;

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
        &env::var("MYSTERIOUSBOT_CONFIG").unwrap_or("./config/mysteriousbot.yml".to_owned());
    let db_file = &env::var("MYSTERIOUSBOT_DB").unwrap_or("./db/mysteriousbot.sqlite3".to_owned());
    let pool = Pool::new(SqliteConnectionManager::file(db_file))
        .expect("Couldn't put database in the pool. Party foul.");
    let handler = Handler {
        config: serde_yaml::from_str(
            &read_to_string(config_file)
                .expect(&format!("Config file at {} could not be read", config_file)),
        )
        .expect("Config could not be parsed"),
        emoji_cache: EmojiCache::new(),
        counter_factory: CounterFactory::new(pool.clone()).expect("Cannot create counter factory"),
        pool,
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
