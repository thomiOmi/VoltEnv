# VoltEnv

## Tech Stack
- **Runtime:** Bun
- **Backend:** Tauri v2 (Rust, plugins only — no custom commands)
- **Frontend:** Nuxt 4 (SSR: false, `app/` dir), Nuxt UI v4, Pinia, VueUse
- **Tools:** ESLint + stylistic, `cargo fmt` + `cargo clippy`

## Deprecated — Do Not Use
- Tauri v1 APIs (window.create, shell.open, etc.)
- unjs `node`-only packages (nitro node-server preset, h3 `serveStatic`)
- Node built-in packages (`fs`, `path`, `process`)
- `axios`, `node-fetch` — use `@tauri-apps/plugin-http` or `ofetch`

## Rules
1. **Frontend tooling:** Use [unjs.io](https://unjs.io) ecosystem first (consola, ofetch, citty, etc.)
2. **Tauri backend:** Use Tauri plugins first — no custom `invoke_handler` commands
3. **Rust:** Run `cargo fmt` then `cargo clippy` before committing
4. **Frontend:** Run `cd src-frontend && bun run typecheck && bun run lint:fix` before committing
5. **Service logic:** Lives in Pinia store (`src-frontend/app/stores/services.ts`), not Rust
6. **Process management:** Use `@tauri-apps/plugin-shell` `Command` class (import: `{ Command }`)
7. **Nuxt UI:** Primary `green`, neutral `zinc`; see `.agents/skills/nuxt-ui/SKILL.md`
8. **optimizeDeps:** Add all Tauri plugin JS packages to `nuxt.config.ts` → `vite.optimizeDeps.include`
9. **Capabilities:** New Tauri plugin needs permission entry in `src-tauri/capabilities/default.json`
