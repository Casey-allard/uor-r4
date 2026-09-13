//! Active-entry single-coordinate sensitivity with a fixed decoder; test-only witnesses.
use super::*;
use serde_json::{json, Value};
use sha2::Digest as _;
use std::{fs, io::Write, path::Path, time::Instant};
type RunResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const EPS: f64 = 1e-10;
const PARENT: &str = "blake3:8f8e5c342c83b062274744033a96668e9c079dc47a9c4d1b138baa7ea4889b6a";
const PRIOR_TREE: &str = "blake3:1198ef4788397b9bf8cae20b8df4b23573bc9861eb812a51baf359f2545618b1";
const CAP: &str = "blake3:0f82b7286bb1699b9ea3bf4193f152702f3b22e702cf31bcd0a0f56596bbf8b8";
const OLDER: &str = "blake3:ade9a1cfbf39f0828ec1ec2b53775c05fcbaaf53fa2189b6a37ab7a636ce5e58";
const REUSE_INDICES: [usize; 2] = [1176, 1930];
const CONTROLS: [Intervention; 3] = [
    Intervention::Full,
    Intervention::ContextDisabled,
    Intervention::StateDisabled,
];
fn digest(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}
fn documents(value: &Value) -> RunResult<Vec<Vec<u8>>> {
    value
        .as_array()
        .ok_or("documents")?
        .iter()
        .map(|v| Ok(v.as_str().ok_or("document")?.as_bytes().to_vec()))
        .collect()
}
fn write(out: &Path, name: &str, value: &impl Serialize) -> RunResult<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    let existing = fs::read_dir(out)?.try_fold(0_u64, |sum, e| -> std::io::Result<u64> {
        Ok(sum + e?.metadata()?.len())
    })?;
    if existing + bytes.len() as u64 > 64 * 1024 * 1024 - 128 * 1024 {
        return Err("report write exceeds storage limit with seal reserve".into());
    }
    fs::File::create_new(out.join(name))?.write_all(&bytes)?;
    Ok(())
}
fn guard(start: Instant) -> RunResult<()> {
    if start.elapsed().as_secs() >= 180 {
        return Err("complete active sensitivity experiment time guard".into());
    }
    Ok(())
}
#[derive(Clone, Serialize)]
struct Row {
    document: usize,
    position: usize,
    target: u16,
    prediction: u16,
    nll: f64,
    roots: [u16; LANES],
    source: Option<u64>,
}
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
fn rows(model: &SharedCore, docs: &[Vec<u8>], control: Intervention) -> Result<Vec<Row>> {
    let mut result = Vec::new();
    for (document, doc) in docs.iter().enumerate() {
        let mut session = model.session(control);
        for (position, target) in doc
            .iter()
            .map(|&b| u16::from(b))
            .chain(std::iter::once(EOS))
            .enumerate()
        {
            let prediction = session.predict();
            let nll = target_nll(&mut session, target);
            result.push(Row {
                document,
                position,
                target,
                prediction,
                nll,
                roots: session.state_roots(),
                source: session.last_source(),
            });
            if target != EOS {
                session.observe(target as u8)?;
            }
        }
    }
    Ok(result)
}
fn compare(base: &[Row], changed: &[Row]) -> RunResult<Value> {
    if base.len() != changed.len() {
        return Err("row count differs".into());
    }
    let (mut lost, mut gained, mut predictions, mut sources, mut roots) = (0, 0, 0, 0, 0);
    let mut deltas = Vec::with_capacity(base.len());
    for (a, b) in base.iter().zip(changed) {
        if (a.document, a.position, a.target) != (b.document, b.position, b.target) {
            return Err("row alignment differs".into());
        }
        if !a.nll.is_finite() || !b.nll.is_finite() {
            return Err("nonfinite row loss".into());
        }
        let correct_to_wrong = a.prediction == a.target && b.prediction != b.target;
        let wrong_to_correct = a.prediction != a.target && b.prediction == b.target;
        let root_delta = a
            .roots
            .iter()
            .zip(b.roots)
            .filter(|(x, y)| **x != *y)
            .count();
        lost += usize::from(correct_to_wrong);
        gained += usize::from(wrong_to_correct);
        predictions += usize::from(a.prediction != b.prediction);
        sources += usize::from(a.source != b.source);
        roots += root_delta;
        deltas.push(json!({"document":a.document,"position":a.position,"target":a.target,"parent_prediction":a.prediction,"candidate_prediction":b.prediction,"parent_nll":a.nll,"candidate_nll":b.nll,"nll_delta":b.nll-a.nll,"correct_to_wrong":correct_to_wrong,"wrong_to_correct":wrong_to_correct,"root_identity_hamming":root_delta,"source_identity_hamming":usize::from(a.source!=b.source)}));
    }
    Ok(
        json!({"positions":base.len(),"correct_to_wrong":lost,"wrong_to_correct":gained,"prediction_hamming":predictions,"source_identity_hamming":sources,"root_identity_hamming":roots,"hamming_scope":"Aligned categorical identities, never bit Hamming or a geometric metric.","rows":deltas}),
    )
}
fn verified_metrics(
    model: &SharedCore,
    docs: &[Vec<u8>],
    control: Intervention,
    rows: &[Row],
) -> RunResult<Metrics> {
    let metrics = model.evaluate(docs, control)?;
    if rows.is_empty()
        || (rows.iter().map(|r| r.nll).sum::<f64>() / rows.len() as f64 - metrics.mean_nll).abs()
            > EPS
        || rows.iter().filter(|r| r.prediction == r.target).count() != metrics.correct
    {
        return Err("row/evaluate mismatch".into());
    }
    Ok(metrics)
}
fn generate(
    model: &SharedCore,
    prompts: &[String],
    controls: &[Intervention],
) -> Result<Vec<Value>> {
    let mut outputs = Vec::new();
    for prompt in prompts {
        for &control in controls {
            let (bytes, eos, work) = model.generate(prompt.as_bytes(), 96, control)?;
            outputs.push(json!({"prompt":prompt,"control":control,"bytes":bytes,"text":String::from_utf8_lossy(&bytes),"eos":eos,"work":work}));
        }
    }
    Ok(outputs)
}

