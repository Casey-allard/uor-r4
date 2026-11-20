//! claude-439: the `--pick-arms` mode. The query stream q is R, DEV-tune, R+ and DEV+;
//! SEGMENTS holds one byte per q position (0 R, 1 DEV-tune, 2 R+, 3 DEV+, 4 DEV+ second
//! copy). The main span is every position with segment at most 1; the plus span is the rest.
#![cfg_attr(not(feature = "cuda"), allow(dead_code))]

use crate::{fail, Failure};
use std::ops::Range;
use uor_r4_discrete::kn_ceiling::{PickTarget, RawTables};
use uor_r4_discrete::kn_pick::PickRow;
use uor_r4_discrete::ngram_count::NGRAM_BOS;

/// Most targets one GPU lookup call takes.
pub const LOOKUP_CHUNK: usize = 1 << 24;
/// The Pick read's window.
pub const CONTEXT: usize = 192;
/// The trunk's vocabulary.
pub const VOCAB: usize = 4096;
/// DEV-tune's records (claude-420 split v1); the bands resample exactly these.
pub const DEV_TUNE_RECORDS: usize = 2_670;

/// The bands' unit is DEV-tune's records: a segments file that cut or merged one fails.
pub fn check_band_records(records: usize) -> Result<(), Failure> {
    if records != DEV_TUNE_RECORDS {
        return Err(fail(format!(
            "the bands resample {records} records, not DEV-tune's {DEV_TUNE_RECORDS}"
        )));
    }
    Ok(())
}

/// Where the plus span starts and how many targets the main span holds.
#[derive(Debug, PartialEq, Eq)]
pub struct Spans {
    /// First q index of the plus span.
    pub split: usize,
    /// Non-BOS positions before `split`.
    pub main_targets: usize,
}

/// Splits q into the main and plus spans by SEGMENTS.
pub fn split_spans(query: &[u16], segments: &[u8]) -> Result<Spans, Failure> {
    if query.len() != segments.len() {
        return Err(fail(format!(
            "segments hold {} bytes for {} query positions",
            segments.len(),
            query.len()
        )));
    }
    if let Some(bad) = segments.iter().find(|&&s| s > 4) {
        return Err(fail(format!("segment {bad} is above 4")));
    }
    let split = segments
        .iter()
        .position(|&s| s >= 2)
        .ok_or_else(|| fail("no plus span: no segment 2, 3 or 4"))?;
    if split == 0 {
        return Err(fail("the main span is empty"));
    }
    if let Some(at) = segments[split..].iter().position(|&s| s <= 1) {
        return Err(fail(format!(
            "segment {} at q {} follows the plus span's start",
            segments[split + at],
            split + at
        )));
    }
    if query[split] != NGRAM_BOS {
        return Err(fail("the plus span must start with BOS"));
    }
    let main_targets = query[..split].iter().filter(|&&t| t != NGRAM_BOS).count();
    Ok(Spans {
        split,
        main_targets,
    })
}

/// Cuts `span` into consecutive ranges, each starting at a BOS and holding at most
/// `max_targets` non-BOS positions. Refuses a record with more targets than that.
pub fn chunk_bounds(span: &[u16], max_targets: usize) -> Result<Vec<Range<usize>>, Failure> {
    if span.first() != Some(&NGRAM_BOS) {
        return Err(fail("a lookup span must start with BOS"));
    }
    struct Record {
        start: usize,
        targets: usize,
    }
    let mut records: Vec<Record> = Vec::new();
    for (at, &token) in span.iter().enumerate() {
        if token == NGRAM_BOS {
            records.push(Record {
                start: at,
                targets: 0,
            });
        } else if let Some(last) = records.last_mut() {
            last.targets += 1;
        }
    }
    let mut out = Vec::new();
    let mut begin = 0;
    let mut held = 0;
    for record in &records {
        if record.targets > max_targets {
            return Err(fail(format!(
                "the record at {} holds {} targets, above the chunk cap {max_targets}",
                record.start, record.targets
            )));
        }
        if held + record.targets > max_targets {
            out.push(begin..record.start);
            begin = record.start;
            held = 0;
        }
        held += record.targets;
    }
    out.push(begin..span.len());
    Ok(out)
}

