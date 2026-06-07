# VoltEnv Code Audit Report
**Role:** Senior Full-Stack Security Engineer & Performance Architect
**Status:** Completed

## 1. TAURI & RUST SECURITY (IPC Bridge & Memory)

### [CRITICAL] Command Injection Vulnerability
- **File:** `src-tauri/src/commands/service.rs` (L217-L232)
- **Issue:** Fungsi `run_command` mengeksekusi string mentah menggunakan `sh -c` atau `cmd /C`. Argumen yang disubstitusi di `substitute_args` tidak di-escape dengan benar. Jika `def.post_install_commands` atau `def.start_args` mengandung input yang dapat dikontrol (misalnya dari custom service definition), ini bisa memicu eksekusi kode arbitrer.
- **Impact:** Remote Code Execution (RCE) jika penyerang dapat memanipulasi metadata service.

### [CRITICAL] Nginx Config Injection
- **File:** `src-tauri/src/vhost/mod.rs` (L17-L45)
- **Issue:** Variabel `domain` dan `root` dimasukkan langsung ke dalam template blok server Nginx tanpa validasi atau escaping karakter khusus (seperti `;` atau `}`).
- **Impact:** Penyerang dapat merusak konfigurasi Nginx atau mengarahkan traffic ke lokasi yang tidak diinginkan.

### [CRITICAL] Potential SQL Injection
- **File:** `src-tauri/src/commands/database.rs` (L79, L107, L130)
- **Issue:** Meskipun ada sanitasi `is_alphanumeric`, penggunaan format string untuk `CREATE DATABASE` dan `GRANT` tetap berisiko jika logika sanitasi terlewati atau tidak mencakup semua vektor serangan pada MySQL.
- **Impact:** Manipulasi database atau eskalasi hak akses.

### [LOW] Permissive Capabilities
- **File:** `src-tauri/capabilities/default.json`
- **Issue:** Penggunaan `fs:default` dengan cakupan `$APPDATA/**` terlalu luas.
- **Suggestion:** Batasi hanya ke direktori `voltenv` spesifik di dalam AppData.

---

## 2. NUXT v4 & VUE CONVENTIONS

### [SUGGESTION] Potential Memory Leak in Log Store
- **File:** `src-frontend/app/stores/logManager.ts`
- **Issue:** Objek `logs` (Record) menyimpan log per service instance. Jika user menjalankan banyak service berbeda dari waktu ke waktu, data log service yang sudah berhenti tetap tersimpan di memori frontend selama aplikasi berjalan.
- **Fix:** Implementasikan pembersihan `logs[key]` saat service dihentikan atau gunakan `Map` dengan mekanisme LRU.

### [SUGGESTION] SSR/CSR Boundary Safety
- **Issue:** Proyek menggunakan `ssr: false`, yang sudah benar untuk Tauri. Namun, beberapa inisialisasi di `app.vue` (onMounted) bisa dioptimalkan untuk memastikan tidak ada race condition saat mengakses API Tauri.

---

## 3. NUXT UI & ACCESSIBILITY (a11y)

### [SUGGESTION] Missing ARIA Labels
- **Files:** `src-frontend/app/pages/settings.vue`, `src-frontend/app/components/LogConsole.vue`
- **Issue:** Tombol yang hanya menggunakan ikon (seperti Trash/Pencil/Eraser) tidak memiliki `aria-label`, sehingga menyulitkan pengguna screen reader.
- **Fix:** Tambahkan atribut `aria-label` atau `label` yang deskriptif pada `UButton`.

### [SUGGESTION] Focus Management
- **Issue:** Saat membuka modal (seperti di Settings), fokus tidak selalu kembali ke elemen pemicu setelah modal ditutup.

---

## 4. BUNDLE SIZE & PERFORMANCE OPTIMIZATION

