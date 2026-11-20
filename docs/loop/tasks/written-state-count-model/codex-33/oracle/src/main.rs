#![forbid(unsafe_code)]

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs;
use std::io::{self, Write};

const BOS: u16 = 0;
const EOS: u16 = 1;
const HIGHEST: usize = 10;
const NO_PREDECESSOR: u16 = 4096;
type Failure = Box<dyn Error>;

struct Dose {
    tokens: usize,
    records: usize,
    sha: &'static str,
    receipt_sha: &'static str,
}

fn validate_records(tokens: &[u16]) -> Result<usize, Failure> {
    let mut previous = EOS;
    let mut records = 0;
    for (ordinal, &token) in tokens.iter().enumerate() {
        if token >= NO_PREDECESSOR || (token == BOS) != (previous == EOS) {
            return Err(format!("invalid token or record boundary at ordinal {ordinal}").into());
        }
        records += usize::from(token == BOS);
        previous = token;
    }
    if records == 0 || previous != EOS {
        return Err("missing complete record at end of token stream".into());
    }
    Ok(records)
}

fn decode_uort(bytes: &[u8]) -> Result<Vec<u16>, Failure> {
    if bytes.len() < 64 || &bytes[..4] != b"UORT" {
        return Err("missing UORT header".into());
    }
    let ordinal_limit = u64::from_le_bytes(bytes[8..16].try_into()?);
    if ordinal_limit > u64::from(u32::MAX)
        || ordinal_limit.checked_mul(2) != Some((bytes.len() - 64) as u64)
    {
        return Err("invalid UORT ordinal bound or payload length".into());
    }
    Ok(bytes[64..]
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect())
}

