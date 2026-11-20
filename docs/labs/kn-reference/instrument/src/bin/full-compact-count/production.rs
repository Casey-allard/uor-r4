//! Frozen TRAIN ingestion and resource accounting for the full-count spool.
use super::*;

const TRAIN_CHUNK: usize = 1 << 20;
const FULL_PARTITION_BUDGET: u64 = 1 << 24;
const SMALL_PARTITION_BUDGET: u64 = 1 << 20;
// The pinned C128 finite control deliberately exercises many range seams.
const C128_PARTITION_BUDGET: u64 = 1 << 12;

pub struct Training {
    pub tokens: Vec<u16>,
    pub sha: [u8; 32],
    pub histograms: [Vec<u64>; 2],
}

pub fn read_training(dose: Dose, path: &Path, keep_tokens: bool) -> Result<Training, Fail> {
    let mut file = BufReader::new(File::open(path)?);
    let expected_bytes = 64 + 2 * dose.tokens as u64;
    if file.get_ref().metadata()?.len() != expected_bytes {
        return Err("wrong TRAIN file size".into());
    }
    let mut header = [0u8; 64];
    file.read_exact(&mut header)?;
    if &header[..4] != b"UORT"
        || u64::from_le_bytes(header[8..16].try_into()?) != dose.tokens as u64
    {
        return Err("wrong TRAIN UORT header".into());
    }
    let mut hash = Sha256::new();
    hash.update(header);
    let mut tokens = Vec::new();
    if keep_tokens {
        tokens.try_reserve_exact(dose.tokens)?;
    }
    let mut histograms = std::array::from_fn(|_| vec![0; NGRAM_VOCABULARY as usize]);
    let mut previous = 1u16;
    let mut records = 0;
    let mut bytes = vec![0u8; TRAIN_CHUNK];
    let mut remaining = dose.tokens;
    while remaining != 0 {
        let block_tokens = remaining.min(bytes.len() / 2);
        let block = &mut bytes[..block_tokens * 2];
        file.read_exact(block)?;
        hash.update(&*block);
        for pair in block.chunks_exact(2) {
            let token = u16::from_le_bytes([pair[0], pair[1]]);
            if u32::from(token) >= NGRAM_VOCABULARY || (token == 0) != (previous == 1) {
                return Err("invalid TRAIN record boundaries or token domain".into());
            }
            if token == 0 {
                records += 1;
            } else {
                histograms[0][token as usize] += 1;
                histograms[1][previous as usize] += 1;
            }
            previous = token;
            if keep_tokens {
                tokens.push(token);
            }
        }
        remaining -= block_tokens;
    }
    if file.read(&mut bytes[..1])? != 0 || records != dose.records || previous != 1 {
        return Err("wrong TRAIN token or record count or missing final EOS".into());
    }
    let sha = hash.finalize();
    if format!("{sha:x}") != dose.training {
        return Err("unregistered TRAIN input digest".into());
    }
    Ok(Training {
        tokens,
        sha: sha.into(),
        histograms,
    })
}

#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
pub struct Plan {
    pub base_ranges: [Vec<PartitionRange>; 2],
    pub gpu_bytes: u64,
    pub host_bytes: u64,
    pub receipt: Value,
}

