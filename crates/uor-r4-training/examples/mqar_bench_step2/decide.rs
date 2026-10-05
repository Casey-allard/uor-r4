//! `mode=decide runs=DIR out=NEW_REPORT_ROOT`: the Step 2 decision table
//! (References #820). It reads every sealed `layout=fact` report root
//! directly under `runs` (each verified against its manifest), and applies
//! the rules frozen in `final-assessment.md` Step 2 with the ordering fix of
//! `completeness-critique.md` point 5. The rules and thresholds are the
//! constants below; they were fixed before any full run.
//!
//! **Cells and metric.** The decision cells are `g >= 1` with a multi-piece
//! key (2 or 3 pieces), for each reply form: 6 per form. The metric is the
//! full-value accuracy (every value piece, teacher forced) on the final
//! in-class fresh-pairing evaluation. A cell **passes** for an arm when every
//! seed scores at least `PASS` on it and the arm has at least its required
//! seeds. A form passes when all its decision cells pass; the arm passes
//! when both forms do.
//!
//! **Rules, per production pattern (`rararr`, `rrarra`), in this order:**
//! 1. `none` passes: there is no lag problem in this pattern; stop lineage
//!    research. Evaluated first: when it fires, rules 2 and 3 cannot assign
//!    credit to a lineage and are reported as not informative.
//! 2. Otherwise, `f2` passes: F2 is sufficient.
//! 3. Only when `none` fails: if the learned depthwise conv is within `TIE`
//!    of F2 (seed means) on every decision cell, the geometric claim for `j`
//!    is retracted (F2 is kept as the cheapest exact form); if it is more
//!    than `TIE` below F2 on some cell, the claim stands against the conv
//!    control. Added guard: when F2 itself does not pass, rule 3 is not
//!    informative (two failing arms tying says nothing about `j`); the
//!    literal tie is still recorded (`r3_literal_tie_within_0_03`). A conv
//!    that exceeds F2 by more than `TIE` somewhere, and is nowhere more than
//!    `TIE` below it, also retracts the claim.
//! 4. If every arm present passes REHEARSE and fails BARE, the remaining gap
//!    is reply form/data ("{k} is {v}" targets), not mechanism.
//! 5. Capability: an arm counts on a cell only when the cell passes and its
//!    seed mean meets the rehearse + R-nlet reference rule's accuracy on the
//!    same evaluation sets (mean at least rule mean minus `RULE_TOLERANCE`).
//!    `meets_rule` and `beats_rule` are also reported per cell.
//! 6. Conditional Step 2b: if the conv fails a decision cell that some
//!    lineage arm passes, Step 2b (lags 0-3) is justified.
//! 7. Informational: cells where `qk` (query + key lineage) passes and `f2`
//!    fails (query-side lineage needed).
//!
//! Rules 1-3 are decided only when none, F2 and conv have all their seeds,
//! and rule 6 only when conv has. Rules 1-3 and 6 are applied to both forms
//! together and to each form alone. `aaaaaa` is diagnostic: tabulated, never decided.

use std::collections::BTreeSet;

use super::fact::{cell_name, relevant_cell, Form, FACT_SCHEMA};
use super::*;

const PASS: f64 = 0.95;
const TIE: f64 = 0.03;
const RULE_TOLERANCE: f64 = 0.01;
const PRODUCTION_PATTERNS: [&str; 2] = ["rararr", "rrarra"];
const DIAGNOSTIC_PATTERNS: [&str; 1] = ["aaaaaa"];
const LINEAGE_ARMS: [&str; 6] = ["f2", "qk", "qk_jj", "identity", "so4", "wprev"];
const DECISION_SCHEMA: &str = "uor-r4/mqar-bench/step2-decision/v1";

/// Seeds each arm needs before its cells can pass.
fn required_seeds(arm: &str) -> usize {
    match arm {
        "none" | "f2" | "qk" | "conv" | "qk_jj" => 3,
        _ => 2,
    }
}

fn decision_cells(form: Option<Form>) -> Vec<String> {
    let forms = match form {
        Some(form) => vec![form],
        None => vec![Form::Rehearse, Form::Bare],
    };
    let mut cells = Vec::new();
    for form in forms {
        for gap in 1..=3 {
            for key_len in 2..=3 {
                if relevant_cell(gap, key_len) {
                    cells.push(cell_name(form, gap, key_len));
                }
            }
        }
    }
    cells
}

