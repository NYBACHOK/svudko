use std::path::Path;

use anyhow::Context;
use rustls::pki_types::{CertificateDer, pem::PemObject};

pub const DOMAIN: &str = "app.sync.svudko";

pub async fn load_or_generate_cert(
    cert_file: &Path,
    key_file: &Path,
) -> anyhow::Result<CertificateDer<'static>> {
    if cert_file.exists() && key_file.exists() {
        let cert_pem = std::fs::read_to_string(cert_file)?;
        return Ok(CertificateDer::from_pem_slice(cert_pem.as_bytes())?.into_owned());
    }

    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_owned(), DOMAIN.to_owned()])
        .context("failed to generate self-signed certificate")?;

    let (write_cert, write_signing) = tokio::join!(
        tokio::fs::write(cert_file, cert.cert.pem()),
        tokio::fs::write(key_file, cert.signing_key.serialize_pem())
    );

    write_cert.context("failed to write certificate")?;
    write_signing.context("failed to write signing key")?;

    Ok(cert.cert.into())
}
