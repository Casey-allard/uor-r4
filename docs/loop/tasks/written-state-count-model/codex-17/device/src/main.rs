#![forbid(unsafe_code)]
use std::{env, fs, io::Write, path::Path};
use uor_all_starts_finite::Candidates;
use uor_r4_integer::{
    assertion_value::{self, AssertionValue},
    content_selector::BOUNDARY,
    discrete_d11::{spiralcore_orbit_roots, ExactCopyState, ROLE_COPY_STOPS},
    span_selector::{Bank, SpanChoice, CONT, START},
    value_selector::{Decisions, ExactBank},
};

const ABI: u32 = 0x32415355;
const TRACE: u32 = 0x314f5355;
const LIMIT: usize = 24;
const CAP: usize = 256;
const WORDS: usize = 8 + 3 * LIMIT + CAP * 8;
const STEPS: usize = 8;
const WRITER: usize = 8 + 3 * 20;
const OUT: usize = WRITER + STEPS * 8;
const NIL: u32 = u32::MAX;
const NO_RADIUS: u32 = 65535;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone)]
struct Row {
    tape: Vec<u32>,
    classes: [u32; 4096],
    roots: [u32; 4096],
    frozen: AssertionValue,
    fact: u32,
    question: u32,
    bank: [[u32; 8]; CAP],
    used: usize,
    mode: u32,
}

#[derive(Clone, Copy)]
struct Capture {
    source: u32,
    end: u32,
    depth: u32,
    length: u32,
    signature: u32,
    packed: [u32; 10],
}
impl Capture {
    fn slots(&self) -> usize {
        (self.depth + self.length) as usize
    }
    fn shape(&self) -> u32 {
        self.depth | (self.length << 8)
    }

    fn record(&self, label: u32, cost: u32) -> [u32; 8] {
        let mut packed = self.packed;
        // Finite plant only. With input root 0, the first nibble of up to
        // three segments gets root 1 (8 L1) or root 2 (4 L1).
        let mut units = cost / 4;
        for root in &mut packed[..self.slots()] {
            if units == 0 {
                break;
            }
            assert_eq!(*root, 0);
            let step = units.min(2);
            *root = if step == 2 { 1 } else { 2 };
            units -= step;
        }
        assert_eq!(units, 0);
        self.record_with(label, 0, packed)
    }

    fn record_with(&self, label: u32, radius: u32, packed: [u32; 10]) -> [u32; 8] {
        let data: [u32; 5] = core::array::from_fn(|i| packed[i * 2] | (packed[i * 2 + 1] << 16));
        [
            self.shape() | (label << 16),
            self.signature,
            radius,
            data[0],
            data[1],
            data[2],
            data[3],
            data[4],
        ]
    }
}

impl Row {
    fn new(cap: bool) -> Self {
        let mut classes = [0; 4096];
        for id in [42, 10, 11, 13, 15, 45, 46, 47, 48, 49, 50] {
            classes[id] = START;
        }
        classes[14] = CONT;
        let tape = if cap {
            vec![
                0, 55, 42, 10, 11, 13, 15, 14, 45, 46, 47, 48, 201, 49, 50, 42,
            ]
        } else {
            vec![0, 55, 42, 10, 11, 13, 15, 14, 45, 16, 46, 47, 42, 201]
        };
        let question = if cap { 15 } else { 12 };
        let roots = [0; 4096];
        let span = SpanChoice {
            fact_start: Some(2),
            fact_end: Some(2),
            query_start: Some(question),
            query_end: Some(question),
            ..Default::default()
        };
        let bank = Bank {
            classes: classes.to_vec(),
            relations: vec![
                [1, 0, BOUNDARY, BOUNDARY, 0],
                [2, 0, 0, BOUNDARY, 0],
                [3, 0, 0, 0, 0],
            ],
        };
        let exact = ExactBank {
            connectors: vec![[1, 10, BOUNDARY, BOUNDARY]],
        };
        let frozen =
            assertion_value::decide(&tape, &roots, &bank, &exact, span, ROLE_COPY_STOPS).unwrap();
        assert_eq!(frozen.value.eligible_mask & frozen.value.word_start_mask, 7);
        Self {
            tape,
            classes,
            roots,
            frozen,
            fact: 2,
            question,
            bank: [[0; 8]; CAP],
            used: 0,
            mode: 0,
        }
    }
    fn candidate(&self, source: u32) -> Capture {
        let fence = self.frozen.fence.unwrap();
        let end = (source + 1..fence)
            .find(|&at| self.classes[self.tape[at as usize] as usize] != CONT)
            .unwrap_or(fence);
        let mut packed = [0; 10];
        let mut signature = 0;
        for (slot, pos) in (self.fact + 1..end).enumerate() {
            let id = self.tape[pos as usize] as usize;
            packed[slot] = self.roots[id];
            signature |= self.classes[id] << (2 * slot);
        }
        Capture {
            source,
            end,
            depth: source - self.fact - 1,
            length: end - source,
            signature,
            packed,
        }
    }
    fn plant(&mut self, costs: [u32; 3], omit_onset: Option<u32>) {
        for (source, cost) in (4..=6).zip(costs) {
            let candidate = self.candidate(source);
            if Some(source) != omit_onset {
                // Duplicated across two authored fixture family tags.
                for _ in 0..2 {
                    self.bank[self.used] = candidate.record(0, 0);
                    self.used += 1;
                }
            }
            self.bank[self.used] = candidate.record(1, cost);
            self.used += 1;
        }
    }
    fn encode(&self) -> [u32; WORDS] {
        let mut bytes = [0; WORDS];
        bytes[..8].copy_from_slice(&[
            self.tape.len() as u32,
            self.fact,
            self.question,
            self.frozen.fence.unwrap_or(NIL),
            self.frozen.value.eligible_mask,
            self.frozen.value.word_start_mask,
            self.used as u32,
            self.mode,
        ]);
        for (index, &id) in self.tape.iter().enumerate() {
            bytes[8 + index] = id;
            bytes[8 + LIMIT + index] = self.classes[id as usize];
            bytes[8 + 2 * LIMIT + index] = self.roots[id as usize];
        }
        for (dest, record) in bytes[8 + 3 * LIMIT..].chunks_exact_mut(8).zip(&self.bank) {
            dest.copy_from_slice(record);
        }
        bytes
    }
}

