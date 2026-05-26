# VoltEnv Technical Documentation

## Overview
VoltEnv adalah manajer lingkungan pengembangan lokal yang ringan, menggunakan Tauri v2 (Rust), Nuxt 4 (Frontend), dan Bun (Package Manager).

## Backend (Rust)
Terletak di folder `src-tauri`.

### Modul Utama:
- `modules::paths`: Menangani lokasi file binary secara cross-platform (~/.voltenv/bin). Menggunakan crate `directories` dan `dirs`.
- `modules::services`: Mengelola siklus hidup proses menggunakan `tokio::process`. Menangani spawn dan kill proses secara asynchronous.
- `modules::commands`: Berisi fungsi yang diekspos ke frontend melalui Tauri invoke system.

### Keamanan & Efisiensi:
- Menggunakan `tokio` untuk operasi non-blocking.
- Pada Windows, proses dijalankan dengan flag `CREATE_NO_WINDOW` untuk menghindari popup terminal.

## Frontend (Nuxt 4)
Terletak di folder `src-frontend`.

### Teknologi:
- **Nuxt 4**: Menggunakan struktur direktori `app/`.
- **Nuxt UI v3**: Framework komponen berbasis TailwindCSS.
- **Pinia**: Manajemen state untuk status layanan.
- **Tauri API v2**: Untuk komunikasi bridge antara JS dan Rust.

### State Management:
Status layanan disimpan di Pinia dan dipersistensikan secara manual ke `localStorage` di dalam `actions`. Hal ini memungkinkan audit kode yang lebih transparan dibandingkan menggunakan plugin otomatis.

## Instalasi & Pengembangan
1. Install dependencies: `bun install` (di root dan src-frontend).
2. Jalankan mode dev: `bun tauri dev`.
3. Build aplikasi: `bun tauri build`.

## Lokasi Binary
Aplikasi mencari binary di:
- Windows: `%USERPROFILE%\AppData\Local\voltenv\bin\`
- macOS/Linux: `~/.local/share/voltenv/bin/` atau `~/.voltenv/bin/`
