# santity-plugins

Collection of official WebAssembly plugins for the [Santity](https://github.com/segv-oss/santity-core) runtime engine.

## Plugins Directory

- **`ping_pong`**: Example ping-pong plugin demonstrating persistent counter storage, message handlers, and slash command routing using `santity-pdk`.

## Building Plugins

Add the `wasm32-wasip1` target:
```bash
rustup target add wasm32-wasip1
```

Build all plugins in the workspace:
```bash
cargo build --target wasm32-wasip1 --release
```

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE).
