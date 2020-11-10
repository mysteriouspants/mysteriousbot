//! Role Wizard is a lovely fellow who sits behind a large desk. The poor old
//! man is nearly blind, but loves his job for the clerical order of it all. He
//! delights in serving the requests of iDevGames users, granting and revoking
//! public roles on the whims of the requester. Tireless and of infinite
//! patience, he's impossible to catch flustered, and he never sleeps.

use crate::mysterious_message_handler::{MMHResult, MysteriousMessageHandler};
use lazy_static::lazy_static;
use regex::Regex;
use serenity::model::channel::Message;
use serenity::model::guild::{Guild, Role};
use serenity::prelude::Context;

lazy_static! {
    // unwrap is okay here because we've already validated the regex
    /// The way this regex works is that it structures the input to two groups
    /// group 1: the command (either grant or revoke)
    /// group 2: the role
    static ref ROLE_REGEX: Regex = Regex::new(
        "<@!\\d+> role (grant|revoke) (.*)"
    ).unwrap();
}

enum CommandMode {
    Grant,
    Revoke,
}

trait FindRoleByNameIgnoreCase {
    fn find_role_by_name_ignore_case(&self, role_name: &str) -> Option<&Role>;
}

/// The Wizard's configuration!
pub struct RoleWizard {
    /// Roles that the Wizard is allowed to grant on behalf of users.
    allowed_role_grants: Vec<String>,

    /// Roles that the Wizard is allowed to revoke on behalf of users.
    allowed_role_revoke: Vec<String>,
}

impl RoleWizard {
    pub fn new(allowed_role_grants: Vec<String>, allowed_role_revoke: Vec<String>) -> RoleWizard {
        RoleWizard {
            allowed_role_grants,
            allowed_role_revoke,
        }
    }
}

#[async_trait::async_trait]
impl MysteriousMessageHandler for RoleWizard {
    fn is_exclusive(&self) -> bool {
        true
    }

    async fn should_handle(&self, ctx: &Context, msg: &Message) -> MMHResult<bool> {
        return Ok(msg.mentions_me(&ctx).await? && ROLE_REGEX.is_match(&msg.content.to_lowercase()));
    }

    async fn on_message(&self, ctx: &Context, msg: &Message) -> MMHResult<()> {
        let content = msg.content.to_lowercase();

        if let Some(captures) = ROLE_REGEX.captures(&content) {
            // unwrap is okay here because the regex has already enforced that
            // the command will be one of the two strings, grant or revoke.
            let command = match captures.get(1).unwrap().as_str() {
                "grant" => Some(CommandMode::Grant),
                "revoke" => Some(CommandMode::Revoke),
                _ => None,
            }
            .unwrap();
            let role = captures.get(2).unwrap().as_str();
            let guild = msg.guild(&ctx.cache).await.unwrap();

            let role_id = match guild.find_role_by_name_ignore_case(role) {
                Some(role) => role.id,
                None => {
                    msg.channel_id
                        .say(&ctx.http, "No such role by that name, bud.")
                        .await?;

                    return Ok(());
                }
            };

            // unwrap is okay here because the regex only allows the two strings
            let allow_list = match command {
                CommandMode::Grant => &self.allowed_role_grants,
                CommandMode::Revoke => &self.allowed_role_revoke,
            };

            if !allow_list.contains(&role.to_owned()) {
                msg.channel_id
                    .say(&ctx.http, "I'm sorry, I cannot manage that role")
                    .await?;

                return Ok(());
            }

            let mut member = msg.member(&ctx).await?;

            let result = match command {
                CommandMode::Grant => member.add_role(&ctx.http, role_id).await,
                CommandMode::Revoke => member.remove_role(&ctx.http, role_id).await,
            };

            match result {
                Ok(()) => {
                    msg.channel_id.say(&ctx.http, "you got it.").await?;
                }
                Err(e) => {
                    msg.channel_id
                        .say(&ctx.http, "there was a problem modifying your roles.")
                        .await?;
                    println!("Could not modify role {} with error {:?}", role, e)
                }
            }
        } else {
            msg.channel_id
                .say(
                    &ctx.http,
                    "The format for this command is role <grant|revoke> <role name>",
                )
                .await?;
        }

        Ok(())
    }
}

impl FindRoleByNameIgnoreCase for Guild {
    fn find_role_by_name_ignore_case(&self, role_name: &str) -> Option<&Role> {
        let roles = self.roles.values();

        for role in roles {
            if role.name.eq_ignore_ascii_case(role_name) {
                return Some(role);
            }
        }

        return None;
    }
}
