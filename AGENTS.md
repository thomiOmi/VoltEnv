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
- `setx` for Windows PATH modification — use `winreg` crate instead
- `vue-router` v4.x — use v5.x (v5 includes `./volar/sfc-route-blocks` export required by `vue-tsc`)

## Rules
1. **Core logic lives in Rust.** Download, extract, spawn, kill, and health-check all run through custom Tauri commands (`src-tauri/src/modules/commands.rs`, `services.rs`). The frontend store is a thin `invoke()` wrapper.
2. **Frontend tooling:** Use [unjs.io](https://unjs.io) ecosystem first (consola, ofetch, citty, etc.)
3. **Rust:** Run `cargo fmt` then `cargo clippy` before committing
4. **Frontend:** Run `cd src-frontend && bun run typecheck && bun run lint:fix` before committing
5. **Process management:** The Rust backend manages process lifecycle via `tokio::process::Command` (not `tauri-plugin-shell`). The `ServiceProcesses` map tracks instances by `{id}:{version}` key. Hybrid kill strategy: soft (SIGTERM/taskkill /T) → 3s timeout → force (SIGKILL/taskkill /F /T). Frontend reacts to `service-status-changed` events.
6. **Design system:** Golden Amber (`#EAB308`) accent on a pure neutral canvas. See `DESIGN.md` for full color palette, typography (Geist), spacing (4px base), and component styling rules.
7. **Nuxt UI:** Use semantic colors (`text-default`, `bg-elevated`, `border-muted`, etc.) — never raw Tailwind palette colors. Configured in `app.config.ts`. See `.agents/skills/nuxt-ui/SKILL.md` for component API details.
8. **Dashboard layout** uses `UDashboardGroup` + `UDashboardSidebar` (collapsible/resizable) + `UDashboardPanel` (with `#header`/`#body`/`#footer` slots). Put navigation items in `layouts/default.vue`. Content lives in `pages/index.vue` with `UDashboardNavbar` and `UDashboardToolbar`.
9. **Shared types** go in `shared/types/` (auto-imported by Nuxt 4). Pure type definitions only — no Vue or Nitro imports. Example: `shared/types/service.ts` exports `Service` and `ServiceInfo` interfaces.
10. **Custom components** (`app/components/`):
    - Read `DESIGN.md` first for design tokens (colors, spacing, typography).
    - Use Nuxt UI components as base, customize via `ui` prop or `class`.
    - Use semantic utility classes (`text-default`, `text-muted`, `text-highlighted`, `bg-elevated`, `bg-inverted`, `border-default`, `border-muted`) — never raw Tailwind palette.
    - Import types from `#shared` alias or rely on Nuxt auto-import from `shared/types/`.
    - Follow dashboard template patterns: `ServiceCard.vue` uses `UBadge` + `UButton` with semantic colors; `LogConsole.vue` uses `bg-inverted` for terminal background.
    - One logical component per file, PascalCase name.
11. **Capabilities:** New Tauri plugin needs permission entry in `src-tauri/capabilities/default.json`
