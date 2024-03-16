use std::collections::HashMap;

use serde::Deserialize;

use crate::{autoresponder::Autoresponder, command::Command};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub guilds: HashMap<u64, GuildConfig>,
}

#[derive(Debug, Deserialize)]
pub struct GuildConfig {
    #[serde(default)]
    pub commands: Vec<Command>,
    #[serde(default)]
    pub autoresponders: Vec<Autoresponder>,
}

#[cfg(test)]
mod tests {
    use super::Config;
    use std::fs::read_to_string;

    #[test]
    fn can_parse() {
        let config = serde_yaml::from_str::<Config>(
            &read_to_string("config/mysteriousbot.yml").expect("no such file"),
        ).expect("Config is not well-formed");
        eprintln!("{:#?}", config);
    }
}
