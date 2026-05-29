use crate::modules::backup;
use crate::modules::paths::VoltPath;
use crate::utils::path_sep;
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

// prepare_command_env — isolated PATH injection for spawned child processes

/// Prepends the VoltEnv central bin directory to the child process `PATH`
/// so the spawned binary can resolve sibling services and shared libraries
/// without modifying the user's global environment.
///
/// The manipulation is scoped exclusively to this `Command` — no Registry or
/// shell profile is touched.
pub fn prepare_command_env(cmd: &mut tokio::process::Command, app: &AppHandle) {
    let bin_dir = VoltPath::bin_dir(app);
    let bin_dir_str = bin_dir.to_string_lossy().to_string();
    let current = std::env::var("PATH").unwrap_or_default();
    cmd.env("PATH", format!("{}{}{}", bin_dir_str, path_sep(), current));
}

// Central folder init — called once at app startup

/// Ensures `$ROOT/env/` exists (per-service junction folders are created
/// lazily by `switch_service_version`).
///
/// Individual service paths are registered on the OS PATH by
/// `register_service_os_path` when each service is first switched.
pub fn init_central_env_bin(app: &AppHandle) -> Result<(), String> {
    let env = VoltPath::env_dir(app);
    if !env.exists() {
        fs::create_dir_all(&env).map_err(|e| format!("Failed to create env dir: {}", e))?;
    }
    Ok(())
}

// write_windows_path_registry — common Windows Registry write + broadcast

/// Reads the current `PATH` value from `HKEY_CURRENT_USER\Environment`.
#[cfg(target_os = "windows")]
fn read_windows_path_registry() -> Result<String, String> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env_key = hkcu
        .open_subkey_with_flags("Environment", KEY_READ)
        .map_err(|e| format!("Failed to open Registry Environment key: {}", e))?;

    let current: String = env_key.get_value("Path").unwrap_or_default();
    Ok(current)
}

/// Writes `updated_path` to `HKEY_CURRENT_USER\Environment\Path` and
/// broadcasts `WM_SETTINGCHANGE` so running applications detect the change.
///
/// Extracted as a shared helper to avoid duplicating the `winreg` +
/// `windows-sys` unsafe block across three call sites.
#[cfg(target_os = "windows")]
fn write_windows_path_registry(updated_path: &str) -> Result<(), String> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env_key = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .map_err(|e| format!("Failed to open Registry Environment key: {}", e))?;

    env_key
        .set_value("Path", &updated_path)
        .map_err(|e| format!("Failed to update Registry Path: {}", e))?;

    // Broadcast WM_SETTINGCHANGE so running applications detect the updated
    // PATH without a reboot.
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
        };
        let env_str = "Environment\0".encode_utf16().collect::<Vec<u16>>();
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            env_str.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            5000,
            std::ptr::null_mut(),
        );
    }

    Ok(())
}

// register_service_os_path — register env/{service_id} on OS PATH

/// Registers the `$ROOT/env/{service_id}` junction / symlink folder on the
/// OS user PATH **permanently**.
///
/// ## Windows
///
/// Writes directly to the Windows Registry at
/// `HKEY_CURRENT_USER\Environment\Path` via the `winreg` crate, bypassing
/// `setx` entirely.  Registry Path values support up to **32 767 characters**
/// — no risk of the 1024‑character truncation that `setx` imposes.
///
/// After the write a `WM_SETTINGCHANGE` broadcast is sent so running
/// applications (VS Code, terminals, etc.) detect the change immediately
/// without a reboot.
///
/// ## Unix
///
/// Appends `export PATH="$PATH:<dir>"` to `~/.zshrc`, `~/.bashrc`, or
/// `~/.profile` — whichever exists first.  Duplicate lines are never
/// written.
///
/// ## Safety
///
/// - Duplicate check is performed in Rust memory **before** any write.
/// - If the Registry / profile file cannot be read the function returns
///   `Err` — it never writes an empty/destructive value.
/// - Only the semantic `env/{service_id}` path is registered — never a
///   version‑specific binary directory.
pub async fn register_service_os_path(app: &AppHandle, service_id: &str) -> Result<(), String> {
    let env_path = VoltPath::env_service_path(app, service_id);
    let dir_str = env_path.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    {
        // Read current PATH value from Registry (blocking I/O → spawn_blocking)
        let current_path = tokio::task::spawn_blocking(read_windows_path_registry)
            .await
            .map_err(|e| format!("Registry read panicked: {}", e))??;

        // Safety net
        // Backup the current PATH *before* any write.  If backup fails
        // the operation is aborted — the Registry is never touched.
        backup::create_env_backup(service_id, &current_path, app).await?;

        // Duplicate check: if the target is already present, skip the write.
        // The `.contains()` check is safe here because every entry we add
        // has a unique $APPDATA/voltenv/env/<service_id> prefix.
        if current_path.contains(&dir_str) {
            return Ok(());
        }

        let updated_path = if current_path.is_empty() {
            dir_str
        } else {
            format!("{};{}", current_path, dir_str)
        };

        // Write to Registry (blocking I/O → spawn_blocking)
        tokio::task::spawn_blocking(move || write_windows_path_registry(&updated_path))
            .await
            .map_err(|e| format!("Registry write panicked: {}", e))?
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Read current profile content for backup
        let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
        let profile_path = find_or_create_profile(&home);
        let profile_content = if profile_path.exists() {
            fs::read_to_string(&profile_path)
                .map_err(|e| format!("Failed to read profile: {}", e))?
        } else {
            String::new()
        };

        // Safety net
        // Backup the current profile content *before* any write.  If
        // backup fails the operation is aborted — the profile is never
        // modified.
        backup::create_env_backup(service_id, &profile_content, app).await?;

        append_to_shell_profile(&dir_str)
    }
}

