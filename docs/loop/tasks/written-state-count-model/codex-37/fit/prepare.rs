use super::*;
use crate::pick_device::{load_trunk, root_codes, windows, PickDevice, C3_SEED, READ_BATCH};
use std::{
    io::{BufReader, Read},
    time::Instant,
};
use uor_r4_discrete::{
    kn_ceiling::CompactCountLookup,
    kn_pick::{code_pick, exact_pair_pick, permute_code_rows, row_permutation, PickRow},
    ngram_count::read_uort_tokens,
};

const CONTEXT: usize = 192;
const TAPE_HEADER: usize = 112;
const COUNT_ROW: usize = 140;
const COMPACT_BYTES: u64 = 3_912_334_481;
const CAPTURE_BATCH: usize = 65_536;
const CHUNK: usize = 1 << 16;

fn pin(hex: &str) -> Result<[u8; 32], Failure> {
    let mut out = [0; 32];
    hex::decode_to_slice(hex, &mut out)?;
    Ok(out)
}

fn load_r(path: &Path) -> Result<Vec<u16>, Failure> {
    if fs::metadata(path)?.len() != (64 + 2 * (REAL_ROWS + RECORDS)) as u64 {
        return Err(fail("wrong original R UORT byte length"));
    }
    let bytes = fs::read(path)?;
    if digest(&bytes) != R_SHA || bytes.get(16..20) != Some(4096u32.to_le_bytes().as_slice()) {
        return Err(fail("wrong original R UORT digest or vocabulary"));
    }
    let ids = read_uort_tokens(&bytes)?;
    let mut prior = 1;
    let mut records = 0;
    for (at, &id) in ids.iter().enumerate() {
        if id >= 4096 || (id == 0) != (prior == 1) {
            return Err(fail(format!("invalid original R frame at token {at}")));
        }
        records += usize::from(id == 0);
        prior = id;
    }
    if ids.len() != REAL_ROWS + RECORDS || records != RECORDS || prior != 1 {
        return Err(fail("wrong original R row or frame count"));
    }
    Ok(ids)
}

type CountRow = (u64, [u32; WIDTH], [u64; WIDTH]);

struct CountTape {
    stream: BufReader<fs::File>,
    digest: Sha256,
    ordinal: usize,
}

impl CountTape {
    fn open(path: &Path) -> Result<Self, Failure> {
        let file = fs::File::open(path)?;
        if file.metadata()?.len() != (TAPE_HEADER + COUNT_ROW * REAL_ROWS) as u64 {
            return Err(fail("wrong UORCBF01 count tape byte length"));
        }
        let mut stream = BufReader::with_capacity(1 << 20, file);
        let mut header = [0; TAPE_HEADER];
        stream.read_exact(&mut header)?;
        if &header[..8] != b"UORCBF01"
            || word(&header[8..16])? != REAL_ROWS as u64
            || header[16..48] != pin(C_SHA)?
            || header[48..80] != pin(COMPACT_SHA)?
            || header[80..] != pin(R_SHA)?
        {
            return Err(fail(
                "count tape is not bound to full C, compact, and original R",
            ));
        }
        let mut digest = Sha256::new();
        digest.update(header);
        Ok(Self {
            stream,
            digest,
            ordinal: 0,
        })
    }

    fn row(&mut self) -> Result<CountRow, Failure> {
        let mut bytes = [0; COUNT_ROW];
        self.stream.read_exact(&mut bytes)?;
        self.digest.update(bytes);
        let mut counts = [0; WIDTH];
        let mut totals = [0; WIDTH];
        for order in 0..WIDTH {
            counts[order] = u32::from_le_bytes(bytes[4 * order..4 * (order + 1)].try_into()?);
            totals[order] = word(&bytes[44 + 8 * order..44 + 8 * (order + 1)])?;
            if u64::from(counts[order]) > totals[order] {
                return Err(fail(format!(
                    "count exceeds total at R ordinal {}, order {order}",
                    self.ordinal
                )));
            }
        }
        let mass = word(&bytes[132..])?;
        if totals[0] == 0 || !(1..=(1 << 48)).contains(&mass) {
            return Err(fail(format!(
                "invalid KN row at R ordinal {}",
                self.ordinal
            )));
        }
        self.ordinal += 1;
        Ok((mass, counts, totals))
    }

