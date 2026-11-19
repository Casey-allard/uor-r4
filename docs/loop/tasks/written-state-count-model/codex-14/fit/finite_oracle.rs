// Synthetic fixture and arithmetic oracle for kn_ceiling_fit.cu.
#[allow(dead_code)]
#[path = "../../../../../../crates/uor-r4-integer/src/lawful_likelihood.rs"]
mod likelihood;

use likelihood::bits_q12;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

const LANES: usize = 13;
const BUCKETS: usize = 176;
const CELLS: usize = LANES * BUCKETS;
const W: u64 = 1 << 32;
const M: u64 = 1 << 48;
const UNIFORM: u64 = M / 4096;

#[derive(Clone, Copy)]
struct Row {
    bucket: usize,
    novel: bool,
    bytes: u8,
    parity: bool,
    masses: [u64; LANES],
}

impl Row {
    fn write(self, out: &mut impl Write) -> std::io::Result<()> {
        out.write_all(&(self.bucket as u16).to_le_bytes())?;
        out.write_all(&[u8::from(self.novel), self.bytes, u8::from(self.parity)])?;
        words(out, &self.masses)
    }
}

fn present(bucket: usize, expert: usize, restricted: bool) -> bool {
    expert == 0 || expert == LANES - 1 || (!restricted && expert <= bucket / 16 + 1)
}

fn words(out: &mut impl Write, values: &[u64]) -> std::io::Result<()> {
    for value in values {
        out.write_all(&value.to_le_bytes())?;
    }
    Ok(())
}

fn make_rows() -> Vec<Row> {
    let mut rows = Vec::new();
    let mut state = 0x4bc5_9187_caf0_532d_u64;
    for exponent in 0..=48 {
        let power = 1u64 << exponent;
        // All finite powers and all in-range adjacent masses. The upper edge
        // also supplies products and sums with bit 80 set.
        for mass in [power.saturating_sub(1), power, power + 1]
            .into_iter()
            .filter(|m| (1..=M).contains(m))
        {
            let bucket = 16 * (rows.len() % 11) + rows.len() % 16;
            let mut masses = [0; LANES];
            masses[0] = mass;
            masses[LANES - 1] = UNIFORM;
            for (expert, value) in masses.iter_mut().enumerate().take(LANES - 1).skip(1) {
                if present(bucket, expert, false) {
                    state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                    *value = match (rows.len() + expert) % 4 {
                        0 => 0,
                        1 => M,
                        2 => mass,
                        _ => state >> 16,
                    };
                }
            }
            rows.push(Row {
                bucket,
                novel: rows.len() % 3 != 0,
                bytes: rows.len() as u8,
                parity: rows.len() % 2 != 0,
                masses,
            });
        }
    }

    // Both parities contain multiple thread blocks in the same bucket.
    // Include distinct partial blocks, asymmetric fold objectives, and
    // completely unsupported active ML lanes whose floors must be restored.
    for index in 0..1057 {
        let mut masses = [0; LANES];
        masses[0] = if index % 3 == 0 { 1 } else { M };
        masses[1] = if index % 2 == 0 { M } else { 0 };
        masses[LANES - 1] = UNIFORM;
        rows.push(Row {
            bucket: 175,
            novel: index % 4 > 0,
            bytes: (255 - index % 256) as u8,
            parity: index % 2 != 0,
            masses,
        });
    }
    rows
}

fn initial(restricted: bool, near_kn: bool) -> [u64; CELLS] {
    let mut all = [0u64; CELLS];
    for (b, row) in all.chunks_exact_mut(LANES).enumerate() {
        let live: Vec<_> = (0..LANES).filter(|e| present(b, *e, restricted)).collect();
        let share = if near_kn { 1 } else { W / live.len() as u64 };
        for e in live {
            row[e] = share;
        }
        row[0] += W - row.iter().sum::<u64>();
    }
    all
}

fn top(row: &[u64]) -> usize {
    (0..LANES)
        .min_by_key(|&e| (std::cmp::Reverse(row[e]), e))
        .unwrap()
}

fn update(weights: &mut [u64; CELLS], responses: &[u64; CELLS], restricted: bool) {
    for (b, (row, resp)) in weights
        .chunks_exact_mut(LANES)
        .zip(responses.chunks_exact(LANES))
        .enumerate()
    {
        let total = resp.iter().map(|r| *r as u128).sum::<u128>();
        if total == 0 {
            continue;
        }
        for (w, r) in row.iter_mut().zip(resp) {
            *w = ((u128::from(*r) << 32) / total) as u64;
        }
        row[top(row)] += W - row.iter().sum::<u64>();
        for e in 0..LANES {
            if present(b, e, restricted) && row[e] == 0 {
                row[top(row)] -= 1;
                row[e] = 1;
            }
        }
    }
}

