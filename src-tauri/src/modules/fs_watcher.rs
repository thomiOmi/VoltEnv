//! Real-time file system watcher for service binary directories.
//!
//! ## Lifecycle
//!
//! ```text
//! main.rs .setup()
//!   └── FsWatcher::new(app_handle, bin_root)       ← creates & starts watcher
//!        ├── create_dir_all(bin_root)               ← ensures target exists
//!        ├── RecommendedWatcher::new(…)             ← notify's OS-native watcher
//!        ├── watcher.watch(bin_root, Recursive)     ← begins monitoring
//!        └── async_runtime::spawn(event_loop)       ← background event processor
//!              ├── rx.recv().await                   ← async, non-blocking
//!              ├── extract_service_id()              ← maps path → service ID
//!              └── handle.emit("service-status-changed", …)
//!
//! app.manage(fs_watcher)                            ← stored in Tauri state
//!
//! Tauri shutdown
//!   └── FsWatcher dropped                           ← watcher handle dropped
//!        └── notify stops background threads         ← clean teardown
//!        └── mpsc sender dropped                    ← rx.recv() returns None
//!             └── tokio::spawn task exits            ← graceful task termination
//! ```
//!
//! ## Cleanup Contract
//!
//! The watcher is owned by Tauri's managed state via `app.manage(FsWatcher)`.
//! It is dropped automatically when the `App` is dropped at shutdown.
//! When the `RecommendedWatcher` handle is dropped, `notify` stops its internal
//! OS-level watcher and joins its background threads.
//! The `mpsc::UnboundedSender` held by the notify event handler is also dropped,
//! which signals the tokio event-loop task to exit (`rx.recv().await` returns `None`).
//! No explicit teardown is required — the lifetime is fully bound to the Tauri app.
//!
//! ## Edge Cases
//!
//! - **Directory missing at init:** `create_dir_all` is called before `watch`,
//!   so the target always exists. If the directory is deleted between `create_dir_all`
//!   and `watch`, `watch` returns `Err(notify::Error::PathNotFound)` which propagates
//!   up and is logged. The app continues without the watcher.
//! - **Rapid Remove events:** Debounced per service ID (500 ms window).
//!   Multiple file deletions inside the same service directory produce one event.
//! - **Non-service entries skipped:** Temp files (`tmp_*`), dotfiles, and
//!   hidden directories are ignored by `extract_service_id`.
//! - **Watch queue overflow:** `notify` may drop events silently if the kernel
//!   event queue overflows. This is a platform-level limitation.

use crate::modules::catalog::CatalogManager;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tauri::{async_runtime, AppHandle, Emitter, Manager};
use tokio::sync::mpsc;

/// Minimum time between `service-status-changed` emits for the same service ID.
/// Prevents event storms when a user deletes a large directory.
const DEBOUNCE_MS: u64 = 500;

/// Tauri-managed state holding the file system watcher handle.
///
/// Kept alive for the entire app lifetime. Dropped automatically on shutdown.
pub struct FsWatcher {
    /// The underlying notify watcher — stored so it is not dropped.
    /// `RecommendedWatcher` on Windows uses `ReadDirectoryChangesW` via an I/O
    /// completion port; on macOS it uses `FSEventStream`; on Linux `inotify`.
    /// All implementations stop watching when this handle is dropped.
    _watcher: RecommendedWatcher,
}

