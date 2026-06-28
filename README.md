# VoltEnv

Blazing fast, cross-platform local development environment manager focused on PHP Development.

## Features
- **One-Click Setup**: Automated installation of Nginx, PHP, and MySQL.
- **Local HTTPS**: Automatic SSL certificate generation for `.localhost` and custom domains.
- **PHP Extension Manager**: Toggle PHP extensions (xdebug, intl, etc.) directly from the UI.
- **Composer Integration**: Run composer install/update shortcuts from the dashboard.
- **Cross-Platform**: Built with Rust and Tauri for speed and security.

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
VoltEnv manages binaries in the user's local data directory to keep the installer small (< 10MB).
- Windows: `~/.voltenv/bin`
- Linux/macOS: `~/.voltenv/bin`

## License
MIT
