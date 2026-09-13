//! Bounded one-coordinate state profiles with a refitted conditional decoder.
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
const INDICES: [usize; 2] = [1176, 1930];
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
    fs::File::create_new(out.join(name))?.write_all(&serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn guard(start: Instant) -> RunResult<()> {
    if start.elapsed().as_secs() >= 120 {
        return Err("complete coupled-profile experiment time guard".into());
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
    digest(include_bytes!("coupled_profile.rs"))
}
fn config() -> AngularTreeConfig {
    AngularTreeConfig {
        max_depth: 3,
        min_leaf: 16,
        max_seconds: 60,
    }
}
fn settings() -> Vec<[u16; 2]> {
    std::iter::once([119, 119])
        .chain((0..119).map(|r| [r, 119]))
        .chain((0..119).map(|r| [119, r]))
        .collect()
}
fn select(costs: &[f64], pairs: &[[u16; 2]]) -> RunResult<usize> {
    if costs.is_empty()
        || costs.len() != pairs.len()
        || costs.iter().any(|v| !v.is_finite() || *v < 0.0)
    {
        return Err("profile selection input".into());
    }
    let minimum = costs.iter().copied().fold(f64::INFINITY, f64::min);
    if pairs[0] != [119, 119] {
        return Err("baseline setting order".into());
    }
    if costs[0] <= minimum + EPS {
        return Ok(0);
    }
    (0..costs.len())
        .filter(|&i| costs[i] <= minimum + EPS)
        .min_by_key(|&i| pairs[i])
        .ok_or_else(|| "profile minimum".into())
}
fn training_digest(documents: &[Vec<u8>]) -> String {
    let mut hash = blake3::Hasher::new();
    hash.update(b"uor-r4.angular-emission-training/1");
    for doc in documents {
        hash.update(&(doc.len() as u64).to_le_bytes());
        hash.update(doc);
    }
    format!("blake3:{}", hash.finalize())
}
fn ephemeral_cap(cap: &SharedCore, pair: [u16; 2]) -> RunResult<SharedCore> {
    if cap.artifact.angular_tree.is_some()
        || cap.artifact.calibrated.is_none()
        || pair.iter().any(|&r| r >= 120)
    {
        return Err("ephemeral cap input".into());
    }
    let mut changed = cap.clone();
    for (i, r) in INDICES.into_iter().zip(pair) {
        changed.artifact.parameters[i] = r;
    }
    // This digest identifies numerical content, not a qualified source model.
    changed.refresh_identity()?;
    Ok(changed)
}
fn require_only_profile_fields(cap: &SharedCore, changed: &SharedCore) -> RunResult<()> {
    let mut actual = serde_json::to_value(&changed.artifact)?;
    let mut expected = serde_json::to_value(&cap.artifact)?;
    for i in INDICES {
        actual["parameters"][i] = expected["parameters"][i].clone();
    }
    for value in [&mut actual, &mut expected] {
        let object = value.as_object_mut().ok_or("artifact fields")?;
        for field in [
            "angular_tree",
            "schema",
            "implementation",
            "training_digest",
            "training_parent",
            "fit_config",
            "tied_fit_config",
        ] {
            object.remove(field);
        }
    }
    if actual != expected {
        return Err("profile changed fields outside pair/emission/provenance".into());
    }
    Ok(())
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Witness {
    schema: String,
    cap: String,
    selected_parent: String,
    implementation: String,
    driver: String,
    design: String,
    corpus_design: String,
    indices: [usize; 2],
    selected_roots: [u16; 2],
    ephemeral_cap_content_digest: String,
    head_digest: String,
    head: AngularEmission,
    settings: usize,
    parent_nll: f64,
    selected_nll: f64,
}
impl Witness {
    fn replay(
        &self,
        cap: &SharedCore,
        parent: &SharedCore,
        design: &str,
        corpus: &str,
        data: &str,
    ) -> RunResult<SharedCore> {
        if self.schema != "uor-r4.coupled-profile-witness/1"
            || self.cap != cap.artifact_cid()
            || self.selected_parent != parent.artifact_cid()
            || self.implementation != SharedCore::implementation_digest()
            || self.driver != driver_digest()
            || self.design != design
            || self.corpus_design != corpus
            || self.indices != INDICES
            || self.settings != 239
            || !settings().contains(&self.selected_roots)
            || !self.parent_nll.is_finite()
            || !self.selected_nll.is_finite()
            || self.parent_nll < 0.0
            || self.selected_nll < 0.0
        {
            return Err("invalid coupled witness bindings".into());
        }
        let mut model = ephemeral_cap(cap, self.selected_roots)?;
        self.head.validate()?;
        if model.artifact_cid() != self.ephemeral_cap_content_digest
            || self.head.parent != self.ephemeral_cap_content_digest
            || self.head.data_digest != data
            || serde_json::to_value(self.head.config)? != serde_json::to_value(config())?
            || self.head_digest != digest(&serde_json::to_vec(&self.head)?)
        {
            return Err("witness head or ephemeral lineage mismatch".into());
        }
        model.artifact.schema = ANGULAR_TREE_SCHEMA.into();
        model.artifact.implementation = SharedCore::implementation_digest();
        model.artifact.training_digest = None;
        model.artifact.training_parent = None;
        model.artifact.fit_config = None;
        model.artifact.tied_fit_config = None;
        model.artifact.angular_tree = Some(self.head.clone());
        require_only_profile_fields(cap, &model)?;
        // Never expose this in-memory intervention's inherited identity or save
        // it as a standard artifact. The exported witness owns its identity.
        Ok(model)
    }
}
fn complete_file_set(out: &Path, table: &[Value]) -> RunResult<()> {
    let mut expected: std::collections::BTreeSet<String> = [
        "attempt.json",
        "design.json",
        "baseline-reproduction.json",
        "profile.json",
        "witness.json",
        "early-generation.json",
        "runtime-census.json",
        "behavior.json",
        "source.json",
        "result.json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    if table.len() != 239 {
        return Err("incomplete setting receipts".into());
    }
    for (index, row) in table.iter().enumerate() {
        let head_name = format!("setting-{index:03}-head.json");
        let fit_name = format!("setting-{index:03}-fit.json");
        expected.insert(head_name.clone());
        expected.insert(fit_name.clone());
        let payload = fs::read(out.join(&head_name))?;
        if payload.len() > 131072
            || row["head_file"] != head_name
            || row["fit_file"] != fit_name
            || row["head_digest"] != digest(&payload)
        {
            return Err("retained profile head set mismatch".into());
        }
        let receipt: Value = serde_json::from_slice(&fs::read(out.join(&fit_name))?)?;
        if receipt["setting_index"] != index
            || receipt["roots"] != row["roots"]
            || receipt["head_digest"] != row["head_digest"]
            || receipt["head_sha256"] != format!("{:x}", sha2::Sha256::digest(&payload))
        {
            return Err("retained profile receipt mismatch".into());
        }
    }
    let actual: std::collections::BTreeSet<String> = fs::read_dir(out)?
        .map(|entry| entry.map(|e| e.file_name().to_string_lossy().into_owned()))
        .collect::<std::io::Result<_>>()?;
    if actual != expected {
        return Err("incomplete or unexpected attempt file set".into());
    }
    Ok(())
}
fn attempt_size(out: &Path) -> RunResult<u64> {
    let mut total = 0_u64;
    for entry in fs::read_dir(out)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            return Err("unexpected attempt member".into());
        }
        total = total
            .checked_add(entry.metadata()?.len())
            .ok_or("attempt size overflow")?;
    }
    if total > 64 * 1024 * 1024 {
        return Err("coupled attempt storage bound".into());
    }
    Ok(total)
}
#[test]
fn coupled_profile_coverage_witness_lineage_and_global_ties() -> RunResult<()> {
    let pairs = settings();
    assert_eq!(pairs.len(), 239);
    assert_eq!(
        pairs
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        239
    );
    assert_eq!(pairs[0], [119, 119]);
    for r in 0..120 {
        assert!(pairs.contains(&[r, 119]));
        assert!(pairs.contains(&[119, r]));
    }
    assert!(pairs.iter().all(|p| p[0] == 119 || p[1] == 119));
    assert_eq!(select(&[1.0, 1.0], &[[119, 119], [0, 119]])?, 0);
    assert_eq!(
        select(
            &[2.0, 1.0 + 0.5 * EPS, 1.0],
            &[[119, 119], [0, 119], [119, 0]]
        )?,
        1
    );
    assert_eq!(
        select(
            &[2.0, 1.0 + 1.5 * EPS, 1.0],
            &[[119, 119], [0, 119], [119, 0]]
        )?,
        2
    );
    let cap = SharedCore::initialized(745)?.upgrade_calibrated()?;
    let parent = cap.clone();
    let ephemeral = ephemeral_cap(&cap, [0, 119])?;
    let mut branches = vec![None; 512];
    branches[0] = Some(vec![AngularNode::Leaf { score: 3 }]);
    let data = format!("blake3:{}", "a".repeat(64));
    let head = AngularEmission {
        config: config(),
        parent: ephemeral.artifact_cid().into(),
        data_digest: data.clone(),
        branches,
    };
    let witness = Witness {
        schema: "uor-r4.coupled-profile-witness/1".into(),
        cap: cap.artifact_cid().into(),
        selected_parent: parent.artifact_cid().into(),
        implementation: SharedCore::implementation_digest(),
        driver: driver_digest(),
        design: "design".into(),
        corpus_design: "corpus".into(),
        indices: INDICES,
        selected_roots: [0, 119],
        ephemeral_cap_content_digest: ephemeral.artifact_cid().into(),
        head_digest: digest(&serde_json::to_vec(&head)?),
        head,
        settings: 239,
        parent_nll: 2.0,
        selected_nll: 1.0,
    };
    let bytes = cap.to_bytes()?;
    let loaded: Witness = serde_json::from_slice(&serde_json::to_vec(&witness)?)?;
    let replay = loaded.replay(&cap, &parent, "design", "corpus", &data)?;
    require_only_profile_fields(&cap, &replay)?;
    assert_eq!(replay.artifact.parameters[1176], 0);
    assert_eq!(replay.artifact.parameters[1930], 119);
    assert_eq!(cap.to_bytes()?, bytes);
    assert_eq!(replay.session(Intervention::Full).branch_score(0, 0), 3);
    for field in [
        "cap",
        "selected_parent",
        "implementation",
        "driver",
        "design",
        "corpus_design",
        "ephemeral_cap_content_digest",
        "head_digest",
    ] {
        let mut bad = serde_json::to_value(&witness)?;
        bad[field] = json!("changed");
        let bad: Witness = serde_json::from_value(bad)?;
        assert!(bad
            .replay(&cap, &parent, "design", "corpus", &data)
            .is_err());
    }
    let mut bad = witness.clone();
    bad.head.parent = cap.artifact_cid().into();
    assert!(bad
        .replay(&cap, &parent, "design", "corpus", &data)
        .is_err());
    let mut bad = witness.clone();
    bad.head.branches[0] = Some(vec![AngularNode::Leaf { score: 11 }]);
    bad.head_digest = digest(&serde_json::to_vec(&bad.head)?);
    assert!(bad
        .replay(&cap, &parent, "design", "corpus", &data)
        .is_err());
    let mut bad = replay.clone();
    bad.artifact.parameters[0] = (bad.artifact.parameters[0] + 1) % 120;
    assert!(require_only_profile_fields(&cap, &bad).is_err());
    let mut bad = replay;
    bad.artifact.calibrated = None;
    assert!(require_only_profile_fields(&cap, &bad).is_err());
    Ok(())
}
#[test]
#[ignore = "requires sealed decoder/priors/corpus and frozen coupled-profile design"]
fn coupled_profile_run_saved_experiment() -> RunResult<()> {
    let input = fs::canonicalize(std::env::var("UOR_COUPLED_INPUT")?)?;
    let evidence = input
        .parent()
        .and_then(Path::parent)
        .ok_or("evidence parent")?;
    let prior_input = fs::canonicalize(evidence.join("angular-tree-1/attempt-1"))?;
    let cap_input = fs::canonicalize(evidence.join("final-emission-1/attempt-1"))?;
    let older_input = fs::canonicalize(evidence.join("loss-frontier-1/joint-1"))?;
    let corpus = fs::canonicalize(std::env::var("UOR_COUPLED_CORPUS")?)?;
    let path = fs::canonicalize(std::env::var("UOR_COUPLED_DESIGN")?)?;
    let raw = std::path::PathBuf::from(std::env::var("UOR_COUPLED_OUTPUT")?);
    let out = fs::canonicalize(raw.parent().ok_or("output parent")?)?
        .join(raw.file_name().ok_or("output name")?);
    let roots = [&input, &prior_input, &cap_input, &older_input, &corpus];
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
        let expected = json!({"schema":"uor-r4.coupled-profile-design/1","parent":"blake3:8f8e5c342c83b062274744033a96668e9c079dc47a9c4d1b138baa7ea4889b6a","prior_tree":"blake3:1198ef4788397b9bf8cae20b8df4b23573bc9861eb812a51baf359f2545618b1","cap":"blake3:0f82b7286bb1699b9ea3bf4193f152702f3b22e702cf31bcd0a0f56596bbf8b8","older":"blake3:ade9a1cfbf39f0828ec1ec2b53775c05fcbaaf53fa2189b6a37ab7a636ce5e58","corpus_sha256":"541a094ae83a56adbcabb0cd157f0a7a14aef23b6db29f82f2d794975d420adf","indices":[1176,1930],"parent_roots":[119,119],"root_choices":120,"profile_settings":239,"fit_calls_max":239,"profile_rule":"Unchanged parent first, then roots0..118 at1176 with1930=119, then roots0..118 at1930 with1176=119. Each setting starts from the same cap parent.","config":{"max_depth":3,"min_leaf":16,"max_seconds":60},"selection":"Lowest complete causal construction mean NLL after conditional emitter fit; ties within 1e-10 of global minimum prefer unchanged parent, then lexicographic roots. Opened and disabled-control labels excluded.","baseline_exact_reproduction_required":true,"head_payload_bytes_max":131072,"attempt_bytes_max":67108864,"experiment_seconds":120,"construction_documents":12,"construction_positions":672,"opened_positions":381,"generation_prompts":4,"generation_bytes":96,"witness":"Source-bound test-only state-and-head witness; ephemeral cap content identities are not standard learned artifact provenance. No standard changed-model artifact exported.","gates":{"construction_loss_strictly_improves":true,"construction_accuracy_retained":true,"correct_to_wrong_each_prior_max":0,"opened_full_loss_nonincreasing":true,"opened_full_accuracy_retained":true},"promotion":false});
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
        let cap = &priors[2];
        let parent_roots = INDICES.map(|i| parent.artifact.parameters[i]);
        if parent_roots != [119, 119] || INDICES.map(|i| cap.artifact.parameters[i]) != [119, 119] {
            return Err("frozen profile baseline roots".into());
        }
        let pairs = settings();
        let mut costs = Vec::with_capacity(239);
        let mut correct_counts = Vec::with_capacity(239);
        let mut table = Vec::with_capacity(239);
        let data = training_digest(&training);
        let design_digest = digest(&design_bytes);
        let corpus_digest = digest(&corpus_bytes);
        for (index, &pair) in pairs.iter().enumerate() {
            guard(start)?;
            let temporary = ephemeral_cap(cap, pair)?;
            let ephemeral_digest = temporary.artifact_cid().to_string();
            // Each fit sees complete fresh causal construction trajectories for
            // this numerical intervention, never the previous setting's head.
            let (fitted, report) = temporary.fit_angular_emission(&training, config())?;
            require_only_profile_fields(cap, &fitted)?;
            if index == 0 {
                if fitted.to_bytes()? != prior_bytes[0] || fitted.artifact_cid() != PARENT {
                    return Err(
                        "baseline fit failed exact saved-parent reproduction; stop profile".into(),
                    );
                }
                write(
                    &out,
                    "baseline-reproduction.json",
                    &json!({"setting_index":0,"roots":pair,"artifact":PARENT,"same_bytes":true,"same_cid":true,"fit_calls":1,"required_before_alternatives":true}),
                )?;
            }
            let head = fitted
                .artifact
                .angular_tree
                .as_ref()
                .ok_or("fitted angular head")?;
            head.validate()?;
            if head.parent != ephemeral_digest
                || head.data_digest != data
                || serde_json::to_value(head.config)? != serde_json::to_value(config())?
                || report.parent != ephemeral_digest
                || report.data_digest != data
                || report.after.positions != 672
                || !report.after.mean_nll.is_finite()
            {
                return Err("fit lineage or metrics".into());
            }
            let payload = serde_json::to_vec(head)?;
            if payload.len() > 131072 {
                return Err("head payload size bound".into());
            }
            let head_name = format!("setting-{index:03}-head.json");
            let fit_name = format!("setting-{index:03}-fit.json");
            let head_hash = digest(&payload);
            fs::File::create_new(out.join(&head_name))?.write_all(&payload)?;
            write(
                &out,
                &fit_name,
                &json!({"schema":"uor-r4.coupled-profile-setting/1","setting_index":index,"roots":pair,"cap_artifact":CAP,"ephemeral_cap_content_digest":ephemeral_digest,"fit_parent_identity_scope":"Test-only numerical cap content identity; not a standard learned artifact or qualified source-model provenance.","head_file":head_name,"head_digest":head_hash,"head_bytes":payload.len(),"head_sha256":format!("{:x}",sha2::Sha256::digest(&payload)),"fit":report,"standard_changed_artifact_exported":false}),
            )?;
            costs.push(report.after.mean_nll);
            correct_counts.push(report.after.correct);
            table.push(json!({"setting_index":index,"roots":pair,"head_file":head_name,"fit_file":fit_name,"head_digest":head_hash,"head_bytes":payload.len(),"ephemeral_cap_content_digest":ephemeral_digest,"construction_metrics":report.after,"fit_elapsed_ms":report.elapsed_ms,"selected_trees":report.selected_trees,"total_nodes":report.total_nodes}));
            // All head payloads are retained on disk. No discarded candidate is
            // ever refitted to resolve the final global-minimum tie set.
            guard(start)?;
            attempt_size(&out)?;
        }
        if table.len() != 239 {
            return Err("profile fit count".into());
        }
        let selected = select(&costs, &pairs)?;
        let selected_roots = pairs[selected];
        let parent_index = 0;
        let minimum = costs.iter().copied().fold(f64::INFINITY, f64::min);
        let improving = costs.iter().filter(|&&v| v + EPS < costs[0]).count();
        write(
            &out,
            "profile.json",
            &json!({"settings":table,"fit_calls":239,"selected_setting_index":selected,"selected_roots":selected_roots,"parent_roots":parent_roots,"minimum_nll":minimum,"selected_nll":costs[selected],"parent_nll":costs[0],"strictly_improving_settings":improving,"settings_within_minimum_tolerance":costs.iter().filter(|&&v|v<=minimum+EPS).count(),"costs_in_profile_order":costs,"correct_counts_in_profile_order":correct_counts,"opened_and_disabled_labels_used":false,"all_heads_preserved":true}),
        )?;
        let head_file = table[selected]["head_file"]
            .as_str()
            .ok_or("selected head file")?;
        let payload = fs::read(out.join(head_file))?;
        if payload.len() > 131072
            || digest(&payload)
                != table[selected]["head_digest"]
                    .as_str()
                    .ok_or("selected head digest")?
        {
            return Err("selected head bytes changed".into());
        }
        let head: AngularEmission = serde_json::from_slice(&payload)?;
        let receipt: Value = serde_json::from_slice(&fs::read(
            out.join(
                table[selected]["fit_file"]
                    .as_str()
                    .ok_or("selected fit file")?,
            ),
        )?)?;
        if receipt["setting_index"] != selected
            || receipt["roots"] != json!(selected_roots)
            || receipt["head_digest"] != digest(&payload)
            || receipt["head_sha256"] != format!("{:x}", sha2::Sha256::digest(&payload))
            || receipt["ephemeral_cap_content_digest"] != head.parent
            || receipt["fit"]["data_digest"] != data
        {
            return Err("selected receipt binding".into());
        }
        let witness = Witness {
            schema: "uor-r4.coupled-profile-witness/1".into(),
            cap: CAP.into(),
            selected_parent: PARENT.into(),
            implementation: SharedCore::implementation_digest(),
            driver: driver_digest(),
            design: design_digest.clone(),
            corpus_design: corpus_digest.clone(),
            indices: INDICES,
            selected_roots,
            ephemeral_cap_content_digest: head.parent.clone(),
            head_digest: digest(&payload),
            head,
            settings: 239,
            parent_nll: costs[0],
            selected_nll: costs[selected],
        };
        let witness_bytes = serde_json::to_vec_pretty(&witness)?;
        let witness_id = digest(&witness_bytes);
        fs::File::create_new(out.join("witness.json"))?.write_all(&witness_bytes)?;
        let replayed: Witness = serde_json::from_slice(&witness_bytes)?;
        let candidate = replayed.replay(cap, parent, &design_digest, &corpus_digest, &data)?;
        let early = generate(&candidate, &prompts, &[Intervention::Full])?;
        write(
            &out,
            "early-generation.json",
            &json!({"witness":witness_id,"outputs":early}),
        )?;
        guard(start)?;
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
            return Err("profile/witness metric replay".into());
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
            &json!({"implementation":SharedCore::implementation_digest(),"driver":driver_digest(),"tree_source":digest(include_bytes!("../angular_tree.rs")),"design":design_digest,"corpus":corpus_digest,"parent":PARENT,"prior_tree":PRIOR_TREE,"cap":CAP,"older":OLDER,"witness":witness_id,"prior_artifact_sha256":prior_bytes.iter().map(|b|format!("{:x}",sha2::Sha256::digest(b))).collect::<Vec<_>>(),"prior_report_roots":[input,prior_input,cap_input,older_input],"standard_changed_model_artifact_exported":false}),
        )?;
        for (model, bytes) in priors.iter().zip(&prior_bytes) {
            if model.to_bytes()? != *bytes {
                return Err("prior mutated".into());
            }
        }
        require_only_profile_fields(cap, &candidate)?;
        let total_attempt_bytes_before_result = attempt_size(&out)?;
        for root in roots {
            crate::report_output::verify(root)?;
        }
        guard(start)?;
        write(
            &out,
            "result.json",
            &json!({"schema":"uor-r4.coupled-profile-result/1","decision":if optimization&&preservation&&development{"PASS_COUPLED_PROFILE_DEVELOPMENT_GATE"}else{"FAIL_COUPLED_PROFILE_DEVELOPMENT_GATE"},"witness":witness_id,"parent":PARENT,"prior_tree":PRIOR_TREE,"cap":CAP,"older":OLDER,"selected_setting_index":selected,"selected_roots":selected_roots,"parent_roots":parent_roots,"profile_settings":239,"fit_calls":239,"selected_head_refit_calls":0,"baseline_exact_reproduction":true,"strictly_improving_settings":improving,"minimum_nll":minimum,"parent_is_selected":selected==0,"conditional_optimization_gate":optimization,"preservation_gate":preservation,"opened_development_gate":development,"correct_to_wrong_each_prior":lost,"wrong_to_correct_each_prior":gained,"root_identity_hamming_each_prior":root_hamming,"source_identity_hamming_each_prior":source_hamming,"prediction_hamming_each_prior":prediction_hamming,"metrics":metrics,"before":before,"after":after,"elapsed_ms":start.elapsed().as_millis(),"only_pair_and_head_mutable":true,"changed_parameter_count":parent.artifact.parameters.iter().zip(&candidate.artifact.parameters).filter(|(a,b)|a!=b).count(),"all_head_payloads_preserved":true,"attempt_bytes_before_result":total_attempt_bytes_before_result,"ephemeral_cap_content_digest":replayed.ephemeral_cap_content_digest,"ephemeral_identity_scope":"Test-only numerical content; not standard learned artifact provenance.","promoted":false,"scope":"Bounded single-coordinate state profiles with construction-only conditional emitter fits. No exhaustive joint state/head optimum, standard changed artifact, fresh holdout, promotion, language or energy qualification."}),
        )?;
        complete_file_set(&out, &table)?;
        attempt_size(&out)?;
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
    attempt_size(&out)?;
    result
}
