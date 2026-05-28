# VoltEnv

## Tech Stack
- **Runtime:** Bun
- **Backend:** Tauri v2 (Rust, custom commands + plugins)
- **Frontend:** Nuxt 4 (SSR: false, `app/` dir), Nuxt UI v4, Pinia, VueUse
- **Tools:** ESLint + stylistic, `cargo fmt` + `cargo clippy`

## Deprecated — Do Not Use
- Tauri v1 APIs (window.create, shell.open, etc.)
- Node built-in packages (`fs`, `path`, `process`)
- `axios`, `node-fetch` — use `ofetch` or `@tauri-apps/plugin-http`

## Rules
1. **Core logic lives in Rust.** Download, extract, spawn, kill, and health-check all run through custom Tauri commands (`src-tauri/src/modules/commands.rs`, `services.rs`). The frontend store is a thin `invoke()` wrapper.
2. **Frontend tooling:** Use [unjs.io](https://unjs.io) ecosystem first (consola, ofetch, citty, etc.)
3. **Rust:** Run `cargo fmt` then `cargo clippy` before committing
4. **Frontend:** Run `cd src-frontend && bun run typecheck && bun run lint:fix` before committing
5. **Process management:** Use `tauri-plugin-shell` `CommandChild` in Rust via `ServiceProcesses`. The `start_service` command sets up a real-time `CommandEvent::Terminated` listener via `rx.recv().await` — no polling needed. The frontend reacts to `service-status-changed` events.
6. **Nuxt UI:** Primary `green`, neutral `zinc`; see `.agents/skills/nuxt-ui/SKILL.md`
7. **Capabilities:** New Tauri plugin needs permission entry in `src-tauri/capabilities/default.json`
