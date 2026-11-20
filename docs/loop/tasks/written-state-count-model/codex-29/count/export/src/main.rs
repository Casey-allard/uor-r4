#![forbid(unsafe_code)]

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    env,
    error::Error,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

type Failure = Box<dyn Error>;
const HEADER_BYTES: usize = 64;
const C_FULL: usize = 2_075_608_357;
const C_CUT: usize = 20_754_301;
const R_CUT: usize = 15_951_644;
const Q_FULL: usize = 80_271_733;

struct Input {
    name: &'static str,
    bytes: usize,
    prefix_bytes: usize,
    sha256: &'static str,
}

const C: Input = Input {
    name: "c.tokens.u16",
    bytes: HEADER_BYTES + 2 * C_FULL,
    prefix_bytes: HEADER_BYTES + 2 * C_CUT,
    sha256: "91db0e3005deb8a796d170d383b10273633acd781e272d6bd8547e653ef13334",
};
const Q: Input = Input {
    name: "q.tokens.u16",
    bytes: HEADER_BYTES + 2 * Q_FULL,
    prefix_bytes: HEADER_BYTES + 2 * R_CUT,
    sha256: "bab6ee5f2524014e54cb5f72acf555bf39dc9430fdb92dfed3e8e84f9359b454",
};
const CLASSES: Input = Input {
    name: "q.classes.u8",
    bytes: Q_FULL,
    prefix_bytes: R_CUT,
    sha256: "ca686ed8072b78fa03174530862842db60089b24e659f98f95e3e92921b465d5",
};
const SEGMENTS: Input = Input {
    name: "q.segments.u8",
    bytes: Q_FULL,
    prefix_bytes: R_CUT,
    sha256: "eab84282e3ff934f6344a2558db402db6b346f35d6d85dea98200c2fa85aa1ac",
};
const ID_BYTES: Input = Input {
    name: "id-bytes.u8",
    bytes: 4096,
    prefix_bytes: 4096,
    sha256: "befb22f6908751ae522a80336afa584eb2c24e36a10b90ad3105e39271a48ac9",
};

fn require(valid: bool, detail: &str) -> Result<(), Failure> {
    if !valid {
        return Err(detail.into());
    }
    Ok(())
}

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read_prefix(root: &Path, input: &Input) -> Result<(File, Vec<u8>), Failure> {
    let mut file = File::open(root.join(input.name))?;
    require(
        file.metadata()?.len() == input.bytes as u64,
        "wrong source length",
    )?;
    let mut bytes = vec![0; input.prefix_bytes];
    file.read_exact(&mut bytes)?;
    if sha(&bytes) != input.sha256 {
        return Err(format!("wrong source prefix digest for {}", input.name).into());
    }
    Ok((file, bytes))
}

fn read_uort_header(bytes: &[u8], original_count: usize) -> Result<(), Failure> {
    require(bytes.starts_with(b"UORT"), "wrong UORT magic")?;
    let count = u64::from_le_bytes(bytes[8..16].try_into()?);
    require(count == original_count as u64, "wrong UORT count")
}

fn verify_stream(
    bytes: &[u8],
    expected_records: usize,
    classes: Option<&[u8]>,
) -> Result<(), Failure> {
    let mut previous = 1;
    let mut records = 0;
    for (at, pair) in bytes[HEADER_BYTES..].chunks_exact(2).enumerate() {
        let id = u16::from_le_bytes(pair.try_into()?);
        require(id < 4096, "token outside original 12-bit vocabulary")?;
        require((id == 0) == (previous == 1), "broken BOS/EOS boundary")?;
        if let Some(classes) = classes {
            require(classes[at] == if id == 0 { 3 } else { 0 }, "wrong R class")?;
        }
        records += usize::from(id == 0);
        previous = id;
    }
    require(
        previous == 1 && records == expected_records,
        "wrong record count or final EOS",
    )
}

fn write(root: &Path, name: &str, bytes: &[u8]) -> Result<Value, Failure> {
    let mut file = File::options()
        .write(true)
        .create_new(true)
        .open(root.join(name))?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(json!({"file": name, "bytes": bytes.len(), "sha256": sha(bytes)}))
}

fn main() -> Result<(), Failure> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let [split, c420, output] = args.as_slice() else {
        return Err("usage: d29-prep-cr PINNED_C439_SPLIT PINNED_C420_BASE NEW_OUT_DIR".into());
    };
    let split = Path::new(split);
    let output = Path::new(output);
    let (mut c_source, mut c) = read_prefix(split, &C)?;
    let (_, mut r) = read_prefix(split, &Q)?;
    let (_, classes) = read_prefix(split, &CLASSES)?;
    let (_, segments) = read_prefix(split, &SEGMENTS)?;
    let (_, id_bytes) = read_prefix(Path::new(c420), &ID_BYTES)?;
    read_uort_header(&c, C_FULL)?;
    read_uort_header(&r, Q_FULL)?;
    require(
        c[..8] == r[..8] && c[16..HEADER_BYTES] == r[16..HEADER_BYTES],
        "C/R UORT lineage differs",
    )?;

    // Re-derive the historical last-BOS cut by checking the excluded BOS and
    // reading only the bounded remainder through the 1% mark.
    let mark = C_FULL / 100;
    let mut boundary = vec![0; (mark - C_CUT + 1) * 2];
    c_source.read_exact(&mut boundary)?;
    require(boundary[..2] == [0, 0], "C cut does not precede BOS")?;
    require(
        !boundary[2..].chunks_exact(2).any(|word| word == [0, 0]),
        "a later BOS precedes the 1% mark",
    )?;
    verify_stream(&c, 5_402, None)?;
    verify_stream(&r, 4_060, Some(&classes))?;
    require(
        segments.iter().all(|&segment| segment == 0),
        "R contains an evaluation segment",
    )?;

    c[8..16].copy_from_slice(&(C_CUT as u64).to_le_bytes());
    r[8..16].copy_from_slice(&(R_CUT as u64).to_le_bytes());
    fs::create_dir(output)?;
    let outputs = [
        write(output, "c1.tokens.u16", &c)?,
        write(output, "r.tokens.u16", &r)?,
        write(output, "r.classes.u8", &classes)?,
        write(output, "r.segments.u8", &segments)?,
        write(output, "id-bytes.u8", &id_bytes)?,
    ];
    let sources: Vec<_> = [C, Q, CLASSES, SEGMENTS, ID_BYTES]
        .iter()
        .map(|input| {
            json!({"file": input.name, "source_bytes": input.bytes,
               "verified_prefix_bytes": input.prefix_bytes, "prefix_sha256": input.sha256})
        })
        .collect();
    let manifest = json!({
        "schema": "uor-r4.d29-cr-preparation/1", "sources": sources, "outputs": outputs,
        "c": {"original_tokens": C_FULL, "mark_token": mark, "tokens": C_CUT, "records": 5402, "targets": C_CUT - 5402},
        "r": {"tokens": R_CUT, "records": 4060, "targets": R_CUT - 4060, "segment": 0},
        "c_boundary_sha256": sha(&boundary), "header_changed_bytes": [8, 16],
        "held_input": false, "training": false
    });
    let mut bytes = serde_json::to_vec_pretty(&manifest)?;
    bytes.push(b'\n');
    write(output, "manifest.json", &bytes)?;
    File::open(output)?.sync_all()?;
    println!("manifest SHA-256 {}", sha(&bytes));
    Ok(())
}