/// The first index where two row tapes differ; a length difference counts at the shorter length.
pub fn first_row_mismatch(a: &[PickRow], b: &[PickRow]) -> Option<usize> {
    let shared = a.len().min(b.len());
    (0..shared)
        .find(|&i| a[i] != b[i])
        .or((a.len() != b.len()).then_some(shared))
}

/// The scorer's view of each row: its class and whether the pick hit.
pub fn fire_rows_to_targets(rows: &[PickRow]) -> Vec<PickTarget> {
    rows.iter()
        .map(|row| PickTarget {
            class: row.class,
            hit: row.hit,
        })
        .collect()
}

/// claude-441: the scorer's view under the split rule, the class from each row's
/// distance. Refuses a row whose split class disagrees with its own class: not
/// fired stays 0, distance 0 stays 1, and every other distance had class 2.
pub fn fire_rows_to_split_targets(rows: &[PickRow]) -> Result<Vec<PickTarget>, Failure> {
    use uor_r4_discrete::kn_ceiling::pick_split_class;
    rows.iter()
        .enumerate()
        .map(|(at, row)| {
            let class = pick_split_class(row.distance);
            let three = class.min(2);
            if three != row.class {
                return Err(fail(format!(
                    "target {at}: class {} with distance {:?}, split class {class}",
                    row.class, row.distance
                )));
            }
            Ok(PickTarget {
                class,
                hit: row.hit,
            })
        })
        .collect()
}

/// C2's B1: an independent host read of the exact pair. With one distinct code per token,
/// the code Pick's distance is 0 exactly at an exact pair, and its latest-minimum rule is
/// C2's latest-pair rule; so C2 must equal the code Pick's fired rows and be empty elsewhere.
/// Returns the targets compared.
pub fn check_c2(stream: &[u16], c2: &[PickRow], context: usize) -> Result<usize, Failure> {
    use uor_r4_discrete::kn_pick::code_pick_rows;
    let identity: Vec<[u16; 1]> = (0..VOCAB).map(|id| [id as u16]).collect();
    let expected: Vec<PickRow> = code_pick_rows(stream, &identity, context)?
        .into_iter()
        .map(|row| {
            if row.class == 1 {
                row
            } else {
                PickRow {
                    class: 0,
                    hit: false,
                    pick: None,
                    distance: None,
                }
            }
        })
        .collect();
    if let Some(at) = first_row_mismatch(c2, &expected) {
        return Err(fail(format!(
            "B1: C2 differs from the identity-code read at target {at}"
        )));
    }
    if c2.is_empty() {
        return Err(fail("B1: C2 compared no target"));
    }
    Ok(c2.len())
}

/// The `VmHWM:` value of a `/proc/self/status` text, in KiB.
pub fn vm_hwm_kib(status: &str) -> Option<u64> {
    let line = status.lines().find(|line| line.starts_with("VmHWM:"))?;
    line.split_whitespace().nth(1)?.parse().ok()
}

/// What main.rs hands the Pick arms.
pub struct PickRun<'a> {
    pub highest: u32,
    pub query: &'a [u16],
    pub classes: &'a [u8],
    pub id_bytes: &'a [u8],
    pub masses: &'a [u64],
    pub raw: &'a RawTables,
    pub segments_path: &'a str,
    pub trunk_path: &'a str,
    pub check_host: bool,
    pub profile_phases: bool,
    pub out: &'a str,
    pub header: serde_json::Value,
}

