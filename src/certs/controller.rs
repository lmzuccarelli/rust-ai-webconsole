use async_trait::async_trait;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use serde_derive::Deserialize;
use std::fs;
use std::io;

#[derive(Deserialize, Clone, Eq, PartialEq, Hash)]
struct KeyValue {
    name: String,
    value: String,
}

#[async_trait]
pub trait CertificateInterface {
    fn new(mode: String, cert_dir: Option<String>) -> Self;
    async fn get_public_cert(&self) -> io::Result<Vec<CertificateDer<'static>>>;
    async fn get_private_cert(&self) -> io::Result<PrivateKeyDer<'static>>;
}

pub struct ImplCertificateInterface {
    mode: String,
    certs_dir: Option<String>,
}

#[async_trait]
impl CertificateInterface for ImplCertificateInterface {
    fn new(mode: String, certs_dir: Option<String>) -> Self {
        return ImplCertificateInterface { mode, certs_dir };
    }

    async fn get_public_cert(&self) -> io::Result<Vec<CertificateDer<'static>>> {
        match self.mode.as_str() {
            "file" => load_public_key(format!("{}/ssl.cert", self.certs_dir.as_ref().unwrap())),
            &_ => return Err(error(format!("mode {} not available", self.mode))),
        }
    }

    async fn get_private_cert(&self) -> io::Result<PrivateKeyDer<'static>> {
        match self.mode.as_str() {
            "file" => load_private_key(format!("{}/ssl.key", self.certs_dir.as_ref().unwrap())),
            &_ => return Err(error(format!("mode {} not available", self.mode))),
        }
    }
}

fn load_public_key(dir: String) -> io::Result<Vec<CertificateDer<'static>>> {
    let certfile =
        fs::File::open(dir).map_err(|e| error(format!("failed to open {}: {}", "ssl.cert", e)))?;
    let mut reader = io::BufReader::new(certfile);
    rustls_pemfile::certs(&mut reader).collect()
}

fn load_private_key(dir: String) -> io::Result<PrivateKeyDer<'static>> {
    let keyfile =
        fs::File::open(dir).map_err(|e| error(format!("failed to open {}: {}", "ssl.key", e)))?;
    let mut reader = io::BufReader::new(keyfile);
    rustls_pemfile::private_key(&mut reader).map(|key| key.unwrap())
}

pub fn error(err: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}
