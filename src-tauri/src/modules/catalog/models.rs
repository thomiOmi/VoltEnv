use serde::{Deserialize, Serialize};

/// Describes a known service in the VoltEnv catalog.
///
/// Loaded from `$APPDATA/voltenv/config/catalog.json` at startup.
/// Users can add, remove, or edit services by editing that file directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Unique service identifier (e.g. `"nginx"`).
    pub id: String,
    /// Version string used to build the on-disk path
    /// (`$ROOT/bin/{id}/{version}/`).
    pub version: String,
    /// Arguments passed to the binary when starting the service.
    #[serde(default)]
    pub start_args: Vec<String>,
    /// Arguments passed to the binary when performing a graceful shutdown
    /// (e.g. `["-s", "stop"]` for nginx).
    #[serde(default)]
    pub stop_args: Vec<String>,
    /// Download URL template.  The `{version}` placeholder is replaced at
    /// download time with the actual `version` field value.
    ///
    /// Example:
    /// `"https://nginx.org/download/nginx-{version}.zip"`
    #[serde(default)]
    pub url_template: Option<String>,
    /// SHA‑256 checksum template.  The `{version}` placeholder is replaced
    /// the same way as `url_template`.  When absent, integrity verification
    /// is skipped.
    #[serde(default)]
    pub sha256: Option<String>,
    /// PGP signature URL template.  The `{version}` placeholder is replaced
    /// the same way as `url_template`.  When absent, PGP verification is
    /// skipped.
    #[serde(default)]
    pub pgp_signature_url_template: Option<String>,
}
