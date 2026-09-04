//! Santity Moderation Engine Plugin
//!
//! Provides high-assurance moderation actions, audit trails, and concurrency-safe case tracking.

use santity_pdk::prelude::*;
use santity_pdk::{BrandColor, BrandEmbed, Guild};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModCase {
    pub id: u64,
    pub action: String,
    pub target_id: String,
    pub moderator_id: String,
    pub reason: String,
}

struct SantityModPlugin;

fn next_case_id() -> u64 {
    storage::next_id("case_counter").unwrap_or(1)
}

fn log_case(case: &ModCase) {
    let key = format!("case:{}", case.id);
    let _ = storage::set(&key, case);

    let user_cases_key = format!("user_cases:{}", case.target_id);
    let mut cases: Vec<u64> = storage::get(&user_cases_key).ok().flatten().unwrap_or_default();
    cases.push(case.id);
    let _ = storage::set(&user_cases_key, &cases);
}

#[plugin(
    name = "santity_mod",
    version = "0.2.0",
    capabilities("discord:ban_members", "discord:kick_members", "discord:moderate_members")
)]
impl SantityModPlugin {
    #[command(
        name = "ban",
        description = "Ban a user from the server with optional message deletion",
        options(user = "User", reason = "String", delete_days = "Integer")
    )]
    fn ban(event: InteractionEvent) -> ResponseAction {
        let user_id = match event.get_user_id("user") {
            Some(u) => u,
            None => return BrandEmbed::error("Invalid Target", "Target user ID is missing.").into_action(),
        };
        let reason = event.get_str("reason").unwrap_or_else(|| "No reason specified".to_string());
        let delete_days = event.get_int("delete_days").unwrap_or(0) as u32;
        let delete_secs = delete_days * 86400;

        let guild_id = event.guild_id.clone().unwrap_or_default();
        let guild = Guild::new(guild_id);

        match guild.ban(&user_id, Some(delete_secs), Some(&reason)) {
            Ok(_) => {
                let case_id = next_case_id();
                let case = ModCase {
                    id: case_id,
                    action: "BAN".to_string(),
                    target_id: user_id.clone(),
                    moderator_id: event.user.id.clone(),
                    reason: reason.clone(),
                };
                log_case(&case);

                BrandEmbed::mod_action("BAN", &user_id, &event.user.id, &reason, Some(case_id))
                    .into_action()
            }
            Err(err) => {
                error!("Failed to execute ban: {:?}", err);
                BrandEmbed::error("Ban Failed", err.display_message()).into_action()
            }
        }
    }

    #[command(
        name = "kick",
        description = "Kick a user from the server",
        options(user = "User", reason = "String")
    )]
    fn kick(event: InteractionEvent) -> ResponseAction {
        let user_id = match event.get_user_id("user") {
            Some(u) => u,
            None => return BrandEmbed::error("Invalid Target", "Target user ID is missing.").into_action(),
        };
        let reason = event.get_str("reason").unwrap_or_else(|| "No reason specified".to_string());

        let guild_id = event.guild_id.clone().unwrap_or_default();
        let guild = Guild::new(guild_id);

        match guild.kick(&user_id, Some(&reason)) {
            Ok(_) => {
                let case_id = next_case_id();
                let case = ModCase {
                    id: case_id,
                    action: "KICK".to_string(),
                    target_id: user_id.clone(),
                    moderator_id: event.user.id.clone(),
                    reason: reason.clone(),
                };
                log_case(&case);

                BrandEmbed::mod_action("KICK", &user_id, &event.user.id, &reason, Some(case_id))
                    .into_action()
            }
            Err(err) => {
                error!("Failed to execute kick: {:?}", err);
                BrandEmbed::error("Kick Failed", err.display_message()).into_action()
            }
        }
    }

    #[command(
        name = "timeout",
        description = "Timeout a user for a specified duration in seconds",
        options(user = "User", duration_seconds = "Integer", reason = "String")
    )]
    fn timeout(event: InteractionEvent) -> ResponseAction {
        let user_id = match event.get_user_id("user") {
            Some(u) => u,
            None => return BrandEmbed::error("Invalid Target", "Target user ID is missing.").into_action(),
        };
        let duration = event.get_int("duration_seconds").unwrap_or(300) as u64;
        let reason = event.get_str("reason").unwrap_or_else(|| "No reason specified".to_string());

        let guild_id = event.guild_id.clone().unwrap_or_default();
        let guild = Guild::new(guild_id);

        match guild.timeout(&user_id, duration, Some(&reason)) {
            Ok(_) => {
                let case_id = next_case_id();
                let case = ModCase {
                    id: case_id,
                    action: format!("TIMEOUT ({}s)", duration),
                    target_id: user_id.clone(),
                    moderator_id: event.user.id.clone(),
                    reason: reason.clone(),
                };
                log_case(&case);

                BrandEmbed::mod_action(
                    &format!("TIMEOUT ({}s)", duration),
                    &user_id,
                    &event.user.id,
                    &reason,
                    Some(case_id),
                )
                .into_action()
            }
            Err(err) => {
                error!("Failed to execute timeout: {:?}", err);
                BrandEmbed::error("Timeout Failed", err.display_message()).into_action()
            }
        }
    }

    #[command(
        name = "warn",
        description = "Issue a formal warning to a user",
        options(user = "User", reason = "String")
    )]
    fn warn(event: InteractionEvent) -> ResponseAction {
        let user_id = match event.get_user_id("user") {
            Some(u) => u,
            None => return BrandEmbed::error("Invalid Target", "Target user ID is missing.").into_action(),
        };
        let reason = event.get_str("reason").unwrap_or_else(|| "No reason specified".to_string());

        let case_id = next_case_id();
        let case = ModCase {
            id: case_id,
            action: "WARN".to_string(),
            target_id: user_id.clone(),
            moderator_id: event.user.id.clone(),
            reason: reason.clone(),
        };
        log_case(&case);

        BrandEmbed::mod_action("WARN", &user_id, &event.user.id, &reason, Some(case_id))
            .color(BrandColor::WARNING)
            .into_action()
    }

    #[command(
        name = "cases",
        description = "Query recorded moderation history for a user",
        options(user = "User")
    )]
    fn cases(event: InteractionEvent) -> ResponseAction {
        let user_id = match event.get_user_id("user") {
            Some(u) => u,
            None => return BrandEmbed::error("Invalid Target", "Target user ID is missing.").into_action(),
        };

        let user_cases_key = format!("user_cases:{}", user_id);
        let case_ids: Vec<u64> = storage::get(&user_cases_key).ok().flatten().unwrap_or_default();

        if case_ids.is_empty() {
            return BrandEmbed::info(
                "Clean Record",
                format!("User `{}` has 0 recorded moderation violations.", user_id),
            )
            .into_action();
        }

        let mut embed = BrandEmbed::new()
            .color(BrandColor::PRIMARY)
            .title(format!("[MOD // CASES] RECORD FOR `{}`", user_id))
            .description(format!("Total recorded violations: **{}**", case_ids.len()));

        for id in case_ids.iter().take(5) {
            let key = format!("case:{}", id);
            if let Ok(Some(case)) = storage::get::<ModCase>(&key) {
                embed = embed.field(
                    format!("CASE #{:04} // {}", case.id, case.action),
                    format!("Moderator: <@{}>\nReason: > {}", case.moderator_id, case.reason),
                    false,
                );
            }
        }

        embed.into_action()
    }
}
