use crate::modules::platform::PlatformAdapter;

/// Platform adapter for Unix-like systems (macOS, Linux, BSD, …).
///
/// Uses `kill` for process termination and liveness checks. Symlinks are
/// created in `paths.rs` via `std::os::unix::fs::symlink`. PATH registration
/// is handled by the unified `env.rs` module.
///
/// # Signal strategy
///
/// - **Soft kill** — sends `SIGTERM` (signal 15) directly to the PID.
///   The process may catch or ignore this signal.
/// - **Force kill** — sends `SIGKILL` (signal 9) directly to the PID.
///   The kernel immediately terminates the process; it cannot be caught
///   or ignored.
/// - **Liveness check** — sends `kill -0`, which performs no action but
///   returns success if the process exists.
///
/// Both soft and force signals target the **main** PID only — they do **not**
/// automatically propagate to child processes. If the service framework
/// (e.g. Nginx, MySQL) spawns sub‑processes that must also be reaped:
///
/// 1. Start the service binary with
///    [`std::os::unix::process::CommandExt::process_group`] so children
///    inherit the same process group ID (PGID).
/// 2. Send the signal to the **negative** PGID (e.g. `kill -(-pgid)`)
///    to broadcast it to every process in the group.
///
/// The `services` module currently does not use process groups.  If orphan
/// reaping becomes necessary, the orchestration layer should adopt this
/// strategy rather than modifying `PlatformAdapter`.
pub struct UnixAdapter;

impl PlatformAdapter for UnixAdapter {
    async fn kill_process(pid: u32, force: bool) -> Result<(), String> {
        let signal = if force { "-9" } else { "-15" };
        let output = tokio::process::Command::new("kill")
            .args([signal, &pid.to_string()])
            .output()
            .await
            .map_err(|e| format!("Failed to execute kill: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("kill {} failed: {}", signal, stderr.trim()));
        }
        Ok(())
    }

    async fn is_process_alive(pid: u32) -> bool {
        tokio::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
