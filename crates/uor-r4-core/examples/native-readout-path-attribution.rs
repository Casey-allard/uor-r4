//! Saved-only replay of the recorded two-family proposal path. No encoder,
//! gradient, new proposal, artifact export, or serving-policy intervention.
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::{
    native_geometric::learner::{
        geometric_continuation_field::{ContinuationReadCounts, NativeContinuationField},
        geometric_generate::{GenerateReadCounts, NativeGeometricGenerate},
        geometric_read_state_bridge::{BridgeReadCounts, NativeGeometricReadStateBridge},
    },
    report_output,
};
use uor_r4_integer::{
    geometric_source_realizer::{NativeArtifactBinding, NativeSourceRealizer},
    geometric_vocabulary_actions::NativeVocabularyActions,
    h4_tables::H4Code,
};
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const FACTS: [usize; 5] = [97, 156, 151, 245, 392];
const NAMES: [&str; 2] = ["cue.coefficients", "prefix.coefficients"];
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    readout_root: PathBuf,
    expected_readout_report_sha256: String,
    expected_readout_manifest_sha256: String,
    reached_u_root: PathBuf,
    expected_reached_u_report_sha256: String,
    expected_reached_u_manifest_sha256: String,
    inputs_seal_root: PathBuf,
    expected_inputs_manifest_sha256: String,
    inputs: PathBuf,
    expected_inputs_sha256: String,
    labels: PathBuf,
    expected_labels_sha256: String,
    expected_source_metadata_sha256: String,
    expected_generate_sha256: String,
    expected_continuation_sha256: String,
    output: PathBuf,
    maximum_cache_bytes: u64,
    maximum_report_bytes: u64,
}
fn bad(s: &str) -> Box<dyn std::error::Error> {
    std::io::Error::other(s).into()
}
fn need(b: bool, s: &str) -> Result<()> {
    if b {
        Ok(())
    } else {
        Err(bad(s))
    }
}
fn hash(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn bytes(p: &Path) -> Result<Vec<u8>> {
    Ok(fs::read(p)?)
}
fn read(p: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&bytes(p)?)?)
}
fn jhash(v: &Value) -> Result<String> {
    Ok(hash(&serde_json::to_vec(v)?))
}
fn arr<T: serde::de::DeserializeOwned>(v: &Value) -> Result<T> {
    Ok(serde_json::from_value(v.clone())?)
}
fn integer(v: &Value) -> Result<usize> {
    Ok(v.as_u64()
        .ok_or_else(|| bad("integer witness absent"))?
        .try_into()?)
}
fn state(v: &Value) -> Result<Vec<H4Code>> {
    let a: Vec<u8> = arr(v)?;
    need(a.len() == 8, "state requires eight lanes")?;
    a.into_iter()
        .map(|x| H4Code::try_from(x).map_err(Into::into))
        .collect()
}
fn write(c: &Config, name: &str, v: &Value, written: &mut u64) -> Result<()> {
    let b = serde_json::to_vec(v)?;
    *written = written
        .checked_add(b.len() as u64)
        .ok_or_else(|| bad("report size overflow"))?;
    need(
        *written <= c.maximum_report_bytes,
        "report byte cap exceeded",
    )?;
    fs::write(c.output.join(name), b)?;
    Ok(())
}
fn nib(b: &[u8], i: usize) -> Result<i8> {
    let z = *b
        .get(i / 2)
        .ok_or_else(|| bad("packed coordinate absent"))?;
    let q = (if i % 2 == 0 { z & 15 } else { z >> 4 }) as i8;
    let q = if q >= 8 { q - 16 } else { q };
    need((-7..=7).contains(&q), "illegal Q4 code")?;
    Ok(q)
}
fn codes(cp: &Path) -> Result<[Vec<i8>; 2]> {
    let mut out = [Vec::new(), Vec::new()];
    for (f, path) in ["cue/cue-q4.bin", "prefix/prefix-q4.bin"]
        .iter()
        .enumerate()
    {
        let b = bytes(&cp.join(path))?;
        need(b.len() == 480, "960 packed table shape")?;
        out[f] = (0..960).map(|i| nib(&b, i)).collect::<Result<_>>()?;
    }
    Ok(out)
}
fn earliest(v: &[i64]) -> Result<usize> {
    let mut best = 0;
    need(!v.is_empty(), "physical Copy absent")?;
    for i in 1..v.len() {
        if v[i] > v[best] {
            best = i;
        }
    }
    Ok(best)
}
fn stage(base: &[i64], keys: &[Option<usize>], coordinate: usize, delta: i8) -> Result<Vec<i64>> {
    need(keys.len() == base.len() * 8, "physical key shape")?;
    base.iter()
        .enumerate()
        .map(|(j, x)| {
            let n = keys[j * 8..j * 8 + 8]
                .iter()
                .filter(|k| **k == Some(coordinate))
                .count() as i64;
            x.checked_add(n * i64::from(delta) * (1 << 22))
                .ok_or_else(|| bad("staged Copy overflow"))
        })
        .collect()
}
fn settle(current: &mut Vec<i64>, staged: Vec<i64>, accepted: bool) {
    if accepted {
        *current = staged;
    }
}
struct Frame {
    ids: Vec<u32>,
    base: Vec<i64>,
    u: Vec<i64>,
    keys: [Vec<Option<usize>>; 2],
    query: Vec<H4Code>,
    sources: Vec<Vec<H4Code>>,
    target: usize,
}
fn prepare(
    row: &Value,
    model: &NativeGeometricGenerate,
    field: &NativeContinuationField,
) -> Result<Frame> {
    let ids: Vec<u32> = arr(&row["copy_ids"])?;
    let copy: Vec<i64> = arr(&row["copy_q24"])?;
    need(
        ids.len() == copy.len() && !ids.is_empty() && ids.iter().all(|t| *t < 4096),
        "physical Copy IDs/scores shape",
    )?;
    let bank = &row["bank_trace"]["cue_bank"]["bank"];
    let candidates = bank["candidates"]
        .as_array()
        .ok_or_else(|| bad("candidate witness absent"))?;
    need(candidates.len() == ids.len(), "physical provenance shape")?;
    let contexts = bank["context"]["states"]
        .as_array()
        .ok_or_else(|| bad("context state witnesses absent"))?;
    let query = state(
        contexts
            .last()
            .ok_or_else(|| bad("bank final state absent"))?,
    )?;
    let mut sources = Vec::new();
    for (j, c) in candidates.iter().enumerate() {
        need(
            c["occurrence"]["token_id"] == ids[j],
            "physical occurrence token differs",
        )?;
        sources.push(state(
            contexts
                .get(integer(&c["context_position"])?)
                .ok_or_else(|| bad("candidate context position absent"))?,
        )?);
    }
    let mut u = vec![0; 4096];
    field.score_delta_into(
        &state(&row["continuation"]["state_codes"])?,
        model,
        &mut u,
        &mut ContinuationReadCounts::default(),
    )?;
    let base = copy
        .iter()
        .zip(&ids)
        .map(|(c, t)| {
            c.checked_sub(u[*t as usize])
                .ok_or_else(|| bad("BASECopy overflow"))
        })
        .collect::<Result<Vec<_>>>()?;
    let heads = bank["heads"]
        .as_array()
        .ok_or_else(|| bad("final head scores absent"))?;
    for (j, score) in base.iter().enumerate() {
        let sum = heads.iter().try_fold(0i64, |s, h| {
            s.checked_add(
                h["scores_q24"][j]
                    .as_i64()
                    .ok_or_else(|| bad("head score absent"))?,
            )
            .ok_or_else(|| bad("head score overflow"))
        })?;
        need(
            sum == *score,
            "BASECopy differs from final head sum (cue/prefix already included)",
        )?;
    }
    let mut keys = [Vec::new(), Vec::new()];
    for j in 0..ids.len() {
        for lane in 0..8 {
            for f in 0..2 {
                let maps = if f == 0 {
                    &row["bank_trace"]["cue_bank"]["carrier"]["angular_indices"]
                } else {
                    &row["bank_trace"]["prefix"]["angular_indices"]
                };
                let k = &maps[lane][j];
                let key = if f == 0 && k.is_null() {
                    None
                } else {
                    let bin = integer(k)?;
                    need(bin < 120, "angular bin out of range")?;
                    Some(lane * 120 + bin)
                };
                keys[f].push(key);
            }
        }
    }
    Ok(Frame {
        ids,
        base,
        u,
        keys,
        query,
        sources,
        target: integer(&row["term"]["target"])?,
    })
}
fn evaluate(
    f: &Frame,
    base: &[i64],
    model: &NativeGeometricGenerate,
    bridge: &NativeGeometricReadStateBridge,
    reducer: &mut NativeVocabularyActions,
    reductions: &mut usize,
) -> Result<Value> {
    *reductions += 1;
    need(
        *reductions <= 9600,
        "declared full reduction ceiling exceeded",
    )?;
    let donor = earliest(base)?;
    let mut post = vec![H4Code::IDENTITY; 8];
    let mut actions = post.clone();
    let mut scores = vec![0; 960];
    bridge.apply_into(
        &f.query,
        &f.sources[donor],
        &mut post,
        &mut actions,
        &mut scores,
        &mut BridgeReadCounts::default(),
    )?;
    let mut gen = vec![0; 4096];
    model.score_into(&post, &mut gen, &mut GenerateReadCounts::default())?;
    for (x, u) in gen.iter_mut().zip(&f.u) {
        *x = x
            .checked_add(*u)
            .ok_or_else(|| bad("Generate/U overflow"))?;
    }
    let copy = base
        .iter()
        .zip(&f.ids)
        .map(|(x, t)| {
            x.checked_add(f.u[*t as usize])
                .ok_or_else(|| bad("Copy/U overflow"))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut weights = vec![0; reducer.action_count(copy.len())?];
    let mut masses = vec![0; 4096];
    let pool = reducer.reduce_into(&gen, &f.ids, &copy, &mut weights, &mut masses)?;
    let winner = pool.chosen_token_id as usize;
    let legal = reducer.legal_token_ids();
    let mut gm = [0u64; 2];
    for (i, t) in legal.iter().enumerate() {
        if *t as usize == f.target {
            gm[0] = weights[i];
        }
        if *t as usize == winner {
            gm[1] = weights[i];
        }
    }
    let aliases = |token: usize| {
        f.ids
            .iter()
            .enumerate()
            .filter_map(|(i, t)| (*t as usize == token).then_some(i))
            .collect::<Vec<_>>()
    };
    Ok(
        json!({"donor":donor,"post_state":post.iter().map(|x|x.index()).collect::<Vec<_>>(),"pool":pool,"target_mass":masses[f.target],"winner_mass":masses[winner],"mass_gap":i128::from(masses[f.target])-i128::from(masses[winner]),"target_generate_mass":gm[0],"target_copy_mass":masses[f.target]-gm[0],"winner_generate_mass":gm[1],"winner_copy_mass":masses[winner]-gm[1],"probability":masses[f.target] as f64/pool.total_weight_q31 as f64,"target_to_winner_ratio":masses[f.target] as f64/masses[winner] as f64,"target_copy_ordinals":aliases(f.target),"winner_copy_ordinals":aliases(winner),"base_copy_q24":base,"copy_q24":copy,"generate_q24_sha256":hash(&serde_json::to_vec(&gen)?),"copy_q24_sha256":hash(&serde_json::to_vec(&copy)?)}),
    )
}
fn parity(snapshot: &Value, saved: &Value) -> Result<()> {
    need(
        snapshot["pool"] == saved["pool"]
            && snapshot["target_mass"] == saved["target_mass"]
            && snapshot["pool"]["total_weight_q31"] == saved["denominator"]
            && snapshot["post_state"] == saved["factual_state"]
            && snapshot["generate_q24_sha256"] == saved["generate_raw_scores_sha256"],
        "saved native objective endpoint parity differs",
    )
}
fn contrasts(f: &Frame) -> Value {
    let mut out = Vec::new();
    for (a, t) in f.ids.iter().enumerate() {
        if *t as usize != f.target {
            continue;
        }
        for b in 0..f.ids.len() {
            let mult = |keys: &[Option<usize>], j: usize| {
                let mut m = BTreeMap::<usize, usize>::new();
                for k in keys[j * 8..j * 8 + 8].iter().flatten() {
                    *m.entry(*k).or_default() += 1;
                }
                m
            };
            let desired = [mult(&f.keys[0], a), mult(&f.keys[1], a)];
            let rival = [mult(&f.keys[0], b), mult(&f.keys[1], b)];
            out.push(json!({"target_token_ordinal":a,"rival_ordinal":b,"desired_keys":desired,"rival_keys":rival,"both_family_incidence_equal":desired==rival,"baseline_base_gap_q24":f.base[a]-f.base[b],"baseline_u_combined_gap_q24":f.base[a]+f.u[f.target]-f.base[b]-f.u[f.ids[b] as usize],"scope":"target-token alias only; expected source/offset annotations require independent posthoc join; equality proves this BASECopy gap invariant only"}));
        }
    }
    json!(out)
}
fn authenticate(root: &Path, report: &str, manifest: &str) -> Result<Value> {
    report_output::verify(root)?;
    need(
        hash(&bytes(&root.join("report.json"))?) == report
            && hash(&bytes(&root.join("manifest.json"))?) == manifest,
        "sealed report pins differ",
    )?;
    let r = read(&root.join("report.json"))?;
    need(
        r["status"] == "COMPLETED",
        "authority execution not completed",
    )?;
    Ok(r)
}
fn validate_trials(trials: &[Value], old: &[Vec<i8>; 2], candidate: &[Vec<i8>; 2]) -> Result<()> {
    need(trials.len() == 1920, "original trial census differs")?;
    let mut seen = BTreeSet::new();
    let mut last: Option<(f64, String, usize)> = None;
    let mut replay = old.clone();
    for t in trials {
        let name = t["name"]
            .as_str()
            .ok_or_else(|| bad("trial family absent"))?;
        let family = NAMES
            .iter()
            .position(|x| *x == name)
            .ok_or_else(|| bad("unexpected trial family"))?;
        let i = integer(&t["index"])?;
        need(
            i < 960 && seen.insert((family, i)),
            "duplicate/invalid trial coordinate",
        )?;
        let before = t["before"]
            .as_i64()
            .ok_or_else(|| bad("trial before absent"))?;
        let after = t["after"]
            .as_i64()
            .ok_or_else(|| bad("trial after absent"))?;
        let g = t["gradient"]
            .as_f64()
            .ok_or_else(|| bad("trial gradient absent"))? as f32;
        need(g.is_finite(), "nonfinite trial gradient")?;
        let wanted = (before
            + if g > 0. {
                -1
            } else if g < 0. {
                1
            } else {
                0
            })
        .clamp(-7, 7);
        need(
            before == i64::from(old[family][i])
                && after == wanted
                && t["original_master"].as_f64() == Some(before as f64 * 0.25),
            "frozen original/adjacent trial differs",
        )?;
        let predicted = if after == before {
            0.
        } else {
            f64::from(g) * (after - before) as f64 * 0.25
        };
        let stored = t["predicted_delta"]
            .as_f64()
            .ok_or_else(|| bad("predicted delta absent"))?;
        need(
            (predicted - stored).abs() < 1e-12 * (1. + predicted.abs()),
            "actual displacement utility differs",
        )?;
        let order = (stored, name.to_owned(), i);
        if let Some(p) = &last {
            need(p <= &order, "original proposal order differs")?;
        }
        last = Some(order);
        let status = t["status"]
            .as_str()
            .ok_or_else(|| bad("trial status absent"))?;
        need(
            ["accepted", "rejected", "saturated", "zero_gradient"].contains(&status),
            "unexpected trial status",
        )?;
        need(
            (status == "zero_gradient") == (g == 0.)
                && (status == "saturated") == (g != 0. && after == before),
            "no-op status differs",
        )?;
        if status == "accepted" {
            replay[family][i] = after as i8;
        }
        if status == "rejected" {
            need(
                integer(&t["blocking_pool"])? < 377,
                "original first blocker index differs",
            )?;
            need(
                t["blocking_pool"].is_u64() && t["blocking_winner"].is_u64(),
                "original first blocker absent",
            )?;
        }
    }
    need(
        &replay == candidate,
        "accepted path candidate packed endpoint differs",
    )
}
fn run(c: &Config, written: &mut u64) -> Result<Value> {
    let started = Instant::now();
    let report = authenticate(
        &c.readout_root,
        &c.expected_readout_report_sha256,
        &c.expected_readout_manifest_sha256,
    )?;
    let parent = authenticate(
        &c.reached_u_root,
        &c.expected_reached_u_report_sha256,
        &c.expected_reached_u_manifest_sha256,
    )?;
    need(
        report["mode"] == "readout_coadaptation"
            && parent["mode"] == "reached_u"
            && report["native_code_proposals"]["winner"].is_null(),
        "original rejected candidate authority differs",
    )?;
    report_output::verify(&c.inputs_seal_root)?;
    need(
        hash(&bytes(&c.inputs_seal_root.join("manifest.json"))?)
            == c.expected_inputs_manifest_sha256
            && hash(&bytes(&c.inputs)?) == c.expected_inputs_sha256
            && hash(&bytes(&c.labels)?) == c.expected_labels_sha256,
        "panel pins differ",
    )?;
    let inputs = read(&c.inputs)?;
    let labels = read(&c.labels)?;
    need(
        inputs["cases"].as_array().is_some_and(|a| a.len() == 512)
            && labels["cases"].as_array().is_some_and(|a| a.len() == 512),
        "panel cardinality differs",
    )?;
    let cp = c.reached_u_root.join("checkpoint-0001");
    let receipt = read(&cp.join("receipt.json"))?;
    need(
        parent["final_receipt"] == receipt,
        "selected parent receipt differs",
    )?;
    let binding: NativeArtifactBinding = arr(&receipt["parent"])?;
    need(
        binding.metadata_sha256 == c.expected_source_metadata_sha256,
        "Source binding differs",
    )?;
    let native = NativeSourceRealizer::load_native(&cp.join("native"), &binding)?;
    let gen_bytes = bytes(&cp.join("generate.bin"))?;
    let ub = bytes(&cp.join("continuation-field.bin"))?;
    need(
        hash(&gen_bytes) == c.expected_generate_sha256
            && hash(&ub) == c.expected_continuation_sha256,
        "fixed Generate/U pins differ",
    )?;
    let model = NativeGeometricGenerate::from_bytes(&gen_bytes, native.binding())?;
    need(
        model.lanes() == 8 && model.vocab_size() == 4096,
        "fixed Generate shape differs",
    )?;
    let bridge_bytes = bytes(&cp.join("read-state-bridge-categorical.bin"))?;
    need(
        receipt["categorical_sha256"] == hash(&bridge_bytes),
        "bridge pin differs",
    )?;
    let bridge = NativeGeometricReadStateBridge::from_bytes(&bridge_bytes, native.binding())?;
    let field = NativeContinuationField::from_bytes(&ub, &binding, &model)?;
    need(field.applies_to_copy(), "shared-action U v2 required")?;
    let exp = bytes(&cp.join("native/consumer/exp-q31.bin"))?;
    let mut reducer = NativeVocabularyActions::new(native.binding().clone(), &exp)?;
    let candidate_cp = c.readout_root.join("native-candidate-00/checkpoint-0000");
    report_output::verify(&c.readout_root.join("native-candidate-00"))?;
    for other in [
        c.readout_root.join("checkpoint-0000"),
        candidate_cp.clone(),
        c.readout_root.join("checkpoint-0001"),
    ] {
        for file in [
            "generate.bin",
            "continuation-field.bin",
            "read-state-bridge-categorical.bin",
            "cue/cue-joint-q4.bin",
            "native/consumer/exp-q31.bin",
        ] {
            need(
                bytes(&cp.join(file))? == bytes(&other.join(file))?,
                "frozen finite payload differs",
            )?;
        }
    }
    let old = codes(&cp)?;
    let candidate = codes(&candidate_cp)?;
    need(
        codes(&c.readout_root.join("checkpoint-0000"))? == old
            && codes(&c.readout_root.join("checkpoint-0001"))? == old,
        "initial/final retained table differs",
    )?;
    let construction = read(&c.readout_root.join("readout-construction.json"))?;
    let trials = construction["construction"]["trials"]
        .as_array()
        .ok_or_else(|| bad("original trial inventory absent"))?;
    validate_trials(trials, &old, &candidate)?;
    need(
        construction["construction"]["stats"]["protected_pools"] == 377
            && construction["construction"]["stats"]["protected_decisions_preserved"] == true,
        "original guard scope differs",
    )?;
    let ranking = read(&c.readout_root.join("native-code-ranking-gradients.json"))?;
    need(
        ranking.as_object().is_some_and(|m| m.len() == 2),
        "raw ranking family inventory differs",
    )?;
    for name in NAMES {
        let item = &ranking[name];
        let filename = format!("native-code-ranking-gradients/{name}.f32le");
        need(
            item["file"] == filename
                && item["shape"] == json!([960])
                && item["bytes"] == 3840
                && item["missing_gradient_filled_zero"] == false,
            "raw ranking inventory shape differs",
        )?;
        let raw = bytes(&c.readout_root.join(&filename))?;
        need(
            raw.len() == 3840 && item["sha256"] == hash(&raw),
            "raw ranking file pin differs",
        )?;
        for t in trials.iter().filter(|t| t["name"] == name) {
            let i = integer(&t["index"])?;
            let b: [u8; 4] = raw[i * 4..i * 4 + 4].try_into()?;
            need(
                (t["gradient"]
                    .as_f64()
                    .ok_or_else(|| bad("saved gradient missing"))? as f32)
                    .to_bits()
                    == f32::from_le_bytes(b).to_bits(),
                "trial gradient differs from authenticated raw inventory",
            )?;
        }
    }

    let gradient = read(&c.readout_root.join("coefficient-gradient-receipt.json"))?;
    let rows = gradient["rows"]
        .as_array()
        .ok_or_else(|| bad("saved gradient rows absent"))?;
    need(rows.len() == 89, "saved gradient term census differs")?;
    let plan = read(&c.readout_root.join("frontier-plan.json"))?;
    let terms = plan["terms"]
        .as_array()
        .ok_or_else(|| bad("fixed plan absent"))?;
    need(terms.len() == 89, "frozen objective census differs")?;
    let mut reductions = 0usize;
    let mut summaries = Vec::new();
    for (fi, index) in FACTS.iter().enumerate() {
        let row = rows
            .get(fi)
            .ok_or_else(|| bad("factual gradient row absent"))?;
        let term = &row["term"];
        need(
            term["index"] == *index
                && term["position"] == 3
                && term["component"] == 0
                && term["weight"] == 0.2
                && terms[fi]["phase"] == 1,
            "frozen five actual factual terms differ",
        )?;
        for k in [
            "index",
            "position",
            "target",
            "component",
            "weight",
            "parent_actual_prefix_ids",
        ] {
            need(term[k] == terms[fi][k], "objective term/order differs")?;
        }
        let saved = read(
            &c.reached_u_root
                .join(format!("development-0001-row-{index:04}.json")),
        )?;
        need(
            row["id"] == inputs["cases"][*index]["id"]
                && row["id"] == labels["cases"][*index]["id"]
                && row["id"] == saved["id"]
                && row["actual_prefix_ids"] == term["parent_actual_prefix_ids"]
                && row["actual_prefix_ids"] == saved["generation"][3]["actual_prefix_ids"]
                && term["target"] == saved["canonical"][3]["target_label_only"],
            "actual row/prefix/label authority differs",
        )?;
        let prefix: Vec<u32> = arr(&row["actual_prefix_ids"])?;
        need(
            prefix.len() == 3 && !prefix.contains(&1),
            "actual pos3 prefix/EOS differs",
        )?;
        for (pos, t) in prefix.iter().enumerate() {
            need(
                saved["generation"][pos]["pool"]["summary"]["chosen_token_id"] == *t,
                "saved actual winner chain differs",
            )?;
        }
        let f = prepare(row, &model, &field)?;
        need(f.target < 4096, "target vocabulary bound")?;
        let mut current = f.base.clone();
        let baseline = evaluate(&f, &current, &model, &bridge, &mut reducer, &mut reductions)?;
        parity(&baseline, &report["baseline_objective"]["terms"][fi])?;
        need(
            baseline["pool"] == row["pool"]
                && baseline["post_state"] == row["post_state"]
                && baseline["generate_q24_sha256"] == jhash(&row["generate_q24"])?
                && baseline["copy_q24_sha256"] == jhash(&row["copy_q24"])?,
            "saved gradient complete pool parity differs",
        )?;
        let mut current_snapshot = baseline.clone();
        let mut records = Vec::new();
        let mut counts = BTreeMap::<String, usize>::new();
        let mut ever_correct = false;
        for (order, t) in trials.iter().enumerate() {
            let family = if t["name"] == NAMES[0] { 0 } else { 1 };
            let coordinate = integer(&t["index"])?;
            let delta = (t["after"]
                .as_i64()
                .ok_or_else(|| bad("trial after absent"))?
                - t["before"]
                    .as_i64()
                    .ok_or_else(|| bad("trial before absent"))?) as i8;
            let proposed = stage(&current, &f.keys[family], coordinate, delta)?;
            let changed = proposed != current;
            let staged = if changed {
                evaluate(
                    &f,
                    &proposed,
                    &model,
                    &bridge,
                    &mut reducer,
                    &mut reductions,
                )?
            } else {
                current_snapshot.clone()
            };
            let accepted = t["status"] == "accepted";
            let before_correct = current_snapshot["pool"]["chosen_token_id"] == f.target;
            let staged_correct = staged["pool"]["chosen_token_id"] == f.target;
            let mass = |v: &Value, k: &str| -> Result<u128> {
                Ok(u128::from(
                    v[k].as_u64().ok_or_else(|| bad("exact mass absent"))?,
                ))
            };
            let total = |v: &Value| -> Result<u128> {
                Ok(u128::from(
                    v["pool"]["total_weight_q31"]
                        .as_u64()
                        .ok_or_else(|| bad("exact total absent"))?,
                ))
            };
            let probability_gain = mass(&staged, "target_mass")? * total(&current_snapshot)?
                > mass(&current_snapshot, "target_mass")? * total(&staged)?;
            let ratio_gain = mass(&staged, "target_mass")?
                * mass(&current_snapshot, "winner_mass")?
                > mass(&current_snapshot, "target_mass")? * mass(&staged, "winner_mass")?;
            let gap = |s: &Value| -> Result<i128> {
                Ok(i128::from(
                    s["target_mass"]
                        .as_u64()
                        .ok_or_else(|| bad("target mass absent"))?,
                ) - i128::from(
                    s["winner_mass"]
                        .as_u64()
                        .ok_or_else(|| bad("winner mass absent"))?,
                ))
            };
            let gap_gain = gap(&staged)? > gap(&current_snapshot)?;
            let crossing = accepted && !before_correct && staged_correct;
            let regression = accepted && before_correct && !staged_correct;
            for (name, yes) in [
                ("accepted_crossings", crossing),
                ("accepted_regressions", regression),
                (
                    "rejected_hypothetical_corrections",
                    t["status"] == "rejected" && staged_correct,
                ),
                (
                    "rejected_probability_gains",
                    t["status"] == "rejected" && probability_gain,
                ),
                (
                    "rejected_mass_gap_gains",
                    t["status"] == "rejected" && gap_gain,
                ),
                (
                    "rejected_ratio_gains",
                    t["status"] == "rejected" && ratio_gain,
                ),
            ] {
                if yes {
                    *counts.entry(name.to_owned()).or_default() += 1;
                }
            }
            records.push(json!({"order":order,"name":t["name"],"index":coordinate,"before":t["before"],"after":t["after"],"original_status":t["status"],"blocking_pool":t["blocking_pool"],"blocking_winner":t["blocking_winner"],"committed":accepted,"numerically_changed":changed,"previous_correct":before_correct,"staged_correct":staged_correct,"probability_gain":probability_gain,"mass_gap_gain":gap_gain,"ratio_gain":ratio_gain,"accepted_crossing":crossing,"accepted_regression":regression,"staged":staged}));
            settle(&mut current, proposed, accepted);
            if accepted {
                current_snapshot = staged;
                ever_correct |= staged_correct;
            }
        }
        parity(
            &current_snapshot,
            &report["candidate_objective"]["terms"][fi],
        )?;
        let mut expected = f.base.clone();
        for family in 0..2 {
            for i in 0..960 {
                expected = stage(
                    &expected,
                    &f.keys[family],
                    i,
                    candidate[family][i] - old[family][i],
                )?;
            }
        }
        need(
            expected == current,
            "candidate table drift vs accepted path differs",
        )?;
        let name = format!("frame-{index:04}-position-03.json");
        let frame = json!({"schema":"uor-r4.native-readout-path-frame/1","input_index":index,"position":3,"id":row["id"],"term":term,"actual_prefix_ids":prefix,"saved_gradient_row":row,"original_keys":{"cue":f.keys[0],"prefix":f.keys[1]},"baseline":baseline,"candidate_endpoint":current_snapshot,"incidence_contrasts":contrasts(&f),"counts":counts,"ever_correct_accepted_state":ever_correct,"trials":records,"scope":"offline labels interpret target-token aliases; source authority posthoc, never donor selection"});
        let payload = serde_json::to_vec(&frame)?;
        need(
            payload.len() as u64 <= c.maximum_cache_bytes,
            "single frame report/cache cap exceeded",
        )?;
        write(c, &name, &frame, written)?;
        summaries.push(json!({"input_index":index,"position":3,"file":name,"sha256":hash(&payload),"baseline":baseline,"candidate_endpoint":current_snapshot,"counts":counts,"ever_correct_accepted_state":ever_correct}));
    }
    let mut inventory = BTreeMap::new();
    for file in [
        "frontier-plan.json",
        "reference-plan.json",
        "readout-construction.json",
        "coefficient-gradient-receipt.json",
        "native-code-ranking-gradients.json",
    ] {
        inventory.insert(file, hash(&bytes(&c.readout_root.join(file))?));
    }
    Ok(
        json!({"schema":"uor-r4.native-readout-path-attribution/1","status":"COMPLETED","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"frames":summaries,"full_reductions":reductions,"maximum_full_reductions":9600,"original_trials":1920,"original_codes":old,"candidate_codes":candidate,"endpoint_table_parity_verified":true,"resource_scope":{"numerical_cache":"no donor-vector cache; one full4096score/mass workspace at a time, below1MiB","maximum_numerical_cache_bytes":c.maximum_cache_bytes,"maximum_report_bytes":c.maximum_report_bytes,"serialized_frame_limit":c.maximum_cache_bytes,"saved_input_JSON_and_report_construction":"supervised8GiBRAM, separate from numerical cache","cpu_threads":2},"authority_files_sha256":inventory,"readout_report_sha256":c.expected_readout_report_sha256,"readout_manifest_sha256":c.expected_readout_manifest_sha256,"reached_u_report_sha256":c.expected_reached_u_report_sha256,"reached_u_manifest_sha256":c.expected_reached_u_manifest_sha256,"source_metadata_sha256":c.expected_source_metadata_sha256,"generate_sha256":c.expected_generate_sha256,"continuation_sha256":c.expected_continuation_sha256,"elapsed_seconds":started.elapsed().as_secs_f64(),"scope":"saved finite five-factual-prefix attribution only; producer statuses authoritative; finite native Generate/bridge/U scoring from saved states; no guard re-admission, global feasibility, autoregressive rollout, Context encoder replay, new input, gradient, model export, complete-reply, heldout, chat or energy claim"}),
    )
}
fn paths(c: &mut Config) -> Result<()> {
    need(
        c.maximum_cache_bytes > 0
            && c.maximum_cache_bytes <= 64 * 1024 * 1024
            && c.maximum_report_bytes > 0
            && c.maximum_report_bytes <= 64 * 1024 * 1024,
        "64MiB cache/report admission differs",
    )?;
    c.output = output_support::prospective_output(&c.output)?;
    for p in [
        &mut c.readout_root,
        &mut c.reached_u_root,
        &mut c.inputs_seal_root,
        &mut c.inputs,
        &mut c.labels,
    ] {
        *p = fs::canonicalize(&*p)?;
    }
    need(
        c.inputs.starts_with(&c.inputs_seal_root) && c.labels.starts_with(&c.inputs_seal_root),
        "panel files outside admitted seal",
    )?;
    for p in [&c.readout_root, &c.reached_u_root, &c.inputs_seal_root] {
        need(
            !c.output.starts_with(p) && !p.starts_with(&c.output),
            "output overlaps authority",
        )?;
    }
    need(
        c.expected_readout_report_sha256
            == "d1d47739b897c4d6cd7d7e01500d0ba05749dce885dd64bb3530001b7007222b"
            && c.expected_readout_manifest_sha256
                == "01fa3c74d69a6e1d9ba45e82c58cf84a9d0fa203488b77a4757cdbe137f5248b"
            && c.expected_reached_u_report_sha256
                == "2a9f967a955c6dc40b422c94f1b6d1d54f2511a24fb601af97218d49440b9923"
            && c.expected_reached_u_manifest_sha256
                == "6cbfabf807427f3baa2bfcfda7187e1b00f3a205ec9466b30a7437a410baad94"
            && c.expected_inputs_manifest_sha256
                == "d24b87cff9295f34c7996eed3b974e08531c2b594e62fb5b2795f9f2972404a4"
            && c.expected_inputs_sha256
                == "b9661606b280884217a64e0a5b643f8324a90390e47ade7241da0889a5f7c86a"
            && c.expected_labels_sha256
                == "84991e0657b5697c0e061eaa3fe86e4a0ec7ce6bc2be8371b62698c6b8126155"
            && c.expected_source_metadata_sha256
                == "9f0b272e7852a47bbbad0f549e8157a44ff3212af86905907501ec11f3e7989b"
            && c.expected_generate_sha256
                == "4248245471db609b1fc19482e8f180380b292c5832fc90c81ce69947bc4b7737"
            && c.expected_continuation_sha256
                == "82ae9daeb402b288e64492d5b299110b36849907019c952609a1cae6612673ee",
        "prospective fixed identities differ",
    )?;
    Ok(())
}
fn main() -> Result<()> {
    let argv = std::env::args().collect::<Vec<_>>();
    if argv.len() == 3 && argv[1] == "verify-report" {
        report_output::verify(Path::new(&argv[2]))?;
        return Ok(());
    }
    need(
        argv.len() == 2,
        "usage: native-readout-path-attribution CONFIG.json",
    )?;
    let raw = bytes(Path::new(&argv[1]))?;
    let mut c: Config = serde_json::from_slice(&raw)?;
    paths(&mut c)?;
    report_output::claim(&c.output)?;
    let mut written = 0;
    let result = (|| {
        write(
            &c,
            "config.json",
            &serde_json::from_slice(&raw)?,
            &mut written,
        )?;
        run(&c, &mut written)
    })();
    let mut report = match &result {
        Ok(v) => v.clone(),
        Err(e) => {
            json!({"schema":"uor-r4.native-readout-path-attribution/1","status":"FAILED","error":e.to_string(),"scope":"saved diagnostic execution failure; no model-quality verdict"})
        }
    };
    report["external_config_sha256"] = json!(hash(&raw));
    report["attempt_argv"] = json!(argv);
    let data = serde_json::to_vec(&report)?;
    let exceeded = written
        .checked_add(data.len() as u64)
        .map_or(true, |n| n > c.maximum_report_bytes);
    if exceeded {
        report = json!({"schema":"uor-r4.native-readout-path-attribution/1","status":"FAILED","error":"final report byte cap exceeded"});
    }
    fs::write(c.output.join("report.json"), serde_json::to_vec(&report)?)?;
    report_output::seal(&c.output)?;
    report_output::verify(&c.output)?;
    if exceeded {
        return Err(bad("final report byte cap exceeded"));
    }
    result.map(|_| ())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejected_donor_switch_does_not_enter_accepted_path() -> Result<()> {
        let mut base = vec![10, 10, 9];
        let mut keys = vec![None; 24];
        keys[16] = Some(4);
        keys[17] = Some(4);
        need(earliest(&base)? == 0, "earliest physical tie")?;
        let proposed = stage(&base, &keys, 4, 1)?;
        need(
            earliest(&proposed)? == 2,
            "two logical incidences switch donor",
        )?;
        settle(&mut base, proposed.clone(), false);
        assert_eq!(base, vec![10, 10, 9]);
        assert_eq!(earliest(&base)?, 0);
        settle(&mut base, proposed, true);
        assert_eq!(earliest(&base)?, 2);
        Ok(())
    }
    #[test]
    fn signed_shared_key_alias_gap_stays_invariant() -> Result<()> {
        let base = vec![100, 90, 80];
        let mut keys = vec![None; 24];
        keys[0] = Some(8);
        keys[8] = Some(8);
        keys[9] = Some(8);
        let out = stage(&base, &keys, 8, -1)?;
        assert_eq!(out[0] - out[2], 20 - (1 << 22));
        assert_eq!(out[1] - out[0], -10 - (1 << 22));
        let same = stage(&[30, 20], &[Some(1); 16], 1, 1)?;
        assert_eq!(same[0] - same[1], 10);
        Ok(())
    }
    #[test]
    fn physical_alias_pool_uses_actual_native_reducer() -> Result<()> {
        use uor_r4_integer::geometric_source_actions::SourceActionBinding;
        let mut vocab = serde_json::Map::new();
        for (i, s) in ["<|bos|>", "<|eos|>", "<|unk|>", ".", "a", "b"]
            .iter()
            .enumerate()
        {
            vocab.insert((*s).to_owned(), json!(i));
        }
        let tokenizer = json!({"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":vocab,"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]});
        let binding = SourceActionBinding::new(&serde_json::to_vec(&tokenizer)?)?;
        let exp = (0..uor_r4_integer::geometric_read::EXP_TABLE_LEN)
            .flat_map(|i| {
                (((-(i as f64) / 256.).exp() * (1u64 << 31) as f64).round() as u32).to_le_bytes()
            })
            .collect::<Vec<_>>();
        let mut reducer = NativeVocabularyActions::new(binding, &exp)?;
        let trace = reducer.reduce_trace(&[0; 6], &[5, 5], &[0, 0])?;
        assert_eq!(trace.summary.chosen_token_id, 5);
        assert_eq!(trace.token_masses[5].copy_weight_q31, 2 * (1u64 << 31));
        let tied = reducer.reduce_trace(&[0; 6], &[], &[])?;
        assert_eq!(tied.summary.chosen_token_id, 0);
        Ok(())
    }
}
