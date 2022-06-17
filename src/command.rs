use rand::{prelude::SliceRandom, thread_rng};
use serde::Deserialize;
use serde_with::{formats::PreferOne, serde_as, OneOrMany};
use serenity::{
    client::Context,
    model::interactions::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
};

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct Command {
    pub alias: String,
    pub description: String,
    #[serde_as(as = "OneOrMany<_, PreferOne>")]
    pub reply_messages: Vec<String>,
}

impl Command {
    pub async fn handle(&self, interaction: &ApplicationCommandInteraction, ctx: Context) {
        let content = match self.reply_messages.choose(&mut thread_rng()) {
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
        assert!(serde_yaml::from_str::<Command>(yaml).is_err());
    }
}