// register_to_os_env — manual (re-)registration of the env/ root folder

/// Registers the `$ROOT/env/` root folder into the OS user PATH
/// **permanently**.
///
/// On Windows this writes directly to the Registry (same `winreg` approach
/// as `register_service_os_path`).  On Unix it appends to the shell profile.
///
/// Safe to call multiple times — duplicate PATH entries are never written.
#[tauri::command]
pub fn register_to_os_env(app: AppHandle) -> Result<(), String> {
    let env = VoltPath::env_dir(&app);
    let dir_str = env.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    {
        let current_path = read_windows_path_registry()?;

        if current_path.contains(&dir_str) {
            return Ok(());
        }

        let updated_path = if current_path.is_empty() {
            dir_str
        } else {
            format!("{};{}", current_path, dir_str)
        };

        write_windows_path_registry(&updated_path)
    }

    #[cfg(not(target_os = "windows"))]
    {
        append_to_shell_profile(&dir_str)
    }
}

// register_service_environment — create junction + register PATH

/// Creates a directory junction (Windows) / symlink (Unix) at
/// `$ROOT/env/{id}` pointing to the versioned binary directory
/// `$ROOT/bin/{id}/{version}/`, then registers the path permanently
/// on the OS user PATH.
///
/// Any pre-existing junction at the same location is replaced, so
/// re-running or switching versions is safe.
#[tauri::command]
pub async fn register_service_environment(
    app: AppHandle,
    id: String,
    version: String,
) -> Result<(), String> {
    let bin_path = VoltPath::service_binary_path(&app, &id, &version);
    if !bin_path.exists() {
        return Err(format!("Binary not found at {}", bin_path.display()));
    }

    VoltPath::create_env_junction(&app, &id, &version)?;
    register_service_os_path(&app, &id).await
}

// unregister_service_environment — remove junction from env/

/// Removes the junction / symlink at `$ROOT/env/{id}`.
///
/// Succeeds silently if the entry does not exist — no error is returned
/// when the symlink was already cleaned up.
#[tauri::command]
pub fn unregister_service_environment(app: AppHandle, id: String) -> Result<(), String> {
    let env_path = VoltPath::env_service_path(&app, &id);

    if env_path.exists() {
        #[cfg(target_os = "windows")]
        {
            fs::remove_dir(&env_path)
                .map_err(|e| format!("Failed to remove junction {}: {}", env_path.display(), e))?;
        }
        #[cfg(not(target_os = "windows"))]
        {
            fs::remove_file(&env_path)
                .map_err(|e| format!("Failed to remove symlink {}: {}", env_path.display(), e))?;
        }
    }

    Ok(())
}

// restore_os_path_backup — restore PATH from the most recent backup

