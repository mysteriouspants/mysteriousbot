use crate::mysterious_message_handler::{
    MMHResult, MysteriousMessageHandler
};
use futures::future::join_all;
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

#[async_trait::async_trait]
impl MysteriousMessageHandler for AckMessageHandler {
    fn is_exclusive(&self) -> bool {
        false
    }

    async fn should_handle(&self, ctx: &Context, msg: &Message) -> MMHResult<bool> {
        if let Some(guild) = msg.guild(&ctx.cache).await {
            let deny_channel_ids: Vec<Option<ChannelId>> = 
                join_all(self.deny_channels.iter()
                .map(|channel_name| 
                    guild.channel_id_from_name(&ctx.cache, channel_name)
                )).await;

            if deny_channel_ids.contains(&Some(msg.channel_id)) {
                return Ok(false);
            }
        }

        return Ok(msg.mentions_me(&ctx).await?);
    }

    async fn on_message(&self, ctx: &Context, msg: &Message) -> MMHResult<()> {
        msg.channel_id.say(&ctx.http, "I don't know about that").await?;
        println!("Message:\n\t{}", msg.content);
        Ok(())
    }
}
