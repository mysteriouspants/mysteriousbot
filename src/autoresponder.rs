use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use rand::prelude::SliceRandom;
use regex::Regex;
use serde::Deserialize;
use serde_with::{formats::PreferOne, serde_as, DisplayFromStr, DurationSeconds, OneOrMany};
use serenity::{
    client::Context,
    model::{channel::Message, id::GuildId},
};
use tokio::sync::Mutex;

use crate::{counter::CounterFactory, emojicache::EmojiCache};

#[derive(Debug, Deserialize)]
pub struct Autoresponder {
    #[serde(flatten)]
    trigger: AutoresponderTrigger,
    #[serde(flatten)]
    filter: AutoresponderFilter,
    #[serde(flatten)]
    action: AutoresponderAction,
}

impl Autoresponder {
    pub async fn handle(
        &self,
        emojicache: &EmojiCache,
        counter_factory: &CounterFactory,
        context: &Context,
        message: &Message,
        guild_id: &GuildId,
    ) {
        if self.trigger.should_run(context, message) && self.filter.should_run(message).await {
            self.action
                .run(emojicache, counter_factory, context, guild_id, message)
                .await;
        }
    }
}

#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum AutoresponderTrigger {
    MessageMatches {
        #[serde_as(as = "OneOrMany<DisplayFromStr, PreferOne>")]
        message_matches: Vec<Regex>,
    },
    UserMessage {
        #[serde_as(as = "OneOrMany<_, PreferOne>")]
        user_message: Vec<u64>,
    },
    UserMentioned {
        #[serde_as(as = "OneOrMany<_, PreferOne>")]
        user_mentioned: Vec<u64>,
    },
}

impl AutoresponderTrigger {
    fn should_run(&self, context: &Context, message: &Message) -> bool {
        match self {
            Self::MessageMatches { message_matches } => {
                let message_content = message.content_safe(context);
                message_matches
                    .iter()
                    .any(|regex| regex.is_match(&message_content))
            }
            Self::UserMessage { user_message } => user_message
                .iter()
                .any(|user_id| user_id == &message.author.id.get()),
            Self::UserMentioned { user_mentioned } => user_mentioned
                .iter()
                .any(|user_id| message.mentions_user_id(*user_id)),
        }
    }
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct AutoresponderFilter {
    #[serde(default)]
    #[serde_as(as = "OneOrMany<_, PreferOne>")]
    only_in_channels: Vec<u64>,
    #[serde(default = "default_cooldown")]
    #[serde_as(as = "DurationSeconds<u64>")]
    cooldown: Duration,
    #[serde(skip, default = "default_system_time")]
    last_triggered: Arc<Mutex<SystemTime>>,
}

impl AutoresponderFilter {
    async fn should_run(&self, message: &Message) -> bool {
        // basic channel filter
        if self.only_in_channels.len() > 0
            && !self.only_in_channels.contains(&message.channel_id.get())
        {
            return false;
        }

        let now = SystemTime::now();

        let mut last_triggered = self.last_triggered.lock().await;
        let t = now.duration_since(*last_triggered).unwrap_or_default();

        // reject if now is within cooldown of the last triggering
        if t < self.cooldown {
            return false;
        }

        *last_triggered = now;
        true
    }
}

const fn default_cooldown() -> Duration {
    Duration::ZERO
}

fn default_system_time() -> Arc<Mutex<SystemTime>> {
    Arc::new(Mutex::new(SystemTime::UNIX_EPOCH))
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct AutoresponderAction {
    #[serde(default)]
    #[serde_as(as = "OneOrMany<_, PreferOne>")]
    twemojis: Vec<String>,
    #[serde(default)]
    #[serde_as(as = "OneOrMany<_, PreferOne>")]
    reply_messages: Vec<String>,
    #[serde(default)]
    #[serde_as(as = "OneOrMany<_, PreferOne>")]
    counter: Vec<String>,
}

impl AutoresponderAction {
    async fn run(
        &self,
        emojicache: &EmojiCache,
        counter_factory: &CounterFactory,
        context: &Context,
        guild_id: &GuildId,
        message: &Message,
    ) {
        for counter in &self.counter {
            let counter = counter_factory.make_counter(&counter);
            if let Err(e) = counter.increment(message.author.id) {
                log::error!(
                    "Failed to increment counter {:?} for user {} with error {:#?}",
                    counter,
                    message.author.id.get(),
                    e
                );
            }
        }

        for twemoji in &self.twemojis {
            if let Ok(Some(emoji)) = emojicache.get_emoji(&context, guild_id, &twemoji).await {
                if let Err(why) = message.react(context, emoji).await {
                    log::error!("Failed to react to message with reason {:?}", why);
                } else {
                    log::error!("Unknown twemoji {} for guild {}", twemoji, guild_id);
                }
            }
        }

        let content = { self.reply_messages.choose(&mut rand::thread_rng()) };

        if let Some(content) = content {
            if let Err(why) = message.reply(context, content).await {
                log::error!("Failed to autoreply to message with reason {:?}", why);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Autoresponder, AutoresponderAction, AutoresponderFilter, AutoresponderTrigger};

    #[test]
    fn autorespondertrigger_single_messagematches() {
        let yaml = r#"---
        message_matches: foo"#;
        let autorespondertrigger: AutoresponderTrigger = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(
            autorespondertrigger,
            AutoresponderTrigger::MessageMatches { message_matches: _ }
        ));
        if let AutoresponderTrigger::MessageMatches {
            message_matches: regexes,
        } = autorespondertrigger
        {
            assert_eq!(1, regexes.len());
        }
    }

    #[test]
    fn autorespondertrigger_multiple_messagematches() {
        let yaml = r#"---
        message_matches:
          - foo
          - bar"#;
        let autorespondertrigger: AutoresponderTrigger = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(
            autorespondertrigger,
            AutoresponderTrigger::MessageMatches { message_matches: _ }
        ));
        if let AutoresponderTrigger::MessageMatches {
            message_matches: regexes,
        } = autorespondertrigger
        {
            assert_eq!(2, regexes.len());
        }
    }

    #[test]
    fn autorespondertrigger_single_usermessage() {
        let yaml = r#"---
        user_message: 1"#;
        let autorespondertrigger: AutoresponderTrigger = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(
            autorespondertrigger,
            AutoresponderTrigger::UserMessage { user_message: _ }
        ));
        if let AutoresponderTrigger::UserMessage {
            user_message: users,
        } = autorespondertrigger
        {
            assert_eq!(1, users.len());
        }
    }

    #[test]
    fn autorespondertrigger_multiple_usermessage() {
        let yaml = r#"---
        user_message:
          - 1
          - 2"#;
        let autorespondertrigger: AutoresponderTrigger = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(
            autorespondertrigger,
            AutoresponderTrigger::UserMessage { user_message: _ }
        ));
        if let AutoresponderTrigger::UserMessage {
            user_message: users,
        } = autorespondertrigger
        {
            assert_eq!(2, users.len());
        }
    }

