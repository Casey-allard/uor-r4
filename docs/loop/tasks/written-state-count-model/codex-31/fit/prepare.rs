use super::*;
use crate::pick_device::{load_trunk, row_from_words, windows, PickDevice, TRUNK_SHA256};
use std::time::Instant;
use uor_r4_discrete::{
    kn_ceiling::{CompactCountLookup, RawTables},
    kn_filter::KnQuery,
    kn_lookup_device::{compiled_kn_lookup_image, KnLookup},
    kn_score::KnTables,
    kn_score_device::{compiled_kn_score_image, KnScorer},
    ngram_count::{read_uort_tokens, NgramAssembler, NgramOrderDigest},
    ngram_count_device::NgramCounter,
};

const RECORDS: usize = 4060;
const CONTEXT: usize = 192;
const BUDGET: u64 = 1 << 24;

fn pinned(path: &Path, sha: &str) -> Result<Vec<u8>, Failure> {
    let bytes = fs::read(path)?;
    if digest(&bytes) != sha {
        return Err(fail(format!("wrong input identity for {}", path.display())));
    }
    Ok(bytes)
}

fn check_order(want: &Value, got: &NgramOrderDigest) -> Result<(), Failure> {
    if want["order"] != got.order
        || want["targets"] != got.targets
        || want["grams"] != got.grams
        || want["contexts"] != got.contexts
        || want["triple_sha256"] != got.triple_sha256
        || want["context_sha256"] != got.context_sha256
        || want["count_of_counts"] != json!(got.count_of_counts)
    {
        return Err(fail(format!(
            "historical global order {} differs",
            got.order
        )));
    }
    Ok(())
}

fn lookups(
    c1: &[u16],
    r: &[u16],
    original: &Value,
) -> Result<(Vec<u64>, Vec<u32>, Vec<u64>), Failure> {
    let want = KnQuery::from_stream(r, N)?;
    let mut filters = (0..=N)
        .map(|order| want.filter(order))
        .collect::<Result<Vec<_>, _>>()?;
    let mut assemblers = (0..=N)
        .map(|order| NgramAssembler::new_adjusted(order, false))
        .collect::<Result<Vec<_>, _>>()?;
    let counter = NgramCounter::new(0, c1)?;
    counter.count_adjusted_sweep(N, BUDGET, |k, partition| {
        filters[k as usize].keep(&partition)?;
        assemblers[k as usize].push_owned(partition)
    })?;
    let mut adjusted = Vec::new();
    for (k, (a, f)) in assemblers.into_iter().zip(filters).enumerate() {
        let (actual, _, _) = a.finish();
        check_order(&original["orders"][k], &actual)?;
        let (triples, contexts) = f.finish();
        adjusted.push((actual, triples, contexts));
    }
    let top = adjusted
        .last()
        .ok_or_else(|| fail("missing order ten"))?
        .clone();
    let adjusted = KnTables::new(N, adjusted)?;
    let masses = KnScorer::new(0, &adjusted)?.score(r)?;
    drop(adjusted);
    let mut raw = Vec::new();
    for k in 0..N {
        let mut filter = want.filter(k)?;
        let mut assembler = NgramAssembler::new(k, false)?;
        for range in counter.plan(k, BUDGET)? {
            let partition = counter.count_partition(k, &range)?;
            filter.keep(&partition)?;
            assembler.push_owned(partition)?;
        }
        let (actual, _, _) = assembler.finish();
        check_order(&original["raw_orders"][k as usize], &actual)?;
        let (triples, contexts) = filter.finish();
        raw.push((actual, triples, contexts));
    }
    // The original raw order-ten receipt deliberately reused the adjusted
    // digest and table, since the highest order stores its raw count words.
    check_order(&original["raw_orders"][N as usize], &top.0)?;
    raw.push(top);
    drop(counter);
    let raw = RawTables::new(N, raw)?;
    let (counts, totals) = KnLookup::new(0, &raw)?.lookups(r)?;
    Ok((masses, counts, totals))
}

