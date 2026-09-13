//! Conditional incremental decoder selection using construction document folds.
use super::*;
use serde_json::{json, Value};
use sha2::Digest as _;
use std::{fs, io::Write, path::Path, time::Instant};
type RunResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const EPS: f64 = 1e-10;
const PARENT: &str = "blake3:0f82b7286bb1699b9ea3bf4193f152702f3b22e702cf31bcd0a0f56596bbf8b8";
const OLDER: &str = "blake3:ade9a1cfbf39f0828ec1ec2b53775c05fcbaaf53fa2189b6a37ab7a636ce5e58";
const PRIOR_TREE: &str = "blake3:1198ef4788397b9bf8cae20b8df4b23573bc9861eb812a51baf359f2545618b1";
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
    if start.elapsed().as_secs() >= 90 {
        return Err("complete emission experiment time guard".into());
    }
    Ok(())
}
// Whitelist only the emitter and its normal provenance. Every other artifact
// field, including future added source fields, must remain equal.
fn fixed_fields_equal(mut parent: Value, mut candidate: Value) -> RunResult<()> {
    for value in [&mut parent, &mut candidate] {
        let object = value.as_object_mut().ok_or("artifact object")?;
        for key in [
            "angular_tree",
            "schema",
            "implementation",
            "training_digest",
            "fit_config",
            "tied_fit_config",
            "training_parent",
        ] {
            object.remove(key);
        }
    }
    if parent != candidate {
        return Err("non-emission artifact field changed".into());
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
fn require_fixed_trace(comparison: &Value) -> RunResult<()> {
    if comparison["root_identity_hamming"] != 0 || comparison["source_identity_hamming"] != 0 {
        return Err("emitter-only intervention changed root/source trace".into());
    }
    Ok(())
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
fn family() -> [AngularTreeConfig; 4] {
    [(1, 4), (2, 4), (3, 16), (3, 4)].map(|(max_depth, min_leaf)| AngularTreeConfig {
        max_depth,
        min_leaf,
        max_seconds: 60,
    })
}
fn fold_indices(documents: usize, fold: usize) -> RunResult<(Vec<usize>, Vec<usize>)> {
    if documents != 12 || fold >= 3 {
        return Err("frozen fold bounds".into());
    }
    Ok((0..documents).partition(|index| index % 3 != fold))
}
fn data_digest(documents: &[Vec<u8>]) -> String {
    let mut hash = blake3::Hasher::new();
    hash.update(b"uor-r4.angular-emission-training/1");
    for doc in documents {
        hash.update(&(doc.len() as u64).to_le_bytes());
        hash.update(doc);
    }
    format!("blake3:{}", hash.finalize())
}
#[derive(Clone, Default, Serialize)]
struct Pool {
    total_nll: f64,
    positions: usize,
    correct: usize,
    total_nodes: usize,
}
impl Pool {
    fn add(&mut self, metrics: &Metrics, nodes: usize) -> RunResult<()> {
        if metrics.positions == 0
            || !metrics.mean_nll.is_finite()
            || metrics.mean_nll < 0.0
            || metrics.correct > metrics.positions
        {
            return Err("invalid fold metrics".into());
        }
        self.total_nll += metrics.mean_nll * metrics.positions as f64;
        self.positions += metrics.positions;
        self.correct += metrics.correct;
        self.total_nodes += nodes;
        Ok(())
    }
    fn mean(&self) -> RunResult<f64> {
        if self.positions == 0 || !self.total_nll.is_finite() {
            return Err("invalid pooled metrics".into());
        }
        Ok(self.total_nll / self.positions as f64)
    }
}
fn select(pools: &[Pool]) -> RunResult<usize> {
    if pools.len() != 4 {
        return Err("family count".into());
    }
    let means: Vec<f64> = pools.iter().map(Pool::mean).collect::<RunResult<_>>()?;
    let minimum = means.iter().copied().fold(f64::INFINITY, f64::min);
    (0..pools.len())
        .filter(|&i| means[i] <= minimum + EPS)
        .min_by_key(|&i| (pools[i].total_nodes, i))
        .ok_or_else(|| "no selected family".into())
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
#[test]
fn decoder_folds_and_pooled_selection_preserve_document_boundaries_and_weighting() -> RunResult<()>
{
    let mut seen = [0; 12];
    for fold in 0..3 {
        let (train, valid) = fold_indices(12, fold)?;
        assert_eq!(train.len(), 8);
        assert_eq!(valid.len(), 4);
        for i in 0..12 {
            assert_ne!(train.contains(&i), valid.contains(&i));
        }
        for i in valid {
            seen[i] += 1;
            assert_eq!(i % 3, fold);
        }
    }
    assert_eq!(seen, [1; 12]);
    assert!(fold_indices(11, 0).is_err());
    assert!(fold_indices(12, 3).is_err());
    let mut pools = vec![Pool::default(); 4];
    for (i, folds) in [
        [(1, 0.0), (9, 10.0)],
        [(1, 6.0), (9, 6.0)],
        [(1, 7.0), (9, 7.0)],
        [(1, 8.0), (9, 8.0)],
    ]
    .into_iter()
    .enumerate()
    {
        for (positions, mean_nll) in folds {
            pools[i].add(
                &Metrics {
                    positions,
                    correct: 0,
                    mean_nll,
                    work: Work::default(),
                },
                4,
            )?;
        }
    }
    assert_eq!(pools[0].mean()?, 9.0);
    assert_eq!(select(&pools)?, 1); // Unweighted fold means would choose family 0.
    let pools = vec![
        Pool {
            total_nll: 10.0,
            positions: 10,
            correct: 0,
            total_nodes: 20,
        },
        Pool {
            total_nll: 10.0 + 5e-10,
            positions: 10,
            correct: 0,
            total_nodes: 10,
        },
        Pool {
            total_nll: 10.0 + 15e-10,
            positions: 10,
            correct: 0,
            total_nodes: 1,
        },
        Pool {
            total_nll: 10.0 + 5e-10,
            positions: 10,
            correct: 0,
            total_nodes: 10,
        },
    ];
    assert_eq!(select(&pools)?, 1); // Family 2 is near 1 but outside EPS of the global minimum.
    assert!(select(&[]).is_err());
    Ok(())
}
#[test]
#[ignore = "requires sealed cap parents, prior tree, corpus and frozen decoder-validation design"]
fn decoder_validation_run_saved_experiment() -> RunResult<()> {
    let input = fs::canonicalize(std::env::var("UOR_DECODER_INPUT")?)?;
    let older_input = fs::canonicalize(std::env::var("UOR_DECODER_OLDER")?)?;
    let prior_input = fs::canonicalize(std::env::var("UOR_DECODER_PRIOR_TREE")?)?;
    let corpus = fs::canonicalize(std::env::var("UOR_DECODER_CORPUS")?)?;
    let path = fs::canonicalize(std::env::var("UOR_DECODER_DESIGN")?)?;
    let raw = std::path::PathBuf::from(std::env::var("UOR_DECODER_OUTPUT")?);
    let out = fs::canonicalize(raw.parent().ok_or("output parent")?)?
        .join(raw.file_name().ok_or("output name")?);
    for root in [&input, &older_input, &prior_input, &corpus] {
        if out.starts_with(root) {
            return Err("output beneath sealed input".into());
        }
    }
    crate::report_output::claim(&out)?;
    let result = (|| -> RunResult<()> {
        let start = Instant::now();
        for root in [&input, &older_input, &prior_input, &corpus] {
            crate::report_output::verify(root)?;
        }
        if fs::metadata(&path)?.len() > 65536 {
            return Err("design size".into());
        }
        let design_bytes = fs::read(&path)?;
        let design: Value = serde_json::from_slice(&design_bytes)?;
        let expected = json!({"schema":"uor-r4.decoder-validation-design/1","parent":PARENT,"older":OLDER,"prior_tree":PRIOR_TREE,"corpus_sha256":"541a094ae83a56adbcabb0cd157f0a7a14aef23b6db29f82f2d794975d420adf","family":family(),"folds":3,"fold_rule":"document_index modulo 3","selection":"Lowest pooled out-of-fold mean NLL; ties within 1e-10 prefer fewer total fitted nodes, then family order. Labels in each validation fold excluded from incremental topology, leaf-score and cap/tree fitting. Parent already saw all construction documents.","fold_fit_calls":12,"final_refit_calls_max":1,"reuse_prior_tree_when_selected_family_index":3,"experiment_seconds":90,"construction_documents":12,"construction_positions":672,"opened_positions":381,"generation_prompts":4,"generation_bytes":96,"gates":{"construction_loss_strictly_improves":true,"construction_accuracy_retained":true,"correct_to_wrong_each_prior_max":0,"opened_full_loss_nonincreasing":true,"opened_full_accuracy_retained":true},"promotion":false});
        if design != expected {
            return Err("frozen design mismatch".into());
        }
        fs::File::create_new(out.join("design.json"))?.write_all(&design_bytes)?;
        let parent_bytes = fs::read(input.join("candidate.json"))?;
        let older_bytes = fs::read(older_input.join("candidate.json"))?;
        let prior_bytes = fs::read(prior_input.join("candidate.json"))?;
        let parent = SharedCore::from_bytes(&parent_bytes)?;
        let older = SharedCore::from_bytes(&older_bytes)?;
        let prior = SharedCore::from_bytes(&prior_bytes)?;
        for (model, bytes, cid) in [
            (&parent, &parent_bytes, PARENT),
            (&older, &older_bytes, OLDER),
            (&prior, &prior_bytes, PRIOR_TREE),
        ] {
            if model.artifact_cid() != cid || model.to_bytes()? != *bytes {
                return Err("parent artifact identity".into());
            }
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
        let mut pools = vec![Pool::default(); 4];
        let mut cap_pools = vec![Pool::default(); 4];
        let mut fold_records = Vec::new();
        for (family_index, config) in family().into_iter().enumerate() {
            for fold in 0..3 {
                let (train_indices, validation_indices) = fold_indices(training.len(), fold)?;
                let train_docs: Vec<Vec<u8>> =
                    train_indices.iter().map(|&i| training[i].clone()).collect();
                let valid_docs: Vec<Vec<u8>> = validation_indices
                    .iter()
                    .map(|&i| training[i].clone())
                    .collect();
                guard(start)?;
                let (fitted, report) = parent.fit_angular_emission(&train_docs, config)?;
                let bytes = fitted.to_bytes()?;
                let model_name = format!("fold-{family_index}-{fold}-candidate.json");
                let fit_name = format!("fold-{family_index}-{fold}-fit.json");
                fs::File::create_new(out.join(&model_name))?.write_all(&bytes)?;
                write(&out, &fit_name, &report)?;
                guard(start)?;
                let loaded = SharedCore::from_bytes(&bytes)?;
                if loaded.to_bytes()? != bytes || loaded.artifact_cid() != fitted.artifact_cid() {
                    return Err("fold roundtrip".into());
                }
                fixed_fields_equal(
                    serde_json::from_slice(&parent_bytes)?,
                    serde_json::from_slice(&bytes)?,
                )?;
                if report.parent != PARENT
                    || report.data_digest != data_digest(&train_docs)
                    || serde_json::to_value(report.config)? != serde_json::to_value(config)?
                {
                    return Err("fold fit provenance".into());
                }
                let tree = loaded.artifact.angular_tree.as_ref().ok_or("fold tree")?;
                if tree.parent != PARENT
                    || tree.data_digest != report.data_digest
                    || serde_json::to_value(tree.config)? != serde_json::to_value(config)?
                {
                    return Err("fold artifact provenance".into());
                }
                let validation = loaded.evaluate(&valid_docs, Intervention::Full)?;
                let cap_validation = parent.evaluate(&valid_docs, Intervention::Full)?;
                pools[family_index].add(&validation, report.total_nodes)?;
                cap_pools[family_index].add(&cap_validation, 0)?;
                let receipt = json!({"family_index":family_index,"fold":fold,"config":config,"candidate":loaded.artifact_cid(),"candidate_file":model_name,"fit_file":fit_name,"train_document_indices":train_indices,"validation_document_indices":validation_indices,"train_data_digest":data_digest(&train_docs),"validation_data_digest":data_digest(&valid_docs),"train_positions":train_docs.iter().map(|d|d.len()+1).sum::<usize>(),"validation_positions":valid_docs.iter().map(|d|d.len()+1).sum::<usize>(),"training_metrics":report.after,"validation_metrics":validation,"fixed_cap_validation_metrics":cap_validation,"validation_labels_used_for_incremental_fit":false,"parent_previously_trained_on_all_documents":true});
                write(
                    &out,
                    &format!("fold-{family_index}-{fold}-inputs-metrics.json"),
                    &receipt,
                )?;
                fold_records.push(receipt);
                guard(start)?;
            }
        }
        if pools.iter().any(|p| p.positions != 672) || cap_pools.iter().any(|p| p.positions != 672)
        {
            return Err("pooled coverage mismatch".into());
        }
        let selected = select(&pools)?;
        let summaries:Vec<Value>=pools.iter().zip(&cap_pools).enumerate().map(|(i,(p,c))|Ok(json!({"family_index":i,"config":family()[i],"pool":p,"mean_nll":p.mean()?,"fixed_cap_mean_nll":c.mean()?,"incremental_validation_nll_delta":p.mean()?-c.mean()?}))).collect::<RunResult<_>>()?;
        write(
            &out,
            "selection.json",
            &json!({"selected_family_index":selected,"selected_config":family()[selected],"families":summaries,"folds":fold_records,"fold_fit_calls":12,"selection_scope":"Conditional incremental decoder selection: fixed parent already trained on every construction document. No independent model cross-validation or fresh holdout claim.","selection_uses_opened_or_disabled_labels":false}),
        )?;
        guard(start)?;
        let (candidate_bytes, final_report, reused, expected_cid) = if selected == 3 {
            let report: AngularTreeReport =
                serde_json::from_slice(&fs::read(prior_input.join("fit.json"))?)?;
            let tree = prior
                .artifact
                .angular_tree
                .as_ref()
                .ok_or("prior tree payload")?;
            if report.parent != PARENT
                || report.data_digest != data_digest(&training)
                || serde_json::to_value(report.config)? != serde_json::to_value(family()[3])?
                || tree.parent != PARENT
                || tree.data_digest != report.data_digest
                || serde_json::to_value(tree.config)? != serde_json::to_value(family()[3])?
            {
                return Err("sealed prior tree fit provenance".into());
            }
            (
                prior_bytes.clone(),
                report,
                true,
                prior.artifact_cid().to_owned(),
            )
        } else {
            let (fitted, report) = parent.fit_angular_emission(&training, family()[selected])?;
            (
                fitted.to_bytes()?,
                report,
                false,
                fitted.artifact_cid().to_owned(),
            )
        };
        fs::File::create_new(out.join("candidate.json"))?.write_all(&candidate_bytes)?;
        write(&out, "final-fit.json", &final_report)?;
        guard(start)?;
        let candidate = SharedCore::from_bytes(&candidate_bytes)?;
        if candidate.artifact_cid() != expected_cid
            || candidate.to_bytes()? != candidate_bytes
            || (reused && candidate.artifact_cid() != PRIOR_TREE)
            || candidate_bytes.len() > parent_bytes.len() + 1048576
        {
            return Err("candidate identity/storage".into());
        }
        fixed_fields_equal(
            serde_json::from_slice(&parent_bytes)?,
            serde_json::from_slice(&candidate_bytes)?,
        )?;
        if final_report.parent != PARENT
            || final_report.data_digest != data_digest(&training)
            || serde_json::to_value(final_report.config)?
                != serde_json::to_value(family()[selected])?
        {
            return Err("final fit provenance".into());
        }
        let final_tree = candidate
            .artifact
            .angular_tree
            .as_ref()
            .ok_or("final tree payload")?;
        if final_tree.parent != PARENT
            || final_tree.data_digest != final_report.data_digest
            || serde_json::to_value(final_tree.config)? != serde_json::to_value(family()[selected])?
        {
            return Err("final artifact provenance".into());
        }
        write(
            &out,
            "final-construction.json",
            &json!({"selected_family_index":selected,"candidate":candidate.artifact_cid(),"reused_exact_prior_tree":reused,"final_refit_calls":usize::from(!reused),"fit_calls_total":12+usize::from(!reused)}),
        )?;
        let early = generate(&candidate, &prompts, &[Intervention::Full])?;
        write(
            &out,
            "early-generation.json",
            &json!({"candidate":candidate.artifact_cid(),"outputs":early}),
        )?;
        guard(start)?;
        // Only after selection and final artifact construction do we load prior
        // opened/control responses and evaluate the six preservation panels.
        let cap_behavior: Value = serde_json::from_slice(&fs::read(input.join("behavior.json"))?)?;
        let tree_behavior: Value =
            serde_json::from_slice(&fs::read(prior_input.join("behavior.json"))?)?;
        let mut behavior = Vec::new();
        let mut lost = [0_u64; 3];
        let mut metrics = Vec::new();
        for (name, docs) in [
            ("complete_construction", &training),
            ("opened_development", &opened),
        ] {
            for control in CONTROLS {
                guard(start)?;
                let changed = rows(&candidate, docs, control)?;
                let after = verified_metrics(&candidate, docs, control, &changed)?;
                let mut comparisons = Vec::new();
                for (index, model) in [&parent, &older, &prior].into_iter().enumerate() {
                    let base = rows(model, docs, control)?;
                    let before = verified_metrics(model, docs, control, &base)?;
                    let control_value = serde_json::to_value(control)?;
                    let saved = if index < 2 {
                        let artifact = cap_behavior
                            .as_array()
                            .ok_or("saved caps")?
                            .iter()
                            .find(|v| v["artifact"] == model.artifact_cid())
                            .ok_or("saved cap artifact")?;
                        artifact["panels"]
                            .as_array()
                            .ok_or("saved cap panels")?
                            .iter()
                            .find(|v| v["panel"] == name && v["control"] == control_value)
                            .ok_or("saved cap panel")?
                            .clone()
                    } else {
                        let panel = tree_behavior["panels"]
                            .as_array()
                            .ok_or("saved tree panels")?
                            .iter()
                            .find(|v| v["panel"] == name && v["control"] == control_value)
                            .ok_or("saved tree panel")?;
                        json!({"metrics":panel["candidate_metrics"],"rows":panel["candidate_rows"]})
                    };
                    saved_rows_equal(&saved, &base, &before)?;
                    let comparison = compare(&base, &changed)?;
                    require_fixed_trace(&comparison)?;
                    lost[index] += comparison["correct_to_wrong"]
                        .as_u64()
                        .ok_or("lost count")?;
                    if control == Intervention::Full {
                        metrics.push(json!({"panel":name,"parent":model.artifact_cid(),"before":before,"after":after}));
                    }
                    comparisons.push(json!({"parent":model.artifact_cid(),"metrics":before,"rows":base,"comparison":comparison}));
                }
                behavior.push(json!({"panel":name,"control":control,"candidate_metrics":after,"candidate_rows":changed,"comparisons":comparisons}));
            }
        }
        let mut generations = Vec::new();
        for (index, model) in [&parent, &older, &prior, &candidate]
            .into_iter()
            .enumerate()
        {
            let outputs = generate(model, &prompts, &CONTROLS)?;
            if index < 3 {
                let saved = if index < 2 {
                    cap_behavior
                        .as_array()
                        .ok_or("saved caps")?
                        .iter()
                        .find(|v| v["artifact"] == model.artifact_cid())
                        .ok_or("saved cap artifact")?["generation"]
                        .clone()
                } else {
                    tree_behavior["generations"]
                        .as_array()
                        .ok_or("saved generations")?
                        .iter()
                        .find(|v| v["artifact"] == PRIOR_TREE)
                        .ok_or("saved tree generation")?["outputs"]
                        .clone()
                };
                if serde_json::to_value(&outputs)? != saved {
                    return Err("saved generation changed".into());
                }
            }
            generations.push(json!({"artifact":model.artifact_cid(),"outputs":outputs}));
        }
        for (i, first) in early.iter().enumerate() {
            if first != &generations[3]["outputs"][i * 3] {
                return Err("early generation replay".into());
            }
        }
        let construction = metrics
            .iter()
            .find(|m| m["panel"] == "complete_construction" && m["parent"] == PARENT)
            .ok_or("construction metrics")?;
        let before: Metrics = serde_json::from_value(construction["before"].clone())?;
        let after: Metrics = serde_json::from_value(construction["after"].clone())?;
        if (final_report.before.mean_nll - before.mean_nll).abs() > EPS
            || (final_report.after.mean_nll - after.mean_nll).abs() > EPS
            || final_report.before.correct != before.correct
            || final_report.after.correct != after.correct
        {
            return Err("final fit replay".into());
        }
        let optimization =
            after.mean_nll + EPS < before.mean_nll && after.correct >= before.correct;
        let preservation = lost == [0, 0, 0];
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
        write(
            &out,
            "behavior.json",
            &json!({"panels":behavior,"generations":generations}),
        )?;
        write(
            &out,
            "source.json",
            &json!({"implementation":SharedCore::implementation_digest(),"driver":digest(include_bytes!("decoder_validation.rs")),"tree_source":digest(include_bytes!("../angular_tree.rs")),"design":digest(&design_bytes),"corpus":digest(&corpus_bytes),"parent":PARENT,"older":OLDER,"prior_tree":PRIOR_TREE,"candidate":candidate.artifact_cid(),"candidate_sha256":format!("{:x}",sha2::Sha256::digest(&candidate_bytes))}),
        )?;
        for (model, bytes) in [
            (&parent, &parent_bytes),
            (&older, &older_bytes),
            (&prior, &prior_bytes),
        ] {
            if model.to_bytes()? != *bytes {
                return Err("preserved parent mutated".into());
            }
        }
        for root in [&input, &older_input, &prior_input, &corpus] {
            crate::report_output::verify(root)?;
        }
        guard(start)?;
        write(
            &out,
            "result.json",
            &json!({"schema":"uor-r4.decoder-validation-result/1","decision":if optimization&&preservation&&development{"PASS_DECODER_VALIDATION_DEVELOPMENT_GATE"}else{"FAIL_DECODER_VALIDATION_DEVELOPMENT_GATE"},"candidate":candidate.artifact_cid(),"parent":PARENT,"older":OLDER,"prior_tree":PRIOR_TREE,"selected_family_index":selected,"selected_config":family()[selected],"reused_exact_prior_tree":reused,"fold_fit_calls":12,"final_refit_calls":usize::from(!reused),"fit_calls_total":12+usize::from(!reused),"conditional_optimization_gate":optimization,"preservation_gate":preservation,"opened_development_gate":development,"correct_to_wrong_each_prior":lost,"metrics":metrics,"before":before,"after":after,"fit":final_report,"selected_pooled_validation_nll":pools[selected].mean()?,"fixed_cap_pooled_validation_nll":cap_pools[selected].mean()?,"pooled_validation_improves_fixed_cap":pools[selected].mean()?+EPS<cap_pools[selected].mean()?,"elapsed_ms":start.elapsed().as_millis(),"candidate_bytes":candidate_bytes.len(),"fixed_root_source_traces":true,"promoted":false,"scope":"Conditional incremental decoder selection on construction-document folds; fixed parent already saw all documents. No independent model cross-validation, fresh holdout, recurrence fit, promotion, language or energy qualification."}),
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
    result
}
