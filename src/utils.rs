use std::sync::LazyLock;

use wreq::tls::CertStore;
use tracing;

/// Loads the CA certificates from env var PRIMP_CA_BUNDLE or the WebPKI certificate store
///
/// Priority order:
/// 1. PRIMP_CA_BUNDLE environment variable (custom cert bundle)
/// 2. CA_CERT_FILE environment variable (fallback)
/// 3. Mozilla's trusted root certificates from webpki-root-certs (default)
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
                    tracing::info!("Successfully loaded custom CA certificates");
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to read CA cert file '{}': {}. Falling back to built-in certs.",
                        ca_cert_path, e
                    );
                    // Fallback to built-in certs on error
                    let der_certs: Vec<&[u8]> = webpki_root_certs::TLS_SERVER_ROOT_CERTS
                        .iter()
                        .map(|cert| cert.as_ref())
                        .collect();
                    ca_store = ca_store.add_der_certs(der_certs.into_iter());
                }
            }
        } else {
            // Use WebPKI certificate store (Mozilla's latest trusted root certificates)
            tracing::debug!("Using built-in Mozilla root certificates");
            let der_certs: Vec<&[u8]> = webpki_root_certs::TLS_SERVER_ROOT_CERTS
                .iter()
                .map(|cert| cert.as_ref())
                .collect();
            let cert_count = der_certs.len();
            ca_store = ca_store.add_der_certs(der_certs.into_iter());
            tracing::debug!("Loaded {} Mozilla root certificates", cert_count);
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
    use std::env;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_load_ca_certs_with_env_var() {
        // Create a temporary file with a CA certificate
        let ca_cert_path = Path::new("test_ca_cert.pem");
        let ca_cert = "-----BEGIN CERTIFICATE-----
MIIDdTCCAl2gAwIBAgIVAMIIujU9wQIBADANBgkqhkiG9w0BAQUFADBGMQswCQYD
VQQGEwJVUzETMBEGA1UECAwKQ2FsaWZvcm5pYTEWMBQGA1UEBwwNTW91bnRhaW4g
Q29sbGVjdGlvbjEgMB4GA1UECgwXUG9zdGdyZXMgQ29uc3VsdGF0aW9uczEhMB8G
A1UECwwYUG9zdGdyZXMgQ29uc3VsdGF0aW9uczEhMB8GA1UEAwwYUG9zdGdyZXMg
Q29uc3VsdGF0aW9uczEiMCAGCSqGSIb3DQEJARYTcGVyc29uYWwtZW1haWwuY29t
MIIDdTCCAl2gAwIBAgIVAMIIujU9wQIBADANBgkqhkiG9w0BAQUFADBGMQswCQYD
VQQGEwJVUzETMBEGA1UECAwKQ2FsaWZvcm5pYTEWMBQGA1UEBwwNTW91bnRhaW4g
Q29sbGVjdGlvbjEgMB4GA1UECgwXUG9zdGdyZXMgQ29uc3VsdGF0aW9uczEhMB8G
A1UECwwYUG9zdGdyZXMgQ29uc3VsdGF0aW9uczEhMB8GA1UEAwwYUG9zdGdyZXMg
Q29uc3VsdGF0aW9uczEiMCAGCSqGSIb3DQEJARYTcGVyc29uYWwtZW1haWwuY29t
-----END CERTIFICATE-----";
        fs::write(ca_cert_path, ca_cert).unwrap();

        // Set the environment variable
        env::set_var("PRIMP_CA_BUNDLE", ca_cert_path);

        // Call the function
        let result = load_ca_certs();

        // Check the result
        assert!(result.is_some());

        // Clean up
        fs::remove_file(ca_cert_path).unwrap();
    }

    #[test]
    fn test_load_ca_certs_without_env_var() {
        // Call the function
        let result = load_ca_certs();

        // Check the result
        assert!(result.is_some());
    }
}
