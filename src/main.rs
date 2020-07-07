mod ack_message_handler;
mod mysterious_message_handler;
mod role_wizard;
mod word_watcher;

use crate::ack_message_handler::AckMessageHandler;
use crate::mysterious_message_handler::MysteriousMessageHandler;
use crate::role_wizard::RoleWizard;
use crate::word_watcher::WordWatcher;
use serenity::client::Client;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::{EventHandler, Context};
use std::env;
use toml::Value;
use toml::value::Table;


struct Handler {
    message_handlers: Vec<Box<dyn MysteriousMessageHandler>>,
}

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        for handler in &self.message_handlers {
            if handler.should_handle(&ctx, &msg) {
                handler.on_message(&ctx, &msg);

                if handler.is_exclusive() {
                    return;
                }
            }
        }
    }

    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

// this code is kinda unwrappy but I think that's okay because dying in
// initialization is sorta expected on bad config, right?
fn parse_handlers(raw_toml: String) -> Vec<Box<dyn MysteriousMessageHandler>> {
    let toml = raw_toml.parse::<Value>().unwrap();
    let handlers = toml.as_table().unwrap().get("handlers").unwrap().as_array().unwrap();
    let mut parsed_handlers: Vec<Box<dyn MysteriousMessageHandler>> = Vec::new();

    for handler_value in handlers {
        let handler_config = handler_value.as_table().unwrap();
        let handler_type = handler_config.get("type").unwrap().as_str().unwrap();

        match handler_type {
            "RoleWizard" => parsed_handlers.push(Box::new(
                role_wizard_from_config(&handler_config)
            )),
            "AckMessage" => parsed_handlers.push(Box::new(
                ack_message_handler_from_config(&handler_config)
            )),
            "WordWatcher" => parsed_handlers.push(Box::new(
                word_watcher_handler_from_config(&handler_config)
            )),
            _ => { /* do nothing, i guess */ }
        };
    }

    parsed_handlers
}

fn role_wizard_from_config(config: &Table) -> RoleWizard {
    let grants = array_to_string_array(
        config.get("allowed_role_grants").unwrap().as_array().unwrap()
    );
    let revoke = array_to_string_array(
        config.get("allowed_role_revoke").unwrap().as_array().unwrap()
    );

    RoleWizard::new(grants, revoke)
}

fn ack_message_handler_from_config(config: &Table) -> AckMessageHandler {
    let deny_list = array_to_string_array(
        config.get("deny_channels").unwrap().as_array().unwrap()
    );

    AckMessageHandler::new(deny_list)
}

fn word_watcher_handler_from_config(config: &Table) -> WordWatcher {
    let watched_words: Vec<String> = array_to_string_array(
        config.get("watched_words").unwrap().as_array().unwrap()
    );
    let deny_channels: Vec<String> = array_to_string_array(
        config.get("deny_channels").unwrap().as_array().unwrap()
    );
    let suggest_channel = config.get("suggest_channel")
        .unwrap().as_str().unwrap().to_owned();

    WordWatcher::new(watched_words, deny_channels, suggest_channel)
}

fn array_to_string_array(array: &Vec<Value>) -> Vec<String> {
    array.iter().map(|item| item.as_str().unwrap().to_owned()).collect()
}

fn main() {
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
