//! Complete D27 Rust Host-Only Consumer Validator & CLI for W2 Read-Code Capacity Diagnostic (`uor-r4.read-code-capacity/1`)
//! Sheet: `docs/harness/prep/m1-read-code-capacity-2026-10-17.md` (`1fd08b79dee4f5ff5812ff9ed5221700a7b0ee7bc760cdb00b14b2f4cefedc89`)
//! Readiness Map: `w2-bvg-source-readiness-20261018/REPORT.md` (`428d9bead6733593447a7bb44eecad8ef6a6d064cf6e51e913345cd925c180f4`)
//! Interface Prep: `w2-input-interface-preparation-20261018/REPORT.md` (`e8360545e43ce22cf820fbcb077f49acafe51f72efb22b4354056f82fe6bfdd3`)
//!
//! Zero external crate dependencies (`std` only). Implements:
//! 1. Pure-Rust FIPS 180-4 SHA-256 and recursive-descent JSON parser;
//! 2. Canonical `PreparedCorpus::from_bytes` (`UORT` 64-byte header + binary mask + BOS/predecessor
//!    eligibility from `crates/uor-r4-discrete/src/corpus.rs:143-297`);
//! 3. Exact original Q4 model/kernel/driver/domain pins (`e1836149...`, `931a511b...`, `cb090e85...`,
//!    `49fa448c...`, `f43742cb...`), explicitly refusing K16 qualification graph `035b3c...` substitution,
//!    plus fail-closed runtime-manifest file verification;
//! 4. Actual `evaluation-16404.json` (`4fff1d3c...`) 224-batch (`223x64 + 15 = 14,287` rows) flattener
//!    and validator using the real row fields (`top8`, `top1_correct`, `top8_correct`, `admitted_cells`,
//!    `address_rivals`, and exact target-admission indicator `target_admitted = admitted_cells - address_rivals`
//!    summing to `1,920`);
//! 5. Lossless 14,287-row DEV + compact TRAIN stream joins (`VERIFIED_RUNTIME_EVIDENCE` vs
//!    `SUMMARY_PREVIEW_ONLY`), exact nibble order (`root_first == code & 0xF`, `root_last == (code >> 12) & 0xF`),
//!    Precedence 0..5 decision-table evaluator, and full CLI (`main`).

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

pub const READ_CODE_CAPACITY_SCHEMA: &str = "uor-r4.read-code-capacity/1";
pub const DRIVER_SCHEMA: &str = "uor-r4.discrete-corpus-driver/1";
pub const COMPLETION_EVAL_SCHEMA: &str = "uor-r4.completion-evaluation/1";

// Section 4.9 & Authenticated Original Q4 Pinned Identities (6255411191 item 1)
pub const PINNED_SHEET_SHA256: &str =
    "1fd08b79dee4f5ff5812ff9ed5221700a7b0ee7bc760cdb00b14b2f4cefedc89";
pub const PINNED_READINESS_MAP_SHA256: &str =
    "428d9bead6733593447a7bb44eecad8ef6a6d064cf6e51e913345cd925c180f4";
pub const PINNED_INTERFACE_PREP_REPORT_SHA256: &str =
    "e8360545e43ce22cf820fbcb077f49acafe51f72efb22b4354056f82fe6bfdd3";
pub const PINNED_INTERFACE_PREP_MANIFEST_SHA256: &str =
    "c73bfeff532c5cd7437f0c09722a017d8f37324834e247d261f2aba598ad5f24";
pub const PINNED_INTERFACE_REVIEW_SHA256: &str =
    "a51226c82d8665afd3b4f5ca07a3b642684d3b3c22fa584299152e6fafef8e72";
pub const PINNED_RETAINED_INPUT_RECEIPT_SHA256: &str =
    "c020d3c01ae447b77b7eb69b9eaaa01ca439d995653795cd7e2db2265029d2b6";

// Authenticated Historical Q4 Model / Kernel / Driver / Domain Identities (w2-input-interface-preparation REPORT.md §2)
pub const PINNED_ORIGINAL_SOURCE_SHA256: &str =
    "e18361495276e491525674fb3fdce308b0ad787c7fd6860a811adb60dacd9e1a";
pub const PINNED_ORIGINAL_KERNEL_SHA256: &str =
    "931a511bece20641ae00c6b0e943f4f2a380c0f35a824d85fcc0447f1e1ccf5b";
pub const PINNED_ORIGINAL_DRIVER_SHA256: &str =
    "cb090e85e78805acd2b2347373340168a9e2f87aaf72a1dcb42f091e9ff1f7b4";
pub const PINNED_ORIGINAL_MODEL_DOMAIN_A_SHA256: &str =
    "49fa448cb3349ac5711966082a9c53918a693d45e11f282221d240cbb463fa73";
pub const PINNED_ORIGINAL_SNAPSHOT_DOMAIN_B_SHA256: &str =
    "f43742cb951b974856747fc3c162ff9cac1b4477df5296998481800e3a6c0512";
pub const FORBIDDEN_K16_QUAL_SOURCE_SHA256: &str =
    "035b3c127bb5a0eb84e50901cabb09391922e40ec6ffb3b4f638abccf6930b85";

pub const PINNED_Q1_CHECKPOINT_SHA256: &str =
    "a5a22dce8a174bbfe368cd8d166170a6c34bf59137b1418943a9d0422cae4cdd";
pub const PINNED_Q2_CHECKPOINT_SHA256: &str =
    "dde889aa6137c0e28e18dd5ae99a823b6850342286a0d065bdaa82de49b05822";
pub const PINNED_Q3_CHECKPOINT_SHA256: &str =
    "102beed4608f1ddd70d68039bdae8d9372b3c05de2dbc4bc98b96d7595e40982";
pub const PINNED_Q4_CHECKPOINT_SHA256: &str =
    "4477eb5c2480b0b85cc94036a2e15b0d22b654e25deee9c8717aa6e30f8ef7bf";
pub const PINNED_Q4_EVALUATION_SHA256: &str =
    "4fff1d3c04ff6107666b2b8281ef2bbedf3019eeff6bff682bbbb8687bf888c8";
pub const PINNED_Q4_EVALUATION_BYTES: usize = 2_906_559;
pub const PINNED_DEV_TOKENS_SHA256: &str =
    "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b";
pub const PINNED_DEV_MASK_SHA256: &str =
    "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078";
pub const PINNED_TRAIN_TOKENS_SHA256: &str =
    "3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8";
pub const PINNED_TRAIN_MASK_SHA256: &str =
    "b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3";
pub const PINNED_TOKENIZER_SHA256: &str =
    "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89";
pub const PINNED_ORDERED_ROW_BINDING_SHA256: &str =
    "e4ef44fb81d6c3a833212e4857117db71cafb8259b81c286e5b1391129d74446";
pub const PINNED_GEOMETRY_SHA256: &str =
    "46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327";
pub const PINNED_D34_RECORD_SHA256: &str =
    "2717d34bcfcdd1486991925ecab066266a6b56ba34b89727c37f78c821fe3bec";

pub const UORT_MAGIC: &[u8; 4] = b"UORT";
pub const UORT_VERSION: u32 = 1;
pub const UORT_HEADER_BYTES: usize = 64;
pub const PINNED_BOS_TOKEN: &str = "<|bos|>";
pub const PINNED_BOS_ID: u16 = 0;
pub const CONTEXT_WINDOW: usize = 192;

pub const SPLIT_TAG_TRAIN: u64 = 0;
pub const SPLIT_TAG_DEV: u64 = 1;

pub const TRAIN_TOKENS_BYTES: usize = 15_025_658;
pub const TRAIN_PAYLOAD_TOKENS: usize = 7_512_797;
pub const TRAIN_MASK_BYTES: usize = 7_512_797;
pub const TRAIN_DOCUMENTS: usize = 60_000;

pub const DEV_TOKENS_BYTES: usize = 85_776;
pub const DEV_PAYLOAD_TOKENS: usize = 42_856;
pub const DEV_MASK_BYTES: usize = 42_856;
pub const DEV_DOCUMENTS: usize = 400;

pub const DEV_ROWS: usize = 14_287;
pub const DEV_BATCH_ROWS: usize = 64;
pub const DEV_EXPECTED_BATCHES: usize = 224; // 223 * 64 + 15 = 14,287
pub const DEV_LAST_BATCH_ROWS: usize = 15;
pub const MATCHED_TRAIN_TARGETS: usize = 1_049_856;
pub const FULL_TRAIN_TARGETS: usize = 2_425_925;
pub const COMPACT_TRAIN_PRIMARY_BYTES: usize = 4_199_424;
pub const COMPACT_TRAIN_FULL_BYTES: usize = 9_703_700;
pub const SENTINEL_U16: u16 = 0xFFFF;
pub const VOCABULARY: u16 = 4096;
pub const MAX_CODE_COUNT: usize = 65_536;
pub const Q4_EPOCH: i64 = 407;
pub const Q4_WRITER_TOP1: i64 = 149;
pub const Q4_WRITER_TOP8: i64 = 747;
pub const Q4_WRITER_ADMITTED: i64 = 1_920;
pub const MIN_COVERAGE_ROWS: i64 = 12_859;
pub const MIN_MULTIPLICITY_CODES: i64 = 1_024;
pub const MIN_DISCORDANCE_NORMAL: i64 = 100;
pub const FULL_PANEL_MIN_FLOOR: i64 = 24;

// --- Pure-Rust FIPS 180-4 SHA-256 ---

pub struct Sha256 {
    state: [u32; 8],
    buffer: [u8; 64],
    buffered: usize,
    total_bits: u64,
}

impl Sha256 {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    pub fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c,
                0x1f83d9ab, 0x5be0cd19,
            ],
            buffer: [0u8; 64],
            buffered: 0,
            total_bits: 0,
        }
    }

    fn compress(state: &mut [u32; 8], block: &[u8; 64]) {
        let mut w = [0u32; 64];
        for (i, chunk) in block.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(Self::K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }

    pub fn update(&mut self, mut data: &[u8]) {
        self.total_bits = self.total_bits.wrapping_add((data.len() as u64) * 8);
        if self.buffered > 0 {
            let take = (64 - self.buffered).min(data.len());
            self.buffer[self.buffered..self.buffered + take].copy_from_slice(&data[..take]);
            self.buffered += take;
            data = &data[take..];
            if self.buffered == 64 {
                Self::compress(&mut self.state, &self.buffer);
                self.buffered = 0;
            }
        }
        while data.len() >= 64 {
            let mut block = [0u8; 64];
            block.copy_from_slice(&data[..64]);
            Self::compress(&mut self.state, &block);
            data = &data[64..];
        }
        if !data.is_empty() {
            self.buffer[..data.len()].copy_from_slice(data);
            self.buffered = data.len();
        }
    }

    pub fn finalize_hex(mut self) -> String {
        let bit_len = self.total_bits;
        self.buffer[self.buffered] = 0x80;
        self.buffered += 1;
        if self.buffered > 56 {
            for b in &mut self.buffer[self.buffered..64] {
                *b = 0;
            }
            Self::compress(&mut self.state, &self.buffer);
            self.buffered = 0;
        }
        for b in &mut self.buffer[self.buffered..56] {
            *b = 0;
        }
        self.buffer[56..64].copy_from_slice(&bit_len.to_be_bytes());
        Self::compress(&mut self.state, &self.buffer);
        let mut out = String::with_capacity(64);
        for word in self.state {
            out.push_str(&format!("{word:08x}"));
        }
        out
    }
}

pub fn sha256_bytes(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    h.finalize_hex()
}

pub fn is_hex64(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

// --- Pure-Rust Recursive-Descent JSON Parser ---

#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Int(i64),
    Str(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

impl JsonValue {
    pub fn as_object(&self) -> Result<&BTreeMap<String, JsonValue>, String> {
        match self {
            JsonValue::Object(m) => Ok(m),
            _ => Err("expected JSON object".to_string()),
        }
    }
    pub fn as_array(&self) -> Result<&Vec<JsonValue>, String> {
        match self {
            JsonValue::Array(a) => Ok(a),
            _ => Err("expected JSON array".to_string()),
        }
    }
    pub fn as_str(&self) -> Result<&str, String> {
        match self {
            JsonValue::Str(s) => Ok(s.as_str()),
            _ => Err("expected JSON string".to_string()),
        }
    }
    pub fn as_i64(&self) -> Result<i64, String> {
        match self {
            JsonValue::Int(n) => Ok(*n),
            _ => Err("expected strict JSON integer".to_string()),
        }
    }
    pub fn as_bool(&self) -> Result<bool, String> {
        match self {
            JsonValue::Bool(b) => Ok(*b),
            _ => Err("expected JSON boolean".to_string()),
        }
    }
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(m) => m.get(key),
            _ => None,
        }
    }
}

