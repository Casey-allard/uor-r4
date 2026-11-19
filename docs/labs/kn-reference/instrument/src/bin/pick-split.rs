//! pick-split (claude-439): prepares the S5 Pick expert's streams on a host.
//!
//! `pick-split TRAIN.tokens.u16 DEV-TUNE.tokens.u16 DEV-TUNE.novel.u8 OUT-DIR`
//!
//! TRAIN's records split by index: R holds the records whose index mod 128 is 0, C the rest
//! in stream order. The query is R, then DEV-tune, then R+ and DEV+ (`repeat_chunks` with
//! 64-token chunks). It writes `c.tokens.u16`, `q.tokens.u16` (the UORT header copied from
//! TRAIN with the token count replaced), `q.classes.u8` and `q.segments.u8` (one raw byte per
//! query position) and `manifest.json`, and prints the manifest. It refuses the sealed
//! DEV-test stream and never overwrites an existing output. Integer arithmetic only.

use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::ops::Range;
use std::path::Path;
use uor_r4_discrete::kn_pick::repeat_chunks;

const BOS: u16 = 0;
const EOS: u16 = 1;
const CLASS_BOS: u8 = 3;
const R_EVERY: usize = 128;
const CHUNK: usize = 64;
const HEADER: usize = 64;

/// The sha256 of claude-420's sealed `dev-test.tokens.u16` (as in `src/main.rs`).
const DEV_TEST_TOKENS_SHA256: &str =
    "023784b74c98a796401a064d50bf2f83f2f8eb95ab8bbead1cbe2aec6e7a176b";

const OUTPUTS: [&str; 5] = [
    "c.tokens.u16",
    "q.tokens.u16",
    "q.classes.u8",
    "q.segments.u8",
    "manifest.json",
];

type Failure = Box<dyn std::error::Error>;

fn fail(message: impl Into<String>) -> Failure {
    message.into().into()
}

