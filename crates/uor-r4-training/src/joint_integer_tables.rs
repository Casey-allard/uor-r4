//! Offline Rust construction and shared validated loading of integer tables.
//!
//! `export` is an offline compiler operation using F64 transcendentals. The
//! loaded tables and all accesses from the numerical model are integer only.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use serde_json::json;
use uor_r4_core::report_output;

use crate::Result;

pub use uor_r4_integer::tables::{Tables, TOTAL};

const ENTRIES: usize = 65535;
const SCHEMA: &str = "uor-r4.joint-integer-tables/1";

/// Compile the complete finite input domains once, outside served computation.
pub fn export(directory: &Path) -> Result<()> {
    report_output::claim(directory)?;
    let mut file = File::create_new(directory.join("tables.bin"))?;
    for index in 0..ENTRIES {
        let x = (index as f64 - 32767.0) / 256.0;
        let sigmoid = (32768.0 / (1.0 + (-x).exp())).round() as i32;
        let tanh = (16384.0 * x.tanh()).round() as i32;
        let exp = ((-(index as f64) / 256.0).exp() * TOTAL as f64).round() as u64;
        file.write_all(&sigmoid.to_le_bytes())?;
        file.write_all(&tanh.to_le_bytes())?;
        file.write_all(&exp.to_le_bytes())?;
    }
    file.sync_all()?;
    let metadata = json!({
        "schema":SCHEMA, "entries":ENTRIES, "probability_bits":48,
        "payload_sha256":crate::sha256_file(&directory.join("tables.bin"))?,
        "layout":"Each little-endian row: sigmoid i32, tanh i32, exp u64",
        "sigmoid_tanh_domain":"signed affine codes -32767..32767 at exponent -8; output Q15/Q14 nearest ties away",
        "exp_domain":"negative affine differences 0..65534 at exponent -8; output Q48 nearest",
        "compiler":"offline Rust F64 exp/tanh; artifact hash binds actual table; no cross-platform bitwise claim",
        "runtime":"integer indexing only; table construction is not served computation"
    });
    fs::write(
        directory.join("tables.json"),
        serde_json::to_vec_pretty(&metadata)?,
    )?;
    report_output::seal(directory)?;
    report_output::verify(directory)?;
    Ok(())
}