pub fn run(
    dir: &Path,
    trunk: &Path,
    compact: &Path,
    compact_sha: &Path,
    out: &Path,
) -> Result<(), Failure> {
    let start = Instant::now();
    let c1 = read_uort_tokens(&pinned(&dir.join("c1.tokens.u16"), C1_SHA)?)?;
    let r = read_uort_tokens(&pinned(&dir.join("r.tokens.u16"), R_SHA)?)?;
    let classes = pinned(
        &dir.join("r.classes.u8"),
        "ca686ed8072b78fa03174530862842db60089b24e659f98f95e3e92921b465d5",
    )?;
    let segments = pinned(
        &dir.join("r.segments.u8"),
        "eab84282e3ff934f6344a2558db402db6b346f35d6d85dea98200c2fa85aa1ac",
    )?;
    let id_bytes = pinned(
        &dir.join("id-bytes.u8"),
        "befb22f6908751ae522a80336afa584eb2c24e36a10b90ad3105e39271a48ac9",
    )?;
    if c1.len() != 20_754_301
        || r.len() != REAL_ROWS + RECORDS
        || r.iter().filter(|&&id| id == 0).count() != RECORDS
        || r.first() != Some(&0)
        || r.last() != Some(&1)
        || classes.len() != r.len()
        || segments.len() != r.len()
        || id_bytes.len() != 4096
        || segments.iter().any(|&s| s != 0)
        || r.iter()
            .zip(&classes)
            .any(|(&id, &c)| c != if id == 0 { 3 } else { 0 })
    {
        return Err(fail("incorrect sealed C1/R structure"));
    }
    drop((classes, segments, id_bytes));
    let compact_sha = compact_sha
        .to_str()
        .ok_or_else(|| fail("non-UTF8 compact digest"))?;
    let model = load_trunk(trunk)?;
    fs::create_dir(out)?;
    let original: Value = serde_json::from_slice(ORIGINAL)?;
    let (kn, counts, totals) = lookups(&c1, &r, &original)?;
    drop(c1);
    if kn.len() != REAL_ROWS || counts.len() != REAL_ROWS * WIDTH || totals.len() != counts.len() {
        return Err(fail("incorrect reserved-R GPU lookup dimensions"));
    }
    eprintln!(
        "R count/KN lookup complete at {} ms",
        start.elapsed().as_millis()
    );
    let mut expected_ordinals = Vec::new();
    let mut ordinal = 0;
    let mut record = 0;
    let mut position = 0;
    for &id in &r {
        if id == 0 {
            record += 1;
            position = 0;
        } else {
            position += 1;
            if record <= 64 && (position <= 8 || position == 192) {
                expected_ordinals.push(ordinal);
            }
            ordinal += 1;
        }
    }
    let dev = PickDevice::new(&model, None)?;
    let keep = |ordinal| expected_ordinals.binary_search(&ordinal).is_ok();
    let pick = dev.rows(&r, CONTEXT, &keep)?;
    drop(dev);
    if pick.rows.len() != REAL_ROWS || pick.words.len() != expected_ordinals.len() {
        return Err(fail("wrong reserved-R CUDA Pick tape length"));
    }
    eprintln!("R A1 Pick complete at {} ms", start.elapsed().as_millis());

    let compact_bytes = fs::read(compact)?;
    let original_training = <[u8; 32]>::try_from(hex::decode(C1_SHA)?.as_slice())?;
    let original_payload = <[u8; 32]>::try_from(hex::decode(compact_sha)?.as_slice())?;
    let packed =
        CompactCountLookup::from_bytes(&compact_bytes, original_training, original_payload)?;
    let windows = windows(&r, CONTEXT)?;
    let mut probes = Vec::with_capacity(pick.words.len());
    for (&ordinal, kept) in expected_ordinals.iter().zip(&pick.words) {
        let (at, prefix) = windows[ordinal];
        let original_ids: Vec<u32> = prefix.iter().map(|&id| id as u32).collect();
        let served = model.observe_identity_pick(&original_ids)?;
        let row = row_from_words(prefix, r[at], &kept.blocks[0])?;
        if ordinal != kept.ordinal
            || served.block_words.as_slice() != kept.blocks.as_slice()
            || row != pick.rows[ordinal]
        {
            return Err(fail(format!(
                "reloaded D11 Pick trace differs at R ordinal {ordinal}"
            )));
        }
        let mut a = [0u32; WIDTH];
        let mut b = [0u64; WIDTH];
        let mass = packed.lookup(&r, at, &mut a, &mut b)?;
        let mut local = Vec::with_capacity(prefix.len() + 1);
        local.extend_from_slice(prefix);
        local.push(r[at]);
        let mut local_counts = [0; WIDTH];
        let mut local_totals = [0; WIDTH];
        let local_mass =
            packed.lookup(&local, prefix.len(), &mut local_counts, &mut local_totals)?;
        if (mass, a, b) != (local_mass, local_counts, local_totals) {
            return Err(fail(format!(
                "record-local and full tape lookups differ at R ordinal {ordinal}"
            )));
        }
        let span = ordinal * WIDTH..(ordinal + 1) * WIDTH;
        if mass != kn[ordinal]
            || a.as_slice() != &counts[span.clone()]
            || b.as_slice() != &totals[span]
        {
            return Err(fail(format!(
                "reloaded compact lookup differs at R ordinal {ordinal}"
            )));
        }
        probes.push(
            json!({"ordinal": ordinal, "position": at, "prefix": prefix, "target": r[at],
            "pick_class": row.class, "pick_hit": row.hit, "blocks": kept.blocks,
            "kn_q48": mass, "counts": a, "totals": b, "successor": served.successor}),
        );
    }
    if probes.len() < 64
        || !(0..=2).all(|class| probes.iter().any(|probe| probe["pick_class"] == class))
    {
        return Err(fail("reserved-R replay lacks count or Pick classes"));
    }
    drop((windows, packed));
    drop(compact_bytes);
    let probes = serde_json::to_vec_pretty(&probes)?;
    write_new(&out.join("probes.json"), &probes)?;
    let indexes: Vec<_> = expected_ordinals
        .iter()
        .flat_map(|&ordinal| (ordinal as u64).to_le_bytes())
        .collect();
    write_new(&out.join("probe-ordinals.u64"), &indexes)?;
    let rows_path = out.join("train.rows");
    let mut out_rows = BufWriter::with_capacity(1 << 20, new(&rows_path)?);
    row_header(&mut out_rows, REAL_ROWS)?;
    for lo in (0..REAL_ROWS).step_by(1 << 16) {
        let hi = (lo + (1 << 16)).min(REAL_ROWS);
        let raw = lo * WIDTH..hi * WIDTH;
        let picked: Vec<_> = pick.rows[lo..hi]
            .iter()
            .map(|row| PickTarget {
                class: row.class,
                hit: row.hit,
            })
            .collect();
        let table = CeilingTable::from_lookups_with_pick(
            N,
            &kn[lo..hi],
            &counts[raw.clone()],
            &totals[raw],
            &picked,
        )?;
        write_rows(&mut out_rows, &table)?;
    }
    out_rows.flush()?;
    out_rows.get_ref().sync_all()?;
    drop(out_rows);
    let rows_sha = {
        let mut hasher = Sha256::new();
        std::io::copy(&mut fs::File::open(&rows_path)?, &mut hasher)?;
        format!("{:x}", hasher.finalize())
    };
    let receipt = serde_json::to_vec_pretty(&json!({
        "format": "uor-r4.d31.a1-preparation/1",
        "rows": REAL_ROWS, "records": RECORDS,
        "c_sha256": C1_SHA, "r_sha256": R_SHA, "trunk_sha256": TRUNK_SHA256,
        "compact_sha256": compact_sha, "rows_sha256": rows_sha,
        "probes_sha256": digest(&probes), "probe_ordinals_sha256": digest(&indexes),
        "historical_global_orders_verified": [11, 11], "budget": BUDGET,
        "cuda_raw_image": compiled_kn_lookup_image(), "cuda_kn_image": compiled_kn_score_image(),
        "host_source_sha256": digest(include_bytes!("main.rs")), "preparation_source_sha256": digest(include_bytes!("prepare.rs")),
        "device_source_sha256": digest(include_bytes!("device.cu")),
        "elapsed_ms": start.elapsed().as_millis()
    }))?;
    write_new(&out.join("prepare.json"), &receipt)?;
    Ok(())
}
