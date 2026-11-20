#![forbid(unsafe_code)]

use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, env, error::Error, fs, path::Path};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type RoleWords = [u32; 8];

const MODES: [(u32, &str); 4] = [
    (4, "historical-inherited"),
    (5, "full-inherited"),
    (6, "historical-repeated"),
    (7, "full-repeated"),
];
const ROWS: usize = 22;
const WIDTH: usize = 192;
const PACKET_MAGIC: u32 = 0x3141524f;
const SITES: usize = 4096;

struct PacketRow {
    id: u32,
    prefix_hash: String,
}

#[derive(Deserialize)]
struct Ordinary {
    prompt_ids: Vec<u32>,
    generated_ids: Vec<u32>,
    paired: Choice,
    selected: Selection,
    role_replay: Replay,
}

#[derive(Deserialize)]
struct Selection {
    source: Option<u32>,
}

#[derive(Deserialize)]
struct Choice {
    role_words: RoleWords,
}

#[derive(Deserialize)]
struct Replay {
    content: [u32; 2],
}

#[derive(Deserialize)]
struct Grade {
    arm: String,
    row: u32,
    stratum: String,
    requested: Option<Endpoint>,
    excluded: Option<Endpoint>,
    incidental_chosen: bool,
    requested_correct: bool,
    excluded_correct: bool,
    joint_correct: bool,
    selected_source: Option<u32>,
    correct_source: Option<bool>,
    exact_through_eos: Option<bool>,
    unsupported_copy: bool,
    first_error: Option<(usize, Option<u32>, Option<u32>)>,
    role_words: RoleWords,
    generated_ids: Vec<u32>,
}

#[derive(Deserialize)]
struct Endpoint {
    present: bool,
    requested_admitted: bool,
    excluded_admitted: bool,
}

fn prefix_hash(ids: &[u32]) -> String {
    let mut hash = Sha256::new();
    for id in ids {
        hash.update(id.to_le_bytes());
    }
    format!("{:x}", hash.finalize())
}

fn read_index(path: &Path) -> Result<Vec<PacketRow>> {
    let mut index = Vec::new();
    for line in fs::read_to_string(path)?.lines() {
        let (id, hash) = line.split_once('\t').ok_or("invalid index row")?;
        if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err("invalid SHA-256 in device index".into());
        }
        index.push(PacketRow {
            id: id.parse()?,
            prefix_hash: hash.to_string(),
        });
    }
    if !index
        .iter()
        .map(|row| row.id)
        .eq((30100..=30110).chain(30116..=30126))
    {
        return Err("device index differs from the ordered sealed role panel".into());
    }
    Ok(index)
}

fn from_words(bytes: &[u8]) -> Result<Vec<u32>> {
    let chunks = bytes.chunks_exact(4);
    if !chunks.remainder().is_empty() {
        return Err("invalid byte length for little-endian u32 words".into());
    }
    chunks
        .map(|bytes| Ok(u32::from_le_bytes(bytes.try_into()?)))
        .collect()
}

fn packet_rows(words: &[u32], mode: u32) -> Result<(&[u32], &[u32])> {
    let (header, payload) = words
        .split_at_checked(4)
        .ok_or("missing device packet header")?;
    let [magic, packet_mode, row_count, cells] = <[u32; 4]>::try_from(header)?;
    if magic != PACKET_MAGIC || packet_mode != mode || row_count as usize != ROWS || cells == 0 {
        return Err(format!("invalid packet header for mode {mode}").into());
    }
    let state_words = (SITES + cells as usize)
        .checked_mul(4)
        .ok_or("role state overflow")?;
    let (state, prompts) = payload
        .split_at_checked(state_words)
        .ok_or("truncated role state")?;
    if prompts.len() != ROWS * (WIDTH + 3) {
        return Err(format!("invalid device packet row shape for mode {mode}").into());
    }
    Ok((state, prompts))
}

fn validate_prefix(packed: &[u32], index: &PacketRow, ordinary: &Ordinary) -> Result<()> {
    let len = packed[WIDTH] as usize;
    if len == 0
        || len > WIDTH
        || packed[..len] != ordinary.prompt_ids
        || packed[len..WIDTH].iter().any(|&value| value != 0)
        || packed[WIDTH + 1..] != ordinary.role_replay.content
        || prefix_hash(&ordinary.prompt_ids) != index.prefix_hash
    {
        return Err(format!("unaligned original prefix on row {}", index.id).into());
    }
    Ok(())
}

fn exact_device_words(bytes: &[u8], mode: u32) -> Result<Vec<u32>> {
    let words = from_words(bytes)?;
    if words.len() != ROWS * 8 {
        return Err(format!("mode {mode} did not return exactly 22 eight-word rows").into());
    }
    Ok(words)
}

