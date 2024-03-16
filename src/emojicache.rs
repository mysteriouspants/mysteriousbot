use mysterious_cache::{ExpiringCache, SharedCache};
use serenity::Error as DiscordError;
use serenity::{
    client::Context,
    model::{guild::Emoji, id::GuildId},
};
use snafu::{ResultExt, Snafu};
use std::time::Duration;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to call Discord with error {source:?}"))]
    Discord { source: DiscordError },
}

/// An LRU Cache which holds the emojis - cached because hitting the
/// Discord emoji API potentially on every event sounds like a bad time.
pub struct EmojiCache {
    cache: SharedCache<ExpiringCache<u64, Vec<Emoji>>, u64, Vec<Emoji>>,
}

impl EmojiCache {
    pub fn new() -> Self {
        Self {
            cache: SharedCache::with_cache(ExpiringCache::with_capacity_and_timeout(
                5,
                Duration::from_secs(24 * 60 * 60),
            )),
        }
    }
    pub async fn get_emoji(
        &self,
        ctx: &Context,
        guild_id: &GuildId,
        twemoji: &str,
    ) -> Result<Option<Emoji>, Error> {
        let emojis = self.get_emojis(ctx, guild_id).await?;
        Ok(emojis
            .iter()
            .filter(|emoji| emoji.name == twemoji)
            .next()
            .map(Emoji::clone))
    }
    async fn get_emojis(&self, ctx: &Context, guild_id: &GuildId) -> Result<Vec<Emoji>, Error> {
        if let Some(emojis) = self.cache.get(&guild_id.get()) {
            return Ok(emojis);
        } else {
            let emojis = guild_id.emojis(ctx).await.context(DiscordSnafu)?;
            self.cache.insert(guild_id.get(), emojis.clone());
            return Ok(emojis);
        }
    }
}
