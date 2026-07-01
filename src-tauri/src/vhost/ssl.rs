use rcgen::{
    CertificateParams, DistinguishedName, DnType, KeyPair,
};
use std::path::Path;

pub struct SslManager;

impl SslManager {
    pub fn generate_ca() -> Result<(String, String), String> {
        let mut params = CertificateParams::default();
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        params.distinguished_name = DistinguishedName::new();
        params
            .distinguished_name
            .push(DnType::CommonName, "VoltEnv Root CA");
        params
            .distinguished_name
            .push(DnType::OrganizationName, "VoltEnv");

        let key_pair = KeyPair::generate().map_err(|e| e.to_string())?;
        let cert = params.self_signed(&key_pair).map_err(|e| e.to_string())?;

        Ok((cert.pem(), key_pair.serialize_pem()))
    }

    pub fn generate_cert(
        ca_cert_pem: &str,
        ca_key_pem: &str,
        domain: &str,
    ) -> Result<(String, String), String> {
        let ca_key_pair = KeyPair::from_pem(ca_key_pem).map_err(|e| e.to_string())?;

        let ca_params = rcgen::CertificateParams::from_ca_cert_pem(ca_cert_pem).map_err(|e| e.to_string())?;
        let ca_cert = ca_params.self_signed(&ca_key_pair).map_err(|e| e.to_string())?;

        let mut params = CertificateParams::new(vec![domain.to_string()]).map_err(|e| e.to_string())?;
        params.distinguished_name = DistinguishedName::new();
        params.distinguished_name.push(DnType::CommonName, domain);

        let key_pair = KeyPair::generate().map_err(|e| e.to_string())?;
        let cert = params
            .signed_by(&key_pair, &ca_cert, &ca_key_pair)
            .map_err(|e| e.to_string())?;

        Ok((cert.pem(), key_pair.serialize_pem()))
    }
}

impl SslManager {
    pub fn install_ca(_ca_path: &Path) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            let status = std::process::Command::new("certutil")
                .args(["-addstore", "-f", "Root", &_ca_path.to_string_lossy()])
                .status()
                .map_err(|e| e.to_string())?;
            if !status.success() {
                return Err("Failed to install CA via certutil".to_string());
            }
        }

        #[cfg(target_os = "macos")]
        {
            let status = std::process::Command::new("sudo")
                .args([
                    "security",
                    "add-trusted-cert",
                    "-d",
                    "-r",
                    "trustRoot",
                    "-k",
                    "/Library/Keychains/System.keychain",
                    &_ca_path.to_string_lossy(),
                ])
                .status()
                .map_err(|e| e.to_string())?;
            if !status.success() {
                return Err("Failed to install CA via security tool".to_string());
            }
        }

        Ok(())
    }
}
