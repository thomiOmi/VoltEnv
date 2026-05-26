# VoltEnv

Blazing fast, cross-platform local development environment manager.

## Tech Stack
- **Runtime:** Bun
- **Backend:** Rust (Tauri v2, Tokio)
- **Frontend:** Nuxt 4, Nuxt UI v3, TailwindCSS
- **State:** Pinia

## Development
```bash
# Install frontend dependencies
cd src-frontend
bun install

# Run in development mode
bun tauri dev
```

## Binary Path
VoltEnv manages binaries (Nginx, PHP, MySQL) in the user's local data directory to keep the installer small (< 10MB).
- Windows: `~/.voltenv/bin`
- Linux/macOS: `~/.voltenv/bin`

## License
MIT
