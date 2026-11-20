//! Post-hoc inspection of registered transfers in the sealed UORREL01 packet.
use std::{collections::BTreeMap, fs, path::Path};

use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::{read_jsonl, Result, Span};

const WIDTH: usize = 192;
const ROW: usize = 4 * (3 + 6 * WIDTH);
const HEADER: usize = 8 + 16 * 8 * 4;

#[derive(Deserialize)]
struct Fact {
    person: Span,
    relation: Span,
}

#[derive(Deserialize)]
struct Gold {
    row: u32,
    stratum: String,
    factors: Option<[usize; 4]>,
    assertions: Vec<Fact>,
    selected: Vec<usize>,
}

#[derive(Deserialize)]
struct Candidate {
    fact: Position,
    cue: Position,
}

#[derive(Deserialize)]
struct Position {
    start: usize,
    end: usize,
}

impl Position {
    fn span(&self) -> Span {
        [self.start, self.end]
    }
}

#[derive(Deserialize)]
struct Native {
    row: u32,
    lane: String,
    device_row: Option<usize>,
    #[serde(default)]
    candidates: Vec<Candidate>,
}

struct Tape<'a> {
    data: &'a [u8],
    roots: &'a [u8],
}

impl Tape<'_> {
    fn word(&self, i: usize) -> Result<u32> {
        word(self.data, i)
    }

    fn span(&self, i: usize) -> Result<Span> {
        Ok([self.word(i)? as usize, self.word(i + 1)? as usize])
    }

    fn ids(&self, [start, end]: Span) -> Result<Vec<u32>> {
        if start >= end || end > self.word(0)? as usize || end > WIDTH {
            return Err("invalid sealed span".into());
        }
        (start..end).map(|i| self.word(3 + 2 * i)).collect()
    }

    fn queries(&self) -> Result<Vec<Span>> {
        (0..self.word(2)? as usize)
            .map(|i| self.span(3 + WIDTH * 2 + i * 4 + 2))
            .collect()
    }

    fn compare(&self, left: Span, right: Span) -> Result<(bool, Option<u32>)> {
        let left_ids = self.ids(left)?;
        let right_ids = self.ids(right)?;
        if left_ids.len() != right_ids.len() {
            return Ok((false, None));
        }
        let mut distance = 0;
        for (a, b) in (left[0]..left[1]).zip(right[0]..right[1]) {
            let a = self.word(4 + 2 * a)?;
            let b = self.word(4 + 2 * b)?;
            for shift in [0, 4, 8, 12] {
                let a = ((a >> shift) & 15) as usize;
                let b = ((b >> shift) & 15) as usize;
                for axis in 0..8 {
                    let delta = word(self.roots, a * 8 + axis)? as i32
                        - word(self.roots, b * 8 + axis)? as i32;
                    distance += (delta * delta) as u32;
                }
            }
        }
        Ok((left_ids == right_ids, Some(distance)))
    }
}

fn word(data: &[u8], i: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        data.get(i * 4..i * 4 + 4)
            .ok_or("truncated word")?
            .try_into()?,
    ))
}

fn tape(packet: &[u8], index: usize) -> Result<Tape<'_>> {
    let offset = HEADER + index * ROW;
    Ok(Tape {
        data: packet
            .get(offset..offset + ROW)
            .ok_or("missing sealed device row")?,
        roots: &packet[8..HEADER],
    })
}

fn describe(tape: &Tape<'_>, left: Span, right: Span) -> Result<Value> {
    let (literal, distance) = tape.compare(left, right)?;
    let left_ids = tape.ids(left)?;
    let right_ids = tape.ids(right)?;
    let radius = left_ids.len() * 16;
    let status = match distance {
        _ if literal => "literal",
        None => "unequal_bpe_piece_counts",
        Some(0) => "distinct_ids_with_identical_roots",
        Some(d) if d as usize > radius => "outside_radius",
        Some(_) => "inside_radius",
    };
    Ok(
        json!({"fact_ids": left_ids, "query_ids": right_ids, "fact_pieces": left_ids.len(),
        "query_pieces": right_ids.len(), "distance": distance, "threshold_if_comparable": distance.map(|_| radius),
        "status": status}),
    )
}