#[cfg(feature = "cuda")]
fn span_lookups(
    lookup: &uor_r4_discrete::kn_lookup_device::KnLookup,
    raw: &RawTables,
    span: &[u16],
    check_host: bool,
    name: &str,
) -> Result<(Vec<u32>, Vec<u64>), Failure> {
    let mut counts = Vec::new();
    let mut totals = Vec::new();
    // Each chunk starts at a BOS, so its host lookups stand alone: the host check
    // holds one chunk twice, never the whole span.
    for range in chunk_bounds(span, LOOKUP_CHUNK)? {
        let (c, t) = lookup.lookups(&span[range.clone()])?;
        if check_host {
            let (host_c, host_t) = raw.lookups(&span[range])?;
            let first = |a: usize, b: usize, differ: &dyn Fn(usize) -> bool| {
                (0..a.max(b)).find(|&i| differ(i))
            };
            if let Some(i) = first(c.len(), host_c.len(), &|i| c.get(i) != host_c.get(i)) {
                return Err(fail(format!(
                    "{name} lookup counts differ from host at {}",
                    counts.len() + i
                )));
            }
            if let Some(i) = first(t.len(), host_t.len(), &|i| t.get(i) != host_t.get(i)) {
                return Err(fail(format!(
                    "{name} lookup totals differ from host at {}",
                    totals.len() + i
                )));
            }
        }
        counts.extend(c);
        totals.extend(t);
    }
    if check_host {
        if totals.is_empty() {
            return Err(fail(format!("{name}: the host check compared no target")));
        }
        eprintln!("{name}: host lookups = GPU at {} targets", totals.len());
    }
    Ok((counts, totals))
}

#[cfg(feature = "cuda")]
fn rows_per_second(rows: usize, ms: u128) -> u128 {
    rows as u128 * 1000 / ms.max(1)
}

#[cfg(feature = "cuda")]
fn check_rows(device: &[PickRow], host: &[PickRow], arm: &str) -> Result<usize, Failure> {
    if let Some(at) = first_row_mismatch(device, host) {
        return Err(fail(format!(
            "B1: {arm} device row differs from the host rows at target {at}"
        )));
    }
    Ok(device.len())
}

/// The trunk oracle at every kept target: all four blocks' words, and a fired class
/// wherever the oracle has a successor. Returns the first mismatch by target.
#[cfg(feature = "cuda")]
fn check_oracle(
    model: &uor_r4_integer::discrete_d11::IntegerDiscreteModel,
    main: &[u16],
    tape: &crate::pick_device::DeviceTape,
) -> Result<usize, Failure> {
    let windows = crate::pick_device::windows(main, CONTEXT)?;
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let per = tape.words.len().div_ceil(threads).max(1);
    let first: Option<(usize, String)> = std::thread::scope(|scope| {
        let handles: Vec<_> = tape
            .words
            .chunks(per)
            .map(|kept_chunk| {
                let windows = &windows;
                scope.spawn(move || -> Option<(usize, String)> {
                    for kept in kept_chunk {
                        let k = kept.ordinal;
                        let Some(&(_, window)) = windows.get(k) else {
                            return Some((k, format!("no window for target {k}")));
                        };
                        let prefix: Vec<u32> = window.iter().map(|&id| u32::from(id)).collect();
                        let seen = match model.observe_identity_pick(&prefix) {
                            Ok(seen) => seen,
                            Err(e) => return Some((k, format!("oracle at target {k}: {e}"))),
                        };
                        if seen.block_words.as_slice() != kept.blocks.as_slice() {
                            return Some((k, format!("A1 block words differ at target {k}")));
                        }
                        let class = tape.rows.get(k).map(|row| row.class);
                        if seen.successor.is_some() && class.is_none_or(|c| c == 0) {
                            return Some((
                                k,
                                format!("A1 row at target {k} has class 0 where the oracle has a successor"),
                            ));
                        }
                    }
                    None
                })
            })
            .collect();
        handles
            .into_iter()
            .filter_map(|h| {
                h.join()
                    .unwrap_or_else(|_| Some((0, "an oracle thread panicked".into())))
            })
            .min_by_key(|found| found.0)
    });
    if let Some((_, message)) = first {
        return Err(fail(format!("B1: {message}")));
    }
    Ok(tape.words.len())
}