fn hex(digest: &[u8]) -> String {
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// The three inputs the sheet pins (claude-420's split v1 `train.json` and `dev-tune.json`).
const TRAIN_TOKENS_SHA256: &str =
    "c6cba29ead4c745fb47b3f87007d982e72defc9f04102a268709ba0d34d51460";
const DEV_TUNE_TOKENS_SHA256: &str =
    "75c6d1ac479bf120bac3aa60ac160536e8146f8052aa49c002209500799b7d0a";
const DEV_TUNE_NOVEL_SHA256: &str =
    "7df42437260bb767e2b1bdfd0cc55a242bd6e0b632a5bcd754852a7ab8aca6af";

/// Refuses a path naming the sealed DEV-test split, before it is opened.
fn refuse_sealed_path(path: &str) -> Result<(), Failure> {
    if path.to_ascii_lowercase().contains("dev-test") {
        return Err(fail(format!(
            "{path} names the sealed DEV-test split; pick-split never opens it"
        )));
    }
    Ok(())
}

/// Accepts only the three pinned inputs, each in its own place. The sealed DEV-test stream
/// is refused by name as well as by not being on the list.
fn check_inputs(train: &str, dev: &str, novel: &str, pins: [&str; 3]) -> Result<(), Failure> {
    for (role, got) in [
        ("TRAIN", train),
        ("DEV-TUNE", dev),
        ("DEV-TUNE novel", novel),
    ] {
        if got == DEV_TEST_TOKENS_SHA256 {
            return Err(fail(format!(
                "the {role} input is the sealed DEV-test stream; pick-split never reads it"
            )));
        }
    }
    for (role, got, want) in [
        ("TRAIN", train, pins[0]),
        ("DEV-TUNE", dev, pins[1]),
        ("DEV-TUNE novel", novel, pins[2]),
    ] {
        if got != want {
            return Err(fail(format!(
                "the {role} input has sha256 {got}, not the pinned {want}"
            )));
        }
    }
    Ok(())
}

/// The record start offsets of a framed stream `BOS ... EOS` back to back, with the stream
/// length appended. Refuses an empty stream, one not starting with BOS or not ending with
/// EOS, and a BOS not preceded by EOS.
fn record_starts(tokens: &[u16]) -> Result<Vec<usize>, Failure> {
    if tokens.len() < 2 || tokens[0] != BOS || tokens[tokens.len() - 1] != EOS {
        return Err(fail(
            "the stream is not framed: it must start with BOS and end with EOS",
        ));
    }
    let mut starts = vec![0];
    for k in 1..tokens.len() {
        if tokens[k] == BOS {
            if tokens[k - 1] != EOS {
                return Err(fail(format!(
                    "the stream is not framed: BOS at {k} follows no EOS"
                )));
            }
            starts.push(k);
        }
    }
    starts.push(tokens.len());
    Ok(starts)
}

/// R (records with index mod 128 = 0, concatenated) and C as spans of `tokens`, each span a
/// run of consecutive C records, in order. C stays a view so TRAIN is held once.
fn split_records(tokens: &[u16]) -> Result<(Vec<u16>, Vec<Range<usize>>), Failure> {
    let starts = record_starts(tokens)?;
    let mut r = Vec::new();
    let mut c: Vec<Range<usize>> = Vec::new();
    for (i, pair) in starts.windows(2).enumerate() {
        if i % R_EVERY == 0 {
            r.extend_from_slice(&tokens[pair[0]..pair[1]]);
        } else {
            match c.last_mut() {
                Some(last) if last.end == pair[0] => last.end = pair[1],
                _ => c.push(pair[0]..pair[1]),
            }
        }
    }
    Ok((r, c))
}

/// The query R + DEV + R+ + DEV+, its class bytes and its segment bytes.
fn build_query(
    r: &[u16],
    dev: &[u16],
    dev_novel: &[u8],
) -> Result<(Vec<u16>, Vec<u8>, Vec<u8>), Failure> {
    record_starts(r)?;
    record_starts(dev)?;
    if dev_novel.len() != dev.len() {
        return Err(fail(format!(
            "the novel file holds {} bytes for {} DEV-tune tokens",
            dev_novel.len(),
            dev.len()
        )));
    }
    if let Some(k) = (0..dev.len()).find(|&k| dev_novel[k] > CLASS_BOS) {
        return Err(fail(format!(
            "novel byte {} at {k} is past class 3",
            dev_novel[k]
        )));
    }
    if let Some(k) = (0..dev.len()).find(|&k| dev[k] == BOS && dev_novel[k] != CLASS_BOS) {
        return Err(fail(format!(
            "DEV-tune BOS at {k} has novel class {}",
            dev_novel[k]
        )));
    }
    let (r_plus, _) = repeat_chunks(r, CHUNK)?;
    let (dev_plus, dev_second) = repeat_chunks(dev, CHUNK)?;
    let total = r.len() + dev.len() + r_plus.len() + dev_plus.len();
    let mut q = Vec::with_capacity(total);
    let mut classes = Vec::with_capacity(total);
    let mut segments = Vec::with_capacity(total);
    let bos_class = |t: u16| if t == BOS { CLASS_BOS } else { 0 };
    for &t in r {
        q.push(t);
        classes.push(bos_class(t));
        segments.push(0);
    }
    for (k, &t) in dev.iter().enumerate() {
        q.push(t);
        classes.push(dev_novel[k]);
        segments.push(1);
    }
    for &t in &r_plus {
        q.push(t);
        classes.push(bos_class(t));
        segments.push(2);
    }
    let mut flags = dev_second.iter();
    for &t in &dev_plus {
        q.push(t);
        classes.push(bos_class(t));
        let second = if t == BOS {
            false
        } else {
            *flags
                .next()
                .ok_or_else(|| fail("repeat_chunks gave too few flags"))?
        };
        segments.push(if second { 4 } else { 3 });
    }
    if flags.next().is_some() {
        return Err(fail("repeat_chunks gave too many flags"));
    }
    Ok((q, classes, segments))
}

/// Reads a UORT token file in one pass: its 64-byte header, its tokens and its sha256.
fn read_tokens(path: &Path) -> Result<([u8; HEADER], Vec<u16>, String), Failure> {
    let mut file = File::open(path)?;
    let mut header = [0u8; HEADER];
    file.read_exact(&mut header)
        .map_err(|_| fail(format!("{} lacks the 64-byte UORT header", path.display())))?;
    if &header[..4] != b"UORT" {
        return Err(fail(format!("{} lacks the UORT magic", path.display())));
    }
    let count = u64::from_le_bytes(header[8..16].try_into().expect("8 bytes"));
    let count = usize::try_from(count).map_err(|_| fail("token count past usize"))?;
    let mut hasher = Sha256::new();
    hasher.update(header);
    let mut tokens = Vec::with_capacity(count);
    let mut buffer = vec![0u8; 1 << 23];
    let mut carry: Option<u8> = None;
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
        let mut bytes = &buffer[..n];
        if let Some(low) = carry.take() {
            tokens.push(u16::from_le_bytes([low, bytes[0]]));
            bytes = &bytes[1..];
        }
        let mut pairs = bytes.chunks_exact(2);
        tokens.extend(pairs.by_ref().map(|p| u16::from_le_bytes([p[0], p[1]])));
        carry = pairs.remainder().first().copied();
        if tokens.len() > count {
            break;
        }
    }
    if carry.is_some() || tokens.len() != count {
        return Err(fail(format!(
            "{}: the UORT header names {count} tokens but the payload differs",
            path.display()
        )));
    }
    Ok((header, tokens, hex(&hasher.finalize())))
}

