//! One-way scoring and candidate selection on the original, sealed DEV-tune split.
#![forbid(unsafe_code)]

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    io::{BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
};
use uor_r4_discrete::kn_ceiling::{
    CeilingTable, CompactCountLookup, FrozenPickMixture, PickClassRule, PickTarget,
};
use uor_r4_integer::lawful_likelihood::bits_q12;

#[path = "../../../../../labs/kn-reference/instrument/src/pick_arms.rs"]
mod pick_arms;
#[cfg(feature = "cuda")]
#[path = "../../../../../labs/kn-reference/instrument/src/pick_device.rs"]
mod pick_device;
#[cfg(feature = "cuda")]
mod score;

type Failure = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Failure>;
const CLASS_NOVEL: u8 = 1;
const TOKENS: usize = 10_810_087;
const RECORDS: usize = 2670;
const ROWS: usize = TOKENS - RECORDS;
const WIDTH: usize = 11;
const EXPERTS: usize = 14;
const BATCH: usize = 8192;
const CAPTURE_BATCH: usize = 65536;
const COMPACT_BYTES: u64 = 3_912_334_481;
const C_SHA: &str = "cd3adfe949ee34a62bc8d5e48592e091fcece796618d7910d1667d014a4bae2a";
const COMPACT_SHA: &str = "064b691a54895771df1a957b2f97c91163990b6c6b6e09504c43285a97c49ef8";
const R_SHA: &str = "5875b173ad19aefbdd38929af8d88c15344333e941f44128d5e0a67b50102a0a";
const TRUNK_SHA: &str = "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d";
const CENSUS_SHA: &str = "90fbc73a7b19129d39860cece7403027f71d6d952a6825e89bd1619dac3f8467";
const ID_BYTES_SHA: &str = "befb22f6908751ae522a80336afa584eb2c24e36a10b90ad3105e39271a48ac9";
const TOKENIZER_SHA: &str = "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89";

fn fail(message: impl Into<String>) -> Failure {
    message.into().into()
}
fn require(condition: bool, detail: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(fail(detail))
    }
}
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn pin(value: &str) -> Result<[u8; 32]> {
    let mut pin = [0; 32];
    hex::decode_to_slice(value, &mut pin)?;
    Ok(pin)
}
fn word(bytes: &[u8]) -> Result<u64> {
    Ok(u64::from_le_bytes(bytes.try_into()?))
}
fn pinned(path: &Path, expected: &str, len: usize) -> Result<Vec<u8>> {
    let mut file = fs::File::open(path)?;
    require(
        file.metadata()?.len() == len as u64,
        "wrong pinned input length",
    )?;
    let mut bytes = Vec::with_capacity(len);
    Read::by_ref(&mut file)
        .take(len as u64 + 1)
        .read_to_end(&mut bytes)?;
    require(
        bytes.len() == len && sha(&bytes) == expected,
        "pinned input SHA-256 mismatch",
    )?;
    Ok(bytes)
}
fn new(path: &Path) -> Result<fs::File> {
    Ok(fs::File::options()
        .write(true)
        .create_new(true)
        .open(path)?)
}

#[derive(Debug, Default, Clone, Copy)]
struct Tally {
    targets: u64,
    bits_q12: u128,
    bytes: u64,
}
impl Tally {
    fn add(&mut self, bits: u64, bytes: u8) {
        self.targets += 1;
        self.bits_q12 += u128::from(bits);
        self.bytes += u64::from(bytes);
    }
    fn json(self) -> Value {
        let micro =
            (self.bytes > 0).then(|| self.bits_q12 * 1_000_000 / (4096 * u128::from(self.bytes)));
        json!({"targets": self.targets, "bytes": self.bytes, "bits_q12": self.bits_q12.to_string(),
            "bpb_micro": micro.map(|x| x.to_string())})
    }
}

