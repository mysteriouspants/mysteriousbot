use crate::mysterious_message_handler::MysteriousMessageHandler;
use serenity::model::channel::Message;
use serenity::model::id::ChannelId;
use serenity::prelude::Context;


/// Handler which watches messages for specific words. When such a word is
/// found, it directs the utterer to the suggested channel.
pub struct WordWatcher {
    /// Words which get a nudge from the bot.
    watched_words: Vec<String>,
    /// Users who are allowed to say these words without a nudge.
    allow_users_by_tag: Vec<String>,
    /// Channels in which to allow the watched words.
    deny_channels: Vec<String>,
    /// The channel the bot nudges the conversation toward.
    suggest_channel: String,
}

impl WordWatcher {
    pub fn new(
        words: Vec<String>, allow_users: Vec<String>,
        deny_channels: Vec<String>, suggest_channel: String
    ) -> WordWatcher {
        let watched_words = words.iter()
            .map(|word| word.to_lowercase())
            .collect();
        let allow_users_by_tag = allow_users.iter()
            .map(|word| word.to_lowercase())
            .collect();
        WordWatcher {
            watched_words, allow_users_by_tag, deny_channels, suggest_channel
        }
    }
}

impl MysteriousMessageHandler for WordWatcher {
    fn is_exclusive(&self) -> bool {
        false
    }

    fn should_handle(&self, ctx: &Context, msg: &Message) -> bool {
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
                return false;
            }
        }

        // if this was said by a user that is allowed to say these words without
        // a nudge we shouldn't use this handler
        if self.allow_users_by_tag.contains(&msg.author.tag().to_lowercase()) {
            return false;
        }

        // if we find a word that should be nudged we should use this handler
        let haystack = msg.content.to_lowercase();
        for watched_word in &self.watched_words {
            if haystack.contains(watched_word) {
                return true;
            }
        }

        false
    }

    fn on_message(&self, ctx: &Context, msg: &Message) {
        msg.channel_id.say(
            &ctx.http,
            format!(
                "Hey, that sounds like it may be best taken to #{}.",
                self.suggest_channel
            )
        );
    }
}
