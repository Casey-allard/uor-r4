//! CPU oracle for Claude-330's question-keyed ask-head geometry.

use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use serde::Serialize;
use uor_r4_cpu_train::{AskCellBits, AskHeadKept, AskHeadOracle};
use uor_r4_integer::stack::copy_weight_q31;

const SCHEMA: &str = "uor-r4.cpu-ask-head-oracle/1";
const RELATION_TOKENS: std::ops::Range<u32> = 16..32;
const QUERY_SHIFT: u32 = 16;
const HEADER: [u32; 4] = [48, 49, 50, 51];
const VALUE_TOKENS: std::ops::Range<u32> = 528..1_040;
const FILLER_TOKENS: std::ops::Range<u32> = 1_040..4_096;
const BOS: u32 = 0;
const CONTEXT: usize = 192;
const QUESTION_ROW: usize = 186;
const SCORED_POSITION: usize = 190;
const KEY_POSITION: usize = 88;
const VALUE_POSITION: usize = 89;
const TARGET_DISTANCE: usize = 97;
const MAX_FIT_EVALUATIONS: u64 = 200_000;

type Result<T> = std::result::Result<T, String>;

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-ask-head-oracle: {error}");
        std::process::exit(1);
    }
}

#[derive(Serialize)]
struct RelationReading {
    relation: u32,
    question: u32,
    value: u32,
    target_position: usize,
    distance: usize,
    ask_cells: usize,
    target_kept_before_cells: usize,
    target_kept_exact_cells: usize,
    target_kept_fitted_cells: usize,
    target_weighted_before_cells: usize,
    target_weighted_exact_cells: usize,
    target_weighted_fitted_cells: usize,
    target_type_margin_weighted_before_cells: usize,
    target_type_margin_weighted_exact_cells: usize,
    target_type_margin_weighted_fitted_cells: usize,
    start_cell_bits: Vec<AskCellBits>,
    exact_cell_bits: Vec<AskCellBits>,
    fitted_cell_bits: Vec<AskCellBits>,
    retained_before: Vec<RetainedCell>,
    retained_exact: Vec<RetainedCell>,
    retained_fitted: Vec<RetainedCell>,
    weighted_before: Vec<RetainedCell>,
    weighted_exact: Vec<RetainedCell>,
    weighted_fitted: Vec<RetainedCell>,
    type_margin_weighted_before: Vec<RetainedCell>,
    type_margin_weighted_exact: Vec<RetainedCell>,
    type_margin_weighted_fitted: Vec<RetainedCell>,
    start_bits: u64,
    exact_bits: u64,
    fitted_bits: u64,
    fit_evaluations: u64,
    fitted_coordinates: usize,
    fit_budget_exhausted: bool,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
struct RetainedCell {
    depth: usize,
    head: usize,
}

fn run() -> Result<()> {
    let args = parse_args(std::env::args().skip(1))?;
    let artifact = std::fs::read(&args["artifact"])
        .map_err(|error| format!("{}: {error}", args["artifact"]))?;
    let expected = &args["expected-artifact-sha256"];
    validate_sha256(expected)?;
    let actual = uor_r4_discrete::corpus::sha256(&artifact);
    if &actual != expected {
        return Err(format!("artifact SHA-256 differs: got {actual}"));
    }
    let out = PathBuf::from(&args["out"]);
    if out.exists() {
        return Err(format!(
            "{} exists; reports are never replaced",
            out.display()
        ));
    }
    let oracle = AskHeadOracle::from_artifact(&artifact)?;
    let zero = oracle.zero_offset();
    let readings = RELATION_TOKENS
        .into_par_iter()
        .map(|relation| {
            let question = relation + QUERY_SHIFT;
            let value = VALUE_TOKENS.start + (relation - RELATION_TOKENS.start);
            let tokens = one_fact_window(relation, question, value);
            let question_prefix = &tokens[..=QUESTION_ROW];
            let exact = oracle.exact_offset_to_key(question, relation)?;
            let fit = oracle.fit_offset_to_key(question, relation, MAX_FIT_EVALUATIONS)?;
            let before = oracle.kept_at_question(question_prefix, &zero)?;
            let exact_kept = oracle.kept_at_question(question_prefix, &exact)?;
            let fitted_kept = oracle.kept_at_question(question_prefix, &fit.offset)?;
            let retained_before = retained_cells(&before, VALUE_POSITION);
            let retained_exact = retained_cells(&exact_kept, VALUE_POSITION);
            let retained_fitted = retained_cells(&fitted_kept, VALUE_POSITION);
            let weighted_before = weighted_cells(&before, VALUE_POSITION);
            let weighted_exact = weighted_cells(&exact_kept, VALUE_POSITION);
            let weighted_fitted = weighted_cells(&fitted_kept, VALUE_POSITION);
            let type_margin_weighted_before = artifact_type_weighted_cells(&before, VALUE_POSITION);
            let type_margin_weighted_exact =
                artifact_type_weighted_cells(&exact_kept, VALUE_POSITION);
            let type_margin_weighted_fitted =
                artifact_type_weighted_cells(&fitted_kept, VALUE_POSITION);
            Ok(RelationReading {
                relation,
                question,
                value,
                target_position: VALUE_POSITION,
                distance: TARGET_DISTANCE,
                ask_cells: before.len(),
                target_kept_before_cells: retained_before.len(),
                target_kept_exact_cells: retained_exact.len(),
                target_kept_fitted_cells: retained_fitted.len(),
                target_weighted_before_cells: weighted_before.len(),
                target_weighted_exact_cells: weighted_exact.len(),
                target_weighted_fitted_cells: weighted_fitted.len(),
                target_type_margin_weighted_before_cells: type_margin_weighted_before.len(),
                target_type_margin_weighted_exact_cells: type_margin_weighted_exact.len(),
                target_type_margin_weighted_fitted_cells: type_margin_weighted_fitted.len(),
                start_cell_bits: oracle.cell_bits_to_key(question, relation, &zero)?,
                exact_cell_bits: oracle.cell_bits_to_key(question, relation, &exact)?,
                fitted_cell_bits: oracle.cell_bits_to_key(question, relation, &fit.offset)?,
                retained_before,
                retained_exact,
                retained_fitted,
                weighted_before,
                weighted_exact,
                weighted_fitted,
                type_margin_weighted_before,
                type_margin_weighted_exact,
                type_margin_weighted_fitted,
                start_bits: fit.start_bits,
                exact_bits: oracle.total_bits_to_key(question, relation, &exact)?,
                fitted_bits: fit.end_bits,
                fit_evaluations: fit.evaluations,
                fitted_coordinates: fit.changed_coordinates,
                fit_budget_exhausted: fit.budget_exhausted,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let report = serde_json::json!({
        "schema": SCHEMA,
        "artifact_sha256": expected,
        "hardware": {
            "cpu": cpu_model(),
            "executable_sha256": executable_sha256()?,
            "rayon_threads": rayon::current_num_threads(),
        },
        "settings": {
            "context": CONTEXT,
            "question_row": QUESTION_ROW,
            "scored_position": SCORED_POSITION,
            "key_position": KEY_POSITION,
            "value_position": VALUE_POSITION,
            "header": HEADER,
            "distance": TARGET_DISTANCE,
            "kept": oracle.kept_count(),
            "ask_heads": oracle.ask_head_count(),
            "width": oracle.width(),
            "max_fit_evaluations": MAX_FIT_EVALUATIONS,
        },
        "summary": {
            "relations": readings.len(),
            "exact_zero_bit_relations": readings.iter().filter(|r| r.exact_bits == 0).count(),
            "fitted_zero_bit_relations": readings.iter().filter(|r| r.fitted_bits == 0).count(),
            "ask_cells": readings.iter().map(|r| r.ask_cells).sum::<usize>(),
            "target_kept_before_cells": readings.iter().map(|r| r.target_kept_before_cells).sum::<usize>(),
            "target_kept_exact_cells": readings.iter().map(|r| r.target_kept_exact_cells).sum::<usize>(),
            "target_kept_fitted_cells": readings.iter().map(|r| r.target_kept_fitted_cells).sum::<usize>(),
            "target_weighted_before_cells": readings.iter().map(|r| r.target_weighted_before_cells).sum::<usize>(),
            "target_weighted_exact_cells": readings.iter().map(|r| r.target_weighted_exact_cells).sum::<usize>(),
            "target_weighted_fitted_cells": readings.iter().map(|r| r.target_weighted_fitted_cells).sum::<usize>(),
            "target_type_margin_weighted_before_cells": readings.iter().map(|r| r.target_type_margin_weighted_before_cells).sum::<usize>(),
            "target_type_margin_weighted_exact_cells": readings.iter().map(|r| r.target_type_margin_weighted_exact_cells).sum::<usize>(),
            "target_type_margin_weighted_fitted_cells": readings.iter().map(|r| r.target_type_margin_weighted_fitted_cells).sum::<usize>(),
            "relations_with_target_before": readings.iter().filter(|r| r.target_kept_before_cells > 0).count(),
            "relations_with_target_exact": readings.iter().filter(|r| r.target_kept_exact_cells > 0).count(),
            "relations_with_target_fitted": readings.iter().filter(|r| r.target_kept_fitted_cells > 0).count(),
            "fit_evaluations": readings.iter().map(|r| r.fit_evaluations).sum::<u64>(),
            "fit_budget_exhausted_relations": readings.iter().filter(|r| r.fit_budget_exhausted).count(),
        },
        "relations": readings,
        "claim": "CPU ask-query/found-copy geometry oracle at the B2 query row only; not a B2, carried-value, capability or serving result"
    });
    let mut bytes = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    write_new(&out, &bytes)?;
    println!(
        "{}",
        serde_json::json!({
            "schema": SCHEMA,
            "out": out,
            "sha256": uor_r4_discrete::corpus::sha256(&bytes),
            "summary": report["summary"],
        })
    );
    Ok(())
}

fn one_fact_window(relation: u32, question: u32, value: u32) -> Vec<u32> {
    let mut state = u64::from(relation)
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(244)
        | 1;
    let mut tokens = (0..CONTEXT)
        .map(|_| {
            state ^= state >> 12;
            state ^= state << 25;
            state ^= state >> 27;
            FILLER_TOKENS.start
                + (state.wrapping_mul(0x2545_F491_4F6C_DD1D)
                    % u64::from(FILLER_TOKENS.end - FILLER_TOKENS.start)) as u32
        })
        .collect::<Vec<_>>();
    tokens[0] = BOS;
    tokens[KEY_POSITION] = relation;
    tokens[VALUE_POSITION] = value;
    tokens[QUESTION_ROW] = question;
    tokens[QUESTION_ROW + 1..=SCORED_POSITION].copy_from_slice(&HEADER);
    tokens[SCORED_POSITION + 1] = value;
    tokens
}

fn retained_cells(heads: &[AskHeadKept], target: usize) -> Vec<RetainedCell> {
    heads
        .iter()
        .filter(|head| {
            head.kept
                .iter()
                .any(|position| position.position as usize == target)
        })
        .map(|head| RetainedCell {
            depth: head.depth,
            head: head.head,
        })
        .collect()
}

fn weighted_cells(heads: &[AskHeadKept], target: usize) -> Vec<RetainedCell> {
    heads
        .iter()
        .filter(|head| {
            head.kept.iter().any(|position| {
                position.position as usize == target
                    && copy_weight_q31(head.ask_margin_q16, position.distance) > 0
            })
        })
        .map(|head| RetainedCell {
            depth: head.depth,
            head: head.head,
        })
        .collect()
}

fn artifact_type_weighted_cells(heads: &[AskHeadKept], target: usize) -> Vec<RetainedCell> {
    heads
        .iter()
        .filter(|head| {
            head.kept.iter().any(|position| {
                position.position as usize == target
                    && copy_weight_q31(head.artifact_type_margin_q16, position.distance) > 0
            })
        })
        .map(|head| RetainedCell {
            depth: head.depth,
            head: head.head,
        })
        .collect()
}

fn validate_sha256(value: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("--expected-artifact-sha256 is not a SHA-256".to_owned());
    }
    Ok(())
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<BTreeMap<String, String>> {
    const FLAGS: [&str; 3] = ["artifact", "expected-artifact-sha256", "out"];
    let mut parsed = BTreeMap::new();
    let mut args = args;
    while let Some(argument) = args.next() {
        let name = argument
            .strip_prefix("--")
            .filter(|name| FLAGS.contains(name))
            .ok_or_else(|| format!("unknown argument {argument}"))?;
        let value = args
            .next()
            .ok_or_else(|| format!("{argument} needs a value"))?;
        if parsed.insert(name.to_owned(), value).is_some() {
            return Err(format!("{argument} given twice"));
        }
    }
    for name in FLAGS {
        if !parsed.contains_key(name) {
            return Err(format!("--{name} is required"));
        }
    }
    Ok(parsed)
}

fn cpu_model() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                line.strip_prefix("model name\t: ")
                    .map(std::string::ToString::to_string)
            })
        })
        .unwrap_or_else(|| "unknown".to_owned())
}

