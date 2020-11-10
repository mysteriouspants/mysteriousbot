use serenity::model::channel::Message;
use serenity::prelude::Context;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MMError {
    #[error("Serenity returned an error {0} from Discord.")]
    SerenityError(#[from] serenity::Error),
}

pub type MMHResult<T> = Result<T, MMError>;

#[async_trait::async_trait]
pub trait MysteriousMessageHandler: Send + Sync {
    /// Tells whether this event handler is "exclusive," which is to say that,
    /// when an event matches this handler, and is handled by this handler,
    /// ought the event continue to propagate through the handler chain, or
    /// ought it to stop?
    fn is_exclusive(&self) -> bool {
        false
    }

    /// Whether or not the message should be handled by this handler.
    async fn should_handle(&self, ctx: &Context, msg: &Message) -> MMHResult<bool>;

    /// Actually handle the message.
    async fn on_message(&self, ctx: &Context, msg: &Message) -> MMHResult<()>;
}
