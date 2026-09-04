//! Santity VoiceMaster Plugin
//!
//! Join-to-Create dynamic temporary voice channel provisioning and garbage collection
//! with an automated periodic reconciliation timer to prune orphaned ghost rooms after restarts.

use santity_pdk::prelude::*;
use santity_pdk::{BrandEmbed, Guild};

struct SantityVoiceMasterPlugin;

fn track_temporary_channel(guild_id: &str, channel_id: &str) {
    let mut channels: Vec<String> = storage::get("active_temp_channels").ok().flatten().unwrap_or_default();
    if !channels.contains(&channel_id.to_string()) {
        channels.push(channel_id.to_string());
        let _ = storage::set("active_temp_channels", &channels);
    }
    let _ = storage::set(&format!("room_guild:{}", channel_id), &guild_id.to_string());
    let _ = storage::set(&format!("room_members:{}", channel_id), &1u64);
}

#[plugin(
    name = "santity_voicemaster",
    version = "0.2.0",
    events(voice_state_update),
    capabilities("discord:manage_channels")
)]
impl SantityVoiceMasterPlugin {
    #[command(
        name = "voicemaster_setup",
        description = "Configure Join-to-Create dynamic voice master generator",
        options(channel_name = "String")
    )]
    fn setup(event: InteractionEvent) -> ResponseAction {
        let name = event.get_str("channel_name").unwrap_or_else(|| "Join to Create".to_string());
        let guild_id = event.guild_id.clone().unwrap_or_default();
        let guild = Guild::new(guild_id.clone());

        match guild.create_voice_channel_typed(&name, None, Some(64000)) {
            Ok(channel) => {
                info!("Voice generator channel '{}' (ID: {}) provisioned", channel.name, channel.id);
                let _ = storage::set("generator_active", &true);
                let _ = storage::set("generator_channel_id", &channel.id);

                BrandEmbed::success(
                    "VoiceMaster Initialized",
                    format!("Join-to-Create hub `{}` (`{}`) is now operational.", channel.name, channel.id)
                )
                .ephemeral(true)
                .into_action()
            }
            Err(err) => {
                BrandEmbed::error(
                    "VoiceMaster Setup Failed",
                    err.display_message()
                )
                .into_action()
            }
        }
    }

    #[command(
        name = "voicemaster_status",
        description = "Check active temporary voice channel metrics"
    )]
    fn status(_event: InteractionEvent) -> ResponseAction {
        let is_active: bool = storage::get("generator_active").ok().flatten().unwrap_or(false);
        let active_channels: Vec<String> = storage::get("active_temp_channels").ok().flatten().unwrap_or_default();
        let total_created: u64 = storage::get("temp_channels_created").ok().flatten().unwrap_or(0);

        BrandEmbed::new()
            .title("[VOICE // METRICS] VOICEMASTER")
            .description(if is_active { "DYNAMIC PROVISIONER ACTIVE" } else { "NOT CONFIGURED" })
            .field("ACTIVE ROOMS", format!("`{}` currently live", active_channels.len()), true)
            .field("TOTAL PROVISIONED", format!("`{}` lifetime", total_created), true)
            .field("ORPHAN RECONCILER", "Active (60s tick)", true)
            .ephemeral(true)
            .into_action()
    }

    #[command(
        name = "voicemaster_prune",
        description = "Manually run orphan reconciliation to clean up abandoned voice channels"
    )]
    fn manual_prune(event: InteractionEvent) -> ResponseAction {
        let guild_id = event.guild_id.clone().unwrap_or_default();
        let active_channels: Vec<String> = storage::get("active_temp_channels").ok().flatten().unwrap_or_default();
        let count = active_channels.len();

        let guild = Guild::new(guild_id);
        for ch_id in &active_channels {
            let _ = guild.delete_channel(ch_id, Some("VoiceMaster: Manual orphan cleanup"));
        }
        let _ = storage::set("active_temp_channels", &Vec::<String>::new());

        BrandEmbed::success(
            "Reconciliation Complete",
            format!("Pruned `{}` tracked dynamic voice channels.", count)
        )
        .ephemeral(true)
        .into_action()
    }

    #[event(voice_state_update)]
    fn on_voice_state(payload_json: String) -> ResponseAction {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&payload_json) {
            let channel_id = data.get("channel_id").and_then(|v| v.as_str());
            let user_id = data.get("user_id").and_then(|v| v.as_str()).unwrap_or_default();
            let guild_id = data.get("guild_id").and_then(|v| v.as_str()).unwrap_or_default();

            let gen_channel_id: String = storage::get("generator_channel_id").ok().flatten().unwrap_or_default();

            if let Some(ch) = channel_id {
                if !gen_channel_id.is_empty() && ch == gen_channel_id && !guild_id.is_empty() {
                    info!("User {} joined VoiceMaster generator channel. Creating temporary room...", user_id);
                    let guild = Guild::new(guild_id.to_string());
                    let room_name = format!("Channel-{}", user_id);
                    if let Ok(temp_ch) = guild.create_voice_channel_typed(&room_name, None, Some(64000)) {
                        track_temporary_channel(guild_id, &temp_ch.id);
                        let total: u64 = storage::get("temp_channels_created").ok().flatten().unwrap_or(0) + 1;
                        let _ = storage::set("temp_channels_created", &total);
                    }
                }
            }
        }
        ResponseAction::None
    }

    #[timer("voice_orphan_reconciliation")]
    fn reconcile_orphans() {
        let active_channels: Vec<String> = storage::get("active_temp_channels").ok().flatten().unwrap_or_default();
        if active_channels.is_empty() {
            return;
        }

        info!("Running voice orphan reconciliation over {} tracked rooms", active_channels.len());
        let mut remaining = Vec::new();
        for ch_id in active_channels {
            let member_count: u64 = storage::get(&format!("room_members:{}", ch_id)).ok().flatten().unwrap_or(0);
            if member_count == 0 {
                info!("Pruning orphaned temporary voice channel {}", ch_id);
                let guild_id: String = storage::get(&format!("room_guild:{}", ch_id)).ok().flatten().unwrap_or_default();
                let guild = Guild::new(guild_id);
                let _ = guild.delete_channel(&ch_id, Some("VoiceMaster: Pruning orphaned channel"));
                let _ = storage::set(&format!("room_guild:{}", ch_id), &"");
            } else {
                remaining.push(ch_id);
            }
        }
        let _ = storage::set("active_temp_channels", &remaining);
    }
}
