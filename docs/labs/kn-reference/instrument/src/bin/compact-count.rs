//! Complete GPU count materialization and count-storage parity on sealed C128 TRAIN only.
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{env, fs, io::Write, mem::size_of, path::Path};
use uor_r4_discrete::{
    kn_ceiling::{
        encode_compact_lookup, encode_complete_lookup, CompactCountLookup, CompleteCountLookup,
    },
    ngram_count::{read_uort_tokens, NgramAssembler, NgramContext, NgramTriple, NGRAM_MAX_ORDER},
    ngram_count_device::NgramCounter,
};
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

type Fail = Box<dyn std::error::Error>;
const TRAIN: &str = "1466d3790e1c53bc4ac3982712f669967cce7b3ad1d7e0e34259568154d19cb6";
const BASE: &str = "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d";
const N: u32 = NGRAM_MAX_ORDER;
const WIDTH: usize = N as usize + 1;

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), Fail> {
    let mut file = fs::File::options()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn main() -> Result<(), Fail> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let [train, trunk, output] = args.as_slice() else {
        return Err("usage: compact-count C128-UORT PINNED-D11 NEW-OUTPUT".into());
    };
    let source = fs::read(train)?;
    let source_sha = Sha256::digest(&source);
    if format!("{source_sha:x}") != TRAIN {
        return Err("compact count accepts only the predeclared C128 TRAIN prefix".into());
    }
    let tokens = read_uort_tokens(&source)?;
    if tokens.len() != 111_671 || tokens.iter().filter(|&&t| t == 0).count() != 128 {
        return Err("unexpected C128 record boundaries".into());
    }
    let trunk = fs::read(trunk)?;
    if format!("{:x}", Sha256::digest(&trunk)) != BASE {
        return Err("unexpected D11 trunk identity".into());
    }
    let _loaded_trunk = IntegerDiscreteModel::from_bytes(&trunk)?;
    let out = Path::new(output);
    fs::create_dir(out)?;
    let counter = NgramCounter::new(0, &tokens)?;
    let mut raw_orders = Vec::with_capacity(WIDTH);
    for order in 0..=N {
        let mut raw = NgramAssembler::new(order, true)?;
        counter.count_order(order, 1 << 20, &mut raw)?;
        raw_orders.push(raw.finish());
    }
    let mut adjusted = (0..=N)
        .map(|order| NgramAssembler::new_adjusted(order, true))
        .collect::<Result<Vec<_>, _>>()?;
    counter.count_adjusted_sweep(N, 1 << 20, |order, rows| {
        adjusted[order as usize].push_owned(rows)
    })?;
    let adjusted_orders: Vec<_> = adjusted.into_iter().map(NgramAssembler::finish).collect();
    drop(counter);
    let census = |orders: &[(
        uor_r4_discrete::ngram_count::NgramOrderDigest,
        Vec<NgramTriple>,
        Vec<NgramContext>,
    )]| {
        orders
            .iter()
            .map(|(d, _, _)| {
                json!({
                    "order": d.order, "targets": d.targets, "grams": d.grams,
                    "contexts": d.contexts, "triple_sha256": d.triple_sha256,
                    "context_sha256": d.context_sha256, "count_of_counts": d.count_of_counts
                })
            })
            .collect::<Vec<_>>()
    };
    let raw = census(&raw_orders);
    let adjusted = census(&adjusted_orders);
    let old_rows_bytes: usize = raw_orders
        .iter()
        .chain(&adjusted_orders)
        .map(|(_, triples, contexts)| {
            triples.len() * size_of::<NgramTriple>() + contexts.len() * size_of::<NgramContext>()
        })
        .sum();
    let baseline = encode_complete_lookup(N, source_sha.into(), &raw_orders, &adjusted_orders)?;
    let compact = encode_compact_lookup(N, source_sha.into(), &raw_orders, &adjusted_orders)?;
    drop(raw_orders);
    drop(adjusted_orders);
    let old_hash = Sha256::digest(&baseline);
    let new_hash = Sha256::digest(&compact);
    let old_len = baseline.len();
    let new_len = compact.len();
    let baseline_path = out.join("baseline.counts");
    let compact_path = out.join("compact.counts");
    write_new(&baseline_path, &baseline)?;
    write_new(&compact_path, &compact)?;
    drop(baseline);
    drop(compact);
    let old_file = fs::read(baseline_path)?;
    let new_file = fs::read(compact_path)?;
    let old = CompleteCountLookup::from_bytes(&old_file, source_sha.into(), old_hash.into())?;
    drop(old_file);
    let new = CompactCountLookup::from_bytes(&new_file, source_sha.into(), new_hash.into())?;
    let [mut a, mut b] = [[0u32; WIDTH]; 2];
    let [mut x, mut y] = [[0u64; WIDTH]; 2];
    let mut tape_hash = Sha256::new();
    let mut seen = 0;
    let mut check = |stream: &[u16], at: usize| -> Result<(), Fail> {
        let p = old.lookup(stream, at, &mut a, &mut x)?;
        let q = new.lookup(stream, at, &mut b, &mut y)?;
        if (p, a, x) != (q, b, y) {
            return Err(format!("count lookup differs at probe {seen}").into());
        }
        for (count, total) in a.into_iter().zip(x) {
            tape_hash.update(count.to_le_bytes());
            tape_hash.update(total.to_le_bytes());
        }
        tape_hash.update(p.to_le_bytes());
        seen += 1;
        Ok(())
    };
    let mut interior = 0;
    for at in (0..tokens.len()).step_by(13) {
        if tokens[at] != 0 {
            check(&tokens, at)?;
            interior += 1;
        }
    }
    let mut state = 0xb13dd5204ce843a5u64;
    for i in 0..4096 {
        let mut history = [0u16; WIDTH];
        let at = i % WIDTH;
        for token in &mut history[1..] {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            *token = 1 + (state % 4095) as u16;
        }
        if at == 0 {
            history[0] = 1 + (state % 4095) as u16;
        }
        check(&history, at)?;
    }
    let receipt = json!({
        "scope": "complete C128 TRAIN counts only",
        "train_sha256": TRAIN, "train_tokens": tokens.len(), "train_records": 128,
        "raw": raw, "adjusted": adjusted,
        "baseline_count_sha256": format!("{old_hash:x}"),
        "compact_count_sha256": format!("{new_hash:x}"),
        "baseline_count_bytes": old_len, "compact_count_bytes": new_len,
        "baseline_resident_row_bytes": old_rows_bytes, "compact_resident_count_payload_bytes": new_file.len(),
        "base_trunk_sha256": BASE, "base_trunk_bytes": trunk.len(),
        "a1_weight_layout_bytes_unfilled": 44 + 528 * 14 * 8,
        "lookup_tape_sha256": format!("{:x}", tape_hash.finalize()),
        "lookup_targets": seen, "train_interior": interior, "generated_histories": 4096
    });
    write_new(
        &out.join("receipt.json"),
        &serde_json::to_vec_pretty(&receipt)?,
    )?;
    println!("{receipt}");
    Ok(())
}
