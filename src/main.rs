mod ack_message_handler;
mod config_parser;
mod mysterious_message_handler;
mod role_wizard;

use crate::config_parser::parse_handlers;
use crate::mysterious_message_handler::MysteriousMessageHandler;
use dotenv::dotenv;
use serenity::client::Client;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::{Context, EventHandler};
use std::env;
use tokio_compat_02::FutureExt;

struct Handler {
    message_handlers: Vec<Box<dyn MysteriousMessageHandler>>,
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        for handler in &self.message_handlers {
            match handler.should_handle(&ctx, &msg).await {
                Ok(true) => {
                    match handler.on_message(&ctx, &msg).await {
                        Ok(_) => {}
                        Err(err) => {
                            println!("handler {}", err);
                        }
                    }

                    if handler.is_exclusive() {
                        return;
                    }
                }
                Ok(false) => {}
                Err(err) => {
                    println!("Handler {}", err);
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok(); // enable use of .env files
    let token =
        &env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN environment variable is unset, exiting");
    let config_file =
        &env::var("MYSTERIOUSBOT_CONFIG").unwrap_or("./config/mysteriousbot.toml".to_owned());
    let handler = Handler {
        message_handlers: parse_handlers(std::fs::read_to_string(config_file).unwrap()),
    };
    let mut client = Client::builder(token)
        .event_handler(handler)
        .compat()
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().compat().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
