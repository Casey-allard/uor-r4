//! Matched varied/repeated context state learning with a fixed decoder.
//! Test-only evaluation and tracing helpers mirror active_sensitivity.rs unchanged.
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
const SEED: u64 = 20260913;
const PROPOSALS: usize = 8192;
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
        return Err("complete matched context experiment time guard".into());
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
    digest(include_bytes!("context_augmentation.rs"))
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

fn proposal_stream(seed: u64, count: usize) -> RunResult<Vec<(usize, u16)>> {
    if seed == 0 || count == 0 || count > PROPOSALS {
        return Err("proposal bounds".into());
    }
    let mut state = seed;
    let mut draw = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    Ok((0..count)
        .map(|_| {
            (
                TRANSITION + (draw() % (PHASE - TRANSITION) as u64) as usize,
                (draw() % ROOTS as u64) as u16,
            )
        })
        .collect())
}
fn strict_improvement(before: f64, proposed: f64) -> RunResult<bool> {
    if !before.is_finite() || !proposed.is_finite() || before < 0.0 || proposed < 0.0 {
        return Err("nonfinite or negative objective".into());
    }
    Ok(proposed + EPS < before)
}
fn require_only_overrides(
    parent: &SharedCore,
    changed: &SharedCore,
    overrides: &[(usize, u16)],
) -> RunResult<()> {
    if overrides.windows(2).any(|w| w[0].0 >= w[1].0)
        || overrides.iter().any(|&(i, r)| {
            !(TRANSITION..PHASE).contains(&i)
                || usize::from(r) >= ROOTS
                || parent.artifact.parameters[i] == r
                || changed.artifact.parameters[i] != r
        })
    {
        return Err("invalid parameter overrides".into());
    }
    let mut value = serde_json::to_value(&changed.artifact)?;
    let expected = serde_json::to_value(&parent.artifact)?;
    for &(index, _) in overrides {
        value["parameters"][index] = expected["parameters"][index].clone();
    }
    if value != expected {
        return Err("state witness changed frozen artifact fields".into());
    }
    Ok(())
}
fn override_vector(parent: &SharedCore, changed: &SharedCore) -> Vec<(usize, u16)> {
    (TRANSITION..PHASE)
        .filter_map(|i| {
            (parent.artifact.parameters[i] != changed.artifact.parameters[i])
                .then_some((i, changed.artifact.parameters[i]))
        })
        .collect()
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
    dataset: String,
    arm_data: String,
    arm: String,
    seed: u64,
    proposal_stream: String,
    proposals: usize,
    overrides: Vec<(usize, u16)>,
    before: Metrics,
    after: Metrics,
}
impl Witness {
    fn replay(
        &self,
        parent: &SharedCore,
        design: &str,
        corpus: &str,
        dataset: &str,
        arm_data: &str,
        arm: &str,
        stream: &str,
    ) -> RunResult<SharedCore> {
        if self.schema != "uor-r4.context-augmentation-witness/1"
            || self.parent != parent.artifact_cid()
            || self.implementation != SharedCore::implementation_digest()
            || self.driver != driver_digest()
            || self.design != design
            || self.corpus_design != corpus
            || self.dataset != dataset
            || self.arm_data != arm_data
            || self.arm != arm
            || !["repeated", "varied"].contains(&arm)
            || self.seed != SEED
            || self.proposal_stream != stream
            || self.proposals != PROPOSALS
            || self.before.positions != 2688
            || self.after.positions != 2688
            || self.before.correct > 2688
            || self.after.correct > 2688
            || !self.before.mean_nll.is_finite()
            || !self.after.mean_nll.is_finite()
            || self.after.mean_nll < 0.0
            || self.after.mean_nll > self.before.mean_nll + EPS
            || parent.artifact.angular_tree.is_none()
        {
            return Err("unbound context witness".into());
        }
        let mut model = parent.clone();
        for &(i, r) in &self.overrides {
            if !(TRANSITION..PHASE).contains(&i) || usize::from(r) >= ROOTS {
                return Err("override bounds".into());
            }
            model.artifact.parameters[i] = r;
        }
        require_only_overrides(parent, &model, &self.overrides)?;
        // The inherited artifact CID is never exported as this changed state.
        Ok(model)
    }
}
fn document_support(
    model: &SharedCore,
    docs: &[Vec<u8>],
    index: usize,
    start: Instant,
) -> RunResult<Value> {
    let mut document_indices = Vec::new();
    let mut unique = std::collections::BTreeSet::new();
    let mut visits = 0_u64;
    for (i, doc) in docs.iter().enumerate() {
        guard(start)?;
        let counts = trace_coverage(model, std::slice::from_ref(doc), Intervention::Full)?;
        visits += counts[index];
        if counts[index] > 0 {
            document_indices.push(i);
            unique.insert(doc.clone());
        }
    }
    Ok(
        json!({"index":index,"visits":visits,"document_indices":document_indices,"documents":document_indices.len(),"unique_document_contents":unique.len(),"support_before_update":true,"trace_native_equality":true,"selection_input":false,"causal_credit_claim":false}),
    )
}
fn fit_arm(
    parent: &SharedCore,
    docs: &[Vec<u8>],
    proposals: &[(usize, u16)],
    start: Instant,
) -> RunResult<(SharedCore, Value, Metrics, Metrics)> {
    if docs.len() != 48
        || docs.iter().map(|d| d.len() + 1).sum::<usize>() != 2688
        || proposals.len() != PROPOSALS
    {
        return Err("matched fit dimensions".into());
    }
    let mut model = parent.clone();
    let before = model.evaluate(docs, Intervention::Full)?;
    let mut current = before.clone();
    let mut records = Vec::new();
    let mut accepted = Vec::new();
    let mut no_ops = 0;
    for (ordinal, &(index, root)) in proposals.iter().enumerate() {
        guard(start)?;
        if !(TRANSITION..PHASE).contains(&index) || usize::from(root) >= ROOTS {
            return Err("proposal outside domain".into());
        }
        let old = model.artifact.parameters[index];
        let no_op = old == root;
        no_ops += usize::from(no_op);
        model.artifact.parameters[index] = root;
        // Every proposal, including a no-op, recomputes every complete sequence.
        let proposed = model.evaluate(docs, Intervention::Full)?;
        if proposed.positions != 2688 {
            return Err("proposal exposure".into());
        }
        let take = strict_improvement(current.mean_nll, proposed.mean_nll)?;
        if no_op
            && (take
                || proposed.correct != current.correct
                || proposed.mean_nll != current.mean_nll)
        {
            return Err("no-op changed objective".into());
        }
        records.push(json!({"proposal":ordinal,"index":index,"old_root":old,"root":root,"no_op":no_op,"accepted":take,"before_nll":current.mean_nll,"proposed_nll":proposed.mean_nll,"before_correct":current.correct,"proposed_correct":proposed.correct,"positions":proposed.positions}));
        if take {
            model.artifact.parameters[index] = old;
            let support = document_support(&model, docs, index, start)?;
            model.artifact.parameters[index] = root;
            accepted.push(json!({"proposal":ordinal,"index":index,"old_root":old,"root":root,"before_nll":current.mean_nll,"after_nll":proposed.mean_nll,"before_correct":current.correct,"after_correct":proposed.correct,"support":support}));
            current = proposed;
        } else {
            model.artifact.parameters[index] = old;
        }
    }
    require_only_overrides(parent, &model, &override_vector(parent, &model))?;
    Ok((
        model,
        json!({"proposals":proposals.len(),"proposal_evaluate_calls":proposals.len(),"baseline_evaluate_calls":1,"no_op_proposals_fully_evaluated":no_ops,"proposal_positions":proposals.len()*2688,"accepted_updates":accepted.len(),"accepted":accepted,"entries":records,"complete_causal_sequence_recomputation":true,"support_trace_documents":accepted.len()*48,"support_used_for_selection":false,"decoder_refits":0}),
        before,
        current,
    ))
}
fn read_json(path: &Path) -> RunResult<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn saved_panel<'a>(behavior: &'a Value, name: &str, control: Intervention) -> RunResult<&'a Value> {
    let c = serde_json::to_value(control)?;
    behavior["panels"]
        .as_array()
        .ok_or("saved panels")?
        .iter()
        .find(|p| p["panel"] == name && p["control"] == c)
        .ok_or_else(|| "saved panel".into())
}
fn load_sensitivity(
    parent: &SharedCore,
    root: &Path,
    corpus: &str,
) -> RunResult<(SharedCore, String)> {
    let bytes = fs::read(root.join("witness.json"))?;
    let w: Value = serde_json::from_slice(&bytes)?;
    let source = read_json(&root.join("source.json"))?;
    let result = read_json(&root.join("result.json"))?;
    let id = digest(&bytes);
    if w["schema"] != "uor-r4.active-sensitivity-witness/1"
        || w["parent"] != PARENT
        || w["implementation"] != SharedCore::implementation_digest()
        || w["driver"] != digest(include_bytes!("active_sensitivity.rs"))
        || w["design"] != digest(&fs::read(root.join("design.json"))?)
        || w["coverage"] != digest(&fs::read(root.join("coverage.json"))?)
        || w["corpus_design"] != corpus
        || w["selected_index"] != 1937
        || w["selected_root"] != 107
        || w["evaluated_settings"] != 136851
        || source["witness"] != id
        || source["driver"] != w["driver"]
        || source["parent"] != PARENT
        || result["witness"] != id
    {
        return Err("prior sensitivity witness bindings".into());
    }
    let mut model = parent.clone();
    model.artifact.parameters[1937] = 107;
    require_only_overrides(parent, &model, &[(1937, 107)])?;
    Ok((model, id))
}
#[test]
fn context_augmentation_matching_noops_ties_support_and_lineage() -> RunResult<()> {
    let a = proposal_stream(SEED, PROPOSALS)?;
    let b = proposal_stream(SEED, PROPOSALS)?;
    assert_eq!(a, b);
    assert_ne!(a, proposal_stream(SEED + 1, PROPOSALS)?);
    assert!(a
        .iter()
        .all(|&(i, r)| (TRANSITION..PHASE).contains(&i) && usize::from(r) < ROOTS));
    assert!(!strict_improvement(1.0, 1.0)?);
    assert!(!strict_improvement(1.0, 1.0 - EPS / 2.0)?);
    assert!(strict_improvement(1.0, 0.99)?);
    assert!(strict_improvement(f64::NAN, 1.0).is_err());
    let mut parent = SharedCore::initialized(745)?.upgrade_calibrated()?;
    let original = parent.artifact_cid().to_string();
    parent.artifact.schema = ANGULAR_TREE_SCHEMA.into();
    let mut branches = vec![None; 512];
    branches[0] = Some(vec![AngularNode::Leaf { score: 3 }]);
    parent.artifact.angular_tree = Some(AngularEmission {
        config: AngularTreeConfig {
            max_depth: 1,
            min_leaf: 4,
            max_seconds: 60,
        },
        parent: original,
        data_digest: format!("blake3:{}", "a".repeat(64)),
        branches,
    });
    parent.refresh_identity()?;
    let doc = b"A red fox.".to_vec();
    let repeated = vec![doc.clone(), doc];
    let coverage = trace_coverage(&parent, &repeated, Intervention::Full)?;
    let index = (TRANSITION..PHASE)
        .find(|&i| coverage[i] > 0)
        .ok_or("active test index")?;
    let support = document_support(&parent, &repeated, index, Instant::now())?;
    assert_eq!(support["documents"], 2);
    assert_eq!(support["unique_document_contents"], 1);
    let before = parent.evaluate(&repeated, Intervention::Full)?;
    let root = parent.artifact.parameters[index];
    let mut model = parent.clone();
    model.artifact.parameters[index] = root;
    let after = model.evaluate(&repeated, Intervention::Full)?;
    assert_eq!(before.mean_nll, after.mean_nll);
    assert_eq!(before.positions, after.positions);
    assert!(!strict_improvement(before.mean_nll, after.mean_nll)?);
    model.artifact.parameters[index] = (root + 1) % 120;
    require_only_overrides(&parent, &model, &[(index, (root + 1) % 120)])?;
    assert!(require_only_overrides(&parent, &model, &[]).is_err());
    model.artifact.parameters[EMBED] = (model.artifact.parameters[EMBED] + 1) % 120;
    assert!(require_only_overrides(&parent, &model, &[(index, (root + 1) % 120)]).is_err());
    let metrics = Metrics {
        positions: 2688,
        correct: 0,
        mean_nll: 1.0,
        work: Work::default(),
    };
    let w = Witness {
        schema: "uor-r4.context-augmentation-witness/1".into(),
        parent: parent.artifact_cid().into(),
        implementation: SharedCore::implementation_digest(),
        driver: driver_digest(),
        design: "d".into(),
        corpus_design: "c".into(),
        dataset: "ds".into(),
        arm_data: "a".into(),
        arm: "repeated".into(),
        seed: SEED,
        proposal_stream: "s".into(),
        proposals: PROPOSALS,
        overrides: vec![],
        before: metrics.clone(),
        after: metrics,
    };
    w.replay(&parent, "d", "c", "ds", "a", "repeated", "s")?;
    assert!(w
        .replay(&parent, "d", "c", "ds", "a", "varied", "s")
        .is_err());
    let mut bad = w.clone();
    bad.overrides = vec![(0, 1)];
    assert!(bad
        .replay(&parent, "d", "c", "ds", "a", "repeated", "s")
        .is_err());
    Ok(())
}