fn canonical(packed: &mut [u32], signature: u32, depth: usize) {
    for segment in [0..depth, depth..packed.len()] {
        for class in [START, CONT] {
            let locations: Vec<_> = segment
                .clone()
                .filter(|&at| (signature >> (2 * at)) & 3 == class)
                .collect();
            let mut values: Vec<_> = locations.iter().map(|&at| packed[at]).collect();
            values.sort_unstable();
            for (at, value) in locations.into_iter().zip(values) {
                packed[at] = value;
            }
        }
    }
}

fn set_unordered(row: &mut Row) {
    row.mode = 1;
    for record in &mut row.bank[..row.used] {
        let depth = (record[0] & 255) as usize;
        let count = depth + ((record[0] >> 8) & 255) as usize;
        let mut roots: [u32; 10] = core::array::from_fn(|i| record_root(record, i));
        canonical(&mut roots[..count], record[1], depth);
        for (i, pair) in roots.chunks_exact(2).enumerate() {
            record[3 + i] = pair[0] | (pair[1] << 16);
        }
    }
}

fn bank_valid(bank: &[[u32; 8]; CAP], used: usize, mode: u32) -> bool {
    if mode > 1 || used > CAP || bank[used..].iter().any(|entry| *entry != [0; 8]) {
        return false;
    }
    for (i, entry) in bank[..used].iter().enumerate() {
        let (depth, length) = (entry[0] & 255, (entry[0] >> 8) & 255);
        if !(1..=3).contains(&depth)
            || !(1..=7).contains(&length)
            || entry[0] >> 16 > 1
            || (entry[2] != NO_RADIUS && entry[2] > 1280)
            || entry[1] >> (2 * (depth + length)) != 0
            || (entry[1] >> (2 * depth)) & 3 != START
        {
            return false;
        }
        for j in 0..10 {
            let class = (entry[1] >> (2 * j)) & 3;
            if j < depth + length {
                if class > 2 || (j > depth && class != CONT) {
                    return false;
                }
            } else if record_root(entry, j as usize) != 0 {
                return false;
            }
        }
        if mode == 1 {
            let before: [u32; 10] = core::array::from_fn(|k| record_root(entry, k));
            let mut after = before;
            canonical(
                &mut after[..(depth + length) as usize],
                entry[1],
                depth as usize,
            );
            if before != after {
                return false;
            }
        }
        for previous in &bank[..i] {
            if previous[0] & 65535 != entry[0] & 65535 || previous[1] != entry[1] {
                continue;
            }
            if previous[2] != entry[2]
                || ((previous[0] >> 16) != (entry[0] >> 16) && previous[3..] == entry[3..])
            {
                return false;
            }
        }
    }
    true
}
fn record_root(record: &[u32; 8], slot: usize) -> u32 {
    (record[3 + slot / 2] >> (16 * (slot % 2))) & 65535
}

fn distance(a: u32, b: u32) -> u32 {
    let table = spiralcore_orbit_roots();
    (0..4)
        .map(|block| {
            let x = &table[((a >> (block * 4)) & 15) as usize];
            let y = &table[((b >> (block * 4)) & 15) as usize];
            x.iter()
                .zip(y)
                .map(|(&a, &b)| u32::from(a.abs_diff(b)))
                .sum::<u32>()
        })
        .sum()
}

