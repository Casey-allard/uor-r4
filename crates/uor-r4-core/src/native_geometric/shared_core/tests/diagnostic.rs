//! Opt-in, read-only diagnosis of a sealed shared-core attempt. This test-only
//! module does not change the artifact-bound implementation or fit parameters.
use super::super::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::Path;

type DiagnosticResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn write(out: &Path, name: &str, value: &Value) -> DiagnosticResult<()> {
    fs::File::create_new(out.join(name))?.write_all(&serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn loss(score: i16, right: bool) -> f64 {
    let s = f64::from(score) / 4.0;
    libm::log1p(libm::exp(if right { -s } else { s }))
}

fn score(model: &SharedCore, code: u16, landmark: u16) -> i16 {
    model.relative_score(code, landmark, &mut Work::default())
}

#[derive(Default)]
struct Node {
    depth: usize,
    codes: BTreeMap<u16, [usize; 2]>,
    current_errors: usize,
    ties: usize,
    nll: f64,
}

fn node_summary(model: &SharedCore, node: usize, stats: &Node) -> Value {
    let total: usize = stats.codes.values().map(|c| c[0] + c[1]).sum();
    let left: usize = stats.codes.values().map(|c| c[0]).sum();
    let collision_floor: usize = stats.codes.values().map(|c| c[0].min(c[1])).sum();
    let mut minimum_errors = usize::MAX;
    let mut minimum_nll = f64::INFINITY;
    // Exhaust the representable classifiers with the observed states/mix fixed.
    // This is a descriptive lower bound, not fitting or an artifact proposal.
    for landmark in 0..120 {
        let (mut errors, mut nll) = (0, 0.0);
        for (&code, counts) in &stats.codes {
            let s = score(model, code, landmark);
            errors += counts[usize::from(s <= 0)];
            nll += counts[0] as f64 * loss(s, false) + counts[1] as f64 * loss(s, true);
        }
        minimum_errors = minimum_errors.min(errors);
        minimum_nll = minimum_nll.min(nll);
    }
    json!({"node":node,"depth":stats.depth,"positions":total,"target_left":left,
        "unique_codes":stats.codes.len(),"code_conflict_error_floor":collision_floor,
        "current_errors":stats.current_errors,"current_score_ties":stats.ties,
        "current_nll":stats.nll,"minimum_legal_landmark_errors_fixed_states":minimum_errors,
        "minimum_legal_landmark_nll_fixed_states":minimum_nll,
        "constant_majority_errors":left.min(total-left),
        "constant_majority_is_implemented":false})
}

fn collision_summary<K: Ord>(groups: &BTreeMap<K, BTreeMap<u16, usize>>) -> Value {
    let positions: usize = groups.values().flat_map(|v| v.values()).sum();
    let majority_correct: usize = groups
        .values()
        .map(|v| v.values().max().copied().unwrap_or(0))
        .sum();
    json!({"positions":positions,"unique_keys":groups.len(),
        "conflicting_keys":groups.values().filter(|v|v.len()>1).count(),
        "empirical_lookup_error_floor":positions-majority_correct,
        "scope":"Observed equality conflicts only; low collision does not establish learnable semantics or transfer."})
}

fn leaf_losses(
    session: &mut CoreSession<'_>,
    lo: u16,
    hi: u16,
    node: usize,
    depth: usize,
    prefix: f64,
    losses: &mut [f64; 257],
) {
    if hi - lo == 1 {
        losses[usize::from(lo)] = prefix;
        return;
    }
    let mid = (lo + hi) >> 1;
    let s = session.branch_score(node, depth);
    leaf_losses(
        session,
        lo,
        mid,
        (node << 1) + 1,
        depth + 1,
        prefix + loss(s, false),
        losses,
    );
    leaf_losses(
        session,
        mid,
        hi,
        (node << 1) + 2,
        depth + 1,
        prefix + loss(s, true),
        losses,
    );
}

fn analyze(model: &SharedCore, documents: &[Vec<u8>]) -> DiagnosticResult<(Value, Value)> {
    let mut roots = BTreeMap::<[u16; 4], BTreeMap<u16, usize>>::new();
    let mut roots_after_input = BTreeMap::<[u16; 4], BTreeMap<u16, usize>>::new();
    let mut nodes = BTreeMap::<usize, Node>::new();
    let mut byte_counts = BTreeMap::<u16, [usize; 2]>::new();
    let mut first_error_by_depth = [0usize; 9];
    let mut rows = Vec::new();
    let (mut total, mut correct, mut total_nll, mut map_correct, mut map_differs) =
        (0, 0, 0.0, 0, 0);
    let mut predicted_non_text = 0;
    for (document_id, document) in documents.iter().enumerate() {
        let mut session = model.session(Intervention::Full);
        for (position, target) in document
            .iter()
            .map(|&b| u16::from(b))
            .chain(std::iter::once(EOS))
            .enumerate()
        {
            let state = session.state.clone();
            let predicted = session.predict();
            total += 1;
            correct += usize::from(predicted == target);
            predicted_non_text += usize::from(
                predicted != EOS
                    && predicted != 9
                    && predicted != 10
                    && !(32..=126).contains(&predicted),
            );
            *roots
                .entry(state.roots)
                .or_default()
                .entry(target)
                .or_default() += 1;
            if position > 0 {
                *roots_after_input
                    .entry(state.roots)
                    .or_default()
                    .entry(target)
                    .or_default() += 1;
            }
            let count = byte_counts.entry(target).or_default();
            count[0] += 1;
            count[1] += usize::from(target == predicted);
            let mut branches = Vec::new();
            let (mut lo, mut hi, mut node, mut depth) = (0_u16, EOS + 1, 0, 0);
            let mut first_error = None;
            while hi - lo > 1 {
                let mid = (lo + hi) >> 1;
                let right = target >= mid;
                let s = session.branch_score(node, depth);
                let mut work = Work::default();
                let lane = depth & 3;
                let mixed = model.product(
                    state.roots[lane],
                    model.artifact.parameters[OUTPUT_MIX + depth],
                    &mut work,
                );
                let code = model.product(mixed, state.roots[(lane + 1) & 3], &mut work);
                if score(model, code, model.artifact.parameters[OUTPUT + node]) != s {
                    return Err("Diagnostic code disagrees with actual branch score".into());
                }
                let nll = loss(s, right);
                total_nll += nll;
                let error = (s > 0) != right;
                if error && first_error.is_none() {
                    first_error = Some(depth);
                    first_error_by_depth[depth] += 1;
                }
                let stats = nodes.entry(node).or_default();
                stats.depth = depth;
                stats.codes.entry(code).or_default()[usize::from(right)] += 1;
                stats.current_errors += usize::from(error);
                stats.ties += usize::from(s == 0);
                stats.nll += nll;
                branches.push(
                    json!({"node":node,"depth":depth,"code":code,"score":s,"target_right":right}),
                );
                if right {
                    lo = mid;
                } else {
                    hi = mid;
                }
                node = (node << 1) + 1 + usize::from(right);
                depth += 1;
            }
            if first_error.is_none() != (predicted == target) {
                return Err("Target branch walk and runtime prediction disagree".into());
            }
            let mut probabilities = [0.0; 257];
            leaf_losses(&mut session, 0, EOS + 1, 0, 0, 0.0, &mut probabilities);
            let mode = probabilities
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.total_cmp(b.1))
                .ok_or("No leaf")?
                .0 as u16;
            let mass: f64 = probabilities.iter().map(|&v| libm::exp(-v)).sum();
            if (mass - 1.0).abs() > 1e-10 {
                return Err("Leaf probability mass is not one".into());
            }
            map_correct += usize::from(mode == target);
            map_differs += usize::from(mode != predicted);
            if session.state != state {
                return Err("Diagnostic prediction changed runtime state".into());
            }
            rows.push(
                json!({"document":document_id,"position":position,"target":target,
                "predicted":predicted,"leaf_probability_mode":mode,"roots":state.roots,
                "first_wrong_depth":first_error,"branches":branches}),
            );
            if target != EOS {
                session.observe(target as u8)?;
            }
        }
    }
    let node_reports: Vec<_> = nodes
        .iter()
        .map(|(&n, s)| node_summary(model, n, s))
        .collect();
    let byte_reports: Vec<_> = byte_counts
        .into_iter()
        .map(|(b, c)| json!({"byte_or_eos":b,"positions":c[0],"correct":c[1]}))
        .collect();
    Ok((
        json!({"positions":total,"correct":correct,"mean_nll":total_nll/total as f64,
        "root_state_collisions":collision_summary(&roots),"root_state_collisions_after_input":collision_summary(&roots_after_input),
        "first_wrong_depth":first_error_by_depth,"greedy_non_ascii_text_predictions":predicted_non_text,
        "leaf_probability_mode_correct":map_correct,"leaf_probability_mode_differs_from_greedy":map_differs,
        "mode_scope":"Host-only exhaustive decoder comparison, not a serving change or trained candidate.",
        "nodes":node_reports,"per_byte":byte_reports}),
        json!(rows),
    ))
}

fn execute(input: &Path, out: &Path) -> DiagnosticResult<()> {
    crate::report_output::verify(input)?;
    let design: Value = serde_json::from_slice(&fs::read(input.join("design.json"))?)?;
    let fit: Value = serde_json::from_slice(&fs::read(input.join("fit.json"))?)?;
    let original: Value = serde_json::from_slice(&fs::read(input.join("result.json"))?)?;
    let mut summaries = Vec::new();
    let mut parameters = Vec::new();
    for label in ["initial", "candidate"] {
        let bytes = fs::read(input.join(format!("{label}.json")))?;
        let model = SharedCore::from_bytes(&bytes)?;
        parameters.push(model.artifact.parameters.clone());
        for population in ["training", "holdout"] {
            let documents: Vec<Vec<u8>> = design[population]
                .as_array()
                .ok_or("Missing population")?
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(|s| s.as_bytes().to_vec())
                        .ok_or("Invalid document")
                })
                .collect::<std::result::Result<_, _>>()?;
            if documents.is_empty()
                || documents.len() > 64
                || documents.iter().any(|d| d.is_empty() || d.len() > 512)
                || documents.iter().map(|d| d.len() + 1).sum::<usize>() > 8192
            {
                return Err("Diagnostic data bounds".into());
            }
            let (summary, rows) = analyze(&model, &documents)?;
            let expected = match (label, population) {
                ("initial", "training") => &fit["before"],
                ("candidate", "training") => &fit["after"],
                ("initial", _) => &original["initial_holdout"],
                _ => &original["full"],
            };
            if summary["correct"] != expected["correct"]
                || summary["positions"] != expected["positions"]
                || (summary["mean_nll"].as_f64().ok_or("NLL")?
                    - expected["mean_nll"].as_f64().ok_or("Prior NLL")?)
                .abs()
                    > 1e-10
            {
                return Err("Diagnostic disagrees with sealed original metrics".into());
            }
            write(out, &format!("{label}-{population}-rows.json"), &rows)?;
            summaries.push(json!({"model":label,"population":population,"artifact":model.artifact_cid(),"metrics":summary}));
        }
        if model.to_bytes()? != bytes {
            return Err("Saved artifact mutated".into());
        }
    }
    let offsets = [
        EMBED, TRANSITION, QUERY, KEY, READ, PHASE, OUTPUT, OUTPUT_MIX, NULL, PARAMETERS,
    ];
    let differences: Vec<_> = offsets
        .windows(2)
        .map(|w| {
            (w[0]..w[1])
                .filter(|&i| parameters[0][i] != parameters[1][i])
                .count()
        })
        .collect();
    write(
        out,
        "result.json",
        &json!({"decision":"DIAGNOSTIC_COMPLETE_NO_REFIT","source_attempt":input,
        "artifact_bytes_unchanged":true,"original_metrics_reproduced":true,"summaries":summaries,
        "changed_parameters_by_family":differences,"parameter_family_order":["embedding","transition","query","key","read","phase","output","output_mix","null"],
        "limits":"Legal landmark minima condition on frozen states/mixes separately per node; they are not jointly trained, transferable or byte-accuracy predictions. No parameters were changed."}),
    )?;
    crate::report_output::verify(input)?;
    Ok(())
}

