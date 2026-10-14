// Data statistic only (no model): for each eligible DEV target under the corpus
// window policy (response-masked targets, within-document predecessor, sliding
// prefix of at most `context` tokens, documents start at BOS id 0), count how
// many targets are recoverable by a copy-successor (induction) read:
//   the query token q = tokens[p-1] occurred earlier in the window at some
//   position j, and tokens[j+1] == tokens[p] (the target).
// Reports: eligible targets; targets whose query token recurs in the window;
// most-recent-occurrence successor == target; any-occurrence successor == target;
// target token itself present anywhere in the window (copy ceiling).
use std::env;
use std::fs;
fn main() {
    let a: Vec<String> = env::args().collect();
    if a.len() != 4 {
        eprintln!("usage: induction_stat <tokens.u16> <mask.u8> <context>");
        std::process::exit(2);
    }
    let bytes = fs::read(&a[1]).expect("tokens");
    let mask = fs::read(&a[2]).expect("mask");
    let context: usize = a[3].parse().expect("context");
    assert!(bytes.len() >= 64 && &bytes[..4] == b"UORT", "UORT header");
    let count = u64::from_le_bytes(bytes[8..16].try_into().unwrap()) as usize;
    let vocab = u32::from_le_bytes(bytes[16..20].try_into().unwrap());
    assert_eq!(bytes.len(), 64 + 2 * count, "payload length");
    assert_eq!(mask.len(), count, "mask length");
    let tokens: Vec<u32> = bytes[64..]
        .chunks_exact(2)
        .map(|w| u32::from(u16::from_le_bytes([w[0], w[1]])))
        .collect();
    let bos = 0u32;
    assert_eq!(tokens[0], bos, "first token is BOS");
    let documents: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter(|(_, &t)| t == bos)
        .map(|(p, _)| p)
        .collect();
    let eligible: Vec<usize> = (0..count)
        .filter(|&p| p > 0 && tokens[p] != bos && mask[p] == 1)
        .collect();
    let (mut recur, mut recent_hit, mut any_hit, mut copy_ceiling) =
        (0usize, 0usize, 0usize, 0usize);
    let mut recent_hit_by_distance = [0usize; 4]; // disjoint distance buckets: 1-8, 9-32, 33-96, 97-192
    for &p in &eligible {
        let doc = documents.partition_point(|&s| s <= p) - 1;
        let start = p.saturating_sub(context).max(documents[doc]);
        assert!(start < p);
        let query = tokens[p - 1];
        let target = tokens[p];
        // window positions start..p-1 are candidates j with successor j+1 <= p-1 (within the prefix)
        let mut most_recent: Option<usize> = None;
        let mut any = false;
        let mut present = false;
        for j in start..p - 1 {
            if tokens[j] == target {
                present = true;
            }
            if tokens[j] == query {
                most_recent = Some(j);
                if tokens[j + 1] == target {
                    any = true;
                }
            }
        }
        if tokens[p - 1] == target {
            present = true;
        }
        if present {
            copy_ceiling += 1;
        }
        if let Some(j) = most_recent {
            recur += 1;
            if tokens[j + 1] == target {
                recent_hit += 1;
                let d = (p - 1) - j;
                let b = if d <= 8 {
                    0
                } else if d <= 32 {
                    1
                } else if d <= 96 {
                    2
                } else {
                    3
                };
                recent_hit_by_distance[b] += 1;
            }
        }
        if any {
            any_hit += 1;
        }
    }
    println!(
        "scope=DATA_STATISTIC_ONLY_NO_MODEL vocabulary={} tokens={} documents={} context={}",
        vocab,
        count,
        documents.len(),
        context
    );
    println!("eligible_targets={}", eligible.len());
    println!("query_token_recurs_in_window={}", recur);
    println!(
        "most_recent_occurrence_successor_equals_target={} by_distance(1-8,9-32,33-96,97-192)={:?}",
        recent_hit, recent_hit_by_distance
    );
    println!("any_occurrence_successor_equals_target={}", any_hit);
    println!(
        "target_token_present_in_window(copy_ceiling)={}",
        copy_ceiling
    );
}
