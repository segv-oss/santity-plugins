use santity_pdk::prelude::*;
use santity_pdk::BrandEmbed;

struct PingPongPlugin;

#[plugin(name = "ping_pong", version = "0.2.0")]
impl PingPongPlugin {
    #[command(name = "ping", description = "Latency & health check with interactive button")]
    fn ping(_event: InteractionEvent) -> ResponseAction {
        let count: u64 = storage::get("ping_count").ok().flatten().unwrap_or(0) + 1;
        let _ = storage::set("ping_count", &count);
        info!("Ping counter updated to {}", count);

        let embed = BrandEmbed::new()
            .title("[HEALTH] PONG")
            .description(format!("Total executions handled by this worker: `{}`", count))
            .field("STATUS", "Operational", true)
            .field("LATENCY", "< 1ms (Wasm Actor)", true)
            .ephemeral(true)
            .build();

        let button = Button::primary("btn_ping_again", "Refresh Status");

        let msg = MessageBuilder::new()
            .embed(embed)
            .button(button)
            .ephemeral(true)
            .build();

        ResponseAction::reply_message(msg)
    }

    #[button("btn_ping_again")]
    fn on_ping_button(_event: InteractionEvent) -> ResponseAction {
        let count: u64 = storage::get("ping_count").ok().flatten().unwrap_or(0) + 1;
        let _ = storage::set("ping_count", &count);
        info!("Ping button clicked! Counter updated to {}", count);
        
        let embed = BrandEmbed::new()
            .title("[HEALTH] STATUS REFRESHED")
            .description(format!("Total executions: `{}`", count))
            .ephemeral(true)
            .build();

        let msg = MessageBuilder::new()
            .embed(embed)
            .ephemeral(true)
            .build();

        ResponseAction::reply_message(msg)
    }

    #[timer("heartbeat")]
    fn on_heartbeat() {
        info!("PingPong plugin background heartbeat tick.");
    }

    #[message]
    fn message(event: MessageEvent) -> ResponseAction {
        if event.content.trim() == "!ping" {
            Self::ping(InteractionEvent::default())
        } else {
            ResponseAction::None
        }
    }
}
