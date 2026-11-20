//! Resident direct-compact CUDA target features for the next R-only fit.
mod cuda;

use sha2::{Digest, Sha256};
use std::{
    error::Error,
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Read, Seek, SeekFrom, Write},
    path::Path,
};
use uor_r4_count_runtime::{kn_score::KnDiscounts, CompactCountLookup};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Pin = [u8; 32];
const ORDERS: usize = 11;
const PROBE_WORDS: usize = 12;
const MAX_BATCH: usize = 65_536;
const MAX_UORT_BYTES: u64 = 64 << 20;
const MAX_ARTIFACT_BYTES: u64 = 4_000_000_000;
const FINITE_ARTIFACT_BYTES: u64 = 32 << 20;
const FINITE_TOKENS: usize = 256;
const FEATURE_ROW_BYTES: usize = ORDERS * (4 + 8) + 8;

struct Input {
    ids: Vec<u16>,
    targets: usize,
    pin: Pin,
}

fn pin(value: &str) -> Result<Pin> {
    let mut pin = [0; 32];
    hex::decode_to_slice(value, &mut pin)?;
    Ok(pin)
}

fn sha(bytes: &[u8]) -> Pin {
    Sha256::digest(bytes).into()
}

fn uint32(bytes: &[u8]) -> Result<u32> {
    Ok(u32::from_le_bytes(bytes.try_into()?))
}

fn uint64(bytes: &[u8]) -> Result<u64> {
    Ok(u64::from_le_bytes(bytes.try_into()?))
}

fn parse_input(bytes: &[u8], expected: Pin) -> Result<Input> {
    if sha(bytes) != expected {
        return Err("UORT digest mismatch".into());
    }
    let (header, ids) = bytes.split_at_checked(64).ok_or("truncated UORT header")?;
    if &header[..4] != b"UORT" || uint32(&header[4..8])? != 1 || uint32(&header[16..20])? != 4096 {
        return Err("wrong UORT v1 header or token domain".into());
    }
    if uint64(&header[8..16])?.checked_mul(2) != Some(ids.len() as u64) || ids.is_empty() {
        return Err("wrong UORT payload length".into());
    }
    let mut tokens = Vec::with_capacity(ids.len() / 2);
    let mut previous = 1;
    let mut targets = 0;
    for pair in ids.chunks_exact(2) {
        let id = u16::from_le_bytes([pair[0], pair[1]]);
        if id >= 4096 || (id == 0) != (previous == 1) {
            return Err(format!("invalid frame or token ID at UORT index {}", tokens.len()).into());
        }
        targets += usize::from(id != 0);
        previous = id;
        tokens.push(id);
    }
    if previous != 1 || targets == 0 {
        return Err("UORT must end at EOS and include scored targets".into());
    }
    Ok(Input {
        ids: tokens,
        targets,
        pin: expected,
    })
}

fn bounded(path: &Path, max: u64) -> Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let len = file.metadata()?.len();
    if len > max {
        return Err(format!("{} exceeds the configured byte bound", path.display()).into());
    }
    let mut bytes = Vec::with_capacity(len as usize);
    (&mut file).take(max + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 != len {
        return Err("file length changed during bounded read".into());
    }
    Ok(bytes)
}

struct Rows<'a> {
    ids: &'a [u16],
    at: usize,
    frame: usize,
    ordinal: usize,
}

impl<'a> Rows<'a> {
    fn new(input: &'a Input) -> Self {
        Self {
            ids: &input.ids,
            at: 0,
            frame: 0,
            ordinal: 0,
        }
    }

    fn fill(&mut self, batch: usize, probes: &mut Vec<u16>) -> usize {
        probes.clear();
        let first = self.ordinal;
        while self.at < self.ids.len() && self.ordinal - first < batch {
            let at = self.at;
            self.at += 1;
            if self.ids[at] == 0 {
                self.frame = at;
                continue;
            }
            let causal = &self.ids[self.frame.max(at.saturating_sub(10))..=at];
            let start = probes.len();
            probes.resize(start + PROBE_WORDS, 0);
            probes[start] = causal.len() as u16;
            probes[start + 1..start + 1 + causal.len()].copy_from_slice(causal);
            self.ordinal += 1;
        }
        self.ordinal - first
    }
}

