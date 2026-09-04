//! Santity Tickets Plugin
//!
//! Enterprise ticket panels, modal intake forms, and automated lifecycle dispatch.

use santity_pdk::prelude::*;
use santity_pdk::{BrandEmbed, Guild};

struct SantityTicketsPlugin;

fn next_ticket_number() -> u64 {
    storage::next_id("ticket_counter").unwrap_or(1)
}

#[plugin(
    name = "santity_tickets",
    version = "0.2.0",
    capabilities("discord:manage_channels", "discord:send_messages")
)]
impl SantityTicketsPlugin {
    #[command(
        name = "ticket_panel",
        description = "Create an interactive support ticket intake panel in this channel",
        options(title = "String", description = "String")
    )]
    fn ticket_panel(event: InteractionEvent) -> ResponseAction {
        let title = event.get_str("title").unwrap_or_else(|| "Support & Inquiries".to_string());
        let desc = event.get_str("description").unwrap_or_else(|| "Click the button below to open a private ticket with our team.".to_string());

        let embed = BrandEmbed::ticket_panel(&title, &desc).build();
        let button = Button::primary("btn_create_ticket", "Open Ticket");

        let msg = MessageBuilder::new()
            .embed(embed)
            .button(button)
            .ephemeral(false) // Panel is posted publicly
            .build();

        ResponseAction::reply_message(msg)
    }

    #[button("btn_create_ticket")]
    fn on_create_ticket_btn(_event: InteractionEvent) -> ResponseAction {
        let modal = ModalBuilder::new("modal_ticket_intake", "Open Support Ticket")
            .text_input("input_subject", "Subject / Topic", TextInputStyle::Short)
            .text_input("input_details", "Please describe your issue", TextInputStyle::Paragraph)
            .build();

        ResponseAction::modal(modal)
    }

    #[modal_submit("modal_ticket_intake")]
    fn on_ticket_modal_submit(event: InteractionEvent) -> ResponseAction {
        let subject = event.get_field("input_subject").unwrap_or_else(|| "General Inquiry".to_string());
        let details = event.get_field("input_details").unwrap_or_else(|| "No details provided".to_string());

        let ticket_num = next_ticket_number();
        let channel_name = format!("ticket-{:04}", ticket_num);

        let guild_id = event.guild_id.clone().unwrap_or_default();
        let guild = Guild::new(guild_id);

        match guild.create_text_channel_typed(&channel_name, None, Some(&subject)) {
            Ok(channel) => {
                info!("Ticket channel {} (ID: {}) created successfully", channel.name, channel.id);

                BrandEmbed::success(
                    "Ticket Created",
                    format!("Your ticket `#`**{:04}** has been initialized in channel <#{}> (`{}`).\n\n**Subject:** {}\n**Details:** > {}", ticket_num, channel.id, channel.name, subject, details)
                )
                .ephemeral(true)
                .into_action()
            }
            Err(err) => {
                error!("Failed to create ticket channel: {:?}", err);
                BrandEmbed::error(
                    "Ticket Creation Failed",
                    err.display_message()
                )
                .into_action()
            }
        }
    }

    #[button("btn_close_ticket")]
    fn on_close_ticket(event: InteractionEvent) -> ResponseAction {
        let guild_id = event.guild_id.clone().unwrap_or_default();
        let channel_id = event.channel_id.clone();
        let guild = Guild::new(guild_id);

        let _ = guild.delete_channel(&channel_id, Some("Ticket closed by user"));

        BrandEmbed::info(
            "Ticket Closed",
            "This ticket channel has been scheduled for termination."
        )
        .ephemeral(true)
        .into_action()
    }
}
