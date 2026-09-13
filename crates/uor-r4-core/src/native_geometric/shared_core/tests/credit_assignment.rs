//! Development-only categorical credit prototype. Every cost is measured with
//! the real hard runtime. No straight-through derivative or serving change.
use super::*;
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, io::Write, path::Path, time::Instant};

type ProbeResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const EPS: f64 = 1e-9;

#[derive(Clone)]
struct Occurrence {
    document: usize,
    position: usize,
    state: State,
}

// Reconstruct only parameter addresses; observe itself always uses the runtime.
fn accesses(model: &SharedCore, before: &State, after: &State, byte: u8) -> Vec<usize> {
    let mut work = Work::default();
    let mut injected = [0; LANES];
    for lane in 0..LANES {
        let code = model.artifact.parameters[EMBED + (lane << 8) + usize::from(byte)];
        let phase =
            model.artifact.parameters[PHASE + (lane << 4) + usize::from(after.phases[lane] >> 12)];
        let root = model.product(before.roots[lane], code, &mut work);
        injected[lane] = model.product(root, phase, &mut work);
    }
    let mut indices = Vec::with_capacity(8);
    for lane in 0..LANES {
        let inv = model.artifact.geometry.inverses[usize::from(injected[(lane + 1) & 3])];
        let relative = model.product(injected[lane], inv, &mut work);
        indices.push(TRANSITION + model.artifact.geometry.row_bases[lane] + usize::from(relative));
    }
    if let Some(position) = after.last_source {
        let entry = before.ring[(position & 31) as usize];
        for lane in 0..LANES {
            indices.push(
                READ + model.artifact.geometry.row_bases[lane] + usize::from(entry.values[lane]),
            );
        }
    }
    indices
}

fn occurrences(model: &SharedCore, docs: &[Vec<u8>]) -> Result<BTreeMap<usize, Vec<Occurrence>>> {
    let mut result: BTreeMap<usize, Vec<Occurrence>> = BTreeMap::new();
    for (document, doc) in docs.iter().enumerate() {
        let mut session = model.session(Intervention::Full);
        for (position, &byte) in doc.iter().enumerate() {
            let state = session.state.clone();
            session.observe(byte)?;
            for index in accesses(model, &state, &session.state, byte) {
                result.entry(index).or_default().push(Occurrence {
                    document,
                    position,
                    state: state.clone(),
                });
            }
        }
    }
    Ok(result)
}

fn selected_indices(occurrences: &BTreeMap<usize, Vec<Occurrence>>) -> Vec<usize> {
    let mut selected = Vec::new();
    for offset in [TRANSITION, READ] {
        for lane in 0..LANES {
            // Stable ascending traversal retains the lowest index on equal count.
            let mut best = None;
            for (&index, sites) in
                occurrences.range(offset + lane * ROOTS..offset + (lane + 1) * ROOTS)
            {
                if best.is_none_or(|(_, count)| sites.len() > count) {
                    best = Some((index, sites.len()));
                }
            }
            if let Some((index, _)) = best {
                selected.push(index);
            }
        }
    }
    selected
}

// Host likelihood reconstruction calls the actual runtime branch score. Its sum
// is checked against public evaluate on both fixtures and the saved artifact.
fn target_nll(session: &mut CoreSession<'_>, target: u16) -> f64 {
    let (mut lo, mut hi, mut node, mut depth) = (0_u16, EOS + 1, 0, 0);
    let mut loss = 0.0;
    while hi - lo > 1 {
        let mid = (lo + hi) >> 1;
        let right = target >= mid;
        let score = f64::from(session.branch_score(node, depth)) / 4.0;
        loss += libm::log1p(libm::exp(if right { -score } else { score }));
        if right {
            lo = mid;
        } else {
            hi = mid;
        }
        node = (node << 1) + 1 + usize::from(right);
        depth += 1;
    }
    loss
}