/// Restores the OS user PATH from the most recent backup file found in
/// `$ROOT/config/backups/`.
///
/// On Windows the backed‑up string is written directly to the Registry at
/// `HKEY_CURRENT_USER\Environment\Path` and a `WM_SETTINGCHANGE` broadcast
/// is sent so running applications detect the change immediately.
///
/// On Unix the backed‑up string (the entire shell profile content) is
/// written back to the detected shell profile file.
///
/// Returns an informational message on success.
///
/// All disk I/O and Registry operations are wrapped in `spawn_blocking` to
/// keep the Tauri async runtime responsive.
#[tauri::command]
pub async fn restore_os_path_backup(app: AppHandle) -> Result<String, String> {
    let app2 = app.clone();

    tokio::task::spawn_blocking(move || {
        let backups_dir = VoltPath::backups_dir(&app2);

        if !backups_dir.exists() {
            return Err("No backup directory found — no backups to restore.".to_string());
        }

        // Find the most recent .bak file
        let backup_path = find_latest_backup(&backups_dir)
            .ok_or_else(|| "No backup files found in backups directory.".to_string())?;
        let content = fs::read_to_string(&backup_path)
            .map_err(|e| format!("Failed to read backup: {}", e))?;

        let filename = backup_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        #[cfg(target_os = "windows")]
        {
            write_windows_path_registry(&content)?;

            Ok(format!(
                "PATH restored from {} via Windows Registry.",
                filename
            ))
        }

        #[cfg(not(target_os = "windows"))]
        {
            let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
            let profile_path = find_or_create_profile(&home);
            let path_str = profile_path.to_string_lossy().to_string();

            fs::write(&profile_path, &content)
                .map_err(|e| format!("Failed to write {}: {}", path_str, e))?;

            Ok(format!("PATH restored from {} to {}.", filename, path_str))
        }
    })
    .await
    .map_err(|e| format!("restore_os_path_backup panicked: {}", e))?
}

/// Finds the .bak file with the most recent modification time.
fn find_latest_backup(backups_dir: &PathBuf) -> Option<PathBuf> {
    let mut entries: Vec<_> = fs::read_dir(backups_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".bak"))
        .collect();

    if entries.is_empty() {
        return None;
    }

    entries.sort_by_key(|e| {
        std::fs::metadata(e.path())
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });

    entries.last().map(|e| e.path())
}

// Shell‑profile helpers (Unix only)

/// Appends `export PATH="$PATH:<dir>"` to the user's shell profile.
///
/// Tries `~/.zshrc`, `~/.bashrc`, and `~/.profile` in that order, using
/// whichever exists first.  The file is created if none exist.
#[cfg(not(target_os = "windows"))]
fn append_to_shell_profile(dir: &str) -> Result<(), String> {
    let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
    let export_line = format!("export PATH=\"$PATH:{}\"", dir);

    let path = find_or_create_profile(&home);
    let path_str = path.to_string_lossy().to_string();

    let content = if path.exists() {
        let content =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {}", path_str, e))?;
        if content.contains(&export_line) {
            return Ok(());
        }
        content
    } else {
        String::new()
    };

    let mut content = content;
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(&export_line);
    content.push('\n');

    fs::write(&path, &content).map_err(|e| format!("Failed to write {}: {}", path_str, e))?;
    Ok(())
}

/// Detects the user's active shell by reading the `$SHELL` environment
/// variable, then returns the corresponding profile path.
///
/// Resolution order:
/// 1. `$SHELL` env var suffix → `zsh` → `~/.zshrc`, `bash` → `~/.bashrc`
/// 2. Fall back to checking `~/.zshrc`, `~/.bashrc`, `~/.profile` (first existing)
/// 3. Ultimate default: `~/.bashrc`
#[cfg(not(target_os = "windows"))]
fn find_or_create_profile(home: &str) -> std::path::PathBuf {
    // Try to detect the active shell from the $SHELL variable
    if let Ok(shell) = std::env::var("SHELL") {
        let shell_lower = shell.to_lowercase();
        if shell_lower.ends_with("zsh") {
            let p = std::path::Path::new(home).join(".zshrc");
            if p.exists() {
                return p;
            }
        } else if shell_lower.ends_with("bash") {
            let p = std::path::Path::new(home).join(".bashrc");
            if p.exists() {
                return p;
            }
        }
    }

    // Fallback: first existing profile
    for candidate in ["~/.zshrc", "~/.bashrc", "~/.profile"] {
        let p = std::path::Path::new(home).join(candidate.strip_prefix("~/").unwrap_or(candidate));
        if p.exists() {
            return p;
        }
    }

    // Ultimate default
    std::path::Path::new(home).join(".bashrc")
}