/// A writer that hashes and counts what it writes.
struct Hashed {
    out: BufWriter<File>,
    hasher: Sha256,
    bytes: u64,
}

fn create_hashed(path: &Path) -> Result<Hashed, Failure> {
    let file = File::options().write(true).create_new(true).open(path)?;
    Ok(Hashed {
        out: BufWriter::with_capacity(1 << 23, file),
        hasher: Sha256::new(),
        bytes: 0,
    })
}

fn put(w: &mut Hashed, bytes: &[u8]) -> Result<(), Failure> {
    w.out.write_all(bytes)?;
    w.hasher.update(bytes);
    w.bytes += bytes.len() as u64;
    Ok(())
}

fn put_tokens(w: &mut Hashed, tokens: &[u16]) -> Result<(), Failure> {
    let mut bytes = Vec::with_capacity(2 * tokens.len().min(1 << 22));
    for piece in tokens.chunks(1 << 22) {
        bytes.clear();
        bytes.extend(piece.iter().flat_map(|t| t.to_le_bytes()));
        put(w, &bytes)?;
    }
    Ok(())
}

fn finish(w: Hashed) -> Result<serde_json::Value, Failure> {
    let Hashed {
        mut out,
        hasher,
        bytes,
    } = w;
    out.flush()?;
    out.get_ref().sync_all()?;
    Ok(serde_json::json!({ "sha256": hex(&hasher.finalize()), "bytes": bytes }))
}

fn uort_header(template: &[u8; HEADER], count: usize) -> [u8; HEADER] {
    let mut header = *template;
    header[8..16].copy_from_slice(&(count as u64).to_le_bytes());
    header
}

/// Refuses an OUT-DIR that already holds any output.
fn refuse_existing(out_dir: &Path) -> Result<(), Failure> {
    for name in OUTPUTS {
        let path = out_dir.join(name);
        if path.exists() {
            return Err(fail(format!(
                "{} exists; pick-split never overwrites",
                path.display()
            )));
        }
    }
    Ok(())
}

fn records(tokens: &[u16]) -> usize {
    tokens.iter().filter(|&&t| t == BOS).count()
}

/// The sheet's pinned inputs: TRAIN, DEV-tune and its novel classes.
const PINS: [&str; 3] = [
    TRAIN_TOKENS_SHA256,
    DEV_TUNE_TOKENS_SHA256,
    DEV_TUNE_NOVEL_SHA256,
];

fn run(args: &[String]) -> Result<serde_json::Value, Failure> {
    run_pinned(args, PINS)
}