fn suffix_cost(
    model: &SharedCore,
    changed: &SharedCore,
    site: &Occurrence,
    docs: &[Vec<u8>],
) -> Result<f64> {
    let doc = &docs[site.document];
    let mut session = CoreSession {
        model: changed,
        state: site.state.clone(),
        control: Intervention::Full,
        work: Work::default(),
    };
    // Only this occurrence uses changed parameters. Prior loss cannot be affected.
    session.observe(doc[site.position])?;
    session.model = model;
    let mut loss = 0.0;
    for target in doc[site.position + 1..]
        .iter()
        .map(|&b| u16::from(b))
        .chain(std::iter::once(EOS))
    {
        loss += target_nll(&mut session, target);
        if target != EOS {
            session.observe(target as u8)?;
        }
    }
    Ok(loss)
}

// Exact derivative of this finite categorical surrogate, not of tied hard loss.
fn categorical(logits: &[f64], costs: &[f64]) -> (f64, Vec<f64>) {
    let max = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let weights: Vec<_> = logits.iter().map(|x| libm::exp(x - max)).collect();
    let total: f64 = weights.iter().sum();
    let mean: f64 = weights.iter().zip(costs).map(|(w, c)| w * c / total).sum();
    let gradient = weights
        .iter()
        .zip(costs)
        .map(|(w, c)| w / total * (c - mean))
        .collect();
    (mean, gradient)
}

fn minimum(values: &[f64], current: usize) -> usize {
    let mut best = current;
    for (i, &v) in values.iter().enumerate() {
        if v < values[best] - EPS {
            best = i;
        }
    }
    best
}

fn deadline(start: Instant, seconds: u64) -> ProbeResult<()> {
    if start.elapsed().as_secs() >= seconds {
        return Err("credit probe time limit; no complete result".into());
    }
    Ok(())
}