/// One seed of one (pattern, arm): its per-cell model and rule accuracy.
struct SeedRun {
    seed: u64,
    root: String,
    cells: BTreeMap<String, (f64, f64)>,
}

#[derive(Default)]
struct ArmRuns {
    seeds: Vec<SeedRun>,
}

impl ArmRuns {
    fn mean(&self, cell: &str, which: usize) -> Option<f64> {
        let values: Vec<f64> = self
            .seeds
            .iter()
            .filter_map(|run| run.cells.get(cell))
            .map(|pair| if which == 0 { pair.0 } else { pair.1 })
            .collect();
        (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
    }

    fn min(&self, cell: &str) -> Option<f64> {
        self.seeds
            .iter()
            .map(|run| run.cells.get(cell).map(|pair| pair.0))
            .collect::<Option<Vec<f64>>>()
            .and_then(|values| values.into_iter().reduce(f64::min))
    }

    fn cell_passes(&self, arm: &str, cell: &str) -> bool {
        self.seeds.len() >= required_seeds(arm) && self.min(cell).is_some_and(|m| m >= PASS)
    }

    fn passes(&self, arm: &str, cells: &[String]) -> bool {
        cells.iter().all(|cell| self.cell_passes(arm, cell))
    }
}

/// Reads the sealed fact roots under `runs`, keyed by (pattern, arm).
fn collect(runs: &Path) -> Result<(BTreeMap<(String, String), ArmRuns>, Vec<Value>)> {
    let mut grouped: BTreeMap<(String, String), ArmRuns> = BTreeMap::new();
    let mut skipped = Vec::new();
    let mut seen = BTreeSet::new();
    let mut shared_config: Option<(Value, String)> = None;
    let mut entries: Vec<PathBuf> = fs::read_dir(runs)?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<std::io::Result<_>>()?;
    entries.sort();
    for root in entries.into_iter().filter(|path| path.is_dir()) {
        let name = root.display().to_string();
        if let Err(error) = report_output::verify(&root) {
            skipped.push(
                json!({"root": name, "reason": format!("not a verified sealed root: {error}")}),
            );
            continue;
        }
        let report: Value = serde_json::from_slice(&fs::read(root.join("report.json"))?)?;
        if report["schema"] != FACT_SCHEMA || report.get("mode").is_some() {
            skipped.push(json!({"root": name, "reason": "not a fact training report"}));
            continue;
        }
        if report["status"] != "complete" {
            skipped.push(
                json!({"root": name, "reason": "status is not complete", "error": report["error"]}),
            );
            continue;
        }
        // A seed counts only if it trained its full budget: a run cut short by
        // max_seconds is a different (smaller) experiment, not a seed of this one.
        let planned = report["task"]["common"]["steps"].as_u64();
        let completed = report["results"]["steps_completed"].as_u64();
        if report["results"]["stopped_early_at_max_seconds"] == true || completed != planned {
            skipped.push(json!({
                "root": name,
                "reason": "did not train its full step budget",
                "steps_completed": report["results"]["steps_completed"],
                "steps_planned": report["task"]["common"]["steps"],
            }));
            continue;
        }
        // Every counted seed must share one training configuration; only the
        // seed and the execution environment may differ.
        let fingerprint = training_fingerprint(&report);
        match &shared_config {
            None => shared_config = Some((fingerprint, name.clone())),
            Some((first, first_root)) if *first != fingerprint => {
                return Err(invalid(format!(
                    "{name} was trained with a different configuration than {first_root}; \
                     decide one grid at a time"
                )));
            }
            Some(_) => {}
        }
        let config = &report["arm"]["config"];
        let pattern = config["stack_config"]["pattern"]
            .as_str()
            .ok_or_else(|| invalid(format!("{name}: no pattern")))?
            .to_owned();
        let arm = config["lineage"]
            .as_str()
            .ok_or_else(|| invalid(format!("{name}: no lineage")))?
            .to_owned();
        let seed = report["task"]["common"]["seed"]
            .as_u64()
            .ok_or_else(|| invalid(format!("{name}: no seed")))?;
        if !seen.insert((pattern.clone(), arm.clone(), seed)) {
            return Err(invalid(format!(
                "two complete roots for pattern {pattern} arm {arm} seed {seed}; keep one"
            )));
        }
        let cells = seed_cells(&name, &report)?;
        grouped
            .entry((pattern, arm))
            .or_default()
            .seeds
            .push(SeedRun {
                seed,
                root: name,
                cells,
            });
    }
    for runs in grouped.values_mut() {
        runs.seeds.sort_by_key(|run| run.seed);
    }
    Ok((grouped, skipped))
}

/// The per-cell (model, rule) full-value accuracy of one fact report. A cell
/// without a numeric `full_accuracy` or `rule_full_accuracy` is an error: a
/// NaN would silently drop out of the seed minimum and the means.
fn seed_cells(name: &str, report: &Value) -> Result<BTreeMap<String, (f64, f64)>> {
    let accuracy = |cell: &str, record: &Value, field: &str| {
        record[field]
            .as_f64()
            .filter(|value| value.is_finite())
            .ok_or_else(|| invalid(format!("{name}: cell {cell} has no numeric {field}")))
    };
    report["results"]["final_in_class_fresh_pairings"]["cells"]
        .as_object()
        .ok_or_else(|| invalid(format!("{name}: no cells")))?
        .iter()
        .map(|(cell, record)| {
            Ok((
                cell.clone(),
                (
                    accuracy(cell, record, "full_accuracy")?,
                    accuracy(cell, record, "rule_full_accuracy")?,
                ),
            ))
        })
        .collect()
}

/// The parts of a fact report that define the experiment: the shared task
/// settings and the fact layout, without the per-run seed and execution
/// environment (device, threads, model saving, the wall-time cap, probe).
fn training_fingerprint(report: &Value) -> Value {
    let mut common = report["task"]["common"].clone();
    if let Some(map) = common.as_object_mut() {
        for key in [
            "seed",
            "threads",
            "device",
            "save_model",
            "max_seconds",
            "probe_steps",
            "probe_sequences",
        ] {
            map.remove(key);
        }
    }
    json!({"common": common, "fact": report["task"]["fact"]})
}

/// Rules 1-3 and 6 on `cells` for one pattern.
fn core_rules(arms: &BTreeMap<&str, &ArmRuns>, cells: &[String]) -> Value {
    let present = |arm: &str| arms.get(arm).copied();
    let passes = |arm: &str| present(arm).is_some_and(|runs| runs.passes(arm, cells));
    let complete =
        |arm: &str| present(arm).is_some_and(|runs| runs.seeds.len() >= required_seeds(arm));
    let missing: Vec<&str> = ["none", "f2", "conv"]
        .into_iter()
        .filter(|arm| !complete(arm))
        .collect();
    let none_passes = passes("none");
    let mut verdicts = Vec::new();
    if !missing.is_empty() {
        verdicts.push(format!(
            "INCOMPLETE: arms below their required seeds: {missing:?}"
        ));
    }
    let decided = missing.is_empty();
    // Rule 1 first; rules 1-3 only with every required seed of none, F2, conv.
    if !decided {
        verdicts.push("R1-R3 NOT_DECIDED: none, F2 and conv need all their seeds".into());
    } else if none_passes {
        verdicts.push("R1 NO_LAG_PROBLEM: none passes every decision cell on every seed; stop lineage research".into());
    } else if passes("f2") {
        verdicts.push(
            "R2 F2_SUFFICIENT: none fails, F2 passes every decision cell on every seed".into(),
        );
    }
    // Rule 3, only when none fails.
    let conv_gaps: Vec<Value> = cells
        .iter()
        .map(|cell| {
            let conv = present("conv").and_then(|r| r.mean(cell, 0));
            let f2 = present("f2").and_then(|r| r.mean(cell, 0));
            json!({"cell": cell, "conv_mean": conv, "f2_mean": f2,
                "f2_minus_conv": conv.zip(f2).map(|(c, f)| f - c)})
        })
        .collect();
    let diffs: Vec<Option<f64>> = conv_gaps
        .iter()
        .map(|gap| gap["f2_minus_conv"].as_f64())
        .collect();
    let literal_tie = diffs.iter().all(|d| d.is_some_and(|d| d.abs() <= TIE));
    let conv_never_worse = diffs.iter().all(|d| d.is_some_and(|d| d <= TIE));
    let rule3 = if !decided {
        None
    } else if none_passes {
        Some("R3 NOT_INFORMATIVE: none passes, so a conv tie cannot speak to j".to_string())
    } else if !passes("f2") {
        // Guard added to the frozen rule: a tie of two failing arms is a
        // shared failure, not evidence about j (the literal tie is recorded).
        Some(format!(
            "R3 NOT_INFORMATIVE: F2 does not pass, so the conv comparison cannot credit or retract j (literal tie within 0.03: {literal_tie})"
        ))
    } else if conv_never_worse {
        Some(format!(
            "R3 J_CLAIM_RETRACTED: none fails, F2 passes and the learned conv is never more than 0.03 below F2 on a decision cell (within 0.03 everywhere: {literal_tie}); F2 kept only as the cheapest exact form"
        ))
    } else {
        Some("R3 J_CLAIM_STANDS_OVER_CONV: none fails, F2 passes and the learned conv is more than 0.03 below F2 on some decision cell".into())
    };
    verdicts.extend(rule3);
    // Rule 6.
    let step2b: Vec<Value> = cells
        .iter()
        .filter(|_| complete("conv"))
        .filter(|cell| present("conv").is_some_and(|runs| !runs.cell_passes("conv", cell)))
        .filter_map(|cell| {
            let passing: Vec<&str> = LINEAGE_ARMS
                .into_iter()
                .filter(|arm| present(arm).is_some_and(|runs| runs.cell_passes(arm, cell)))
                .collect();
            (!passing.is_empty()).then(|| json!({"cell": cell, "lineage_arms_passing": passing}))
        })
        .collect();
    if !step2b.is_empty() {
        verdicts.push("R6 STEP_2B_JUSTIFIED: the learned conv fails a decision cell that a lineage arm passes".into());
    }
    json!({
        "verdicts": verdicts,
        "none_passes": none_passes,
        "f2_passes": passes("f2"),
        "conv_passes": passes("conv"),
        "qk_passes": passes("qk"),
        "conv_vs_f2": conv_gaps,
        "r3_literal_tie_within_0_03": literal_tie,
        "step2b_cells": step2b,
    })
}

fn pattern_table(arms: &BTreeMap<&str, &ArmRuns>, cells: &[String]) -> Value {
    let mut table = serde_json::Map::new();
    for (&arm, runs) in arms {
        let per_cell: serde_json::Map<String, Value> = cells
            .iter()
            .map(|cell| {
                let mean = runs.mean(cell, 0);
                let rule = runs.mean(cell, 1);
                (
                    cell.clone(),
                    json!({
                        "seeds": runs.seeds.iter()
                            .map(|run| run.cells.get(cell).map(|pair| pair.0)).collect::<Vec<_>>(),
                        "mean": mean, "min": runs.min(cell), "passes": runs.cell_passes(arm, cell),
                        "rule_mean": rule,
                        "meets_rule": mean.zip(rule).map(|(m, r)| m >= r - RULE_TOLERANCE),
                        "capability": runs.cell_passes(arm, cell)
                            && mean.zip(rule).is_some_and(|(m, r)| m >= r - RULE_TOLERANCE),
                        "beats_rule": mean.zip(rule).map(|(m, r)| m > r + RULE_TOLERANCE),
                    }),
                )
            })
            .collect();
        let count = |field: &str| per_cell.values().filter(|c| c[field] == true).count();
        let (meets, capability) = (count("meets_rule"), count("capability"));
        table.insert(
            arm.to_string(),
            json!({
                "seeds": runs.seeds.iter().map(|run| run.seed).collect::<Vec<_>>(),
                "roots": runs.seeds.iter().map(|run| run.root.clone()).collect::<Vec<_>>(),
                "required_seeds": required_seeds(arm),
                "passes_all": runs.passes(arm, &decision_cells(None)),
                "passes_rehearse": runs.passes(arm, &decision_cells(Some(Form::Rehearse))),
                "passes_bare": runs.passes(arm, &decision_cells(Some(Form::Bare))),
                "decision_cells_meeting_rule": meets,
                "decision_cells_capability": capability,
                "cells": per_cell,
            }),
        );
    }
    Value::Object(table)
}

fn decide(grouped: &BTreeMap<(String, String), ArmRuns>) -> Value {
    let all_cells = decision_cells(None);
    let mut patterns = serde_json::Map::new();
    let names: BTreeSet<&str> = grouped.keys().map(|(p, _)| p.as_str()).collect();
    for pattern in names {
        let arms: BTreeMap<&str, &ArmRuns> = grouped
            .iter()
            .filter(|((p, _), _)| p == pattern)
            .map(|((_, arm), runs)| (arm.as_str(), runs))
            .collect();
        let table = pattern_table(&arms, &all_cells);
        let entry = if PRODUCTION_PATTERNS.contains(&pattern) {
            let rehearse = decision_cells(Some(Form::Rehearse));
            let bare = decision_cells(Some(Form::Bare));
            let reply_form = !arms.is_empty()
                && arms
                    .iter()
                    .all(|(arm, runs)| runs.passes(arm, &rehearse) && !runs.passes(arm, &bare));
            let query_lineage: Vec<&String> = all_cells
                .iter()
                .filter(|cell| {
                    arms.get("qk").is_some_and(|r| r.cell_passes("qk", cell))
                        && !arms.get("f2").is_some_and(|r| r.cell_passes("f2", cell))
                })
                .collect();
            json!({
                "role": "production",
                "both_forms": core_rules(&arms, &all_cells),
                "rehearse_only": core_rules(&arms, &rehearse),
                "bare_only": core_rules(&arms, &bare),
                "r4_reply_form_gap": reply_form,
                "r7_query_lineage_cells": query_lineage,
                "arms": table,
            })
        } else {
            json!({
                "role": if DIAGNOSTIC_PATTERNS.contains(&pattern) { "diagnostic" } else { "other (not decided)" },
                "arms": table,
            })
        };
        patterns.insert(pattern.to_string(), entry);
    }
    let headline: Vec<String> = PRODUCTION_PATTERNS
        .iter()
        .map(|pattern| match patterns.get(*pattern) {
            None => format!("{pattern}: NO RUNS"),
            Some(entry) => format!(
                "{pattern}: {}{}",
                entry["both_forms"]["verdicts"]
                    .as_array()
                    .map(|v| v
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join("; "))
                    .unwrap_or_default(),
                if entry["r4_reply_form_gap"] == true {
                    "; R4 REPLY_FORM_GAP: every arm passes rehearse and fails bare"
                } else {
                    ""
                }
            ),
        })
        .collect();
    json!({
        "schema": DECISION_SCHEMA,
        "rules": {
            "pass": PASS, "tie": TIE, "rule_tolerance": RULE_TOLERANCE,
            "metric": "full-value accuracy, final in-class fresh pairings",
            "decision_cells": all_cells,
            "required_seeds": {"none": 3, "f2": 3, "qk": 3, "conv": 3, "qk_jj": 3, "other": 2},
            "production_patterns": PRODUCTION_PATTERNS,
            "diagnostic_patterns": DIAGNOSTIC_PATTERNS,
            "order": "R1 none passes (first) -> R2 F2 passes -> R3 conv tie only if none fails -> R4 reply form -> R5 rule comparison -> R6 Step 2b -> R7 query lineage",
        },
        "headline": headline,
        "patterns": patterns,
    })
}

fn markdown(decision: &Value, skipped: &[Value]) -> String {
    let mut out = String::from("# Step 2 parity bench decision table\n\n");
    for line in decision["headline"].as_array().into_iter().flatten() {
        out.push_str(&format!("- {}\n", line.as_str().unwrap_or_default()));
    }
    out.push_str("\nMetric: full-value accuracy on the final in-class fresh pairings. Decision cells: g >= 1, key pieces >= 2. `min` is the minimum over decision cells of the seed mean; `pass` needs every seed >= 0.95 on every decision cell with the required seeds.\n");
    out.push_str("\nR3 caveat: the conv arm is a learned width-4 causal depthwise convolution over keys k_t..k_{t-3}, so it reaches lags 1-3, while F2 (k_t + j k_{t-1}) reaches lag 1 only; a conv that ties or beats F2 has the wider receptive field, and its result is not a lag-1-only comparison.\n");
    for (pattern, entry) in decision["patterns"].as_object().into_iter().flatten() {
        out.push_str(&format!(
            "\n## {pattern} ({})\n\n| arm | seeds | min rehearse | min bare | pass rehearse | pass bare | cells meeting rule (of 12) | capability cells (of 12) |\n|---|---|---|---|---|---|---|---|\n",
            entry["role"].as_str().unwrap_or_default()
        ));
        for (arm, record) in entry["arms"].as_object().into_iter().flatten() {
            let min_of = |prefix: &str| {
                record["cells"]
                    .as_object()
                    .into_iter()
                    .flatten()
                    .filter(|(cell, _)| cell.starts_with(prefix))
                    .filter_map(|(_, c)| c["mean"].as_f64())
                    .fold(f64::INFINITY, f64::min)
            };
            out.push_str(&format!(
                "| {arm} | {} | {:.3} | {:.3} | {} | {} | {} | {} |\n",
                record["seeds"],
                min_of("rehearse"),
                min_of("bare"),
                record["passes_rehearse"],
                record["passes_bare"],
                record["decision_cells_meeting_rule"],
                record["decision_cells_capability"],
            ));
        }
        for scope in ["both_forms", "rehearse_only", "bare_only"] {
            if let Some(verdicts) = entry[scope]["verdicts"].as_array() {
                out.push_str(&format!("\n{scope}:\n"));
                for verdict in verdicts {
                    out.push_str(&format!("- {}\n", verdict.as_str().unwrap_or_default()));
                }
            }
        }
    }
    if !skipped.is_empty() {
        out.push_str(&format!(
            "\n{} roots skipped (see decision.json).\n",
            skipped.len()
        ));
    }
    out
}

pub(super) fn main(mut args: Args) -> Result<()> {
    let runs = PathBuf::from(
        args.take("runs")
            .ok_or_else(|| invalid("mode=decide needs runs=DIR"))?,
    );
    let out = PathBuf::from(
        args.take("out")
            .ok_or_else(|| invalid("out=NEW_REPORT_ROOT is required"))?,
    );
    args.finish()?;
    if !runs.is_dir() {
        return Err(invalid(format!(
            "runs={} is not a directory",
            runs.display()
        )));
    }
    if out.starts_with(&runs) {
        return Err(invalid("the decision root must not sit under runs="));
    }
    let (grouped, skipped) = collect(&runs)?;
    report_output::claim(&out)?;
    let mut decision = decide(&grouped);
    decision["runs"] = json!(runs);
    decision["skipped"] = json!(skipped);
    decision["argv"] = json!(std::env::args().collect::<Vec<_>>());
    write_json(&out.join("decision.json"), &decision)?;
    fs::write(out.join("decision.md"), markdown(&decision, &skipped))?;
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    eprintln!("{}", markdown(&decision, &skipped));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn runs(arm: &str, seeds: usize, value: impl Fn(&str) -> f64) -> ArmRuns {
        let cells: Vec<String> = decision_cells(None);
        ArmRuns {
            seeds: (0..seeds)
                .map(|seed| SeedRun {
                    seed: seed as u64,
                    root: format!("{arm}-{seed}"),
                    cells: cells.iter().map(|c| (c.clone(), (value(c), 1.0))).collect(),
                })
                .collect(),
        }
    }

    fn grouped(arms: Vec<(&str, ArmRuns)>) -> BTreeMap<(String, String), ArmRuns> {
        arms.into_iter()
            .map(|(arm, r)| (("rararr".to_string(), arm.to_string()), r))
            .collect()
    }

    fn verdicts(decision: &Value) -> String {
        decision["patterns"]["rararr"]["both_forms"]["verdicts"].to_string()
    }

    #[test]
    fn none_passing_is_decided_first_and_mutes_the_conv_tie() {
        let decision = decide(&grouped(vec![
            ("none", runs("none", 3, |_| 0.99)),
            ("f2", runs("f2", 3, |_| 0.99)),
            ("conv", runs("conv", 3, |_| 0.99)),
        ]));
        let text = verdicts(&decision);
        assert!(text.contains("R1 NO_LAG_PROBLEM"), "{text}");
        assert!(!text.contains("R2"), "{text}");
        assert!(text.contains("R3 NOT_INFORMATIVE"), "{text}");
    }

    #[test]
    fn f2_sufficiency_conv_tie_reply_form_and_step2b() {
        let bare_low = |c: &str| if c.starts_with("bare") { 0.5 } else { 0.99 };
        // none fails; F2 passes; the conv ties F2 everywhere.
        let decision = decide(&grouped(vec![
            ("none", runs("none", 3, |_| 0.6)),
            ("f2", runs("f2", 3, |_| 0.99)),
            ("conv", runs("conv", 3, |_| 0.98)),
        ]));
        let text = verdicts(&decision);
        assert!(text.contains("R2 F2_SUFFICIENT"), "{text}");
        assert!(text.contains("R3 J_CLAIM_RETRACTED"), "{text}");
        // The conv falls short: the claim stands, and Step 2b opens.
        let decision = decide(&grouped(vec![
            ("none", runs("none", 3, |_| 0.6)),
            ("f2", runs("f2", 3, |_| 0.99)),
            ("conv", runs("conv", 3, |_| 0.8)),
        ]));
        let text = verdicts(&decision);
        assert!(text.contains("R3 J_CLAIM_STANDS_OVER_CONV"), "{text}");
        assert!(text.contains("R6 STEP_2B_JUSTIFIED"), "{text}");
        // Two failing arms tying is not evidence about j (guard).
        let decision = decide(&grouped(vec![
            ("none", runs("none", 3, |_| 0.1)),
            ("f2", runs("f2", 3, |_| 0.1)),
            ("conv", runs("conv", 3, |_| 0.1)),
        ]));
        let text = verdicts(&decision);
        assert!(
            text.contains("R3 NOT_INFORMATIVE: F2 does not pass"),
            "{text}"
        );
        assert_eq!(
            decision["patterns"]["rararr"]["both_forms"]["r3_literal_tie_within_0_03"],
            true
        );
        // Every arm passes rehearse and fails bare: reply form.
        let decision = decide(&grouped(vec![
            ("none", runs("none", 3, bare_low)),
            ("f2", runs("f2", 3, bare_low)),
            ("conv", runs("conv", 3, bare_low)),
        ]));
        assert_eq!(decision["patterns"]["rararr"]["r4_reply_form_gap"], true);
        let rehearse = decision["patterns"]["rararr"]["rehearse_only"]["verdicts"].to_string();
        assert!(rehearse.contains("R1 NO_LAG_PROBLEM"), "{rehearse}");
        // Too few seeds never pass, and are flagged.
        let decision = decide(&grouped(vec![
            ("none", runs("none", 2, |_| 0.99)),
            ("f2", runs("f2", 3, |_| 0.99)),
            ("conv", runs("conv", 3, |_| 0.995)),
        ]));
        let text = verdicts(&decision);
        assert!(
            text.contains("INCOMPLETE") && text.contains("NOT_DECIDED") && !text.contains("R2 "),
            "{text}"
        );
        assert_eq!(
            decision["patterns"]["rararr"]["arms"]["none"]["passes_all"],
            false
        );
        // Within 0.01 of the reference rule (1.0 here) a cell counts; far
        // below it, none does.
        assert_eq!(
            decision["patterns"]["rararr"]["arms"]["conv"]["decision_cells_meeting_rule"],
            12
        );
        let decision = decide(&grouped(vec![("none", runs("none", 3, |_| 0.5))]));
        assert_eq!(
            decision["patterns"]["rararr"]["arms"]["none"]["decision_cells_meeting_rule"],
            0
        );
    }

    #[test]
    fn a_missing_or_non_numeric_accuracy_is_refused_not_dropped() {
        let report = |record: Value| {
            json!({"results": {"final_in_class_fresh_pairings": {"cells": {
                "rehearse.g1.k2": {"full_accuracy": 0.97, "rule_full_accuracy": 1.0},
                "bare.g1.k2": record,
            }}}})
        };
        let cells = seed_cells(
            "ok",
            &report(json!({"full_accuracy": 0.5, "rule_full_accuracy": 0.9})),
        )
        .expect("numeric cells parse");
        assert_eq!(cells["bare.g1.k2"], (0.5, 0.9));
        for record in [
            json!({"rule_full_accuracy": 0.9}),
            json!({"full_accuracy": null, "rule_full_accuracy": 0.9}),
            json!({"full_accuracy": "0.5", "rule_full_accuracy": 0.9}),
            json!({"full_accuracy": 0.5}),
        ] {
            let error = seed_cells("root-x", &report(record.clone()))
                .expect_err("a missing accuracy must be an error");
            let text = error.to_string();
            assert!(
                text.contains("root-x") && text.contains("bare.g1.k2"),
                "{record}: {text}"
            );
        }
    }

    #[test]
    fn the_decision_cells_are_the_g1_multi_piece_cells_of_both_forms() {
        let cells = decision_cells(None);
        assert_eq!(cells.len(), 12);
        assert!(cells.contains(&"rehearse.g1.k2".to_string()));
        assert!(cells.contains(&"bare.g3.k3".to_string()));
        assert!(!cells
            .iter()
            .any(|c| c.contains(".g0.") || c.ends_with(".k1")));
    }
}