    fn finish(mut self, expected: &str) -> Result<(), Failure> {
        if self.ordinal != REAL_ROWS
            || self.stream.read(&mut [0])? != 0
            || format!("{:x}", self.digest.finalize()) != expected
        {
            return Err(fail("consumed count tape length or SHA-256 differs"));
        }
        Ok(())
    }
}

fn keep_ordinals(ids: &[u16]) -> Vec<usize> {
    let mut kept = Vec::new();
    let (mut frame, mut position, mut ordinal) = (0, 0, 0usize);
    for &id in ids {
        if id == 0 {
            frame += 1;
            position = 0;
            continue;
        }
        position += 1;
        let seam = [READ_BATCH, CAPTURE_BATCH]
            .iter()
            .any(|&batch| (1..=6).any(|k| ordinal.abs_diff(k * batch) <= 1));
        if (frame <= 64 && (position <= 8 || position == CONTEXT || id == 1))
            || seam
            || ordinal >= REAL_ROWS - 2
        {
            kept.push(ordinal);
        }
        ordinal += 1;
    }
    kept
}

pub fn run(
    mode: &str,
    r: &Path,
    trunk: &Path,
    compact: &Path,
    tape: &Path,
    tape_sha: &str,
    out: &Path,
    exact_input: Option<(&Path, &str)>,
) -> Result<(), Failure> {
    if !["identity", "permuted", "exact", "disabled"].contains(&mode) {
        return Err(fail("unknown Pick preparation mode"));
    }
    let start = Instant::now();
    let r = load_r(r)?;
    pin(tape_sha)?;
    if file_digest(tape)? != tape_sha {
        return Err(fail(
            "feature tape differs from the admitted capture SHA-256",
        ));
    }
    if (mode == "exact") != exact_input.is_some() {
        return Err(fail(
            "C2 requires the GPU selector tape and its independently pinned SHA-256",
        ));
    }
    let exact_rows = exact_input
        .map(|(file, sha)| crate::exact::read(file, REAL_ROWS, sha))
        .transpose()?;
    let mut count_tape = CountTape::open(tape)?;
    if fs::metadata(compact)?.len() != COMPACT_BYTES {
        return Err(fail("wrong deployed compact byte length"));
    }
    let compact = fs::read(compact)?;
    let lookup = CompactCountLookup::from_bytes(&compact, pin(C_SHA)?, pin(COMPACT_SHA)?)?;
    let model = load_trunk(trunk)?;
    let windows = windows(&r, CONTEXT)?;
    if windows.len() != REAL_ROWS {
        return Err(fail("wrong causal Pick window count"));
    }
    let kept = keep_ordinals(&r);
    let keep = |i| kept.binary_search(&i).is_ok();
    fs::create_dir(out)?;
    let (pick_rows, kept_words, perm_codes) = match mode {
        "identity" | "permuted" => {
            let perm = (mode == "permuted").then(|| row_permutation(4096, C3_SEED));
            let dev = PickDevice::new(&model, perm.as_deref())?;
            let actual = dev.rows(&r, CONTEXT, &keep)?;
            let perm_codes = if perm.is_some() {
                Some(permute_code_rows(
                    &root_codes(&model.identity_pick_inventory()?.0)?,
                    C3_SEED,
                ))
            } else {
                None
            };
            if actual.rows.len() != REAL_ROWS
                || actual.words.len() != kept.len()
                || actual.words.iter().zip(&kept).any(|(w, &i)| w.ordinal != i)
            {
                return Err(fail("wrong CUDA Pick cardinality or kept ordinals"));
            }
            (Some(actual.rows), actual.words, perm_codes)
        }
        _ => (None, Vec::new(), None),
    };
    eprintln!(
        "causal {mode} Pick complete at {} ms",
        start.elapsed().as_millis()
    );

    let mut arm_rows = ARMS
        .map(|(arm, _)| out.join(format!("{arm}.rows")))
        .iter()
        .map(|path| writer(path))
        .collect::<Result<Vec<_>, _>>()?;
    for ((_, rule), rows) in ARMS.into_iter().zip(&mut arm_rows) {
        rows.write_all(&header(b"R37EM001", REAL_ROWS, rule))?;
    }
    let pick_path = out.join("pick.rows");
    let mut pick_tape = writer(&pick_path)?;
    pick_tape.write_all(b"R37PK001")?;
    pick_tape.write_all(&(REAL_ROWS as u64).to_le_bytes())?;
    pick_tape.write_all(&pin(R_SHA)?)?;
    pick_tape.write_all(&pin(TRUNK_SHA)?)?;
    let mut probes = Vec::with_capacity(kept.len());
    let mut class_counts = [[0u64; 5]; 2];
    let mut hit_counts = [[0u64; 5]; 2];
    for (chunk_index, chunk) in windows.chunks(CHUNK).enumerate() {
        let mut kn = Vec::with_capacity(chunk.len());
        let mut counts = Vec::with_capacity(chunk.len() * WIDTH);
        let mut totals = Vec::with_capacity(chunk.len() * WIDTH);
        let mut picks: [Vec<PickTarget>; 2] =
            std::array::from_fn(|_| Vec::with_capacity(chunk.len()));
        for (i, &(at, prefix)) in chunk.iter().enumerate() {
            let ordinal = chunk_index * CHUNK + i;
            let (mass, raw, total) = count_tape.row()?;
            let target = r[at];
            let (predecessor, distance, device_hit) = match &pick_rows {
                Some(rows) => {
                    let PickRow {
                        pick,
                        distance,
                        hit,
                        ..
                    } = rows[ordinal];
                    (pick.map(|p| p as usize), distance, Some(hit))
                }
                None if exact_rows.is_some() => {
                    let p = exact_rows.as_ref().ok_or_else(|| fail("missing C2 rows"))?[ordinal];
                    let pick = (p != u32::MAX).then_some(p as usize);
                    (pick, pick.map(|_| 0), None)
                }
                None => (None, None, None),
            };
            let successor = predecessor
                .map(|p| {
                    prefix.get(p + 1).copied().filter(|_| p > 0).ok_or_else(|| {
                        fail(format!("invalid Pick predecessor at R ordinal {ordinal}"))
                    })
                })
                .transpose()?;
            let hit = successor == Some(target);
            if predecessor.is_some() != distance.is_some() || device_hit.is_some_and(|v| v != hit) {
                return Err(fail(format!(
                    "inconsistent causal Pick at R ordinal {ordinal}"
                )));
            }
            for value in [
                predecessor.map(|p| p as u32).unwrap_or(u32::MAX),
                distance.unwrap_or(u32::MAX),
                successor.map(u32::from).unwrap_or(u32::MAX),
                u32::from(hit),
            ] {
                pick_tape.write_all(&value.to_le_bytes())?;
            }
            for (index, (_, rule)) in ARMS.into_iter().enumerate() {
                let class = rule.classify(distance);
                class_counts[index][class as usize] += 1;
                hit_counts[index][class as usize] += u64::from(hit);
                picks[index].push(PickTarget { class, hit });
            }
            if kept.get(probes.len()) == Some(&ordinal) {
                let (mut a, mut b) = ([0; WIDTH], [0; WIDTH]);
                let native = lookup.lookup(&r, at, &mut a, &mut b)?;
                if (native, a, b) != (mass, raw, total) {
                    return Err(fail(format!(
                        "native compact row differs at R ordinal {ordinal}"
                    )));
                }
                if mode == "exact" && exact_pair_pick(prefix) != predecessor {
                    return Err(fail(format!(
                        "native exact Pick differs at R ordinal {ordinal}"
                    )));
                }
                let mut blocks = None;
                if mode == "identity" {
                    let ids: Vec<u32> = prefix.iter().map(|&id| u32::from(id)).collect();
                    let native = model.observe_identity_pick(&ids)?;
                    let seen = kept_words[probes.len()].blocks;
                    if native.block_words.as_slice() != seen.as_slice()
                        || native.successor != successor.map(u32::from)
                    {
                        return Err(fail(format!(
                            "native Pick trace differs at R ordinal {ordinal}"
                        )));
                    }
                    struct_boundary_check(prefix, &seen, distance)?;
                    blocks = Some(seen);
                } else if let Some(codes) = &perm_codes {
                    let seen = kept_words[probes.len()].blocks;
                    let want = code_pick(codes, prefix);
                    let got = predecessor.zip(distance).map(|(p, d)| (p, d, seen[0][2]));
                    if want != got {
                        return Err(fail(format!(
                            "native permuted Pick differs at R ordinal {ordinal}"
                        )));
                    }
                    blocks = Some(seen);
                }
                probes.push(json!({
                    "ordinal": ordinal, "position": at, "prefix": prefix, "target": target,
                    "distance": distance, "hit": hit, "predecessor": predecessor,
                    "successor": successor, "blocks": blocks, "kn_q48": mass,
                    "counts": raw, "totals": total
                }));
            }
            kn.push(mass);
            counts.extend(raw);
            totals.extend(total);
        }
        for (index, (_, rule)) in ARMS.into_iter().enumerate() {
            let table = CeilingTable::from_lookups_with_pick_classes(
                N,
                &kn,
                &counts,
                &totals,
                &picks[index],
                rule.classes(),
            )?;
            write_rows(&mut arm_rows[index], &table)?;
        }
    }
    count_tape.finish(tape_sha)?;
    if probes.len() != kept.len()
        || probes.len() < 64
        || class_counts
            .iter()
            .any(|counts| counts.iter().sum::<u64>() != REAL_ROWS as u64)
    {
        return Err(fail(
            "joined tape omitted a registered target or replay probe",
        ));
    }
    for stream in arm_rows {
        finish(stream)?;
    }
    finish(pick_tape)?;
    let probes = serde_json::to_vec_pretty(&probes)?;
    write_new(&out.join("probes.json"), &probes)?;
    let ordinals: Vec<_> = kept
        .iter()
        .flat_map(|&i| (i as u64).to_le_bytes())
        .collect();
    write_new(&out.join("probe-ordinals.u64"), &ordinals)?;
    let mut arms = serde_json::Map::new();
    for (index, (arm, rule)) in ARMS.into_iter().enumerate() {
        arms.insert(
            arm.to_owned(),
            json!({
                "rows_sha256": file_digest(&out.join(format!("{arm}.rows")))?,
                "classes": rule.classes(), "class_counts": class_counts[index],
                "hit_counts": hit_counts[index]
            }),
        );
    }
    let receipt = json!({
        "format": PREPARED, "rows": REAL_ROWS, "records": RECORDS,
        "c_sha256": C_SHA, "r_sha256": R_SHA, "trunk_sha256": TRUNK_SHA,
        "compact_sha256": COMPACT_SHA, "count_tape_sha256": tape_sha,
        "pick_mode": mode, "exact_tape_sha256": exact_input.map(|(_, sha)| sha),
        "exact_device_source_sha256": exact_input.map(|_| digest(include_bytes!("pick_exact.cu"))),
        "permutation_seed": (mode == "permuted").then_some(C3_SEED),
        "pick_tape_sha256": file_digest(&pick_path)?, "arms": arms,
        "probes_sha256": digest(&probes), "probe_ordinals_sha256": digest(&ordinals),
        "cuda_pick_batch": READ_BATCH, "count_capture_batch": CAPTURE_BATCH,
        "host_source_sha256": digest(include_bytes!("main.rs")),
        "prep_source_sha256": digest(include_bytes!("prepare.rs")),
        "device_source_sha256": digest(include_bytes!("device.cu")),
        "elapsed_ms": start.elapsed().as_millis(),
    });
    write_new(
        &out.join("prepare.json"),
        &serde_json::to_vec_pretty(&receipt)?,
    )?;
    Ok(())
}

// A moved distance crossing a rule boundary must fail the trace comparison.
fn struct_boundary_check(
    prefix: &[u16],
    blocks: &[[u32; 5]; 4],
    distance: Option<u32>,
) -> Result<(), Failure> {
    if let Some(d) = distance {
        if prefix.len() < 3 || blocks[0][1] != d {
            return Err(fail("Pick distance differs from native trace"));
        }
        let changed = if d < 3 { 3 } else { 0 };
        if PickClassRule::Five.classify(Some(changed)) == PickClassRule::Five.classify(distance) {
            return Err(fail(
                "planted class-boundary change did not alter the five-class bucket",
            ));
        }
    }
    Ok(())
}
