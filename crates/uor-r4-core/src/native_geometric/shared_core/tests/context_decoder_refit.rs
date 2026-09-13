//! Four frozen state/data cells with one fixed conditional decoder refit rule.
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
        return Err("complete conditional decoder experiment time guard".into());
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
    digest(include_bytes!("context_decoder_refit.rs"))
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
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StateWitness {
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
impl StateWitness {
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
            || self.driver != digest(include_bytes!("context_augmentation.rs"))
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

const CELLS: [&str; 4] = [
    "parent_repeated",
    "learned_repeated",
    "parent_varied",
    "learned_varied",
];
const STATE_FOR_CELL: [usize; 4] = [0, 5, 0, 6];
const DATA_FOR_CELL: [usize; 4] = [0, 0, 1, 1];
fn config() -> AngularTreeConfig {
    AngularTreeConfig {
        max_depth: 3,
        min_leaf: 64,
        max_seconds: 60,
    }
}
fn training_digest(docs: &[Vec<u8>]) -> String {
    let mut hash = blake3::Hasher::new();
    hash.update(b"uor-r4.angular-emission-training/1");
    for doc in docs {
        hash.update(&(doc.len() as u64).to_le_bytes());
        hash.update(doc);
    }
    format!("blake3:{}", hash.finalize())
}
fn ephemeral_cap(cap: &SharedCore, state: &SharedCore) -> RunResult<SharedCore> {
    if cap.artifact.angular_tree.is_some()
        || cap.artifact.calibrated.is_none()
        || cap.artifact.parameters.len() != state.artifact.parameters.len()
    {
        return Err("ephemeral cap input".into());
    }
    let mut changed = cap.clone();
    changed
        .artifact
        .parameters
        .clone_from(&state.artifact.parameters);
    // Only the fixed state can differ. Inherited numerical caps remain exact.
    for i in 0..PARAMETERS {
        if !(TRANSITION..PHASE).contains(&i)
            && cap.artifact.parameters[i] != state.artifact.parameters[i]
        {
            return Err("state changed frozen parameter".into());
        }
    }
    if serde_json::to_value(&cap.artifact.geometry)?
        != serde_json::to_value(&state.artifact.geometry)?
        || serde_json::to_value(&cap.artifact.calibrated)?
            != serde_json::to_value(&state.artifact.calibrated)?
    {
        return Err("state changed caps or geometry".into());
    }
    changed.refresh_identity()?;
    Ok(changed)
}
fn require_only_head_fields(state: &SharedCore, changed: &SharedCore) -> RunResult<()> {
    let mut actual = serde_json::to_value(&changed.artifact)?;
    let mut expected = serde_json::to_value(&state.artifact)?;
    for value in [&mut actual, &mut expected] {
        let object = value.as_object_mut().ok_or("artifact object")?;
        for key in [
            "angular_tree",
            "schema",
            "implementation",
            "training_digest",
            "training_parent",
            "fit_config",
            "tied_fit_config",
        ] {
            object.remove(key);
        }
    }
    if actual != expected {
        return Err("conditional refit changed fixed state/caps/geometry".into());
    }
    Ok(())
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Witness {
    schema: String,
    cell: String,
    cap: String,
    state: String,
    state_parameters: String,
    implementation: String,
    driver: String,
    design: String,
    corpus: String,
    dataset: String,
    context_source: String,
    data: String,
    ephemeral_cap: String,
    head_digest: String,
    head: AngularEmission,
}
impl Witness {
    fn replay(
        &self,
        cap: &SharedCore,
        state: &SharedCore,
        cell: &str,
        state_id: &str,
        design: &str,
        corpus: &str,
        dataset: &str,
        context_source: &str,
        data: &str,
    ) -> RunResult<SharedCore> {
        if self.schema != "uor-r4.context-decoder-refit-witness/1"
            || self.cell != cell
            || !CELLS.contains(&cell)
            || self.cap != cap.artifact_cid()
            || self.state != state_id
            || self.state_parameters != digest(&serde_json::to_vec(&state.artifact.parameters)?)
            || self.implementation != SharedCore::implementation_digest()
            || self.driver != driver_digest()
            || self.design != design
            || self.corpus != corpus
            || self.dataset != dataset
            || self.context_source != context_source
            || self.data != data
        {
            return Err("conditional head witness source/state binding".into());
        }
        let mut model = ephemeral_cap(cap, state)?;
        self.head.validate()?;
        if model.artifact_cid() != self.ephemeral_cap
            || self.head.parent != self.ephemeral_cap
            || self.head.data_digest != data
            || serde_json::to_value(self.head.config)? != serde_json::to_value(config())?
            || self.head_digest != digest(&serde_json::to_vec(&self.head)?)
        {
            return Err("conditional head payload lineage".into());
        }
        model.artifact.schema = ANGULAR_TREE_SCHEMA.into();
        model.artifact.implementation = SharedCore::implementation_digest();
        model.artifact.training_digest = None;
        model.artifact.training_parent = None;
        model.artifact.fit_config = None;
        model.artifact.tied_fit_config = None;
        model.artifact.angular_tree = Some(self.head.clone());
        require_only_head_fields(state, &model)?;
        // This intervention is identified only by its exported witness, never
        // by the stale in-memory model CID or a normal changed-model artifact.
        Ok(model)
    }
}
fn same_state(base: &[Row], changed: &[Row]) -> RunResult<()> {
    if base.len() != changed.len()
        || base.iter().zip(changed).any(|(a, b)| {
            (a.document, a.position, a.target, a.roots, a.source)
                != (b.document, b.position, b.target, b.roots, b.source)
        })
    {
        return Err("decoder refit changed teacher-forced state/source".into());
    }
    Ok(())
}
fn benefit(before: &Metrics, after: &Metrics) -> bool {
    after.mean_nll + EPS < before.mean_nll
        && after.correct >= before.correct
        && before.positions == after.positions
}
fn metric_equal(a: &Metrics, b: &Metrics) -> bool {
    a.positions == b.positions && a.correct == b.correct && (a.mean_nll - b.mean_nll).abs() <= EPS
}
#[test]
fn context_decoder_refit_repetition_lineage_cells_and_comparison() -> RunResult<()> {
    assert_eq!(CELLS[0], "parent_repeated");
    assert_eq!(STATE_FOR_CELL, [0, 5, 0, 6]);
    assert_eq!(DATA_FOR_CELL, [0, 0, 1, 1]);
    let cap = SharedCore::initialized(745)?.upgrade_calibrated()?;
    let docs = vec![
        b"red red blue blue red blue.".to_vec(),
        b"one two one two one two.".to_vec(),
    ];
    let repeated = docs
        .iter()
        .flat_map(|d| std::iter::repeat_n(d.clone(), 4))
        .collect::<Vec<_>>();
    let (original, _) = cap.fit_angular_emission(
        &docs,
        AngularTreeConfig {
            max_depth: 1,
            min_leaf: 4,
            max_seconds: 60,
        },
    )?;
    let (duplicated, _) = cap.fit_angular_emission(
        &repeated,
        AngularTreeConfig {
            max_depth: 1,
            min_leaf: 16,
            max_seconds: 60,
        },
    )?;
    let original_head = original
        .artifact
        .angular_tree
        .as_ref()
        .ok_or("test original head")?;
    let duplicated_head = duplicated
        .artifact
        .angular_tree
        .as_ref()
        .ok_or("test duplicate head")?;
    assert_eq!(
        serde_json::to_value(&original_head.branches)?,
        serde_json::to_value(&duplicated_head.branches)?
    );
    assert_ne!(original_head.data_digest, duplicated_head.data_digest);
    let mut head = original_head.clone();
    head.config = config();
    let ephemeral = ephemeral_cap(&cap, &original)?;
    head.parent = ephemeral.artifact_cid().into();
    let w = Witness {
        schema: "uor-r4.context-decoder-refit-witness/1".into(),
        cell: CELLS[0].into(),
        cap: cap.artifact_cid().into(),
        state: "state".into(),
        state_parameters: digest(&serde_json::to_vec(&original.artifact.parameters)?),
        implementation: SharedCore::implementation_digest(),
        driver: driver_digest(),
        design: "design".into(),
        corpus: "corpus".into(),
        dataset: "dataset".into(),
        context_source: "source".into(),
        data: head.data_digest.clone(),
        ephemeral_cap: ephemeral.artifact_cid().into(),
        head_digest: digest(&serde_json::to_vec(&head)?),
        head,
    };
    let loaded = w.replay(
        &cap, &original, CELLS[0], "state", "design", "corpus", "dataset", "source", &w.data,
    )?;
    require_only_head_fields(&original, &loaded)?;
    assert!(w
        .replay(
            &cap, &original, CELLS[1], "state", "design", "corpus", "dataset", "source", &w.data
        )
        .is_err());
    let mut wrong = loaded.clone();
    wrong.artifact.parameters[READ] = (wrong.artifact.parameters[READ] + 1) % 120;
    assert!(require_only_head_fields(&original, &wrong).is_err());
    let mut bad = w.clone();
    bad.head.parent = "unbound".into();
    assert!(bad
        .replay(
            &cap, &original, CELLS[0], "state", "design", "corpus", "dataset", "source", &w.data
        )
        .is_err());
    let a = rows(&original, &docs, Intervention::Full)?;
    let b = rows(&loaded, &docs, Intervention::Full)?;
    same_state(&a, &b)?;
    let m = Metrics {
        positions: 10,
        correct: 4,
        mean_nll: 2.0,
        work: Work::default(),
    };
    let mut n = m.clone();
    assert!(!benefit(&m, &n));
    n.mean_nll = 1.9;
    assert!(benefit(&m, &n));
    n.correct = 3;
    assert!(!benefit(&m, &n));
    Ok(())
}

#[test]
#[ignore = "requires sealed context witnesses/data and frozen four-cell refit design"]
fn context_decoder_refit_run_saved_experiment() -> RunResult<()> {
    let input = fs::canonicalize(std::env::var("UOR_CONTEXT_REFIT_INPUT")?)?;
    let evidence = input
        .parent()
        .and_then(Path::parent)
        .ok_or("evidence parent")?;
    let decoder_input = fs::canonicalize(evidence.join("decoder-validation-1/attempt-1"))?;
    let prior_input = fs::canonicalize(evidence.join("angular-tree-1/attempt-1"))?;
    let cap_input = fs::canonicalize(evidence.join("final-emission-1/attempt-1"))?;
    let older_input = fs::canonicalize(evidence.join("loss-frontier-1/joint-1"))?;
    let sensitivity_input = fs::canonicalize(evidence.join("active-sensitivity-1/attempt-1"))?;
    let corpus = fs::canonicalize(std::env::var("UOR_CONTEXT_REFIT_CORPUS")?)?;
    let path = fs::canonicalize(std::env::var("UOR_CONTEXT_REFIT_DESIGN")?)?;
    let raw = std::path::PathBuf::from(std::env::var("UOR_CONTEXT_REFIT_OUTPUT")?);
    let out = fs::canonicalize(raw.parent().ok_or("output parent")?)?
        .join(raw.file_name().ok_or("output name")?);
    let context_source_bytes = fs::read(input.join("source.json"))?;
    let context_source: Value = serde_json::from_slice(&context_source_bytes)?;
    let data_root = fs::canonicalize(
        context_source["dataset_root"]
            .as_str()
            .ok_or("dataset root")?,
    )?;
    let roots = [
        &input,
        &decoder_input,
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
        let expected = json!({"schema":"uor-r4.context-decoder-refit-design/1","parent":PARENT,"prior_tree":PRIOR_TREE,"cap":CAP,"older":OLDER,"corpus_sha256":"541a094ae83a56adbcabb0cd157f0a7a14aef23b6db29f82f2d794975d420adf","cells":CELLS,"config":{"max_depth":3,"min_leaf":64,"max_seconds":60},"fit_calls":4,"state_proposals":0,"experiment_seconds":180,"attempt_bytes":67108864,"head_bytes":131072,"documents_per_fit":48,"positions_per_fit":2688,"construction_positions":672,"opened_positions":381,"probe_positions":672,"original_prompts":4,"probe_prompts":4,"generation_bytes":96,"baseline_reproduction":"Exact angular branch payload equality to 8f8; metadata may differ; first fit only before gate","failed_reproduction":"Stop remaining fits and seal FAILED_REPRODUCTION","gates":{"probe_full_loss_strict_improvement":true,"probe_full_accuracy_retained":true,"correct_to_wrong_each_prior_max":0,"construction_full_loss_nonincreasing":true,"construction_full_accuracy_retained":true,"opened_full_loss_nonincreasing":true,"opened_full_accuracy_retained":true,"generation_termination_required":true},"contrasts":{"decoder_only":["parent_varied","parent_repeated"],"varied_state_increment":["learned_varied","parent_varied"],"repeated_state_increment":["learned_repeated","parent_repeated"],"total_varied_pipeline":["learned_varied","learned_repeated"]},"final_method_rule":"decoder_only AND varied_state_increment; all four contrasts separately reported; each requires probe Full NLL improvement >1e-10 and correct count nondecrease","promotion":false});
        for (key, value) in expected.as_object().ok_or("design object")? {
            if design[key] != *value {
                return Err(format!("frozen design field differs: {key}").into());
            }
        }
        fs::File::create_new(out.join("design.json"))?.write_all(&design_bytes)?;
        let corpus_bytes = fs::read(corpus.join("design.json"))?;
        let dataset_bytes = fs::read(data_root.join("dataset.json"))?;
        if design["corpus_sha256"] != format!("{:x}", sha2::Sha256::digest(&corpus_bytes))
            || design["dataset_sha256"] != format!("{:x}", sha2::Sha256::digest(&dataset_bytes))
            || design["context_source_sha256"]
                != format!("{:x}", sha2::Sha256::digest(&context_source_bytes))
        {
            return Err("frozen input SHA binding".into());
        }
        let design_digest = digest(&design_bytes);
        let corpus_digest = digest(&corpus_bytes);
        let dataset_digest = digest(&dataset_bytes);
        let context_digest = digest(&context_source_bytes);
        if context_source["implementation"] != SharedCore::implementation_digest()
            || context_source["driver"] != digest(include_bytes!("context_augmentation.rs"))
            || context_source["design"] != digest(&fs::read(input.join("design.json"))?)
            || context_source["corpus"] != corpus_digest
            || context_source["dataset"] != dataset_digest
            || context_source["proposal_stream"]
                != digest(&fs::read(input.join("proposal-stream.json"))?)
            || context_source["parent"] != PARENT
            || context_source["prior_tree"] != PRIOR_TREE
            || context_source["cap"] != CAP
            || context_source["older"] != OLDER
        {
            return Err("context source bindings".into());
        }
        let data_source = read_json(&data_root.join("source.json"))?;
        if data_source["preparation_source_blake3"] != digest(include_bytes!("context_data.rs"))
            || data_source["dataset_blake3"] != dataset_digest
            || data_source["corpus_sha256"] != design["corpus_sha256"]
            || data_source["model_calls"] != 0
        {
            return Err("prepared data binding".into());
        }
        let original: Value = serde_json::from_slice(&corpus_bytes)?;
        let dataset: Value = serde_json::from_slice(&dataset_bytes)?;
        let construction = documents(&original["training"])?;
        let opened = documents(&original["holdout"])?;
        let repeated = documents(&dataset["repeated"])?;
        let varied = documents(&dataset["varied"])?;
        let prompts: Vec<String> = serde_json::from_value(original["prompts"].clone())?;
        if construction.len() != 12
            || construction.iter().map(|d| d.len() + 1).sum::<usize>() != 672
            || opened.iter().map(|d| d.len() + 1).sum::<usize>() != 381
            || prompts.len() != 4
            || [repeated.as_slice(), varied.as_slice()]
                .iter()
                .any(|d| d.len() != 48 || d.iter().map(|x| x.len() + 1).sum::<usize>() != 2688)
            || repeated
                != construction
                    .iter()
                    .flat_map(|d| std::iter::repeat_n(d.clone(), 4))
                    .collect::<Vec<_>>()
            || repeated
                .iter()
                .zip(&varied)
                .any(|(a, b)| a.len() != b.len())
        {
            return Err("frozen data dimensions/repetition".into());
        }
        let datasets = [&repeated, &varied];
        let mut priors = Vec::new();
        let mut ids = Vec::new();
        let mut prior_bytes = Vec::new();
        for (root, cid) in [
            (&decoder_input, PARENT),
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
            ids.push(cid.to_string());
            priors.push(model);
        }
        let (old_state, old_id) = load_sensitivity(&priors[0], &sensitivity_input, &corpus_digest)?;
        if context_source["prior_sensitivity_witness"] != old_id {
            return Err("context sensitivity lineage".into());
        }
        priors.push(old_state);
        ids.push(old_id);
        for (arm_index, arm) in ["repeated", "varied"].iter().enumerate() {
            let bytes = fs::read(input.join(format!("{arm}-witness.json")))?;
            let w: StateWitness = serde_json::from_slice(&bytes)?;
            let model = w.replay(
                &priors[0],
                context_source["design"].as_str().ok_or("context design")?,
                &corpus_digest,
                &dataset_digest,
                &digest(&serde_json::to_vec(datasets[arm_index])?),
                arm,
                context_source["proposal_stream"]
                    .as_str()
                    .ok_or("stream digest")?,
            )?;
            let id = digest(&bytes);
            if context_source["witnesses"][arm_index] != id {
                return Err("state witness identity".into());
            }
            priors.push(model);
            ids.push(id);
        }
        let parent = &priors[0];
        let cap = &priors[2];
        write(
            &out,
            "source.json",
            &json!({"implementation":SharedCore::implementation_digest(),"driver":driver_digest(),"tree_source":digest(include_bytes!("../angular_tree.rs")),"context_driver":digest(include_bytes!("context_augmentation.rs")),"design":design_digest,"corpus":corpus_digest,"dataset":dataset_digest,"context_source":context_digest,"input":input,"dataset_root":data_root,"prior_identities":ids,"prior_artifact_sha256":prior_bytes.iter().map(|b|format!("{:x}",sha2::Sha256::digest(b))).collect::<Vec<_>>(),"cells":CELLS,"state_for_cell":STATE_FOR_CELL,"data_for_cell":DATA_FOR_CELL,"state_proposals":0,"standard_changed_model_artifact_exported":false}),
        )?;
        let mut candidates = Vec::new();
        let mut witness_ids = Vec::new();
        let mut early = Vec::new();
        let mut fit_reports = Vec::new();
        for (cell_index, cell) in CELLS.iter().enumerate() {
            guard(start)?;
            let state = &priors[STATE_FOR_CELL[cell_index]];
            let docs = datasets[DATA_FOR_CELL[cell_index]];
            let temporary = ephemeral_cap(cap, state)?;
            let ephemeral_id = temporary.artifact_cid().to_string();
            let (fitted, fit) = temporary.fit_angular_emission(docs, config())?;
            require_only_head_fields(state, &fitted)?;
            let head = fitted.artifact.angular_tree.as_ref().ok_or("fitted head")?;
            head.validate()?;
            let data = training_digest(docs);
            if head.parent != ephemeral_id
                || head.data_digest != data
                || serde_json::to_value(head.config)? != serde_json::to_value(config())?
                || fit.parent != ephemeral_id
                || fit.data_digest != data
                || fit.after.positions != 2688
                || !fit.after.mean_nll.is_finite()
            {
                return Err("fit head lineage or dimensions".into());
            }
            let head_bytes = serde_json::to_vec(head)?;
            if head_bytes.len() > 131072 || serde_json::to_vec_pretty(head)?.len() > 131072 {
                return Err("head storage bound".into());
            }
            write(&out, &format!("{cell}-head.json"), head)?;
            write(&out, &format!("{cell}-fit.json"), &fit)?;
            let witness = Witness {
                schema: "uor-r4.context-decoder-refit-witness/1".into(),
                cell: (*cell).into(),
                cap: CAP.into(),
                state: ids[STATE_FOR_CELL[cell_index]].clone(),
                state_parameters: digest(&serde_json::to_vec(&state.artifact.parameters)?),
                implementation: SharedCore::implementation_digest(),
                driver: driver_digest(),
                design: design_digest.clone(),
                corpus: corpus_digest.clone(),
                dataset: dataset_digest.clone(),
                context_source: context_digest.clone(),
                data: data.clone(),
                ephemeral_cap: ephemeral_id,
                head_digest: digest(&head_bytes),
                head: head.clone(),
            };
            write(&out, &format!("{cell}-witness.json"), &witness)?;
            let saved = fs::read(out.join(format!("{cell}-witness.json")))?;
            let replayed: Witness = serde_json::from_slice(&saved)?;
            let saved_head: AngularEmission =
                serde_json::from_slice(&fs::read(out.join(format!("{cell}-head.json")))?)?;
            if serde_json::to_vec(&saved_head)? != serde_json::to_vec(&replayed.head)? {
                return Err("saved head differs from witness".into());
            }
            let candidate = replayed.replay(
                cap,
                state,
                cell,
                &ids[STATE_FOR_CELL[cell_index]],
                &design_digest,
                &corpus_digest,
                &dataset_digest,
                &context_digest,
                &data,
            )?;
            let checked = candidate.evaluate(docs, Intervention::Full)?;
            if !metric_equal(&checked, &fit.after) {
                return Err("fit/head witness metrics replay".into());
            }
            let witness_id = digest(&saved);
            if cell_index == 0 {
                let baseline = parent
                    .artifact
                    .angular_tree
                    .as_ref()
                    .ok_or("baseline head")?;
                let expected_payload = serde_json::to_vec(&baseline.branches)?;
                let actual_payload = serde_json::to_vec(&head.branches)?;
                let matching = expected_payload == actual_payload;
                let differences = baseline
                    .branches
                    .iter()
                    .zip(&head.branches)
                    .enumerate()
                    .filter_map(|(i, (a, b))| {
                        (serde_json::to_value(a).ok() != serde_json::to_value(b).ok()).then_some(i)
                    })
                    .collect::<Vec<_>>();
                write(
                    &out,
                    "baseline-reproduction.json",
                    &json!({"matched":matching,"fit_calls":1,"remaining_fits_started":false,"expected_parent":PARENT,"expected_branch_payload":digest(&expected_payload),"actual_branch_payload":digest(&actual_payload),"different_branch_indices":differences,"compared_exact_branch_payload_only":true,"metadata_expected_to_differ":true,"witness":witness_id}),
                )?;
                if !matching {
                    write(
                        &out,
                        "result.json",
                        &json!({"schema":"uor-r4.context-decoder-refit-result/1","decision":"FAILED_REPRODUCTION","fit_calls":1,"remaining_fit_calls_not_run":3,"state_proposals":0,"witness":witness_id,"actual_head_file":format!("{cell}-head.json"),"elapsed_ms":start.elapsed().as_millis(),"promoted":false,"scope":"Repeated-data minimum-leaf64 first fit did not reproduce exact8f8 branches. Remaining cells and quality comparisons NOT_RUN; no automatic retry."}),
                    )?;
                    return Ok(());
                }
            }
            let outputs = generate(&candidate, &prompts, &[Intervention::Full])?;
            write(
                &out,
                &format!("{cell}-early-generation.json"),
                &json!({"cell":cell,"witness":witness_id,"outputs":outputs}),
            )?;
            fit_reports.push(json!({"cell":cell,"witness":witness_id,"state":ids[STATE_FOR_CELL[cell_index]],"data_arm":if DATA_FOR_CELL[cell_index]==0{"repeated"}else{"varied"},"fit":fit,"head_bytes":head_bytes.len()}));
            candidates.push(candidate);
            witness_ids.push(witness_id);
            early.push(outputs);
        }
        // All four prescribed heads and state-bound witnesses are now fixed.
        let probes = documents(&dataset["probe"])?;
        let probe_prompts: Vec<String> = serde_json::from_value(dataset["prompts"].clone())?;
        if probes.len() != 12
            || probes.iter().map(|d| d.len() + 1).sum::<usize>() != 672
            || probe_prompts.len() != 4
        {
            return Err("probe dimensions".into());
        }
        let previous = read_json(&decoder_input.join("behavior.json"))?;
        let sensitivity_previous = read_json(&sensitivity_input.join("behavior.json"))?;
        let context_previous = [
            read_json(&input.join("repeated-behavior.json"))?,
            read_json(&input.join("varied-behavior.json"))?,
        ];
        let mut prior_panels = Vec::new();
        let mut prior_rows = 0;
        let mut saved_prior_panels = Vec::new();
        for (name, docs) in [
            ("complete_construction", &construction),
            ("opened_development", &opened),
        ] {
            for control in CONTROLS {
                let old_panel = saved_panel(&previous, name, control)?;
                let sensitivity_panel = saved_panel(&sensitivity_previous, name, control)?;
                let mut panel = Vec::new();
                for (index, model) in priors.iter().enumerate() {
                    guard(start)?;
                    let actual = rows(model, docs, control)?;
                    let metrics = verified_metrics(model, docs, control, &actual)?;
                    let saved = match index {
                        0 => {
                            json!({"metrics":old_panel["candidate_metrics"],"rows":old_panel["candidate_rows"]})
                        }
                        1..=3 => old_panel["comparisons"]
                            .as_array()
                            .ok_or("prior comparisons")?
                            .iter()
                            .find(|v| v["parent"] == ids[index])
                            .ok_or("prior comparison")?
                            .clone(),
                        4 => {
                            if sensitivity_panel["witness"] != ids[index] {
                                return Err("old witness panel id".into());
                            }
                            json!({"metrics":sensitivity_panel["witness_metrics"],"rows":sensitivity_panel["witness_rows"]})
                        }
                        5..=6 => {
                            let p = saved_panel(&context_previous[index - 5], name, control)?;
                            if p["witness"] != ids[index] {
                                return Err("context witness panel id".into());
                            }
                            json!({"metrics":p["witness_metrics"],"rows":p["witness_rows"]})
                        }
                        _ => return Err("unexpected prior count".into()),
                    };
                    saved_rows_equal(&saved, &actual, &metrics)?;
                    prior_rows += actual.len();
                    saved_prior_panels.push(json!({"panel":name,"control":control,"prior":ids[index],"metrics":metrics,"rows":actual}));
                    panel.push((actual, metrics));
                }
                prior_panels.push((name, control, panel));
            }
        }
        let mut prior_outputs = Vec::new();
        let mut replayed_prior_outputs = 0;
        for (index, model) in priors.iter().enumerate() {
            guard(start)?;
            let outputs = generate(model, &prompts, &CONTROLS)?;
            let saved = match index {
                0..=3 => previous["generations"]
                    .as_array()
                    .ok_or("saved generations")?
                    .iter()
                    .find(|v| v["artifact"] == ids[index])
                    .ok_or("saved prior generation")?["outputs"]
                    .clone(),
                4 => sensitivity_previous["generations"]
                    .as_array()
                    .ok_or("sensitivity generations")?
                    .iter()
                    .find(|v| v["witness"] == ids[index])
                    .ok_or("sensitivity generation")?["outputs"]
                    .clone(),
                5..=6 => context_previous[index - 5]["outputs"].clone(),
                _ => return Err("prior generation index".into()),
            };
            if serde_json::to_value(&outputs)? != saved {
                return Err("saved original continuation replay".into());
            }
            replayed_prior_outputs += outputs.len();
            prior_outputs.push(json!({"prior":ids[index],"outputs":outputs}));
        }
        write(
            &out,
            "prior-behavior.json",
            &json!({"panels":saved_prior_panels,"generations":prior_outputs,"replayed_rows":prior_rows,"replayed_outputs":replayed_prior_outputs}),
        )?;
        let previous_probe = read_json(&input.join("probe.json"))?;
        let mut probe_bases = Vec::new();
        let mut prior_probe_rows = 0;
        let mut prior_probe_outputs = Vec::new();
        for (name, index) in [("parent", 0), ("repeated", 5), ("varied", 6)] {
            let model = &priors[index];
            let mut panels = Vec::new();
            for control in CONTROLS {
                guard(start)?;
                let actual = rows(model, &probes, control)?;
                let metrics = verified_metrics(model, &probes, control, &actual)?;
                let c = serde_json::to_value(control)?;
                let saved = previous_probe["panels"]
                    .as_array()
                    .ok_or("probe panels")?
                    .iter()
                    .find(|v| v["arm"] == name && v["control"] == c)
                    .ok_or("prior probe panel")?;
                saved_rows_equal(saved, &actual, &metrics)?;
                prior_probe_rows += actual.len();
                panels.push((actual, metrics));
            }
            let outputs = generate(model, &probe_prompts, &CONTROLS)?;
            let saved = previous_probe["generations"]
                .as_array()
                .ok_or("probe generations")?
                .iter()
                .find(|v| v["arm"] == name)
                .ok_or("prior probe generation")?;
            if serde_json::to_value(&outputs)? != saved["outputs"] {
                return Err("prior probe generation replay".into());
            }
            prior_probe_outputs.push(json!({"prior":ids[index],"outputs":outputs}));
            probe_bases.push(panels);
        }
        let mut cell_results = Vec::new();
        let mut probe_metrics = Vec::new();
        let mut census = Vec::new();
        for (cell_index, cell) in CELLS.iter().enumerate() {
            let candidate = &candidates[cell_index];
            let mut panels = Vec::new();
            let mut lost = [0_u64; 7];
            let mut gained = [0_u64; 7];
            let mut prediction_hamming = [0_u64; 7];
            let mut root_hamming = [0_u64; 7];
            let mut source_hamming = [0_u64; 7];
            let mut construction_gate = true;
            let mut opened_gate = true;
            let mut full_metrics = Vec::new();
            for (name, control, base_panel) in &prior_panels {
                guard(start)?;
                let docs = if *name == "complete_construction" {
                    &construction
                } else {
                    &opened
                };
                let actual = rows(candidate, docs, *control)?;
                let after = verified_metrics(candidate, docs, *control, &actual)?;
                same_state(&base_panel[STATE_FOR_CELL[cell_index]].0, &actual)?;
                let mut comparisons = Vec::new();
                for (i, (base, before)) in base_panel.iter().enumerate() {
                    let comparison = compare(base, &actual)?;
                    lost[i] += comparison["correct_to_wrong"].as_u64().ok_or("lost")?;
                    gained[i] += comparison["wrong_to_correct"].as_u64().ok_or("gained")?;
                    prediction_hamming[i] += comparison["prediction_hamming"]
                        .as_u64()
                        .ok_or("prediction hamming")?;
                    root_hamming[i] += comparison["root_identity_hamming"]
                        .as_u64()
                        .ok_or("root hamming")?;
                    source_hamming[i] += comparison["source_identity_hamming"]
                        .as_u64()
                        .ok_or("source hamming")?;
                    if *control == Intervention::Full {
                        full_metrics.push(
                            json!({"panel":name,"prior":ids[i],"before":before,"after":after}),
                        );
                        if *name == "complete_construction" && i == 0 {
                            construction_gate &= after.mean_nll <= before.mean_nll + EPS
                                && after.correct >= before.correct;
                        }
                        if *name == "opened_development" {
                            opened_gate &= after.mean_nll <= before.mean_nll + EPS
                                && after.correct >= before.correct;
                        }
                    }
                    comparisons
                        .push(json!({"prior":ids[i],"before":before,"comparison":comparison}));
                }
                panels.push(json!({"panel":name,"control":control,"witness":witness_ids[cell_index],"metrics":after,"rows":actual,"comparisons":comparisons,"fixed_state_and_source_reproduced":true}));
            }
            let mut probe_panels = Vec::new();
            let mut full = None;
            let base_index = match STATE_FOR_CELL[cell_index] {
                0 => 0,
                5 => 1,
                6 => 2,
                _ => return Err("cell state mapping".into()),
            };
            for (control_index, control) in CONTROLS.iter().enumerate() {
                guard(start)?;
                let actual = rows(candidate, &probes, *control)?;
                let after = verified_metrics(candidate, &probes, *control, &actual)?;
                same_state(&probe_bases[base_index][control_index].0, &actual)?;
                if cell_index == 0 {
                    // Exact branch reproduction must also retain the complete
                    // fixed-parent probe prediction and loss rows.
                    saved_rows_equal(
                        &json!({"metrics":probe_bases[0][control_index].1,"rows":probe_bases[0][control_index].0}),
                        &actual,
                        &after,
                    )?;
                }
                let mut comparisons = Vec::new();
                for (i, name) in ["parent", "repeated", "varied"].iter().enumerate() {
                    comparisons.push(json!({"prior_state":name,"before":probe_bases[i][control_index].1,"comparison":compare(&probe_bases[i][control_index].0,&actual)?}));
                }
                if *control == Intervention::Full {
                    full = Some(after.clone());
                }
                probe_panels.push(json!({"control":control,"metrics":after,"rows":actual,"comparisons":comparisons,"fixed_state_and_source_reproduced":true}));
            }
            probe_metrics.push(full.ok_or("probe Full metric")?);
            let outputs = generate(candidate, &prompts, &CONTROLS)?;
            for (i, first) in early[cell_index].iter().enumerate() {
                if first != &outputs[i * 3] {
                    return Err("early original output replay".into());
                }
            }
            let probe_outputs = generate(candidate, &probe_prompts, &CONTROLS)?;
            let original_termination = early[cell_index].iter().all(|v| v["eos"] == true);
            let probe_termination = probe_outputs.iter().step_by(3).all(|v| v["eos"] == true);
            let termination = original_termination && probe_termination;
            let preservation = lost == [0; 7];
            cell_results.push(json!({"cell":cell,"witness":witness_ids[cell_index],"fixed_state":ids[STATE_FOR_CELL[cell_index]],"construction_retention_gate":construction_gate,"opened_development_gate":opened_gate,"preservation_gate":preservation,"generation_termination_gate":termination,"original_four_termination_gate":original_termination,"probe_four_termination_gate":probe_termination,"numeric_model_gate":construction_gate&&opened_gate&&preservation&&termination,"correct_to_wrong_each_prior":lost,"wrong_to_correct_each_prior":gained,"prediction_hamming_each_prior":prediction_hamming,"root_identity_hamming_each_prior":root_hamming,"source_identity_hamming_each_prior":source_hamming,"full_metrics":full_metrics,"generation_coherence":"QUALITATIVE_REVIEW_REQUIRED","general_language_qualified":false}));
            write(
                &out,
                &format!("{cell}-behavior.json"),
                &json!({"panels":panels,"outputs":outputs,"probe_panels":probe_panels,"probe_outputs":probe_outputs}),
            )?;
            let mut session = candidate.session(Intervention::Full);
            let mut max_comparisons = 0;
            let mut predictions = Vec::new();
            for step in 0..512 {
                guard(start)?;
                let before = session.work();
                let prediction = session.predict();
                let comparisons = session.work().products - before.products;
                if comparisons > 27 {
                    return Err("angular comparison bound".into());
                }
                max_comparisons = max_comparisons.max(comparisons);
                predictions.push(prediction);
                session.observe((step & 255) as u8)?;
            }
            census.push(json!({"cell":cell,"witness":witness_ids[cell_index],"steps":512,"max_angular_comparisons_per_predict":max_comparisons,"bound":27,"work":session.work(),"predictions":predictions,"allocation_measurement":"NOT_RUN_IN_THIS_DRIVER"}));
        }
        let contrasts=[("decoder_only",2,0),("varied_state_increment",3,2),("repeated_state_increment",1,0),("total_varied_pipeline",3,1)].map(|(name,changed,base)|json!({"name":name,"candidate":CELLS[changed],"baseline":CELLS[base],"before":probe_metrics[base],"after":probe_metrics[changed],"probe_full_nll_delta":probe_metrics[changed].mean_nll-probe_metrics[base].mean_nll,"probe_full_correct_delta":probe_metrics[changed].correct as i64-probe_metrics[base].correct as i64,"gate":benefit(&probe_metrics[base],&probe_metrics[changed])}));
        let decoder_only = benefit(&probe_metrics[0], &probe_metrics[2]);
        let state_increment = benefit(&probe_metrics[2], &probe_metrics[3]);
        let method_gate = decoder_only && state_increment;
        write(
            &out,
            "probe.json",
            &json!({"all_four_heads_fixed_before_probe_evaluation":true,"fixed_head_replay_rows":prior_probe_rows,"fixed_head_replay_outputs":36,"prior_generations":prior_probe_outputs,"contrasts":contrasts,"method_gate":method_gate,"scope":"Previously opened authored recombination development panel; no independent fresh holdout claim."}),
        )?;
        write(&out, "runtime-census.json", &census)?;
        for (model, bytes) in priors.iter().take(4).zip(&prior_bytes) {
            if model.to_bytes()? != *bytes {
                return Err("prior artifact mutated".into());
            }
        }
        for (i, candidate) in candidates.iter().enumerate() {
            require_only_head_fields(&priors[STATE_FOR_CELL[i]], candidate)?;
        }
        for root in roots {
            crate::report_output::verify(root)?;
        }
        let accepted = method_gate && cell_results[3]["numeric_model_gate"] == true;
        write(
            &out,
            "result.json",
            &json!({"schema":"uor-r4.context-decoder-refit-result/1","decision":if accepted{"PASS_CONTEXT_DECODER_REFIT_DEVELOPMENT_GATE"}else{"FAIL_CONTEXT_DECODER_REFIT_DEVELOPMENT_GATE"},"baseline_branch_reproduction":true,"fit_calls":4,"state_proposals":0,"decoder_only_gate":decoder_only,"varied_state_increment_gate":state_increment,"method_gate":method_gate,"contrasts":contrasts,"cells":cell_results,"fit_reports":fit_reports,"prior_identities":ids,"prior_replayed_rows":prior_rows,"prior_replayed_original_outputs":replayed_prior_outputs,"prior_replayed_probe_rows":prior_probe_rows,"prior_replayed_probe_outputs":36,"elapsed_ms":start.elapsed().as_millis(),"promoted":false,"scope":"Four prescribed conditional decoder fits on frozen state snapshots and construction data. All seven prior states/artifacts replayed, with fixed states unchanged by head refits. Previously opened recombination probes and actual outputs compared after all four fits; no new state learning, regularization, JEPA, fresh broad holdout, normal changed artifact, promotion or language/energy qualification."}),
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
    let bytes = fs::read_dir(&out)?.try_fold(0_u64, |sum, e| -> std::io::Result<u64> {
        Ok(sum + e?.metadata()?.len())
    })?;
    if bytes > 64 * 1024 * 1024 {
        return Err("sealed attempt storage limit".into());
    }
    result
}
