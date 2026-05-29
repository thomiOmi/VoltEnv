use std::sync::OnceLock;

/// Returns the shared `reqwest::Client`, building it once on first access.
///
/// The client uses a 5-minute timeout and is reused across all download
/// operations for connection pooling (DNS, TLS, TCP).
pub fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("Failed to build reqwest Client")
    })
}

/// Returns the PATH separator for the current OS (`;` on Windows, `:` on Unix).
pub fn path_sep() -> &'static str {
    if cfg!(target_os = "windows") {
        ";"
    } else {
        ":"
    }
}

/// Checks whether a given TCP port is available (not in use by another process).
/// Tries to bind to 127.0.0.1:{port} asynchronously; returns `true` if the bind succeeds.
#[tauri::command]
pub async fn is_port_available(port: u16) -> bool {
    tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .is_ok()
}
