//! Post-score diagnostics over sealed ordinary captures. No model is loaded.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, env, error::Error, fs, path::Path};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Span = [u32; 2];
const GOLD: &str = "220ec3c5fd0c839bed864c051458b30f95668cc1b7a26c725d646d189fdc8096";
const D26: &str = "b17352679c64a96bf657fad7640d4bb05ea3de0ecd08dadd13e9d2756e979bc4";
const ARMS: [&str; 4] = [
    "historical-inherited",
    "historical-repeated",
    "full-inherited",
    "full-repeated",
];
const FLOOR: i32 = 12;

#[derive(Deserialize)]
struct Gold {
    row: u32,
    stratum: String,
    family: String,
    original_ids: Vec<u32>,
    role_targets: Targets,
}

#[derive(Deserialize)]
struct Targets {
    requested: Option<Span>,
    excluded: Option<Span>,
}

#[derive(Deserialize)]
struct Capture {
    prompt_ids: Vec<u32>,
    artifact_sha256: String,
    paired: Paired,
    role_replay: Trace,
}

#[derive(Deserialize)]
struct Paired {
    role_words: [u32; 8],
}

#[derive(Deserialize)]
struct Trace {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    span: Span,
    eligibility: String,
    requested_admitted: bool,
    excluded_admitted: bool,
    contrast: Option<i32>,
    scores: [i32; 2],
    excluded_decision_scores: Option<[i32; 2]>,
}

#[derive(Clone, Copy, Serialize)]
struct Ranked {
    span: Span,
    scores: [i32; 2],
    rank_score: i32,
}

#[derive(Serialize)]
struct Endpoint<'a> {
    target: Span,
    target_scores: Option<[i32; 2]>,
    target_rank_score: Option<i32>,
    target_eligibility: Option<&'a str>,
    excluded_minus_requested: Option<i32>,
    raw_excluded_minus_floor: Option<i32>,
    raw_excluded_minus_best_admitted_rival: Option<i32>,
    equal_raw_excluded_rivals: Option<usize>,
    winner: Option<Ranked>,
    leading_rivals: Vec<Ranked>,
    equal_scoring_rivals: Vec<Ranked>,
    gold_minus_best_rival: Option<i32>,
    best: Option<i32>,
    runner_up: Option<i32>,
    first_loss: &'static str,
    floor_only_recoverable: bool,
}

#[derive(Serialize)]
struct Row<'a> {
    row: u32,
    family: &'a str,
    arm: &'static str,
    requested: Endpoint<'a>,
    excluded: Endpoint<'a>,
}

