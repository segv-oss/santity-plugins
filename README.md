# santity-plugins

Official collection and automated CI/CD build repository for WebAssembly plugins compatible with the Santity runtime engine.

---

## Plugin Installation via the `santity` CLI

`santity-plugins` uses CI/CD pipelines to compile plugins into WebAssembly Component Model binaries (`.component.wasm`) and publish them directly to releases:

```bash
# 1. Start Santity daemon
santity up

# 2. Install plugin directly from release binary or local build
santity plugin add https://github.com/segv-oss/santity-plugins/releases/latest/download/santity_guard.component.wasm

# 3. Monitor live dashboard
santity ui
```

---

## Official Plugins Registry

| Plugin | Capabilities | Description |
|:---|:---|:---|
| **`ping_pong`** | None | Health and latency probe with interactive button refresh and persistent counter storage |
| **`santity_mod`** | `ban_members`, `kick_members`, `moderate_members` | Enterprise moderation actions, reason logging, and atomic sequential case ID tracking |
| **`santity_guard`** | `ban_members`, `kick_members`, `manage_channels`, `view_audit_log` | Anti-nuke and anti-raid defense engine with sliding-window audit log caching |
| **`santity_tickets`** | `manage_channels`, `send_messages` | Dynamic ticket intake panels, modal submission forms, and private channel dispatch |
| **`santity_roles`** | `manage_roles` | Interactive self-assignable button role panels with atomic toggle state |
| **`santity_voicemaster`** | `manage_channels` | Join-to-Create dynamic temporary voice room generator and automatic ghost channel garbage collector |

---

## Building Locally

### Prerequisites
- Rust stable toolchain
- Target: `rustup target add wasm32-unknown-unknown`
- `cargo install cargo-component`

### Build All Plugins
```bash
cargo build --workspace --target wasm32-unknown-unknown --release
```

---

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE).
