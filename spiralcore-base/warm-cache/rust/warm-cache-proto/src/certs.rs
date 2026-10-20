//! Test-certificate loader; no private key is committed.
use std::path::PathBuf;
use std::sync::OnceLock;

use rustls::pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};

pub fn dir() -> PathBuf {
    std::env::var("WARM_CERT_DIR").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("certs"))
}

pub fn bytes(name: &str) -> &'static [u8] {
    static CACHE: OnceLock<std::sync::Mutex<std::collections::HashMap<String, &'static [u8]>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    let mut guard = cache.lock().unwrap();
    if let Some(b) = guard.get(name) { return b; }
    let data = std::fs::read(dir().join(name)).unwrap_or_else(|e| panic!("read {}: {}", name, e));
    let leaked: &'static [u8] = Box::leak(data.into_boxed_slice());
    guard.insert(name.to_string(), leaked);
    leaked
}

pub fn cert(name: &str) -> CertificateDer<'static> {
    CertificateDer::from_pem_slice(bytes(name)).unwrap_or_else(|e| panic!("cert {}: {}", name, e))
}

pub fn key(name: &str) -> PrivateKeyDer<'static> {
    PrivateKeyDer::from_pem_slice(bytes(name)).unwrap_or_else(|e| panic!("key {}: {}", name, e))
}