#[test]
#[ignore = "Requires a saved sealed attempt and an exclusive diagnostic output directory"]
fn run_saved_artifact_diagnostic() -> DiagnosticResult<()> {
    let input = std::env::var("UOR_SHARED_DIAGNOSTIC_INPUT")?;
    let output = std::env::var("UOR_SHARED_DIAGNOSTIC_OUTPUT")?;
    let out = Path::new(&output);
    separate_output(Path::new(&input), out)?;
    crate::report_output::claim(out)?;
    let result = execute(Path::new(&input), out);
    if let Err(e) = &result {
        write(out, "failure.json", &json!({"error":e.to_string()}))?;
    }
    crate::report_output::seal(out)?;
    crate::report_output::verify(out)?;
    result
}

/// Resolve aliases before claiming anything. Requiring an existing output
/// parent avoids creating directories beneath the sealed input during validation.
fn separate_output(input: &Path, output: &Path) -> DiagnosticResult<()> {
    let input = fs::canonicalize(input)?;
    let output = if output.exists() {
        fs::canonicalize(output)?
    } else {
        fs::canonicalize(output.parent().ok_or("Output needs an existing parent")?)?
            .join(output.file_name().ok_or("Output needs a directory name")?)
    };
    if output.starts_with(input) {
        return Err("Diagnostic output must be outside the sealed input".into());
    }
    Ok(())
}