fn key(context: &[u16]) -> u128 {
    let digits = context.iter().enumerate().fold(0, |word, (index, &id)| {
        word | (u128::from(id) << (120 - 12 * context.len() + 12 * index))
    });
    digits | ((context.len() as u128) << 120)
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct Occurrence {
    context: u128,
    next: u16,
    predecessor: u16,
}

fn occurrences(tokens: &[u16], order: usize) -> Vec<Occurrence> {
    let mut result = Vec::with_capacity(tokens.len());
    for record in tokens.split_inclusive(|&token| token == EOS) {
        for target in (order.max(1))..record.len() {
            let start = target - order;
            let predecessor = start.checked_sub(1).map_or(NO_PREDECESSOR, |at| record[at]);
            result.push(Occurrence {
                context: key(&record[start..target]),
                next: record[target],
                predecessor,
            });
        }
    }
    result.sort_unstable();
    result
}

#[derive(Default)]
struct Context {
    total: u64,
    followers: u32,
    singles: u32,
    doubles: u32,
    others: u32,
}

struct Census {
    order: usize,
    triples: Sha256,
    contexts: Sha256,
    count_of_counts: [u64; 4],
    total: u64,
    triple_rows: u64,
    context_rows: u64,
}

impl Census {
    fn new(order: usize, adjusted: bool) -> Self {
        let (triple_domain, context_domain) = if adjusted {
            (
                "uor-r4/ngram-adjusted-triples/v1\0",
                "uor-r4/ngram-adjusted-contexts/v1\0",
            )
        } else {
            ("uor-r4/ngram-triples/v1\0", "uor-r4/ngram-contexts/v1\0")
        };
        let hash = |domain: &str| {
            let mut sha = Sha256::new();
            sha.update(domain);
            sha.update([order as u8]);
            sha
        };
        Self {
            order,
            triples: hash(triple_domain),
            contexts: hash(context_domain),
            count_of_counts: [0; 4],
            total: 0,
            triple_rows: 0,
            context_rows: 0,
        }
    }

    fn count(&mut self, key: u128, token: u16, count: u32, context: &mut Context) {
        let mut row = [0; 22];
        row[..16].copy_from_slice(&key.to_le_bytes());
        row[16..18].copy_from_slice(&token.to_le_bytes());
        row[18..].copy_from_slice(&count.to_le_bytes());
        self.triples.update(row);
        self.triple_rows += 1;
        if let 1..=4 = count {
            self.count_of_counts[count as usize - 1] += 1;
        }
        context.total += u64::from(count);
        context.followers += 1;
        match count {
            1 => context.singles += 1,
            2 => context.doubles += 1,
            _ => context.others += 1,
        }
    }

    fn context(&mut self, key: u128, context: Context) {
        let mut row = [0; 40];
        row[..16].copy_from_slice(&key.to_le_bytes());
        row[16..24].copy_from_slice(&context.total.to_le_bytes());
        row[24..28].copy_from_slice(&context.followers.to_le_bytes());
        row[28..32].copy_from_slice(&context.singles.to_le_bytes());
        row[32..36].copy_from_slice(&context.doubles.to_le_bytes());
        row[36..].copy_from_slice(&context.others.to_le_bytes());
        self.contexts.update(row);
        self.context_rows += 1;
        self.total += context.total;
    }

    fn finish(self) -> Value {
        json!({
            "order": self.order, "targets": self.total,
            "grams": self.triple_rows, "contexts": self.context_rows,
            "count_of_counts": self.count_of_counts,
            "triple_sha256": format!("{:x}", self.triples.finalize()),
            "context_sha256": format!("{:x}", self.contexts.finalize())
        })
    }
}

fn adjusted_count(gram: &[Occurrence], order: usize) -> Result<u32, Failure> {
    if order == HIGHEST || gram[0].predecessor == NO_PREDECESSOR {
        return Ok(u32::try_from(gram.len())?);
    }
    let mut distinct = 1;
    for pair in gram.windows(2) {
        distinct += u32::from(pair[0].predecessor != pair[1].predecessor);
    }
    if gram[gram.len() - 1].predecessor == NO_PREDECESSOR {
        return Err("mixed BOS-start and interior occurrences for one gram".into());
    }
    Ok(distinct)
}

fn count(tokens: &[u16], order: usize) -> Result<(Value, Value), Failure> {
    let rows = occurrences(tokens, order);
    let mut raw = Census::new(order, false);
    let mut adjusted = Census::new(order, true);
    for context in rows.chunk_by(|a, b| a.context == b.context) {
        let mut raw_context = Context::default();
        let mut adjusted_context = Context::default();
        let key = context[0].context;
        for gram in context.chunk_by(|a, b| a.next == b.next) {
            let token = gram[0].next;
            raw.count(key, token, u32::try_from(gram.len())?, &mut raw_context);
            adjusted.count(
                key,
                token,
                adjusted_count(gram, order)?,
                &mut adjusted_context,
            );
        }
        raw.context(key, raw_context);
        adjusted.context(key, adjusted_context);
    }
    Ok((raw.finish(), adjusted.finish()))
}

fn verify_row(expected: &Value, observed: &Value, domain: &str) -> Result<(), Failure> {
    for field in [
        "order",
        "targets",
        "grams",
        "contexts",
        "count_of_counts",
        "triple_sha256",
        "context_sha256",
    ] {
        if expected[field].is_null() || observed[field] != expected[field] {
            return Err(format!(
                "first divergent order {} {domain}.{field}, expected {}, observed {}",
                observed["order"], expected[field], observed[field]
            )
            .into());
        }
    }
    Ok(())
}

fn main() -> Result<(), Failure> {
    let arguments: Vec<_> = std::env::args().collect();
    let [_, name, input, expected] = arguments.as_slice() else {
        return Err("usage: d33-count-oracle c128|c1 TRAIN.uort GPU-RECEIPT.json".into());
    };
    let dose = match name.as_str() {
        "c128" => Dose {
            tokens: 111_671,
            records: 128,
            sha: "1466d3790e1c53bc4ac3982712f669967cce7b3ad1d7e0e34259568154d19cb6",
            receipt_sha: "af7d8c9ee99b0fda69ec4c258051928ae9103ec4df7597d6212a94235deaa5e2",
        },
        "c1" => Dose {
            tokens: 20_754_301,
            records: 5_402,
            sha: "8d94e88ac31305ad6973734181a9727f520da46a5ecc263b944247ceedd07030",
            receipt_sha: "34e3cd6e9d4ae35a78284a36911547f4ca4109b52e0e85e09fb3af54630f3df7",
        },
        _ => return Err("unregistered dose".into()),
    };
    let bytes = fs::read(input)?;
    if format!("{:x}", Sha256::digest(&bytes)) != dose.sha {
        return Err("TRAIN input SHA mismatch".into());
    }
    let tokens = decode_uort(&bytes)?;
    drop(bytes);
    if tokens.len() != dose.tokens || validate_records(&tokens)? != dose.records {
        return Err("registered TRAIN token or record count mismatch".into());
    }

    let receipt = fs::read(expected)?;
    if format!("{:x}", Sha256::digest(&receipt)) != dose.receipt_sha {
        return Err("GPU receipt SHA mismatch".into());
    }
    let expected: Value = serde_json::from_slice(&receipt)?;
    let counts = expected.get("counts").unwrap_or(&expected);
    for domain in ["raw", "adjusted"] {
        if counts[domain].as_array().map(Vec::len) != Some(HIGHEST + 1) {
            return Err(format!("wrong number of registered {domain} orders").into());
        }
    }
    let stdout = io::stdout();
    let mut output = stdout.lock();
    writeln!(
        output,
        "{}",
        json!({"dose": name, "train_sha256": dose.sha,
        "gpu_receipt_sha256": dose.receipt_sha, "tokens": dose.tokens, "records": dose.records})
    )?;
    for order in 0..=HIGHEST {
        let (raw, adjusted) = count(&tokens, order)?;
        writeln!(output, "{}", json!({"raw": raw, "adjusted": adjusted}))?;
        output.flush()?;
        verify_row(&counts["raw"][order], &raw, "raw")?;
        verify_row(&counts["adjusted"][order], &adjusted, "adjusted")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_predecessor_census_and_domain_controls() -> Result<(), Failure> {
        let tokens = [0, 7, 3, 2, 1, 0, 7, 3, 2, 1, 0, 8, 3, 2, 1];
        assert_eq!(validate_records(&tokens)?, 3);
        let rows = occurrences(&tokens, 1);
        let gram = |context, next| {
            rows.iter()
                .copied()
                .filter(move |row| row.context == key(&[context]) && row.next == next)
                .collect::<Vec<_>>()
        };
        assert_eq!(gram(3, 2).len(), 3);
        assert_eq!(adjusted_count(&gram(3, 2), 1)?, 2);
        assert_eq!(adjusted_count(&gram(BOS, 7), 1)?, 2);
        assert!(gram(EOS, BOS).is_empty());
        assert!(gram(EOS, 7).is_empty());
        let (raw, adjusted) = count(&tokens, 0)?;
        assert_eq!(raw["targets"], 12);
        assert_eq!(adjusted["targets"], 6);
        assert_ne!(raw["triple_sha256"], adjusted["triple_sha256"]);
        let mut corrupted = raw.clone();
        corrupted["triple_sha256"] = json!("altered");
        assert!(verify_row(&raw, &corrupted, "raw").is_err());
        let mut planted = Census::new(1, false);
        let mut original = Census::new(1, false);
        planted.count(key(&[3]), 2, 4, &mut Context::default());
        original.count(key(&[3]), 2, 3, &mut Context::default());
        assert_ne!(
            planted.finish()["triple_sha256"],
            original.finish()["triple_sha256"]
        );
        Ok(())
    }

    #[test]
    fn top_order_preserves_words_in_distinct_domains() -> Result<(), Failure> {
        let mut tokens = Vec::new();
        for predecessor in [13, 13, 14] {
            tokens.extend([BOS, predecessor]);
            tokens.extend(2..=12);
            tokens.push(EOS);
        }
        assert_eq!(validate_records(&tokens)?, 3);
        const KEY: u128 = 0x0a00_b00_a00_900_800_700_600_500_400_300_2;
        assert_eq!(key(&(2..=11).collect::<Vec<_>>()), KEY);
        let rows = occurrences(&tokens, HIGHEST);
        let top = rows
            .iter()
            .copied()
            .filter(|row| row.context == KEY && row.next == 12)
            .collect::<Vec<_>>();
        assert_eq!(
            top.iter().map(|row| row.predecessor).collect::<Vec<_>>(),
            [13, 13, 14]
        );
        assert_eq!(adjusted_count(&top, HIGHEST)?, 3);
        let (raw, adjusted) = count(&tokens, HIGHEST)?;
        for field in ["targets", "grams", "contexts", "count_of_counts"] {
            assert_eq!(raw[field], adjusted[field]);
        }
        for field in ["triple_sha256", "context_sha256"] {
            assert_ne!(raw[field], adjusted[field]);
        }
        Ok(())
    }

    #[test]
    fn malformed_inputs_fail_closed() {
        for tokens in [
            &[][..],
            &[0, 2][..],
            &[2, 1],
            &[0, 4096, 1],
            &[0, 0, 1],
            &[0, 1, 2, 1],
        ] {
            assert!(validate_records(tokens).is_err());
        }
        let mut bytes = [0; 64];
        bytes[..4].copy_from_slice(b"UORT");
        bytes[8..16].copy_from_slice(&u64::MAX.to_le_bytes());
        assert!(decode_uort(&bytes).is_err());
        assert!(decode_uort(&bytes[..63]).is_err());
    }
}
