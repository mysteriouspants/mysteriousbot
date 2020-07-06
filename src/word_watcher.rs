use crate::cmdr_message_handler::CmdrMessageHandler;
use serenity::model::channel::Message;
use serenity::prelude::Context;


/// Handler which watches messages for specific words. When such a word is
/// found, it directs the utterer to the suggested channel.
pub struct WordWatcher {
    watched_words: Vec<String>,
    suggest_channel: String,
}

impl WordWatcher {
    pub fn new(
        words: Vec<String>, suggest_channel: String
    ) -> WordWatcher {
        let watched_words = words.iter()
            .map(|word| word.to_lowercase())
            .collect();
        WordWatcher { watched_words, suggest_channel }
    }
}

impl CmdrMessageHandler for WordWatcher {
    fn is_exclusive(&self) -> bool {
        false
    }

    fn should_handle(&self, _ctx: &Context, msg: &Message) -> bool {
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
