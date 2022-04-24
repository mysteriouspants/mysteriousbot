use regex::Regex;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::{collections::HashMap, num::ParseIntError, str::FromStr};
use toml::de::Error as TomlError;

/// Configures the mysterious bot.
#[derive(Debug)]
pub struct Config {
    /// Per-guild config.
    pub guilds: HashMap<u64, GuildConfig>,
}

impl FromStr for Config {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let sc: SerializedConfig = toml::from_str(value).with_context(|_| TomlSnafu {
            string: value.to_owned(),
        })?;
        let mut guilds = HashMap::new();

        for (k, v) in sc.guilds.into_iter() {
            guilds.insert(
                k.parse().with_context(|_| NonIntegerGuildIdSnafu {
                    string: k.to_owned(),
                })?,
                v.to_guild_config(),
            );
        }

        Ok(Config { guilds })
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to parse {string} as TOML with error {source:?}"))]
    Toml { string: String, source: TomlError },
    #[snafu(display("{string} is not an integer and cannot be interpreted as a Guild ID"))]
    NonIntegerGuildId {
        string: String,
        source: ParseIntError,
    },
}

/// Configures the mysterious bot.
#[derive(Debug, Deserialize, Serialize)]
struct SerializedConfig {
    /// Per-guild config.
    guilds: HashMap<String, SerializedGuildConfig>,
}

/// Configures how the mysterious bot will behave in a specific guild.
#[derive(Debug)]
pub struct GuildConfig {
    /// Slash-responders, which are simple shortcuts that echo a
    /// preset response when invoked. This is for the lulz.
    pub slash_responders: Vec<SlashResponderConfig>,
    /// Auto-emojis, which automatically react to messages mentioning or
    /// sent by a specific individual with a specific emoji. This too is
    /// for the lulz.
    pub autoemojis: Vec<AutoEmojiConfig>,
    pub autoresponders: Vec<AutoResponderConfig>,
}

/// Configures how the mysterious bot will behave in a specific guild.
#[derive(Debug, Deserialize, Serialize)]
struct SerializedGuildConfig {
    /// Slash-responders, which are simple shortcuts that echo a
    /// preset response when invoked. This is for the lulz.
    slash_responders: Option<Vec<SlashResponderConfig>>,
    /// Auto-emojis, which automatically react to messages mentioning or
    /// sent by a specific individual with a specific emoji. This too is
    /// for the lulz.
    autoemojis: Option<Vec<AutoEmojiConfig>>,
    autoresponders: Option<Vec<AutoResponderConfig>>,
}

impl SerializedGuildConfig {
    fn to_guild_config(self) -> GuildConfig {
        GuildConfig {
            slash_responders: self.slash_responders.unwrap_or_else(|| Vec::default()),
            autoemojis: self.autoemojis.unwrap_or_else(|| Vec::default()),
            autoresponders: self.autoresponders.unwrap_or_else(|| Vec::default()),
        }
    }
}

/// A slash responder echoes a preset response when invoked.
#[derive(Debug, Deserialize, Serialize)]
pub struct SlashResponderConfig {
    /// The name of the slash command.
    pub command: String,
    /// A little help string.
    pub description: String,
    /// The resonse to echo back.
    pub response: String,
}

/// The trigger for a behavior.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoTrigger {
    /// Trigger when someone is mentioned, eg `Let's go bug @P33ky!`.
    Mention(u64),
    /// Trigger when someone says something, eg any time Skorp says
    /// anything.
    Message(u64),
    /// Trigger when anything matches this regex.
    #[serde(with = "serde_regex")]
    Match(Regex),
}

/// Reusable filter for allowing behavior only in or never in specific
/// channels.
#[derive(Debug, Deserialize, Serialize)]
pub struct ChannelFilter {
    /// Do not automatically put an emoji reaction in these channels.
    pub ignore_channels: Option<Vec<u64>>,
    /// When present only react in these channels.
    pub only_in_channels: Option<Vec<u64>>,
}

/// Automatically reacts to messages with emojis. This can be really
/// freaking annoying, so it's a good idea to limit it in some ways.
#[derive(Debug, Deserialize, Serialize)]
pub struct AutoEmojiConfig {
    /// The user to annotate on mention of.
    pub on: AutoTrigger,
    /// Relevant channel filters to restrict the annotation behavior.
    #[serde(flatten)]
    pub channel_filter: ChannelFilter,
    /// The twemojis to place, eg `:swedishfish:` here is simply
    /// `swedishfish`.
    pub twemojis: Vec<String>,
}

/// A simple responder which replies to a message.
#[derive(Debug, Deserialize, Serialize)]
pub struct AutoResponderConfig {
    /// The trigger which initiates the reply.
    pub on: AutoTrigger,
    /// Relevant channel filters to restrict the responder.
    #[serde(flatten)]
    pub channel_filter: ChannelFilter,
    /// The message to reply with.
    pub message: String,
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::*;

    #[test]
    fn config_is_valid() {
        let config =
            Config::from_str(&read_to_string("config/mysteriousbot.toml").unwrap()).unwrap();
        println!("{:?}", config);
    }
}