fn reference(row: &Row) -> [u32; OUT] {
    let mut trace = [NIL; OUT];
    trace[0] = 5;
    trace[1] = row.frozen.value.eligible_mask & row.frozen.value.word_start_mask;
    trace[7] = 0;
    let Ok(view) = Candidates::capture(&row.tape, &row.classes, row.fact, row.question, row.frozen)
    else {
        return trace;
    };
    if !bank_valid(&row.bank, row.used, row.mode) || view.fence.is_none() {
        return trace;
    }
    let mut scores = Vec::new();
    let mut unknown = false;
    for (i, proposal) in view.legacy.iter().enumerate() {
        let Some(proposal) = proposal else {
            continue;
        };
        let mut capture = row.candidate(proposal.source);
        if row.mode == 1 {
            let count = capture.slots();
            canonical(
                &mut capture.packed[..count],
                capture.signature,
                capture.depth as usize,
            );
        }
        let out = &mut trace[8 + i * 20..8 + (i + 1) * 20];
        out[..3].copy_from_slice(&[capture.source, capture.end, 1]);
        if !proposal.writer_reachable {
            continue;
        }
        out[2] = 2;
        out[3] = capture.shape();
        out[4] = capture.signature;
        out[10..10 + capture.slots()].copy_from_slice(&capture.packed[..capture.slots()]);
        let mut hits = [NIL; 2];
        let mut radius = NO_RADIUS;
        for item in row.bank[..row.used]
            .iter()
            .filter(|item| item[0] & 65535 == capture.shape() && item[1] == capture.signature)
        {
            let total = capture.packed[..capture.slots()]
                .iter()
                .enumerate()
                .map(|(i, &query)| distance(query, record_root(item, i)))
                .sum();
            let label = (item[0] >> 16) as usize;
            hits[label] = hits[label].min(total);
            radius = item[2];
        }
        out[5..8].copy_from_slice(&[hits[0], hits[1], radius]);
        if hits.contains(&NIL) || radius == NO_RADIUS || hits[0] > radius {
            unknown = true;
            continue;
        }
        let (num, den) = (hits[1] as i32 - hits[0] as i32, capture.slots() as i32);
        out[2] = 3;
        out[8] = num as u32;
        out[9] = den as u32;
        scores.push((num, den, capture.source));
    }
    let cmp = |x: &(i32, i32, u32), y: &(i32, i32, u32)| (x.0 * y.1).cmp(&(y.0 * x.1));
    scores.sort_by(|a, b| cmp(b, a));
    for ((n, d, _), out) in scores.iter().zip(trace[3..7].chunks_exact_mut(2)) {
        out.copy_from_slice(&[*n as u32, *d as u32]);
    }
    trace[0] = if unknown {
        2
    } else if scores.is_empty() {
        1
    } else if scores[0].0 <= 0 {
        4
    } else if scores.len() > 1 && cmp(&scores[0], &scores[1]).is_eq() {
        3
    } else {
        0
    };
    if trace[0] != 0 {
        return trace;
    }
    let source = scores[0].2;
    trace[2] = source;
    let mut state = ExactCopyState::new(Some(source), row.tape.len() as u32);
    for i in 0..STEPS {
        let step = state
            .write(&row.tape, NIL, ROLE_COPY_STOPS)
            .unwrap()
            .finish_with_eos(true, 1);
        trace[WRITER + i * 8..WRITER + (i + 1) * 8].copy_from_slice(&step.words());
        trace[7] += 1;
        state = step.state;
        if step.release != 0 {
            assert!(step.completed_span());
            break;
        }
    }
    trace
}

