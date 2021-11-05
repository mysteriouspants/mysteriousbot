use caches::{Cache, AdaptiveCache};
use std::{time::{Duration, Instant}, sync::Arc};
use thiserror::Error;
use serenity::{client::Context, model::{guild::Emoji, id::GuildId}};
use serenity::Error as DiscordError;
use parking_lot::RwLock;

#[derive(Debug, Error)]
pub enum GetEmojiError {
  #[error("Failed to call Discord with error {0:?}")]
  Discord(#[from] DiscordError),
}

struct CacheValue {
    emojis: Vec<Emoji>,
    inserted_at: Instant,
}

/// An LRU Cache which holds the emojis - cached because hitting the
/// Discord emoji API potentially on every event sounds like a bad time.
pub struct EmojiCache {
    cache: Arc<RwLock<AdaptiveCache<GuildId, CacheValue>>>,
    timeout: Duration,
}

impl EmojiCache {
    pub fn new() -> Self {
        let cache: AdaptiveCache<GuildId, CacheValue> = AdaptiveCache::new(5).unwrap();
        Self {
            cache: Arc::from(RwLock::from(cache)),
            timeout: Duration::from_secs(24 * 60 * 60),
        }
    }
    pub async fn get_emoji(&self, ctx: &Context, guild_id: &GuildId, twemoji: &str) -> Result<Option<Emoji>, GetEmojiError> {
      let emojis = self.get_emojis(ctx, guild_id).await?;
      Ok(emojis.iter().filter(|emoji| {
        emoji.name == twemoji
      }).next().map(Emoji::clone))
    }

    async fn get_emojis(&self, ctx: &Context, guild_id: &GuildId) -> Result<Vec<Emoji>, GetEmojiError> {
        // look up stored emojis
        if let Some(value) = self.cache.write().get(guild_id) {
            if value.inserted_at.elapsed() > self.timeout {
                return Ok(value.emojis.clone());
            }
        }

        // try to load new emojis
        let emojis = guild_id.emojis(ctx).await?;

        self.cache.write().put(*guild_id, CacheValue {
            emojis: emojis.clone(),
            inserted_at: Instant::now(),
        });

        Ok(emojis)
    }
}
