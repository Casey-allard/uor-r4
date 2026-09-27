//! Validated loading of retained integer lookup tables.
//! Offline floating table compilation belongs to the training tool, not this crate.

use std::fs;
use std::path::Path;

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
        crate::format::verify_sealed(directory)?;
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

    pub fn synthetic_for_test() -> Self {
        let mut sigmoid = Vec::with_capacity(ENTRIES);
        let mut tanh = Vec::with_capacity(ENTRIES);
        let mut exp = Vec::with_capacity(ENTRIES);
        for i in 0..ENTRIES {
            let s = ((i as i64 * 32768) / (ENTRIES as i64 - 1)) as i32;
            let t = (((i as i64 - 32767) * 16384) / 32767) as i32;
            let shift = (i / 1024).min(48) as u32;
            let e = TOTAL >> shift;
            sigmoid.push(s);
            tanh.push(t);
            exp.push(e);
        }
        sigmoid[32767] = 16384;
        tanh[32767] = 0;
        exp[0] = TOTAL;
        for i in 1..ENTRIES {
            if exp[i] > exp[i - 1] {
                exp[i] = exp[i - 1];
            }
        }
        Self {
            sigmoid,
            tanh,
            exp,
            sha256: "synthetic_tables_hash".to_owned(),
        }
    }
}