fn compare_role_words(
    row: u32,
    mode: u32,
    primary: &[u32],
    cold: &[u32],
    ordinary: RoleWords,
) -> Result<()> {
    if primary != ordinary || cold != ordinary {
        return Err(format!(
            "row {row} mode {mode} eight-word mismatch: primary {primary:?}; cold {cold:?}; ordinary {ordinary:?}"
        ).into());
    }
    Ok(())
}

fn read_grades(path: &Path) -> Result<BTreeMap<(String, u32), Grade>> {
    let grades: Vec<Grade> = serde_json::from_slice(&fs::read(path)?)?;
    let mut by_row = BTreeMap::new();
    for grade in grades {
        let key = (grade.arm.clone(), grade.row);
        if by_row.insert(key, grade).is_some() {
            return Err("duplicate ordinary grade row".into());
        }
    }
    if by_row.len() != ROWS * MODES.len() {
        return Err("ordinary grade length differs from the registered panel and arms".into());
    }
    Ok(by_row)
}

fn token_changes(before: &[u32], after: &[u32]) -> Vec<(usize, Option<u32>, Option<u32>)> {
    (0..before.len().max(after.len()))
        .filter_map(|offset| {
            let historical = before.get(offset).copied();
            let current = after.get(offset).copied();
            (historical != current).then_some((offset, historical, current))
        })
        .collect()
}