fn saved_rows_equal(saved: &Value, actual: &[Row], before: &Metrics) -> RunResult<()> {
    let metrics: Metrics = serde_json::from_value(saved["metrics"].clone())?;
    if metrics.positions != before.positions
        || metrics.correct != before.correct
        || (metrics.mean_nll - before.mean_nll).abs() > EPS
    {
        return Err("saved artifact metrics changed".into());
    }
    let saved = saved["rows"].as_array().ok_or("saved rows")?;
    if saved.len() != actual.len() {
        return Err("saved row count".into());
    }
    for (a, b) in saved.iter().zip(actual) {
        if a["document"] != b.document
            || a["position"] != b.position
            || a["prediction"] != b.prediction
            || a["target"] != b.target
            || a["roots"] != json!(b.roots)
            || a["source"] != json!(b.source)
            || (a["nll"].as_f64().ok_or("saved NLL")? - b.nll).abs() > EPS
        {
            return Err("saved artifact row changed".into());
        }
    }
    Ok(())
}
fn driver_digest() -> String {
    digest(include_bytes!("active_sensitivity.rs"))
}
fn select(costs: &[f64], parent_index: usize) -> RunResult<usize> {
    if parent_index >= costs.len() || costs.iter().any(|v| !v.is_finite() || *v < 0.0) {
        return Err("invalid exhaustive costs".into());
    }
    let minimum = costs.iter().copied().fold(f64::INFINITY, f64::min);
    if costs[parent_index] <= minimum + EPS {
        return Ok(parent_index);
    }
    costs
        .iter()
        .position(|&v| v <= minimum + EPS)
        .ok_or_else(|| "no minimum".into())
}

// Test-only mirror of observe: native parameter/product/score primitives remain
// authoritative. Every step is checked against the actual native observe path.
// Production source is unchanged; this mirror is bound by driver_digest().
fn traced_parameter(model: &SharedCore, visits: &mut [u64], at: usize, work: &mut Work) -> u16 {
    visits[at] += 1;
    model.parameter(at, work)
}
fn traced_observe(session: &mut CoreSession<'_>, byte: u8, visits: &mut [u64]) -> Result<()> {
    if session.state.seen == u64::MAX {
        return Err(CoreError::InvalidInput("position exhausted"));
    }
    let m = session.model;
    let identity = m.artifact.geometry.identity;
    let mut injected = [identity; LANES];
    for (lane, target) in injected.iter_mut().enumerate() {
        let previous = if session.control == Intervention::StateDisabled {
            identity
        } else {
            session.state.roots[lane]
        };
        let code = traced_parameter(
            m,
            visits,
            EMBED + (lane << 8) + usize::from(byte),
            &mut session.work,
        );
        *target = if session.control == Intervention::TransportDisabled {
            code
        } else {
            m.product(previous, code, &mut session.work)
        };
        // Byte b uses canonical token address b+2, leaving BOS/EOS distinct.
        session.state.phases[lane] = session.state.phases[lane]
            .wrapping_add(m.artifact.geometry.tokens[usize::from(byte) + 2].phases[lane]);
        session.work.table_reads = session.work.table_reads.saturating_add(1);
        if session.control != Intervention::ZetaDisabled {
            let phase = traced_parameter(
                m,
                visits,
                PHASE + (lane << 4) + usize::from(session.state.phases[lane] >> 12),
                &mut session.work,
            );
            *target = m.product(*target, phase, &mut session.work);
        }
    }
    let query_code = traced_parameter(
        m,
        visits,
        QUERY + usize::from(injected[1]),
        &mut session.work,
    );
    let query = m.product(injected[0], query_code, &mut session.work);
    let null = traced_parameter(m, visits, NULL, &mut session.work);
    let mut score = m.relative_score(query, null, &mut session.work);
    let mut selected = None;
    if session.control != Intervention::ContextDisabled {
        let lower = session.state.seen.saturating_sub(CONTEXT as u64);
        for position in lower..session.state.seen {
            let entry = session.state.ring[(position & 31) as usize];
            session.work.candidates = session.work.candidates.saturating_add(1);
            let candidate = m.relative_score(query, entry.key, &mut session.work);
            if candidate > score {
                score = candidate;
                selected = Some(entry);
            }
        }
    }
    session.state.last_source = selected.map(|entry| entry.position);
    for lane in 0..LANES {
        let inverse = m.artifact.geometry.inverses[usize::from(injected[(lane + 1) & 3])];
        session.work.table_reads = session.work.table_reads.saturating_add(1);
        let relative = m.product(injected[lane], inverse, &mut session.work);
        let lane_base = m.artifact.geometry.row_bases[lane];
        session.work.table_reads = session.work.table_reads.saturating_add(1);
        let transition = traced_parameter(
            m,
            visits,
            TRANSITION + lane_base + usize::from(relative),
            &mut session.work,
        );
        let mut next = m.product(injected[lane], transition, &mut session.work);
        if let Some(entry) = selected {
            let read = traced_parameter(
                m,
                visits,
                READ + lane_base + usize::from(entry.values[lane]),
                &mut session.work,
            );
            next = m.product(next, read, &mut session.work);
        }
        session.state.roots[lane] = next;
    }
    let key = traced_parameter(
        m,
        visits,
        KEY + usize::from(session.state.roots[0]),
        &mut session.work,
    );
    session.state.ring[(session.state.seen & 31) as usize] = Entry {
        key,
        values: session.state.roots,
        byte,
        position: session.state.seen,
    };
    session.state.seen += 1;
    Ok(())
}