struct Metadata {
    descriptors: Vec<u64>,
    headers: Vec<(usize, Vec<u8>)>,
}

impl Metadata {
    fn check_chunk(&self, offset: usize, bytes: &[u8]) -> Result<()> {
        let end = offset + bytes.len();
        for (start, header) in &self.headers {
            let lo = offset.max(*start);
            let hi = end.min(*start + header.len());
            if lo < hi && bytes[lo - offset..hi - offset] != header[lo - start..hi - start] {
                return Err("compact header bytes changed during upload".into());
            }
        }
        Ok(())
    }
}

fn metadata(file: &mut File, length: u64, train: Pin) -> Result<Metadata> {
    if !(44..=MAX_ARTIFACT_BYTES).contains(&length) {
        return Err("invalid compact artifact byte length".into());
    }
    let mut main = [0; 44];
    file.read_exact(&mut main)?;
    if &main[..8] != b"UORKNC02" || uint32(&main[8..12])? != 10 || main[12..] != train {
        return Err("wrong compact format, maximum order or TRAIN digest".into());
    }
    let mut data = Vec::with_capacity(ORDERS * 6);
    let mut headers = vec![(0, main.to_vec())];
    let mut at = main.len() as u64;
    for k in 0..ORDERS {
        file.seek(SeekFrom::Start(at))?;
        let mut header = [0; 72];
        file.read_exact(&mut header)?;
        headers.push((at as usize, header.to_vec()));
        let followers = uint32(&header[..4])?;
        let contexts = uint32(&header[4..8])?;
        let keys = at + 72;
        let rows = keys + u64::from(contexts) * ((12 * k as u64).div_ceil(8) + 4);
        at = rows + u64::from(followers) * if k == 10 { 6 } else { 10 };
        if at > length || (followers == 0) != (contexts == 0) {
            return Err("invalid compact order lengths".into());
        }
        let mut spectra = [0; 4];
        for (count, bytes) in spectra.iter_mut().zip(header[40..].chunks_exact(8)) {
            *count = uint64(bytes)?;
        }
        let (discounts, _) = KnDiscounts::with_fallback(spectra)?;
        data.extend([
            keys,
            rows,
            u64::from(contexts) | (u64::from(followers) << 32),
        ]);
        data.extend(discounts.dq);
    }
    if at != length {
        return Err("compact artifact has trailing bytes".into());
    }
    file.rewind()?;
    Ok(Metadata {
        descriptors: data,
        headers,
    })
}

#[derive(Debug, PartialEq, Eq)]
struct Feature {
    counts: [u32; ORDERS],
    totals: [u64; ORDERS],
    mass: u64,
}

impl Feature {
    fn write(&self, output: &mut impl Write) -> Result<()> {
        if !(1..=(1 << 48)).contains(&self.mass)
            || self
                .counts
                .iter()
                .zip(self.totals)
                .any(|(&count, total)| u64::from(count) > total)
        {
            return Err("invalid feature count, total or Q48 word".into());
        }
        let mut row = [0; FEATURE_ROW_BYTES];
        for (dst, count) in row[..44].chunks_exact_mut(4).zip(self.counts) {
            dst.copy_from_slice(&count.to_le_bytes());
        }
        for (dst, total) in row[44..132].chunks_exact_mut(8).zip(self.totals) {
            dst.copy_from_slice(&total.to_le_bytes());
        }
        row[132..].copy_from_slice(&self.mass.to_le_bytes());
        output.write_all(&row)?;
        Ok(())
    }
}

struct Output {
    partial: OsString,
    writer: BufWriter<File>,
}

