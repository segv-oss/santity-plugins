# santity-plugins

Official collection and automated CI/CD build repository for WebAssembly plugins compatible with the [Santity](https://github.com/segv-oss/santity-core) runtime engine.

---

## 🚀 One-Line Plugin Installation via the `santity` CLI

`santity-plugins` uses GitHub Actions CI/CD to automatically compile every plugin into WebAssembly Component Model binaries (`.component.wasm`) and publish them directly to GitHub Releases.

You can install official compiled plugins directly from GitHub Releases into your live Santity engine with one command:

```bash
# 1. Start Santity daemon
santity up

# 2. Install plugin directly from GitHub Release URL
santity plugin add https://github.com/segv-oss/santity-plugins/releases/latest/download/ping_pong.component.wasm

# 3. Monitor live dashboard
santity ui
```

---

## 📦 Official Plugins Registry

| Plugin | Description | Release Binary |
| :--- | :--- | :--- |
| **`ping_pong`** | Ping-pong plugin featuring persistent counter storage, logging, and slash commands. | `ping_pong.component.wasm` |

---

## 🛠️ Building & Releasing Plugins (CI/CD)

Whenever a version tag (e.g. `v0.1.0`) is pushed to `santity-plugins`, GitHub Actions automatically:
1. Compiles Rust guest code to `wasm32-unknown-unknown`.
2. Packages the binary using `wasm-tools component new`.
3. Uploads the final `${plugin_name}.component.wasm` asset to GitHub Releases.

To build locally:
```bash
santity build --release
```

---

## 📜 License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE).