fn probe(model: &SharedCore, docs: &[Vec<u8>], seconds: u64) -> ProbeResult<Value> {
    if docs.is_empty()
        || docs.len() > 4
        || docs.iter().any(|d| d.is_empty() || d.len() > 32)
        || !(1..=120).contains(&seconds)
    {
        return Err("credit probe bounds: 1..4 documents, 1..32 bytes each, 1..120 seconds".into());
    }
    let start = Instant::now();
    let before_bytes = model.to_bytes()?;
    let baseline = model.evaluate(docs, Intervention::Full)?;
    let positions = baseline.positions as f64;
    let baseline_loss = baseline.mean_nll * positions;
    let sites = occurrences(model, docs)?;
    let transition_sites: usize = sites
        .iter()
        .filter(|(i, _)| **i < QUERY)
        .map(|(_, v)| v.len())
        .sum();
    if transition_sites != LANES * docs.iter().map(Vec::len).sum::<usize>() {
        return Err("transition access census differs".into());
    }
    let mut reconstructed_loss = 0.0;
    for doc in docs {
        let mut session = model.session(Intervention::Full);
        for &byte in doc {
            reconstructed_loss += target_nll(&mut session, u16::from(byte));
            session.observe(byte)?;
        }
        reconstructed_loss += target_nll(&mut session, EOS);
    }
    if (reconstructed_loss - baseline_loss).abs() > EPS {
        return Err("host loss parity".into());
    }
    let indices = selected_indices(&sites);
    let mut rows = Vec::new();
    let (mut improvements, mut transition_improvements, mut read_improvements) = (0, 0, 0);
    let (mut chosen_gain, mut oracle_gain) = (0.0, 0.0);
    let (mut suffix_replays, mut global_replays) = (0, 0);
    for index in indices {
        let current = usize::from(model.artifact.parameters[index]);
        let selected_sites = &sites[&index];
        let baseline_suffixes: Vec<_> = selected_sites
            .iter()
            .map(|site| suffix_cost(model, model, site, docs))
            .collect::<Result<_>>()?;
        suffix_replays += selected_sites.len();
        let mut local = vec![0.0; ROOTS];
        let mut actual = vec![0.0; ROOTS];
        let mut changed = model.clone();
        for root in 0..ROOTS {
            deadline(start, seconds)?;
            changed.artifact.parameters[index] = root as u16;
            for (site, &base) in selected_sites.iter().zip(&baseline_suffixes) {
                local[root] += suffix_cost(model, &changed, site, docs)? - base;
                suffix_replays += 1;
            }
            actual[root] =
                changed.evaluate(docs, Intervention::Full)?.mean_nll * positions - baseline_loss;
            global_replays += 1;
        }
        if local[current].abs() > EPS || actual[current].abs() > EPS {
            return Err("unchanged replay differs from baseline".into());
        }
        let (expected_delta, gradient) = categorical(&[0.0; ROOTS], &local);
        let proposed = minimum(&local, current);
        let oracle = minimum(&actual, current);
        let improved = actual[proposed] < -EPS;
        let family = if index < QUERY { "transition" } else { "read" };
        improvements += usize::from(improved);
        if family == "transition" {
            transition_improvements += usize::from(improved);
        } else {
            read_improvements += usize::from(improved);
        }
        // Negative contribution is retained if the surrogate makes hard loss worse.
        chosen_gain -= actual[proposed];
        oracle_gain -= actual[oracle];
        let predicted_improving = local.iter().filter(|&&x| x < -EPS).count();
        let confirmed_improving = local
            .iter()
            .zip(&actual)
            .filter(|(a, b)| **a < -EPS && **b < -EPS)
            .count();
        rows.push(json!({"parameter":index,"family":family,
            "lane":(index - if family=="transition" {TRANSITION} else {READ}) / ROOTS,
            "occurrences":selected_sites.iter().map(|s|json!({"document":s.document,"position":s.position})).collect::<Vec<_>>(),
            "current_root":current,"proposed_root":proposed,"oracle_root":oracle,
            "surrogate_delta_by_root":local,"hard_delta_by_root":actual,
            "uniform_surrogate_expected_delta":expected_delta,"uniform_logit_gradient":gradient,
            "proposed_hard_delta":actual[proposed],"oracle_hard_delta":actual[oracle],
            "predicted_improving_roots":predicted_improving,"confirmed_improving_roots":confirmed_improving,
            "proposed_improves_hard_loss":improved}));
    }
    let fraction = if oracle_gain > EPS {
        Some(chosen_gain / oracle_gain)
    } else {
        None
    };
    let pass = rows.len() == 8
        && improvements >= 6
        && transition_improvements >= 2
        && read_improvements >= 2
        && fraction.is_some_and(|x| x >= 0.5);
    if model.to_bytes()? != before_bytes {
        return Err("parent mutated".into());
    }
    deadline(start, seconds)?;
    Ok(
        json!({"schema":"uor-r4.credit-assignment-result/1","parent":model.artifact_cid(),
        "decision":if pass {"PASS_OCCURRENCE_CREDIT_DIRECTION_PROBE"}else{"FAIL_OCCURRENCE_CREDIT_DIRECTION_PROBE"},
        "baseline":baseline,"rows":rows,"improving_rows":improvements,
        "transition_improving_rows":transition_improvements,"read_improving_rows":read_improvements,
        "sum_proposed_hard_gain":chosen_gain,"sum_oracle_hard_gain":oracle_gain,"oracle_gain_fraction":fraction,
        "suffix_replays":suffix_replays,"global_replays":global_replays,"elapsed_ms":start.elapsed().as_millis(),
        "parent_bytes_unchanged":true,"artifact_exported":false,"joint_fit_run":false,
        "scope":"Finite development intervention surrogate; not an unbiased tied-parameter gradient, language qualification, or efficient trainer."}),
    )
}

