use crate::mysterious_message_handler::{
    MMHResult, MysteriousMessageHandler
};
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

    fn should_handle(&self, ctx: &Context, msg: &Message) -> MMHResult<bool> {
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

        return Ok(msg.mentions_user_id(ctx.cache.read().user.id));
    }

    fn on_message(&self, ctx: &Context, msg: &Message) -> MMHResult<()> {
        msg.channel_id.say(&ctx.http, "I don't know about that")?;
        println!("Message:\n\t{}", msg.content);
        Ok(())
    }
}
