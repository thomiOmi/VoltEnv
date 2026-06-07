use super::PlatformAdapter;

pub struct WindowsAdapter;

impl PlatformAdapter for WindowsAdapter {
    async fn kill_process(pid: u32, force: bool) -> Result<(), String> {
        let mut cmd = tokio::process::Command::new("taskkill");
        if force {
            cmd.args(["/F", "/T", "/PID", &pid.to_string()]);
        } else {
            cmd.args(["/T", "/PID", &pid.to_string()]);
        }
        let output = cmd
            .output()
            .await
            .map_err(|e| format!("taskkill failed: {}", e))?;
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
            Ok(o) => String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()),
            Err(_) => false,
        }
    }
}