struct Tune {
    ids: Vec<u16>,
    novel: Vec<u8>,
    mask: Vec<u8>,
    bytes: Vec<u8>,
    uort: Vec<u8>,
}
impl Tune {
    fn load(root: &Path) -> Result<Self> {
        let census = fs::read(root.join("dev-tune.json"))?;
        require(sha(&census) == CENSUS_SHA, "wrong DEV-tune census")?;
        let census: Value = serde_json::from_slice(&census)?;
        require(
            census["split"] == "dev-tune"
                && census["records"] == RECORDS
                && census["tokens"] == TOKENS
                && census["targets"] == ROWS
                && census["id_bytes_sha256"] == ID_BYTES_SHA
                && census["tokenizer_sha256"] == TOKENIZER_SHA,
            "wrong DEV-tune census identity",
        )?;
        let load = |index: usize, name: &str, len: usize| -> Result<Vec<u8>> {
            let entry = &census["files"][index];
            require(
                entry["file"] == name && entry["bytes"] == len,
                "unexpected census entry",
            )?;
            pinned(&root.join(name), string(&entry["sha256"])?, len)
        };
        let raw = load(0, "dev-tune.tokens.u16", 64 + 2 * TOKENS)?;
        let mask = load(1, "dev-tune.mask.u8", TOKENS)?;
        let segments = load(2, "dev-tune.segment.u8", TOKENS)?;
        let novel = load(3, "dev-tune.novel.u8", TOKENS)?;
        let bytes = pinned(&root.join("id-bytes.u8"), ID_BYTES_SHA, 4096)?;
        require(
            raw[..4] == *b"UORT"
                && raw[4..8] == 1u32.to_le_bytes()
                && raw[16..20] == 4096u32.to_le_bytes(),
            "wrong preserved UORT header",
        )?;
        let ids = uor_r4_discrete::ngram_count::read_uort_tokens(&raw)?;
        let split = Self {
            ids,
            novel,
            mask,
            bytes,
            uort: raw,
        };
        split.validate(&census, &segments)?;
        Ok(split)
    }

    fn validate(&self, census: &Value, segments: &[u8]) -> Result<()> {
        let mut all = Tally::default();
        let mut response = Tally::default();
        let mut classes = [[Tally::default(); 3]; 2];
        let (mut frames, mut position, mut prior) = (0, 0, 1);
        for (at, &id) in self.ids.iter().enumerate() {
            let (class, mask) = (self.novel[at] as usize, self.mask[at]);
            if id >= 4096
                || (id == 0) != (prior == 1)
                || mask > 1
                || segments[at] > 3
                || (segments[at] == 1 && mask != 0)
                || (segments[at] >= 2 && mask != 1)
                || (id == 1 && (segments[at] != 0 || mask != 1))
            {
                return Err(fail(format!(
                    "wrong DEV-tune token, segment, mask or frame at {at}"
                )));
            }
            prior = id;
            if id == 0 {
                require(
                    class == 3 && mask == 0 && segments[at] == 0,
                    "BOS must be outside all scoring masks",
                )?;
                frames += 1;
                position = 0;
                continue;
            }
            let len = self.bytes[id as usize];
            require(
                class < 3 && (class == 2) == (position < 12),
                "wrong DEV-tune novelty class",
            )?;
            position += 1;
            all.add(0, len);
            classes[0][class].add(0, len);
            if mask == 1 {
                response.add(0, len);
                classes[1][class].add(0, len);
            }
        }
        require(
            frames == RECORDS
                && prior == 1
                && census["target_bytes"] == all.bytes
                && census["response_bytes"] == response.bytes
                && census["response_targets"] == response.targets
                && census["targets"] == all.targets
                && self.bytes[..3] == [0; 3],
            "DEV-tune aggregate census mismatch",
        )?;
        for (class, name) in ["seen", "novel", "short_context"].into_iter().enumerate() {
            let count = &census["novel"]["classes"][name];
            require(
                count["bytes"] == classes[0][class].bytes
                    && count["targets"] == classes[0][class].targets
                    && count["response_bytes"] == classes[1][class].bytes
                    && count["response_targets"] == classes[1][class].targets,
                "DEV-tune class census mismatch",
            )?;
        }
        Ok(())
    }
}
fn string(value: &Value) -> Result<&str> {
    value.as_str().ok_or_else(|| fail("missing string field"))
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).map(PathBuf::from).collect();
    match args.as_slice() {
        [mode, root] if mode == Path::new("validate") => {
            let tune = Tune::load(root)?;
            println!("{}", json!({"uort_sha256": sha(&tune.uort), "census_sha256": CENSUS_SHA, "rows": ROWS, "records": RECORDS}));
            Ok(())
        }
        #[cfg(feature = "cuda")]
        [mode, root, tape, digest] if mode == Path::new("verify-c2") => score::verify_c2(root, tape, digest),
        #[cfg(feature = "cuda")]
        [mode, runner] if mode == Path::new("finite") => score::finite(runner),
        #[cfg(feature = "cuda")]
        [mode, root, go, go_sha, compact, trunk, tape, c2_tape, runner, fit_root, out] if mode == Path::new("score") =>
            score::run(root, go, go_sha, compact, trunk, tape, c2_tape, runner, fit_root, out),
        _ => Err(fail("usage: validate SPLIT_ROOT | finite CUDA_RUNNER | verify-c2 SPLIT_ROOT C2_TAPE C2_SHA | score SPLIT_ROOT GO GO_SHA COMPACT TRUNK TAPE C2_TAPE CUDA_RUNNER FIT_ROOT OUT")),
    }
}
