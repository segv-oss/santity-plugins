//! Buttons Demo: showcases Phase 1 rich messaging.
//!
//! - `/panel` posts an embed with three color buttons (public message)
//! - Clicking a button answers with an ephemeral confirmation only the
//!   clicking user can see
//!
//! Component custom_ids follow the routing convention "{plugin_name}:..." so
//! santity-core can target this plugin's actor directly.

use santity_pdk::message::{ActionRow, Button, Embed, OutgoingMessage};
use santity_pdk::prelude::*;

const PLUGIN: &str = "buttons_demo";

fn role_button(label: &'static str, role: &'static str) -> Button {
    Button::primary(label, format!("{PLUGIN}:role:{role}"))
}

#[allow(dead_code)]
struct ButtonsDemo;

#[allow(dead_code)]
#[plugin(
    name = "buttons_demo",
    version = "0.1.0",
    author = "SEGv",
    description = "Rich messaging demo panel with role buttons"
)]
impl ButtonsDemo {
    #[command(name = "panel", description = "Post the interactive color-role panel")]
    fn panel(_event: InteractionEvent) -> ResponseAction {
        info!("Posting interactive role panel");
        OutgoingMessage::new()
            .content("Grab a color below!")
            .embed(
                Embed::new()
                    .title("Color Roles")
                    .description("Click a button to pick your favorite color.")
                    .color(0x5865_F2)
                    .field("Red", "Passion & energy", true)
                    .field("Green", "Growth & harmony", true)
                    .field("Blue", "Calm & trust", true)
                    .footer("Buttons demo powered by Santity"),
            )
            .row(ActionRow::new()
                .push(role_button("Red", "red"))
                .push(role_button("Green", "green"))
                .push(role_button("Blue", "blue")))
            .into_action()
    }

    /// Receives every component interaction whose custom_id starts with
    /// "{PLUGIN}:". Answers ephemerally so only the clicker sees it.
    #[component]
    fn on_component(event: InteractionEvent) -> ResponseAction {
        let custom_id = event.custom_id.unwrap_or_default();
        let choice = custom_id.rsplit(':').next().unwrap_or("unknown");
        let (emoji, blurb): (&str, &str) = match choice {
            "red" => ("🔴", "Passion & energy"),
            "green" => ("🟢", "Growth & harmony"),
            "blue" => ("🔵", "Calm & trust"),
            other => {
                warn!("Unknown component custom_id: {other}");
                return ResponseAction::none();
            }
        };
        info!("{} clicked the {choice} button", event.user.username);
        OutgoingMessage::new()
            .content(format!("{emoji} You picked **{choice}** — {blurb}!"))
            .ephemeral()
            .into_action()
    }

    #[message]
    fn on_msg(event: MessageEvent) -> ResponseAction {
        if event.content.trim() == "!panel" {
            Self::panel(InteractionEvent::default())
        } else {
            ResponseAction::none()
        }
    }
}
