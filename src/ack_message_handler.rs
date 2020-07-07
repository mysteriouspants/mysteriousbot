use crate::mysterious_message_handler::MysteriousMessageHandler;
use serenity::model::channel::Message;
use serenity::model::id::ChannelId;
use serenity::prelude::Context;


/// A simple handler which, whenever the bot is mentioned, just replies with a
/// little something so that people know it is still active.
pub struct AckMessageHandler {
    /// Channels that the bot will never ack in.
    deny_channels: Vec<String>
}

impl AckMessageHandler {
    pub fn new(deny_channels: Vec<String>) -> AckMessageHandler {
        AckMessageHandler { deny_channels }
    }
}

impl MysteriousMessageHandler for AckMessageHandler {
    fn is_exclusive(&self) -> bool {
        false
    }

    fn should_handle(&self, ctx: &Context, msg: &Message) -> bool {
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

        if let Ok(current_user) = ctx.http.get_current_user() {
            let user_id = current_user.id;
            
            if msg.mentions_user_id(user_id) {
                return true;
            }
        }

        // just silently ignore any failures, I think that's fair for something
        // this trivial
        false
    }

    fn on_message(&self, ctx: &Context, msg: &Message) {
        msg.channel_id.say(&ctx.http, "I don't know about that");
    }
}
