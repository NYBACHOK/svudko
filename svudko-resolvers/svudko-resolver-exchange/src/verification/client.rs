use quinn::rustls::client::danger::HandshakeSignatureValid;
use quinn::rustls::crypto::CryptoProvider;
use quinn::rustls::pki_types::{CertificateDer, UnixTime};
use quinn::rustls::server::danger::{ClientCertVerified, ClientCertVerifier};
use quinn::rustls::{self, DistinguishedName, OtherError, SignatureScheme};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use svudko_common::POISONED_LOCK_MSG;
use svudko_common::hostname::Hostname;
use tokio::sync::mpsc::UnboundedSender;
use x509_parser::asn1_rs::FromDer;
use x509_parser::certificate::X509Certificate;

use crate::models::ClientId;

#[derive(Debug)]
pub struct WhiteListClientVerifier {
    provider: Arc<CryptoProvider>,
    trusted_signatures: Arc<RwLock<HashSet<String>>>,
    tx: UnboundedSender<ClientId>,
}

impl WhiteListClientVerifier {
    pub fn new(
        trusted_signatures: Arc<RwLock<HashSet<String>>>,
        tx: UnboundedSender<ClientId>,
    ) -> Self {
        let provider = Arc::new(rustls::crypto::ring::default_provider());

        Self {
            provider,
            trusted_signatures,
            tx,
        }
    }
}

impl ClientCertVerifier for WhiteListClientVerifier {
    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        &[]
    }

    fn verify_client_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _now: UnixTime,
    ) -> Result<ClientCertVerified, rustls::Error> {
        let mut hasher = Sha256::new();
        hasher.update(end_entity.as_ref());
        let signature = data_encoding::HEXLOWER.encode(&hasher.finalize());

        let allowed = self
            .trusted_signatures
            .read()
            .expect(POISONED_LOCK_MSG)
            .contains(&signature);

        if allowed {
            return Ok(rustls::server::danger::ClientCertVerified::assertion());
        }

        let (_, certificate) = X509Certificate::from_der(end_entity)
            .map_err(|e| rustls::Error::Other(OtherError(Arc::new(e))))?;

        let common_name = certificate
            .subject()
            .iter_common_name()
            .next()
            .ok_or_else(|| rustls::Error::General("Missing CommonName".into()))?
            .as_str()
            .map_err(|_| rustls::Error::General("Invalid CommonName encoding".into()))?;

        let _ = self.tx.send(ClientId {
            hostname: Hostname::new(common_name),
            id: signature,
        });

        Err(rustls::Error::General(
            "missing permission for connection".into(),
        ))
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &quinn::rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, quinn::rustls::Error> {
        quinn::rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &quinn::rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, quinn::rustls::Error> {
        quinn::rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.provider
            .signature_verification_algorithms
            .supported_schemes()
    }
}