fn write(out: &Path, name: &str, value: &Value) -> ProbeResult<()> {
    fs::File::create_new(out.join(name))?.write_all(&serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

#[test]
fn credit_categorical_gradient_matches_finite_difference() {
    let logits = [0.3, -0.2, 1.1, 0.4];
    let costs = [2.0, -4.0, 0.5, 1.2];
    let (_, gradient) = categorical(&logits, &costs);
    for i in 0..logits.len() {
        let (mut a, mut b) = (logits, logits);
        a[i] += 1e-5;
        b[i] -= 1e-5;
        let finite = (categorical(&a, &costs).0 - categorical(&b, &costs).0) / 2e-5;
        assert!((finite - gradient[i]).abs() < 1e-8);
    }
    let shifted: Vec<_> = costs.iter().map(|c| c + 40.0).collect();
    for (a, b) in gradient.iter().zip(categorical(&logits, &shifted).1) {
        assert!((a - b).abs() < 1e-12);
    }
    assert!(gradient.iter().sum::<f64>().abs() < 1e-12);
    assert_eq!(minimum(&[1.0, 1.0, 1.0], 2), 2);
}

#[test]
fn credit_access_addresses_and_single_use_replay_are_causal() {
    let model = model();
    let docs = vec![b"ababa".to_vec()];
    let sites = occurrences(model, &docs).unwrap();
    let baseline = model.evaluate(&docs, Intervention::Full).unwrap();
    for (&index, occurrences) in &sites {
        for site in occurrences {
            let mut changed = model.clone();
            changed.artifact.parameters[index] =
                ((usize::from(model.artifact.parameters[index]) + 1) % 120) as u16;
            let mut original = CoreSession {
                model,
                state: site.state.clone(),
                control: Intervention::Full,
                work: Work::default(),
            };
            let mut alternative = CoreSession {
                model: &changed,
                state: site.state.clone(),
                control: Intervention::Full,
                work: Work::default(),
            };
            original.observe(docs[0][site.position]).unwrap();
            alternative.observe(docs[0][site.position]).unwrap();
            // A used group operand change must change its lane: finite products cancel.
            assert_ne!(original.state.roots, alternative.state.roots);
            let base_suffix = suffix_cost(model, model, site, &docs).unwrap();
            let mut full = model.session(Intervention::Full);
            let mut prefix = 0.0;
            for &b in &docs[0][..=site.position] {
                prefix += target_nll(&mut full, u16::from(b));
                full.observe(b).unwrap();
            }
            assert!(
                (prefix + base_suffix - baseline.mean_nll * baseline.positions as f64).abs() < 1e-9
            );
        }
    }
    // Exactly one access makes local replacement and tied replacement identical.
    let mut comparisons = 0;
    for (&index, occurrences) in sites.iter().filter(|(_, v)| v.len() == 1) {
        let mut changed = model.clone();
        changed.artifact.parameters[index] = (model.artifact.parameters[index] + 1) % 120;
        let local = suffix_cost(model, &changed, &occurrences[0], &docs).unwrap()
            - suffix_cost(model, model, &occurrences[0], &docs).unwrap();
        let global = (changed
            .evaluate(&docs, Intervention::Full)
            .unwrap()
            .mean_nll
            - baseline.mean_nll)
            * baseline.positions as f64;
        // Future rerouting may newly access this parameter. Only assert equality
        // when the changed trajectory also uses it exactly once.
        if self::occurrences(&changed, &docs)
            .unwrap()
            .get(&index)
            .is_some_and(|v| v.len() == 1)
        {
            assert!((local - global).abs() < 1e-8);
            comparisons += 1;
        }
    }
    assert!(comparisons > 0, "single-use comparison must execute");
}

#[test]
fn credit_probe_rejects_unbounded_inputs() {
    assert!(probe(model(), &[vec![0; 33]], 1).is_err());
    assert!(probe(model(), &[vec![0]], 0).is_err());
    assert!(probe(model(), &[], 1).is_err());
}

#[test]
#[ignore = "explicit parent, frozen design, exclusively claimed report required"]
fn credit_run_saved_artifact_probe() -> ProbeResult<()> {
    let input = fs::canonicalize(std::env::var("UOR_CREDIT_INPUT")?)?;
    let out_raw = std::path::PathBuf::from(std::env::var("UOR_CREDIT_OUTPUT")?);
    let out = fs::canonicalize(out_raw.parent().ok_or("output parent")?)?
        .join(out_raw.file_name().ok_or("output name")?);
    if out.starts_with(&input) {
        return Err("output beneath sealed input".into());
    }
    let design_path = fs::canonicalize(std::env::var("UOR_CREDIT_DESIGN")?)?;
    crate::report_output::claim(&out)?;
    let result = (|| -> ProbeResult<()> {
        crate::report_output::verify(&input)?;
        if fs::metadata(&design_path)?.len() > 65536 {
            return Err("design size".into());
        }
        let raw = fs::read(&design_path)?;
        let design: Value = serde_json::from_slice(&raw)?;
        if design["schema"] != "uor-r4.credit-assignment-probe/1"
            || design["root_alternatives"] != 120
            || design["max_seconds"] != 120
            || design["gate"]
                != json!({"expected_rows":8,"min_improving":6,"min_improving_per_family":2,"min_fraction_of_total_oracle_gain":0.5})
        {
            return Err("design contract".into());
        }
        write(&out, "design.json", &design)?;
        write(
            &out,
            "source.json",
            &json!({"design_blake3":blake3::hash(&raw).to_hex().to_string(),
            "probe_source_blake3":blake3::hash(include_bytes!("credit_assignment.rs")).to_hex().to_string()}),
        )?;
        let bytes = fs::read(input.join("candidate.json"))?;
        let model = SharedCore::from_bytes(&bytes)?;
        if model.to_bytes()? != bytes || design["parent"] != model.artifact_cid() {
            return Err("parent identity".into());
        }
        let docs = design["documents"]
            .as_array()
            .ok_or("documents")?
            .iter()
            .map(|d| d.as_str().map(|s| s.as_bytes().to_vec()).ok_or("document"))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let report = probe(&model, &docs, 120)?;
        crate::report_output::verify(&input)?;
        if model.to_bytes()? != bytes {
            return Err("parent changed".into());
        }
        write(&out, "result.json", &report)?;
        println!(
            "{}",
            json!({"decision":report["decision"],"improving_rows":report["improving_rows"],"oracle_gain_fraction":report["oracle_gain_fraction"]})
        );
        Ok(())
    })();
    if let Err(error) = &result {
        write(
            &out,
            "failure.json",
            &json!({"status":"INCOMPLETE","error":error.to_string()}),
        )?;
    }
    crate::report_output::seal(&out)?;
    crate::report_output::verify(&out)?;
    result
}

// Follow-up trace of the one demonstrated harmful proposal; no search or fit.
fn trace(
    model: &SharedCore,
    docs: &[Vec<u8>],
    single: Option<(usize, u16, usize, usize)>,
) -> Result<Vec<Value>> {
    let mut changed = model.clone();
    if let Some((index, root, _, _)) = single {
        changed.artifact.parameters[index] = root;
    }
    let mut rows = Vec::new();
    for (document, doc) in docs.iter().enumerate() {
        let mut session = model.session(Intervention::Full);
        let mut last_accesses = Vec::new();
        for (position, target) in doc
            .iter()
            .map(|&b| u16::from(b))
            .chain(std::iter::once(EOS))
            .enumerate()
        {
            rows.push(
                json!({"document":document,"position":position,"target":target,
                "roots":session.state.roots,"source":session.last_source(),
                "prediction":session.predict(),"nll":target_nll(&mut session,target),
                "previous_observe_accesses":last_accesses}),
            );
            if target != EOS {
                let before = session.state.clone();
                session.model = if single.is_some_and(|(_, _, d, p)| d == document && p == position)
                {
                    &changed
                } else {
                    model
                };
                session.observe(target as u8)?;
                session.model = model;
                last_accesses = accesses(model, &before, &session.state, target as u8);
            }
        }
    }
    Ok(rows)
}

fn trace_difference(base: &[Value], other: &[Value]) -> Value {
    let (mut roots, mut sources, mut predictions, mut root_positions) = (0, 0, 0, 0);
    let mut first = None;
    for (a, b) in base.iter().zip(other) {
        let count = (0..LANES)
            .filter(|&lane| a["roots"][lane] != b["roots"][lane])
            .count();
        roots += count;
        root_positions += usize::from(count > 0);
        sources += usize::from(a["source"] != b["source"]);
        predictions += usize::from(a["prediction"] != b["prediction"]);
        if first.is_none() && (count > 0 || a["source"] != b["source"]) {
            first = Some(json!({"document":a["document"],"position":a["position"]}));
        }
    }
    json!({"positions":base.len(),"root_slots":base.len()*LANES,
        "root_id_hamming":roots,"positions_with_root_change":root_positions,
        "selected_source_hamming":sources,"predicted_token_hamming":predictions,
        "first_state_or_source_difference":first,
        "nll_delta":other.iter().map(|x|x["nll"].as_f64().unwrap_or(f64::NAN)).sum::<f64>()-base.iter().map(|x|x["nll"].as_f64().unwrap_or(f64::NAN)).sum::<f64>()})
}

#[test]
fn credit_trace_unchanged_control_and_likelihood_parity() {
    let docs = vec![b"ababa".to_vec()];
    let m = model();
    let a = trace(m, &docs, None).unwrap();
    let index = TRANSITION;
    let b = trace(m, &docs, Some((index, m.artifact.parameters[index], 0, 0))).unwrap();
    assert_eq!(a, b);
    let metrics = m.evaluate(&docs, Intervention::Full).unwrap();
    assert!(
        (a.iter().map(|x| x["nll"].as_f64().unwrap()).sum::<f64>()
            - metrics.mean_nll * metrics.positions as f64)
            .abs()
            < EPS
    );
    assert_eq!(trace_difference(&a, &b)["root_id_hamming"], 0);
}

#[test]
#[ignore = "explicit sealed failed probe and new trace attempt required"]
fn credit_trace_failed_read_proposal() -> ProbeResult<()> {
    let input = fs::canonicalize(std::env::var("UOR_CREDIT_INPUT")?)?;
    let prior = fs::canonicalize(std::env::var("UOR_CREDIT_PRIOR")?)?;
    let raw = std::path::PathBuf::from(std::env::var("UOR_CREDIT_OUTPUT")?);
    let out = fs::canonicalize(raw.parent().ok_or("output parent")?)?
        .join(raw.file_name().ok_or("output name")?);
    if out.starts_with(&input) || out.starts_with(&prior) {
        return Err("output under sealed root".into());
    }
    crate::report_output::claim(&out)?;
    let result = (|| -> ProbeResult<()> {
        crate::report_output::verify(&input)?;
        crate::report_output::verify(&prior)?;
        let raw_model = fs::read(input.join("candidate.json"))?;
        let model = SharedCore::from_bytes(&raw_model)?;
        let design: Value = serde_json::from_slice(&fs::read(prior.join("design.json"))?)?;
        let report: Value = serde_json::from_slice(&fs::read(prior.join("result.json"))?)?;
        if report["decision"] != "FAIL_OCCURRENCE_CREDIT_DIRECTION_PROBE"
            || report["parent"] != model.artifact_cid()
            || design["parent"] != model.artifact_cid()
        {
            return Err("failed parent probe required".into());
        }
        let docs = design["documents"]
            .as_array()
            .ok_or("documents")?
            .iter()
            .map(|d| d.as_str().map(|s| s.as_bytes().to_vec()).ok_or("document"))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if docs.len() != 4 || docs.iter().any(|d| d.len() != 32) {
            return Err("frozen trace data bounds".into());
        }
        let row = report["rows"]
            .as_array()
            .ok_or("rows")?
            .iter()
            .find(|r| r["parameter"] == 1930)
            .ok_or("read parameter")?;
        if row["proposed_root"] != 26
            || row["oracle_root"] != 76
            || row["proposed_hard_delta"].as_f64().is_none_or(|x| x <= 0.0)
        {
            return Err("demonstrated failure differs".into());
        }
        let base = trace(&model, &docs, None)?;
        let mut comparisons = Vec::new();
        for root in [26, 76] {
            let mut changed = model.clone();
            changed.artifact.parameters[1930] = root;
            let full = trace(&changed, &docs, None)?;
            let full_delta = trace_difference(&base, &full);
            if (full_delta["nll_delta"].as_f64().ok_or("delta")?
                - row["hard_delta_by_root"][usize::from(root)]
                    .as_f64()
                    .ok_or("prior delta")?)
            .abs()
                > EPS
            {
                return Err("prior tied replay mismatch".into());
            }
            let mut individual = Vec::new();
            for site in row["occurrences"].as_array().ok_or("sites")? {
                let d = site["document"].as_u64().ok_or("site doc")? as usize;
                let p = site["position"].as_u64().ok_or("site pos")? as usize;
                if d >= docs.len() || p >= docs[d].len() {
                    return Err("site bounds".into());
                }
                let one = trace(&model, &docs, Some((1930, root, d, p)))?;
                individual.push(json!({"document":d,"position":p,"difference":trace_difference(&base,&one),"trace":one}));
            }
            let sum: f64 = individual
                .iter()
                .map(|x| x["difference"]["nll_delta"].as_f64().unwrap_or(f64::NAN))
                .sum();
            if (sum
                - row["surrogate_delta_by_root"][usize::from(root)]
                    .as_f64()
                    .ok_or("prior surrogate")?)
            .abs()
                > EPS
            {
                return Err("prior occurrence replay mismatch".into());
            }
            comparisons.push(
                json!({"replacement_root":root,"full_difference":full_delta,"full_trace":full,
                "individual_occurrences":individual,"sum_individual_nll_delta":sum,
                "interaction_residual":full_delta["nll_delta"].as_f64().ok_or("full delta")?-sum}),
            );
        }
        if model.to_bytes()? != raw_model {
            return Err("parent changed".into());
        }
        crate::report_output::verify(&input)?;
        crate::report_output::verify(&prior)?;
        write(
            &out,
            "source.json",
            &json!({"probe_source_blake3":blake3::hash(include_bytes!("credit_assignment.rs")).to_hex().to_string(),"parent_blake3":blake3::hash(&raw_model).to_hex().to_string(),"prior_manifest_blake3":blake3::hash(&fs::read(prior.join("manifest.json"))?).to_hex().to_string()}),
        )?;
        write(
            &out,
            "result.json",
            &json!({"schema":"uor-r4.credit-trace/1","decision":"FAILED_PROPOSAL_TRACE_COMPLETE_NO_FIT",
            "parent":model.artifact_cid(),"parameter":1930,"baseline_trace":base,"comparisons":comparisons,
            "scope":"Aligned categorical root/source/output Hamming, not bit distances of IDs. Full and individual interventions reproduce sealed prior losses; no artifact export or new holdout."}),
        )?;

        Ok(())
    })();
    if let Err(e) = &result {
        write(
            &out,
            "failure.json",
            &json!({"status":"INCOMPLETE","error":e.to_string()}),
        )?;
    }
    crate::report_output::seal(&out)?;
    crate::report_output::verify(&out)?;
    result
}