/// The record's file is claimed first: an existing path stops the run before any work.
/// A run that fails removes the empty file it claimed, so no 0-byte file sits at a
/// record's name beside a crashed run.
#[cfg(feature = "cuda")]
pub fn run(job: PickRun) -> Result<(), Failure> {
    let out = job.out;
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out)?;
    let result = run_claimed(job, file);
    if result.is_err() {
        if let Err(e) = std::fs::remove_file(out) {
            eprintln!("pick-arms: could not remove the claimed {out}: {e}");
        }
    }
    result
}

#[cfg(feature = "cuda")]
fn run_claimed(job: PickRun, mut file: std::fs::File) -> Result<(), Failure> {
    use crate::pick_device::{load_trunk, root_codes, PickDevice, C3_SEED, TRUNK_SHA256};
    use crate::pick_fit::{combine, run_main, run_plus, SpanInputs};
    use serde_json::{json, Map, Value};
    use std::time::Instant;
    use uor_r4_discrete::kn_lookup_device::KnLookup;
    use uor_r4_discrete::kn_pick::{
        code_pick_rows, exact_pair_rows, permute_code_rows, row_permutation,
    };

    let started = Instant::now();
    let mut phases = Map::new();
    fn phase(phases: &mut Map<String, Value>, name: &str, at: Instant) -> u128 {
        let ms = at.elapsed().as_millis();
        eprintln!("pick-arms: {name}, {ms} ms");
        phases.insert(format!("{name}_ms"), json!(ms));
        ms
    }

    // 1. Segments, spans and the trunk.
    let at = Instant::now();
    let segments = std::fs::read(job.segments_path)?;
    let segments_sha256 = crate::sha256_hex(&segments);
    let spans = split_spans(job.query, &segments)?;
    if job.classes.len() != job.query.len() {
        return Err(fail("classes and query lengths differ"));
    }
    let model = load_trunk(std::path::Path::new(job.trunk_path))?;
    let (blocks, _, _) = model.identity_pick_inventory()?;
    let codes = root_codes(&blocks)?;
    let perm = row_permutation(VOCAB, C3_SEED);
    let main = &job.query[..spans.split];
    let plus = &job.query[spans.split..];
    let main_target_segments: Vec<u8> = main
        .iter()
        .zip(&segments[..spans.split])
        .filter(|&(&t, _)| t != NGRAM_BOS)
        .map(|(_, &s)| s)
        .collect();
    let dev_tune_targets = main_target_segments.iter().filter(|&&s| s == 1).count();
    eprintln!(
        "pick-arms: main span {} positions, {} targets ({dev_tune_targets} DEV-tune); plus span {} positions",
        main.len(),
        spans.main_targets,
        plus.len()
    );
    phase(&mut phases, "load", at);

    let lookup = KnLookup::new(0, job.raw)?;
    let dev_a1 = PickDevice::new(&model, None)?;
    let mut b1 = Map::new();

    // 2-4. The main span: lookups, the three arms' reads and B1.
    let at = Instant::now();
    let (main_counts, main_totals) = span_lookups(&lookup, job.raw, main, job.check_host, "main")?;
    phase(&mut phases, "main_lookups", at);

    let at = Instant::now();
    let keep = |k: usize| job.check_host && main_target_segments.get(k) == Some(&1);
    let a1_main = dev_a1.rows(main, CONTEXT, &keep)?;
    let ms = phase(&mut phases, "a1_main_read", at);
    phases.insert(
        "a1_main_rows_per_s".into(),
        json!(rows_per_second(a1_main.rows.len(), ms)),
    );

    let at = Instant::now();
    let dev_c3 = PickDevice::new(&model, Some(&perm))?;
    let c3_main = dev_c3.rows(main, CONTEXT, &|_| false)?;
    drop(dev_c3);
    let ms = phase(&mut phases, "c3_main_read", at);
    phases.insert(
        "c3_main_rows_per_s".into(),
        json!(rows_per_second(c3_main.rows.len(), ms)),
    );

    let at = Instant::now();
    let c2_rows = exact_pair_rows(main, VOCAB, CONTEXT)?;
    phase(&mut phases, "c2_main_host", at);

    if job.check_host {
        let at = Instant::now();
        let a1_rows = check_rows(
            &a1_main.rows,
            &code_pick_rows(main, &codes, CONTEXT)?,
            "A1 main",
        )?;
        let permuted = permute_code_rows(&codes, C3_SEED);
        let c3_rows = check_rows(
            &c3_main.rows,
            &code_pick_rows(main, &permuted, CONTEXT)?,
            "C3 main",
        )?;
        let c2_checked = check_c2(main, &c2_rows, CONTEXT)?;
        if dev_tune_targets == 0 {
            return Err(fail("B1: the segments mark no DEV-tune target"));
        }
        if a1_main.words.len() != dev_tune_targets {
            return Err(fail(format!(
                "B1: kept words at {} targets, expected {dev_tune_targets} DEV-tune targets",
                a1_main.words.len()
            )));
        }
        let oracle_targets = check_oracle(&model, main, &a1_main)?;
        b1.insert("a1_main_rows".into(), json!(a1_rows));
        b1.insert("c3_rows".into(), json!(c3_rows));
        b1.insert("c2_rows".into(), json!(c2_checked));
        b1.insert("oracle_targets".into(), json!(oracle_targets));
        phase(&mut phases, "b1_main", at);
    }
    let a1 = fire_rows_to_targets(&a1_main.rows);
    let c3 = fire_rows_to_targets(&c3_main.rows);
    let c2 = fire_rows_to_targets(&c2_rows);
    // claude-441: the same rows under the split rule.
    let a1s = fire_rows_to_split_targets(&a1_main.rows)?;
    let c3s = fire_rows_to_split_targets(&c3_main.rows)?;
    drop((a1_main, c3_main, c2_rows));
    let main_inputs = SpanInputs {
        highest: job.highest,
        query: main,
        segments: &segments[..spans.split],
        classes: &job.classes[..spans.split],
        id_bytes: job.id_bytes,
        kn_masses: job
            .masses
            .get(..spans.main_targets)
            .ok_or_else(|| fail("fewer KN masses than main-span targets"))?,
        counts: &main_counts,
        totals: &main_totals,
        fit_segment: 0,
    };
    // The main arms run, and the main span's lookups are freed, before the
    // plus span is looked up: the two spans' lookups never coexist.
    let at = Instant::now();
    let split = crate::pick_fit::SplitReads { a1: &a1s, c3: &c3s };
    let main_result = run_main(&main_inputs, &a1, &c2, &c3, Some(split))?;
    check_band_records(main_result.records)?;
    phase(&mut phases, "main_arms", at);
    drop(main_inputs);
    drop((main_counts, main_totals, a1, c2, c3, a1s, c3s));

    // The plus span: lookups, A1 and B1.
    let at = Instant::now();
    let (plus_counts, plus_totals) = span_lookups(&lookup, job.raw, plus, job.check_host, "plus")?;
    drop(lookup);
    phase(&mut phases, "plus_lookups", at);

    let at = Instant::now();
    let a1_plus = dev_a1.rows(plus, CONTEXT, &|_| false)?;
    drop(dev_a1);
    let ms = phase(&mut phases, "a1_plus_read", at);
    phases.insert(
        "a1_plus_rows_per_s".into(),
        json!(rows_per_second(a1_plus.rows.len(), ms)),
    );
    if job.check_host {
        let at = Instant::now();
        let rows = check_rows(
            &a1_plus.rows,
            &code_pick_rows(plus, &codes, CONTEXT)?,
            "A1 plus",
        )?;
        b1.insert("a1_plus_rows".into(), json!(rows));
        phase(&mut phases, "b1_plus", at);
    }
    let plus_a1 = fire_rows_to_targets(&a1_plus.rows);
    drop(a1_plus);
    let plus_inputs = SpanInputs {
        highest: job.highest,
        query: plus,
        segments: &segments[spans.split..],
        classes: &job.classes[spans.split..],
        id_bytes: job.id_bytes,
        kn_masses: job
            .masses
            .get(spans.main_targets..)
            .ok_or_else(|| fail("fewer KN masses than main-span targets"))?,
        counts: &plus_counts,
        totals: &plus_totals,
        fit_segment: 2,
    };

    // 5. The plus arms, then the record of both spans.
    let at = Instant::now();
    let plus_result = run_plus(&plus_inputs, &plus_a1)?;
    phase(&mut phases, "plus_arms", at);
    let arms = combine(main_result, plus_result)?;

    // 6. The record, never over an existing file.
    let mut summary = job.header;
    summary["pick_arms"] = arms;
    summary["b1"] = if job.check_host {
        Value::Object(b1)
    } else {
        Value::Null
    };
    summary["trunk_sha256"] = json!(TRUNK_SHA256);
    summary["segments_sha256"] = json!(segments_sha256);
    if job.profile_phases {
        summary["phases_ms"] = Value::Object(phases);
    }
    // The whole process's VmHWM, counting included; null where it cannot be read.
    let peak = std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| vm_hwm_kib(&status));
    eprintln!("pick-arms: peak RSS {peak:?} KiB");
    summary["peak_rss_kib"] = json!(peak);
    summary["peak_rss_source"] = json!("VmHWM of the whole process");
    use std::io::Write;
    file.write_all((serde_json::to_string_pretty(&summary)? + "\n").as_bytes())?;
    eprintln!("wrote {}, {} ms", job.out, started.elapsed().as_millis());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const B: u16 = NGRAM_BOS;

    #[test]
    fn the_bands_resample_dev_tune_records_only() {
        assert!(check_band_records(DEV_TUNE_RECORDS).is_ok());
        assert!(check_band_records(DEV_TUNE_RECORDS - 1).is_err());
        assert!(check_band_records(DEV_TUNE_RECORDS + 1).is_err());
    }

    #[test]
    fn c2_equals_the_identity_code_read() {
        use uor_r4_discrete::kn_pick::exact_pair_rows;
        // Pairs (5 6) and (7 8) repeat, (5 9) does not; a second record restarts.
        let stream = [B, 5, 6, 7, 8, 5, 6, 7, 8, 5, 9, 6, 1, B, 7, 8, 7, 8, 1];
        let c2 = exact_pair_rows(&stream, VOCAB, CONTEXT).unwrap();
        assert!(c2.iter().any(|row| row.class == 1 && row.hit));
        assert!(c2.iter().any(|row| row.class == 0));
        assert_eq!(check_c2(&stream, &c2, CONTEXT).unwrap(), c2.len());
        // Latest tie: (5 6) occurs twice before the last target, followed by 7 then 8.
        let tied = [B, 5, 6, 7, 5, 6, 8, 5, 6, 8, 1];
        let rows = exact_pair_rows(&tied, VOCAB, CONTEXT).unwrap();
        // Target 8 at q 9 is non-BOS row 8; its window's latest pair is at p = 5.
        assert_eq!((rows[8].pick, rows[8].hit), (Some(5), true));
        assert_eq!(check_c2(&tied, &rows, CONTEXT).unwrap(), rows.len());
        let mut wrong = c2.clone();
        let fired = wrong.iter().position(|row| row.class == 1).unwrap();
        wrong[fired].pick = wrong[fired].pick.map(|p| p - 1);
        assert!(check_c2(&stream, &wrong, CONTEXT).is_err());
        assert!(check_c2(&[B], &[], CONTEXT).is_err());
    }

    #[test]
    fn split_spans_finds_the_plus_span() {
        assert_eq!(B, 0);
        let query = [B, 5, 6, B, 7, B, 8, 9, B, 10];
        let segments = [0, 0, 0, 1, 1, 2, 2, 2, 4, 3];
        assert_eq!(
            split_spans(&query, &segments).unwrap(),
            Spans {
                split: 5,
                main_targets: 3
            }
        );
        // Length mismatch, a segment above 4, a main segment after the plus span,
        // no plus span, an empty main span and a plus span not at a BOS.
        assert!(split_spans(&query, &segments[..9]).is_err());
        assert!(split_spans(&query, &[0, 0, 0, 1, 1, 2, 2, 2, 5, 3]).is_err());
        assert!(split_spans(&query, &[0, 0, 0, 1, 1, 2, 2, 1, 4, 3]).is_err());
        assert!(split_spans(&query, &[0; 10]).is_err());
        assert!(split_spans(&query, &[2; 10]).is_err());
        assert!(split_spans(&query, &[0, 0, 0, 1, 2, 2, 2, 2, 4, 3]).is_err());
    }

    #[test]
    fn chunk_bounds_cut_at_records() {
        let span = [B, 5, 6, B, 7, B, 8, 9, 10, B, B, 11];
        let ranges = chunk_bounds(&span, 3).unwrap();
        assert_eq!(ranges, vec![0..5, 5..10, 10..12]);
        let mut next = 0;
        for range in &ranges {
            assert_eq!(range.start, next);
            assert_eq!(span[range.start], B);
            let targets = span[range.clone()].iter().filter(|&&t| t != B).count();
            assert!(targets <= 3);
            next = range.end;
        }
        assert_eq!(next, span.len());
        assert_eq!(chunk_bounds(&span, 16).unwrap(), vec![0..12]);
        assert_eq!(
            chunk_bounds(&span, 2).unwrap_err().to_string(),
            "the record at 5 holds 3 targets, above the chunk cap 2"
        );
        assert!(chunk_bounds(&span[1..], 16).is_err());
    }

    #[test]
    fn first_row_mismatch_finds_the_first_difference() {
        let row = |class: u8| PickRow {
            class,
            hit: class == 2,
            pick: None,
            distance: None,
        };
        let a = [row(0), row(1), row(2)];
        assert_eq!(first_row_mismatch(&a, &a), None);
        assert_eq!(first_row_mismatch(&a, &[row(0), row(3), row(1)]), Some(1));
        assert_eq!(first_row_mismatch(&a, &a[..2]), Some(2));
        assert_eq!(first_row_mismatch(&a[..1], &a), Some(1));
        let targets = fire_rows_to_targets(&a);
        assert_eq!(
            targets[2],
            PickTarget {
                class: 2,
                hit: true
            }
        );
    }

    #[test]
    fn split_targets_follow_the_distance() {
        let row = |class: u8, distance: Option<u32>| PickRow {
            class,
            hit: class != 0,
            pick: distance.map(|_| 1),
            distance,
        };
        let rows = [
            row(0, None),
            row(1, Some(0)),
            row(2, Some(1)),
            row(2, Some(2)),
            row(2, Some(3)),
            row(2, Some(40)),
        ];
        let got = fire_rows_to_split_targets(&rows).unwrap();
        let classes: Vec<u8> = got.iter().map(|t| t.class).collect();
        assert_eq!(classes, vec![0, 1, 2, 3, 4, 4]);
        assert!(got.iter().zip(&rows).all(|(t, r)| t.hit == r.hit));
        // A row whose class disagrees with its distance is refused.
        for bad in [row(1, Some(2)), row(2, Some(0)), row(0, Some(1)), row(2, None)] {
            assert!(fire_rows_to_split_targets(&[bad]).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn vm_hwm_reads_the_peak_line() {
        let status = "Name:\tkn\nVmPeak:\t  900 kB\nVmHWM:\t  4321 kB\nVmRSS:\t 12 kB\n";
        assert_eq!(vm_hwm_kib(status), Some(4321));
        assert_eq!(vm_hwm_kib("Name:\tkn\n"), None);
    }
}