fn tally_into(tally: &mut [u64; 6], row: &Row, bits: u64) {
    tally[0] += 1;
    tally[1] += u64::from(row.bytes);
    tally[2] += bits;
    if row.novel {
        tally[3] += 1;
        tally[4] += u64::from(row.bytes);
        tally[5] += bits;
    }
}

fn evaluate(
    rows: &[Row],
    weights: &[u64; CELLS],
    fold: Option<bool>,
) -> Result<([u64; 6], [u64; CELLS]), Box<dyn Error>> {
    let mut tally = [0; 6];
    let mut response = [0; CELLS];
    for row in rows.iter().filter(|r| fold.is_none_or(|p| r.parity == p)) {
        let start = row.bucket * LANES;
        let terms: [u128; LANES] =
            std::array::from_fn(|e| u128::from(weights[start + e]) * u128::from(row.masses[e]));
        let sum = terms.iter().sum::<u128>();
        for (e, term) in terms.into_iter().enumerate() {
            response[start + e] += ((term << 32) / sum) as u64;
        }
        tally_into(&mut tally, row, bits_q12(((sum >> 32) as u64).max(1))?);
    }
    Ok((tally, response))
}

fn fit(
    out: &mut impl Write,
    rows: &[Row],
    restricted: bool,
    near_kn: bool,
    fold: Option<bool>,
    primary: bool,
) -> Result<[u64; 6], Box<dyn Error>> {
    let mut weights = initial(restricted, near_kn);
    words(out, &weights)?;
    let mut trace = Vec::<u64>::new();
    let mut dwell = [0; CELLS];
    for pass in 0..=if primary { 64 } else { 24 } {
        let (tally, response) = evaluate(rows, &weights, fold)?;
        words(out, &tally)?;
        trace.push(tally[2]);
        let tolerance = tally[0] * 4096 / 10_000;
        let plateau = pass >= 24
            && pass % 8 == 0
            && trace[pass].abs_diff(trace[pass - 8]) <= tolerance
            && trace[pass - 8..]
                .windows(2)
                .all(|w| w[0].abs_diff(w[1]) <= tolerance);
        if (primary && (plateau || pass == 64)) || (!primary && pass == 24) {
            let (score, _) = evaluate(rows, &weights, fold.map(|p| !p))?;
            words(out, &score)?;
            words(out, &dwell)?;
            return Ok(score);
        }
        words(out, &response)?;
        update(&mut weights, &response, restricted);
        for (count, weight) in dwell.iter_mut().zip(weights) {
            *count += u64::from(weight == 1);
        }
        words(out, &weights)?;
    }
    Err("oracle did not reach its bounded endpoint".into())
}

fn write_expected(path: &Path, rows: &[Row]) -> Result<(), Box<dyn Error>> {
    let mut out = BufWriter::new(File::create_new(path)?);
    let mut grouped = rows.to_vec();
    grouped.sort_by_key(|row| (row.parity, row.bucket));
    let mut kn = [0; 6];
    for row in &grouped {
        let bits = bits_q12(row.masses[0])?;
        words(&mut out, &[bits])?;
        tally_into(&mut kn, row, bits);
    }
    words(&mut out, &kn)?;
    fit(&mut out, rows, false, true, None, true)?;
    fit(&mut out, rows, false, false, None, false)?;
    fit(&mut out, rows, true, true, None, false)?;
    fit(&mut out, rows, true, false, None, false)?;
    let even = fit(&mut out, rows, false, false, Some(false), false)?;
    let odd = fit(&mut out, rows, false, false, Some(true), false)?;
    let split: [u64; 6] = std::array::from_fn(|field| even[field] + odd[field]);
    words(&mut out, &split)?;
    out.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("Usage: finite-oracle NEW_FIXTURE_DIR")?;
    let path = Path::new(&path);
    fs::create_dir(path)?;
    let rows = make_rows();
    let mut frame = Vec::new();
    for row in &rows {
        row.write(&mut frame)?;
    }
    fs::write(path.join("frame.bin"), &frame)?;
    fs::write(path.join("truncated.bin"), &frame[..frame.len() - 1])?;
    let mut invalid = frame.clone();
    invalid[2] = 2;
    fs::write(path.join("bad-class.bin"), invalid)?;
    let mut invalid = frame.clone();
    invalid[5..13].copy_from_slice(&(M + 1).to_le_bytes());
    fs::write(path.join("bad-mass.bin"), invalid)?;
    let mut invalid = frame;
    invalid[5 + 2 * 8] = 1;
    fs::write(path.join("absent-lane.bin"), invalid)?;
    fs::write(path.join("targets"), rows.len().to_string())?;
    write_expected(&path.join("oracle.bin"), &rows)
}