impl Output {
    fn new(path: &Path, input: &Input, train: Pin, compact: Pin) -> Result<Self> {
        match fs::symlink_metadata(path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
            Ok(_) => return Err("refusing to overwrite existing output".into()),
        }
        let mut partial = path.as_os_str().to_owned();
        partial.push(".partial");
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&partial)?;
        let mut writer = BufWriter::with_capacity(1 << 20, file);
        writer.write_all(b"UORCBF01")?;
        writer.write_all(&(input.targets as u64).to_le_bytes())?;
        writer.write_all(&train)?;
        writer.write_all(&compact)?;
        writer.write_all(&input.pin)?;
        Ok(Self { partial, writer })
    }

    fn finish(mut self, path: &Path, targets: usize) -> Result<()> {
        self.writer.flush()?;
        let file = self.writer.into_inner()?;
        if file.metadata()?.len() != 112 + (targets * FEATURE_ROW_BYTES) as u64 {
            return Err("feature output length disagrees with admitted target count".into());
        }
        file.sync_all()?;
        drop(file);
        fs::hard_link(&self.partial, path)?;
        fs::remove_file(&self.partial)?;
        let dir = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        File::open(dir)?.sync_all()?;
        Ok(())
    }
}

fn finite_oracle(
    artifact: &Path,
    train: Pin,
    compact: Pin,
    input: &Input,
    dest: &Path,
) -> Result<()> {
    if input.ids.len() > FINITE_TOKENS {
        return Err("host oracle is restricted to the finite fixture".into());
    }
    let bytes = bounded(artifact, FINITE_ARTIFACT_BYTES)?;
    let native = CompactCountLookup::from_bytes(&bytes, train, compact)?;
    let mut output = Output::new(dest, input, train, compact)?;
    // Separate walk over framing, with no shared Rows/batching code.
    for record in input.ids.split_inclusive(|&id| id == 1) {
        for at in 1..record.len() {
            let history = &record[at.saturating_sub(10)..=at];
            let mut feature = Feature {
                counts: [0; ORDERS],
                totals: [0; ORDERS],
                mass: 0,
            };
            feature.mass = native.lookup(
                history,
                history.len() - 1,
                &mut feature.counts,
                &mut feature.totals,
            )?;
            feature.write(&mut output.writer)?;
        }
    }
    output.finish(dest, input.targets)
}

fn encode_fixture(ids: &[u16]) -> Vec<u8> {
    let mut out = vec![0; 64];
    out[..4].copy_from_slice(b"UORT");
    out[4..8].copy_from_slice(&1u32.to_le_bytes());
    out[8..16].copy_from_slice(&(ids.len() as u64).to_le_bytes());
    out[16..20].copy_from_slice(&4096u32.to_le_bytes());
    out.extend(ids.iter().flat_map(|&id| id.to_le_bytes()));
    out
}

fn fixture() -> Vec<u8> {
    encode_fixture(&[
        0, 1, 0, 33, 21, 22, 268, 86, 421, 19, 20, 23, 3, 223, 1, 0, 7, 11, 28, 617, 484, 971,
        2460, 81, 3, 261, 1, 0, 4095, 1,
    ])
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [mode, dest] if mode == "fixture" => {
            let mut file = OpenOptions::new().create_new(true).write(true).open(dest)?;
            file.write_all(&fixture())?;
            file.sync_all()?;
        }
        [mode, artifact, train, compact, uort, uort_pin, dest, extra @ ..] => {
            let (train, compact) = (pin(train)?, pin(compact)?);
            let input = parse_input(&bounded(Path::new(uort), MAX_UORT_BYTES)?, pin(uort_pin)?)?;
            let (artifact, dest) = (Path::new(artifact), Path::new(dest));
            match (mode.as_str(), extra) {
                ("oracle", []) => finite_oracle(artifact, train, compact, &input, dest)?,
                ("cuda", [batch]) => {
                    let batch: usize = batch.parse()?;
                    if !(1..=MAX_BATCH).contains(&batch) {
                        return Err("batch rows must be 1..=65536".into());
                    }
                    cuda::run(artifact, train, compact, &input, dest, batch)?;
                }
                _ => return Err("invalid count feature command".into()),
            }
        }
        _ => return Err("usage: d36-count-batch fixture NEW-UORT | (oracle|cuda) COMPACT TRAIN_SHA COMPACT_SHA UORT UORT_SHA NEW-FEATURES [BATCH_ROWS for cuda]".into()),
    }
    Ok(())
}

