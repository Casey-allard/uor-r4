//! Pair production CUDA count partitions in bounded host memory and write their native words.
use super::*;
use std::io::{Seek, SeekFrom};
use uor_r4_discrete::{
    kn_ceiling::RawTables,
    kn_filter::KnQuery,
    kn_score::KnTables,
    ngram_count::{
        partition_from_triples, NgramAssembler, NgramPartition, PrimeRouteKey, UnigramContinuation,
    },
};
#[cfg(feature = "cuda")]
use uor_r4_discrete::{
    kn_lookup_device::{compiled_kn_lookup_image, KnLookup},
    kn_score_device::{compiled_kn_score_image, KnScorer},
    ngram_count_device::{compiled_ngram_count_image, NgramCounter},
};

const HEADER: u64 = 44;
const ORDER_HEADER: u64 = 72;
const BUDGET: u64 = 1 << 20;
#[cfg(feature = "cuda")]
const MAX_PARTITION: u64 = 1 << 22;

struct OrderLayout {
    order: u32,
    header: u64,
    context_start: u64,
    follower_start: u64,
    end: u64,
    contexts: u64,
    followers: u32,
}

impl OrderLayout {
    fn new(start: u64, expected: &[NgramOrderDigest; 2]) -> Result<Self, Fail> {
        let [raw, _] = expected;
        let order = raw.order;
        let contexts = raw.contexts;
        let followers = raw.grams.try_into()?;
        let context_start = start + ORDER_HEADER;
        let follower_start = context_start + contexts * (key_width(order) + 4) as u64;
        let end = follower_start + u64::from(followers) * follower_width(order);
        Ok(Self {
            order,
            header: start,
            context_start,
            follower_start,
            end,
            contexts,
            followers,
        })
    }
}

fn key_width(order: u32) -> usize {
    (order as usize * 12).div_ceil(8)
}
fn follower_width(order: u32) -> u64 {
    if order == N {
        6
    } else {
        10
    }
}

struct OrderWriter<'a> {
    layout: &'a OrderLayout,
    keys: BufWriter<&'a File>,
    rows: BufWriter<&'a File>,
    contexts: u64,
    followers: u32,
}

impl<'a> OrderWriter<'a> {
    fn new(
        file: &'a File,
        row_file: &'a File,
        layout: &'a OrderLayout,
        spectra: [[u64; 4]; 2],
    ) -> Result<Self, Fail> {
        let mut keys = BufWriter::with_capacity(1 << 20, file);
        keys.seek(SeekFrom::Start(layout.header))?;
        keys.write_all(&layout.followers.to_le_bytes())?;
        keys.write_all(&u32::try_from(layout.contexts)?.to_le_bytes())?;
        for word in spectra.into_iter().flatten() {
            keys.write_all(&word.to_le_bytes())?;
        }
        let mut rows = BufWriter::with_capacity(1 << 20, row_file);
        rows.seek(SeekFrom::Start(layout.follower_start))?;
        Ok(Self {
            layout,
            keys,
            rows,
            contexts: 0,
            followers: 0,
        })
    }

    fn append(&mut self, raw: &NgramPartition, adjusted: &NgramPartition) -> Result<(), Fail> {
        let order = self.layout.order;
        let shared = order == N;
        if raw.contexts.len() != adjusted.contexts.len()
            || raw.triples.len() != adjusted.triples.len()
        {
            return Err("raw and adjusted partition cardinalities differ".into());
        }
        let end_context = self.contexts + raw.contexts.len() as u64;
        let end_follower = self
            .followers
            .checked_add(u32::try_from(raw.triples.len())?)
            .ok_or("too many followers")?;
        if end_context > self.layout.contexts || end_follower > self.layout.followers {
            return Err("GPU partition exceeds registered artifact section".into());
        }
        for (a, b) in raw.contexts.iter().zip(&adjusted.contexts) {
            if (a.key, a.followers) != (b.key, b.followers)
                || PrimeRouteKey::from_word(a.key)?.order() != order
            {
                return Err("raw/adjusted context support or key order differs".into());
            }
            let body = ((a.key & ((1u128 << 120) - 1)) >> (120 - order * 12)).to_be_bytes();
            self.keys.write_all(&body[16 - key_width(order)..])?;
            self.keys.write_all(&self.followers.to_le_bytes())?;
            self.followers = self
                .followers
                .checked_add(a.followers)
                .ok_or("too many context followers")?;
        }
        if self.followers != end_follower {
            return Err("summed context followers differ from partition triples".into());
        }
        for (a, b) in raw.triples.iter().zip(&adjusted.triples) {
            if (a.key, a.next) != (b.key, b.next) || (shared && a.count != b.count) {
                return Err("raw/adjusted follower support or shared count differs".into());
            }
            self.rows.write_all(&a.next.to_le_bytes())?;
            self.rows.write_all(&a.count.to_le_bytes())?;
            if !shared {
                self.rows.write_all(&b.count.to_le_bytes())?;
            }
        }
        self.contexts = end_context;
        Ok(())
    }

