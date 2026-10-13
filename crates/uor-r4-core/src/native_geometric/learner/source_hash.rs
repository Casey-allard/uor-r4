//! Raw artifact hashing only; no model evaluator or predictor.
use sha2::{Digest, Sha256};
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|x| format!("{x:02x}")).collect()
}
pub fn sha256_hex(b: &[u8]) -> String {
    hex(&Sha256::digest(b))
}
