use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncRead};
use tokio::process::{ChildStderr, ChildStdout};

#[allow(async_fn_in_trait)]
pub trait PlatformAdapter {
    async fn kill_process(pid: u32, force: bool) -> Result<(), String>;
    async fn is_process_alive(pid: u32) -> bool;
}

fn log_reader_inner(
    app: AppHandle,
    sid: String,
    ver: String,
    stream: impl AsyncRead + Unpin + Send + 'static,
    is_error: bool,
) -> tokio::task::JoinHandle<()> {
    let reader = tokio::io::BufReader::new(stream);
    tokio::spawn(async move {
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let payload = serde_json::json!({
                "service_id": sid,
                "version": ver,
                "message": line,
                "is_error": is_error,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            });

            let _ = app.emit(&format!("service-log:{}", sid), payload.clone());
            let _ = app.emit("service-log", payload);
        }
    })
}

pub fn start_log_reader(
    app: AppHandle,
    service_id: &str,
    version: &str,
    stream: ChildStdout,
    is_error: bool,
) -> tokio::task::JoinHandle<()> {
    log_reader_inner(
        app,
        service_id.to_string(),
        version.to_string(),
        stream,
        is_error,
    )
}

pub fn start_log_reader_err(
    app: AppHandle,
    service_id: &str,
    version: &str,
    stream: ChildStderr,
    is_error: bool,
) -> tokio::task::JoinHandle<()> {
    log_reader_inner(
        app,
        service_id.to_string(),
        version.to_string(),
        stream,
        is_error,
    )
}

#[cfg(not(target_os = "windows"))]
pub mod unix;
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "windows")]
pub type Platform = self::windows::WindowsAdapter;
#[cfg(not(target_os = "windows"))]
pub type Platform = self::unix::UnixAdapter;