fn fixtures() -> Vec<(&'static str, Row, u32, u32, Vec<u32>)> {
    let advance = |cost, omit| {
        let mut row = Row::new(false);
        row.plant(cost, omit);
        row
    };
    let right = advance([4, 8, 8], None);
    let wrong = advance([8, 8, 8], None);
    let tie = advance([8, 12, 20], None);
    let unseen = advance([4, 8, 8], Some(5));
    let mut capped = Row::new(true);
    capped.plant([8, 8, 8], None);
    let mut fence = right.clone();
    fence.frozen.fence = Some(11);
    let mut padding = right.clone();
    padding.bank[255][0] = 1;
    let mut none = right.clone();
    none.frozen.value = Decisions::default();
    let mut ordered = Row::new(false);
    ordered.roots[11] = 2; // candidate 5 bridge: root 0 then root 2
    ordered.roots[13] = 15; // its leading value root is equidistant to 0 and 1
    for source in 4..=6 {
        let captured = ordered.candidate(source);
        let mut onset = captured.packed;
        let mut negative = captured.packed;
        let radius = if source == 5 {
            onset[2] = 0;
            negative.swap(0, 1);
            negative[2] = 1;
            4
        } else {
            negative[0] = 2;
            0
        };
        ordered.bank[ordered.used] = captured.record_with(0, radius, onset);
        ordered.bank[ordered.used + 1] = captured.record_with(0, radius, onset);
        ordered.bank[ordered.used + 2] = captured.record_with(1, radius, negative);
        ordered.used += 3;
    }
    let mut unordered = ordered.clone();
    set_unordered(&mut unordered);
    assert_eq!(ordered.used, unordered.used);
    assert_eq!(ordered.encode().len(), unordered.encode().len());
    vec![
        ("known_right", right, 0, 5, vec![13, 15, 14, 45, 1]),
        ("planted_wrong", wrong, 0, 4, vec![11, 13, 15, 14, 45, 1]),
        ("exact_tie", tie, 3, NIL, vec![]),
        ("unseen_shape", unseen, 2, NIL, vec![]),
        (
            "cap_reject_4",
            capped,
            0,
            5,
            vec![13, 15, 14, 45, 46, 47, 48, 1],
        ),
        ("false_fence", fence, 5, NIL, vec![]),
        ("nonzero_bank_padding", padding, 5, NIL, vec![]),
        ("empty_eligibility", none, 1, NIL, vec![]),
        ("ordered_control", ordered, 0, 5, vec![13, 15, 14, 45, 1]),
        (
            "same_byte_unordered",
            unordered,
            0,
            4,
            vec![11, 13, 15, 14, 45, 1],
        ),
    ]
}

fn save(path: &Path, words: impl IntoIterator<Item = u32>) -> Result<()> {
    let bytes: Vec<_> = words.into_iter().flat_map(u32::to_le_bytes).collect();
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let variants = fixtures();
    let reference: Vec<_> = variants
        .iter()
        .map(|(_, row, _, _, _)| self::reference(row))
        .collect();
    let tx_size = variants.len() as u32;
    for ((name, _, status, source, expected), trace) in variants.iter().zip(&reference) {
        let emitted: Vec<_> = trace[WRITER..]
            .chunks_exact(8)
            .take(trace[7] as usize)
            .map(|s| s[4])
            .collect();
        assert_eq!(
            (trace[0], trace[2], &emitted),
            (*status, *source, expected),
            "{name}"
        );
        if *name == "planted_wrong" {
            assert_ne!(emitted[0], variants[0].4[0]);
        }
        if *name == "cap_reject_4" {
            assert_eq!(&trace[8..11], &[4, 5, 1]);
        }
        if *name == "ordered_control" {
            assert_eq!(&trace[8 + 20 + 5..8 + 20 + 8], &[4, 12, 4]);
        }
        if *name == "same_byte_unordered" {
            assert_eq!(&trace[8 + 20 + 5..8 + 20 + 8], &[4, 4, 4]);
        }
        if *name == "exact_tie" {
            assert_eq!(&trace[3..7], &[8, 2, 12, 3]);
        }
    }
    match args.as_slice() {
        [mode, input, expected] if mode == "generate" => {
            save(
                Path::new(input),
                [ABI, tx_size]
                    .into_iter()
                    .chain(variants.iter().flat_map(|(_, row, _, _, _)| row.encode())),
            )?;
            save(
                Path::new(expected),
                [TRACE, tx_size]
                    .into_iter()
                    .chain(reference.into_iter().flatten()),
            )?;
        }
        [mode, device] if mode == "verify" => {
            let bytes = fs::read(device)?;
            let expected_len = (2 + variants.len() * OUT) * 4;
            if bytes.len() != expected_len {
                return Err("trace length mismatch".into());
            }
            let values: Vec<_> = bytes
                .chunks_exact(4)
                .map(|w| u32::from_le_bytes(w.try_into().unwrap()))
                .collect();
            if values[..2] != [TRACE, tx_size] {
                return Err("trace header mismatch".into());
            }
            for ((name, _, _, _, _), (expected, observed)) in variants
                .iter()
                .zip(reference.iter().zip(values[2..].chunks_exact(OUT)))
            {
                if let Some((offset, (&x, &y))) = expected
                    .iter()
                    .zip(observed)
                    .enumerate()
                    .find(|(_, (a, b))| a != b)
                {
                    return Err(
                        format!("{name} trace word {offset}: expected {x}, device {y}").into(),
                    );
                }
                let emitted: Vec<_> = observed[WRITER..]
                    .chunks_exact(8)
                    .take(observed[7] as usize)
                    .map(|s| s[4])
                    .collect();
                println!(
                    "{name} status={} source={} steps={} ids={emitted:?}",
                    observed[0], observed[2], observed[7]
                );
            }
        }
        _ => return Err("use generate INPUT EXPECTED or verify DEVICE_TRACE".into()),
    }
    Ok(())
}