/// Peak simultaneously live byte payloads from ngram_count_device.rs. All
/// buffers use a single CUDA stream, so a drop either enqueues a stream-ordered
/// free before the next allocation or synchronizes before freeing. Allocator
/// rounding, driver state, and loaded code stay under the separate reserve.
fn device_payload(stream_tokens: u64, targets: u64) -> (u64, u64, Value) {
    const SCAN_BLOCK_WORDS: u64 = 2048;
    const BUCKET_TILE: u64 = 1 << 20;
    const RADIX_TILE: u64 = 4096;
    const SELECT_WINDOW: u64 = 1 << 26;
    const HISTOGRAM_BLOCKS: u64 = 256;
    let scan = |words: u64, width: u64| width * (words.div_ceil(SCAN_BLOCK_WORDS).max(1) + 1);
    let slots = u64::from(NGRAM_VOCABULARY);
    let positions = stream_tokens.saturating_sub(1);
    let resident = stream_tokens * 2 + positions * 4;
    let bucket_words = slots * positions.div_ceil(BUCKET_TILE).max(1);
    let bucket_hist_bytes = 4 * (bucket_words + 1);
    let bucket_build = bucket_hist_bytes + scan(bucket_words, 4);
    let admitted_histogram = HISTOGRAM_BLOCKS * slots * 4 + slots * 8;

    // The order-zero selector allocates the entire flags window even when
    // the input is smaller. The lookback selector allocates at most that much.
    let selection = targets * 4 + 4 * (SELECT_WINDOW + 1) + scan(SELECT_WINDOW, 4);
    let radix_words = 64 * targets.div_ceil(RADIX_TILE);
    let sort = targets * 12 + 4 * (radix_words + 1) + scan(radix_words, 4);

    // A worst-case partition can contain one unique gram and context per
    // target. The adjusted path is the larger one. Before emitting grams,
    // sorted positions, context+left flags, gram flags+copy and gram starts
    // can coexist. Gram emission replaces the gram flags with result arrays.
    let boundaries = (20 * targets + 12 + scan(targets, 4)).max(24 * targets + 16);
    let gram_emit = 58 * targets + 48;
    // Context processing drops all per-target arrays. Its retained gram
    // payload is 50 bytes/gram including the adjusted count scan. The flag
    // copy is gone before spectrum scans and 16-byte context records.
    let context_prepare = (54 * targets + 36 + scan(targets, 4)).max(58 * targets + 40);
    let context_spectra = 54 * targets + 36 + scan(targets, 8);
    let context_emit = 70 * targets + 36;
    let phases = [
        ("bucket_build", bucket_build),
        ("admitted_histogram", admitted_histogram),
        ("selection", selection),
        ("radix_sort", sort),
        ("boundaries", boundaries),
        ("gram_emit", gram_emit),
        ("context_prepare", context_prepare),
        ("context_spectra", context_spectra),
        ("context_emit", context_emit),
    ];
    let max = phases.iter().map(|(_, bytes)| *bytes).max().unwrap_or(0);
    let phases = phases
        .into_iter()
        .map(|(name, extra)| (name.to_owned(), json!(resident + extra)))
        .collect::<serde_json::Map<_, _>>();
    (resident + max, bucket_hist_bytes, Value::Object(phases))
}

pub fn plan(dose: Dose, training: &Training, census: &Census) -> Result<Plan, Fail> {
    use uor_r4_discrete::ngram_count::{NgramContext, NgramTriple};

    let budget = match dose.name {
        "c" => FULL_PARTITION_BUDGET,
        "c128" => C128_PARTITION_BUDGET,
        _ => SMALL_PARTITION_BUDGET,
    };
    let base_ranges = [
        plan_partitions(&training.histograms[0], budget)?,
        plan_partitions(&training.histograms[1], budget)?,
    ];
    for (k, ranges) in base_ranges.iter().enumerate() {
        if ranges.iter().map(|range| range.targets).sum::<u64>() != census[k][0].targets {
            return Err("TRAIN histogram targets differ from independent census".into());
        }
    }
    let max_targets = base_ranges
        .iter()
        .flatten()
        .map(|p| p.targets)
        .max()
        .unwrap_or(0);
    let predecessor_max = base_ranges[1].iter().map(|p| p.targets).max().unwrap_or(0);

    let n = dose.tokens as u64;
    let (gpu_bytes, bucket_hist_bytes, gpu_phase_peaks) = device_payload(n, max_targets);

    // The spool drains both assemblers after each pair. Two typed partitions
    // and the CUDA download/record construction buffers can overlap. Bound
    // these by three copies of the largest predecessor range, with a separate
    // single-domain bound for the order-zero pass.
    let record_bytes =
        (std::mem::size_of::<NgramContext>() + std::mem::size_of::<NgramTriple>()) as u64;
    let host_work = 3 * predecessor_max * record_bytes;
    let host_bytes = (n * 2).max(host_work).max(bucket_hist_bytes) + (32 << 20);
    let receipt = json!({
        "dose": dose.name, "train_sha256": dose.training, "census_sha256": dose.census,
        "train_tokens": dose.tokens, "train_records": dose.records, "original_count_bytes": dose.bytes,
        "partition_target_budget": budget, "max_partition_targets_upper_bound": max_targets,
        "order0_ranges": base_ranges[0].len(), "predecessor_ranges": base_ranges[1].len(),
        "host_payload_bound_bytes": host_bytes, "gpu_payload_bound_bytes": gpu_bytes,
        "gpu_phase_payload_bounds_bytes": gpu_phase_peaks,
        "range_geometry": base_ranges.iter().map(|ranges| ranges.iter().map(|p| [u64::from(p.lo), u64::from(p.hi), p.targets]).collect::<Vec<_>>()).collect::<Vec<_>>()
    });
    Ok(Plan {
        base_ranges,
        gpu_bytes,
        host_bytes,
        receipt,
    })
}