### [CRITICAL] Missing Release Profile Optimization
- **File:** `src-tauri/Cargo.toml`
- **Issue:** Tidak ada konfigurasi `[profile.release]`. Tanpa ini, binary Rust akan jauh lebih besar dari target < 10MB.
- **Fix:** Tambahkan:
  ```toml
  [profile.release]
  lto = true
  codegen-units = 1
  panic = "abort"
  strip = true
  opt-level = "z" # Optimize for size
  ```

### [SUGGESTION] Dead Code / Oversized Imports
- **File:** `src-tauri/Cargo.toml`
- **Issue:** Dependensi `tokio` menggunakan fitur `full`. Jika hanya butuh `process`, `net`, dan `rt-multi-thread`, ini bisa dikurangi untuk mempercepat waktu kompilasi.

---

## NEXT STEPS & EXECUTION PLAN
1. **Refactor Security Layer:** Implementasi sanitasi ketat dan gunakan `Command::args` alih-alih substitusi string manual.
2. **Optimize Release Profile:** Update `Cargo.toml` untuk menekan ukuran binary.
3. **Enhance Log Management:** Tambahkan logic pembersihan log di Pinia store.
4. **Improve A11y:** Tambahkan ARIA labels ke semua tombol interaktif.


## 5. DETAILED EXECUTION PLAN (POST-AUDIT)

Berikut adalah langkah-langkah detail untuk memperbaiki temuan di atas:

### Fase 1: Security & Performance Hardening (Rust)
1. **Optimize Release Profile:**
   - Ubah `src-tauri/Cargo.toml` dengan menambahkan profil release (LTO, codegen-units, panic abort).
2. **Fix Command Injection:**
   - Modifikasi `src-tauri/src/commands/service.rs`.
   - Ubah fungsi `run_command` dan `substitute_args` untuk tidak menggunakan shell execution. Gunakan `std::process::Command` atau `tokio::process::Command` dengan argumen yang dipisahkan secara eksplisit.
3. **Fix Nginx Config Injection:**
   - Modifikasi `src-tauri/src/vhost/mod.rs`.
   - Tambahkan fungsi validasi untuk `domain` (regex check) dan sanitasi untuk `root` path.
4. **Hardening Database Commands:**
   - Modifikasi `src-tauri/src/commands/database.rs`.
   - Perketat sanitasi `is_alphanumeric` dan pastikan penggunaan backticks (`) dilakukan secara konsisten pada semua identifier MySQL.

### Fase 2: Frontend Stability & Accessibility (Nuxt)
1. **Fix Memory Leak in Logs:**
   - Modifikasi `src-frontend/app/stores/logManager.ts`.
   - Tambahkan action `removeServiceLogs(serviceId, version)`.
   - Panggil action tersebut saat service dihentikan di `services.ts`.
2. **Improve A11y:**
   - Modifikasi `src-frontend/app/pages/settings.vue`, `LogConsole.vue`, dan `ServiceTable.vue`.
   - Tambahkan `aria-label` pada semua icon-only buttons.
   - Gunakan `aria-live` pada log console untuk memberi tahu pengguna screen reader jika ada log baru yang kritikal.

### Fase 3: Verification
1. **Unit Testing:** Tambahkan test case di Rust untuk memverifikasi fungsi sanitasi database dan vhost.
2. **Build Test:** Jalankan `bun build` untuk memastikan ukuran binary berkurang secara signifikan.


## 6. OBSERVATIONS ON TESTING INFRASTRUCTURE
Selama audit, ditemukan bahwa proyek saat ini **belum memiliki unit test** baik di sisi Rust maupun Nuxt.
- Tidak ada direktori `tests/` di `src-tauri` maupun `src-frontend`.
- Tidak ada atribut `#[test]` dalam kode sumber Rust.
- `package.json` tidak memiliki script `test`.

**Rekomendasi Tambahan:**
Sangat disarankan untuk mulai mengimplementasikan testing framework (seperti `vitest` untuk frontend dan `cargo test` dengan mock untuk Rust) terutama untuk memvalidasi logika sanitasi keamanan yang akan diimplementasikan.

---
*Laporan ini dihasilkan secara otomatis sebagai bagian dari proses audit keamanan dan performa VoltEnv.*
