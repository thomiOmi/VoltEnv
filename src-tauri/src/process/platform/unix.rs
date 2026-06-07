use super::PlatformAdapter;

pub struct UnixAdapter;

impl PlatformAdapter for UnixAdapter {
    async fn kill_process(pid: u32, force: bool) -> Result<(), String> {
        let signal = if force { "-9" } else { "-15" };
        let output = tokio::process::Command::new("kill")
            .args([signal, &pid.to_string()])
            .output()
            .await
            .map_err(|e| format!("kill failed: {}", e))?;
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