    #[test]
    fn autorespondertrigger_single_usermention() {
        let yaml = r#"---
        user_mentioned: 1"#;
        let autorespondertrigger: AutoresponderTrigger = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(
            autorespondertrigger,
            AutoresponderTrigger::UserMentioned { user_mentioned: _ }
        ));
        if let AutoresponderTrigger::UserMentioned {
            user_mentioned: users,
        } = autorespondertrigger
        {
            assert_eq!(1, users.len());
        }
    }

    #[test]
    fn autorespondertrigger_multiple_usermentioned() {
        let yaml = r#"---
        user_mentioned:
          - 1
          - 2"#;
        let autorespondertrigger: AutoresponderTrigger = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(
            autorespondertrigger,
            AutoresponderTrigger::UserMentioned { user_mentioned: _ }
        ));
        if let AutoresponderTrigger::UserMentioned {
            user_mentioned: users,
        } = autorespondertrigger
        {
            assert_eq!(2, users.len());
        }
    }

    #[test]
    fn autoresponderaction_single_twemoji() {
        let yaml = r#"---
        twemojis: PingBad"#;
        let autoresponderaction: AutoresponderAction = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(1, autoresponderaction.twemojis.len());
        assert_eq!(0, autoresponderaction.reply_messages.len());
    }

    #[test]
    fn autoresponderaction_multiple_twemoji() {
        let yaml = r#"---
        twemojis:
          - PingBad
          - PES_Ping"#;
        let autoresponderaction: AutoresponderAction = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(2, autoresponderaction.twemojis.len());
        assert_eq!(0, autoresponderaction.reply_messages.len());
    }

    #[test]
    fn autoresponderaction_single_replymessage() {
        let yaml = r#"---
        reply_messages: foo"#;
        let autoresponderaction: AutoresponderAction = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(0, autoresponderaction.twemojis.len());
        assert_eq!(1, autoresponderaction.reply_messages.len());
    }

    #[test]
    fn autoresponderaction_multiple_replymessage() {
        let yaml = r#"---
        reply_messages:
          - foo
          - bar"#;
        let autoresponderaction: AutoresponderAction = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(0, autoresponderaction.twemojis.len());
        assert_eq!(2, autoresponderaction.reply_messages.len());
    }

    #[test]
    fn autoresponderaction_many_actions() {
        let yaml = r#"---
        twemojis: PingBad
        reply_messages: Grr"#;
        let autoresponderaction: AutoresponderAction = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(1, autoresponderaction.twemojis.len());
        assert_eq!(1, autoresponderaction.reply_messages.len());
    }

    #[test]
    fn autoresponderfilter_single_channel() {
        let yaml = r#"---
        only_in_channels: 1"#;
        let autoresponderfilter: AutoresponderFilter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(1, autoresponderfilter.only_in_channels.len());
        assert_eq!(Duration::ZERO, autoresponderfilter.cooldown);
    }

    #[test]
    fn autoresponder_basic_definition1() {
        let yaml = r#"---
        user_mentioned: 261559920485335040 # P33ky
        twemojis: pingsock
        only_in_channels:
          - 499363186957352970 #alliance
          - 499363309070319616 #nsfw"#;
        let _: Autoresponder = serde_yaml::from_str(yaml).unwrap();
    }

    #[test]
    fn autoresponder_basic_definition2() {
        let yaml = r#"---
        user_message: 139425197118849025 # Skorpion Medion
        twemojis: swedishfish
        only_in_channels:
          - 499363186957352970 # alliance
          - 499363309070319616 # nsfw"#;
        let _: Autoresponder = serde_yaml::from_str(yaml).unwrap();
    }
}
