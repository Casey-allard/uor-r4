//! The seven registered configs, checked on the host against the crate the pod
//! builds: each parses as a `DriverConfig`, its bytes are the grader's pinned
//! sha256, and the seven differ only in the pinned table (card C, G-Init).

use r_prime_zeta_vs_nulls_instrument::ARMS;
use sha2::{Digest, Sha256};
use uor_r4_discrete::training::DriverConfig;

/// The draft's `file_sha256` pins, in `ARMS` order.
const TABLE_FILES: [&str; 7] = [
    "83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b",
    "33b4e459cef8303f9d502f6f3e66327ec9e045026e7c8de896ad3ff350c2c415",
    "57225bbb63387d9a6f39e7cb6415aa0e317d19534633bb731a3d7559efd4c3b9",
    "e03d6aef42608e1db3f4d33a0b034fd2c113cd9c48f366e0b6319d20d4f77175",
    "229e93f58ad36cb3f1fc2fa4ab194ab8ce06798a67e39537352b9817cc5df345",
    "36fec6323ae1daf15d965ff5cd1161f66b7a4a8a8b0a3261e9094e9c125a9468",
    "25ec9bd29ce7fec20721826bab93f46caed8720cda6387cd43c5bddc62f0edea",
];

#[test]
fn the_seven_configs_parse_are_pinned_and_differ_only_in_the_table() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../evidence");
    let mut rest = None;
    for (i, (arm, pinned)) in ARMS.iter().enumerate() {
        let bytes = std::fs::read(dir.join(format!("{arm}.json"))).unwrap();
        let digest: String = Sha256::digest(&bytes).iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(digest, *pinned, "{arm}: config bytes moved from the pin");
        serde_json::from_slice::<DriverConfig>(&bytes).unwrap_or_else(|e| panic!("{arm}: {e}"));
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let table = value["codebook_initialization"]["table"].take();
        assert_eq!(value["codebook_initialization"]["kind"], "pinned_table_v1", "{arm}");
        assert_eq!(table["sha256"], TABLE_FILES[i], "{arm}: table pin differs from the draft");
        match &rest {
            None => rest = Some(value),
            Some(first) => assert_eq!(&value, first, "{arm} differs from p8 beyond the table"),
        }
    }
}
