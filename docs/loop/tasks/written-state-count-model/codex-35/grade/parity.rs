#![forbid(unsafe_code)]
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type Span = [usize; 2];
fn need(ok: bool, why: &str) -> Result<()> {
    if !ok {
        return Err(why.into());
    }
    Ok(())
}
fn read<T: serde::de::DeserializeOwned>(p: &Path) -> Result<T> {
    Ok(serde_json::from_reader(fs::File::open(p)?)?)
}

#[derive(Deserialize)]
struct Index {
    packet_row: usize,
    row: u32,
    fact: Option<Span>,
    candidates: Vec<Span>,
}
#[derive(Deserialize, PartialEq, Eq)]
struct Range {
    start: usize,
    end: usize,
}
#[derive(Deserialize, PartialEq, Eq, Debug, Clone)]
struct Pair {
    candidate: usize,
    rival: usize,
    nearest_forward: u64,
    nearest_reverse: u64,
    margin: i64,
}
#[derive(Deserialize)]
struct Device {
    packet_row: usize,
    winner_forward: Option<usize>,
    winner_reversed: Option<usize>,
    candidates: Vec<Range>,
    pairs: Vec<Pair>,
}
#[derive(Deserialize)]
struct NativeRank {
    fact_span: Span,
    selection: Selection,
    eligible_sources: Vec<usize>,
    orientation: String,
    pairs: Vec<Pair>,
}
#[derive(Deserialize)]
struct Selection {
    source: Option<usize>,
}
#[derive(Deserialize)]
struct Candidate {
    span: Range,
}
#[derive(Deserialize)]
struct NativePool {
    fact_span: Span,
    candidates: Vec<Candidate>,
}
#[derive(Deserialize)]
struct Replay {
    #[serde(default)]
    pair_rank_sources: Vec<NativeRank>,
    #[serde(default)]
    pair_rank_candidates: Vec<NativePool>,
}
#[derive(Deserialize)]
struct Native {
    role_replay: Replay,
    selected: Selection,
}

fn run() -> Result<Value> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [index, decoded, forward, reverse] = args.as_slice() else {
        return Err("usage: d35-parity PACKET_INDEX DECODED_DEVICE FORWARD_CAPTURE_DIR REVERSED_CAPTURE_DIR".into());
    };
    let indexes: Vec<Index> = read(Path::new(index))?;
    let device: Vec<Device> = read(Path::new(decoded))?;
    need(
        !indexes.is_empty() && indexes.len() == device.len(),
        "device and index row counts differ",
    )?;
    let mut rows = BTreeMap::<u32, Vec<(&Index, &Device)>>::new();
    let mut pair_count = 0;
    for (n, (i, d)) in indexes.iter().zip(&device).enumerate() {
        need(
            n == i.packet_row && n == d.packet_row,
            "device and packet order differ",
        )?;
        need(
            i.candidates
                == d.candidates
                    .iter()
                    .map(|r| [r.start, r.end])
                    .collect::<Vec<_>>(),
            "device span mismatch",
        )?;
        pair_count += d.pairs.len();
        rows.entry(i.row).or_default().push((i, d));
    }
    for (folder, reversed) in [(forward, false), (reverse, true)] {
        let path = Path::new(folder);
        let mut captured = BTreeSet::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?.path();
            if entry.extension().is_some_and(|s| s == "json") {
                captured.insert(
                    entry
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .ok_or("filename")?
                        .parse::<u32>()?,
                );
            }
        }
        need(
            rows.keys().copied().collect::<BTreeSet<_>>() == captured,
            "device and ordinary capture prompt coverage differ",
        )?;
        for (&row, entries) in &rows {
            let native: Native = read(&path.join(format!("{row}.json")))?;
            let ranks = &native.role_replay.pair_rank_sources;
            let pools = &native.role_replay.pair_rank_candidates;
            let facts: BTreeSet<_> = entries.iter().filter_map(|(i, _)| i.fact).collect();
            need(
                ranks.len() == facts.len()
                    && ranks.iter().map(|r| r.fact_span).collect::<BTreeSet<_>>() == facts,
                "native assertion rank coverage differs",
            )?;
            need(
                pools.len() == facts.len()
                    && pools.iter().map(|r| r.fact_span).collect::<BTreeSet<_>>() == facts,
                "native candidate projection coverage differs",
            )?;
            for (i, d) in entries {
                let winner = if reversed {
                    d.winner_reversed
                } else {
                    d.winner_forward
                };
                let Some(fact) = i.fact else {
                    need(
                        entries.len() == 1
                            && ranks.is_empty()
                            && i.candidates.is_empty()
                            && winner.is_none()
                            && native.selected.source.is_none(),
                        "unexpected nonempty unmatched fact",
                    )?;
                    continue;
                };
                let rank = ranks
                    .iter()
                    .find(|r| r.fact_span == fact)
                    .ok_or("rank missing")?;
                let pool = pools
                    .iter()
                    .find(|r| r.fact_span == fact)
                    .ok_or("candidate pool missing")?;
                let starts: Vec<_> = i.candidates.iter().map(|c| c[0]).collect();
                need(
                    rank.eligible_sources == starts && rank.selection.source == winner,
                    "native winner or eligible addresses differ",
                )?;
                let spans: Vec<_> = pool
                    .candidates
                    .iter()
                    .filter(|c| starts.contains(&c.span.start))
                    .map(|c| [c.span.start, c.span.end])
                    .collect();
                need(spans == i.candidates, "native eligible span ends differ")?;
                need(
                    rank.orientation == if reversed { "reversed" } else { "forward" },
                    "native orientation differs",
                )?;
                need(
                    rank.pairs.len() == d.pairs.len(),
                    "native pair count differs",
                )?;
                for (native, dev) in rank.pairs.iter().zip(&d.pairs) {
                    let mut expected = dev.clone();
                    if reversed {
                        expected.margin = expected.margin.checked_neg().ok_or("margin overflow")?
                    }
                    need(
                        native == &expected,
                        "native pair addresses, distances or margin differ",
                    )?;
                }
            }
        }
    }
    Ok(
        json!({"prompts":rows.len(),"packet_rows":indexes.len(),"ordered_pairs_each_orientation":pair_count,"orientations":2}),
    )
}
fn main() -> Result<()> {
    println!("{}", run()?);
    Ok(())
}