impl FsWatcher {
    /// Create a new file system watcher that monitors `bin_root` for `Remove` events.
    ///
    /// The watcher runs on a background tokio task and emits `service-status-changed`
    /// Tauri events when a file or directory is deleted inside a service's subdirectory.
    ///
    /// # Contract
    ///
    /// - `bin_root` must be a directory path (created if missing).
    /// - The returned `FsWatcher` **must** be kept alive (e.g., via `app.manage()`)
    ///   for the entire time monitoring is required.
    /// - The background task exits when the `FsWatcher` is dropped.
    ///
    /// # Errors
    ///
    /// Propagates errors from `notify` (e.g., `PathNotFound` if `bin_root` was deleted
    /// between `create_dir_all` and `watch`).
    pub fn new(
        app_handle: AppHandle,
        bin_root: PathBuf,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Ensure the watch target exists — handles the case where bin/ hasn't been
        // created yet (e.g., first launch with no installed services).
        std::fs::create_dir_all(&bin_root)?;

        let (tx, rx) = mpsc::unbounded_channel::<Result<Event, notify::Error>>();

        // Configure the OS-native watcher. The closure is called from notify's
        // internal thread and must be Send + 'static. We bridge to tokio via the
        // unbounded mpsc channel.
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                // If the receiver has been dropped (shutting down), errors are ignored.
                let _ = tx.send(res);
            },
            Config::default(),
        )?;

        // Watch recursively — we need to detect deletes at any nesting level inside
        // bin/{service_id}/{version}/...
        watcher.watch(&bin_root, RecursiveMode::Recursive)?;

        // Spawn the async event processor. It runs independently and exits when
        // the mpsc channel closes (triggered by FsWatcher drop).
        let handle = app_handle.clone();
        async_runtime::spawn(async move {
            Self::event_loop(handle, rx, bin_root).await;
        });

        Ok(Self { _watcher: watcher })
    }

    /// Background event processor. Reads events from the mpsc channel, debounces
    /// them per service ID, and emits `service-status-changed` Tauri events.
    async fn event_loop(
        app_handle: AppHandle,
        mut rx: mpsc::UnboundedReceiver<Result<Event, notify::Error>>,
        bin_root: PathBuf,
    ) {
        let mut last_emit: HashMap<String, Instant> = HashMap::new();
        let debounce = Duration::from_millis(DEBOUNCE_MS);

        while let Some(result) = rx.recv().await {
            let event = match result {
                Ok(e) => e,
                // notify errors are logged but do not crash the watcher
                Err(e) => {
                    eprintln!("[fs_watcher] notify error: {:?}", e);
                    continue;
                }
            };

            // We only care about Remove events: Remove(File), Remove(Folder), Remove(Other).
            // Create, Modify, Rename events are ignored.
            if !matches!(event.kind, EventKind::Remove(_)) {
                continue;
            }

            for path in &event.paths {
                let Some(service_id) = Self::extract_service_id(&bin_root, path) else {
                    continue;
                };

                let now = Instant::now();
                let should_emit = last_emit
                    .get(&service_id)
                    .map(|last| now.duration_since(*last) >= debounce)
                    .unwrap_or(true);

                if should_emit {
                    last_emit.insert(service_id.clone(), now);

                    // If the entire service directory has been deleted, clear
                    // the cached active version so get_service_status does not
                    // return stale data.
                    let service_dir = bin_root.join(&service_id);
                    if !service_dir.exists() {
                        if let Some(catalog) = app_handle.try_state::<CatalogManager>() {
                            let _ = catalog.remove_active_version(&app_handle, &service_id);
                        }
                    }

                    let _ = app_handle.emit(
                        "service-status-changed",
                        serde_json::json!({
                            "id": service_id,
                            "status": "Stopped"
                        }),
                    );
                }
            }
        }

        // Channel closed — watcher was dropped, task exits gracefully.
        eprintln!("[fs_watcher] event loop exiting (shutting down)");
    }

    /// Extract the service ID from an absolute path inside `bin_root`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// /root/bin/mysql/8.0.32/bin/mysql.exe  →  Some("mysql")
    /// /root/bin/nginx/1.25.0/nginx.exe      →  Some("nginx")
    /// /root/bin/tmp_mysql.zip                →  None  (temp file)
    /// /root/bin/.hidden                      →  None
    /// ```
    fn extract_service_id(bin_root: &Path, path: &Path) -> Option<String> {
        let relative = path.strip_prefix(bin_root).ok()?;
        let first_component = relative.components().next()?;
        let name = first_component.as_os_str().to_str()?;

        // Filter out non-service entries
        if name.starts_with('.') || name.starts_with("tmp_") || name.starts_with('_') {
            return None;
        }

        Some(name.to_string())
    }
}