    fn finish(mut self) -> Result<(), Fail> {
        self.keys.flush()?;
        self.rows.flush()?;
        if self.contexts != self.layout.contexts
            || self.followers != self.layout.followers
            || self.keys.stream_position()? != self.layout.follower_start
            || self.rows.stream_position()? != self.layout.end
            || self.layout.context_start != self.layout.header + ORDER_HEADER
        {
            return Err("written order partition lengths differ from registered bounds".into());
        }
        Ok(())
    }
}

/// Write the complete canonical count source for the streaming pruner. The
/// source only becomes usable after the independently pinned census passes.
#[cfg(feature = "cuda")]
pub fn spool(
    dose: Dose,
    training: production::Training,
    expected: &Census,
    plan: production::Plan,
    output: &Path,
) -> Result<Value, Fail> {
    use uor_r4_discrete::device::Device;

    let started = std::time::Instant::now();
    let gpu = Device::memory_info(0)?;
    let meminfo = fs::read_to_string("/proc/meminfo")?;
    let available_kib: u64 = meminfo
        .lines()
        .find_map(|line| line.strip_prefix("MemAvailable:"))
        .and_then(|value| value.split_whitespace().next())
        .ok_or("cannot measure host MemAvailable")?
        .parse()?;
    // In addition to the counted arrays, CUDA needs allocator/driver space.
    let gpu_required = plan.gpu_bytes + (512 << 20);
    let host_required = plan.host_bytes + (512 << 20);
    let available_host = available_kib * 1024 + training.tokens.capacity() as u64 * 2;
    if (gpu.free_bytes as u64) < gpu_required || available_host < host_required {
        return Err(format!(
            "insufficient count headroom: GPU {} free, {} required; host {} free, {} required",
            gpu.free_bytes, gpu_required, available_host, host_required,
        )
        .into());
    }

    let mut pos = HEADER;
    let mut layouts = Vec::with_capacity(WIDTH);
    for census in expected {
        let layout = OrderLayout::new(pos, census)?;
        pos = layout.end;
        layouts.push(layout);
    }
    if pos != dose.bytes {
        return Err("independent census does not imply the registered count-file size".into());
    }
    let production::Training {
        tokens,
        sha: training_sha,
        ..
    } = training;
    let counter = NgramCounter::new(0, &tokens)?;
    drop(tokens);
    fs::create_dir(output)?;
    let partial = output.join("compact.counts.partial");
    let mut file = File::options()
        .create_new(true)
        .write(true)
        .open(&partial)?;
    let row_file = File::options().write(true).open(&partial)?;
    file.set_len(pos)?;
    file.write_all(b"UORKNC02")?;
    file.write_all(&N.to_le_bytes())?;
    file.write_all(&training_sha)?;
    let mut continuation = UnigramContinuation::new();
    let mut produced = Vec::with_capacity(WIDTH);
    let mut max_targets = 0;
    let mut range_counts = Vec::with_capacity(WIDTH);
    for order in (0..=N).rev() {
        let order_started = std::time::Instant::now();
        let census = &expected[order as usize];
        let spectra = [census[0].count_of_counts, census[1].count_of_counts];
        let mut writer = OrderWriter::new(&file, &row_file, &layouts[order as usize], spectra)?;
        let mut raw_assembler = NgramAssembler::new(order, false)?;
        let mut adjusted_assembler = NgramAssembler::new_adjusted(order, false)?;
        let mut consume = |raw: NgramPartition, adjusted: NgramPartition| -> Result<(), Fail> {
            writer.append(&raw, &adjusted)?;
            raw_assembler.push_owned(raw)?;
            adjusted_assembler.push_owned(adjusted)?;
            raw_assembler.drain_partitions()?;
            adjusted_assembler.drain_partitions()?;
            Ok(())
        };

        let histogram = counter.partition_histogram(order)?;
        let ranges = &plan.base_ranges[usize::from(order != 0)];
        let (mut targets, mut nonempty_ranges) = (0, 0);
        let mut unigrams = if order == 0 {
            Some(NgramAssembler::new(0, true)?)
        } else {
            None
        };
        for upper in ranges {
            let actual = histogram[upper.lo as usize..upper.hi as usize]
                .iter()
                .sum::<u64>();
            if actual > upper.targets || (order <= 1 && actual != upper.targets) {
                return Err("GPU range target count differs from the TRAIN plan bound".into());
            }
            targets += actual;
            nonempty_ranges += usize::from(actual != 0);
            max_targets = max_targets.max(actual);
            let range = PartitionRange {
                targets: actual,
                ..*upper
            };
            let raw = counter.count_partition(order, &range)?;
            if let Some(unigrams) = unigrams.as_mut() {
                unigrams.push_owned(raw)?;
                unigrams.drain_partitions()?;
                continue;
            }
            if order == 1 {
                continuation.add(&raw)?;
            }
            let adjusted = if order == N {
                raw.clone()
            } else {
                counter.count_partition_adjusted(order, N, &range)?
            };
            consume(raw, adjusted)?;
        }
        if targets != census[0].targets {
            return Err("GPU histogram does not cover all independent census targets".into());
        }
        if let Some(unigrams) = unigrams {
            let (raw, rows, _) = unigrams.finish();
            if raw != census[0] {
                return Err("GPU unigram count differs from independent census".into());
            }
            let adjusted = continuation.adjust(N, &rows)?;
            consume(partition_from_triples(rows), adjusted)?;
        }
        writer.finish()?;
        let (raw, _, _) = raw_assembler.finish();
        let (adjusted, _, _) = adjusted_assembler.finish();
        let observed = [raw, adjusted];
        verify_digests(
            std::slice::from_ref(&observed),
            std::slice::from_ref(census),
            dose.archive_aliases_raw_top_hashes,
        )?;
        produced.push(observed);
        range_counts.push([ranges.len(), nonempty_ranges]);
        eprintln!(
            "order {order}: identical pinned census in {}s",
            order_started.elapsed().as_secs()
        );
    }
    drop(counter);
    file.sync_all()?;
    drop(file);
    drop(row_file);

    let mut source = BufReader::with_capacity(1 << 20, File::open(&partial)?);
    let mut hash = Sha256::new();
    if std::io::copy(&mut source, &mut hash)? != dose.bytes {
        return Err("written count spool has the wrong physical length".into());
    }
    let pin = format!("{:x}", hash.finalize());
    if dose.known_artifact.is_some_and(|known| pin != known) {
        return Err("C128 spool differs from the frozen count artifact".into());
    }
    produced.reverse();
    range_counts.reverse();
    let receipt = json!({
        "plan": plan.receipt,
        "artifact_sha256": pin, "artifact_bytes": dose.bytes,
        "max_partition_targets": max_targets, "planned_and_nonempty_partitions_by_order": range_counts,
        "gpu_free_bytes_before": gpu.free_bytes, "gpu_required_bytes": gpu_required,
        "host_available_bytes_including_train_buffer": available_host,
        "host_required_bytes": host_required,
        "gpu_count_image": compiled_ngram_count_image(), "wall_ms": started.elapsed().as_millis(),
        "raw": produced.iter().map(|pair| digest_json(&pair[0])).collect::<Vec<_>>(),
        "adjusted": produced.iter().map(|pair| digest_json(&pair[1])).collect::<Vec<_>>(),
    });
    let mut writer = new_writer(&output.join("receipt.json.partial"))?;
    serde_json::to_writer_pretty(&mut writer, &receipt)?;
    seal(writer)?;
    fs::rename(partial, output.join("compact.counts"))?;
    fs::rename(
        output.join("receipt.json.partial"),
        output.join("receipt.json"),
    )?;
    File::open(output)?.sync_all()?;
    Ok(receipt)
}

