use std::time::Duration;
use thiserror::Error;
use mysterious_cache::{ExpiringCache, SharedCache};
use serenity::{client::Context, model::{guild::Emoji, id::GuildId}};
use serenity::Error as DiscordError;

#[derive(Debug, Error)]
pub enum GetEmojiError {
  #[error("Failed to call Discord with error {0:?}")]
  Discord(#[from] DiscordError),
}

/// An LRU Cache which holds the emojis - cached because hitting the
/// Discord emoji API potentially on every event sounds like a bad time.
pub struct EmojiCache {
    cache: SharedCache<ExpiringCache<GuildId, Vec<Emoji>>, GuildId, Vec<Emoji>>,
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
    pub async fn get_emoji(&self, ctx: &Context, guild_id: &GuildId, twemoji: &str) -> Result<Option<Emoji>, GetEmojiError> {
      let emojis = self.get_emojis(ctx, guild_id).await?;
      Ok(emojis.iter().filter(|emoji| {
        emoji.name == twemoji
      }).next().map(Emoji::clone))
    }
    async fn get_emojis(&self, ctx: &Context, guild_id: &GuildId) -> Result<Vec<Emoji>, GetEmojiError> {
      if let Some(emojis) = self.cache.get(guild_id) {
        return Ok(emojis);
      } else {
        let emojis = guild_id.emojis(ctx).await?;
        self.cache.insert(*guild_id, emojis.clone());
        return Ok(emojis);
      }
    }
}
