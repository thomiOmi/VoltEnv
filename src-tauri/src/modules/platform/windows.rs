use crate::modules::platform::PlatformAdapter;

/// Platform adapter for Windows.
///
/// Uses `taskkill` for process termination and `tasklist` for liveness
/// checks. Directory symlinks are created in `paths.rs` via the `junction`
/// crate.
pub struct WindowsAdapter;

impl PlatformAdapter for WindowsAdapter {
    async fn kill_process(pid: u32, force: bool) -> Result<(), String> {
        let mut cmd = tokio::process::Command::new("taskkill");

        if force {
            // Force kill with /F (force) and /T (process tree)
            cmd.args(["/F", "/T", "/PID", &pid.to_string()]);
        } else {
            // Soft kill also uses /T to avoid orphaned child processes
            cmd.args(["/T", "/PID", &pid.to_string()]);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Failed to execute taskkill: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("taskkill failed: {}", stderr.trim()));
        }
        Ok(())
    }

    async fn is_process_alive(pid: u32) -> bool {
        let output = tokio::process::Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/NH"])
            .output()
            .await;

        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                // Correctly handle non-English OS locales: the "INFO: No tasks"
                // string is localized on non-English Windows.  Checking for the
                // PID itself in the raw output is locale-independent.
                stdout.contains(&pid.to_string())
            }
            Err(_) => false,
        }
    }
}
