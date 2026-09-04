//! Santity Roles Plugin
//!
//! Interactive self-assignable button roles with atomic toggle state.

use santity_pdk::prelude::*;
use santity_pdk::{BrandEmbed, Guild};

struct SantityRolesPlugin;

#[plugin(
    name = "santity_roles",
    version = "0.2.0",
    capabilities("discord:manage_roles")
)]
impl SantityRolesPlugin {
    #[command(
        name = "role_panel",
        description = "Create a self-assignable role selection button panel",
        options(title = "String", role_id = "String", description = "String")
    )]
    fn role_panel(event: InteractionEvent) -> ResponseAction {
        let title = event.get_str("title").unwrap_or_else(|| "Role Assignment".to_string());
        let role_id = match event.get_str("role_id") {
            Some(r) => r,
            None => return BrandEmbed::error("Invalid Role", "Role ID is required.").into_action(),
        };
        let desc = event.get_str("description").unwrap_or_else(|| "Click below to toggle this role on your profile.".to_string());

        // Store role binding in storage
        let key = format!("panel_role:{}", role_id);
        let _ = storage::set(&key, &role_id);

        let embed = BrandEmbed::new()
            .title(format!("[ROLES] {}", title.to_uppercase()))
            .description(format!("{}\n\n**Bound Role:** <@&{}>", desc, role_id))
            .build();

        let button_id = format!("btn_role_{}", role_id);
        let button = Button::primary(button_id, "Toggle Role");

        let msg = MessageBuilder::new()
            .embed(embed)
            .button(button)
            .ephemeral(false)
            .build();

        ResponseAction::reply_message(msg)
    }

    #[button("btn_role_")]
    fn on_role_button(event: InteractionEvent) -> ResponseAction {
        let custom_id = event.custom_id.clone().unwrap_or_default();
        let role_id = custom_id.trim_start_matches("santity_roles:").trim_start_matches("btn_role_");

        let user_id = event.user.id.clone();
        let guild_id = event.guild_id.clone().unwrap_or_default();
        let guild = Guild::new(guild_id);

        let user_role_key = format!("user_has_role:{}:{}", user_id, role_id);
        let has_role: bool = storage::get(&user_role_key).ok().flatten().unwrap_or(false);

        if has_role {
            match guild.remove_role(&user_id, role_id, Some("Self-assigned role removed")) {
                Ok(_) => {
                    let _ = storage::set(&user_role_key, &false);
                    BrandEmbed::info(
                        "Role Removed",
                        format!("The role <@&{}> was removed from your profile.", role_id)
                    )
                    .ephemeral(true)
                    .into_action()
                }
                Err(err) => BrandEmbed::error("Role Update Failed", err.display_message()).into_action(),
            }
        } else {
            match guild.add_role(&user_id, role_id, Some("Self-assigned role added")) {
                Ok(_) => {
                    let _ = storage::set(&user_role_key, &true);
                    BrandEmbed::success(
                        "Role Granted",
                        format!("The role <@&{}> was assigned to your profile.", role_id)
                    )
                    .ephemeral(true)
                    .into_action()
                }
                Err(err) => BrandEmbed::error("Role Update Failed", err.display_message()).into_action(),
            }
        }
    }
}
