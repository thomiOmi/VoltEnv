use crate::utils::http_client;
use std::fs;
use std::io::Read;
use std::path::Path;

/// Verifies downloaded files using SHA-256 checksums or PGP signatures.
///
/// CPU-heavy operations (SHA-256 streaming, `gpg --verify`) run on the
/// blocking thread pool via `tokio::task::spawn_blocking` so the async
/// runtime is never blocked.
pub struct Verifier;

impl Verifier {
    /// Computes the SHA-256 hex digest of `file_path` and compares it
    /// (case-insensitively) against `expected_hash`.
    ///
    /// Cryptographic hash computation runs inside
    /// `tokio::task::spawn_blocking` so the async runtime is not starved
    /// of worker threads while hashing large archives.
    ///
    /// The file is read in 8 KiB chunks so memory usage stays constant
    /// regardless of file size.
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

    /// Downloads a PGP signature from `sig_url`, writes it to a temp file,
    /// and runs `gpg --verify`.
    ///
    /// The `gpg` subprocess invocation is offloaded to
    /// `tokio::task::spawn_blocking` because it is a CPU-heavy
    /// child-process operation that could stall the async runtime.
    ///
    /// The temporary signature file is cleaned up via
    /// `tokio::fs::remove_file` after verification completes.
    ///
    /// Returns a clear error message when `gpg` is not installed.
    pub async fn pgp_signature(sig_url: &str, target_path: &Path) -> Result<(), String> {
        let sig_response = http_client()
            .get(sig_url)
            .send()
            .await
            .map_err(|e| format!("Failed to download PGP signature: {}", e))?;

        if !sig_response.status().is_success() {
            return Err(format!(
                "PGP signature download failed (HTTP {})",
                sig_response.status()
            ));
        }

        let sig_bytes = sig_response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read PGP signature: {}", e))?;

        // Write signature to a temp file next to the target
        let sig_path = target_path.with_extension("zip.asc");
        tokio::fs::write(&sig_path, &sig_bytes)
            .await
            .map_err(|e| format!("Failed to write PGP signature: {}", e))?;

        let sig = sig_path.to_path_buf();
        let target = target_path.to_path_buf();
        let sig_for_cleanup = sig_path.to_path_buf();

        let gpg_output = tokio::task::spawn_blocking(move || {
            std::process::Command::new("gpg")
                .args([
                    "--verify",
                    &sig.to_string_lossy(),
                    &target.to_string_lossy(),
                ])
                .output()
        })
        .await
        .map_err(|e| format!("gpg task panicked: {}", e))?
        .map_err(|e| {
            let _ = fs::remove_file(&sig_for_cleanup);
            if e.kind() == std::io::ErrorKind::NotFound {
                "PGP verification requires `gpg` to be installed on your system".to_string()
            } else {
                format!("Failed to run gpg: {}", e)
            }
        })?;

        let _ = tokio::fs::remove_file(&sig_path).await;

        if !gpg_output.status.success() {
            let stderr = String::from_utf8_lossy(&gpg_output.stderr);
            return Err(format!(
                "PGP signature verification failed:\n{}",
                stderr.trim()
            ));
        }

        Ok(())
    }
}

/// Synchronous SHA-256 computation.  Reads the file in 8 KiB chunks so
/// large archives do not spike memory.
///
/// Intended to be called inside `tokio::task::spawn_blocking`.
fn compute_sha256_hex(path: &Path) -> Result<String, String> {
    use sha2::Digest;
    let mut file = fs::File::open(path).map_err(|e| format!("Failed to open for hash: {}", e))?;
    let mut hasher = sha2::Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| format!("Failed to read for hash: {}", e))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
