use std::fs;
use std::io::Read;
use std::path::Path;
use crate::utils::{VoltResult, VoltError};

pub struct Verifier;

impl Verifier {
    pub async fn sha256(file_path: &Path, expected_hash: &str) -> VoltResult<()> {
        let path = file_path.to_path_buf();
        let expected = expected_hash.to_string();

        let computed = tokio::task::spawn_blocking(move || compute_sha256_hex(&path))
            .await
            .map_err(|e| VoltError::Custom(format!("Hash computation panicked: {}", e)))??;

        if !computed.eq_ignore_ascii_case(&expected) {
            return Err(VoltError::Custom(format!(
                "Hash mismatch: expected {}, got {}",
                expected, computed
            )));
        }

        Ok(())
    }
}

fn compute_sha256_hex(path: &Path) -> VoltResult<String> {
    use sha2::Digest;
    let mut file = fs::File::open(path)?;
    let mut hasher = sha2::Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
