mod ack_message_handler;
mod config_parser;
mod mysterious_message_handler;
mod role_wizard;
mod verbal_morality_handler;

use crate::config_parser::parse_handlers;
use crate::mysterious_message_handler::MysteriousMessageHandler;
use dotenv::dotenv;
use serenity::client::Client;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::{EventHandler, Context};
use std::env;


struct Handler {
    message_handlers: Vec<Box<dyn MysteriousMessageHandler>>,
}

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        for handler in &self.message_handlers {
            match handler.should_handle(&ctx, &msg) {
                Ok(true) => {
                    match handler.on_message(&ctx, &msg) {
                        Ok(_) => { },
                        Err(err) => {
                            println!("handler {}", err.as_ref());
                        }
                    }

                    if handler.is_exclusive() {
                        return;
                    }
                },
                Ok(false) => {},
                Err(err) => {
                    println!("Handler {}", err.as_ref());
                }
            }
        }
    }

    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn main() {
    dotenv().ok(); // enable use of .env files
    let token = &env::var("DISCORD_TOKEN")
        .expect("DISCORD_TOKEN environment variable is unset, exiting");
    let config_file = &env::var("MYSTERIOUSBOT_CONFIG")
        .unwrap_or("./config/mysteriousbot.toml".to_owned());
    let handler = Handler {
        message_handlers: parse_handlers(
            std::fs::read_to_string(config_file).unwrap()
        )
    };
    let mut client = Client::new(
        token, handler
    ).expect("Error creating the client");

    if let Err(why) = client.start() {
        println!("An error occurred while running the client: {:?}", why);
    }
}
