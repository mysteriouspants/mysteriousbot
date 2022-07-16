use rand::{prelude::SliceRandom, thread_rng};
use serde::Deserialize;
use serde_with::{formats::PreferOne, serde_as, OneOrMany};
use serenity::{
    client::Context,
    model::interactions::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
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
        interaction: &ApplicationCommandInteraction,
        ctx: Context,
        counter_factory: &CounterFactory,
    ) {
        if self.reply_messages.is_empty() {
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
    interaction: &ApplicationCommandInteraction,
    messages: &Vec<String>,
) {
    let content = match messages.choose(&mut thread_rng()) {
        Some(content) => content,
        None => {
            log::error!("No responses configured");
            return;
        }
    };

    let r = interaction
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(content))
        })
        .await
        .err();

    if let Some(e) = r {
        log::error!("Failed to respond to interaction with error {:?}", e);
    }
}

async fn handle_counter_leaderboard(
    ctx: &Context,
    interaction: &ApplicationCommandInteraction,
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

    let response_status = interaction
        .create_interaction_response(ctx, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    for (name, count) in named_counts {
                        message.embed(|embed| embed.field(name, count.to_string(), false));
                    }

                    message
                })
        })
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
