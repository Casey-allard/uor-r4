//! Opt-in exact hard-error capacity census on preserved development states.
//! This does not fit, mutate an artifact, or evaluate a fresh population.
use super::*;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

type AuditResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Deserialize)]
struct TraceBranch {
    node: usize,
    target_right: bool,
}

#[derive(Deserialize)]
struct TraceRow {
    document: usize,
    position: usize,
    target: u16,
    roots: [u16; LANES],
    branches: Vec<TraceBranch>,
}

struct Pattern {
    bits: Vec<u64>,
    landmark: u16,
    threshold: i16,
}

#[derive(Clone, Copy)]
struct Best {
    errors: usize,
    left: usize,
    right: usize,
}

fn write(out: &Path, name: &str, value: &Value) -> AuditResult<()> {
    fs::File::create_new(out.join(name))?.write_all(&serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn read_json(path: &Path) -> AuditResult<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn final_mask(positions: usize) -> u64 {
    let tail = positions & 63;
    if tail == 0 {
        u64::MAX
    } else {
        (1_u64 << tail) - 1
    }
}

fn count_errors(
    left: &[u64],
    right: &[u64],
    targets: &[u64],
    positions: usize,
    union: bool,
) -> usize {
    left.iter()
        .zip(right)
        .zip(targets)
        .enumerate()
        .map(|(word, ((&a, &b), &target))| {
            let prediction = if union { a | b } else { a & b };
            let mask = if word + 1 == targets.len() {
                final_mask(positions)
            } else {
                u64::MAX
            };
            ((prediction ^ target) & mask).count_ones() as usize
        })
        .sum()
}

fn patterns(model: &SharedCore, rows: &[([u16; LANES], bool)], lane: usize) -> Vec<Pattern> {
    let words = rows.len().div_ceil(64);
    let mut unique = BTreeMap::<Vec<u64>, (u16, i16)>::new();
    for landmark in 0..120 {
        let scores: Vec<_> = rows
            .iter()
            .map(|(roots, _)| model.relative_score(roots[lane], landmark, &mut Work::default()))
            .collect();
        for threshold in -5..=5 {
            let mut bits = vec![0_u64; words];
            for (position, &score) in scores.iter().enumerate() {
                if score > threshold {
                    bits[position >> 6] |= 1_u64 << (position & 63);
                }
            }
            // Equal hard predictions are equivalent for this error census.
            // Their likelihoods need not agree; loss is not optimized here.
            unique.entry(bits).or_insert((landmark, threshold));
        }
    }
    unique
        .into_iter()
        .map(|(bits, (landmark, threshold))| Pattern {
            bits,
            landmark,
            threshold,
        })
        .collect()
}

fn node_rows(traces: &[TraceRow], node: usize) -> AuditResult<Vec<([u16; LANES], bool)>> {
    let mut rows = Vec::new();
    for trace in traces {
        let included = node == 0 || trace.target < 128;
        let branches: Vec<_> = trace.branches.iter().filter(|b| b.node == node).collect();
        if !included {
            if !branches.is_empty() {
                return Err("Trace contains an impossible ASCII target-path node".into());
            }
            continue;
        }
        let right = trace.target >= if node == 0 { 128 } else { 64 };
        if branches.len() != 1 || branches[0].target_right != right {
            return Err("Saved trace node/target contract disagrees".into());
        }
        rows.push((trace.roots, right));
    }
    Ok(rows)
}

fn witness(
    model: &SharedCore,
    rows: &[([u16; LANES], bool)],
    depth: usize,
    left: &Pattern,
    right: &Pattern,
    union: bool,
    expected_errors: usize,
) -> AuditResult<Value> {
    let branch = CapBranch {
        landmarks: [left.landmark, right.landmark],
        thresholds: [left.threshold, right.threshold],
        union,
    };
    let (mut errors, mut nll) = (0usize, 0.0);
    for (position, &(roots, target)) in rows.iter().enumerate() {
        let score = model.cap_score(roots, branch, depth, &mut Work::default());
        let a = (left.bits[position >> 6] >> (position & 63)) & 1 != 0;
        let b = (right.bits[position >> 6] >> (position & 63)) & 1 != 0;
        if (score > 0) != if union { a || b } else { a && b } {
            return Err("Witness bitset formula disagrees with actual cap_score".into());
        }
        errors += usize::from((score > 0) != target);
        let margin = f64::from(score) / 4.0;
        nll += libm::log1p(libm::exp(if target { -margin } else { margin }));
    }
    if errors != expected_errors {
        return Err("Witness replay disagrees with enumerated optimum".into());
    }
    Ok(json!({
        "landmarks":branch.landmarks,"thresholds":branch.thresholds,"union":union,
        "errors":errors,"host_nll":nll,"host_mean_nll":nll/rows.len() as f64,
        "loss_is_minimized":false,
        "scope":"One exact hard-error witness; equivalent hard patterns can have different losses. No changed artifact is constructed."
    }))
}

fn audit_node(
    model: &SharedCore,
    rows: &[([u16; LANES], bool)],
    node: usize,
    depth: usize,
    expected_errors: usize,
) -> AuditResult<Value> {
    let current = model
        .artifact
        .calibrated
        .as_ref()
        .ok_or("Expected schema-2 calibrated artifact")?
        .branches[node];
    let lane = depth & 3;
    let mut targets = vec![0_u64; rows.len().div_ceil(64)];
    let (mut current_errors, mut rights, mut current_nll) = (0usize, 0usize, 0.0);
    for (position, &(roots, target)) in rows.iter().enumerate() {
        if target {
            targets[position >> 6] |= 1_u64 << (position & 63);
            rights += 1;
        }
        let a = model.relative_score(roots[lane], current.landmarks[0], &mut Work::default())
            > current.thresholds[0];
        let b = model.relative_score(
            roots[(lane + 1) & 3],
            current.landmarks[1],
            &mut Work::default(),
        ) > current.thresholds[1];
        let current_score = model.cap_score(roots, current, depth, &mut Work::default());
        let prediction = current_score > 0;
        let margin = f64::from(current_score) / 4.0;
        current_nll += libm::log1p(libm::exp(if target { -margin } else { margin }));
        if prediction != if current.union { a || b } else { a && b } {
            return Err("Current cap_score disagrees with separate operand formula".into());
        }
        current_errors += usize::from(prediction != target);
    }
    if current_errors != expected_errors {
        return Err("Current branch errors disagree with sealed calibration report".into());
    }
    let left = patterns(model, rows, lane);
    let right = patterns(model, rows, (lane + 1) & 3);
    let mut best = [Best {
        errors: usize::MAX,
        left: 0,
        right: 0,
    }; 2];
    let mut pairs = 0usize;
    for (i, a) in left.iter().enumerate() {
        for (j, b) in right.iter().enumerate() {
            pairs += 1;
            for (operator, chosen) in best.iter_mut().enumerate() {
                let errors = count_errors(&a.bits, &b.bits, &targets, rows.len(), operator == 1);
                if errors < chosen.errors {
                    *chosen = Best {
                        errors,
                        left: i,
                        right: j,
                    };
                }
            }
        }
    }
    let and_witness = witness(
        model,
        rows,
        depth,
        &left[best[0].left],
        &right[best[0].right],
        false,
        best[0].errors,
    )?;
    let or_witness = witness(
        model,
        rows,
        depth,
        &left[best[1].left],
        &right[best[1].right],
        true,
        best[1].errors,
    )?;
    let optimum = best[0].errors.min(best[1].errors);
    if optimum > current_errors || optimum > rights.min(rows.len() - rights) {
        return Err("Exhaustive class unexpectedly excludes current/constant classifier".into());
    }
    Ok(json!({
        "node":node,"depth":depth,"positions":rows.len(),"target_right":rights,
        "current_errors":current_errors,"current_host_nll":current_nll,"constant_majority_errors":rights.min(rows.len()-rights),
        "exact_minimum_hard_errors":optimum,
        "legal_settings_per_operand":1320,
        "distinct_left_patterns":left.len(),"distinct_right_patterns":right.len(),
        "distinct_pattern_pairs_examined":pairs,"operator_pair_combinations_examined":pairs*2,
        "and":and_witness,"or":or_witness,
        "scope":"Exact hard-error minimum over both landmarks, both thresholds and union/intersection, with saved roots fixed. Not trained performance, joint-model capacity or fresh transfer."
    }))
}

fn execute(model_path: &Path, rows_path: &Path, out: &Path) -> AuditResult<()> {
    if rows_path.file_name().and_then(|n| n.to_str()) != Some("candidate-training-rows.json") {
        return Err("Expected the sealed candidate-training trace file".into());
    }
    let model_root = model_path.parent().ok_or("Model report parent")?;
    let rows_root = rows_path.parent().ok_or("Trace report parent")?;
    crate::report_output::verify(model_root)?;
    crate::report_output::verify(rows_root)?;
    let model_bytes = fs::read(model_path)?;
    let model = SharedCore::from_bytes(&model_bytes)?;
    let upgraded_bytes = fs::read(model_root.join("upgraded.json"))?;
    let upgraded = SharedCore::from_bytes(&upgraded_bytes)?;
    let legacy_bytes = fs::read(model_root.join("legacy.json"))?;
    let legacy = SharedCore::from_bytes(&legacy_bytes)?;
    let representation = read_json(&model_root.join("result.json"))?;
    let calibration = read_json(&model_root.join("calibration.json"))?;
    let source_result = read_json(&rows_root.join("result.json"))?;
    let summaries = source_result["summaries"]
        .as_array()
        .ok_or("Trace summaries")?;
    let matching: Vec<_> = summaries
        .iter()
        .filter(|s| s["model"] == "candidate" && s["population"] == "training")
        .collect();
    if matching.len() != 1
        || matching[0]["artifact"] != legacy.artifact_cid()
        || representation["legacy"] != legacy.artifact_cid()
        || representation["upgraded"] != upgraded.artifact_cid()
        || representation["calibrated"] != model.artifact_cid()
        || calibration["parent"] != upgraded.artifact_cid()
        || model.artifact.parameters != upgraded.artifact.parameters
        || model.artifact.parameters != legacy.artifact.parameters
        || model.artifact.geometry != upgraded.artifact.geometry
        || model.artifact.geometry != legacy.artifact.geometry
        || model.artifact.angular != upgraded.artifact.angular
        || model.artifact.angular != legacy.artifact.angular
        || model
            .artifact
            .calibrated
            .as_ref()
            .ok_or("Calibrated emitter")?
            .parent
            != upgraded.artifact_cid()
        || upgraded
            .artifact
            .calibrated
            .as_ref()
            .ok_or("Upgraded emitter")?
            .parent
            != legacy.artifact_cid()
        || legacy.artifact.calibrated.is_some()
    {
        return Err("Saved roots and calibration artifact lineage disagree".into());
    }
    if fs::metadata(rows_path)?.len() > 16 * 1024 * 1024 {
        return Err("Trace byte bound".into());
    }
    let rows_bytes = fs::read(rows_path)?;
    let traces: Vec<TraceRow> = serde_json::from_slice(&rows_bytes)?;
    let mut identities = BTreeSet::new();
    if traces.len() != 672
        || traces.iter().any(|r| {
            r.target > EOS
                || r.roots.iter().any(|&v| v >= 120)
                || !identities.insert((r.document, r.position))
        })
    {
        return Err("Trace size, root, byte or occurrence identity bound".into());
    }
    let mut reports = Vec::new();
    for (node, depth, positions, position_key, error_key) in [
        (
            0usize,
            0usize,
            672usize,
            "root_positions",
            "root_branch_errors",
        ),
        (1, 1, 660, "ascii_positions", "ascii_branch_errors"),
    ] {
        let rows = node_rows(&traces, node)?;
        if rows.len() != positions || calibration[position_key].as_u64() != Some(positions as u64) {
            return Err("Frozen branch population disagrees with calibration".into());
        }
        let errors = usize::try_from(
            calibration[error_key]
                .as_u64()
                .ok_or("Calibration error count")?,
        )?;
        reports.push(audit_node(&model, &rows, node, depth, errors)?);
    }
    if model.to_bytes()? != model_bytes
        || upgraded.to_bytes()? != upgraded_bytes
        || legacy.to_bytes()? != legacy_bytes
    {
        return Err("Artifact roundtrip changed preserved bytes".into());
    }
    crate::report_output::verify(model_root)?;
    crate::report_output::verify(rows_root)?;
    write(
        out,
        "result.json",
        &json!({
            "decision":"EXACT_FROZEN_STATE_CAPACITY_COMPLETE_NO_REFIT",
            "artifact":model.artifact_cid(),"upgraded":upgraded.artifact_cid(),"legacy":legacy.artifact_cid(),
            "model_source":model_path,"trace_source":rows_path,
            "model_bytes_blake3":format!("blake3:{}",blake3::hash(&model_bytes)),
            "trace_bytes_blake3":format!("blake3:{}",blake3::hash(&rows_bytes)),
            "artifact_bytes_unchanged":true,"source_seals_verified":true,
            "current_calibration_counts_reproduced":true,"summaries":reports,
            "joint_fit":"NOT_RUN","fresh_evaluation":"NOT_RUN","promoted":false,
            "scope":"Exhaustive hard-error census on original development states. Witnesses are descriptive parameters only; no model is changed or saved and no new candidate is selected."
        }),
    )?;
    Ok(())
}

fn output_path(raw: &Path, model_root: &Path, rows_root: &Path) -> AuditResult<PathBuf> {
    let resolved = if raw.exists() {
        fs::canonicalize(raw)?
    } else {
        fs::canonicalize(raw.parent().ok_or("Output parent")?)?
            .join(raw.file_name().ok_or("Output directory name")?)
    };
    if resolved.starts_with(model_root) || resolved.starts_with(rows_root) {
        return Err("Output must be outside both sealed source reports".into());
    }
    Ok(resolved)
}

#[test]
#[ignore = "Requires preserved calibrated artifact, sealed trace and an exclusive output directory"]
fn run_exact_calibrated_capacity() -> AuditResult<()> {
    let model_path = fs::canonicalize(std::env::var("UOR_CALIBRATED_AUDIT_MODEL")?)?;
    let rows_path = fs::canonicalize(std::env::var("UOR_CALIBRATED_AUDIT_ROWS")?)?;
    let out = output_path(
        Path::new(&std::env::var("UOR_CALIBRATED_AUDIT_OUTPUT")?),
        model_path.parent().ok_or("Model parent")?,
        rows_path.parent().ok_or("Trace parent")?,
    )?;
    crate::report_output::claim(&out)?;
    let result = execute(&model_path, &rows_path, &out);
    if let Err(error) = &result {
        write(&out, "failure.json", &json!({"error":error.to_string()}))?;
    }
    crate::report_output::seal(&out)?;
    crate::report_output::verify(&out)?;
    result
}

#[test]
fn calibrated_bitset_errors_match_boolean_operations_and_ignore_padding() {
    for positions in [1usize, 63, 64, 65, 79, 128, 129] {
        let words = positions.div_ceil(64);
        let (mut a, mut b, mut targets) =
            (vec![0_u64; words], vec![0_u64; words], vec![0_u64; words]);
        let mut expected = [0usize; 2];
        for i in 0..positions {
            let left = i % 3 == 0;
            let right = i % 5 < 2;
            let target = i % 7 < 3;
            for (bits, set) in [(&mut a, left), (&mut b, right), (&mut targets, target)] {
                if set {
                    bits[i >> 6] |= 1_u64 << (i & 63);
                }
            }
            expected[0] += usize::from((left && right) != target);
            expected[1] += usize::from((left || right) != target);
        }
        // Deliberately poison unused prediction bits while keeping target padding zero.
        a[words - 1] |= !final_mask(positions);
        b[words - 1] |= !final_mask(positions);
        for (operator, count) in expected.into_iter().enumerate() {
            assert_eq!(
                count_errors(&a, &b, &targets, positions, operator == 1),
                count
            );
        }
    }
}
