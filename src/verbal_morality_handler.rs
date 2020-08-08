use crate::mysterious_message_handler::{
    MMHResult, MysteriousMessageHandler
};
use serenity::model::channel::Message;
use serenity::model::id::ChannelId;
use serenity::prelude::Context;


/// Handler which watches messages for specific words. When such a word is
/// found, it directs the utterer to the suggested channel.
pub struct VerbalMoralityHandler {
    /// Words which get a nudge from the bot.
    bad_words: Vec<String>, // bad words, whatcha gonna do?
    /// Users who are allowed to say these words without a nudge.
    allow_users_by_tag: Vec<String>,
    /// Channels in which to allow the watched words.
    deny_channels: Vec<String>,
    /// The message to display on infraction.
    warning_message: String,
}

impl VerbalMoralityHandler {
    pub fn new(
        words: Vec<String>, allow_users: Vec<String>,
        deny_channels: Vec<String>, warning_message: String
    ) -> VerbalMoralityHandler {
        let bad_words = words.iter()
            .map(|word| word.to_lowercase())
            .collect();
        let allow_users_by_tag = allow_users.iter()
            .map(|word| word.to_lowercase())
            .collect();
        VerbalMoralityHandler {
            bad_words, allow_users_by_tag, deny_channels, warning_message
        }
    }
}

impl MysteriousMessageHandler for VerbalMoralityHandler {
    fn is_exclusive(&self) -> bool {
        false
    }

    fn should_handle(&self, ctx: &Context, msg: &Message) -> MMHResult<bool> {
        // if this is in a channel that is allowed to say these words without
        // a nudge we shouldn't use this handler
        if let Some(guild_lock) = msg.guild(&ctx.cache) {
            let guild = guild_lock.read();
            let deny_channel_ids: Vec<Option<ChannelId>> = 
                self.deny_channels.iter()
                .map(|channel_name| 
                    guild.channel_id_from_name(&ctx.cache, channel_name)
                ).collect();

            if deny_channel_ids.contains(&Some(msg.channel_id)) {
                return Ok(false);
            }
        }

        // if this was said by a user that is allowed to say these words without
        // a nudge we shouldn't use this handler
        if self.allow_users_by_tag.contains(&msg.author.tag().to_lowercase()) {
            return Ok(false);
        }

        // if we find a word that should be nudged we should use this handler
        let haystack = msg.content.to_lowercase();
        for watched_word in &self.bad_words {
            if haystack.contains(watched_word) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn on_message(&self, ctx: &Context, msg: &Message) -> MMHResult<()> {
        let user = match msg.author_nick(&ctx.http) {
            Some(u) => u,
            None => {
                msg.author.name.to_owned()
            }
        };

        let warning_message = self.warning_message
            .replace("{{user}}", &user);
        msg.channel_id.say(&ctx.http, &warning_message)?;

        Ok(())
    }
}