fn summarize(rows: &[&Grade]) -> Value {
    let count = |include: fn(&Grade) -> bool| rows.iter().filter(|row| include(row)).count();
    let changes: Vec<_> = rows
        .iter()
        .filter_map(|row| {
            row.first_error
                .map(|(offset, expected, actual)| json!([row.row, offset, expected, actual]))
        })
        .collect();
    json!({
        "rows": rows.len(),
        "requested_present": count(|row| row.requested.as_ref().is_some_and(|r| r.present)),
        "excluded_present": count(|row| row.excluded.as_ref().is_some_and(|r| r.present)),
        "both_present": count(|row| row.requested.as_ref().is_some_and(|r| r.present)
            && row.excluded.as_ref().is_some_and(|r| r.present)),
        "requested_admitted": count(|row| row.requested.as_ref().is_some_and(|r| r.requested_admitted)),
        "excluded_admitted": count(|row| row.excluded.as_ref().is_some_and(|r| r.excluded_admitted)),
        "both_admitted": count(|row| row.requested.as_ref().is_some_and(|r| r.requested_admitted)
            && row.excluded.as_ref().is_some_and(|r| r.excluded_admitted)),
        "requested_correct": count(|row| row.requested_correct),
        "excluded_correct": count(|row| row.excluded_correct),
        "joint_correct": count(|row| row.joint_correct),
        "incidental_chosen": count(|row| row.incidental_chosen),
        "source_selected": count(|row| row.selected_source.is_some()),
        "correct_source": count(|row| row.correct_source == Some(true)),
        "exact_through_eos": count(|row| row.exact_through_eos == Some(true)),
        "unsupported_copy": count(|row| row.unsupported_copy),
        "first_errors": changes,
    })
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let [packet, primary, cold, ordinary, ordinary_cold, grades] = args.as_slice() else {
        return Err("usage: d32-role-join PACKETS CUDA_PRIMARY CUDA_COLD ORDINARY_PRIMARY ORDINARY_COLD GRADES".into());
    };
    let index = read_index(&Path::new(packet).join("index.tsv"))?;
    let grades = read_grades(Path::new(grades))?;
    let mut shared_packet = None;
    let mut all_rows = Vec::new();
    let mut summaries = Vec::new();
    for (mode, arm) in MODES {
        let input = from_words(&fs::read(
            Path::new(packet).join(format!("mode-{mode}.bin")),
        )?)?;
        let (state, packed_rows) = packet_rows(&input, mode)?;
        let payload = &input[4..];
        match &shared_packet {
            None => shared_packet = Some(payload.to_vec()),
            Some(previous) if previous != payload => {
                return Err(format!("device packet {mode} differs outside the mode word").into());
            }
            Some(_) => {}
        }
        let filename = format!("mode-{mode}.out");
        let primary_words =
            exact_device_words(&fs::read(Path::new(primary).join(&filename))?, mode)?;
        let cold_words = exact_device_words(&fs::read(Path::new(cold).join(&filename))?, mode)?;
        let mut groups: BTreeMap<&str, Vec<&Grade>> = BTreeMap::new();

        for (position, row) in index.iter().enumerate() {
            let file = format!("{}.json", row.id);
            let primary = fs::read(Path::new(ordinary).join(arm).join(&file))?;
            let cold = fs::read(Path::new(ordinary_cold).join(arm).join(&file))?;
            if primary != cold {
                return Err(
                    format!("ordinary {arm} cold capture differs on row {}", row.id).into(),
                );
            }
            let client: Ordinary = serde_json::from_slice(&primary)?;
            let start = position * (WIDTH + 3);
            validate_prefix(&packed_rows[start..start + WIDTH + 3], row, &client)?;
            let start = position * 8;
            let cuda = &primary_words[start..start + 8];
            let cold = &cold_words[start..start + 8];
            let rust = client.paired.role_words;
            compare_role_words(row.id, mode, cuda, cold, rust)?;
            let grade = grades
                .get(&(arm.to_string(), row.id))
                .ok_or("grade row missing")?;
            if grade.role_words != rust
                || grade.generated_ids != client.generated_ids
                || grade.selected_source != client.selected.source
            {
                return Err(format!("grade {arm} cannot be joined on row {}", row.id).into());
            }
            let historical = grades
                .get(&(MODES[0].1.to_string(), row.id))
                .ok_or("historical grade missing")?;
            let differences = token_changes(&historical.generated_ids, &grade.generated_ids);
            groups.entry(&grade.stratum).or_default().push(grade);
            all_rows.push(json!({
                "row": row.id,
                "arm": arm,
                "mode": mode,
                "prompt_sha256_le32": row.prefix_hash,
                "role_words": rust,
                "selected_source": grade.selected_source,
                "first_change_from_historical": differences.first(),
                "emission_changes": differences,
            }));
        }
        let groups: BTreeMap<_, _> = groups
            .into_iter()
            .map(|(name, rows)| (name, summarize(&rows)))
            .collect();
        if groups.get("crossed").map(|row| &row["rows"]) != Some(&json!(16))
            || groups.get("direct").map(|row| &row["rows"]) != Some(&json!(2))
            || groups.get("incidental_only").map(|row| &row["rows"]) != Some(&json!(2))
            || groups.get("unspecified").map(|row| &row["rows"]) != Some(&json!(2))
        {
            return Err(format!("incorrect denominators on mode {mode}").into());
        }
        summaries
            .push(json!({"arm": arm, "mode": mode, "state_words": state.len(), "strata": groups}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({"arms": summaries, "rows": all_rows}))?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_partial_and_extra_device_rows() -> Result<()> {
        let good = [0; ROWS * 8 * 4];
        assert_eq!(exact_device_words(&good, 4)?.len(), ROWS * 8);
        assert!(exact_device_words(&good[..good.len() - 1], 4).is_err());
        assert!(exact_device_words(&good[..good.len() - 4], 4).is_err());
        assert!(exact_device_words(&[0; ROWS * 8 * 4 + 4], 4).is_err());
        Ok(())
    }

    #[test]
    fn rejects_wrong_role_word_in_either_launch() -> Result<()> {
        let words = [13, 17, u32::MAX, u32::MAX, 38, 0, (-1_i32) as u32, 0];
        let mut wrong = words;
        wrong[6] = 0;
        compare_role_words(30100, 4, &words, &words, words)?;
        assert!(compare_role_words(30100, 4, &wrong, &words, words).is_err());
        assert!(compare_role_words(30100, 4, &words, &wrong, words).is_err());
        Ok(())
    }

    #[test]
    fn rejects_wrong_prefix_hash_and_nonzero_padding() -> Result<()> {
        let mut packed = [0; WIDTH + 3];
        packed[..2].copy_from_slice(&[0, 42]);
        packed[WIDTH..].copy_from_slice(&[2, 1, 2]);
        let ordinary = Ordinary {
            prompt_ids: vec![0, 42],
            generated_ids: vec![],
            paired: Choice { role_words: [0; 8] },
            selected: Selection { source: None },
            role_replay: Replay { content: [1, 2] },
        };
        let mut index = PacketRow {
            id: 30100,
            prefix_hash: prefix_hash(&[0, 42]),
        };
        validate_prefix(&packed, &index, &ordinary)?;
        packed[2] = 1;
        assert!(validate_prefix(&packed, &index, &ordinary).is_err());
        packed[2] = 0;
        index.prefix_hash = prefix_hash(&[42, 0]);
        assert!(validate_prefix(&packed, &index, &ordinary).is_err());
        Ok(())
    }

    #[test]
    fn includes_new_tokens_and_early_eos_in_literal_delta() {
        assert_eq!(
            token_changes(&[17, 18, 1], &[17, 1]),
            vec![(1, Some(18), Some(1)), (2, Some(1), None)]
        );
        assert_eq!(
            token_changes(&[17, 1], &[17, 18, 1]),
            vec![(1, Some(1), Some(18)), (2, None, Some(1))]
        );
    }
}