#[test]
#[ignore = "requires sealed matched dataset, decoder/priors/corpus and frozen context design"]
fn context_augmentation_run_saved_experiment() -> RunResult<()> {
    let input = fs::canonicalize(std::env::var("UOR_CONTEXT_INPUT")?)?;
    let evidence = input
        .parent()
        .and_then(Path::parent)
        .ok_or("evidence parent")?;
    let prior_input = fs::canonicalize(evidence.join("angular-tree-1/attempt-1"))?;
    let cap_input = fs::canonicalize(evidence.join("final-emission-1/attempt-1"))?;
    let older_input = fs::canonicalize(evidence.join("loss-frontier-1/joint-1"))?;
    let sensitivity_input = fs::canonicalize(evidence.join("active-sensitivity-1/attempt-1"))?;
    let corpus = fs::canonicalize(std::env::var("UOR_CONTEXT_CORPUS")?)?;
    let data_root = fs::canonicalize(std::env::var("UOR_CONTEXT_DATA")?)?;
    let path = fs::canonicalize(std::env::var("UOR_CONTEXT_DESIGN")?)?;
    let raw = std::path::PathBuf::from(std::env::var("UOR_CONTEXT_OUTPUT")?);
    let out = fs::canonicalize(raw.parent().ok_or("output parent")?)?
        .join(raw.file_name().ok_or("output name")?);
    let roots = [
        &input,
        &prior_input,
        &cap_input,
        &older_input,
        &sensitivity_input,
        &corpus,
        &data_root,
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
        let expected = json!({"schema":"uor-r4.context-augmentation-design/1","parent":PARENT,"prior_tree":PRIOR_TREE,"cap":CAP,"older":OLDER,"corpus_sha256":"541a094ae83a56adbcabb0cd157f0a7a14aef23b6db29f82f2d794975d420adf","parameter_range":[1024,2224],"seed":SEED,"proposals_per_arm":PROPOSALS,"proposal_rng":"xorshift64: x ^= x<<13; x ^= x>>7; x ^= x<<17; two draws per proposal index=1024+draw%1200,root=draw%120","arms":["repeated","varied"],"documents_per_arm":48,"positions_per_arm":2688,"training_selection":"Accept strict complete causal mean NLL improvement greater than 1e-10; unchanged on ties; every proposal including no-op evaluates all positions.","experiment_seconds":180,"attempt_bytes":67108864,"decoder_refits":0,"construction_positions":672,"opened_positions":381,"generation_prompts":4,"generation_bytes":96,"promotion":false,"gates":{"method_probe_full_loss_strict_below_parent_and_repeated":true,"method_probe_full_accuracy_retained":true,"correct_to_wrong_each_prior_max":0,"opened_full_loss_nonincreasing":true,"opened_full_accuracy_retained":true,"construction_loss_nonincreasing":true,"construction_accuracy_retained":true,"generation_termination_required":true}});
        for (key, value) in expected.as_object().ok_or("design object")? {
            if design[key] != *value {
                return Err(format!("frozen design field differs: {key}").into());
            }
        }
        fs::File::create_new(out.join("design.json"))?.write_all(&design_bytes)?;
        let corpus_bytes = fs::read(corpus.join("design.json"))?;
        if design["corpus_sha256"] != format!("{:x}", sha2::Sha256::digest(&corpus_bytes)) {
            return Err("corpus identity".into());
        }
        let original: Value = serde_json::from_slice(&corpus_bytes)?;
        let construction = documents(&original["training"])?;
        let opened = documents(&original["holdout"])?;
        let prompts: Vec<String> = serde_json::from_value(original["prompts"].clone())?;
        let dataset_bytes = fs::read(data_root.join("dataset.json"))?;
        if design["dataset_sha256"] != format!("{:x}", sha2::Sha256::digest(&dataset_bytes)) {
            return Err("frozen dataset SHA mismatch".into());
        }
        let dataset: Value = serde_json::from_slice(&dataset_bytes)?;
        let data_source = read_json(&data_root.join("source.json"))?;
        if data_source["schema"] != "uor-r4.matched-context-data-source/1"
            || data_source["corpus_sha256"] != design["corpus_sha256"]
            || data_source["preparation_source_blake3"] != digest(include_bytes!("context_data.rs"))
            || data_source["dataset_blake3"] != digest(&dataset_bytes)
            || data_source["model_calls"] != 0
        {
            return Err("dataset source bindings".into());
        }
        let repeated = documents(&dataset["repeated"])?;
        let varied = documents(&dataset["varied"])?;
        // Probe bytes are bound before learning; no probe target is evaluated or
        // consulted until both final witnesses have been written and reloaded.
        if dataset["schema"] != "uor-r4.matched-context-data/1"
            || dataset["prompts"].as_array().is_none_or(|p| p.len() != 4)
            || construction.len() != 12
            || construction.iter().map(|d| d.len() + 1).sum::<usize>() != 672
            || opened.iter().map(|d| d.len() + 1).sum::<usize>() != 381
            || prompts.len() != 4
            || [repeated.as_slice(), varied.as_slice()].iter().any(|docs| {
                docs.len() != 48 || docs.iter().map(|d| d.len() + 1).sum::<usize>() != 2688
            })
        {
            return Err("matched dataset dimensions".into());
        }
        if repeated
            != construction
                .iter()
                .flat_map(|d| std::iter::repeat_n(d.clone(), 4))
                .collect::<Vec<_>>()
        {
            return Err("repeat arm differs from four original passes".into());
        }
        let unique_repeated = repeated
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        let unique_varied = varied
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        if unique_repeated != 12
            || unique_varied != 48
            || varied
                .iter()
                .zip(&repeated)
                .any(|(a, b)| a.len() != b.len())
        {
            return Err("context diversity does not differ".into());
        }
        let mut priors = Vec::new();
        let mut prior_bytes = Vec::new();
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
        let previous_source = read_json(&input.join("source.json"))?;
        if previous_source["candidate"] != PARENT
            || previous_source["prior_tree"] != PRIOR_TREE
            || previous_source["parent"] != CAP
            || previous_source["older"] != OLDER
            || previous_source["implementation"] != SharedCore::implementation_digest()
        {
            return Err("decoder source bindings".into());
        }
        let design_digest = digest(&design_bytes);
        let corpus_digest = digest(&corpus_bytes);
        let dataset_digest = digest(&dataset_bytes);
        let (old_witness, old_witness_id) =
            load_sensitivity(parent, &sensitivity_input, &corpus_digest)?;
        let proposals = proposal_stream(SEED, PROPOSALS)?;
        let stream_bytes = serde_json::to_vec_pretty(&proposals)?;
        let stream_digest = digest(&stream_bytes);
        write(&out, "proposal-stream.json", &proposals)?;
        write(
            &out,
            "forecast.json",
            &json!({"design":design_digest,"dataset":dataset_digest,"proposal_stream":stream_digest,"seed":SEED,"fit_calls":2,"proposal_evaluate_calls_per_arm":PROPOSALS,"positions_per_evaluate":2688,"total_proposal_positions":2*PROPOSALS*2688,"noops_evaluate_all_positions":true,"support_selection_penalty":false,"unique_repeated_documents":unique_repeated,"unique_varied_documents":unique_varied,"probe_evaluations_before_final_witnesses":0}),
        )?;
        let mut models = Vec::new();
        let mut witnesses = Vec::new();
        let mut fit_metrics = Vec::new();
        let mut early_outputs = Vec::new();
        for (arm, docs) in [("repeated", &repeated), ("varied", &varied)] {
            guard(start)?;
            let (fitted, profile, before, after) = fit_arm(parent, docs, &proposals, start)?;
            let arm_digest = digest(&serde_json::to_vec(docs)?);
            let witness = Witness {
                schema: "uor-r4.context-augmentation-witness/1".into(),
                parent: PARENT.into(),
                implementation: SharedCore::implementation_digest(),
                driver: driver_digest(),
                design: design_digest.clone(),
                corpus_design: corpus_digest.clone(),
                dataset: dataset_digest.clone(),
                arm_data: arm_digest.clone(),
                arm: arm.into(),
                seed: SEED,
                proposal_stream: stream_digest.clone(),
                proposals: PROPOSALS,
                overrides: override_vector(parent, &fitted),
                before: before.clone(),
                after: after.clone(),
            };
            write(&out, &format!("{arm}-profile.json"), &profile)?;
            write(&out, &format!("{arm}-witness.json"), &witness)?;
            let bytes = fs::read(out.join(format!("{arm}-witness.json")))?;
            let replayed: Witness = serde_json::from_slice(&bytes)?;
            let model = replayed.replay(
                parent,
                &design_digest,
                &corpus_digest,
                &dataset_digest,
                &arm_digest,
                arm,
                &stream_digest,
            )?;
            require_only_overrides(parent, &model, &witness.overrides)?;
            if model.artifact.parameters != fitted.artifact.parameters {
                return Err("final state witness replay".into());
            }
            let final_check = model.evaluate(docs, Intervention::Full)?;
            if final_check.mean_nll != after.mean_nll
                || final_check.correct != after.correct
                || final_check.positions != after.positions
            {
                return Err("fit/witness objective replay".into());
            }
            let id = digest(&bytes);
            let early = generate(&model, &prompts, &[Intervention::Full])?;
            write(
                &out,
                &format!("{arm}-early-generation.json"),
                &json!({"arm":arm,"witness":id,"outputs":early}),
            )?;
            fit_metrics.push(json!({"arm":arm,"witness":id,"before":before,"after":after,"changed_parameters":witness.overrides.len(),"accepted_updates":profile["accepted_updates"],"no_op_proposals_fully_evaluated":profile["no_op_proposals_fully_evaluated"]}));
            models.push(model);
            witnesses.push(id);
            early_outputs.push(early);
        }
        // Frozen new-combination probes become evaluation input only here.
        let probes = documents(&dataset["probe"])?;
        if probes.len() != 12
            || probes.iter().map(|d| d.len() + 1).sum::<usize>() != 672
            || probes.iter().any(|p| {
                varied.contains(p)
                    || repeated.contains(p)
                    || construction.contains(p)
                    || opened.contains(p)
            })
        {
            return Err("probe overlap or empty probe".into());
        }
        let mut probe_panels = Vec::new();
        let mut probe_full = Vec::new();
        let mut probe_generations = Vec::new();
        let probe_prompts: Vec<String> = serde_json::from_value(dataset["prompts"].clone())?;
        for (name, model) in [
            ("parent", parent),
            ("repeated", &models[0]),
            ("varied", &models[1]),
        ] {
            probe_generations
                .push(json!({"arm":name,"outputs":generate(model,&probe_prompts,&CONTROLS)?}));
            for control in CONTROLS {
                guard(start)?;
                let actual = rows(model, &probes, control)?;
                let metric = verified_metrics(model, &probes, control, &actual)?;
                if control == Intervention::Full {
                    probe_full.push(metric.clone());
                }
                probe_panels
                    .push(json!({"arm":name,"control":control,"metrics":metric,"rows":actual}));
            }
        }
        let method_gate = probe_full[2].mean_nll + EPS < probe_full[0].mean_nll
            && probe_full[2].mean_nll + EPS < probe_full[1].mean_nll
            && probe_full[2].correct >= probe_full[0].correct
            && probe_full[2].correct >= probe_full[1].correct;
        write(
            &out,
            "probe.json",
            &json!({"dataset":dataset_digest,"evaluated_after_both_final_witnesses":true,"final_witnesses":witnesses,"method_gate":method_gate,"panels":probe_panels,"generations":probe_generations,"fresh_independent_holdout_claim":false,"scope":"Frozen new combinations authored before this matched experiment; opened once after both final arms were fixed."}),
        )?;
        let previous = read_json(&input.join("behavior.json"))?;
        let sensitivity_previous = read_json(&sensitivity_input.join("behavior.json"))?;
        let mut bases = Vec::new();
        let mut prior_replay_rows = 0_usize;
        for (name, docs) in [
            ("complete_construction", &construction),
            ("opened_development", &opened),
        ] {
            for control in CONTROLS {
                let saved = saved_panel(&previous, name, control)?;
                let old_saved = saved_panel(&sensitivity_previous, name, control)?;
                let mut per_panel = Vec::new();
                for (index, model) in priors.iter().enumerate() {
                    guard(start)?;
                    let actual = rows(model, docs, control)?;
                    let metric = verified_metrics(model, docs, control, &actual)?;
                    let old = if index == 0 {
                        json!({"metrics":saved["candidate_metrics"],"rows":saved["candidate_rows"]})
                    } else {
                        saved["comparisons"]
                            .as_array()
                            .ok_or("prior comparisons")?
                            .iter()
                            .find(|v| v["parent"] == model.artifact_cid())
                            .ok_or("prior comparison")?
                            .clone()
                    };
                    saved_rows_equal(&old, &actual, &metric)?;
                    prior_replay_rows += actual.len();
                    per_panel.push((model.artifact_cid().to_string(), actual, metric));
                }
                let actual = rows(&old_witness, docs, control)?;
                let metric = verified_metrics(&old_witness, docs, control, &actual)?;
                if old_saved["witness"] != old_witness_id {
                    return Err("saved fifth witness id".into());
                }
                saved_rows_equal(
                    &json!({"metrics":old_saved["witness_metrics"],"rows":old_saved["witness_rows"]}),
                    &actual,
                    &metric,
                )?;
                prior_replay_rows += actual.len();
                per_panel.push((old_witness_id.clone(), actual, metric));
                bases.push((name, control, per_panel));
            }
        }
        let mut saved_generations = Vec::new();
        let mut prior_replay_outputs = 0_usize;
        for model in &priors {
            let outputs = generate(model, &prompts, &CONTROLS)?;
            let saved = previous["generations"]
                .as_array()
                .ok_or("prior generations")?
                .iter()
                .find(|v| v["artifact"] == model.artifact_cid())
                .ok_or("prior generation")?;
            if saved["outputs"] != serde_json::to_value(&outputs)? {
                return Err("prior generation differs".into());
            }
            prior_replay_outputs += outputs.len();
            saved_generations.push(json!({"artifact":model.artifact_cid(),"outputs":outputs}));
        }
        let old_outputs = generate(&old_witness, &prompts, &CONTROLS)?;
        let old_saved = sensitivity_previous["generations"]
            .as_array()
            .ok_or("fifth generations")?
            .iter()
            .find(|v| v["witness"] == old_witness_id)
            .ok_or("fifth generation")?;
        if old_saved["outputs"] != serde_json::to_value(&old_outputs)? {
            return Err("fifth generation differs".into());
        }
        prior_replay_outputs += old_outputs.len();
        saved_generations.push(json!({"witness":old_witness_id,"outputs":old_outputs}));
        let mut arm_results = Vec::new();
        let mut census = Vec::new();
        for (arm_index, arm) in ["repeated", "varied"].iter().enumerate() {
            let candidate = &models[arm_index];
            let mut panels = Vec::new();
            let mut lost = [0_u64; 5];
            let mut gained = [0_u64; 5];
            let mut root_hamming = [0_u64; 5];
            let mut source_hamming = [0_u64; 5];
            let mut prediction_hamming = [0_u64; 5];
            let mut full_metrics = Vec::new();
            let mut construction_gate = true;
            let mut opened_gate = true;
            for (name, control, base_panel) in &bases {
                guard(start)?;
                let docs = if *name == "complete_construction" {
                    &construction
                } else {
                    &opened
                };
                let actual = rows(candidate, docs, *control)?;
                let after = verified_metrics(candidate, docs, *control, &actual)?;
                let mut comparisons = Vec::new();
                for (i, (id, base, before)) in base_panel.iter().enumerate() {
                    let comparison = compare(base, &actual)?;
                    lost[i] += comparison["correct_to_wrong"].as_u64().ok_or("lost")?;
                    gained[i] += comparison["wrong_to_correct"].as_u64().ok_or("gained")?;
                    root_hamming[i] += comparison["root_identity_hamming"]
                        .as_u64()
                        .ok_or("roots")?;
                    source_hamming[i] += comparison["source_identity_hamming"]
                        .as_u64()
                        .ok_or("sources")?;
                    prediction_hamming[i] += comparison["prediction_hamming"]
                        .as_u64()
                        .ok_or("predictions")?;
                    if *control == Intervention::Full {
                        full_metrics
                            .push(json!({"panel":name,"prior":id,"before":before,"after":after}));
                        if *name == "complete_construction" && i == 0 {
                            construction_gate &= after.mean_nll <= before.mean_nll + EPS
                                && after.correct >= before.correct;
                        }
                        if *name == "opened_development" {
                            opened_gate &= after.mean_nll <= before.mean_nll + EPS
                                && after.correct >= before.correct;
                        }
                    }
                    comparisons.push(
                        json!({"prior":id,"metrics":before,"rows":base,"comparison":comparison}),
                    );
                }
                panels.push(json!({"panel":name,"control":control,"witness":witnesses[arm_index],"witness_metrics":after,"witness_rows":actual,"comparisons":comparisons}));
            }
            let outputs = generate(candidate, &prompts, &CONTROLS)?;
            for (i, early) in early_outputs[arm_index].iter().enumerate() {
                if early != &outputs[i * 3] {
                    return Err("early/final generation replay".into());
                }
            }
            let termination = early_outputs[arm_index].iter().all(|v| v["eos"] == true);
            let preservation = lost == [0; 5];
            arm_results.push(json!({"arm":arm,"witness":witnesses[arm_index],"construction_retention_gate":construction_gate,"opened_development_gate":opened_gate,"preservation_gate":preservation,"generation_termination_gate":termination,"numeric_model_gate":construction_gate&&opened_gate&&preservation&&termination,"correct_to_wrong_each_prior":lost,"wrong_to_correct_each_prior":gained,"root_identity_hamming_each_prior":root_hamming,"source_identity_hamming_each_prior":source_hamming,"prediction_hamming_each_prior":prediction_hamming,"full_metrics":full_metrics,"generation_coherence":"QUALITATIVE_REVIEW_REQUIRED","general_language_qualified":false}));
            write(
                &out,
                &format!("{arm}-behavior.json"),
                &json!({"panels":panels,"outputs":outputs,"prior_generations":saved_generations}),
            )?;
            let mut session = candidate.session(Intervention::Full);
            let mut maximum = 0;
            let mut predictions = Vec::new();
            for step in 0..512 {
                guard(start)?;
                let before = session.work();
                let prediction = session.predict();
                let count = session.work().products - before.products;
                if count > 27 {
                    return Err("angular predict bound".into());
                }
                maximum = maximum.max(count);
                predictions.push(prediction);
                session.observe((step & 255) as u8)?;
            }
            census.push(json!({"arm":arm,"witness":witnesses[arm_index],"steps":512,"max_angular_comparisons_per_predict":maximum,"bound":27,"work":session.work(),"predictions":predictions,"allocation_measurement":"NOT_RUN_IN_THIS_DRIVER"}));
        }
        write(&out, "runtime-census.json", &census)?;
        for (model, bytes) in priors.iter().zip(&prior_bytes) {
            if model.to_bytes()? != *bytes {
                return Err("prior mutated".into());
            }
        }
        for root in roots {
            crate::report_output::verify(root)?;
        }
        let final_gate = method_gate && arm_results[1]["numeric_model_gate"] == true;
        write(
            &out,
            "source.json",
            &json!({"implementation":SharedCore::implementation_digest(),"driver":driver_digest(),"tree_source":digest(include_bytes!("../angular_tree.rs")),"trace_reference_driver":digest(include_bytes!("active_sensitivity.rs")),"design":design_digest,"corpus":corpus_digest,"dataset":dataset_digest,"dataset_root":data_root,"proposal_stream":stream_digest,"parent":PARENT,"prior_tree":PRIOR_TREE,"cap":CAP,"older":OLDER,"prior_sensitivity_witness":old_witness_id,"witnesses":witnesses,"prior_artifact_sha256":prior_bytes.iter().map(|b|format!("{:x}",sha2::Sha256::digest(b))).collect::<Vec<_>>(),"prior_report_roots":[input,prior_input,cap_input,older_input,sensitivity_input],"standard_changed_model_artifact_exported":false}),
        )?;
        write(
            &out,
            "result.json",
            &json!({"schema":"uor-r4.context-augmentation-result/1","decision":if final_gate{"PASS_CONTEXT_AUGMENTATION_DEVELOPMENT_GATE"}else{"FAIL_CONTEXT_AUGMENTATION_DEVELOPMENT_GATE"},"method_gate":method_gate,"arms":arm_results,"fits":fit_metrics,"fit_calls":2,"proposals_per_arm":PROPOSALS,"proposal_evaluate_calls":2*PROPOSALS,"total_proposal_positions":2*PROPOSALS*2688,"prior_replayed_rows":prior_replay_rows,"prior_replayed_outputs":prior_replay_outputs,"probe_full_metrics":{"parent":probe_full[0],"repeated":probe_full[1],"varied":probe_full[2]},"elapsed_ms":start.elapsed().as_millis(),"decoder_unchanged":true,"decoder_refits":0,"promoted":false,"scope":"One matched varied-context versus repeated-original construction-only state learning experiment under fixed geometry and decoder. Both final witnesses fixed before new-combination probes. All four prior artifacts and prior sensitivity witness replayed. No fresh broad holdout, JEPA objective, standard changed artifact, promotion, language or energy qualification."}),
        )?;
        guard(start)?;
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
        return Err("sealed attempt exceeds storage limit".into());
    }
    result
}
