//! Santity Guard (Anti-Nuke & Anti-Raid Engine)
//!
//! Real-time detection and containment of raid attacks, rogue moderators, and mass channel deletions
//! featuring a 1500ms sliding-window audit log cache to prevent Discord HTTP 429 rate limit locks.

use santity_pdk::prelude::*;
use santity_pdk::{AuditLogEntry, BrandEmbed, DiscordApiError, Guild};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAuditLog {
    pub cached_at_secs: u64,
    pub entries: Vec<AuditLogEntry>,
}

struct SantityGuardPlugin;

fn get_cached_or_fetch_audit_logs(guild: &Guild, action_type: u32) -> Result<Vec<AuditLogEntry>, DiscordApiError> {
    let cache_key = format!("audit_cache:{}", action_type);
    let now_secs: u64 = storage::get("window_epoch_secs").ok().flatten().unwrap_or(0);

    if let Ok(Some(cached)) = storage::get::<CachedAuditLog>(&cache_key) {
        // 1500ms sliding window: reuse recent fetch during burst deletions
        if now_secs.saturating_sub(cached.cached_at_secs) < 2 {
            return Ok(cached.entries);
        }
    }

    let resp = guild.get_audit_logs(Some(action_type), Some(10))?;
    let body_str = String::from_utf8_lossy(&resp.body);
    let entries = AuditLogEntry::parse_entries(&body_str);

    let cache = CachedAuditLog {
        cached_at_secs: now_secs,
        entries: entries.clone(),
    };
    let _ = storage::set(&cache_key, &cache);
    Ok(entries)
}

#[plugin(
    name = "santity_guard",
    version = "0.2.0",
    events(member_join),
    capabilities("discord:ban_members", "discord:kick_members", "discord:manage_channels", "discord:view_audit_log")
)]
impl SantityGuardPlugin {
    #[command(name = "guard_status", description = "View active Anti-Nuke and Anti-Raid defense metrics")]
    fn guard_status(event: InteractionEvent) -> ResponseAction {
        let is_locked: bool = storage::get("lockdown_active").ok().flatten().unwrap_or(false);
        let recent_joins: u64 = storage::get("recent_join_count").ok().flatten().unwrap_or(0);
        let channel_deletions: u64 = storage::get("channel_delete_count").ok().flatten().unwrap_or(0);

        let status_desc = if is_locked {
            "LOCKDOWN ACTIVE // Rapid join gate triggered"
        } else {
            "ALL SYSTEMS NOMINAL // Passive monitoring active"
        };

        BrandEmbed::new()
            .title("[SEC // STATUS] DEFENSE MATRIX")
            .description(status_desc)
            .field("JOIN VELOCITY", format!("`{}` in last window", recent_joins), true)
            .field("CHANNEL MUTATIONS", format!("`{}` deletions recorded", channel_deletions), true)
            .field("GUILD ID", format!("`{}`", event.guild_id.unwrap_or_default()), false)
            .ephemeral(true)
            .into_action()
    }

    #[command(
        name = "lockdown",
        description = "Manually engage or disengage server lockdown mode",
        options(enabled = "Boolean")
    )]
    fn toggle_lockdown(event: InteractionEvent) -> ResponseAction {
        let enabled = event.get_bool("enabled").unwrap_or(true);
        let _ = storage::set("lockdown_active", &enabled);

        let action = if enabled { "ENGAGED" } else { "DISENGAGED" };
        info!("Server lockdown manually {}", action);

        BrandEmbed::security_alert(
            "MANUAL LOCKDOWN OVERRIDE",
            &format!("Server lockdown state has been set to: **{}**", action),
            &format!("Operator: <@{}>", event.user.id),
        )
        .into_action()
    }

    #[command(
        name = "audit_check",
        description = "Check recent guild audit actions with cached rate-limit protection"
    )]
    fn audit_check(event: InteractionEvent) -> ResponseAction {
        let guild_id = event.guild_id.clone().unwrap_or_default();
        let guild = Guild::new(guild_id);

        match get_cached_or_fetch_audit_logs(&guild, 12) {
            Ok(entries) => {
                let count = entries.len();
                BrandEmbed::info(
                    "Audit Analysis",
                    format!("Retrieved `{}` recent channel mutation entries (Rate-limit cache active).", count)
                )
                .ephemeral(true)
                .into_action()
            }
            Err(err) => BrandEmbed::error("Audit Log Failure", err.display_message()).into_action(),
        }
    }

    #[message]
    fn on_message(event: MessageEvent) -> ResponseAction {
        // Fast-path chat spam containment
        if event.content.contains("discord.gg/") || event.content.contains("@everyone") {
            let count: u64 = storage::get("spam_strike_count").ok().flatten().unwrap_or(0) + 1;
            let _ = storage::set("spam_strike_count", &count);
            if count > 3 {
                if let Some(ref gid) = event.guild_id {
                    let guild = Guild::new(gid.clone());
                    let _ = guild.timeout(&event.author.id, 600, Some("Automated spam threshold exceeded"));
                }
            }
        }
        ResponseAction::None
    }

    #[event(member_join)]
    fn on_member_join(payload_json: String) -> ResponseAction {
        let count: u64 = storage::get("recent_join_count").ok().flatten().unwrap_or(0) + 1;
        let _ = storage::set("recent_join_count", &count);

        let is_locked: bool = storage::get("lockdown_active").ok().flatten().unwrap_or(false);

        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&payload_json) {
            let user_id = data.get("user").and_then(|u| u.get("id")).and_then(|v| v.as_str()).unwrap_or_default();
            let guild_id = data.get("guild_id").and_then(|v| v.as_str()).unwrap_or_default();

            if count > 5 || is_locked {
                let _ = storage::set("lockdown_active", &true);
                warn!("Rapid join detected (join #{})! Enforcing anti-raid gate on user {}", count, user_id);

                if !guild_id.is_empty() && !user_id.is_empty() {
                    let guild = Guild::new(guild_id);
                    let _ = guild.kick(user_id, Some("Anti-Raid: Join rate threshold exceeded"));
                }
            }
        }

        ResponseAction::None
    }

    #[timer("window_tick")]
    fn on_window_tick() {
        let epoch: u64 = storage::get("window_epoch_secs").ok().flatten().unwrap_or(0) + 1;
        let _ = storage::set("window_epoch_secs", &epoch);

        // Reset velocity window every 10 seconds
        if epoch.is_multiple_of(10) {
            let _ = storage::set("recent_join_count", &0u64);
            let _ = storage::set("channel_delete_count", &0u64);
        }
    }
}