#[cfg(test)]
mod checks {
    use super::*;
    #[test]
    fn signed_landmark_cannot_represent_constant_left_on_complete_root_domain() {
        let model = super::super::model();
        let stats = Node {
            codes: (0..120).map(|r| (r, [1, 0])).collect(),
            ..Node::default()
        };
        let report = node_summary(model, 0, &stats);
        assert!(
            report["minimum_legal_landmark_errors_fixed_states"]
                .as_u64()
                .unwrap()
                > 0
        );
        assert_eq!(report["code_conflict_error_floor"], 0);
        assert_eq!(report["constant_majority_errors"], 0);
    }
    #[test]
    fn collision_bound_counts_incompatible_targets_without_conflating_repeats() {
        let groups = BTreeMap::from([
            (0, BTreeMap::from([(2, 3), (4, 2)])),
            (1, BTreeMap::from([(2, 4)])),
        ]);
        let report = collision_summary(&groups);
        assert_eq!(report["positions"], 9);
        assert_eq!(report["conflicting_keys"], 1);
        assert_eq!(report["empirical_lookup_error_floor"], 2);
    }

    #[test]
    fn output_guard_preserves_sealed_input_and_rejects_aliases() -> DiagnosticResult<()> {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "uor-shared-path-check-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&root)?;
        let input = root.join("input");
        crate::report_output::claim(&input)?;
        crate::report_output::seal(&input)?;
        assert!(separate_output(&input, &input).is_err());
        assert!(separate_output(&input, &input.join("child")).is_err());
        #[cfg(unix)]
        {
            let alias = root.join("alias");
            std::os::unix::fs::symlink(&input, &alias)?;
            assert!(separate_output(&input, &alias).is_err());
            assert!(separate_output(&input, &alias.join("child")).is_err());
        }
        assert!(separate_output(&input, &root.join("outside")).is_ok());
        crate::report_output::verify(&input)?;
        assert!(!input.join("child").exists());
        fs::remove_dir_all(root)?; // This test's own empty fixture only.
        Ok(())
    }
}
