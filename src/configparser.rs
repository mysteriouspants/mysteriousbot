use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Configures the mysterious bot.
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// Per-guild config.
    pub guilds: HashMap<String, GuildConfig>,
}

/// Configures how the mysterious bot will behave in a specific guild.
#[derive(Debug, Deserialize, Serialize)]
pub struct GuildConfig {
    /// Slash-responders, which are simple shortcuts that echo a
    /// preset response when invoked. This is for the lulz.
    pub slash_responders: Option<Vec<SlashResponderConfig>>,
    /// Auto-emojis, which automatically react to messages mentioning or
    /// sent by a specific individual with a specific emoji. This too is
    /// for the lulz.
    pub autoemojis: Option<Vec<AutoEmojiConfig>>,
}

/// A slash responder echoes a preset response when invoked.
#[derive(Debug, Deserialize, Serialize)]
pub struct SlashResponderConfig {
    /// The name of the slash command.
    pub command: String,
    /// The resonse to echo back.
    pub response: String,
}

/// The trigger for an auto-emoji.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoEmojiTrigger {
    /// Trigger when someone is mentioned, eg `Let's go bug @P33ky!`.
    Mention(u64),
    /// Trigger when someone says something, eg any time Skorp says
    /// anything.
    Message(u64),
}

/// Automatically reacts to messages with emojis. This can be really
/// freaking annoying, so it's a good idea to limit it in some ways.
#[derive(Debug, Deserialize, Serialize)]
pub struct AutoEmojiConfig {
    /// The user to annotate on mention of.
    pub on: AutoEmojiTrigger,
    /// The twemojis to place, eg `:swedishfish:` here is simply
    /// `swedishfish`.
    pub twemojis: Vec<String>,
    /// Do not automatically put an emoji reaction in these channels.
    pub ignore_channels: Option<Vec<u64>>,
    /// When present only react in these channels.
    pub only_in_channels: Option<Vec<u64>>,
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::*;

    #[test]
    fn config_is_valid() {
        let config: Config = toml::from_str(
            &read_to_string("config/mysteriousbot.toml").unwrap()
        ).unwrap();
        println!("{:?}", config);
    }
}