#[cfg(feature = "cuda")]
pub fn count(
    dose: Dose,
    tokens: &[u16],
    training: [u8; 32],
    expected: Census,
    output: &Path,
) -> Result<Value, Fail> {
    let started = std::time::Instant::now();
    let mut pos = HEADER;
    let mut layouts = Vec::with_capacity(WIDTH);
    for census in &expected {
        let layout = OrderLayout::new(pos, census)?;
        pos = layout.end;
        layouts.push(layout);
    }
    if pos != dose.bytes {
        return Err("registered cardinalities do not imply the registered file size".into());
    }
    let artifact = output.join("compact.counts");
    let mut file = File::options()
        .write(true)
        .create_new(true)
        .open(&artifact)?;
    let row_file = File::options().write(true).open(&artifact)?;
    file.set_len(pos)?;
    file.write_all(b"UORKNC02")?;
    file.write_all(&N.to_le_bytes())?;
    file.write_all(&training)?;
    let counter = NgramCounter::new(0, tokens)?;
    let (probe_tokens, _) = probe_query(dose, tokens);
    let query = KnQuery::from_stream(&probe_tokens, N)?;
    let mut continuation = UnigramContinuation::new();
    let mut raw_tables = Vec::with_capacity(WIDTH);
    let mut adjusted_tables = Vec::with_capacity(WIDTH);
    let mut produced = Vec::with_capacity(WIDTH);
    let mut max_partition = 0u64;
    for order in (0..=N).rev() {
        let order_started = std::time::Instant::now();
        let census = &expected[order as usize];
        let spectra = [census[0].count_of_counts, census[1].count_of_counts];
        let mut writer = OrderWriter::new(&file, &row_file, &layouts[order as usize], spectra)?;
        let mut raw_filter = query.filter(order)?;
        let mut adjusted_filter = query.filter(order)?;
        let mut raw_assembler = NgramAssembler::new(order, false)?;
        let mut adjusted_assembler = NgramAssembler::new_adjusted(order, false)?;
        let mut consume = |raw: NgramPartition, adjusted: NgramPartition| -> Result<(), Fail> {
            writer.append(&raw, &adjusted)?;
            raw_filter.keep(&raw)?;
            adjusted_filter.keep(&adjusted)?;
            raw_assembler.push_owned(raw)?;
            adjusted_assembler.push_owned(adjusted)?;
            Ok(())
        };
        let plan = counter.plan(order, BUDGET)?;
        max_partition = max_partition.max(plan.iter().map(|p| p.targets).max().unwrap_or(0));
        if max_partition > MAX_PARTITION {
            return Err("partition exceeds the registered host and device resource bound".into());
        }
        if order == 0 {
            let mut raw = NgramAssembler::new(0, true)?;
            for range in &plan {
                raw.push_owned(counter.count_partition(0, range)?)?;
            }
            let (digest, unigrams, _) = raw.finish();
            if digest != census[0] {
                return Err("GPU raw unigram count differs from independent census".into());
            }
            let adjusted = continuation.adjust(N, &unigrams)?;
            consume(partition_from_triples(unigrams), adjusted)?;
        } else {
            for range in &plan {
                let raw = counter.count_partition(order, range)?;
                if order == 1 {
                    continuation.add(&raw)?;
                }
                let adjusted = if order == N {
                    raw.clone()
                } else {
                    counter.count_partition_adjusted(order, N, range)?
                };
                consume(raw, adjusted)?;
            }
        }
        writer.finish()?;
        let (raw, _, _) = raw_assembler.finish();
        let (adjusted, _, _) = adjusted_assembler.finish();
        let observed = [raw, adjusted];
        verify_digests(
            std::slice::from_ref(&observed),
            std::slice::from_ref(census),
            dose.archive_aliases_raw_top_hashes,
        )?;
        produced.push(observed.clone());
        let [raw, adjusted] = observed;
        let (raw_rows, raw_contexts) = raw_filter.finish();
        let (adjusted_rows, adjusted_contexts) = adjusted_filter.finish();
        raw_tables.push((raw, raw_rows, raw_contexts));
        adjusted_tables.push((adjusted, adjusted_rows, adjusted_contexts));
        eprintln!(
            "order {order}: exact independent census in {}s",
            order_started.elapsed().as_secs()
        );
    }
    drop(counter);
    file.sync_all()?;
    drop(file);
    drop(row_file);
    let materialized_ms = started.elapsed().as_millis();
    raw_tables.reverse();
    adjusted_tables.reverse();
    let raw_tables = RawTables::new(N, raw_tables)?;
    let adjusted_tables = KnTables::new(N, adjusted_tables)?;
    let (device_counts, device_totals) = KnLookup::new(0, &raw_tables)?.lookups(&probe_tokens)?;
    let device_masses = KnScorer::new(0, &adjusted_tables)?.score(&probe_tokens)?;
    let mut index_by_position = Vec::with_capacity(probe_tokens.len());
    let mut scored = 0;
    for id in probe_tokens {
        index_by_position.push(scored);
        scored += usize::from(id != 0);
    }
    if device_counts.len() != scored * WIDTH
        || device_totals.len() != scored * WIDTH
        || device_masses.len() != scored
    {
        return Err("GPU lookup returned the wrong native word shape".into());
    }
    let bytes = fs::read(&artifact)?;
    if bytes.len() as u64 != dose.bytes {
        return Err("materialization changed the registered artifact length".into());
    }
    let sha = Sha256::digest(&bytes);
    let pin = format!("{sha:x}");
    if dose.known_artifact.is_some_and(|known| pin != known) {
        return Err("C128 artifact differs from D30".into());
    }
    let lookup = CompactCountLookup::from_bytes(&bytes, training, sha.into())?;
    let decoded = lookup.canonical_digests()?;
    produced.reverse();
    verify_digests(&decoded, &produced, false)?;
    verify_digests(&decoded, &expected, dose.archive_aliases_raw_top_hashes)?;
    let replay = replay(
        &lookup,
        dose,
        tokens,
        &output.join("lookup.jsonl"),
        |original, at, probe, counts, totals, mass| {
            let (mut raw_counts, mut raw_totals) = ([0; WIDTH], [0; WIDTH]);
            raw_tables.lookup_target(original, at, &mut raw_counts, &mut raw_totals)?;
            let kn = adjusted_tables.score_target(original, at)?;
            if (counts, totals, mass) != (raw_counts.as_slice(), raw_totals.as_slice(), kn) {
                return Err(format!("native lookup differs from the original GPU records at probe-stream position {}", probe.target_at).into());
            }
            // The concatenated GPU probe tape contains a BOS delimiter. A
            // synthetic position-zero target has no context on its own.
            if at > 0 {
                let position = index_by_position[probe.target_at];
                let words = position * WIDTH..(position + 1) * WIDTH;
                if (counts, totals, mass)
                    != (
                        &device_counts[words.clone()],
                        &device_totals[words],
                        device_masses[position],
                    )
                {
                    return Err(format!(
                        "native lookup differs from CUDA lookup at probe-stream position {}",
                        probe.target_at
                    )
                    .into());
                }
            }
            Ok(())
        },
    )?;
    Ok(
        json!({"artifact_sha256": pin, "artifact_bytes": bytes.len(),
        "gpu_count_image": compiled_ngram_count_image(), "gpu_lookup_image": compiled_kn_lookup_image(),
        "gpu_kn_score_image": compiled_kn_score_image(), "partition_target_budget": BUDGET,
        "original_train_cuda_lookup_targets": replay["original_train_targets"],
        "max_partition_targets": max_partition, "materialization_wall_ms": materialized_ms,
        "materialization_with_readback_wall_ms": started.elapsed().as_millis(),
        "raw": decoded.iter().map(|pair| digest_json(&pair[0])).collect::<Vec<_>>(),
        "adjusted": decoded.iter().map(|pair| digest_json(&pair[1])).collect::<Vec<_>>(), "replay": replay}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_discrete::kn_ceiling::encode_compact_lookup;
    use uor_r4_discrete::ngram_count::NgramTriple;

    #[test]
    fn streamed_file_matches_original_writer_and_roundtrips_full_census() -> Result<(), Fail> {
        let scratch =
            std::env::temp_dir().join(format!("uor-full-compact-count-{}", std::process::id()));
        let mut file = File::options()
            .write(true)
            .create_new(true)
            .open(&scratch)?;
        let rows = File::options().write(true).open(&scratch)?;
        let result = (|| -> Result<(), Fail> {
            let training = [7u8; 32];
            file.write_all(b"UORKNC02")?;
            file.write_all(&N.to_le_bytes())?;
            file.write_all(&training)?;
            let mut raw = Vec::new();
            let mut adjusted = Vec::new();
            let mut expected = Vec::new();
            let mut cursor = HEADER;
            for k in 0..=N {
                let mut raw_assembler = NgramAssembler::new(k, true)?;
                let mut adjusted_assembler = NgramAssembler::new_adjusted(k, true)?;
                let make_partition = |token, count| -> Result<NgramPartition, Fail> {
                    let key = PrimeRouteKey::from_context(&vec![token; k as usize])?.word();
                    Ok(partition_from_triples(vec![NgramTriple {
                        key,
                        next: token,
                        count,
                    }]))
                };
                let pairs = if k == 0 {
                    let a = make_partition(1, 17)?;
                    let b = make_partition(2, 3)?;
                    let raw = partition_from_triples([a.triples, b.triples].concat());
                    let a = make_partition(1, 4)?;
                    let b = make_partition(2, 2)?;
                    let adjusted = partition_from_triples([a.triples, b.triples].concat());
                    vec![(raw, adjusted)]
                } else {
                    vec![
                        (
                            make_partition(5, 17)?,
                            make_partition(5, if k == N { 17 } else { 4 })?,
                        ),
                        (
                            make_partition(3073, 3)?,
                            make_partition(3073, if k == N { 3 } else { 2 })?,
                        ),
                    ]
                };
                for (a, b) in &pairs {
                    raw_assembler.push(a)?;
                    adjusted_assembler.push(b)?;
                    raw_assembler.drain_partitions()?;
                    adjusted_assembler.drain_partitions()?;
                }
                let a = raw_assembler.finish();
                let b = adjusted_assembler.finish();
                let census = [a.0.clone(), b.0.clone()];
                let layout = OrderLayout::new(cursor, &census)?;
                let spectra = [census[0].count_of_counts, census[1].count_of_counts];
                let mut writer = OrderWriter::new(&file, &rows, &layout, spectra)?;
                for (a, b) in &pairs {
                    writer.append(a, b)?;
                }
                writer.finish()?;
                cursor = layout.end;
                raw.push(a);
                adjusted.push(b);
                expected.push(census);
            }
            file.sync_all()?;
            let bytes = fs::read(&scratch)?;
            assert_eq!(bytes, encode_compact_lookup(N, training, &raw, &adjusted)?);
            let sha = Sha256::digest(&bytes).into();
            let lookup = CompactCountLookup::from_bytes(&bytes, training, sha)?;
            let decoded = lookup.canonical_digests()?;
            verify_digests(&decoded, &expected, false)?;
            let mut alias = expected.clone();
            alias[N as usize][0] = alias[N as usize][1].clone();
            verify_digests(&decoded, &alias, true)?;
            assert!(verify_digests(&decoded, &alias, false).is_err());
            alias[N as usize][0].grams += 1;
            assert!(verify_digests(&decoded, &alias, true).is_err());
            let mut damage = bytes.clone();
            *damage.last_mut().ok_or("empty artifact")? ^= 8;
            assert!(CompactCountLookup::from_bytes(&damage, training, sha).is_err());
            Ok(())
        })();
        fs::remove_file(scratch)?;
        result
    }
}
