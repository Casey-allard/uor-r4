//! Offline Rust construction and validated loading of the integer bridge tables.
//!
//! `export` is an offline compiler operation using F64 transcendentals. The
//! loaded tables and all accesses from the numerical model are integer only.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use serde_json::json;
use uor_r4_core::report_output;

use crate::{invalid, Result};

pub const TOTAL: u64 = 1 << 48;
const ENTRIES: usize = 65535;
const BYTES: usize = ENTRIES * 16;
const SCHEMA: &str = "uor-r4.joint-integer-tables/1";

pub struct Tables {
    pub(crate) sigmoid: Vec<i32>,
    pub(crate) tanh: Vec<i32>,
    pub(crate) exp: Vec<u64>,
    pub sha256: String,
}

impl Tables {
    pub fn load(directory: &Path) -> Result<Self> {
        report_output::verify(directory)?;
        let metadata: serde_json::Value =
            serde_json::from_slice(&fs::read(directory.join("tables.json"))?)?;
        let path = directory.join("tables.bin");
        if metadata["schema"] != SCHEMA
            || metadata["entries"].as_u64() != Some(ENTRIES as u64)
            || metadata["probability_bits"] != 48
            || fs::metadata(&path)?.len() != BYTES as u64
            || metadata["payload_sha256"] != crate::sha256_file(&path)?
        {
            return Err(invalid("integer table schema, size or hash differs"));
        }
        let bytes = fs::read(path)?;
        let mut sigmoid = Vec::with_capacity(ENTRIES);
        let mut tanh = Vec::with_capacity(ENTRIES);
        let mut exp = Vec::with_capacity(ENTRIES);
        for row in bytes.chunks_exact(16) {
            let s = i32::from_le_bytes([row[0], row[1], row[2], row[3]]);
            let t = i32::from_le_bytes([row[4], row[5], row[6], row[7]]);
            let e = u64::from_le_bytes([
                row[8], row[9], row[10], row[11], row[12], row[13], row[14], row[15],
            ]);
            if !(0..=32768).contains(&s) || !(-16384..=16384).contains(&t) || e > TOTAL {
                return Err(invalid("integer table code outside declared range"));
            }
            sigmoid.push(s);
            tanh.push(t);
            exp.push(e);
        }
        if sigmoid.windows(2).any(|v| v[0] > v[1])
            || tanh.windows(2).any(|v| v[0] > v[1])
            || exp.windows(2).any(|v| v[0] < v[1])
            || exp[0] != TOTAL
            || sigmoid[32767] != 16384
            || tanh[32767] != 0
        {
            return Err(invalid("integer table monotonicity or origin differs"));
        }
        Ok(Self {
            sigmoid,
            tanh,
            exp,
            sha256: crate::sha256_file(&directory.join("tables.json"))?,
        })
    }
}

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
