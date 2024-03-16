use rand::{prelude::SliceRandom, thread_rng};
use serde::Deserialize;
use serde_with::{formats::PreferOne, serde_as, OneOrMany};
use serenity::{
    all::{CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage},
    client::Context,
    model::application::CommandInteraction,
};

use crate::counter::{Counter, CounterFactory};

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct Command {
    pub alias: String,
    pub description: String,
    #[serde(default)]
    #[serde_as(as = "OneOrMany<_, PreferOne>")]
    pub reply_messages: Vec<String>,
    pub counter_leaderboard: Option<String>,
}

impl Command {
    pub async fn handle(
        &self,
        interaction: &CommandInteraction,
        ctx: Context,
        counter_factory: &CounterFactory,
    ) {
        if !self.reply_messages.is_empty() {
            handle_reply_message(&ctx, interaction, &self.reply_messages).await;
        }

        if let Some(counter_name) = &self.counter_leaderboard {
            let counter = counter_factory.make_counter(&counter_name);

            handle_counter_leaderboard(&ctx, interaction, &counter).await;
        }
    }
}

async fn handle_reply_message(
    ctx: &Context,
    interaction: &CommandInteraction,
    messages: &Vec<String>,
) {
    let content = match messages.choose(&mut thread_rng()) {
        Some(content) => content,
        None => {
            log::error!("No responses configured");
            return;
        }
    };

    let interaction_response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new().content(content),
    );

    let r = interaction
        .create_response(&ctx.http, interaction_response)
        .await
        .err();

    if let Some(e) = r {
        log::error!("Failed to respond to interaction with error {:?}", e);
    }
}

async fn handle_counter_leaderboard(
    ctx: &Context,
    interaction: &CommandInteraction,
    counter: &Counter,
) {
    let guild_id = match interaction.guild_id {
        Some(guild_id) => guild_id,
        None => {
            log::warn!("Interaction occurred without guild_id, aborting.");
            return;
        }
    };

    let top_counts = match counter.top_counts(interaction.user.id) {
        Ok(top_counts) => top_counts,
        Err(e) => {
            log::error!(
                "Failed to retrieve top counts for counter {:?} with error {:#?}",
                counter,
                e
            );
            return;
        }
    };

    let mut named_counts = vec![];

    for (user_id, count) in top_counts {
        let user = match user_id.to_user(ctx).await {
            Ok(user) => user,
            Err(e) => {
                log::error!(
                    "Failed to get user info for user {} with error {:#?}",
                    user_id,
                    e
                );
                continue;
            }
        };

        let nick = match user.nick_in(ctx, guild_id).await {
            Some(nick) => nick,
            None => user.name,
        };

        named_counts.push((nick, count));
    }

    let mut embed = CreateEmbed::new();

    for (name, count) in named_counts {
        embed = embed.field(name, count.to_string(), false);
    }

    let interaction_response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new().add_embed(embed),
    );

    let response_status = interaction
        .create_response(ctx, interaction_response)
        .await
        .err();

    if let Some(e) = response_status {
        log::error!("Failed to publish leaderboard response with error {:#?}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::Command;

    #[test]
    fn command_singlereplymessage_deserialization() {
        let yaml = r#"---
        alias: a_command
        description: does stuff
        reply_messages: hello, world!"#;
        let command: Command = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(1, command.reply_messages.len());
    }

    #[test]
    fn command_multiplereplymessage_deserialization() {
        let yaml = r#"---
        alias: a_command
        description: does stuff
        reply_messages:
          - hello, world!
          - ¡hola, mundo!"#;
        let command: Command = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(2, command.reply_messages.len());
    }

    #[test]
    fn command_noreplymessage_deserialization() {
        let yaml = r#"---
        alias: a_command
        description: does stuff"#;
        assert!(serde_yaml::from_str::<Command>(yaml).is_ok());
    }
}