fn trace_coverage(
    model: &SharedCore,
    docs: &[Vec<u8>],
    control: Intervention,
) -> RunResult<Vec<u64>> {
    let mut visits = vec![0; PARAMETERS];
    let mut selected_reads = 0_u64;
    for doc in docs {
        let mut actual = model.session(control);
        let mut traced = model.session(control);
        for target in doc
            .iter()
            .map(|&b| u16::from(b))
            .chain(std::iter::once(EOS))
        {
            if actual.predict() != traced.predict()
                || target_nll(&mut actual, target) != target_nll(&mut traced, target)
                || actual.state != traced.state
                || actual.work != traced.work
            {
                return Err("traced/untraced prediction, state, loss or work differs".into());
            }
            if target != EOS {
                actual.observe(target as u8)?;
                selected_reads += u64::from(actual.last_source().is_some());
                traced_observe(&mut traced, target as u8, &mut visits)?;
                if actual.state != traced.state || actual.work != traced.work {
                    return Err("traced/native observe state or work differs".into());
                }
            }
        }
    }
    let bytes = docs.iter().map(|d| d.len() as u64).sum::<u64>();
    if visits[QUERY..KEY].iter().sum::<u64>() != bytes
        || visits[KEY..READ].iter().sum::<u64>() != bytes
        || (0..LANES).any(|lane| {
            visits[TRANSITION + lane * ROOTS..TRANSITION + (lane + 1) * ROOTS]
                .iter()
                .sum::<u64>()
                != bytes
        })
        || visits[READ..PHASE].iter().sum::<u64>() != selected_reads * LANES as u64
        || (control == Intervention::ContextDisabled && selected_reads != 0)
    {
        return Err("native trace family read census differs".into());
    }
    Ok(visits)
}
fn active_indices(visits: &[u64]) -> RunResult<Vec<usize>> {
    if visits.len() != PARAMETERS {
        return Err("parameter coverage length".into());
    }
    Ok((TRANSITION..PHASE).filter(|&i| visits[i] > 0).collect())
}
fn require_only_selected_changed(
    parent: &SharedCore,
    changed: &SharedCore,
    selected: Option<usize>,
) -> RunResult<()> {
    let mut actual = serde_json::to_value(&changed.artifact)?;
    let expected = serde_json::to_value(&parent.artifact)?;
    if let Some(index) = selected {
        actual["parameters"][index] = expected["parameters"][index].clone();
    }
    if actual != expected {
        return Err("witness altered another artifact field".into());
    }
    Ok(())
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Witness {
    schema: String,
    parent: String,
    implementation: String,
    driver: String,
    design: String,
    corpus_design: String,
    coverage: String,
    active_indices: Vec<usize>,
    selected_index: Option<usize>,
    selected_root: Option<u16>,
    evaluated_settings: usize,
    parent_nll: f64,
    selected_nll: f64,
}
impl Witness {
    fn replay(
        &self,
        parent: &SharedCore,
        design: &str,
        corpus: &str,
        coverage: &str,
        expected_active: &[usize],
    ) -> RunResult<SharedCore> {
        if self.schema != "uor-r4.active-sensitivity-witness/1"
            || self.parent != parent.artifact_cid()
            || self.implementation != SharedCore::implementation_digest()
            || self.driver != driver_digest()
            || self.design != design
            || self.corpus_design != corpus
            || self.coverage != coverage
            || self.active_indices != expected_active
            || self.active_indices.is_empty()
            || self.active_indices.len() > 1200
            || self
                .active_indices
                .iter()
                .any(|i| !(TRANSITION..PHASE).contains(i))
            || self.active_indices.windows(2).any(|w| w[0] >= w[1])
            || self.evaluated_settings != 1 + self.active_indices.len() * (ROOTS - 1)
            || !self.parent_nll.is_finite()
            || !self.selected_nll.is_finite()
            || self.parent_nll < 0.0
            || self.selected_nll < 0.0
            || parent.artifact.angular_tree.is_none()
        {
            return Err("unbound or invalid active sensitivity witness".into());
        }
        let mut changed = parent.clone();
        match (self.selected_index, self.selected_root) {
            (None, None) => {}
            (Some(index), Some(root))
                if self.active_indices.contains(&index)
                    && usize::from(root) < ROOTS
                    && root != parent.artifact.parameters[index] =>
            {
                changed.artifact.parameters[index] = root;
            }
            _ => return Err("invalid selected coordinate".into()),
        }
        require_only_selected_changed(parent, &changed, self.selected_index)?;
        // Inherited CID is never exported as a changed model identity.
        Ok(changed)
    }
}
fn reused_metric(
    oracle: &Value,
    index: usize,
    root: u16,
    parent_roots: [u16; 2],
) -> RunResult<(f64, usize)> {
    let position = if index == REUSE_INDICES[0] {
        usize::from(root) * ROOTS + usize::from(parent_roots[1])
    } else if index == REUSE_INDICES[1] {
        usize::from(parent_roots[0]) * ROOTS + usize::from(root)
    } else {
        return Err("not a reuse coordinate".into());
    };
    if usize::from(root) >= ROOTS || parent_roots.iter().any(|&r| usize::from(r) >= ROOTS) {
        return Err("reuse root bounds".into());
    }
    let cost = oracle["costs_row_major"][position]
        .as_f64()
        .ok_or("reuse cost")?;
    let correct = oracle["correct_counts_row_major"][position]
        .as_u64()
        .ok_or("reuse correct")? as usize;
    if !cost.is_finite() || cost < 0.0 || correct > 672 {
        return Err("reuse metric bounds".into());
    }
    Ok((cost, correct))
}
#[test]
fn active_sensitivity_trace_lineage_coverage_selection_and_reuse() -> RunResult<()> {
    let mut parent = SharedCore::initialized(745)?.upgrade_calibrated()?;
    let cap_cid = parent.artifact_cid().to_string();
    parent.artifact.schema = ANGULAR_TREE_SCHEMA.into();
    let mut branches = vec![None; 512];
    branches[0] = Some(vec![AngularNode::Leaf { score: 3 }]);
    parent.artifact.angular_tree = Some(AngularEmission {
        config: AngularTreeConfig {
            max_depth: 1,
            min_leaf: 4,
            max_seconds: 60,
        },
        parent: cap_cid,
        data_digest: format!("blake3:{}", "a".repeat(64)),
        branches,
    });
    // Deliberately heterogeneous entries prevent an incorrect traced address
    // from hiding behind the equal-root values of the saved baseline.
    for index in TRANSITION..PHASE {
        parent.artifact.parameters[index] = ((index * 37 + index / ROOTS * 11) % ROOTS) as u16;
    }
    parent.refresh_identity()?;
    let docs = vec![b"A red fox. A red fox!".to_vec(), (0..80).collect()];
    for control in [
        Intervention::Full,
        Intervention::ContextDisabled,
        Intervention::StateDisabled,
        Intervention::ZetaDisabled,
        Intervention::TransportDisabled,
    ] {
        trace_coverage(&parent, &docs, control)?;
    }
    let visits = trace_coverage(&parent, &docs, Intervention::Full)?;
    assert!(visits[READ..PHASE].iter().sum::<u64>() > 0);
    let active = active_indices(&visits)?;
    let inactive = (TRANSITION..PHASE)
        .find(|&i| visits[i] == 0)
        .ok_or("test inactive entry")?;
    let original_rows = serde_json::to_value(rows(&parent, &docs, Intervention::Full)?)?;
    let mut changed = parent.clone();
    changed.artifact.parameters[inactive] = (changed.artifact.parameters[inactive] + 1) % 120;
    assert_eq!(
        original_rows,
        serde_json::to_value(rows(&changed, &docs, Intervention::Full)?)?
    );
    let witness = Witness {
        schema: "uor-r4.active-sensitivity-witness/1".into(),
        parent: parent.artifact_cid().into(),
        implementation: SharedCore::implementation_digest(),
        driver: driver_digest(),
        design: "design".into(),
        corpus_design: "corpus".into(),
        coverage: "coverage".into(),
        active_indices: active.clone(),
        selected_index: Some(active[0]),
        selected_root: Some((parent.artifact.parameters[active[0]] + 1) % 120),
        evaluated_settings: 1 + active.len() * 119,
        parent_nll: 2.0,
        selected_nll: 1.0,
    };
    let loaded: Witness = serde_json::from_slice(&serde_json::to_vec(&witness)?)?;
    let changed = loaded.replay(&parent, "design", "corpus", "coverage", &active)?;
    assert_eq!(
        parent
            .artifact
            .parameters
            .iter()
            .zip(&changed.artifact.parameters)
            .filter(|(a, b)| a != b)
            .count(),
        1
    );
    for field in [
        "schema",
        "parent",
        "implementation",
        "driver",
        "design",
        "corpus_design",
        "coverage",
    ] {
        let mut bad = serde_json::to_value(&witness)?;
        bad[field] = json!("changed");
        assert!(serde_json::from_value::<Witness>(bad)?
            .replay(&parent, "design", "corpus", "coverage", &active)
            .is_err());
    }
    let mut bad = witness.clone();
    bad.selected_index = Some(inactive);
    assert!(bad
        .replay(&parent, "design", "corpus", "coverage", &active)
        .is_err());
    let mut bad = witness.clone();
    bad.active_indices.reverse();
    assert!(bad
        .replay(&parent, "design", "corpus", "coverage", &active)
        .is_err());
    let mut bad_model = changed;
    bad_model.artifact.angular_tree = None;
    assert!(require_only_selected_changed(&parent, &bad_model, witness.selected_index).is_err());
    assert_eq!(select(&[1.0, 1.0, 1.0], 0)?, 0);
    assert_eq!(select(&[2.0, 1.0 + 0.5 * EPS, 1.0], 0)?, 1);
    assert!(select(&[f64::NAN], 0).is_err());
    let oracle = json!({"costs_row_major":(0..14400).map(|i|i as f64).collect::<Vec<_>>(),"correct_counts_row_major":vec![1;14400]});
    assert_eq!(reused_metric(&oracle, 1176, 3, [119, 119])?, (479.0, 1));
    assert_eq!(reused_metric(&oracle, 1930, 3, [119, 119])?, (14283.0, 1));
    assert!(reused_metric(&oracle, 1500, 3, [119, 119]).is_err());
    Ok(())
}
#[test]
#[ignore = "requires sealed decoder/priors/corpus and frozen active-sensitivity design"]
fn active_sensitivity_run_saved_experiment() -> RunResult<()> {
    let input = fs::canonicalize(std::env::var("UOR_ACTIVE_INPUT")?)?;
    let evidence = input
        .parent()
        .and_then(Path::parent)
        .ok_or("evidence parent")?;
    let prior_input = fs::canonicalize(evidence.join("angular-tree-1/attempt-1"))?;
    let cap_input = fs::canonicalize(evidence.join("final-emission-1/attempt-1"))?;
    let older_input = fs::canonicalize(evidence.join("loss-frontier-1/joint-1"))?;
    let corpus = fs::canonicalize(std::env::var("UOR_ACTIVE_CORPUS")?)?;
    let path = fs::canonicalize(std::env::var("UOR_ACTIVE_DESIGN")?)?;
    let raw = std::path::PathBuf::from(std::env::var("UOR_ACTIVE_OUTPUT")?);
    let out = fs::canonicalize(raw.parent().ok_or("output parent")?)?
        .join(raw.file_name().ok_or("output name")?);
    let reuse_input = fs::canonicalize(evidence.join("tree-state-pair-1/attempt-1"))?;
    let roots = [
        &input,
        &prior_input,
        &cap_input,
        &older_input,
        &corpus,
        &reuse_input,
    ];
    for root in roots {
        if out.starts_with(root) {
            return Err("output beneath sealed input".into());
        }
    }
    crate::report_output::claim(&out)?;
    let result = (|| -> RunResult<()> {
        let start = Instant::now();
        for root in roots {
            crate::report_output::verify(root)?;
        }
        if fs::metadata(&path)?.len() > 65536 {
            return Err("design size".into());
        }
        let design_bytes = fs::read(&path)?;
        let design: Value = serde_json::from_slice(&design_bytes)?;
        let expected = json!({"schema":"uor-r4.active-sensitivity-design/1","parent":PARENT,"prior_tree":PRIOR_TREE,"cap":CAP,"older":OLDER,"corpus_sha256":"541a094ae83a56adbcabb0cd157f0a7a14aef23b6db29f82f2d794975d420adf","parameter_range":[1024,2224],"maximum_active_entries":1200,"alternatives_per_entry":119,"reuse_indices":REUSE_INDICES,"selection":"Lowest complete causal construction mean NLL; ties within 1e-10 of global minimum prefer unchanged parent, then parameter index and root. Opened and disabled-control labels excluded.","experiment_seconds":180,"attempt_bytes":67108864,"construction_documents":12,"construction_positions":672,"opened_positions":381,"generation_prompts":4,"generation_bytes":96,"witness":"Source-bound test-only parameter witness; no standard changed-model artifact or inherited candidate CID.","decoder_refits":0,"gates":{"construction_loss_strictly_improves":true,"construction_accuracy_retained":true,"correct_to_wrong_each_prior_max":0,"opened_full_loss_nonincreasing":true,"opened_full_accuracy_retained":true},"promotion":false});
        if design != expected {
            return Err("frozen design mismatch".into());
        }
        fs::File::create_new(out.join("design.json"))?.write_all(&design_bytes)?;
        let mut prior_bytes = Vec::new();
        let mut priors = Vec::new();
        for (root, cid) in [
            (&input, PARENT),
            (&prior_input, PRIOR_TREE),
            (&cap_input, CAP),
            (&older_input, OLDER),
        ] {
            let bytes = fs::read(root.join("candidate.json"))?;
            let model = SharedCore::from_bytes(&bytes)?;
            if model.artifact_cid() != cid || model.to_bytes()? != bytes {
                return Err("prior artifact identity".into());
            }
            prior_bytes.push(bytes);
            priors.push(model);
        }
        let parent = &priors[0];
        let previous_source: Value = serde_json::from_slice(&fs::read(input.join("source.json"))?)?;
        if previous_source["candidate"] != PARENT
            || previous_source["prior_tree"] != PRIOR_TREE
            || previous_source["parent"] != CAP
            || previous_source["older"] != OLDER
            || previous_source["implementation"] != SharedCore::implementation_digest()
        {
            return Err("decoder source bindings".into());
        }
        let corpus_bytes = fs::read(corpus.join("design.json"))?;
        if design["corpus_sha256"] != format!("{:x}", sha2::Sha256::digest(&corpus_bytes)) {
            return Err("corpus identity".into());
        }
        let original: Value = serde_json::from_slice(&corpus_bytes)?;
        let training = documents(&original["training"])?;
        let opened = documents(&original["holdout"])?;
        let prompts: Vec<String> = serde_json::from_value(original["prompts"].clone())?;
        if training.len() != 12
            || training.iter().map(|d| d.len() + 1).sum::<usize>() != 672
            || opened.iter().map(|d| d.len() + 1).sum::<usize>() != 381
            || prompts.len() != 4
        {
            return Err("corpus sizes".into());
        }
        let design_digest = digest(&design_bytes);
        let corpus_digest = digest(&corpus_bytes);
        let reuse_source: Value =
            serde_json::from_slice(&fs::read(reuse_input.join("source.json"))?)?;
        let reuse_design_bytes = fs::read(reuse_input.join("design.json"))?;
        let reuse_oracle_bytes = fs::read(reuse_input.join("oracle.json"))?;
        let reuse_oracle: Value = serde_json::from_slice(&reuse_oracle_bytes)?;
        let parent_roots = REUSE_INDICES.map(|i| parent.artifact.parameters[i]);
        if reuse_source["parent"] != PARENT
            || reuse_source["implementation"] != SharedCore::implementation_digest()
            || reuse_source["driver"] != digest(include_bytes!("tree_state_pair.rs"))
            || reuse_source["design"] != digest(&reuse_design_bytes)
            || reuse_source["corpus"] != corpus_digest
            || reuse_source["prior_artifact_sha256"][0]
                != format!("{:x}", sha2::Sha256::digest(&prior_bytes[0]))
            || reuse_oracle["indices"] != json!(REUSE_INDICES)
            || reuse_oracle["parent_roots"] != json!(parent_roots)
            || reuse_oracle["settings"] != 14400
            || reuse_oracle["decoder_refits"] != 0
            || reuse_oracle["all_causal_trajectories_recomputed"] != true
            || reuse_oracle["opened_and_disabled_labels_used"] != false
            || reuse_oracle["costs_row_major"]
                .as_array()
                .ok_or("reuse costs")?
                .len()
                != 14400
            || reuse_oracle["correct_counts_row_major"]
                .as_array()
                .ok_or("reuse correct counts")?
                .len()
                != 14400
        {
            return Err("sealed prior single-coordinate reuse bindings".into());
        }
        let visits = trace_coverage(parent, &training, Intervention::Full)?;
        let active = active_indices(&visits)?;
        if active.is_empty() || active.len() > 1200 {
            return Err("active domain bounds".into());
        }
        let baseline = parent.evaluate(&training, Intervention::Full)?;
        let reused_baseline = reused_metric(
            &reuse_oracle,
            REUSE_INDICES[0],
            parent_roots[0],
            parent_roots,
        )?;
        if (baseline.mean_nll - reused_baseline.0).abs() > EPS
            || baseline.correct != reused_baseline.1
            || baseline.positions != 672
        {
            return Err("reuse baseline differs".into());
        }
        let reuse_count = active.iter().filter(|i| REUSE_INDICES.contains(i)).count() * 119;
        let evaluated_settings = 1 + active.len() * 119;
        let coverage = json!({"schema":"uor-r4.active-sensitivity-coverage/1","parent":PARENT,"implementation":SharedCore::implementation_digest(),"driver":driver_digest(),"design":design_digest,"corpus":corpus_digest,"range":[TRANSITION,PHASE],"visits_all_parameters":visits,"active_indices":active,"active_entries":active.len(),"inactive_entries":1200-active.len(),"evaluated_settings":evaluated_settings,"reused_settings":reuse_count,"fresh_proposal_evaluate_calls":evaluated_settings-1-reuse_count,"baseline_evaluate_calls":1,"complete_causal_documents_per_evaluate":12,"positions_per_evaluate":672,"trace_native_stepwise_equality":true,"trace_predict_loss_state_work_equality":true,"coverage_is_causal_credit":false,"scope":"Only a single entry intervention can be pruned when baseline never reads it; simultaneous changes are not covered.","prior_table":digest(&reuse_oracle_bytes)});
        let coverage_bytes = serde_json::to_vec_pretty(&coverage)?;
        let coverage_digest = digest(&coverage_bytes);
        // Exclusive write and sync before any alternative evaluation. The final
        // seal binds this immutable forecast and the complete profile together.
        let mut coverage_file = fs::File::create_new(out.join("coverage.json"))?;
        coverage_file.write_all(&coverage_bytes)?;
        coverage_file.sync_all()?;
        guard(start)?;
        let mut scoring = parent.clone();
        let mut costs = vec![baseline.mean_nll];
        let mut correct_counts = vec![baseline.correct];
        let mut settings: Vec<Option<(usize, u16)>> = vec![None];
        let mut profile = vec![
            json!({"setting":0,"index":null,"root":null,"nll":baseline.mean_nll,"correct":baseline.correct,"source":"actual_baseline"}),
        ];
        let mut fresh_calls = 0;
        for &index in &active {
            let original = parent.artifact.parameters[index];
            for root in 0..ROOTS as u16 {
                if root == original {
                    continue;
                }
                guard(start)?;
                let (nll, correct, source) = if REUSE_INDICES.contains(&index) {
                    let (nll, correct) = reused_metric(&reuse_oracle, index, root, parent_roots)?;
                    (nll, correct, "sealed_tree_state_pair_single_coordinate")
                } else {
                    scoring.artifact.parameters[index] = root;
                    let metric = scoring.evaluate(&training, Intervention::Full)?;
                    fresh_calls += 1;
                    if metric.positions != 672
                        || !metric.mean_nll.is_finite()
                        || metric.mean_nll < 0.0
                    {
                        return Err("sensitivity metric bounds".into());
                    }
                    (
                        metric.mean_nll,
                        metric.correct,
                        "actual_full_causal_evaluate",
                    )
                };
                profile.push(json!({"setting":costs.len(),"index":index,"root":root,"nll":nll,"correct":correct,"source":source}));
                costs.push(nll);
                correct_counts.push(correct);
                settings.push(Some((index, root)));
            }
            scoring.artifact.parameters[index] = original;
        }
        if costs.len() != evaluated_settings || fresh_calls != evaluated_settings - 1 - reuse_count
        {
            return Err("frozen coverage/call census differs".into());
        }
        require_only_selected_changed(parent, &scoring, None)?;
        let parent_index = 0;
        let selected = select(&costs, parent_index)?;
        let selected_index = settings[selected].map(|v| v.0);
        let selected_root = settings[selected].map(|v| v.1);
        let minimum = costs.iter().copied().fold(f64::INFINITY, f64::min);
        let improving = costs.iter().filter(|&&v| v + EPS < costs[0]).count();
        let witness = Witness {
            schema: "uor-r4.active-sensitivity-witness/1".into(),
            parent: PARENT.into(),
            implementation: SharedCore::implementation_digest(),
            driver: driver_digest(),
            design: design_digest.clone(),
            corpus_design: corpus_digest.clone(),
            coverage: coverage_digest.clone(),
            active_indices: active.clone(),
            selected_index,
            selected_root,
            evaluated_settings,
            parent_nll: costs[0],
            selected_nll: costs[selected],
        };
        let witness_bytes = serde_json::to_vec_pretty(&witness)?;
        let witness_id = digest(&witness_bytes);
        fs::File::create_new(out.join("witness.json"))?.write_all(&witness_bytes)?;
        write(
            &out,
            "profile.json",
            &json!({"coverage":coverage_digest,"settings":evaluated_settings,"selected_setting":selected,"selected_index":selected_index,"selected_root":selected_root,"parent_nll":costs[0],"selected_nll":costs[selected],"minimum_nll":minimum,"settings_within_minimum_tolerance":costs.iter().filter(|&&v|v<=minimum+EPS).count(),"strictly_improving_settings":improving,"fresh_proposal_evaluate_calls":fresh_calls,"reused_settings":reuse_count,"prior_table":digest(&reuse_oracle_bytes),"entries":profile,"decoder_refits":0,"opened_and_disabled_labels_used":false}),
        )?;
        let replayed: Witness = serde_json::from_slice(&witness_bytes)?;
        let candidate = replayed.replay(
            parent,
            &design_digest,
            &corpus_digest,
            &coverage_digest,
            &active,
        )?;
        let early = generate(&candidate, &prompts, &[Intervention::Full])?;
        write(
            &out,
            "early-generation.json",
            &json!({"witness":witness_id,"outputs":early}),
        )?;
        guard(start)?;
        // Selection is fixed and the witness is exported before any opened or
        // disabled-control labels are inspected.
        let previous: Value = serde_json::from_slice(&fs::read(input.join("behavior.json"))?)?;
        let mut behavior = Vec::new();
        let mut lost = [0_u64; 4];
        let mut gained = [0_u64; 4];
        let mut root_hamming = [0_u64; 4];
        let mut source_hamming = [0_u64; 4];
        let mut prediction_hamming = [0_u64; 4];
        let mut metrics = Vec::new();
        for (name, docs) in [
            ("complete_construction", &training),
            ("opened_development", &opened),
        ] {
            for control in CONTROLS {
                guard(start)?;
                let changed = rows(&candidate, docs, control)?;
                let after = verified_metrics(&candidate, docs, control, &changed)?;
                let saved_panel = previous["panels"]
                    .as_array()
                    .ok_or("prior panels")?
                    .iter()
                    .find(|v| {
                        v["panel"] == name
                            && v["control"] == serde_json::to_value(control).unwrap_or(Value::Null)
                    })
                    .ok_or("prior panel")?;
                let mut comparisons = Vec::new();
                for (index, model) in priors.iter().enumerate() {
                    let base = rows(model, docs, control)?;
                    let before = verified_metrics(model, docs, control, &base)?;
                    let saved = if index == 0 {
                        json!({"metrics":saved_panel["candidate_metrics"],"rows":saved_panel["candidate_rows"]})
                    } else {
                        saved_panel["comparisons"]
                            .as_array()
                            .ok_or("prior comparisons")?
                            .iter()
                            .find(|v| v["parent"] == model.artifact_cid())
                            .ok_or("prior comparison")?
                            .clone()
                    };
                    saved_rows_equal(&saved, &base, &before)?;
                    let comparison = compare(&base, &changed)?;
                    lost[index] += comparison["correct_to_wrong"]
                        .as_u64()
                        .ok_or("lost count")?;
                    gained[index] += comparison["wrong_to_correct"]
                        .as_u64()
                        .ok_or("gained count")?;
                    root_hamming[index] += comparison["root_identity_hamming"]
                        .as_u64()
                        .ok_or("root count")?;
                    source_hamming[index] += comparison["source_identity_hamming"]
                        .as_u64()
                        .ok_or("source count")?;
                    prediction_hamming[index] += comparison["prediction_hamming"]
                        .as_u64()
                        .ok_or("prediction count")?;
                    if control == Intervention::Full {
                        metrics.push(json!({"panel":name,"parent":model.artifact_cid(),"before":before,"after":after}));
                    }
                    comparisons.push(json!({"parent":model.artifact_cid(),"metrics":before,"rows":base,"comparison":comparison}));
                }
                behavior.push(json!({"panel":name,"control":control,"witness":witness_id,"witness_metrics":after,"witness_rows":changed,"comparisons":comparisons}));
            }
        }
        let mut generations = Vec::new();
        for model in &priors {
            let outputs = generate(model, &prompts, &CONTROLS)?;
            let saved = previous["generations"]
                .as_array()
                .ok_or("saved generations")?
                .iter()
                .find(|v| v["artifact"] == model.artifact_cid())
                .ok_or("saved artifact generation")?;
            if serde_json::to_value(&outputs)? != saved["outputs"] {
                return Err("prior generation changed".into());
            }
            generations.push(json!({"artifact":model.artifact_cid(),"outputs":outputs}));
        }
        let outputs = generate(&candidate, &prompts, &CONTROLS)?;
        for (i, first) in early.iter().enumerate() {
            if first != &outputs[i * 3] {
                return Err("early generation replay".into());
            }
        }
        generations.push(json!({"witness":witness_id,"outputs":outputs}));
        let construction = metrics
            .iter()
            .find(|m| m["panel"] == "complete_construction" && m["parent"] == PARENT)
            .ok_or("construction metrics")?;
        let before: Metrics = serde_json::from_value(construction["before"].clone())?;
        let after: Metrics = serde_json::from_value(construction["after"].clone())?;
        if (before.mean_nll - replayed.parent_nll).abs() > EPS
            || (after.mean_nll - replayed.selected_nll).abs() > EPS
            || before.correct != correct_counts[parent_index]
            || after.correct != correct_counts[selected]
        {
            return Err("oracle/witness metric replay".into());
        }
        let optimization =
            after.mean_nll + EPS < before.mean_nll && after.correct >= before.correct;
        let preservation = lost == [0, 0, 0, 0];
        let mut development = true;
        for item in metrics
            .iter()
            .filter(|m| m["panel"] == "opened_development")
        {
            let base: Metrics = serde_json::from_value(item["before"].clone())?;
            let changed: Metrics = serde_json::from_value(item["after"].clone())?;
            development &=
                changed.mean_nll <= base.mean_nll + EPS && changed.correct >= base.correct;
        }
        guard(start)?;
        let mut session = candidate.session(Intervention::Full);
        let mut max_comparisons = 0;
        let mut census_predictions = Vec::with_capacity(512);
        for step in 0..512 {
            guard(start)?;
            let before = session.work();
            let prediction = session.predict();
            let comparisons = session.work().products - before.products;
            if comparisons > 27 {
                return Err("witness prediction angular comparison bound".into());
            }
            max_comparisons = max_comparisons.max(comparisons);
            census_predictions.push(prediction);
            session.observe((step & 255) as u8)?;
        }
        write(
            &out,
            "runtime-census.json",
            &json!({"witness":witness_id,"steps":512,"max_angular_comparisons_per_predict":max_comparisons,"bound":27,"work":session.work(),"predictions":census_predictions,"allocation_measurement":"NOT_RUN_IN_THIS_DRIVER"}),
        )?;
        write(
            &out,
            "behavior.json",
            &json!({"panels":behavior,"generations":generations}),
        )?;
        write(
            &out,
            "source.json",
            &json!({"implementation":SharedCore::implementation_digest(),"driver":driver_digest(),"tree_source":digest(include_bytes!("../angular_tree.rs")),"coverage":coverage_digest,"reused_table":digest(&reuse_oracle_bytes),"reused_table_root":reuse_input,"design":design_digest,"corpus":corpus_digest,"parent":PARENT,"prior_tree":PRIOR_TREE,"cap":CAP,"older":OLDER,"witness":witness_id,"prior_artifact_sha256":prior_bytes.iter().map(|b|format!("{:x}",sha2::Sha256::digest(b))).collect::<Vec<_>>(),"prior_report_roots":[input,prior_input,cap_input,older_input],"standard_changed_model_artifact_exported":false}),
        )?;
        for (model, bytes) in priors.iter().zip(&prior_bytes) {
            if model.to_bytes()? != *bytes {
                return Err("prior mutated".into());
            }
        }
        require_only_selected_changed(parent, &candidate, selected_index)?;
        if fs::read(out.join("coverage.json"))? != coverage_bytes {
            return Err("frozen coverage changed".into());
        };
        for root in roots {
            crate::report_output::verify(root)?;
        }
        guard(start)?;
        write(
            &out,
            "result.json",
            &json!({"schema":"uor-r4.active-sensitivity-result/1","decision":if optimization&&preservation&&development{"PASS_ACTIVE_SENSITIVITY_DEVELOPMENT_GATE"}else{"FAIL_ACTIVE_SENSITIVITY_DEVELOPMENT_GATE"},"witness":witness_id,"parent":PARENT,"prior_tree":PRIOR_TREE,"cap":CAP,"older":OLDER,"selected_index":selected_index,"selected_root":selected_root,"active_entries":active.len(),"evaluated_settings":evaluated_settings,"fresh_proposal_evaluate_calls":fresh_calls,"reused_settings":reuse_count,"strictly_improving_settings":improving,"minimum_nll":minimum,"parent_is_selected":selected==parent_index,"conditional_optimization_gate":optimization,"preservation_gate":preservation,"opened_development_gate":development,"correct_to_wrong_each_prior":lost,"wrong_to_correct_each_prior":gained,"root_identity_hamming_each_prior":root_hamming,"source_identity_hamming_each_prior":source_hamming,"prediction_hamming_each_prior":prediction_hamming,"metrics":metrics,"before":before,"after":after,"elapsed_ms":start.elapsed().as_millis(),"only_selected_parameter_indices_mutable":true,"changed_parameter_count":parent.artifact.parameters.iter().zip(&candidate.artifact.parameters).filter(|(a,b)|a!=b).count(),"decoder_unchanged":true,"decoder_refits":0,"promoted":false,"scope":"Complete baseline-active single-coordinate construction sensitivity scan under fixed angular decoder, with separately retained prior controls and actual generation. Test-only source-bound witness; no standard changed artifact, fresh holdout, decoder refit, promotion, language or energy qualification."}),
        )?;
        guard(start)?;
        let mut total = 0_u64;
        for entry in fs::read_dir(&out)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                return Err("non-file report entry".into());
            }
            total = total
                .checked_add(entry.metadata()?.len())
                .ok_or("report size overflow")?;
        }
        if total > 64 * 1024 * 1024 - 128 * 1024 {
            return Err("report storage limit including manifest reserve".into());
        }
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
    let total = fs::read_dir(&out)?.try_fold(0_u64, |sum, e| -> std::io::Result<u64> {
        Ok(sum + e?.metadata()?.len())
    })?;
    if total > 64 * 1024 * 1024 {
        return Err("sealed attempt storage limit".into());
    }
    result
}
