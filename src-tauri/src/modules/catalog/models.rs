use serde::{Deserialize, Serialize};

/// Describes a known service in the VoltEnv catalog.
///
/// Loaded from `$APPDATA/voltenv/config/catalog.json` at startup.
/// Users can add, remove, or edit services by editing that file directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Unique service identifier (e.g. `"nginx"`).
    pub id: String,
    /// Human-friendly display name (e.g. `"Nginx"`).
    /// Falls back to the capitalized `id` when empty.
    #[serde(default)]
    pub name: String,
    /// Default port the service listens on (e.g. `80` for Nginx).
    #[serde(default = "default_port")]
    pub port: u16,
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

/// Default port for services — `0` indicates "unknown / not yet configured".
const fn default_port() -> u16 {
    0
}

impl ServiceConfig {
    /// Returns the display name, falling back to a capitalized version of
    /// `id` when `name` is empty.
    pub fn display_name(&self) -> String {
        if self.name.is_empty() {
            let mut c = self.id.chars();
            match c.next() {
                None => self.id.clone(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        } else {
            self.name.clone()
        }
    }
}

// -- IPC response types mirroring shared/types/service.ts ---------------

/// Serialised form of `Service['status']` from the TypeScript shared types.
/// `#[serde(untagged)]` produces:
///   - `"Running"` / `"Stopped"` / `"Starting"` as plain JSON strings
///   - `{ "Error": "message" }` for the error variant
#[derive(Clone, Serialize)]
#[serde(untagged)]
#[allow(dead_code)]
pub enum ServiceStatusValue {
    Running,
    Stopped,
    Starting,
    Error {
        #[serde(rename = "Error")]
        message: String,
    },
}

/// IPC response for the `get_catalog` command.
/// Mirrors the `ServiceInfo` interface from `shared/types/service.ts`.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceInfoResponse {
    pub id: String,
    pub name: String,
    pub port: u16,
    pub version: String,
    pub versions: Vec<String>,
    /// Subset of `versions` that are actually installed on disk.
    /// Populated by scanning `bin/{id}/` at startup.
    #[serde(default)]
    pub installed_versions: Vec<String>,
    pub download_url: String,
    pub sha256: Option<String>,
    pub pgp_signature_url: Option<String>,
}

/// IPC response for the `get_service_status` command.
/// Mirrors the `Service` interface from `shared/types/service.ts`.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceStatusResponse {
    pub id: String,
    pub name: String,
    pub version: String,
    pub status: ServiceStatusValue,
    pub port: u16,
}