struct JsonParser<'a> {
    src: &'a [u8],
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn skip_ws(&mut self) {
        while self.pos < self.src.len()
            && matches!(self.src[self.pos], b' ' | b'\t' | b'\r' | b'\n')
        {
            self.pos += 1;
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_ws();
        if self.pos >= self.src.len() {
            return Err("unexpected EOF in JSON".to_string());
        }
        match self.src[self.pos] {
            b'{' => self.parse_object(),
            b'[' => self.parse_array(),
            b'"' => self.parse_string().map(JsonValue::Str),
            b't' => {
                if self.src[self.pos..].starts_with(b"true") {
                    self.pos += 4;
                    Ok(JsonValue::Bool(true))
                } else {
                    Err("invalid token starting with 't'".to_string())
                }
            }
            b'f' => {
                if self.src[self.pos..].starts_with(b"false") {
                    self.pos += 5;
                    Ok(JsonValue::Bool(false))
                } else {
                    Err("invalid token starting with 'f'".to_string())
                }
            }
            b'n' => {
                if self.src[self.pos..].starts_with(b"null") {
                    self.pos += 4;
                    Ok(JsonValue::Null)
                } else {
                    Err("invalid token starting with 'n'".to_string())
                }
            }
            b'-' | b'0'..=b'9' => self.parse_int(),
            c => Err(format!("unexpected JSON byte {c:#04x} at pos {}", self.pos)),
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        if self.src.get(self.pos) != Some(&b'"') {
            return Err("expected '\"'".to_string());
        }
        self.pos += 1;
        let mut out = Vec::new();
        while self.pos < self.src.len() {
            let b = self.src[self.pos];
            self.pos += 1;
            if b == b'"' {
                return String::from_utf8(out).map_err(|e| e.to_string());
            } else if b == b'\\' {
                if self.pos >= self.src.len() {
                    return Err("unterminated escape in JSON string".to_string());
                }
                let esc = self.src[self.pos];
                self.pos += 1;
                match esc {
                    b'"' | b'\\' | b'/' => out.push(esc),
                    b'b' => out.push(0x08),
                    b'f' => out.push(0x0c),
                    b'n' => out.push(b'\n'),
                    b'r' => out.push(b'\r'),
                    b't' => out.push(b'\t'),
                    b'u' => {
                        if self.pos + 4 > self.src.len() {
                            return Err("short \\uXXXX escape".to_string());
                        }
                        let hex = std::str::from_utf8(&self.src[self.pos..self.pos + 4])
                            .map_err(|e| e.to_string())?;
                        let cp = u32::from_str_radix(hex, 16).map_err(|e| e.to_string())?;
                        self.pos += 4;
                        if let Some(ch) = char::from_u32(cp) {
                            let mut buf = [0u8; 4];
                            out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        } else {
                            return Err("invalid unicode scalar in \\u escape".to_string());
                        }
                    }
                    _ => return Err(format!("unsupported JSON escape \\{}", esc as char)),
                }
            } else {
                out.push(b);
            }
        }
        Err("unterminated JSON string".to_string())
    }

    fn parse_int(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        if self.src.get(self.pos) == Some(&b'-') {
            self.pos += 1;
        }
        while self.pos < self.src.len() && self.src[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        if self.pos < self.src.len() && matches!(self.src[self.pos], b'.' | b'e' | b'E') {
            return Err(format!("floating-point number forbidden in integer JSON field at pos {start}"));
        }
        let s = std::str::from_utf8(&self.src[start..self.pos]).map_err(|e| e.to_string())?;
        let n = s
            .parse::<i64>()
            .map_err(|e| format!("invalid integer {s:?}: {e}"))?;
        Ok(JsonValue::Int(n))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // '['
        self.skip_ws();
        let mut items = Vec::new();
        if self.src.get(self.pos) == Some(&b']') {
            self.pos += 1;
            return Ok(JsonValue::Array(items));
        }
        loop {
            items.push(self.parse_value()?);
            self.skip_ws();
            match self.src.get(self.pos) {
                Some(&b',') => {
                    self.pos += 1;
                }
                Some(&b']') => {
                    self.pos += 1;
                    return Ok(JsonValue::Array(items));
                }
                _ => return Err(format!("expected ',' or ']' in array at pos {}", self.pos)),
            }
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // '{'
        self.skip_ws();
        let mut map = BTreeMap::new();
        if self.src.get(self.pos) == Some(&b'}') {
            self.pos += 1;
            return Ok(JsonValue::Object(map));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.skip_ws();
            if self.src.get(self.pos) != Some(&b':') {
                return Err(format!("expected ':' after key {key:?} at pos {}", self.pos));
            }
            self.pos += 1;
            let val = self.parse_value()?;
            map.insert(key, val);
            self.skip_ws();
            match self.src.get(self.pos) {
                Some(&b',') => {
                    self.pos += 1;
                }
                Some(&b'}') => {
                    self.pos += 1;
                    return Ok(JsonValue::Object(map));
                }
                _ => return Err(format!("expected ',' or '}}' in object at pos {}", self.pos)),
            }
        }
    }
}

pub fn parse_json(bytes: &[u8]) -> Result<JsonValue, String> {
    let mut p = JsonParser { src: bytes, pos: 0 };
    let v = p.parse_value()?;
    p.skip_ws();
    if p.pos != bytes.len() {
        return Err(format!("trailing bytes after JSON root at pos {}", p.pos));
    }
    Ok(v)
}

fn req_i64(obj: &JsonValue, field: &str, min_v: Option<i64>, max_v: Option<i64>) -> Result<i64, String> {
    let v = obj
        .get(field)
        .ok_or_else(|| format!("missing integer field {field}"))?
        .as_i64()
        .map_err(|e| format!("{field}: {e}"))?;
    if let Some(lo) = min_v {
        if v < lo {
            return Err(format!("{field}={v} < minimum {lo}"));
        }
    }
    if let Some(hi) = max_v {
        if v > hi {
            return Err(format!("{field}={v} > maximum {hi}"));
        }
    }
    Ok(v)
}

fn req_bool(obj: &JsonValue, field: &str) -> Result<bool, String> {
    obj.get(field)
        .ok_or_else(|| format!("missing boolean field {field}"))?
        .as_bool()
        .map_err(|e| format!("{field}: {e}"))
}

fn req_str<'a>(obj: &'a JsonValue, field: &str) -> Result<&'a str, String> {
    obj.get(field)
        .ok_or_else(|| format!("missing string field {field}"))?
        .as_str()
        .map_err(|e| format!("{field}: {e}"))
}

// --- Canonical UORT & PreparedCorpus Reader (crates/uor-r4-discrete/src/corpus.rs:143-297) ---

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalCorpusTargetRow {
    pub row_ordinal: usize,
    pub document_id: usize,
    pub record: u64,
    pub doc_start: usize,
    pub context_start: usize,
    pub query_occurrence: usize,
    pub query_token: u16,
    pub target_position: usize,
    pub target_token: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalPreparedCorpusSummary {
    pub uort_version: u32,
    pub payload_tokens: usize,
    pub vocabulary: u32,
    pub bos_id: u16,
    pub document_count: usize,
    pub eligible_target_count: usize,
    pub rows: Vec<CanonicalCorpusTargetRow>,
}

pub fn parse_uort_header(
    bytes: &[u8],
    expected_payload_tokens: Option<usize>,
    expected_vocabulary: u32,
) -> Result<(u32, usize, u32), String> {
    if bytes.len() < UORT_HEADER_BYTES {
        return Err(format!(
            "token file is shorter than the 64-byte UORT header (len={})",
            bytes.len()
        ));
    }
    if &bytes[0..4] != UORT_MAGIC {
        return Err("token file does not start with UORT magic".to_string());
    }
    let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    if version != UORT_VERSION {
        return Err(format!("unsupported UORT version {version} != {UORT_VERSION}"));
    }
    let count = u64::from_le_bytes([
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    ]) as usize;
    if count == 0 {
        return Err("UORT token count is zero".to_string());
    }
    if let Some(exp) = expected_payload_tokens {
        if count != exp {
            return Err(format!("UORT header token count {count} != expected {exp}"));
        }
    }
    let vocabulary = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    if vocabulary != expected_vocabulary {
        return Err(format!(
            "UORT header vocabulary {vocabulary} != expected {expected_vocabulary}"
        ));
    }
    if bytes[20..64].iter().any(|&b| b != 0) {
        return Err("UORT header reserved bytes 20..64 are non-zero".to_string());
    }
    Ok((version, count, vocabulary))
}

pub fn parse_prepared_corpus_uort(
    tokens_bytes: &[u8],
    mask_bytes: &[u8],
    bos_id: u16,
    split_tag: u64,
    context: usize,
    expected_payload_tokens: Option<usize>,
    expected_documents: Option<usize>,
    expected_eligible_targets: Option<usize>,
    limit_rows: Option<usize>,
) -> Result<CanonicalPreparedCorpusSummary, String> {
    if bos_id != PINNED_BOS_ID {
        return Err(format!("BOS id {bos_id} != pinned {PINNED_BOS_ID}"));
    }
    if context == 0 {
        return Err("context window is zero".to_string());
    }
    let (version, count, vocabulary) =
        parse_uort_header(tokens_bytes, expected_payload_tokens, VOCABULARY as u32)?;
    let expected_len = UORT_HEADER_BYTES
        .checked_add(count.checked_mul(2).ok_or("UORT byte count overflow")?)
        .ok_or("UORT byte count overflow")?;
    if tokens_bytes.len() != expected_len {
        return Err(format!(
            "UORT token file byte length {} != 64 + 2 * {} ({})",
            tokens_bytes.len(),
            count,
            expected_len
        ));
    }
    if mask_bytes.len() != count {
        return Err(format!(
            "response mask length {} != UORT token count {}",
            mask_bytes.len(),
            count
        ));
    }

    let mut tokens = Vec::with_capacity(count);
    for chunk in tokens_bytes[UORT_HEADER_BYTES..].chunks_exact(2) {
        let id = u16::from_le_bytes([chunk[0], chunk[1]]);
        if (id as u32) >= vocabulary {
            return Err(format!("token id {id} >= vocabulary {vocabulary}"));
        }
        tokens.push(id);
    }
    for (p, &m) in mask_bytes.iter().enumerate() {
        if m != 0 && m != 1 {
            return Err(format!("response mask byte {m} at position {p} is not binary (0 or 1)"));
        }
    }
    if tokens.first().copied() != Some(bos_id) {
        return Err("corpus does not start with BOS token".to_string());
    }

    let documents: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter_map(|(i, &id)| (id == bos_id).then_some(i))
        .collect();
    if documents.is_empty() {
        return Err("corpus contains no BOS document starts".to_string());
    }
    if (documents.len() as u64) >= (1u64 << 62) {
        return Err("document count exceeds 62-bit split-prefixed record id space".to_string());
    }
    if let Some(exp_docs) = expected_documents {
        if documents.len() != exp_docs {
            return Err(format!("document count {} != expected {exp_docs}", documents.len()));
        }
    }

    let eligible: Vec<usize> = tokens
        .iter()
        .copied()
        .zip(mask_bytes.iter().copied())
        .enumerate()
        .filter_map(|(i, (id, m))| (i > 0 && id != bos_id && m == 1).then_some(i))
        .collect();
    if eligible.is_empty() {
        return Err("corpus contains no eligible masked response targets".to_string());
    }
    if let Some(exp_elig) = expected_eligible_targets {
        if eligible.len() != exp_elig {
            return Err(format!(
                "eligible target count {} != expected {exp_elig}",
                eligible.len()
            ));
        }
    }

    let take_n = match limit_rows {
        Some(n) => {
            if n > eligible.len() {
                return Err(format!(
                    "requested limit_rows {n} > eligible targets {}",
                    eligible.len()
                ));
            }
            n
        }
        None => eligible.len(),
    };

    let mut rows = Vec::with_capacity(take_n);
    for (ordinal, &position) in eligible.iter().take(take_n).enumerate() {
        let doc = match documents.binary_search(&position) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };
        let doc_start = *documents
            .get(doc)
            .ok_or_else(|| format!("missing document start for ordinal {ordinal}"))?;
        let start = position.saturating_sub(context).max(doc_start);
        if start >= position {
            return Err(format!(
                "eligible target ordinal {ordinal} at position {position} has no in-document predecessor (doc_start={doc_start})"
            ));
        }
        let query_occurrence = position - 1;
        let query_token = tokens[query_occurrence];
        let target_token = tokens[position];
        let record = ((split_tag & 0x3) << 62) | (doc as u64);
        rows.push(CanonicalCorpusTargetRow {
            row_ordinal: ordinal,
            document_id: doc,
            record,
            doc_start,
            context_start: start,
            query_occurrence,
            query_token,
            target_position: position,
            target_token,
        });
    }

    Ok(CanonicalPreparedCorpusSummary {
        uort_version: version,
        payload_tokens: count,
        vocabulary,
        bos_id,
        document_count: documents.len(),
        eligible_target_count: eligible.len(),
        rows,
    })
}

pub fn verify_code_endpoints(code: u16, root_first: u8, root_last: u8) -> Result<(), String> {
    let r0 = (code & 0xF) as u8;
    let r3 = ((code >> 12) & 0xF) as u8;
    if root_first != r0 {
        return Err(format!(
            "root_first {root_first} != low nibble r0 ({r0}) of code {code:#06x}"
        ));
    }
    if root_last != r3 {
        return Err(format!(
            "root_last {root_last} != high nibble r3 ({r3}) of code {code:#06x}"
        ));
    }
    Ok(())
}

// --- Sealed Q4 evaluation-16404.json (4fff1d3c...) Batch Flattener & Row Validator (6255411191 items 2 & 3) ---

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SealedEvalRow {
    pub row_ordinal: usize,
    pub top8: [u16; 8],
    pub top1_correct: u8,
    pub top8_correct: u8,
    pub admitted_cells: u8,
    pub address_rivals: u8,
    pub target_admitted: u8, // exact GPU indicator: admitted_cells - address_rivals in {0, 1}
    pub token_hinge: i64,
    pub address_hinge: i64,
    pub total_loss_numerator: i64,
    pub token_rivals: i64,
}

pub fn flatten_sealed_evaluation_batches(
    eval_doc: &JsonValue,
    dev_corpus_rows: &[CanonicalCorpusTargetRow],
    strict_k8_q4_pins: bool,
) -> Result<Vec<SealedEvalRow>, String> {
    let eval = if eval_doc.get("schema").and_then(|v| v.as_str().ok()) == Some(COMPLETION_EVAL_SCHEMA) {
        eval_doc
            .get("evaluation")
            .ok_or("completion evaluation wrapper missing inner evaluation")?
    } else if eval_doc
        .get("baseline_evaluation")
        .and_then(|e| e.get("schema"))
        .and_then(|v| v.as_str().ok())
        == Some(DRIVER_SCHEMA)
    {
        eval_doc
            .get("baseline_evaluation")
            .ok_or("missing baseline_evaluation")?
    } else {
        eval_doc
    };

    if req_str(eval, "schema")? != DRIVER_SCHEMA {
        return Err(format!("sealed evaluation schema != {DRIVER_SCHEMA}"));
    }
    if req_bool(eval, "evaluation_mutates_model")? {
        return Err("sealed evaluation_mutates_model must be false".to_string());
    }
    if req_i64(eval, "updates_applied_by_evaluation", Some(0), Some(0))? != 0 {
        return Err("sealed updates_applied_by_evaluation must be 0".to_string());
    }
    if strict_k8_q4_pins {
        if req_i64(eval, "model_epoch", None, None)? != Q4_EPOCH {
            return Err(format!("sealed evaluation model_epoch != {Q4_EPOCH}"));
        }
        if let Some(v) = eval.get("source_sha256") {
            if v.as_str()? != PINNED_ORIGINAL_SOURCE_SHA256 {
                return Err("sealed evaluation source_sha256 != PINNED_ORIGINAL_SOURCE_SHA256".to_string());
            }
        }
        if let Some(v) = eval.get("kernel_sha256") {
            if v.as_str()? != PINNED_ORIGINAL_KERNEL_SHA256 {
                return Err("sealed evaluation kernel_sha256 != PINNED_ORIGINAL_KERNEL_SHA256".to_string());
            }
        }
        if let Some(v) = eval.get("operational_driver_sha256") {
            if v.as_str()? != PINNED_ORIGINAL_DRIVER_SHA256 {
                return Err("sealed evaluation operational_driver_sha256 != PINNED_ORIGINAL_DRIVER_SHA256".to_string());
            }
        }
        if let Some(v) = eval.get("model_sha256") {
            if v.as_str()? != PINNED_ORIGINAL_MODEL_DOMAIN_A_SHA256 {
                return Err("sealed evaluation model_sha256 != PINNED_ORIGINAL_MODEL_DOMAIN_A_SHA256".to_string());
            }
        }
        if let Some(v) = eval.get("rows") {
            if v.as_i64()? != (DEV_ROWS as i64) {
                return Err("sealed evaluation top-level rows != DEV_ROWS".to_string());
            }
        }
        if let Some(v) = eval.get("admitted_rows") {
            if v.as_i64()? != Q4_WRITER_ADMITTED {
                return Err("sealed evaluation top-level admitted_rows != Q4_WRITER_ADMITTED".to_string());
            }
        }
        if let Some(v) = eval.get("top1_correct_rows") {
            if v.as_i64()? != Q4_WRITER_TOP1 {
                return Err("sealed evaluation top-level top1_correct_rows != Q4_WRITER_TOP1".to_string());
            }
        }
        if let Some(v) = eval.get("top8_correct_rows") {
            if v.as_i64()? != Q4_WRITER_TOP8 {
                return Err("sealed evaluation top-level top8_correct_rows != Q4_WRITER_TOP8".to_string());
            }
        }
    }

    let batches = eval
        .get("batches")
        .ok_or("sealed evaluation missing `batches` array (batches[*].rows)")?
        .as_array()?;
    let expected_batches = dev_corpus_rows.len().div_ceil(DEV_BATCH_ROWS);
    if batches.len() != expected_batches {
        return Err(format!(
            "sealed evaluation batch count {} != expected {expected_batches}",
            batches.len()
        ));
    }

    let mut out = Vec::with_capacity(dev_corpus_rows.len());
    let mut top1_sum = 0i64;
    let mut top8_sum = 0i64;
    let mut target_admitted_sum = 0i64;

    for (b_idx, batch) in batches.iter().enumerate() {
        let b_rows = batch
            .get("rows")
            .ok_or_else(|| format!("sealed evaluation batch[{b_idx}] missing `rows`"))?
            .as_array()?;
        let expected_b_len = if b_idx + 1 < expected_batches {
            DEV_BATCH_ROWS
        } else {
            dev_corpus_rows.len() - b_idx * DEV_BATCH_ROWS
        };
        if b_rows.len() != expected_b_len {
            return Err(format!(
                "sealed evaluation batch[{b_idx}] row count {} != expected {expected_b_len}",
                b_rows.len()
            ));
        }

        for r_val in b_rows {
            let ordinal = out.len();
            let c_row = &dev_corpus_rows[ordinal];
            let top8_arr = r_val
                .get("top8")
                .ok_or_else(|| format!("sealed row[{ordinal}] missing `top8`"))?
                .as_array()?;
            if top8_arr.len() != 8 {
                return Err(format!("sealed row[{ordinal}].top8 len {} != 8", top8_arr.len()));
            }
            let mut top8 = [0u16; 8];
            let mut seen = BTreeSet::new();
            for (k, tv) in top8_arr.iter().enumerate() {
                let tid = tv.as_i64()?;
                if tid < 0 || tid >= (VOCABULARY as i64) || !seen.insert(tid) {
                    return Err(format!(
                        "sealed row[{ordinal}].top8[{k}] invalid or duplicate token {tid}"
                    ));
                }
                top8[k] = tid as u16;
            }

            let h1 = req_i64(r_val, "top1_correct", Some(0), Some(1))? as u8;
            let h8 = req_i64(r_val, "top8_correct", Some(0), Some(1))? as u8;
            let cells = req_i64(r_val, "admitted_cells", Some(1), Some(8))? as u8;
            let ar = req_i64(r_val, "address_rivals", Some(0), Some(8))? as u8;
            if cells < ar {
                return Err(format!(
                    "sealed row[{ordinal}] admitted_cells ({cells}) < address_rivals ({ar})"
                ));
            }
            let target_admitted = cells - ar;
            if target_admitted > 1 {
                return Err(format!(
                    "sealed row[{ordinal}] target_admitted = admitted_cells - address_rivals = {target_admitted} not in {{0, 1}}"
                ));
            }
            if !(h1 <= h8 && h8 <= target_admitted) {
                return Err(format!(
                    "sealed row[{ordinal}] rank/admission nesting violated: h1={h1}, h8={h8}, target_admitted={target_admitted}"
                ));
            }

            let t_hinge = req_i64(r_val, "token_hinge", Some(0), None)?;
            let a_hinge = req_i64(r_val, "address_hinge", Some(0), None)?;
            let total_loss = req_i64(r_val, "total_loss_numerator", Some(0), None)?;
            let t_rivals = req_i64(r_val, "token_rivals", Some(0), Some((VOCABULARY as i64) - 1))?;
            if total_loss != 8 * t_hinge + 32 * a_hinge {
                return Err(format!(
                    "sealed row[{ordinal}] total_loss_numerator {total_loss} != 8*{t_hinge} + 32*{a_hinge}"
                ));
            }

            // Join to ordered DEV corpus target_token
            let target_tok = c_row.target_token;
            if (h1 == 1) != (top8[0] == target_tok) {
                return Err(format!(
                    "sealed row[{ordinal}] top1_correct ({h1}) != (top8[0] {} == target_token {target_tok})",
                    top8[0]
                ));
            }
            if (h8 == 1) != top8.contains(&target_tok) {
                return Err(format!(
                    "sealed row[{ordinal}] top8_correct ({h8}) != top8.contains({target_tok})"
                ));
            }
            if let Some(tp_val) = r_val.get("target_position") {
                if tp_val.as_i64()? != (c_row.target_position as i64) {
                    return Err(format!("sealed row[{ordinal}].target_position mismatch"));
                }
            }
            if let Some(tt_val) = r_val.get("target_token") {
                if tt_val.as_i64()? != (target_tok as i64) {
                    return Err(format!("sealed row[{ordinal}].target_token mismatch"));
                }
            }

            top1_sum += h1 as i64;
            top8_sum += h8 as i64;
            target_admitted_sum += target_admitted as i64;
            out.push(SealedEvalRow {
                row_ordinal: ordinal,
                top8,
                top1_correct: h1,
                top8_correct: h8,
                admitted_cells: cells,
                address_rivals: ar,
                target_admitted,
                token_hinge: t_hinge,
                address_hinge: a_hinge,
                total_loss_numerator: total_loss,
                token_rivals: t_rivals,
            });
        }
    }

    if out.len() != dev_corpus_rows.len() {
        return Err(format!(
            "flattened sealed evaluation rows {} != DEV rows {}",
            out.len(),
            dev_corpus_rows.len()
        ));
    }
    if strict_k8_q4_pins {
        if top1_sum != Q4_WRITER_TOP1 {
            return Err(format!("sealed evaluation top1_sum {top1_sum} != {Q4_WRITER_TOP1}"));
        }
        if top8_sum != Q4_WRITER_TOP8 {
            return Err(format!("sealed evaluation top8_sum {top8_sum} != {Q4_WRITER_TOP8}"));
        }
        if target_admitted_sum != Q4_WRITER_ADMITTED {
            return Err(format!(
                "sealed evaluation target_admitted_sum (sum(admitted_cells - address_rivals)) {target_admitted_sum} != {Q4_WRITER_ADMITTED}"
            ));
        }
    }
    Ok(out)
}

// --- Per-Row W2 DEV Lossless Verification (6255411191 items 1, 2, 3) ---

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PerRowVerificationSummary {
    pub rows_verified: usize,
    pub dev_corpus_joined: bool,
    pub sealed_eval_joined: bool,
    pub token_cells_joined: bool,
    pub writer_hit1_sum: i64,
    pub writer_hit8_sum: i64,
    pub target_admitted_sum: i64,
    pub admitted_cell_count_sum: i64,
    pub address_rival_count_sum: i64,
    pub table_hit1_sum: i64,
    pub table_hit8_sum: i64,
    pub unigram_hit1_sum: i64,
    pub bigram_hit1_sum: i64,
    pub seen_code_sum: i64,
    pub b_u: i64,
    pub c_u: i64,
    pub b_g: i64,
    pub c_g: i64,
    pub b_w: i64,
    pub c_w: i64,
}

pub fn verify_per_row_records(
    rows: &[JsonValue],
    expected_dev_rows: usize,
    expected_admitted_k: usize,
    dev_corpus_rows: Option<&[CanonicalCorpusTargetRow]>,
    sealed_eval_rows: Option<&[SealedEvalRow]>,
    token_cells: Option<&[u16]>,
) -> Result<PerRowVerificationSummary, String> {
    if rows.len() != expected_dev_rows {
        return Err(format!(
            "per-row count {} != expected {expected_dev_rows}",
            rows.len()
        ));
    }
    if let Some(dc) = dev_corpus_rows {
        if dc.len() != expected_dev_rows {
            return Err(format!("dev_corpus_rows len {} != {expected_dev_rows}", dc.len()));
        }
    }
    if let Some(se) = sealed_eval_rows {
        if se.len() != expected_dev_rows {
            return Err(format!("sealed_eval_rows len {} != {expected_dev_rows}", se.len()));
        }
    }
    if let Some(tc) = token_cells {
        if tc.len() != (VOCABULARY as usize) {
            return Err(format!("token_cells len {} != {VOCABULARY}", tc.len()));
        }
        for (tok_id, &cell) in tc.iter().enumerate() {
            if cell > 255 {
                return Err(format!("token_cells[{tok_id}] = {cell} > 255"));
            }
        }
    }

    let mut summary = PerRowVerificationSummary {
        rows_verified: rows.len(),
        dev_corpus_joined: dev_corpus_rows.is_some(),
        sealed_eval_joined: sealed_eval_rows.is_some(),
        token_cells_joined: token_cells.is_some(),
        writer_hit1_sum: 0,
        writer_hit8_sum: 0,
        target_admitted_sum: 0,
        admitted_cell_count_sum: 0,
        address_rival_count_sum: 0,
        table_hit1_sum: 0,
        table_hit8_sum: 0,
        unigram_hit1_sum: 0,
        bigram_hit1_sum: 0,
        seen_code_sum: 0,
        b_u: 0,
        c_u: 0,
        b_g: 0,
        c_g: 0,
        b_w: 0,
        c_w: 0,
    };

    for (idx, row) in rows.iter().enumerate() {
        let ord_val = req_i64(row, "row_ordinal", Some(0), Some((expected_dev_rows - 1) as i64))? as usize;
        if ord_val != idx {
            return Err(format!("row[{idx}].row_ordinal {ord_val} != {idx}"));
        }
        let doc_id = req_i64(row, "document_id", Some(0), None)? as usize;
        let target_pos = req_i64(row, "target_position", Some(1), None)? as usize;
        let query_occ = match row.get("query_occurrence") {
            Some(v) => v.as_i64()? as usize,
            None => target_pos - 1,
        };
        if query_occ != target_pos - 1 {
            return Err(format!(
                "row[{idx}].query_occurrence {query_occ} != target_position - 1 ({})",
                target_pos - 1
            ));
        }
        let q_tok = req_i64(row, "query_token", Some(0), Some((VOCABULARY - 1) as i64))? as u16;
        let t_tok = req_i64(row, "target_token", Some(1), Some((VOCABULARY - 1) as i64))? as u16;

        if let Some(dc) = dev_corpus_rows {
            let c_row = &dc[idx];
            if doc_id != c_row.document_id {
                return Err(format!(
                    "row[{idx}].document_id {doc_id} != DEV corpus document_id {}",
                    c_row.document_id
                ));
            }
            if let Some(rec_v) = row.get("record") {
                if (rec_v.as_i64()? as u64) != c_row.record {
                    return Err(format!("row[{idx}].record != DEV corpus record"));
                }
            }
            if target_pos != c_row.target_position {
                return Err(format!(
                    "row[{idx}].target_position {target_pos} != DEV corpus {}",
                    c_row.target_position
                ));
            }
            if q_tok != c_row.query_token {
                return Err(format!(
                    "row[{idx}].query_token {q_tok} != DEV corpus {}",
                    c_row.query_token
                ));
            }
            if t_tok != c_row.target_token {
                return Err(format!(
                    "row[{idx}].target_token {t_tok} != DEV corpus {}",
                    c_row.target_token
                ));
            }
        }

        // Exact nibble order (6255245665 item 2)
        let code = req_i64(row, "code", Some(0), Some((MAX_CODE_COUNT - 1) as i64))? as u16;
        let rf = req_i64(row, "root_first", Some(0), Some(15))? as u8;
        let rl = req_i64(row, "root_last", Some(0), Some(15))? as u8;
        verify_code_endpoints(code, rf, rl)?;

        // 16 physical cell slots (`admitted_cell_slots`, with `0xFFFF` sentinel; no invented `admitted_tokens` array!)
        let slots_arr = row
            .get("admitted_cell_slots")
            .ok_or_else(|| format!("row[{idx}] missing 16-element `admitted_cell_slots` u16 array"))?
            .as_array()?;
        if slots_arr.len() != 16 {
            return Err(format!(
                "row[{idx}].admitted_cell_slots len {} != 16",
                slots_arr.len()
            ));
        }
        let mut non_sentinel_cells = Vec::with_capacity(16);
        for (slot_i, sv) in slots_arr.iter().enumerate() {
            let c_val = sv.as_i64()?;
            if c_val < 0 || c_val > (SENTINEL_U16 as i64) {
                return Err(format!("row[{idx}].admitted_cell_slots[{slot_i}] out of u16 range"));
            }
            let c_u16 = c_val as u16;
            if c_u16 != SENTINEL_U16 {
                if c_u16 > 255 {
                    return Err(format!(
                        "row[{idx}].admitted_cell_slots[{slot_i}] = {c_u16} outside 0..255 or 0xFFFF"
                    ));
                }
                non_sentinel_cells.push(c_u16);
            }
        }

        let admitted_cells_cnt = req_i64(row, "admitted_cells", Some(1), Some(expected_admitted_k as i64))? as usize;
        if admitted_cells_cnt != non_sentinel_cells.len() {
            return Err(format!(
                "row[{idx}].admitted_cells {admitted_cells_cnt} != non-0xFFFF slot count {}",
                non_sentinel_cells.len()
            ));
        }
        let address_rivals = req_i64(row, "address_rivals", Some(0), Some(expected_admitted_k as i64))? as usize;
        if admitted_cells_cnt < address_rivals {
            return Err(format!(
                "row[{idx}] admitted_cells ({admitted_cells_cnt}) < address_rivals ({address_rivals})"
            ));
        }
        // Exact target-cell admission indicator (6255411191 item 3): admitted_cells - address_rivals in {0, 1}
        let target_admitted = admitted_cells_cnt - address_rivals;
        if target_admitted > 1 {
            return Err(format!(
                "row[{idx}] target_admitted = admitted_cells - address_rivals = {target_admitted} not in {{0, 1}}"
            ));
        }
        if let Some(tc_val) = row.get("target_cell_admitted") {
            let declared_ta = match tc_val {
                JsonValue::Bool(b) => usize::from(*b),
                JsonValue::Int(n) => *n as usize,
                _ => return Err(format!("row[{idx}].target_cell_admitted must be bool or 0/1")),
            };
            if declared_ta != target_admitted {
                return Err(format!(
                    "row[{idx}].target_cell_admitted {declared_ta} != admitted_cells - address_rivals ({target_admitted})"
                ));
            }
        }
        if let Some(tc_map) = token_cells {
            let target_cell = tc_map[t_tok as usize];
            let cell_present = non_sentinel_cells.contains(&target_cell);
            if cell_present != (target_admitted == 1) {
                return Err(format!(
                    "row[{idx}] token_cells[{t_tok}]={target_cell} membership in admitted_cell_slots ({cell_present}) != target_admitted ({target_admitted})"
                ));
            }
        }

        // Writer top-8 and hits (using canonical producer names `top8`, `top1_correct`, `top8_correct`)
        let w_top8_arr = row
            .get("top8")
            .ok_or_else(|| format!("row[{idx}] missing `top8`"))?
            .as_array()?;
        if w_top8_arr.len() != 8 {
            return Err(format!("row[{idx}].top8 len {} != 8", w_top8_arr.len()));
        }
        let mut w_top8 = [0u16; 8];
        let mut seen_w = BTreeSet::new();
        for (k, v) in w_top8_arr.iter().enumerate() {
            let tid = v.as_i64()?;
            if tid < 0 || tid >= (VOCABULARY as i64) || !seen_w.insert(tid) {
                return Err(format!("row[{idx}].top8[{k}] invalid or duplicate token {tid}"));
            }
            w_top8[k] = tid as u16;
        }
        let w_top1 = w_top8[0];
        let w_hit1 = req_i64(row, "top1_correct", Some(0), Some(1))? == 1;
        let w_hit8 = req_i64(row, "top8_correct", Some(0), Some(1))? == 1;
        if w_hit1 != (w_top1 == t_tok) {
            return Err(format!("row[{idx}].top1_correct disagrees with top8[0] == target_token"));
        }
        if w_hit8 != w_top8.contains(&t_tok) {
            return Err(format!("row[{idx}].top8_correct disagrees with top8.contains(target_token)"));
        }
        if !(usize::from(w_hit1) <= usize::from(w_hit8) && usize::from(w_hit8) <= target_admitted) {
            return Err(format!("row[{idx}] rank/admission nesting violated"));
        }

        if let Some(se) = sealed_eval_rows {
            let s_row = &se[idx];
            if w_top8 != s_row.top8 {
                return Err(format!("row[{idx}].top8 != sealed evaluation top8"));
            }
            if usize::from(w_hit1) != (s_row.top1_correct as usize) {
                return Err(format!("row[{idx}].top1_correct != sealed evaluation top1_correct"));
            }
            if usize::from(w_hit8) != (s_row.top8_correct as usize) {
                return Err(format!("row[{idx}].top8_correct != sealed evaluation top8_correct"));
            }
            if admitted_cells_cnt != (s_row.admitted_cells as usize) {
                return Err(format!("row[{idx}].admitted_cells != sealed evaluation admitted_cells"));
            }
            if address_rivals != (s_row.address_rivals as usize) {
                return Err(format!("row[{idx}].address_rivals != sealed evaluation address_rivals"));
            }
            if target_admitted != (s_row.target_admitted as usize) {
                return Err(format!("row[{idx}] target_admitted != sealed evaluation target_admitted"));
            }
        }

        // Table & ordinary comparator predictions
        let t_top1 = req_i64(row, "table_top1", Some(0), Some((VOCABULARY - 1) as i64))? as u16;
        let t_top8_arr = row
            .get("table_top8")
            .ok_or_else(|| format!("row[{idx}] missing `table_top8`"))?
            .as_array()?;
        if t_top8_arr.len() != 8 {
            return Err(format!("row[{idx}].table_top8 len {} != 8", t_top8_arr.len()));
        }
        let mut t_top8 = [0u16; 8];
        let mut seen_t = BTreeSet::new();
        for (k, v) in t_top8_arr.iter().enumerate() {
            let tid = v.as_i64()?;
            if tid < 0 || tid >= (VOCABULARY as i64) || !seen_t.insert(tid) {
                return Err(format!("row[{idx}].table_top8[{k}] invalid or duplicate token {tid}"));
            }
            t_top8[k] = tid as u16;
        }
        if t_top8[0] != t_top1 {
            return Err(format!("row[{idx}].table_top8[0] != table_top1"));
        }

        let u_top1 = req_i64(row, "unigram_1049856_top1", Some(0), Some((VOCABULARY - 1) as i64))? as u16;
        let g_top1 = req_i64(row, "bigram_top1", Some(0), Some((VOCABULARY - 1) as i64))? as u16;

        let t_hit1 = req_bool(row, "table_hit1")?;
        let t_hit8 = req_bool(row, "table_hit8")?;
        let u_hit1 = req_bool(row, "unigram_1049856_hit1")?;
        let g_hit1 = req_bool(row, "bigram_hit1")?;
        let seen_code = req_bool(row, "seen_code")?;

        if t_hit1 != (t_top1 == t_tok) {
            return Err(format!("row[{idx}].table_hit1 != (table_top1 == target_token)"));
        }
        if t_hit8 != t_top8.contains(&t_tok) {
            return Err(format!("row[{idx}].table_hit8 != table_top8.contains(target_token)"));
        }
        if u_hit1 != (u_top1 == t_tok) {
            return Err(format!("row[{idx}].unigram_1049856_hit1 != (unigram_1049856_top1 == target_token)"));
        }
        if g_hit1 != (g_top1 == t_tok) {
            return Err(format!("row[{idx}].bigram_hit1 != (bigram_top1 == target_token)"));
        }

        summary.writer_hit1_sum += i64::from(w_hit1);
        summary.writer_hit8_sum += i64::from(w_hit8);
        summary.target_admitted_sum += target_admitted as i64;
        summary.admitted_cell_count_sum += admitted_cells_cnt as i64;
        summary.address_rival_count_sum += address_rivals as i64;
        summary.table_hit1_sum += i64::from(t_hit1);
        summary.table_hit8_sum += i64::from(t_hit8);
        summary.unigram_hit1_sum += i64::from(u_hit1);
        summary.bigram_hit1_sum += i64::from(g_hit1);
        summary.seen_code_sum += i64::from(seen_code);

        if !t_hit1 && u_hit1 {
            summary.b_u += 1;
        } else if t_hit1 && !u_hit1 {
            summary.c_u += 1;
        }
        if !t_hit1 && g_hit1 {
            summary.b_g += 1;
        } else if t_hit1 && !g_hit1 {
            summary.c_g += 1;
        }
        if !t_hit1 && w_hit1 {
            summary.b_w += 1;
        } else if t_hit1 && !w_hit1 {
            summary.c_w += 1;
        }
    }

    Ok(summary)
}

// --- Compact TRAIN Stream Verification ---

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompactTrainStreamSummary {
    pub capture_mode: String,
    pub stream_bytes: usize,
    pub stream_sha256: String,
    pub rows_verified: usize,
    pub matched_prefix_rows: usize,
    pub distinct_codes_seen: usize,
    pub distinct_codes_seen_total: usize,
    pub codes_with_count_ge_2: i64,
    pub codes_with_count_ge_2_total: i64,
    pub corpus_join_verified: bool,
    pub uort_header_verified: bool,
    pub query_sidecar_sha256: Option<String>,
    pub bigram_rows_verified: usize,
}

pub fn verify_compact_train_stream_bytes(
    data: &[u8],
    capture_mode: &str,
    expected_rows_override: Option<usize>,
    allow_synthetic_length: bool,
    require_query_handoff: bool,
    query_sidecar_bytes: Option<&[u8]>,
    corpus_tokens_bytes: Option<&[u8]>,
    corpus_mask_bytes: Option<&[u8]>,
) -> Result<CompactTrainStreamSummary, String> {
    if data.is_empty() || data.len() % 4 != 0 {
        return Err(format!(
            "compact TRAIN stream byte length {} must be positive and a multiple of 4",
            data.len()
        ));
    }
    let total_rows = data.len() / 4;
    let expected_rows = if !allow_synthetic_length {
        let mode_exp = match capture_mode {
            "q_only_2425925" => FULL_TRAIN_TARGETS,
            "production_forward_matched_1049856_fallback" | "matched_prefix_1049856" => {
                MATCHED_TRAIN_TARGETS
            }
            _ => return Err(format!("unrecognized capture_mode {capture_mode:?}")),
        };
        if let Some(ovr) = expected_rows_override {
            if ovr != mode_exp {
                return Err(format!(
                    "expected_rows {ovr} conflicts with capture_mode {capture_mode:?} ({mode_exp})"
                ));
            }
        }
        mode_exp
    } else {
        expected_rows_override.unwrap_or(total_rows)
    };

    if total_rows != expected_rows {
        return Err(format!(
            "compact TRAIN stream rows {total_rows} != expected {expected_rows}"
        ));
    }

    if let Some(sc) = query_sidecar_bytes {
        if sc.len() != 2 * total_rows {
            return Err(format!(
                "query sidecar byte length {} != 2 * {total_rows}",
                sc.len()
            ));
        }
    }

    let corpus_summary = match (corpus_tokens_bytes, corpus_mask_bytes) {
        (Some(tb), Some(mb)) => Some(parse_prepared_corpus_uort(
            tb,
            mb,
            PINNED_BOS_ID,
            SPLIT_TAG_TRAIN,
            CONTEXT_WINDOW,
            if allow_synthetic_length {
                None
            } else {
                Some(TRAIN_PAYLOAD_TOKENS)
            },
            if allow_synthetic_length {
                None
            } else {
                Some(TRAIN_DOCUMENTS)
            },
            if allow_synthetic_length {
                None
            } else {
                Some(FULL_TRAIN_TARGETS)
            },
            Some(total_rows),
        )?),
        (None, None) => None,
        _ => return Err("both corpus_tokens_bytes and corpus_mask_bytes required".to_string()),
    };

    if require_query_handoff && corpus_summary.is_none() && query_sidecar_bytes.is_none() {
        return Err(
            "compact TRAIN stream requires canonical PreparedCorpus join (Mode A) or query sidecar (Mode B)"
                .to_string(),
        );
    }

    let matched_limit = total_rows.min(MATCHED_TRAIN_TARGETS);
    let mut code_counts = vec![0u32; MAX_CODE_COUNT];
    let mut matched_counts = vec![0u32; MAX_CODE_COUNT];
    let mut bigram_rows = 0usize;

    for idx in 0..total_rows {
        let off = 4 * idx;
        let code = u16::from_le_bytes([data[off], data[off + 1]]);
        let target = u16::from_le_bytes([data[off + 2], data[off + 3]]);
        if target == PINNED_BOS_ID || target == SENTINEL_U16 || target >= VOCABULARY {
            return Err(format!(
                "compact TRAIN stream row {idx}: invalid, BOS, or sentinel target token {target:#06x}"
            ));
        }
        if let Some(ref cs) = corpus_summary {
            let c_row = &cs.rows[idx];
            if target != c_row.target_token {
                return Err(format!(
                    "compact TRAIN stream row {idx} (pos={}): target {target} != corpus target {}",
                    c_row.target_position, c_row.target_token
                ));
            }
            let q_tok = c_row.query_token;
            if q_tok == SENTINEL_U16 || q_tok >= VOCABULARY {
                return Err(format!("compact TRAIN stream row {idx}: invalid corpus query {q_tok}"));
            }
            if let Some(sc) = query_sidecar_bytes {
                let sc_q = u16::from_le_bytes([sc[2 * idx], sc[2 * idx + 1]]);
                if sc_q != q_tok {
                    return Err(format!(
                        "compact TRAIN stream row {idx}: sidecar query {sc_q} != corpus query {q_tok}"
                    ));
                }
            }
            if idx < matched_limit {
                bigram_rows += 1;
            }
        } else if let Some(sc) = query_sidecar_bytes {
            let sc_q = u16::from_le_bytes([sc[2 * idx], sc[2 * idx + 1]]);
            if sc_q == SENTINEL_U16 || sc_q >= VOCABULARY {
                return Err(format!(
                    "compact TRAIN stream row {idx}: invalid sidecar query {sc_q:#06x}"
                ));
            }
            if idx < matched_limit {
                bigram_rows += 1;
            }
        }

        code_counts[code as usize] += 1;
        if idx < matched_limit {
            matched_counts[code as usize] += 1;
        }
    }

    let distinct_matched = matched_counts.iter().filter(|&&c| c >= 1).count();
    let distinct_total = code_counts.iter().filter(|&&c| c >= 1).count();
    let ge2_matched = matched_counts.iter().filter(|&&c| c >= 2).count() as i64;
    let ge2_total = code_counts.iter().filter(|&&c| c >= 2).count() as i64;

    Ok(CompactTrainStreamSummary {
        capture_mode: capture_mode.to_string(),
        stream_bytes: data.len(),
        stream_sha256: sha256_bytes(data),
        rows_verified: total_rows,
        matched_prefix_rows: matched_limit,
        distinct_codes_seen: distinct_matched,
        distinct_codes_seen_total: distinct_total,
        codes_with_count_ge_2: ge2_matched,
        codes_with_count_ge_2_total: ge2_total,
        corpus_join_verified: corpus_summary.is_some(),
        uort_header_verified: corpus_summary.is_some(),
        query_sidecar_sha256: query_sidecar_bytes.map(sha256_bytes),
        bigram_rows_verified: bigram_rows,
    })
}

// --- Runtime Manifest & Instrument Validity (Precedence 0, 6255411191 item 1) ---

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeManifestBinding {
    pub manifest_sha256: String,
    pub original_source_sha256: String,
    pub original_kernel_sha256: String,
    pub original_driver_sha256: String,
    pub original_model_domain_a_sha256: String,
    pub target_compiled_source_sha256: String,
    pub target_kernel_sha256: String,
    pub target_binary_sha256: String,
}

pub fn verify_runtime_manifest_bytes(bytes: &[u8]) -> Result<RuntimeManifestBinding, String> {
    let doc = parse_json(bytes)?;
    let orig_src = req_str(&doc, "original_source_sha256")?;
    let orig_ker = req_str(&doc, "original_kernel_sha256")?;
    let orig_drv = req_str(&doc, "original_operational_driver_sha256")?;
    let orig_dom_a = req_str(&doc, "original_model_domain_a_sha256")?;
    if orig_src != PINNED_ORIGINAL_SOURCE_SHA256 {
        return Err(format!(
            "runtime manifest original_source_sha256 {orig_src:?} != authenticated Q4 {PINNED_ORIGINAL_SOURCE_SHA256:?}"
        ));
    }
    if orig_ker != PINNED_ORIGINAL_KERNEL_SHA256 {
        return Err(format!(
            "runtime manifest original_kernel_sha256 {orig_ker:?} != authenticated Q4 {PINNED_ORIGINAL_KERNEL_SHA256:?}"
        ));
    }
    if orig_drv != PINNED_ORIGINAL_DRIVER_SHA256 {
        return Err(format!(
            "runtime manifest original_operational_driver_sha256 {orig_drv:?} != authenticated Q4 {PINNED_ORIGINAL_DRIVER_SHA256:?}"
        ));
    }
    if orig_dom_a != PINNED_ORIGINAL_MODEL_DOMAIN_A_SHA256 {
        return Err(format!(
            "runtime manifest original_model_domain_a_sha256 {orig_dom_a:?} != authenticated Q4 {PINNED_ORIGINAL_MODEL_DOMAIN_A_SHA256:?}"
        ));
    }

    let tgt_src = req_str(&doc, "target_compiled_source_sha256")?;
    let tgt_ker = req_str(&doc, "target_kernel_sha256")?;
    let tgt_bin = req_str(&doc, "target_binary_sha256")?;
    for (k, v) in [
        ("target_compiled_source_sha256", tgt_src),
        ("target_kernel_sha256", tgt_ker),
        ("target_binary_sha256", tgt_bin),
    ] {
        if !is_hex64(v) {
            return Err(format!("runtime manifest {k} must be bound 64-hex, got {v:?}"));
        }
    }
    if tgt_src == FORBIDDEN_K16_QUAL_SOURCE_SHA256 {
        return Err(
            "runtime manifest target_compiled_source_sha256 cannot substitute K16 qualification graph 035b3c..."
                .to_string(),
        );
    }

    Ok(RuntimeManifestBinding {
        manifest_sha256: sha256_bytes(bytes),
        original_source_sha256: orig_src.to_string(),
        original_kernel_sha256: orig_ker.to_string(),
        original_driver_sha256: orig_drv.to_string(),
        original_model_domain_a_sha256: orig_dom_a.to_string(),
        target_compiled_source_sha256: tgt_src.to_string(),
        target_kernel_sha256: tgt_ker.to_string(),
        target_binary_sha256: tgt_bin.to_string(),
    })
}

pub fn verify_instrument_validity(
    record: &JsonValue,
    strict_k8_q4_pins: bool,
    require_bound_runtime_pins: bool,
    runtime_manifest: Option<&RuntimeManifestBinding>,
) -> (bool, Vec<String>, String) {
    let mut faults = Vec::new();

    if record.get("schema").and_then(|v| v.as_str().ok()) != Some(READ_CODE_CAPACITY_SCHEMA) {
        faults.push(format!("schema != {READ_CODE_CAPACITY_SCHEMA}"));
    }
    if record.get("zero_apply").and_then(|v| v.as_bool().ok()) != Some(true) {
        faults.push("zero_apply must be true".to_string());
    }
    if record.get("applied_updates").and_then(|v| v.as_i64().ok()) != Some(0) {
        faults.push("applied_updates must be 0".to_string());
    }
    if record.get("updates_applied").and_then(|v| v.as_i64().ok()).unwrap_or(0) != 0 {
        faults.push("updates_applied must be 0".to_string());
    }
    if record.get("snapshot_unchanged").and_then(|v| v.as_bool().ok()) != Some(true) {
        faults.push("snapshot_unchanged must be true".to_string());
    }
    if record.get("model_compute_executed").and_then(|v| v.as_bool().ok()) != Some(true) {
        faults.push("model_compute_executed must be true".to_string());
    }

    if strict_k8_q4_pins {
        if record.get("epoch").and_then(|v| v.as_i64().ok()) != Some(Q4_EPOCH) {
            faults.push(format!("epoch != {Q4_EPOCH}"));
        }
        if record.get("checkpoint_sha256").and_then(|v| v.as_str().ok())
            != Some(PINNED_Q4_CHECKPOINT_SHA256)
        {
            faults.push("checkpoint_sha256 mismatch".to_string());
        }
        if record.get("panel_binding_sha256").and_then(|v| v.as_str().ok())
            != Some(PINNED_ORDERED_ROW_BINDING_SHA256)
        {
            faults.push("panel_binding_sha256 mismatch".to_string());
        }
    }

    let rec_src = record.get("source_sha256").and_then(|v| v.as_str().ok()).unwrap_or("");
    if !is_hex64(rec_src) {
        faults.push("missing or invalid 64-hex source_sha256".to_string());
    }
    if rec_src == FORBIDDEN_K16_QUAL_SOURCE_SHA256 {
        faults.push(
            "source_sha256 cannot substitute K16 qualification graph 035b3c... for W2".to_string(),
        );
    }

    // Original -> target source bridge & authenticated runtime manifest (6255411191 item 1)
    match record.get("original_to_target_bridge") {
        None => {
            if require_bound_runtime_pins {
                faults.push("missing original_to_target_bridge object".to_string());
            }
        }
        Some(bridge) => {
            if bridge.get("original_source_sha256").and_then(|v| v.as_str().ok())
                != Some(PINNED_ORIGINAL_SOURCE_SHA256)
            {
                faults.push(format!(
                    "original_to_target_bridge.original_source_sha256 != authenticated Q4 {PINNED_ORIGINAL_SOURCE_SHA256}"
                ));
            }
            if bridge.get("original_kernel_sha256").and_then(|v| v.as_str().ok())
                != Some(PINNED_ORIGINAL_KERNEL_SHA256)
            {
                faults.push(format!(
                    "original_to_target_bridge.original_kernel_sha256 != authenticated Q4 {PINNED_ORIGINAL_KERNEL_SHA256}"
                ));
            }
            let tgt_src = bridge
                .get("target_compiled_source_sha256")
                .and_then(|v| v.as_str().ok())
                .unwrap_or("");
            let tgt_ker = bridge
                .get("target_kernel_sha256")
                .and_then(|v| v.as_str().ok())
                .unwrap_or("");
            let tgt_bin = bridge
                .get("target_binary_sha256")
                .and_then(|v| v.as_str().ok())
                .unwrap_or("");
            let rt_man = bridge
                .get("runtime_manifest_sha256")
                .and_then(|v| v.as_str().ok())
                .unwrap_or("");
            if require_bound_runtime_pins {
                if !is_hex64(tgt_src) || tgt_src == FORBIDDEN_K16_QUAL_SOURCE_SHA256 {
                    faults.push("target_compiled_source_sha256 must be bound non-K16 64-hex in runtime mode".to_string());
                } else if rec_src != tgt_src {
                    faults.push("record.source_sha256 != bridge.target_compiled_source_sha256".to_string());
                }
                if !is_hex64(tgt_ker) {
                    faults.push("target_kernel_sha256 must be bound 64-hex in runtime mode".to_string());
                }
                if !is_hex64(tgt_bin) {
                    faults.push("target_binary_sha256 must be bound 64-hex in runtime mode".to_string());
                }
                if !is_hex64(rt_man) {
                    faults.push("runtime_manifest_sha256 must be bound 64-hex in runtime mode".to_string());
                }
                match runtime_manifest {
                    None => faults.push(
                        "verified runtime mode requires an authenticated --runtime-manifest file on disk (not a formatted string)"
                            .to_string(),
                    ),
                    Some(rm) => {
                        if rm.manifest_sha256 != rt_man {
                            faults.push("runtime_manifest file SHA-256 != bridge.runtime_manifest_sha256".to_string());
                        }
                        if rm.target_compiled_source_sha256 != tgt_src
                            || rm.target_kernel_sha256 != tgt_ker
                            || rm.target_binary_sha256 != tgt_bin
                        {
                            faults.push("runtime_manifest target pins disagree with record bridge".to_string());
                        }
                    }
                }
            } else {
                for (k, v) in [
                    ("target_compiled_source_sha256", tgt_src),
                    ("target_kernel_sha256", tgt_ker),
                    ("target_binary_sha256", tgt_bin),
                    ("runtime_manifest_sha256", rt_man),
                ] {
                    if v != "UNBOUND" && !is_hex64(v) {
                        faults.push(format!("bridge.{k} must be 'UNBOUND' or 64-hex"));
                    }
                }
            }
        }
    }

    let mut capture_mode = "unknown".to_string();
    match record.get("instrument_validity") {
        None => {
            faults.push("missing instrument_validity object".to_string());
            return (true, faults, capture_mode);
        }
        Some(val) => {
            if let Some(rrt) = val.get("root_round_trip") {
                if rrt.get("sample_rows").and_then(|v| v.as_i64().ok()).unwrap_or(0) < 1024
                    || rrt.get("mismatches").and_then(|v| v.as_i64().ok()) != Some(0)
                    || rrt.get("absent_roots").and_then(|v| v.as_i64().ok()) != Some(0)
                {
                    faults.push("root_round_trip check failed".to_string());
                }
            } else {
                faults.push("missing root_round_trip".to_string());
            }

            if let Some(tca) = val.get("train_capture") {
                capture_mode = tca
                    .get("mode")
                    .and_then(|v| v.as_str().ok())
                    .unwrap_or("")
                    .to_string();
                if capture_mode == "q_only_2425925" {
                    if tca.get("sample_rows").and_then(|v| v.as_i64().ok()).unwrap_or(0) < 1024
                        || tca
                            .get("production_forward_disagreements")
                            .and_then(|v| v.as_i64().ok())
                            != Some(0)
                    {
                        faults.push("q_only_2425925 train_capture check failed".to_string());
                    }
                } else if capture_mode == "production_forward_matched_1049856_fallback" {
                    if tca
                        .get("smaller_table_bias_disclosed")
                        .and_then(|v| v.as_bool().ok())
                        != Some(true)
                    {
                        faults.push("fallback requires smaller_table_bias_disclosed=true".to_string());
                    }
                } else {
                    faults.push(format!("unrecognized train_capture.mode {capture_mode:?}"));
                }
            } else {
                faults.push("missing train_capture".to_string());
            }

            if let Some(sra) = val.get("sealed_receipt_agreement") {
                if sra.get("dev_rows").and_then(|v| v.as_i64().ok()) != Some(DEV_ROWS as i64)
                    || sra.get("row_disagreements").and_then(|v| v.as_i64().ok()) != Some(0)
                {
                    faults.push("sealed_receipt_agreement dev_rows/row_disagreements failed".to_string());
                }
                if strict_k8_q4_pins {
                    if sra.get("evaluation_sha256").and_then(|v| v.as_str().ok())
                        != Some(PINNED_Q4_EVALUATION_SHA256)
                        || sra.get("writer_top1").and_then(|v| v.as_i64().ok()) != Some(Q4_WRITER_TOP1)
                        || sra.get("writer_top8").and_then(|v| v.as_i64().ok()) != Some(Q4_WRITER_TOP8)
                        || sra.get("writer_admitted").and_then(|v| v.as_i64().ok())
                            != Some(Q4_WRITER_ADMITTED)
                    {
                        faults.push("sealed_receipt_agreement Q4 pins mismatch".to_string());
                    }
                }
            } else {
                faults.push("missing sealed_receipt_agreement".to_string());
            }

            if let Some(ctc) = val.get("count_table_conservation") {
                if ctc.get("matched_train_total").and_then(|v| v.as_i64().ok())
                    != Some(MATCHED_TRAIN_TARGETS as i64)
                    || ctc.get("bigram_train_total").and_then(|v| v.as_i64().ok())
                        != Some(MATCHED_TRAIN_TARGETS as i64)
                    || ctc.get("invalid_tokens").and_then(|v| v.as_i64().ok()) != Some(0)
                    || ctc.get("negative_counts").and_then(|v| v.as_i64().ok()) != Some(0)
                {
                    faults.push("count_table_conservation failed".to_string());
                }
                if capture_mode == "q_only_2425925"
                    && ctc.get("full_train_total").and_then(|v| v.as_i64().ok())
                        != Some(FULL_TRAIN_TARGETS as i64)
                {
                    faults.push("full_train_total != 2425925".to_string());
                }
            } else {
                faults.push("missing count_table_conservation".to_string());
            }

            if let Some(det) = val.get("determinism") {
                if det.get("repeat_rows").and_then(|v| v.as_i64().ok()).unwrap_or(0) < 1000
                    || det.get("byte_identical").and_then(|v| v.as_bool().ok()) != Some(true)
                {
                    faults.push("determinism check failed".to_string());
                }
            } else {
                faults.push("missing determinism".to_string());
            }

            if val.get("symmetry_action_non_roots").and_then(|v| v.as_i64().ok()) != Some(0) {
                faults.push("symmetry_action_non_roots != 0".to_string());
            }
            if val.get("cuda_or_status_fault").and_then(|v| v.as_bool().ok()) != Some(false) {
                faults.push("cuda_or_status_fault != false".to_string());
            }
            if val.get("d23_d25_d32_ratchets_pass").and_then(|v| v.as_bool().ok()) != Some(true) {
                faults.push("d23_d25_d32_ratchets_pass != true".to_string());
            }
        }
    }

    if require_bound_runtime_pins {
        match record.get("producer_file_pins") {
            None => faults.push("missing producer_file_pins".to_string()),
            Some(pins) => {
                if pins.get("dev_tokens_sha256").and_then(|v| v.as_str().ok())
                    != Some(PINNED_DEV_TOKENS_SHA256)
                    || pins.get("dev_mask_sha256").and_then(|v| v.as_str().ok())
                        != Some(PINNED_DEV_MASK_SHA256)
                    || pins.get("train_tokens_sha256").and_then(|v| v.as_str().ok())
                        != Some(PINNED_TRAIN_TOKENS_SHA256)
                    || pins.get("train_mask_sha256").and_then(|v| v.as_str().ok())
                        != Some(PINNED_TRAIN_MASK_SHA256)
                    || pins.get("tokenizer_sha256").and_then(|v| v.as_str().ok())
                        != Some(PINNED_TOKENIZER_SHA256)
                    || pins.get("bos_token").and_then(|v| v.as_str().ok()) != Some(PINNED_BOS_TOKEN)
                    || pins.get("bos_id").and_then(|v| v.as_i64().ok()) != Some(PINNED_BOS_ID as i64)
                {
                    faults.push("producer_file_pins mismatch".to_string());
                }
                let exp_stream_bytes = if capture_mode == "q_only_2425925" {
                    COMPACT_TRAIN_FULL_BYTES as i64
                } else {
                    COMPACT_TRAIN_PRIMARY_BYTES as i64
                };
                if pins.get("compact_train_stream_bytes").and_then(|v| v.as_i64().ok())
                    != Some(exp_stream_bytes)
                    || !pins
                        .get("compact_train_stream_sha256")
                        .and_then(|v| v.as_str().ok())
                        .map_or(false, is_hex64)
                {
                    faults.push("producer_file_pins compact_train_stream mismatch".to_string());
                }
            }
        }
    }

    (!faults.is_empty(), faults, capture_mode)
}

// --- Exact Threshold Predicates & Precedence 0..5 Decision Table ---

pub fn exceeds_paired_floor_strict(diff: i64, b_plus_c: i64, min_floor: i64) -> bool {
    diff > min_floor && (diff * diff) > 4 * b_plus_c
}

pub fn ge_paired_floor(diff: i64, b_plus_c: i64, min_floor: i64) -> bool {
    diff >= min_floor && (diff * diff) >= 4 * b_plus_c
}

pub fn ge_minus_paired_floor(diff: i64, b_plus_c: i64, min_floor: i64) -> bool {
    let deficit = -diff;
    if deficit <= 0 {
        return true;
    }
    deficit <= min_floor || (deficit * deficit) <= 4 * b_plus_c
}

/// Exact two-sided binomial (McNemar) p <= 0.05 check for n = b + c < 100 without floating-point inaccuracy:
/// 2 * sum_{i=0..min(b,c)} C(n, i) <= 0.05 * 2^n  <=>  40 * sum_{i=0..min(b,c)} C(n, i) <= 2^n.
pub fn exact_mcnemar_p_le_005(b: i64, c: i64) -> bool {
    let n = (b + c) as u32;
    if n == 0 {
        return false;
    }
    let k = b.min(c) as u32;
    let mut tail_sum: u128 = 0;
    let mut binom: u128 = 1;
    for i in 0..=k {
        if i > 0 {
            binom = binom * ((n - i + 1) as u128) / (i as u128);
        }
        tail_sum += binom;
    }
    40u128.saturating_mul(tail_sum) <= (1u128 << n)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct W2DecisionOutcome {
    pub verification_tier: &'static str,
    pub runtime_verdict_authorized: bool,
    pub precedence: u8,
    pub verdict: String,
    pub candidate_verdict: String,
    pub fault_or_underpowered_reasons: Vec<String>,
    pub t_top1: i64,
    pub u_top1: i64,
    pub g_top1: i64,
    pub m_top1: i64,
    pub w_top1: i64,
    pub coverage_rows: i64,
    pub codes_with_count_ge_2: i64,
    pub capture_mode: String,
}

pub fn evaluate_w2_decision_table(
    record: &JsonValue,
    strict_k8_q4_pins: bool,
    require_lossless_evidence: bool,
    dev_corpus_rows: Option<&[CanonicalCorpusTargetRow]>,
    sealed_eval_rows: Option<&[SealedEvalRow]>,
    compact_train_summary: Option<&CompactTrainStreamSummary>,
    runtime_manifest: Option<&RuntimeManifestBinding>,
    token_cells: Option<&[u16]>,
) -> Result<W2DecisionOutcome, String> {
    let tier = if require_lossless_evidence {
        "VERIFIED_RUNTIME_EVIDENCE"
    } else {
        "SUMMARY_PREVIEW_ONLY"
    };

    let (inst_fault, faults, capture_mode) = verify_instrument_validity(
        record,
        strict_k8_q4_pins,
        require_lossless_evidence,
        runtime_manifest,
    );
    if inst_fault {
        return Ok(W2DecisionOutcome {
            verification_tier: tier,
            runtime_verdict_authorized: false,
            precedence: 0,
            verdict: "INSTRUMENT_FAULT".to_string(),
            candidate_verdict: "INSTRUMENT_FAULT".to_string(),
            fault_or_underpowered_reasons: faults,
            t_top1: 0,
            u_top1: 0,
            g_top1: 0,
            m_top1: 0,
            w_top1: 0,
            coverage_rows: 0,
            codes_with_count_ge_2: 0,
            capture_mode,
        });
    }

    let eval_blk = record.get("evaluability").ok_or("missing evaluability")?;
    let agg = record.get("aggregates").ok_or("missing aggregates")?;
    let dev_rows = req_i64(eval_blk, "dev_rows", Some(1), None)?;
    if strict_k8_q4_pins && dev_rows != (DEV_ROWS as i64) {
        return Err(format!("evaluability.dev_rows {dev_rows} != {DEV_ROWS}"));
    }
    let coverage_rows = req_i64(eval_blk, "coverage_rows", Some(0), Some(dev_rows))?;
    let codes_ge_2 = req_i64(
        eval_blk,
        "codes_with_count_ge_2",
        Some(0),
        Some(MAX_CODE_COUNT as i64),
    )?;

    let t_top1 = req_i64(agg, "T_table_matched_top1", Some(0), Some(dev_rows))?;
    let u_top1 = req_i64(agg, "U_unigram_matched_top1", Some(0), Some(dev_rows))?;
    let g_top1 = req_i64(agg, "G_bigram_matched_top1", Some(0), Some(dev_rows))?;
    let w_top1 = req_i64(agg, "W_writer_top1", Some(0), Some(dev_rows))?;
    if strict_k8_q4_pins && w_top1 != Q4_WRITER_TOP1 {
        return Ok(W2DecisionOutcome {
            verification_tier: tier,
            runtime_verdict_authorized: false,
            precedence: 0,
            verdict: "INSTRUMENT_FAULT".to_string(),
            candidate_verdict: "INSTRUMENT_FAULT".to_string(),
            fault_or_underpowered_reasons: vec![format!("W_writer_top1 {w_top1} != {Q4_WRITER_TOP1}")],
            t_top1,
            u_top1,
            g_top1,
            m_top1: u_top1.max(g_top1),
            w_top1,
            coverage_rows,
            codes_with_count_ge_2: codes_ge_2,
            capture_mode,
        });
    }

    let disc = agg.get("paired_discordance").ok_or("missing paired_discordance")?;
    let extract_bc = |name: &str, comp: i64| -> Result<(i64, i64), String> {
        let entry = disc.get(name).ok_or_else(|| format!("missing paired_discordance.{name}"))?;
        let b = req_i64(entry, "b", Some(0), Some(dev_rows))?;
        let c = req_i64(entry, "c", Some(0), Some(dev_rows))?;
        if b + c > dev_rows || (c - b) != (t_top1 - comp) {
            return Err(format!("paired_discordance.{name} conservation failure"));
        }
        Ok((b, c))
    };

    let (b_u, c_u, b_g, c_g, b_w, c_w) = match (
        extract_bc("U", u_top1),
        extract_bc("G", g_top1),
        extract_bc("W", w_top1),
    ) {
        (Ok((bu, cu)), Ok((bg, cg)), Ok((bw, cw))) => (bu, cu, bg, cg, bw, cw),
        (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
            return Ok(W2DecisionOutcome {
                verification_tier: tier,
                runtime_verdict_authorized: false,
                precedence: 0,
                verdict: "INSTRUMENT_FAULT".to_string(),
                candidate_verdict: "INSTRUMENT_FAULT".to_string(),
                fault_or_underpowered_reasons: vec![e],
                t_top1,
                u_top1,
                g_top1,
                m_top1: u_top1.max(g_top1),
                w_top1,
                coverage_rows,
                codes_with_count_ge_2: codes_ge_2,
                capture_mode,
            });
        }
    };

    if require_lossless_evidence {
        let rows_arr = record
            .get("rows")
            .ok_or("verified runtime mode requires lossless 14,287-row `rows` array")?
            .as_array()?;
        if dev_corpus_rows.is_none() || (strict_k8_q4_pins && sealed_eval_rows.is_none()) || compact_train_summary.is_none() {
            return Err(
                "verified runtime mode requires canonical DEV corpus, sealed Q4 evaluation, and compact TRAIN stream joins"
                    .to_string(),
            );
        }
        let row_sum = verify_per_row_records(
            rows_arr,
            dev_rows as usize,
            8,
            dev_corpus_rows,
            sealed_eval_rows,
            token_cells,
        )?;
        if row_sum.table_hit1_sum != t_top1
            || row_sum.unigram_hit1_sum != u_top1
            || row_sum.bigram_hit1_sum != g_top1
            || row_sum.writer_hit1_sum != w_top1
            || (strict_k8_q4_pins && row_sum.writer_hit8_sum != Q4_WRITER_TOP8)
            || (strict_k8_q4_pins && row_sum.target_admitted_sum != Q4_WRITER_ADMITTED)
            || row_sum.seen_code_sum != coverage_rows
            || (row_sum.b_u, row_sum.c_u) != (b_u, c_u)
            || (row_sum.b_g, row_sum.c_g) != (b_g, c_g)
            || (row_sum.b_w, row_sum.c_w) != (b_w, c_w)
        {
            return Err("row-derived totals or paired discordance do not match declared aggregates".to_string());
        }
        if let Some(cts) = compact_train_summary {
            if cts.codes_with_count_ge_2 != codes_ge_2 {
                return Err("compact TRAIN stream codes_with_count_ge_2 != evaluability.codes_with_count_ge_2".to_string());
            }
            if let Some(pins) = record.get("producer_file_pins") {
                if pins.get("compact_train_stream_sha256").and_then(|v| v.as_str().ok())
                    != Some(cts.stream_sha256.as_str())
                {
                    return Err("compact TRAIN stream SHA-256 != producer_file_pins.compact_train_stream_sha256".to_string());
                }
            }
        }
    }

    // Precedence 1: UNDERPOWERED (§4.6)
    let mut under_reasons = Vec::new();
    if coverage_rows * 100 < 90 * dev_rows {
        under_reasons.push(format!("coverage {coverage_rows}/{dev_rows} < 90.00%"));
    }
    if codes_ge_2 < MIN_MULTIPLICITY_CODES {
        under_reasons.push(format!("codes_with_count_ge_2 ({codes_ge_2}) < {MIN_MULTIPLICITY_CODES}"));
    }
    for (name, b, c) in [("U", b_u, c_u), ("G", b_g, c_g)] {
        if b + c < MIN_DISCORDANCE_NORMAL && !exact_mcnemar_p_le_005(b, c) {
            under_reasons.push(format!("co-primary {name} has b+c={} < 100 and McNemar p > 0.05", b + c));
        }
    }
    let m_top1 = u_top1.max(g_top1);
    if !under_reasons.is_empty() {
        return Ok(W2DecisionOutcome {
            verification_tier: tier,
            runtime_verdict_authorized: require_lossless_evidence,
            precedence: 1,
            verdict: if require_lossless_evidence {
                "UNDERPOWERED".to_string()
            } else {
                "PREVIEW_UNDERPOWERED".to_string()
            },
            candidate_verdict: "UNDERPOWERED".to_string(),
            fault_or_underpowered_reasons: under_reasons,
            t_top1,
            u_top1,
            g_top1,
            m_top1,
            w_top1,
            coverage_rows,
            codes_with_count_ge_2: codes_ge_2,
            capture_mode,
        });
    }

    let max_bc_ug = (b_u + c_u).max(b_g + c_g);
    let bc_w = b_w + c_w;
    let f_val = (FULL_PANEL_MIN_FLOOR as f64).max(2.0 * (max_bc_ug as f64).sqrt());
    let h_val = (FULL_PANEL_MIN_FLOOR as f64).max(2.0 * (bc_w as f64).sqrt());
    let pathological = (m_top1 as f64 + f_val) < (w_top1 as f64 - h_val);
    let t_ge_w_minus_h = ge_minus_paired_floor(t_top1 - w_top1, bc_w, FULL_PANEL_MIN_FLOOR);
    let t_gt_m_plus_f = exceeds_paired_floor_strict(t_top1 - m_top1, max_bc_ug, FULL_PANEL_MIN_FLOOR);
    let t_ge_w_plus_h = ge_paired_floor(t_top1 - w_top1, bc_w, FULL_PANEL_MIN_FLOOR);

    let (precedence, cand) = if pathological && !t_ge_w_minus_h {
        (4, "H_rep")
    } else if t_gt_m_plus_f {
        (2, "H_law")
    } else if t_ge_w_plus_h {
        (3, "H_mixed")
    } else if t_ge_w_minus_h {
        (4, "H_rep")
    } else {
        (5, "WRITER_DOMINATES")
    };

    if let Some(decl) = record.get("verdict").and_then(|v| v.as_str().ok()) {
        if decl != cand {
            return Err(format!("declared verdict {decl:?} != recomputed {cand:?}"));
        }
    }

    Ok(W2DecisionOutcome {
        verification_tier: tier,
        runtime_verdict_authorized: require_lossless_evidence,
        precedence,
        verdict: if require_lossless_evidence {
            cand.to_string()
        } else {
            format!("PREVIEW_{cand}")
        },
        candidate_verdict: cand.to_string(),
        fault_or_underpowered_reasons: Vec::new(),
        t_top1,
        u_top1,
        g_top1,
        m_top1,
        w_top1,
        coverage_rows,
        codes_with_count_ge_2: codes_ge_2,
        capture_mode,
    })
}

// --- Fail-Closed Host Input Pin Verifier ---

pub fn verify_host_pinned_inputs() -> Result<BTreeMap<String, String>, String> {
    let required: [(&str, &str, &str, Option<usize>); 11] = [
        (
            "sheet_sha256",
            "/Users/casey.allard/uor-r4/docs/harness/prep/m1-read-code-capacity-2026-10-17.md",
            PINNED_SHEET_SHA256,
            None,
        ),
        (
            "readiness_map_sha256",
            "/Users/casey.allard/uor-r4-local/w2-bvg-source-readiness-20261018/REPORT.md",
            PINNED_READINESS_MAP_SHA256,
            None,
        ),
        (
            "interface_prep_report_sha256",
            "/Users/casey.allard/uor-r4-local/w2-input-interface-preparation-20261018/REPORT.md",
            PINNED_INTERFACE_PREP_REPORT_SHA256,
            None,
        ),
        (
            "interface_prep_manifest_sha256",
            "/Users/casey.allard/uor-r4-local/w2-input-interface-preparation-20261018/SHA256SUMS",
            PINNED_INTERFACE_PREP_MANIFEST_SHA256,
            None,
        ),
        (
            "interface_review_sha256",
            "/Users/casey.allard/uor-r4-local/w2-input-interface-preparation-review-20261018/REPORT.md",
            PINNED_INTERFACE_REVIEW_SHA256,
            None,
        ),
        (
            "retained_input_receipt_sha256",
            "/Users/casey.allard/uor-r4-local/w2-retained-input-authentication-20261018/receipt.json",
            PINNED_RETAINED_INPUT_RECEIPT_SHA256,
            None,
        ),
        (
            "dev_tokens_sha256",
            "/Users/casey.allard/uor-r4-local/codex-cf1-k16-rows-fastpath/qualification-prep/operator/data/ft/dev/tokens.u16",
            PINNED_DEV_TOKENS_SHA256,
            Some(DEV_TOKENS_BYTES),
        ),
        (
            "dev_mask_sha256",
            "/Users/casey.allard/uor-r4-local/codex-cf1-k16-rows-fastpath/qualification-prep/operator/data/ft/dev/response_mask.u8",
            PINNED_DEV_MASK_SHA256,
            Some(DEV_MASK_BYTES),
        ),
        (
            "train_tokens_sha256",
            "/Users/casey.allard/uor-r4-local/codex-cf1-k16-rows-fastpath/qualification-prep/operator/data/mw-balp/train/tokens.u16",
            PINNED_TRAIN_TOKENS_SHA256,
            Some(TRAIN_TOKENS_BYTES),
        ),
        (
            "train_mask_sha256",
            "/Users/casey.allard/uor-r4-local/codex-cf1-k16-rows-fastpath/qualification-prep/operator/data/mw-balp/train/response_mask.u8",
            PINNED_TRAIN_MASK_SHA256,
            Some(TRAIN_MASK_BYTES),
        ),
        (
            "d34_record_sha256",
            "/Users/casey.allard/uor-r4-local/codex-attention-preflight-inputs-v2/d34-successor-table-record.json",
            PINNED_D34_RECORD_SHA256,
            None,
        ),
    ];

    let mut out = BTreeMap::new();
    for (key, path_str, exp_sha, exp_len) in required {
        let p = Path::new(path_str);
        if !p.is_file() {
            return Err(format!("missing required host-pinned input for {key}: {path_str}"));
        }
        let bytes = fs::read(p).map_err(|e| format!("read {path_str}: {e}"))?;
        if let Some(len) = exp_len {
            if bytes.len() != len {
                return Err(format!("{key} byte length {} != expected {len}", bytes.len()));
            }
        }
        let sha = sha256_bytes(&bytes);
        if sha != exp_sha {
            return Err(format!("{key} SHA-256 {sha} != expected {exp_sha}"));
        }
        if key == "dev_tokens_sha256" {
            let _ = parse_uort_header(&bytes[..UORT_HEADER_BYTES], Some(DEV_PAYLOAD_TOKENS), VOCABULARY as u32)?;
        } else if key == "train_tokens_sha256" {
            let _ = parse_uort_header(&bytes[..UORT_HEADER_BYTES], Some(TRAIN_PAYLOAD_TOKENS), VOCABULARY as u32)?;
        }
        out.insert(key.to_string(), sha);
    }
    out.insert("original_source_sha256".to_string(), PINNED_ORIGINAL_SOURCE_SHA256.to_string());
    out.insert("original_kernel_sha256".to_string(), PINNED_ORIGINAL_KERNEL_SHA256.to_string());
    out.insert("target_compiled_source_sha256".to_string(), "UNBOUND".to_string());
    out.insert("target_kernel_sha256".to_string(), "UNBOUND".to_string());
    out.insert("target_binary_sha256".to_string(), "UNBOUND".to_string());
    out.insert("runtime_manifest_sha256".to_string(), "UNBOUND".to_string());
    Ok(out)
}

// --- Self-Test & CLI Main ---

pub fn build_uort_bytes(tokens: &[u16], vocabulary: u32) -> Vec<u8> {
    let mut out = vec![0u8; UORT_HEADER_BYTES + 2 * tokens.len()];
    out[0..4].copy_from_slice(UORT_MAGIC);
    out[4..8].copy_from_slice(&UORT_VERSION.to_le_bytes());
    out[8..16].copy_from_slice(&(tokens.len() as u64).to_le_bytes());
    out[16..20].copy_from_slice(&vocabulary.to_le_bytes());
    for (i, &t) in tokens.iter().enumerate() {
        let off = UORT_HEADER_BYTES + 2 * i;
        out[off..off + 2].copy_from_slice(&t.to_le_bytes());
    }
    out
}

pub fn run_self_test() -> Result<(), String> {
    // 1. Canonical UORT + binary mask + BOS eligibility self-test
    let uort = build_uort_bytes(&[0, 10, 42, 10, 42, 0, 99, 4095], 4096);
    let mask = [0u8, 0, 1, 0, 1, 1, 0, 1];
    let parsed = parse_prepared_corpus_uort(
        &uort,
        &mask,
        PINNED_BOS_ID,
        SPLIT_TAG_TRAIN,
        CONTEXT_WINDOW,
        Some(8),
        Some(2),
        Some(3),
        None,
    )?;
    if parsed.document_count != 2 || parsed.eligible_target_count != 3 {
        return Err("canonical UORT document/eligible count mismatch".to_string());
    }
    let raw_no_header = [0u8; 16];
    if parse_prepared_corpus_uort(
        &raw_no_header,
        &mask,
        PINNED_BOS_ID,
        SPLIT_TAG_TRAIN,
        CONTEXT_WINDOW,
        None,
        None,
        None,
        None,
    )
    .is_ok()
    {
        return Err("raw u16 without UORT header must fail".to_string());
    }

    // 2. Nibble order self-test
    verify_code_endpoints(0x4321, 1, 4)?;
    if verify_code_endpoints(0x4321, 4, 1).is_ok() {
        return Err("reversed nibble endpoints must fail".to_string());
    }

    // 3. Target admission vs nonempty admission set self-test (admitted_cells - address_rivals)
    let row_json = br#"{
        "row_ordinal": 0,
        "document_id": 0,
        "record": 4611686018427387904,
        "target_position": 2,
        "query_occurrence": 1,
        "query_token": 10,
        "target_token": 42,
        "code": 17185,
        "root_first": 1,
        "root_last": 4,
        "admitted_cells": 8,
        "address_rivals": 8,
        "admitted_cell_slots": [10, 11, 12, 13, 14, 15, 16, 17, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535],
        "top8": [1, 2, 3, 4, 5, 6, 7, 8],
        "top1_correct": 0,
        "top8_correct": 0,
        "table_top1": 42,
        "table_top8": [42, 1, 2, 3, 4, 5, 6, 7],
        "unigram_1049856_top1": 10,
        "bigram_top1": 42,
        "table_hit1": true,
        "table_hit8": true,
        "unigram_1049856_hit1": false,
        "bigram_hit1": true,
        "seen_code": true
    }"#;
    let row_val = parse_json(row_json)?;
    // Notice admitted_cells = 8 (> 0), address_rivals = 8 => target_admitted = 8 - 8 = 0!
    let row_sum = verify_per_row_records(&[row_val], 1, 8, None, None, None)?;
    if row_sum.target_admitted_sum != 0 || row_sum.admitted_cell_count_sum != 8 {
        return Err(format!(
            "expected target_admitted_sum == 0 when admitted_cells == address_rivals == 8, got {}",
            row_sum.target_admitted_sum
        ));
    }

    Ok(())
}

pub fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "Usage: verify_w2_read_code_capacity [--self-test] [--verify-host-pins] [--record <path>] [--sealed-evaluation <path>] [--compact-train-stream <path>] [--runtime-manifest <path>] [--capture-mode <mode>] [--allow-summary-preview]"
        );
        return;
    }

    if args.iter().any(|a| a == "--self-test") {
        match run_self_test() {
            Ok(()) => {
                println!("{{\"self_test\":\"PASS\"}}");
                return;
            }
            Err(e) => {
                eprintln!("SELF_TEST_FAILED: {e}");
                process::exit(1);
            }
        }
    }

    if args.iter().any(|a| a == "--verify-host-pins") {
        match verify_host_pinned_inputs() {
            Ok(map) => {
                println!("{{\"verify_host_pins\":\"PASS\",\"pins_checked\":{}}}", map.len());
                return;
            }
            Err(e) => {
                eprintln!("HOST_PINS_FAILED: {e}");
                process::exit(1);
            }
        }
    }

    let find_arg = |flag: &str| -> Option<PathBuf> {
        args.windows(2)
            .find_map(|w| (w[0] == flag).then(|| PathBuf::from(&w[1])))
    };
    let allow_preview = args.iter().any(|a| a == "--allow-summary-preview");
    let capture_mode = args
        .windows(2)
        .find_map(|w| (w[0] == "--capture-mode").then(|| w[1].as_str()))
        .unwrap_or("q_only_2425925");

    let compact_summary = if let Some(stream_path) = find_arg("--compact-train-stream") {
        let raw = fs::read(&stream_path).unwrap_or_else(|e| {
            eprintln!("read {}: {e}", stream_path.display());
            process::exit(1);
        });
        let tb = fs::read(
            "/Users/casey.allard/uor-r4-local/codex-cf1-k16-rows-fastpath/qualification-prep/operator/data/mw-balp/train/tokens.u16",
        )
        .unwrap_or_else(|e| {
            eprintln!("read TRAIN tokens.u16: {e}");
            process::exit(1);
        });
        let mb = fs::read(
            "/Users/casey.allard/uor-r4-local/codex-cf1-k16-rows-fastpath/qualification-prep/operator/data/mw-balp/train/response_mask.u8",
        )
        .unwrap_or_else(|e| {
            eprintln!("read TRAIN response_mask.u8: {e}");
            process::exit(1);
        });
        if sha256_bytes(&tb) != PINNED_TRAIN_TOKENS_SHA256
            || sha256_bytes(&mb) != PINNED_TRAIN_MASK_SHA256
        {
            eprintln!("TRAIN corpus SHA-256 mismatch");
            process::exit(1);
        }
        Some(
            verify_compact_train_stream_bytes(
                &raw,
                capture_mode,
                None,
                false,
                true,
                None,
                Some(&tb),
                Some(&mb),
            )
            .unwrap_or_else(|e| {
                eprintln!("COMPACT_TRAIN_STREAM_FAILED: {e}");
                process::exit(1);
            }),
        )
    } else {
        None
    };

    if let Some(rec_path) = find_arg("--record") {
        let raw = fs::read(&rec_path).unwrap_or_else(|e| {
            eprintln!("read {}: {e}", rec_path.display());
            process::exit(1);
        });
        let rec = parse_json(&raw).unwrap_or_else(|e| {
            eprintln!("parse {}: {e}", rec_path.display());
            process::exit(1);
        });
        let (dev_rows, sealed_rows, rt_man) = if !allow_preview {
            let dt = fs::read(
                "/Users/casey.allard/uor-r4-local/codex-cf1-k16-rows-fastpath/qualification-prep/operator/data/ft/dev/tokens.u16",
            )
            .unwrap_or_else(|e| {
                eprintln!("read DEV tokens.u16: {e}");
                process::exit(1);
            });
            let dm = fs::read(
                "/Users/casey.allard/uor-r4-local/codex-cf1-k16-rows-fastpath/qualification-prep/operator/data/ft/dev/response_mask.u8",
            )
            .unwrap_or_else(|e| {
                eprintln!("read DEV response_mask.u8: {e}");
                process::exit(1);
            });
            if sha256_bytes(&dt) != PINNED_DEV_TOKENS_SHA256
                || sha256_bytes(&dm) != PINNED_DEV_MASK_SHA256
            {
                eprintln!("DEV corpus SHA-256 mismatch");
                process::exit(1);
            }
            let dev_parsed = parse_prepared_corpus_uort(
                &dt,
                &dm,
                PINNED_BOS_ID,
                SPLIT_TAG_DEV,
                CONTEXT_WINDOW,
                Some(DEV_PAYLOAD_TOKENS),
                Some(DEV_DOCUMENTS),
                Some(DEV_ROWS),
                None,
            )
            .unwrap_or_else(|e| {
                eprintln!("parse DEV UORT: {e}");
                process::exit(1);
            });
            let se_path = find_arg("--sealed-evaluation").unwrap_or_else(|| {
                eprintln!("verified runtime mode requires --sealed-evaluation <evaluation-16404.json>");
                process::exit(1);
            });
            let se_bytes = fs::read(&se_path).unwrap_or_else(|e| {
                eprintln!("read {}: {e}", se_path.display());
                process::exit(1);
            });
            if se_bytes.len() != PINNED_Q4_EVALUATION_BYTES
                || sha256_bytes(&se_bytes) != PINNED_Q4_EVALUATION_SHA256
            {
                eprintln!("sealed evaluation-16404.json size/SHA-256 mismatch");
                process::exit(1);
            }
            let se_doc = parse_json(&se_bytes).unwrap_or_else(|e| {
                eprintln!("parse sealed evaluation: {e}");
                process::exit(1);
            });
            let se_flat = flatten_sealed_evaluation_batches(&se_doc, &dev_parsed.rows, true)
                .unwrap_or_else(|e| {
                    eprintln!("flatten sealed evaluation: {e}");
                    process::exit(1);
                });
            let rm_path = find_arg("--runtime-manifest").unwrap_or_else(|| {
                eprintln!("verified runtime mode requires --runtime-manifest <path>");
                process::exit(1);
            });
            let rm_bytes = fs::read(&rm_path).unwrap_or_else(|e| {
                eprintln!("read {}: {e}", rm_path.display());
                process::exit(1);
            });
            let rm_parsed = verify_runtime_manifest_bytes(&rm_bytes).unwrap_or_else(|e| {
                eprintln!("verify runtime manifest: {e}");
                process::exit(1);
            });
            (Some(dev_parsed.rows), Some(se_flat), Some(rm_parsed))
        } else {
            (None, None, None)
        };

        let outcome = evaluate_w2_decision_table(
            &rec,
            true,
            !allow_preview,
            dev_rows.as_deref(),
            sealed_rows.as_deref(),
            compact_summary.as_ref(),
            rt_man.as_ref(),
            None,
        )
        .unwrap_or_else(|e| {
            eprintln!("W2_VERIFICATION_FAILED: {e}");
            process::exit(1);
        });
        println!(
            "{{\"verification_tier\":{:?},\"runtime_verdict_authorized\":{},\"precedence\":{},\"verdict\":{:?},\"candidate_verdict\":{:?},\"T\":{},\"U\":{},\"G\":{},\"M\":{},\"W\":{},\"coverage_rows\":{},\"codes_with_count_ge_2\":{}}}",
            outcome.verification_tier,
            outcome.runtime_verdict_authorized,
            outcome.precedence,
            outcome.verdict,
            outcome.candidate_verdict,
            outcome.t_top1,
            outcome.u_top1,
            outcome.g_top1,
            outcome.m_top1,
            outcome.w_top1,
            outcome.coverage_rows,
            outcome.codes_with_count_ge_2
        );
        if outcome.precedence == 0 {
            process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn w2_rust_validator_self_test() {
        run_self_test().expect("W2 Rust self-test should pass");
    }
}
