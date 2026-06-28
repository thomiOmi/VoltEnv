use std::fs;
use std::path::PathBuf;

pub struct HostsManager;

impl HostsManager {
    #[cfg(windows)]
    fn hosts_path() -> PathBuf {
        PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
    }

    #[cfg(not(windows))]
    fn hosts_path() -> PathBuf {
        PathBuf::from("/etc/hosts")
    }

    pub fn add_entry(domain: &str) -> Result<(), String> {
        let path = Self::hosts_path();
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;

        if content.contains(domain) {
            return Ok(());
        }

        let new_line = format!("\n127.0.0.1 {}\n", domain);
        fs::write(&path, content + &new_line).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                format!("Permission denied. Please run with administrator/sudo privileges to update hosts file.")
            } else {
                e.to_string()
            }
        })
    }
}