fn main() -> Result<()> {
    run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_probes_survive_batch_and_eos_boundaries() -> Result<()> {
        let ids = [0, 10, 11, 1, 0, 13, 14, 15, 1, 0, 1];
        let bytes = encode_fixture(&ids);
        let input = parse_input(&bytes, sha(&bytes))?;
        let expected: &[&[u16]] = &[
            &[0, 10],
            &[0, 10, 11],
            &[0, 10, 11, 1],
            &[0, 13],
            &[0, 13, 14],
            &[0, 13, 14, 15],
            &[0, 13, 14, 15, 1],
            &[0, 1],
        ];
        for batch in [1, 3, 4, 7, MAX_BATCH] {
            let mut stream = Rows::new(&input);
            let mut probes = Vec::new();
            let mut ordinal = 0;
            loop {
                let n = stream.fill(batch, &mut probes);
                if n == 0 {
                    break;
                }
                assert_eq!(probes.len(), n * PROBE_WORDS);
                for probe in probes.chunks_exact(PROBE_WORDS) {
                    let len = probe[0] as usize;
                    assert_eq!(&probe[1..=len], expected[ordinal]);
                    assert!(probe[len + 1..].iter().all(|&x| x == 0));
                    ordinal += 1;
                }
            }
            assert_eq!(ordinal, expected.len());
        }
        Ok(())
    }

    #[test]
    fn exact_ten_history_at_interior_batch_seam() -> Result<()> {
        let mut ids: Vec<u16> = (0..=15)
            .map(|id| if id == 0 { id } else { id + 1 })
            .collect();
        ids.push(1);
        let bytes = encode_fixture(&ids);
        let input = parse_input(&bytes, sha(&bytes))?;
        let mut rows = Rows::new(&input);
        let mut probes = Vec::new();
        assert_eq!(rows.fill(10, &mut probes), 10);
        assert_eq!(&probes[9 * PROBE_WORDS + 1..10 * PROBE_WORDS], &ids[..11]);
        assert_eq!(rows.fill(1, &mut probes), 1);
        assert_eq!(&probes[1..], &ids[1..12]);
        assert_eq!(rows.fill(32, &mut probes), 5);
        assert_eq!(&probes[4 * PROBE_WORDS + 1..], &ids[6..]);
        Ok(())
    }

    #[test]
    fn rejects_planted_invalid_frames_lengths_and_digest() {
        for ids in [
            &[10, 1][..],
            &[0, 0, 1],
            &[0, 1, 2, 1],
            &[0, 10],
            &[0, 4096, 1],
            &[],
        ] {
            let bytes = encode_fixture(ids);
            assert!(parse_input(&bytes, sha(&bytes)).is_err(), "{ids:?}");
        }
        let mut bytes = fixture();
        let correct = sha(&bytes);
        bytes.pop();
        assert!(parse_input(&bytes, sha(&bytes)).is_err());
        assert!(parse_input(&bytes, correct).is_err());
    }

    #[test]
    fn rejects_a_changed_header_fragment_before_launch() -> Result<()> {
        let metadata = Metadata {
            descriptors: Vec::new(),
            headers: vec![(2, vec![1, 2, 3, 4])],
        };
        metadata.check_chunk(0, &[88, 88, 1, 2])?;
        metadata.check_chunk(4, &[3, 4, 88])?;
        assert!(metadata.check_chunk(4, &[3, 9, 88]).is_err());
        Ok(())
    }

    #[test]
    fn feature_wire_preserves_every_integer_word() -> Result<()> {
        let feature = Feature {
            counts: std::array::from_fn(|i| u32::MAX - i as u32),
            totals: std::array::from_fn(|i| (1u64 << 42) + 3 * i as u64),
            mass: (1 << 48) - 1,
        };
        let mut out = Vec::new();
        feature.write(&mut out)?;
        assert_eq!(out.len(), FEATURE_ROW_BYTES);
        for i in 0..ORDERS {
            assert_eq!(uint32(&out[i * 4..i * 4 + 4])?, feature.counts[i]);
            assert_eq!(uint64(&out[44 + i * 8..52 + i * 8])?, feature.totals[i]);
        }
        assert_eq!(uint64(&out[132..])?, feature.mass);
        Ok(())
    }
}