fn endpoint(
    candidates: &[Candidate],
    target: Span,
    words: [u32; 8],
    excluded: bool,
) -> Result<Endpoint<'_>> {
    let raw_pair = |candidate: &Candidate| {
        candidate
            .excluded_decision_scores
            .unwrap_or(candidate.scores)
    };
    let rank = |candidate: &Candidate| {
        if excluded {
            candidate.excluded_admitted.then(|| raw_pair(candidate)[1])
        } else {
            candidate.contrast
        }
    };
    if candidates
        .iter()
        .any(|candidate| candidate.requested_admitted != candidate.contrast.is_some())
    {
        return Err("requested candidate admission/contrast inconsistent".into());
    }
    let scores = |candidate: &Candidate| {
        if excluded {
            raw_pair(candidate)
        } else {
            candidate.scores
        }
    };
    let mut ranking: Vec<_> = candidates
        .iter()
        .filter_map(|candidate| {
            rank(candidate).map(|rank_score| Ranked {
                span: candidate.span,
                scores: scores(candidate),
                rank_score,
            })
        })
        .collect();
    ranking.sort_unstable_by_key(|item| (std::cmp::Reverse(item.rank_score), item.span));
    let best = ranking.first().copied();
    let runner_up = ranking.get(1).map(|r| r.rank_score);
    let winner = best.filter(|lead| {
        (!excluded || lead.rank_score >= FLOOR)
            && runner_up.is_none_or(|second| lead.rank_score > second)
    });
    let idx = usize::from(excluded);
    let span_words = [words[idx * 2], words[idx * 2 + 1]];
    if span_words != winner.map_or([u32::MAX; 2], |w| w.span)
        || words[4 + idx] as i32 != best.map_or(i32::MIN, |w| w.rank_score)
        || words[6 + idx] as i32 != runner_up.unwrap_or(i32::MIN)
    {
        return Err("rank reconstruction differs from captured role words".into());
    }
    let candidate = candidates.iter().find(|c| c.span == target);
    let score_pair = candidate.map(scores);
    let gold_rank = candidate.and_then(rank);
    let rivals: Vec<_> = ranking.into_iter().filter(|r| r.span != target).collect();
    let leading_rivals = rivals.first().map_or_else(Vec::new, |rival| {
        rivals
            .iter()
            .copied()
            .take_while(|r| r.rank_score == rival.rank_score)
            .collect()
    });
    let margin = gold_rank
        .zip(leading_rivals.first())
        .map(|(gold, rival)| gold - rival.rank_score);
    let equal_scoring_rivals = rivals
        .into_iter()
        .filter(|r| Some(r.rank_score) == gold_rank)
        .collect();
    let first_loss = match (candidate, gold_rank, margin) {
        (None, _, _) => "absent",
        (_, None, _) if excluded => "excluded_contrast_gate",
        (_, None, _) => "requested_admission",
        (_, Some(_), Some(delta)) if delta < 0 => "outranked",
        (_, Some(_), Some(0)) => "tied",
        (_, Some(score), _) if excluded && score < FLOOR => "floor_only",
        _ => "selected",
    };
    let exclusion_pair = score_pair.filter(|_| excluded);
    Ok(Endpoint {
        target,
        target_scores: score_pair,
        target_rank_score: gold_rank,
        target_eligibility: candidate.map(|c| c.eligibility.as_str()),
        excluded_minus_requested: exclusion_pair.map(|[requested, excluded]| excluded - requested),
        raw_excluded_minus_floor: exclusion_pair.map(|[_, excluded]| excluded - FLOOR),
        raw_excluded_minus_best_admitted_rival: exclusion_pair
            .zip(leading_rivals.first())
            .map(|([_, excluded], rival)| excluded - rival.rank_score),
        equal_raw_excluded_rivals: exclusion_pair.map(|[_, excluded]| {
            candidates
                .iter()
                .filter(|c| c.span != target && raw_pair(c)[1] == excluded)
                .count()
        }),
        winner,
        leading_rivals,
        equal_scoring_rivals,
        gold_minus_best_rival: margin,
        best: best.map(|w| w.rank_score),
        runner_up,
        first_loss,
        floor_only_recoverable: first_loss == "floor_only",
    })
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args_os().collect();
    let [_, gold_file, capture_root] = args.as_slice() else {
        return Err("usage: d32-role-attribution SEALED_GOLD CAPTURE_ROOT".into());
    };
    let bytes = fs::read(gold_file)?;
    if format!("{:x}", Sha256::digest(&bytes)) != GOLD {
        return Err("sealed role gold SHA-256 mismatch".into());
    }
    let gold: Vec<Gold> = serde_json::from_slice(&bytes)?;
    let mut results = Vec::new();
    for row in gold.iter().filter(|row| row.stratum == "crossed") {
        let requested = row
            .role_targets
            .requested
            .ok_or("missing crossed requested")?;
        let excluded = row
            .role_targets
            .excluded
            .ok_or("missing crossed excluded")?;
        for arm in ARMS {
            let capture: Capture = serde_json::from_slice(&fs::read(
                Path::new(capture_root)
                    .join(arm)
                    .join(format!("{}.json", row.row)),
            )?)?;
            if capture.artifact_sha256 != D26 || capture.prompt_ids != row.original_ids {
                return Err(format!("capture binding mismatch for {arm}/{}", row.row).into());
            }
            let attribution = |span, side| {
                endpoint(
                    &capture.role_replay.candidates,
                    span,
                    capture.paired.role_words,
                    side,
                )
            };
            let record = Row {
                row: row.row,
                family: &row.family,
                arm,
                requested: attribution(requested, false)?,
                excluded: attribution(excluded, true)?,
            };
            results.push(serde_json::to_value(record)?);
        }
    }
    if results.len() != 64 {
        return Err("incomplete crossed panel".into());
    }
    let mut summary = BTreeMap::new();
    for row in &results {
        for side in ["requested", "excluded"] {
            let key = (
                row["arm"].as_str().ok_or("missing arm")?,
                side,
                row[side]["first_loss"]
                    .as_str()
                    .ok_or("missing attribution")?,
            );
            *summary.entry(key).or_insert(0) += 1;
        }
    }
    let out = serde_json::json!({
        "excluded_floor": FLOOR,
        "crossed_rows_per_arm": results.len() / ARMS.len(),
        "counts": summary.into_iter().map(|((arm, role, first_loss), count)| serde_json::json!({
            "arm": arm, "role": role, "first_loss": first_loss, "count": count,
        })).collect::<Vec<_>>(),
        "rows": results,
    });
    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}
