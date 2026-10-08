//! Exhaustive one-shared-prototype-code neighborhood at a sealed emission endpoint.
//! Fixed-state diagnostic only: no gradient, checkpoint mutation or own-prefix gain claim.
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::{
    native_geometric::learner::{
        geometric_continuation_field::NativeContinuationField,
        geometric_generate::GenerateReadCounts,
        native_bank_generate::{
            BankPin, BoundNativeBytes, NativeBankArtifacts, NativeBankGenerator, OwnedBankSegment,
            OwnedBankSource, PinnedBankSnapshot, SnapshotSourceStatus,
        },
    },
    report_output,
};
use uor_r4_integer::{
    geometric_cue_carrier::CueCarrierMetadata,
    geometric_prefix_transport::PrefixTransportMetadata,
    geometric_source_actions::SourceActionBinding,
    geometric_source_realizer::NativeArtifactBinding,
    geometric_vocabulary_actions::{GeneratePatchCache, NativeVocabularyActions},
};
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    emission_root: PathBuf,
    expected_report_sha256: String,
    expected_manifest_sha256: String,
    expected_source_metadata_sha256: String,
    expected_generate_sha256: String,
    expected_continuation_sha256: String,
    inputs: PathBuf,
    inputs_seal_root: PathBuf,
    expected_inputs_sha256: String,
    frontier_root: PathBuf,
    expected_frontier_report_sha256: String,
    expected_frontier_manifest_sha256: String,
    output: PathBuf,
    maximum_cache_bytes: u64,
    maximum_report_bytes: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Panel {
    schema: String,
    cases: Vec<Packet>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Packet {
    id: String,
    segments: Vec<Segment>,
    query_ids: Vec<u32>,
    actual_prefix_ids: Vec<u32>,
}
#[derive(Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Segment {
    Source {
        event: u64,
        record: u64,
        commit: u64,
        scope: String,
        entity: Vec<u32>,
        relation: u32,
        view: u32,
        original_source_ids: Vec<u32>,
    },
    Context {
        event: u64,
        role: u32,
        token_ids: Vec<u32>,
    },
}
#[derive(Deserialize)]
struct Term {
    index: usize,
    position: usize,
    target: u32,
    component: usize,
    weight: f64,
    parent_actual_prefix_ids: Vec<u32>,
}
#[derive(Deserialize)]
struct SavedTokenMass {
    token_id: u32,
    weight_q31: u64,
    generate_weight_q31: u64,
    copy_weight_q31: u64,
}
#[derive(Deserialize)]
struct SavedProtectedPool {
    term: Value,
    generate_q24: Vec<i64>,
    copy_ids: Vec<u32>,
    copy_q24: Vec<i64>,
    token_masses: Vec<SavedTokenMass>,
}
struct Row {
    cache: GeneratePatchCache,
    target: u32,
    component: usize,
    weight: f64,
    conditional: Vec<i64>,
    baseline_gap: f64,
    baseline_correct: bool,
}
fn bad(s: &str) -> Box<dyn std::error::Error> {
    std::io::Error::other(s).into()
}
fn require(ok: bool, s: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(bad(s))
    }
}
fn bytes(p: &Path) -> Result<Vec<u8>> {
    Ok(fs::read(p)?)
}
fn hash(b: &[u8]) -> String {
    hex::encode(Sha256::digest(b))
}
fn file_hash(p: &Path) -> Result<String> {
    Ok(hash(&bytes(p)?))
}
fn read(p: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&bytes(p)?)?)
}
fn text(v: &Value) -> Result<&str> {
    v.as_str().ok_or_else(|| bad("required string absent"))
}
fn write(c: &Config, name: &str, value: &Value, written: &mut u64) -> Result<()> {
    let b = serde_json::to_vec(value)?;
    *written = written
        .checked_add(b.len() as u64)
        .ok_or_else(|| bad("report byte overflow"))?;
    require(
        *written <= c.maximum_report_bytes,
        "report byte cap exhausted",
    )?;
    fs::write(c.output.join(name), b)?;
    Ok(())
}
fn snapshot(p: &Packet) -> Result<PinnedBankSnapshot> {
    require(
        p.actual_prefix_ids.is_empty(),
        "supplied input prefix excluded",
    )?;
    let mut scope = None;
    let mut commit = 0;
    for segment in &p.segments {
        if let Segment::Source {
            scope: s,
            commit: c,
            ..
        } = segment
        {
            require(
                !s.is_empty() && scope.map_or(true, |old: &String| old == s),
                "mixed/empty bank scope",
            )?;
            scope = Some(s);
            commit = commit.max(*c);
        }
    }
    let scope = scope.ok_or_else(|| bad("required Source bank absent"))?;
    Ok(PinnedBankSnapshot {
        pin: BankPin {
            lineage: 0,
            commit,
            scope: scope.as_bytes().to_vec(),
        },
        query_ids: p.query_ids.clone(),
        segments: p
            .segments
            .iter()
            .map(|s| match s {
                Segment::Source {
                    event,
                    record,
                    commit,
                    scope,
                    entity,
                    relation,
                    view,
                    original_source_ids,
                } => OwnedBankSegment::Source(OwnedBankSource {
                    event: *event,
                    record: *record,
                    commit: *commit,
                    scope: scope.as_bytes().to_vec(),
                    entity: entity.clone(),
                    relation: *relation,
                    view: *view,
                    status: SnapshotSourceStatus::Found,
                    original_token_ids: original_source_ids.clone(),
                }),
                Segment::Context {
                    event,
                    role,
                    token_ids,
                } => OwnedBankSegment::Context {
                    event: *event,
                    role: *role,
                    token_ids: token_ids.clone(),
                },
            })
            .collect(),
    })
}
fn admit_paths(c: &mut Config) -> Result<()> {
    for p in [
        &c.emission_root,
        &c.inputs,
        &c.inputs_seal_root,
        &c.frontier_root,
        &c.output,
    ] {
        require(
            p.is_absolute()
                && !p
                    .components()
                    .any(|x| matches!(x, std::path::Component::ParentDir)),
            "absolute paths without parent traversal required",
        )?;
    }
    require(
        c.maximum_cache_bytes > 0
            && c.maximum_cache_bytes <= 128 * 1024 * 1024
            && c.maximum_report_bytes >= 1024 * 1024,
        "invalid bounded diagnostic caps",
    )?;
    c.emission_root = fs::canonicalize(&c.emission_root)?;
    c.inputs = fs::canonicalize(&c.inputs)?;
    c.inputs_seal_root = fs::canonicalize(&c.inputs_seal_root)?;
    c.frontier_root = fs::canonicalize(&c.frontier_root)?;
    c.output = output_support::prospective_output(&c.output)?;
    require(
        c.inputs.starts_with(&c.inputs_seal_root),
        "input lies outside declared seal",
    )?;
    for p in [&c.emission_root, &c.inputs_seal_root, &c.frontier_root] {
        require(
            !c.output.starts_with(p) && !p.starts_with(&c.output),
            "output overlaps retained sealed material",
        )?;
    }
    Ok(())
}
fn run(c: &Config, written: &mut u64) -> Result<Value> {
    let clock = Instant::now();
    report_output::verify(&c.emission_root)?;
    report_output::verify(&c.inputs_seal_root)?;
    report_output::verify(&c.frontier_root)?;
    require(
        file_hash(&c.emission_root.join("report.json"))? == c.expected_report_sha256
            && file_hash(&c.emission_root.join("manifest.json"))? == c.expected_manifest_sha256
            && file_hash(&c.inputs)? == c.expected_inputs_sha256
            && file_hash(&c.frontier_root.join("report.json"))?
                == c.expected_frontier_report_sha256
            && file_hash(&c.frontier_root.join("manifest.json"))?
                == c.expected_frontier_manifest_sha256,
        "retained evidence pins differ",
    )?;
    let report = read(&c.emission_root.join("report.json"))?;
    let cp = c.emission_root.join("checkpoint-0001");
    let receipt = read(&cp.join("receipt.json"))?;
    require(
        report["status"] == "COMPLETED"
            && report["constrained_emission_learning"] == true
            && report["native_code_proposals"]["winner"] == 0
            && report["final_receipt"] == receipt
            && receipt["step"] == 1,
        "selected emission endpoint differs",
    )?;
    let binding: NativeArtifactBinding = serde_json::from_value(receipt["parent"].clone())?;
    require(
        binding.metadata_sha256 == c.expected_source_metadata_sha256,
        "Source endpoint pin differs",
    )?;
    let gen = bytes(&cp.join("generate.bin"))?;
    let bridge = bytes(&cp.join("read-state-bridge-categorical.bin"))?;
    let exp = bytes(&cp.join("native/consumer/exp-q31.bin"))?;
    let exp_hash = hash(&exp);
    let cue = bytes(&cp.join("cue/cue-q4.bin"))?;
    let prefix = bytes(&cp.join("prefix/prefix-q4.bin"))?;
    let joint = if cp.join("cue/cue-joint-q4.bin").exists() {
        Some(bytes(&cp.join("cue/cue-joint-q4.bin"))?)
    } else {
        None
    };
    let cue_metadata: CueCarrierMetadata =
        serde_json::from_value(read(&cp.join("cue/native-metadata.json"))?)?;
    let prefix_metadata: PrefixTransportMetadata =
        serde_json::from_value(read(&cp.join("prefix/native-metadata.json"))?)?;
    require(
        hash(&gen) == c.expected_generate_sha256
            && receipt["generate_sha256"] == c.expected_generate_sha256,
        "Generate endpoint pin differs",
    )?;
    let mut generator = NativeBankGenerator::load(NativeBankArtifacts {
        native_directory: &cp.join("native"),
        source_binding: &binding,
        generate: BoundNativeBytes {
            bytes: &gen,
            sha256: &c.expected_generate_sha256,
        },
        bridge: Some(BoundNativeBytes {
            bytes: &bridge,
            sha256: text(&receipt["categorical_sha256"])?,
        }),
        cue_packed: &cue,
        cue_joint_packed: joint.as_deref(),
        cue_metadata: &cue_metadata,
        prefix_packed: &prefix,
        prefix_metadata: &prefix_metadata,
        exp: BoundNativeBytes {
            bytes: &exp,
            sha256: &exp_hash,
        },
    })?;
    require(
        generator.generate_model().lanes() == 8 && generator.generate_model().vocab_size() == 4096,
        "retained Generate shape differs",
    )?;
    let u = bytes(&cp.join("continuation-field.bin"))?;
    require(
        hash(&u) == c.expected_continuation_sha256
            && receipt["continuation_sha256"] == c.expected_continuation_sha256,
        "U endpoint pin differs",
    )?;
    let field = NativeContinuationField::from_bytes(&u, &binding, generator.generate_model())?;
    for lane in 0..8 {
        for relative in 0..120 {
            require(
                field.coefficient_unary(lane, relative)? == 0,
                "prototype-dependent U is nonzero",
            )?;
        }
    }
    generator = generator.with_continuation_field(BoundNativeBytes {
        bytes: &u,
        sha256: &c.expected_continuation_sha256,
    })?;
    let action_binding = SourceActionBinding::new(&bytes(&cp.join("native/tokenizer.json"))?)?;
    let mut reducer = NativeVocabularyActions::new(action_binding, &exp)?;
    require(
        reducer.legal_token_ids().iter().copied().eq(0u32..4096),
        "full legal vocabulary differs",
    )?;
    let panel: Panel = serde_json::from_slice(&bytes(&c.inputs)?)?;
    require(
        panel.schema == "uor-r4.native-source-bank-probe-input/1" && panel.cases.len() == 512,
        "frozen input panel differs",
    )?;
    let plan = read(&c.frontier_root.join("plan.json"))?;
    let plan_report = read(&c.frontier_root.join("report.json"))?;
    require(
        plan_report["status"] == "COMPLETED"
            && plan_report["plan_sha256"] == file_hash(&c.frontier_root.join("plan.json"))?
            && plan["schema"] == "uor-r4.reached-frontier-plan/1",
        "frontier plan/report binding differs",
    )?;
    let source_terms = plan["terms"]
        .as_array()
        .ok_or_else(|| bad("frontier terms absent"))?;
    let terms: Vec<Term> = serde_json::from_value(plan["terms"].clone())?;
    require(
        terms.len() == 588
            && terms.iter().filter(|t| t.component == 0).count() == 504
            && terms.iter().filter(|t| t.component == 1).count() == 84,
        "frontier term coverage differs",
    )?;
    let tokens = terms
        .iter()
        .filter(|t| t.component == 0)
        .map(|t| t.target)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    require(
        tokens == vec![617, 2997],
        "failed-entry target census differs",
    )?;
    let objective = read(&c.emission_root.join("native-code-final-objective.json"))?;
    let saved = objective["terms"]
        .as_array()
        .ok_or_else(|| bad("saved final objective terms absent"))?;
    let protected: Vec<SavedProtectedPool> = serde_json::from_slice(&bytes(
        &c.emission_root
            .join("native-candidate-00/emission-reloaded-pools.json"),
    )?)?;
    require(
        saved.len() == 588 && protected.len() == 84,
        "saved pool coverage differs",
    )?;
    let mut rows = Vec::with_capacity(588);
    let mut seen = BTreeSet::new();
    let mut protected_index = 0;
    let mut baseline = [0.0f64; 2];
    let mut cache_bytes = 0u64;
    let mut counts = GenerateReadCounts::default();
    let mut bank_cache = std::collections::BTreeMap::new();
    let codes = tokens
        .iter()
        .map(|&token| {
            (0..8)
                .map(|lane| generator.generate_model().prototypes()[token as usize * 8 + lane])
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    for (i, t) in terms.iter().enumerate() {
        require(
            t.index < 512
                && t.target < 4096
                && t.component < 2
                && t.weight.is_finite()
                && t.weight > 0.0
                && t.position == t.parent_actual_prefix_ids.len()
                && seen.insert((t.index, t.position)),
            "illegal/duplicate frontier term",
        )?;
        require(
            t.component != 0 || t.position == 0,
            "failed frontier is not empty prefix",
        )?;
        let packet = &panel.cases[t.index];
        require(
            plan["canonical_reference"]["rows"][t.index]["id"] == packet.id,
            "frontier/input row identity differs",
        )?;
        if let std::collections::btree_map::Entry::Vacant(entry) = bank_cache.entry(t.index) {
            entry.insert(generator.admit_bank(snapshot(packet)?)?);
        }
        let step = generator.step(&bank_cache[&t.index], &t.parent_actual_prefix_ids)?;
        require(
            saved[i]["term"]["index"] == t.index
                && saved[i]["term"]["position"] == t.position
                && saved[i]["term"]["target"] == t.target
                && saved[i]["term"]["component"] == t.component
                && saved[i]["term"]["weight"] == t.weight
                && saved[i]["term"]["parent_actual_prefix_ids"]
                    == json!(t.parent_actual_prefix_ids)
                && saved[i]["factual_state"]
                    == json!(step
                        .post_state
                        .iter()
                        .map(|v| v.index())
                        .collect::<Vec<_>>())
                && saved[i]["generate_raw_scores_sha256"]
                    == hash(&serde_json::to_vec(&step.generate_raw_scores_q24)?)
                && saved[i]["pool"] == serde_json::to_value(&step.actions.summary)?,
            "actual native state/Generate/full pool differs from saved final objective",
        )?;
        if t.component == 1 {
            let old = &protected[protected_index];
            require(
                old.term == saved[i]["term"]
                    && old.copy_ids == step.copy_token_ids
                    && old.copy_q24 == step.copy_raw_scores_q24
                    && old.generate_q24 == step.generate_raw_scores_q24
                    && old.token_masses.len() == step.actions.token_masses.len()
                    && old
                        .token_masses
                        .iter()
                        .zip(&step.actions.token_masses)
                        .all(|(a, b)| {
                            a.token_id == b.token_id
                                && a.weight_q31 == b.weight_q31
                                && a.generate_weight_q31 == b.generate_weight_q31
                                && a.copy_weight_q31 == b.copy_weight_q31
                        })
                    && step.actions.summary.chosen_token_id == t.target,
                "actual protected complete pool differs",
            )?;
            protected_index += 1;
        }
        let cache = reducer.prepare_generate_patch_cache(
            step.generate_raw_scores_q24.clone(),
            step.copy_token_ids.clone(),
            step.copy_raw_scores_q24.clone(),
        )?;
        require(
            cache.summary().chosen_token_id == step.actions.summary.chosen_token_id
                && cache.summary().chosen_weight_q31 == step.actions.summary.chosen_weight_q31
                && cache.summary().reference_q24 == step.actions.summary.max_score_q24
                && cache.summary().total_weight_q31 == step.actions.summary.total_weight_q31,
            "target-free patch cache differs from factual reducer",
        )?;
        let target_mass = cache.token_masses()[t.target as usize];
        require(
            target_mass > 0
                && saved[i]["target_mass"] == target_mass
                && saved[i]["denominator"] == cache.summary().total_weight_q31,
            "native cached objective masses differ",
        )?;
        let ce = -(target_mass as f64 / cache.summary().total_weight_q31 as f64).ln();
        baseline[t.component] += t.weight * ce;
        let mut conditional = Vec::with_capacity(1920);
        for (token_index, &token) in tokens.iter().enumerate() {
            for lane in 0..8 {
                let mut scores = [0i64; 120];
                generator.generate_model().code_conditional_scores_into(
                    &step.post_state,
                    lane,
                    token as usize,
                    &mut scores,
                    &mut counts,
                )?;
                require(
                    scores[codes[token_index][lane] as usize]
                        == cache.generate_scores()[token as usize],
                    "current prototype conditional score is not factual",
                )?;
                conditional.extend_from_slice(&scores);
            }
        }
        cache_bytes = cache_bytes
            .checked_add(
                (4096 * 24 + conditional.len() * 8 + step.copy_token_ids.len() * 12) as u64,
            )
            .ok_or_else(|| bad("cache byte projection overflow"))?;
        require(
            cache_bytes <= c.maximum_cache_bytes,
            "auxiliary cache projection exceeds admitted cap",
        )?;
        write(
            c,
            &format!("pool-{i:04}.json"),
            &json!({"term":source_terms[i],"id":packet.id,"post_state":step.post_state.iter().map(|v|v.index()).collect::<Vec<_>>(),
            "generate_q24":cache.generate_scores(),"copy_ids":cache.copy_token_ids(),"copy_q24":cache.copy_scores(),"token_masses":cache.token_masses(),"summary":cache.summary(),
            "conditional_scores_q24":conditional,"conditional_order":"ascending census token then lane0..7 then code0..119",
            "conditional_scores_sha256":hash(&serde_json::to_vec(&conditional)?),"census_token_ids":tokens,"current_codes":codes}),
            written,
        )?;
        let summary = cache.summary();
        rows.push(Row {
            cache,
            target: t.target,
            component: t.component,
            weight: t.weight,
            conditional,
            baseline_gap: (summary.chosen_weight_q31 as f64 / target_mass as f64).ln(),
            baseline_correct: summary.chosen_token_id == t.target,
        });
    }
    require(
        protected_index == 84
            && rows
                .iter()
                .filter(|r| r.component == 0 && r.baseline_correct)
                .count()
                == 0,
        "baseline entry/protected census differs",
    )?;
    for (component, key) in [(0, "task"), (1, "reference")] {
        let expected = objective[key]
            .as_f64()
            .ok_or_else(|| bad("saved objective absent"))?;
        require(
            (baseline[component] - expected).abs() <= 1e-10 * (1.0 + expected.abs()),
            "full captured baseline objective differs",
        )?;
    }
    drop(bank_cache);
    drop(panel);
    drop(protected);
    let capture_seconds = clock.elapsed().as_secs_f64();
    let census_clock = Instant::now();
    let mut triples = Vec::with_capacity(1920);
    let mut noops = 0;
    let mut positive = 0;
    let mut fallbacks = 0usize;
    for (token_index, &token) in tokens.iter().enumerate() {
        for lane in 0..8 {
            for code in 0..120 {
                let noop = code == codes[token_index][lane] as usize;
                noops += usize::from(noop);
                let mut loss = [0.0f64; 2];
                let mut failed = Vec::new();
                let mut gained = Vec::new();
                let mut gap_improved = 0;
                let mut gap_worsened = 0;
                let mut gap_tied = 0;
                let mut row_results = Vec::with_capacity(588);
                let mut changed_rows = 0;
                let mut triple_fallbacks = 0;
                for (i, row) in rows.iter().enumerate() {
                    let new_score = row.conditional[(token_index * 8 + lane) * 120 + code];
                    changed_rows +=
                        usize::from(new_score != row.cache.generate_scores()[token as usize]);
                    let patch =
                        reducer.evaluate_generate_patch(&row.cache, &[(token, new_score)])?;
                    let summary = patch.summary();
                    let mass = patch.token_mass(&row.cache, row.target)?;
                    require(
                        mass > 0 && summary.total_weight_q31 >= mass,
                        "candidate full support differs",
                    )?;
                    triple_fallbacks += usize::from(summary.used_full_reduction);
                    let ce = -(mass as f64 / summary.total_weight_q31 as f64).ln();
                    loss[row.component] += row.weight * ce;
                    let correct = summary.chosen_token_id == row.target;
                    if row.component == 1 && !correct {
                        failed.push(i);
                    }
                    if row.component == 0 && correct && !row.baseline_correct {
                        gained.push(i);
                    }
                    let gap = (summary.chosen_weight_q31 as f64 / mass as f64).ln();
                    let delta = gap - row.baseline_gap;
                    if row.component == 0 {
                        if delta < 0.0 {
                            gap_improved += 1;
                        } else if delta > 0.0 {
                            gap_worsened += 1;
                        } else {
                            gap_tied += 1;
                        }
                    }
                    if noop {
                        require(
                            new_score == row.cache.generate_scores()[token as usize]
                                && summary.chosen_token_id == row.cache.summary().chosen_token_id
                                && summary.total_weight_q31 == row.cache.summary().total_weight_q31
                                && mass == row.cache.token_masses()[row.target as usize],
                            "no-op does not reproduce complete factual pool",
                        )?;
                    }
                    row_results.push(json!({"row":i,"target_mass":mass,"denominator":summary.total_weight_q31,"winner":summary.chosen_token_id,
                "winner_mass":summary.chosen_weight_q31,"reference_q24":summary.reference_q24,"correct":correct,"ce":ce,"log_winner_target_gap_delta":delta}));
                }
                let preserved = failed.is_empty();
                let is_positive = preserved && !gained.is_empty();
                positive += usize::from(is_positive);
                fallbacks += triple_fallbacks;
                let combined = loss[0] + loss[1];
                let summary = json!({"token":token,"lane":lane,"code":code,"current_code":codes[token_index][lane],"noop":noop,
            "rows":588,"changed_score_rows":changed_rows,"protected_failures":failed,"newly_correct_failed_entries":gained,
            "protected_all_preserved":preserved,"positive_neighborhood_witness":is_positive,"task":loss[0],"reference":loss[1],"combined":combined,
            "combined_delta":combined-baseline.iter().sum::<f64>(),"ce_descends":combined<baseline.iter().sum::<f64>()-1e-10*(1.0+baseline.iter().sum::<f64>().abs()),
            "failed_gap_counts":{"improved":gap_improved,"worsened":gap_worsened,"tied":gap_tied},"full_reduction_fallbacks":triple_fallbacks});
                write(
                    c,
                    &format!("triple-{token}-{lane}-{code:03}.json"),
                    &json!({"summary":summary,"row_results":row_results}),
                    written,
                )?;
                triples.push(summary);
            }
        }
    }
    require(
        triples.len() == 1920 && noops == 16,
        "complete shared prototype neighborhood coverage differs",
    )?;
    Ok(
        json!({"schema":"uor-r4.native-generate-prototype-neighborhood/1","status":"COMPLETED","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),
        "fixed_objective_terms":588,"failed_frontier_terms":504,"protected_terms":84,"target_token_census":tokens,"triples":triples,
        "noop_controls":noops,"positive_neighborhood_witnesses":positive,"baseline":{"task":baseline[0],"reference":baseline[1],"combined":baseline.iter().sum::<f64>()},
        "capture_seconds":capture_seconds,"census_seconds":census_clock.elapsed().as_secs_f64(),"elapsed_seconds":clock.elapsed().as_secs_f64(),
        "conditional_scorer_counts":counts,"prototype_codes_sha256":hash(generator.generate_model().prototypes()),"native_generate_metadata":generator.generate_model().metadata(),"auxiliary_cache_array_bytes":cache_bytes,"full_reduction_fallbacks":fallbacks,"report_payload_bytes_before_final":written,
        "identity":{"report_sha256":c.expected_report_sha256,"manifest_sha256":c.expected_manifest_sha256,"source_metadata_sha256":binding.metadata_sha256,
            "generate_sha256":c.expected_generate_sha256,"continuation_sha256":c.expected_continuation_sha256,"inputs_sha256":c.expected_inputs_sha256,
            "frontier_report_sha256":c.expected_frontier_report_sha256,"frontier_manifest_sha256":c.expected_frontier_manifest_sha256,"plan_sha256":file_hash(&c.frontier_root.join("plan.json"))?,
            "config_sha256":file_hash(&c.output.join("config.json"))?,"saved_final_objective_sha256":file_hash(&c.emission_root.join("native-code-final-objective.json"))?,"numerical_U_zero":true},
        "scope":"exposed fixed-state finite shared prototype neighborhood; positive means all84 pooled winners retained and at least1 previously failed empty-prefix entry corrected; CE descent separate; no candidate exported, no own-prefix complete reply, global capacity, transfer/chat or efficiency qualification"}),
    )
}
fn main() -> Result<()> {
    let argv = std::env::args().collect::<Vec<_>>();
    if argv.len() == 3 && argv[1] == "verify-report" {
        report_output::verify(Path::new(&argv[2]))?;
        return Ok(());
    }
    require(
        argv.len() == 2,
        "usage: native-generate-prototype-neighborhood CONFIG.json",
    )?;
    let config_bytes = bytes(Path::new(&argv[1]))?;
    let mut c: Config = serde_json::from_slice(&config_bytes)?;
    admit_paths(&mut c)?;
    report_output::claim(&c.output)?;
    let mut written = 0;
    write(
        &c,
        "config.json",
        &serde_json::from_slice(&config_bytes)?,
        &mut written,
    )?;
    let outcome = run(&c, &mut written);
    let report = match &outcome {
        Ok(v) => v.clone(),
        Err(e) => {
            json!({"schema":"uor-r4.native-generate-prototype-neighborhood/1","status":"FAILED","error":e.to_string(),
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"scope":"execution/admission failure; no model-quality verdict"})
        }
    };
    let final_bytes = serde_json::to_vec_pretty(&report)?;
    if outcome.is_ok()
        && written
            .checked_add(final_bytes.len() as u64)
            .map_or(true, |n| n > c.maximum_report_bytes)
    {
        fs::write(
            c.output.join("report.json"),
            serde_json::to_vec(
                &json!({"status":"FAILED","error":"final report exceeds admitted byte cap","scope":"execution failure; no model-quality verdict"}),
            )?,
        )?;
        report_output::seal(&c.output)?;
        report_output::verify(&c.output)?;
        return Err(bad("final report exceeds admitted byte cap"));
    }
    fs::write(c.output.join("report.json"), final_bytes)?;
    report_output::seal(&c.output)?;
    report_output::verify(&c.output)?;
    outcome.map(|_| ())
}
