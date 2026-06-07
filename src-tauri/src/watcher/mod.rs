use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tauri::{async_runtime, AppHandle, Emitter};
use tokio::sync::mpsc;

const DEBOUNCE_MS: u64 = 500;

pub struct FsWatcher {
    _watcher: RecommendedWatcher,
}

impl FsWatcher {
    pub fn new(
        app_handle: AppHandle,
        bin_root: PathBuf,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        std::fs::create_dir_all(&bin_root)?;

        let (tx, rx) = mpsc::unbounded_channel::<Result<Event, notify::Error>>();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                let _ = tx.send(res);
            },
            Config::default(),
        )?;

        watcher.watch(&bin_root, RecursiveMode::Recursive)?;

        let handle = app_handle.clone();
        async_runtime::spawn(async move {
            Self::event_loop(handle, rx, bin_root).await;
        });

        Ok(Self { _watcher: watcher })
    }

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
                Err(e) => {
                    eprintln!("[watcher] notify error: {:?}", e);
                    continue;
                }
            };

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
                    let _ = app_handle.emit(
                        "service-status-changed",
                        serde_json::json!({
                            "id": service_id,
                            "status": "stopped"
                        }),
                    );
                }
            }
        }
    }

    fn extract_service_id(bin_root: &Path, path: &Path) -> Option<String> {
        let relative = path.strip_prefix(bin_root).ok()?;
        let first_component = relative.components().next()?;
        let name = first_component.as_os_str().to_str()?;

        if name.starts_with('.') || name.starts_with("tmp_") || name.starts_with('_') {
            return None;
        }

        Some(name.to_string())
    }
}
