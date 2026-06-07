use std::fs;
use std::io::Read;
use std::path::Path;

pub struct Verifier;

impl Verifier {
    pub async fn sha256(file_path: &Path, expected_hash: &str) -> Result<(), String> {
        let path = file_path.to_path_buf();
        let expected = expected_hash.to_string();

        let computed = tokio::task::spawn_blocking(move || compute_sha256_hex(&path))
            .await
            .map_err(|e| format!("Hash computation panicked: {}", e))?
            .map_err(|e| format!("Failed to compute hash: {}", e))?;

        if !computed.eq_ignore_ascii_case(&expected) {
            return Err(format!(
                "Hash mismatch: expected {}, got {}",
                expected, computed
            ));
        }

        Ok(())
    }
}

fn compute_sha256_hex(path: &Path) -> Result<String, String> {
    use sha2::Digest;
    let mut file = fs::File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let mut hasher = sha2::Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| format!("Read error: {}", e))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