fn executable_sha256() -> Result<String> {
    let path = std::env::current_exe().map_err(|error| error.to_string())?;
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    Ok(uor_r4_discrete::corpus::sha256(&bytes))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    file.write_all(bytes).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_window_places_one_fact_inside_the_far_bucket() {
        let tokens = one_fact_window(17, 33, 529);
        assert_eq!(tokens.len(), CONTEXT);
        assert_eq!(tokens[0], BOS);
        assert_eq!(tokens[KEY_POSITION], 17);
        assert_eq!(tokens[VALUE_POSITION], 529);
        assert_eq!(tokens[QUESTION_ROW], 33);
        assert_eq!(&tokens[QUESTION_ROW + 1..=SCORED_POSITION], &HEADER);
        assert_eq!(tokens[SCORED_POSITION + 1], 529);
        assert_eq!(QUESTION_ROW - VALUE_POSITION, TARGET_DISTANCE);
        let b2_query_rows = [CONTEXT - 24, CONTEXT - 18, CONTEXT - 12, CONTEXT - 6];
        assert_eq!(b2_query_rows, [168, 174, 180, 186]);
        assert_eq!(b2_query_rows[3], QUESTION_ROW);
        assert_eq!(SCORED_POSITION, QUESTION_ROW + 4);
        assert!(tokens.iter().filter(|&&token| token == 17).count() == 1);
        assert!(tokens.iter().filter(|&&token| token == 529).count() == 2);
    }

    #[test]
    fn artifact_hash_argument_is_required_to_be_sha256() {
        assert!(validate_sha256(&"a".repeat(64)).is_ok());
        assert!(validate_sha256(&"a".repeat(63)).is_err());
        assert!(validate_sha256(&"g".repeat(64)).is_err());
    }

    #[test]
    fn weighted_retention_excludes_positions_at_or_beyond_the_margin() {
        let head = |head, distance| AskHeadKept {
            depth: 2,
            head,
            ask_margin_q16: 4 << 16,
            artifact_type_margin_q16: 8 << 16,
            query: 0,
            key: 0,
            kept: vec![uor_r4_cpu_train::AskKeptPosition {
                position: VALUE_POSITION as u32,
                distance,
            }],
        };
        let heads = [head(2, 3), head(3, 4)];
        assert_eq!(retained_cells(&heads, VALUE_POSITION).len(), 2);
        assert_eq!(
            weighted_cells(&heads, VALUE_POSITION),
            vec![RetainedCell { depth: 2, head: 2 }]
        );
        assert_eq!(
            artifact_type_weighted_cells(&heads, VALUE_POSITION).len(),
            2
        );
    }
}