pub fn audit(stage: &Path, gold_path: &Path, relation_path: &Path, gpu: &Path) -> Result<()> {
    let (gold_sha, gold) = read_jsonl::<Gold>(gold_path)?;
    let (relation_sha, relation) = read_jsonl::<crate::Relation>(relation_path)?;
    let (native_sha, native) = read_jsonl::<Native>(&stage.join("native.jsonl"))?;
    let packet = fs::read(stage.join("input.bin"))?;
    if !packet.starts_with(b"UORREL01")
        || packet.len() < HEADER
        || (packet.len() - HEADER) % ROW != 0
    {
        return Err("invalid sealed UORREL01 packet".into());
    }
    let output = fs::read(gpu)?;
    let output_row = uor_r4_integer::relation_retrieval::DEVICE_OUTPUT_ROW_BYTES;
    if output.len() / output_row != (packet.len() - HEADER) / ROW
        || output.len() % output_row != 0
        || output != fs::read(stage.join("oracle.bin"))?
    {
        return Err("sealed input and verified device output do not match".into());
    }
    let declared: BTreeMap<u32, Span> = relation.into_iter().map(|r| (r.row, r.relation)).collect();
    let cell_rows: Vec<_> = include_str!("../panel/cells.tsv").lines().collect();
    let mut families: BTreeMap<(&str, &str), Vec<Value>> = BTreeMap::new();
    for key in gold.iter().filter(|key| key.stratum == "transfer") {
        let [cell, direction, ..] = key.factors.ok_or("transfer row has no crossed factors")?;
        let [target] = key.selected.as_slice() else {
            return Err(format!("expected one target fact on transfer row {}", key.row).into());
        };
        let columns: Vec<_> = cell_rows
            .get(cell)
            .ok_or("unknown panel cell")?
            .split('\t')
            .collect();
        let family = (
            *columns.get(2 + direction).ok_or("invalid fact cue")?,
            *columns.get(4 + direction).ok_or("invalid query cue")?,
        );
        let head = key.assertions.get(*target).ok_or("target out of bounds")?;
        let relation = *declared.get(&key.row).ok_or("missing supplied relation")?;
        let narrow = native
            .iter()
            .find(|n| n.row == key.row && n.lane == "person_relation")
            .ok_or("missing supplied lane")?;
        let broad = native
            .iter()
            .find(|n| n.row == key.row && n.lane == "person")
            .ok_or("missing person lane")?;
        let broad_tape = tape(
            &packet,
            broad.device_row.ok_or("missing person device row")?,
        )?;
        let queries = broad_tape.queries()?;
        let observed = narrow.device_row.map(|i| -> Result<Value> {
            let narrow_tape = tape(&packet, i)?;
            let tape_window = 12..12 + WIDTH * 8;
            if narrow_tape.queries()? != [relation]
                || narrow_tape.word(0)? != broad_tape.word(0)?
                || narrow_tape.data[tape_window.clone()] != broad_tape.data[tape_window]
                || narrow_tape.word(1)? as usize != narrow.candidates.len() {
                return Err("sealed supplied lane span mismatch".into());
            }
            for (candidate_index, candidate) in narrow.candidates.iter().enumerate() {
                if narrow_tape.span(3 + WIDTH * 2 + candidate_index * 4)? != candidate.cue.span() {
                    return Err("sealed cue differs from indexed cue".into());
                }
            }
            let device = &output[i * output_row..][..output_row];
            Ok(json!({"device_row": i, "exact_outcome": word(device, 0)?, "geometry_outcome": word(device, 1)?}))
        }).transpose()?;
        let exposed_target = broad.candidates.iter().any(|candidate| {
            candidate.fact.span() == head.person && candidate.cue.span() == head.relation
        });
        let eligible_relation = queries.contains(&relation);
        let target = describe(&broad_tape, head.relation, relation)?;
        let blocker = if !exposed_target {
            "target_cue_not_exposed"
        } else if !eligible_relation {
            "relation_not_one_eligible_whole_word"
        } else {
            target["status"]
                .as_str()
                .ok_or("missing comparison status")?
        };
        let admitted = |comparison: &Value| {
            matches!(
                comparison["status"].as_str(),
                Some("literal" | "distinct_ids_with_identical_roots" | "inside_radius")
            )
        };
        let mut alternative_evidence = Vec::new();
        let mut competing_declared_pairs = Vec::new();
        for candidate in &broad.candidates {
            if candidate.fact.span() != head.person {
                competing_declared_pairs.push(describe(
                    &broad_tape,
                    candidate.cue.span(),
                    relation,
                )?);
            }
            for &query in &queries {
                if query == relation && candidate.fact.span() == head.person {
                    continue;
                }
                let comparison = describe(&broad_tape, candidate.cue.span(), query)?;
                if admitted(&comparison) {
                    alternative_evidence.push(json!({"cue": candidate.cue.span(), "query": query,
                        "declared_query": query == relation, "comparison": comparison}));
                }
            }
        }
        families.entry(family).or_default().push(json!({"row": key.row, "comparison": target,
            "first_blocker": blocker, "device": observed, "alternative_evidence": alternative_evidence,
            "competing_declared_pairs": competing_declared_pairs}));
    }
    let report: Vec<_> = families.into_iter().map(|((fact, query), rows)| {
        let mut blockers = BTreeMap::new();
        let mut comparisons = Vec::new();
        let mut competitors = Vec::new();
        let mut alternatives = BTreeMap::new();
        let mut device_outcomes = BTreeMap::new();
        for row in &rows {
            *blockers.entry(row["first_blocker"].as_str().unwrap_or("missing")).or_insert(0) += 1;
            if !comparisons.contains(&row["comparison"]) {
                comparisons.push(row["comparison"].clone());
            }
            for pair in row["competing_declared_pairs"].as_array().into_iter().flatten() {
                if !competitors.contains(pair) {
                    competitors.push(pair.clone());
                }
            }
            if row["alternative_evidence"].as_array().is_some_and(|a| !a.is_empty()) {
                alternatives.insert(row["row"].to_string(), row["alternative_evidence"].clone());
            }
            let device = &row["device"];
            let outcome = if device.is_null() {
                "not_staged".to_owned()
            } else {
                format!("{}/{}", device["exact_outcome"], device["geometry_outcome"])
            };
            *device_outcomes.entry(outcome).or_insert(0) += 1;
        }
        json!({"fact_cue": fact, "query_word": query, "rows": rows.len(), "first_blockers": blockers,
            "row_ids": rows.iter().map(|r| &r["row"]).collect::<Vec<_>>(), "target_comparisons": comparisons,
            "competing_comparisons_to_declared_query": competitors, "alternative_evidence_by_row": alternatives,
            "device_exact_geometry_outcomes": device_outcomes})
    }).collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({"schema": "uor-r4.d27-transfer-failure/1",
        "lane": "person_relation", "alternative_evidence_lane": "person", "families": report,
        "inputs": {"device_input_sha256": format!("{:x}", Sha256::digest(&packet)),
            "device_output_sha256": format!("{:x}", Sha256::digest(&output)),
            "native_index_sha256": native_sha, "gold_sha256": gold_sha, "supplied_relation_sha256": relation_sha}}))?
    );
    Ok(())
}
