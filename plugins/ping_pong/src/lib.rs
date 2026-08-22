use santity_pdk::prelude::*;

#[allow(dead_code)]
struct PingPongPlugin;

#[allow(dead_code)]
#[plugin(name = "ping_pong", version = "0.1.0")]
impl PingPongPlugin {

    #[command(name = "ping", description = "Ping pong test command with persistent counter")]
    fn ping(_event: InteractionEvent) -> ResponseAction {
        let count: u64 = storage::get("ping_count").ok().flatten().unwrap_or(0) + 1;
        let _ = storage::set("ping_count", &count);
        info!("Ping counter updated to {}", count);
        ResponseAction::reply(format!("Pong! 🏓 (Total pings handled by this plugin: {})", count))
    }

    #[message]
    fn message(event: MessageEvent) -> ResponseAction {
        if event.content.trim() == "!ping" {
            Self::ping(InteractionEvent::default())
        } else {
            ResponseAction::none()
        }
    }
}
