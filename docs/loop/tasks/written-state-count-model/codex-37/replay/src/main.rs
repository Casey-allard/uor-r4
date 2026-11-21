//! Independent wire, framing and sampled ordinary-reader parity for a GPU feature tape.
use sha2::{Digest, Sha256};
use std::{
    env,
    error::Error,
    fs,
    fs::File,
    io::{BufReader, Read},
};
use uor_r4_count_runtime::CompactCountLookup;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const ROW: usize = 140;
const HEADER: usize = 112;
const ORDERS: usize = 11;

fn pin(s: &str) -> Result<[u8; 32]> {
    Ok(hex::decode(s)?
        .try_into()
        .map_err(|_| "digest must be 32 bytes")?)
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let [compact_path, train_sha, compact_sha, uort_path, uort_sha, tape_path, width] =
        args.as_slice()
    else {
        return Err("expected COMPACT TRAIN_SHA COMPACT_SHA UORT UORT_SHA TAPE WIDTH".into());
    };
    let width: usize = width.parse()?;
    if !(1..=65536).contains(&width) {
        return Err("invalid batch width".into());
    }
    let (train, compact, uort) = (pin(train_sha)?, pin(compact_sha)?, pin(uort_sha)?);
    let source = fs::read(uort_path)?;
    if source.len() < 64
        || Sha256::digest(&source)[..] != uort
        || &source[..4] != b"UORT"
        || source[4..8] != 1u32.to_le_bytes()
        || source[16..20] != 4096u32.to_le_bytes()
    {
        return Err("invalid UORT header or identity".into());
    }
    let declared = u64::from_le_bytes(source[8..16].try_into()?);
    if declared.checked_mul(2) != Some((source.len() - 64) as u64) {
        return Err("UORT length mismatch".into());
    }
    let ids: Vec<u16> = source[64..]
        .chunks_exact(2)
        .map(|v| u16::from_le_bytes([v[0], v[1]]))
        .collect();
    if ids.last() != Some(&1) {
        return Err("missing terminal EOS".into());
    }
    let table_bytes = fs::read(compact_path)?;
    let table = CompactCountLookup::from_bytes(&table_bytes, train, compact)?;
    let file = File::open(tape_path)?;
    let file_len = file.metadata()?.len();
    let mut reader = BufReader::with_capacity(1 << 20, file);
    let mut header = [0; HEADER];
    reader.read_exact(&mut header)?;
    let rows = u64::from_le_bytes(header[8..16].try_into()?);
    if &header[..8] != b"UORCBF01"
        || header[16..48] != train
        || header[48..80] != compact
        || header[80..112] != uort
        || rows
            .checked_mul(ROW as u64)
            .and_then(|n| n.checked_add(HEADER as u64))
            != Some(file_len)
    {
        return Err("tape header identity or length mismatch".into());
    }
    let mut hash = Sha256::new();
    hash.update(header);
    let mut ordinal = 0usize;
    let mut sampled = 0usize;
    let mut records = 0usize;
    let mut row = [0; ROW];
    for record in ids.split_inclusive(|&id| id == 1) {
        if record.first() != Some(&0)
            || record.len() < 2
            || record[1..].iter().any(|&id| id == 0 || id >= 4096)
        {
            return Err(format!("bad framing at record {records}").into());
        }
        for pos in 1..record.len() {
            reader.read_exact(&mut row)?;
            hash.update(row);
            let mut counts = [0; ORDERS];
            let mut totals = [0; ORDERS];
            for (dst, word) in counts.iter_mut().zip(row[..44].chunks_exact(4)) {
                *dst = u32::from_le_bytes(word.try_into()?);
            }
            for (dst, word) in totals.iter_mut().zip(row[44..132].chunks_exact(8)) {
                *dst = u64::from_le_bytes(word.try_into()?);
            }
            let mass = u64::from_le_bytes(row[132..].try_into()?);
            if !(1..=1u64 << 48).contains(&mass)
                || (0..ORDERS)
                    .any(|k| u64::from(counts[k]) > totals[k] || (k > pos && totals[k] != 0))
            {
                return Err(format!(
                    "invalid feature at ordinal {ordinal}, record {records}, position {pos}"
                )
                .into());
            }
            if pos == 1
                || pos + 1 == record.len()
                || ordinal % width == 0
                || ordinal % width == width - 1
                || ordinal % 4093 == 197
            {
                let mut nc = [0; ORDERS];
                let mut nt = [0; ORDERS];
                let history = &record[pos.saturating_sub(10)..=pos];
                let nm = table.lookup(history, history.len() - 1, &mut nc, &mut nt)?;
                if nm != mass || nc != counts || nt != totals {
                    return Err(format!("native mismatch ordinal {ordinal} record {records} source position {pos} target {}: counts {counts:?}/{nc:?} totals {totals:?}/{nt:?} Q48 {mass}/{nm}", record[pos]).into());
                }
                sampled += 1;
            }
            ordinal += 1;
        }
        records += 1;
    }
    if rows != ordinal as u64 || reader.read(&mut [0])? != 0 {
        return Err("trailing or missing ordinal".into());
    }
    println!(
        r#"{{"rows":{rows},"records":{records},"width":{width},"bytes":{file_len},"native_rows":{sampled},"words_per_native_row":23,"sha256":"{}"}}"#,
        hex::encode(hash.finalize())
    );
    Ok(())
}
fn main() -> Result<()> {
    run()
}
