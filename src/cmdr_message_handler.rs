use serenity::model::channel::Message;
use serenity::prelude::Context;


pub trait CmdrMessageHandler : Send + Sync {
    /// Tells whether this event handler is "exclusive," which is to say that,
    /// when an event matches this handler, and is handled by this handler,
    /// ought the event continue to propagate through the handler chain, or
    /// ought it to stop?
    fn is_exclusive(&self) -> bool {
        false
    }

    /// Whether or not the message should be handled by this handler.
    fn should_handle(&self, ctx: &Context, msg: &Message) -> bool;

    /// Actually handle the message.
    fn on_message(&self, ctx: &Context, msg: &Message);
}
