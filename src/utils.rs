use std::sync::LazyLock;

use wreq::tls::CertStore;
use tracing;

/// Loads the CA certificates from env var PRIMP_CA_BUNDLE or the system's native certificate store
///
/// Priority order:
/// 1. PRIMP_CA_BUNDLE environment variable (custom cert bundle)
/// 2. CA_CERT_FILE environment variable (fallback)
/// 3. System's native certificate store (default - uses rustls-native-certs)
pub fn load_ca_certs() -> Option<&'static CertStore> {
    static CERT_STORE: LazyLock<Result<CertStore, anyhow::Error>> = LazyLock::new(|| {
        let mut ca_store = CertStore::builder();

        if let Ok(ca_cert_path) = std::env::var("PRIMP_CA_BUNDLE").or(std::env::var("CA_CERT_FILE"))
        {
            // Use CA certificate bundle from env var
            tracing::info!("Loading CA certs from: {}", ca_cert_path);
            match std::fs::read(&ca_cert_path) {
                Ok(cert_data) => {
                    ca_store = ca_store.add_stack_pem_certs(&cert_data);
                    tracing::info!("Successfully loaded custom CA certificates from file");
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to read CA cert file '{}': {}. Falling back to system certificates.",
                        ca_cert_path, e
                    );
                    // Fallback to system certs on error
                    let cert_result = rustls_native_certs::load_native_certs();

                    if !cert_result.errors.is_empty() {
                        for err in &cert_result.errors {
                            tracing::warn!("Error loading some system certificates: {}", err);
                        }
                    }

                    if cert_result.certs.is_empty() {
                        return Err(anyhow::Error::msg(
                            "No system certificates could be loaded"
                        ));
                    }

                    let cert_count = cert_result.certs.len();
                    let der_certs: Vec<&[u8]> = cert_result.certs
                        .iter()
                        .map(|cert| cert.as_ref())
                        .collect();
                    ca_store = ca_store.add_der_certs(der_certs.into_iter());
                    tracing::info!("Loaded {} certificates from system certificate store", cert_count);
                }
            }
        } else {
            // Use system's native certificate store (default)
            tracing::debug!("Loading certificates from system certificate store");
            let cert_result = rustls_native_certs::load_native_certs();

            if !cert_result.errors.is_empty() {
                for err in &cert_result.errors {
                    tracing::warn!("Error loading some system certificates: {}", err);
                }
            }

            if cert_result.certs.is_empty() {
                return Err(anyhow::Error::msg(
                    "No system certificates could be loaded"
                ));
            }

            let cert_count = cert_result.certs.len();
            let der_certs: Vec<&[u8]> = cert_result.certs
                .iter()
                .map(|cert| cert.as_ref())
                .collect();
            ca_store = ca_store.add_der_certs(der_certs.into_iter());
            tracing::info!("Loaded {} certificates from system certificate store", cert_count);
        }

        ca_store.build().map_err(|e| anyhow::Error::msg(format!("Failed to build cert store: {}", e)))
    });

    match CERT_STORE.as_ref() {
        Ok(cert_store) => {
            tracing::debug!("CA certificate store ready");
            Some(cert_store)
        }
        Err(err) => {
            tracing::error!("Failed to load CA certs: {:?}", err);
            None
        }
    }
}

#[cfg(test)]
mod load_ca_certs_tests {
    use super::*;

    #[test]
    fn test_load_system_ca_certs() {
        // Test loading system certificates
        let result = load_ca_certs();

        // System should have at least some certificates
        assert!(result.is_some(), "Failed to load system CA certificates");
    }
}