fn run_pinned(args: &[String], pins: [&str; 3]) -> Result<serde_json::Value, Failure> {
    let [train_path, dev_path, novel_path, out_dir] = args else {
        return Err(fail(
            "usage: pick-split TRAIN.tokens.u16 DEV-TUNE.tokens.u16 DEV-TUNE.novel.u8 OUT-DIR",
        ));
    };
    for path in [train_path, dev_path, novel_path] {
        refuse_sealed_path(path)?;
    }
    let out_dir = Path::new(out_dir);
    refuse_existing(out_dir)?;

    let (_, dev, dev_sha256) = read_tokens(Path::new(dev_path))?;
    let novel = std::fs::read(novel_path)?;
    let novel_sha256 = hex(&Sha256::digest(&novel));
    let (train_header, train, train_sha256) = read_tokens(Path::new(train_path))?;
    check_inputs(&train_sha256, &dev_sha256, &novel_sha256, pins)?;

    // Every input check runs before the first output is created.
    let (r, c) = split_records(&train)?;
    let (q, classes, segments) = build_query(&r, &dev, &novel)?;
    std::fs::create_dir_all(out_dir)?;
    refuse_existing(out_dir)?;
    let c_tokens: usize = c.iter().map(|s| s.len()).sum();
    let c_records: usize = c.iter().map(|s| records(&train[s.clone()])).sum();
    let mut c_out = create_hashed(&out_dir.join("c.tokens.u16"))?;
    put(&mut c_out, &uort_header(&train_header, c_tokens))?;
    for span in &c {
        put_tokens(&mut c_out, &train[span.clone()])?;
    }
    let c_file = finish(c_out)?;
    let train_tokens = train.len();
    let train_records = records(&train);
    drop(train);

    let mut q_out = create_hashed(&out_dir.join("q.tokens.u16"))?;
    put(&mut q_out, &uort_header(&train_header, q.len()))?;
    put_tokens(&mut q_out, &q)?;
    let q_file = finish(q_out)?;
    let mut classes_out = create_hashed(&out_dir.join("q.classes.u8"))?;
    put(&mut classes_out, &classes)?;
    let classes_file = finish(classes_out)?;
    let mut segments_out = create_hashed(&out_dir.join("q.segments.u8"))?;
    put(&mut segments_out, &segments)?;
    let segments_file = finish(segments_out)?;

    let mut segment_tokens = [0usize; 5];
    let mut segment_targets = [0usize; 5];
    for (k, &s) in segments.iter().enumerate() {
        segment_tokens[s as usize] += 1;
        if q[k] != BOS {
            segment_targets[s as usize] += 1;
        }
    }
    let r_plus_tokens = segment_tokens[2];
    let dev_plus_tokens = segment_tokens[3] + segment_tokens[4];
    let manifest = serde_json::json!({
        "schema": "uor-r4.pick-split/1",
        "unit": "claude-439",
        "rules": {
            "split": "R: TRAIN records with index mod 128 = 0; C: the rest",
            "plus": "P+: 64-token chunks, each followed by its copy",
        },
        "inputs": {
            "train_sha256": train_sha256,
            "dev_tune_sha256": dev_sha256,
            "dev_tune_novel_sha256": novel_sha256,
        },
        "outputs": {
            "c.tokens.u16": c_file,
            "q.tokens.u16": q_file,
            "q.classes.u8": classes_file,
            "q.segments.u8": segments_file,
        },
        "counts": {
            "train": { "records": train_records, "tokens": train_tokens },
            "r": { "records": records(&r), "tokens": r.len() },
            "c": { "records": c_records, "tokens": c_tokens },
            "dev": { "records": records(&dev), "tokens": dev.len() },
            "r_plus": { "records": records(&r), "tokens": r_plus_tokens },
            "dev_plus": { "records": records(&dev), "tokens": dev_plus_tokens },
            "query_tokens": q.len(),
            "non_bos_targets_by_segment": segment_targets,
        },
    });
    let mut manifest_out = File::options()
        .write(true)
        .create_new(true)
        .open(out_dir.join("manifest.json"))?;
    manifest_out.write_all(serde_json::to_string_pretty(&manifest)?.as_bytes())?;
    manifest_out.write_all(b"\n")?;
    manifest_out.sync_all()?;
    Ok(manifest)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(manifest) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&manifest).expect("manifest json")
            );
        }
        Err(error) => {
            eprintln!("pick-split: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Record `i` is `BOS, 2 + i % 7, ..., EOS` with `1 + i % 3` body tokens.
    fn record(i: usize) -> Vec<u16> {
        let mut out = vec![BOS];
        out.extend((0..1 + i % 3).map(|j| (2 + (i + j) % 7) as u16));
        out.push(EOS);
        out
    }

    #[test]
    fn the_split_takes_every_128th_record() {
        let stream: Vec<u16> = (0..300).flat_map(record).collect();
        let (r, c) = split_records(&stream).unwrap();
        let want_r: Vec<u16> = [0, 128, 256].iter().flat_map(|&i| record(i)).collect();
        assert_eq!(r, want_r);
        let c_tokens: Vec<u16> = c.iter().flat_map(|s| stream[s.clone()].to_vec()).collect();
        let want_c: Vec<u16> = (0..300).filter(|i| i % 128 != 0).flat_map(record).collect();
        assert_eq!(c_tokens, want_c);
        assert_eq!(r.len() + c_tokens.len(), stream.len());
        assert_eq!(records(&r), 3);
        assert_eq!(records(&c_tokens), 297);
        record_starts(&r).unwrap();
        record_starts(&c_tokens).unwrap();
    }

    #[test]
    fn the_query_is_r_dev_rplus_devplus() {
        let r: Vec<u16> = (0..2).flat_map(record).collect();
        let mut dev = vec![BOS];
        dev.extend((0..70).map(|j| 2 + (j % 5) as u16));
        dev.push(EOS);
        dev.extend(record(4));
        let novel: Vec<u8> = dev
            .iter()
            .enumerate()
            .map(|(k, &t)| if t == BOS { 3 } else { (k % 3) as u8 })
            .collect();
        let (q, classes, segments) = build_query(&r, &dev, &novel).unwrap();
        let (r_plus, _) = repeat_chunks(&r, CHUNK).unwrap();
        let (dev_plus, second) = repeat_chunks(&dev, CHUNK).unwrap();
        let total = r.len() + dev.len() + r_plus.len() + dev_plus.len();
        assert_eq!(
            (q.len(), classes.len(), segments.len()),
            (total, total, total)
        );
        let mut want_q = r.clone();
        want_q.extend(&dev);
        want_q.extend(&r_plus);
        want_q.extend(&dev_plus);
        assert_eq!(q, want_q);
        // Segments are contiguous and in order 0, 1, 2, then 3 and 4 inside DEV+.
        assert!(segments
            .windows(2)
            .all(|w| w[0] <= w[1] || (w[0] == 4 && w[1] == 3)));
        let base = r.len() + dev.len() + r_plus.len();
        assert!(segments[..r.len()].iter().all(|&s| s == 0));
        assert!(segments[r.len()..r.len() + dev.len()]
            .iter()
            .all(|&s| s == 1));
        assert!(segments[r.len() + dev.len()..base].iter().all(|&s| s == 2));
        let mut flags = second.iter();
        for (k, &t) in dev_plus.iter().enumerate() {
            let want = if t == BOS {
                3
            } else if *flags.next().unwrap() {
                4
            } else {
                3
            };
            assert_eq!(segments[base + k], want, "DEV+ position {k}");
        }
        assert!(flags.next().is_none());
        // Body 70 cut 64 + 6: 70 second-copy tokens; record(4) body 2: 2 more.
        assert_eq!(segments.iter().filter(|&&s| s == 4).count(), 72);
        for k in 0..total {
            let in_dev = (r.len()..r.len() + dev.len()).contains(&k);
            let want = if in_dev {
                novel[k - r.len()]
            } else if q[k] == BOS {
                3
            } else {
                0
            };
            assert_eq!(classes[k], want, "class at {k}");
        }
        assert_eq!(
            classes
                .iter()
                .zip(&q)
                .filter(|(&c, &t)| c == 3 && t == BOS)
                .count(),
            records(&q)
        );
    }

    #[test]
    fn a_stream_without_framing_is_refused() {
        assert!(split_records(&[2, 3, EOS]).is_err());
        assert!(split_records(&[BOS, 2, 3]).is_err());
        assert!(split_records(&[BOS, 2, BOS, 3, EOS]).is_err());
        assert!(split_records(&[]).is_err());
        let dev = record(1);
        assert!(build_query(&[2, EOS], &dev, &[3, 0, 0, 0]).is_err());
    }

    #[test]
    fn only_the_pinned_inputs_are_read() {
        let (t, d, n) = (
            TRAIN_TOKENS_SHA256,
            DEV_TUNE_TOKENS_SHA256,
            DEV_TUNE_NOVEL_SHA256,
        );
        assert!(check_inputs(t, d, n, PINS).is_ok());
        let sealed = DEV_TEST_TOKENS_SHA256;
        assert!(check_inputs(sealed, d, n, PINS).is_err());
        assert!(check_inputs(t, sealed, n, PINS).is_err());
        assert!(check_inputs(t, d, sealed, PINS).is_err());
        let other = "0".repeat(64);
        assert!(check_inputs(&other, d, n, PINS).is_err());
        assert!(check_inputs(t, &other, n, PINS).is_err());
        assert!(check_inputs(t, d, &other, PINS).is_err());
        assert!(check_inputs(d, t, n, PINS).is_err());
        assert!(refuse_sealed_path("/mnt/x/split-v1/dev-test.tokens.u16").is_err());
        assert!(refuse_sealed_path("/mnt/x/DEV-TEST/a.u16").is_err());
        assert!(refuse_sealed_path("/mnt/x/split-v1/dev-tune.tokens.u16").is_ok());
    }

    #[test]
    fn existing_outputs_are_never_overwritten() {
        let dir = std::env::temp_dir().join(format!("pick-split-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let write = |name: &str, bytes: &[u8]| std::fs::write(dir.join(name), bytes).unwrap();
        let uort = |tokens: &[u16]| {
            let mut bytes = vec![0u8; HEADER];
            bytes[..4].copy_from_slice(b"UORT");
            bytes[8..16].copy_from_slice(&(tokens.len() as u64).to_le_bytes());
            bytes.extend(tokens.iter().flat_map(|t| t.to_le_bytes()));
            bytes
        };
        let train: Vec<u16> = (0..3).flat_map(record).collect();
        let dev = record(5);
        write("train.u16", &uort(&train));
        write("dev.u16", &uort(&dev));
        write("novel.u8", &[3, 0, 1, 0, 0]);
        let out = dir.join("out");
        std::fs::create_dir_all(&out).unwrap();
        std::fs::write(out.join("q.segments.u8"), b"keep").unwrap();
        let args: Vec<String> = ["train.u16", "dev.u16", "novel.u8"]
            .iter()
            .map(|n| dir.join(n).display().to_string())
            .chain([out.display().to_string()])
            .collect();
        let digest = |name: &str| hex(&Sha256::digest(std::fs::read(dir.join(name)).unwrap()));
        let (t, d, n) = (digest("train.u16"), digest("dev.u16"), digest("novel.u8"));
        let pins = [t.as_str(), d.as_str(), n.as_str()];
        assert!(run_pinned(&args, pins).is_err());
        assert_eq!(std::fs::read(out.join("q.segments.u8")).unwrap(), b"keep");
        assert!(!out.join("c.tokens.u16").exists());
        std::fs::remove_file(out.join("q.segments.u8")).unwrap();
        // The sheet's pins refuse these synthetic inputs before any output is made.
        assert!(run(&args).is_err());
        assert!(std::fs::read_dir(&out).unwrap().next().is_none());
        let manifest = run_pinned(&args, pins).unwrap();
        assert_eq!(manifest["counts"]["r"]["records"], 1);
        assert_eq!(manifest["counts"]["c"]["records"], 2);
        assert_eq!(manifest["inputs"]["train_sha256"], serde_json::json!(t));
        assert!(run_pinned(&args, pins).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
