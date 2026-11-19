// Verify an ordinary client trace against the packed retained rows and write
// GPU replay inputs. Full-support masses are derived from the selected rows.
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

const VOCABULARY: u16 = 4096;
const ROW_BYTES: u64 = 20;
const Q32: u64 = 1 << 32;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn invalid(description: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, description)
}

fn row(file: &mut File, index: u64) -> Result<(u128, u16, u64)> {
    let mut bytes = [0; ROW_BYTES as usize];
    file.seek(SeekFrom::Start(index * ROW_BYTES))?;
    file.read_exact(&mut bytes)?;
    let key = u128::from_le_bytes(bytes[..16].try_into()?);
    let token = u16::from_le_bytes(bytes[16..18].try_into()?);
    if !(1..VOCABULARY).contains(&token) || !(1..=31).contains(&bytes[18]) || bytes[19] != 0 {
        return Err(invalid("invalid retained row").into());
    }
    Ok((key, token, 1u64 << bytes[18]))
}

fn retained(
    file: &mut File,
    row_count: u64,
    prefix: &[u16],
) -> Result<(Option<u128>, Vec<(u16, u64)>)> {
    let boundary = prefix
        .iter()
        .rposition(|&token| token <= 1)
        .map_or(0, |i| i + usize::from(prefix[i] == 1));
    let causal = &prefix[boundary.max(prefix.len().saturating_sub(10))..];
    for order in (0..=causal.len()).rev() {
        let order_word = (order as u128) << 120;
        let key = causal[causal.len() - order..]
            .iter()
            .rev()
            .enumerate()
            .fold(order_word, |word, (slot, &token)| {
                word | (u128::from(token) << (108 - 12 * slot))
            });
        let mut low = 0;
        let mut high = row_count;
        while low < high {
            let midpoint = low + (high - low) / 2;
            if row(file, midpoint)?.0 < key {
                low = midpoint + 1;
            } else {
                high = midpoint;
            }
        }
        let mut reads = Vec::new();
        for index in low..row_count {
            let entry = row(file, index)?;
            if entry.0 != key {
                break;
            }
            reads.push((entry.1, entry.2));
        }
        if !reads.is_empty() {
            return Ok((Some(key), reads));
        }
    }
    Ok((None, Vec::new()))
}

fn masses(reads: &[(u16, u64)]) -> Vec<u64> {
    let uniform = if reads.is_empty() { Q32 } else { Q32 / 32 };
    let legal = u64::from(VOCABULARY - 1);
    let mut mass = vec![0; usize::from(VOCABULARY)];
    for token in 1..VOCABULARY {
        mass[usize::from(token)] = uniform / legal + u64::from(u64::from(token) <= uniform % legal);
    }
    let total = reads.iter().map(|&(_, count)| count).sum::<u64>();
    let mut cumulative = 0;
    let mut prior = 0;
    for &(token, count) in reads {
        cumulative += count;
        let boundary = u128::from(Q32 - uniform) * u128::from(cumulative) / u128::from(total);
        mass[usize::from(token)] += (boundary - prior) as u64;
        prior = boundary;
    }
    mass
}

fn pairs(field: &str) -> Result<Vec<(u16, u64)>> {
    field
        .split_ascii_whitespace()
        .map(|pair| {
            let (token, value) = pair
                .split_once(':')
                .ok_or_else(|| invalid("expected TOKEN:VALUE"))?;
            Ok((token.parse()?, value.parse()?))
        })
        .collect()
}

fn json_pairs(pairs: &[(u16, u64)]) -> String {
    pairs
        .iter()
        .map(|&(token, value)| format!("[{token},{value}]"))
        .collect::<Vec<_>>()
        .join(",")
}

fn main() -> Result<()> {
    let args = env::args_os().collect::<Vec<_>>();
    if args.len() != 4 {
        return Err(
            invalid("usage: actual-oracle rows queries-out oracle-out < client.tsv").into(),
        );
    }
    let mut table = File::open(&args[1])?;
    let bytes = table.metadata()?.len();
    if bytes % ROW_BYTES != 0 {
        return Err(invalid("partial retained row").into());
    }
    let mut queries = BufWriter::new(File::create(Path::new(&args[2]))?);
    let mut oracle = BufWriter::new(File::create(Path::new(&args[3]))?);
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let header = lines.next().ok_or_else(|| invalid("missing prefix"))??;
    let mut header = header.split_ascii_whitespace();
    let limit = header
        .next()
        .ok_or_else(|| invalid("missing context limit"))?
        .parse::<usize>()?;
    let mut prefix = header
        .map(str::parse::<u16>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if limit < 10 || prefix.len() > limit || prefix.iter().any(|&token| token >= VOCABULARY) {
        return Err(invalid("invalid original context or token ID").into());
    }

    writeln!(oracle, "[")?;
    let mut steps = 0;
    for line in lines {
        let line = line?;
        let fields = line.split('\t').collect::<Vec<_>>();
        let [order, key, top1, winner_mass, native_reads, native_top] = fields.as_slice() else {
            return Err(invalid("invalid native step fields").into());
        };
        let winner = top1.parse::<u16>()?;
        let (derived_key, reads) = retained(&mut table, bytes / ROW_BYTES, &prefix)?;
        let match_hex = derived_key.map_or_else(|| "-".into(), |word| format!("{word:032x}"));
        let derived_order = derived_key.map_or(0, |word| (word >> 120) as u8);
        let mass = masses(&reads);
        let mut top = (1..VOCABULARY)
            .map(|id| (id, mass[usize::from(id)]))
            .collect::<Vec<_>>();
        top.sort_unstable_by_key(|&(id, value)| (std::cmp::Reverse(value), id));
        top.truncate(8);
        if order.parse::<u8>()? != derived_order
            || *key != match_hex
            || pairs(native_reads)? != reads
            || pairs(native_top)? != top
            || top[0] != (winner, winner_mass.parse()?)
            || mass.iter().sum::<u64>() != Q32
        {
            return Err(invalid(&format!("native trace disagreement at step {steps}")).into());
        }

        for (i, token) in prefix.iter().enumerate() {
            write!(queries, "{}{token}", if i == 0 { "" } else { " " })?;
        }
        writeln!(queries)?;
        let key_json = derived_key.map_or_else(|| "null".into(), |_| format!("\"{match_hex}\""));
        if steps > 0 {
            writeln!(oracle, ",")?;
        }
        write!(
            oracle,
            "{{\"order\":{derived_order},\"key\":{key_json},\"reads\":[{}],\"top1\":{winner},\"top8\":[{}],\"mass\":{:?}}}",
            json_pairs(&reads),
            json_pairs(&top),
            mass
        )?;
        steps += 1;
        if prefix.len() == limit {
            prefix.remove(0);
        }
        prefix.push(winner);
    }
    if steps == 0 {
        return Err(invalid("trace contained no steps").into());
    }
    writeln!(oracle, "\n]")?;
    queries.flush()?;
    oracle.flush()?;
    eprintln!("verified {steps} native steps against retained rows and derived exact full support");
    Ok(())
}
