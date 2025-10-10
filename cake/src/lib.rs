//! cake, an application of quiche

use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Generate self-signed certificate and private key for testing
pub fn generate_self_signed_cert(cert_path: &Path, key_path: &Path) -> Result<()> {
    // Generate certificate using rcgen
    let subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string()];

    let mut params = rcgen::CertificateParams::new(subject_alt_names)?;
    let mut distinguished_name = rcgen::DistinguishedName::new();
    distinguished_name.push(rcgen::DnType::CommonName, "localhost");
    params.distinguished_name = distinguished_name;

    // Generate key pair
    let key_pair = rcgen::KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    // Write to files
    let mut cert_file = File::create(cert_path)?;
    cert_file.write_all(cert_pem.as_bytes())?;

    let mut key_file = File::create(key_path)?;
    key_file.write_all(key_pem.as_bytes())?;

    Ok(())
}

/// Partial request structure to track HTTP/3 requests
#[derive(Default)]
pub struct PartialRequest {
    pub headers: Vec<quiche::h3::Header>,
    pub body: Vec<u8>,
    pub finished: bool,
}

/// Hex dump utility for debugging
pub fn hex_dump(buf: &[u8]) -> String {
    let vec: Vec<String> = buf.iter().map(|b| format!("{:02x}", b)).collect();
    vec.join("")
}
