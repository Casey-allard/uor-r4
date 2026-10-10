//! Explicit import of the completed #2101 native-anchored derivative population.
//! This is neither legacy failed-run recovery nor permission to recompute derivatives.
use super::*;
const SOURCE: &str = "2c31a22e6fbef3bd37dade8cbb7daae79f13876a";
const REPORT: &str = "b1d27f7b15e65d0aa8ea7e0a995b8bc76979610f97dbcb3055c8bbb434d0ac9b";
const MANIFEST: &str = "cdadd7c5cbbbdca2225a71006ab58f70129e8df9e015b92f35b19359028a7167";
const BINARY: &str = "1c5af0fb13b4d957b5665037ca437486985a882260882dca5c07c82d4fdbc848";
const CONFIG: &str = "8af92899f3aff976ed2acd030837767f9f65827c03b4ea3e559594f759d501b0";
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SavedProtectedCredit {
    pub contract_version: u32,
    pub root: PathBuf,
    pub runtime_root: PathBuf,
    pub observation_root: PathBuf,
    pub expected_runtime_identity_sha256: String,
    pub expected_observation_manifest_sha256: String,
}
impl SavedProtectedCredit {
    pub(super) fn input_roots(&self) -> Vec<PathBuf> {
        vec![
            self.root.clone(),
            self.runtime_root.clone(),
            self.observation_root.clone(),
        ]
    }
    pub(super) fn validate(&self) -> Result<()> {
        replay_require(
            self.contract_version == 1
                && [
                    &self.expected_runtime_identity_sha256,
                    &self.expected_observation_manifest_sha256,
                ]
                .iter()
                .all(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())),
            "saved protected-credit contract/pins invalid",
        )
    }
}
// Normalize only the declared relocatable input paths. Every other typed field,
// including every report/manifest/row hash, remains part of exact equality.
fn relocated_inputs(config: &prefix::Config) -> Result<Value> {
    let mut v = serde_json::to_value(config)?;
    for key in ["retained_intermediate_root", "retained_probe_root"] {
        if let Some(p) = v.get_mut(key) {
            *p = json!("RELOCATED_INPUT");
        }
    }
    if let Some(e) = v.get_mut("episode").filter(|x| x.is_object()) {
        for key in ["typed_authority", "retained_supplement_root"] {
            if let Some(p) = e.get_mut(key) {
                *p = json!("RELOCATED_INPUT");
            }
        }
        if let Some(p) = e
            .get_mut("retained_projection")
            .and_then(|x| x.get_mut("root"))
        {
            *p = json!("RELOCATED_INPUT");
        }
        if let Some(phases) = e.get_mut("phases").and_then(Value::as_array_mut) {
            for phase in phases {
                if let Some(p) = phase.get_mut("capture").and_then(|x| x.get_mut("root")) {
                    *p = json!("RELOCATED_INPUT");
                }
            }
        }
    }
    Ok(v)
}
// Permit only the authenticated phase's declared capture-root relocation.
// Scientific values and every other provenance field remain exact JSON values.
fn verify_relocated_witness(
    old: &Value,
    current: &Value,
    old_root: &Path,
    current_root: &Path,
) -> Result<()> {
    replay_require(
        old.pointer("/original_derivation/capture_root") == Some(&json!(old_root))
            && current.pointer("/original_derivation/capture_root") == Some(&json!(current_root)),
        "normalized witness capture root is not the declared input authority",
    )?;
    let mut aligned = current.clone();
    let slot = aligned
        .pointer_mut("/original_derivation/capture_root")
        .ok_or_else(|| bad("normalized witness capture-root provenance absent"))?;
    *slot = json!(old_root);
    replay_require(
        &aligned == old,
        "normalized witness scientific/provenance values differ beyond declared capture root",
    )
}
fn copy_file(a: &Args, source: &Path, leaf: &str) -> Result<Value> {
    validate_inherited_leaf(leaf)?;
    let bytes = fs::read(source)?;
    let dest = a.out.join(leaf);
    if dest.exists() {
        replay_require(
            fs::read(&dest)? == bytes,
            "saved-credit existing scientific copy differs",
        )?;
    } else {
        use std::io::Write;
        let mut out = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&dest)?;
        out.write_all(&bytes)?;
    }
    Ok(json!({"source_file":source,"file":leaf,"bytes":bytes.len(),"sha256":sha256_bytes(&bytes)}))
}
pub(super) fn verify_copies(a: &Args, inherited: &Value) -> Result<()> {
    let copies = inherited["copied_files"]
        .as_array()
        .ok_or_else(|| bad("saved-credit copy inventory missing"))?;
    for e in copies {
        let leaf = e["file"]
            .as_str()
            .ok_or_else(|| bad("saved-credit copy leaf missing"))?;
        validate_inherited_leaf(leaf)?;
        let p = a.out.join(leaf);
        replay_require(
            sha256_file(&p)?
                == e["sha256"]
                    .as_str()
                    .ok_or_else(|| bad("saved-credit copy SHA missing"))?
                && fs::metadata(&p)?.len()
                    == e["bytes"]
                        .as_u64()
                        .ok_or_else(|| bad("saved-credit copy size missing"))?,
            "saved-credit scientific copy changed after preparation",
        )?;
    }
    if let Some(relocations) = inherited["witness_relocations"].as_array() {
        for r in relocations {
            let old_leaf = r["producer_file"]
                .as_str()
                .ok_or_else(|| bad("relocated producer witness absent"))?;
            let new_leaf = r["file"]
                .as_str()
                .ok_or_else(|| bad("relocated current witness absent"))?;
            validate_inherited_leaf(old_leaf)?;
            validate_inherited_leaf(new_leaf)?;
            let old = read(&a.out.join(old_leaf))?;
            let current = read(&a.out.join(new_leaf))?;
            verify_relocated_witness(
                &old,
                &current,
                Path::new(
                    r["producer_capture_root"]
                        .as_str()
                        .ok_or_else(|| bad("producer capture root absent"))?,
                ),
                Path::new(
                    r["current_capture_root"]
                        .as_str()
                        .ok_or_else(|| bad("current capture root absent"))?,
                ),
            )?;
        }
    }
    Ok(())
}
fn decode_jacobian(bytes: &[u8]) -> Result<Vec<f32>> {
    replay_require(bytes.len() == 1920 * 4, "saved Jacobian shape differs")?;
    let values = bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect::<Vec<_>>();
    replay_require(
        values.iter().all(|v| v.is_finite()),
        "saved Jacobian nonfinite",
    )?;
    Ok(values)
}
/// Call after original391 assembly and before any proposal. This closes the
/// relocated-witness contract against current, independently prepared native pools.
pub(super) fn verify_population(
    authority: &SavedProtectedCredit,
    frames: &[shared::Frame],
    pools: &[shared::Pool],
) -> Result<()> {
    replay_require(
        frames.len() == 391 && pools.len() == 391,
        "saved-credit current391 population incomplete",
    )?;
    let population = read(&authority.root.join("coupled-population.json"))?;
    let rows = population["rows"]
        .as_array()
        .ok_or_else(|| bad("saved-credit population rows absent"))?;
    replay_require(
        rows.len() == 391,
        "saved-credit producer population incomplete",
    )?;
    for (i, ((f, p), row)) in frames.iter().zip(pools).zip(rows).enumerate() {
        let keys = f
            .cue_keys
            .iter()
            .flat_map(|k| k.iter().flatten().copied())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        replay_require(
            row["row"] == i
                && row["input_index"] == f.input
                && row["position"] == f.position
                && row["id"] == f.id
                && row["actual_prefix_ids"] == json!(f.prefix)
                && row["target_label_only"] == f.target
                && row["donor"] == p.donor
                && row["post_state"] == json!(p.post.iter().map(|x| x.index()).collect::<Vec<_>>())
                && row["prefix_key_union"] == json!(keys),
            "saved-credit current original native row differs",
        )?;
        if i < 380 {
            let bytes = fs::read(
                authority
                    .root
                    .join(format!("protected-original-masses-{i:03}.u64le")),
            )?;
            let mut masses = vec![0u64; 4096];
            for m in &p.trace.token_masses {
                masses[m.token_id as usize] = m.weight_q31;
            }
            replay_require(
                bytes
                    == masses
                        .iter()
                        .flat_map(|x| x.to_le_bytes())
                        .collect::<Vec<_>>(),
                "saved-credit current protected original full masses differ",
            )?;
        }
    }
    Ok(())
}
pub(super) fn load(
    a: &Args,
    authority: &SavedProtectedCredit,
    frames: &[shared::Frame],
) -> Result<(Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Value)> {
    authority.validate()?;
    report_output::verify(&authority.root)?;
    replay_require(
        sha256_file(&authority.root.join("report.json"))? == REPORT
            && sha256_file(&authority.root.join("manifest.json"))? == MANIFEST
            && sha256_file(&authority.root.join("config.json"))? == CONFIG,
        "saved #2101 completed report/manifest/config differs",
    )?;
    let report = read(&authority.root.join("report.json"))?;
    replay_require(
        report["status"] == "COMPLETED"
            && report["source_commit"] == SOURCE
            && report["new_backward_calls"] == 411
            && report["protected_margin_backward_calls"] == 380,
        "saved completed protected producer scope differs",
    )?;
    let cfg = read(&authority.root.join("config.json"))?;
    let old = &cfg["coupled_episode_learning"];
    replay_require(
        old["donor_credit"] == "full_pool_utility"
            && old["prefix_transaction"] == "protected_joint_vector"
            && old["retained_gradient"].is_null()
            && old["retained_export"].is_null(),
        "saved producer credit policy differs",
    )?;
    let original = ContinuationParent::from_checkpoint(&a.checkpoint)?;
    replay_require(
        original.binding.metadata_sha256 == shared::SOURCE
            && sha256_bytes(&original.generate) == shared::G_SHA
            && sha256_file(&a.checkpoint.join("continuation-field.bin"))? == shared::U_SHA,
        "saved-credit original frozen Source/Generate/U epoch differs",
    )?;
    let expected: prefix::Config = serde_json::from_value(old["original_inputs"].clone())?;
    let current = &a
        .coupled_episode_learning
        .as_ref()
        .ok_or_else(|| bad("saved-credit coupled config missing"))?
        .original_inputs;
    replay_require(
        relocated_inputs(&expected)? == relocated_inputs(current)?,
        "saved-credit original input authorities differ",
    )?;
    report_output::verify(&authority.observation_root)?;
    replay_require(
        sha256_file(&authority.observation_root.join("manifest.json"))?
            == authority.expected_observation_manifest_sha256,
        "saved observation manifest differs",
    )?;
    let runtime_path = authority.runtime_root.join("runtime-identity.json");
    replay_require(
        sha256_file(&runtime_path)? == authority.expected_runtime_identity_sha256,
        "saved runtime identity differs",
    )?;
    let runtime = read(&runtime_path)?;
    let launch = read(&authority.observation_root.join("launch.json"))?;
    let execution = read(&authority.observation_root.join("execution.json"))?;
    let attempt = read(&authority.root.join("attempt.json"))?;
    let external = read(&authority.root.join("external-config-binding.json"))?;
    replay_require(
        runtime["source_commit"] == SOURCE
            && runtime["binary_sha256"] == BINARY
            && runtime["config_sha256"] == CONFIG
            && sha256_file(&authority.runtime_root.join("geometric-frozen-map-fit"))? == BINARY
            && execution["exit_code"] == 0
            && launch["binary_sha256"] == BINARY
            && execution["binary_sha256"] == BINARY
            && launch["config_sha256"] == CONFIG
            && execution["config_sha256"] == CONFIG
            && launch["argv"] == attempt["argv"]
            && launch["pid"] == attempt["pid"]
            && launch["started_utc"] == execution["started_utc"]
            && external["sha256"] == CONFIG
            && external["attempt_argv"] == attempt["argv"],
        "saved producer actual runtime/launch binding differs",
    )?;
    let forward = read(&authority.root.join("coupled-forward-parity.json"))?;
    let receipt = read(&authority.root.join("coupled-gradient-receipt.json"))?;
    replay_require(
        forward["all_before_any_backward"] == true
            && forward["physical_frames"] == 31
            && forward["raw_G_Copy_U_donor_post_full_alias_pool"] == true
            && receipt["physical_backward_calls"] == 31
            && receipt["weighted_roles"] == 32
            && receipt["protected_margin_backward_calls"] == 380
            && receipt["total_fresh_backward_calls"] == 411
            && receipt["donor_credit"] == "full_pool_utility"
            && receipt["prefix_transaction"] == "protected_joint_vector"
            && receipt["extracted_families"] == json!([PREFIX, GENERATE])
            && frames.len() == 31,
        "saved objective derivative/parity scope differs",
    )?;
    let terms = receipt["perterm"]
        .as_array()
        .ok_or_else(|| bad("retained perterm gradients absent"))?;
    replay_require(
        terms.len() == 31,
        "retained weighted physical terms incomplete",
    )?;
    let mut ps = vec![0f32; COUNT];
    let mut gs = vec![0f32; COUNT];
    let mut copied = Vec::new();
    let mut seen = BTreeSet::new();
    for (i, (term, f)) in terms.iter().zip(frames).enumerate() {
        replay_require(
            term["physical_index"] == i
                && term["input_index"] == f.input
                && term["position"] == f.position
                && term["target"] == f.target
                && term["weight"].as_f64() == Some(f.weight),
            "retained weighted term identity differs",
        )?;
        let families = term["families"]
            .as_array()
            .ok_or_else(|| bad("retained term family arrays absent"))?;
        replay_require(families.len() == 2, "retained term both families missing")?;
        for (name, sum) in [(PREFIX, &mut ps), (GENERATE, &mut gs)] {
            let e = families
                .iter()
                .find(|e| e["family"] == name)
                .ok_or_else(|| bad("retained term family missing"))?;
            let leaf = e["file"]
                .as_str()
                .ok_or_else(|| bad("retained raw gradient file missing"))?;
            replay_require(
                e["shape"] == json!([960])
                    && e["bytes"] == 3840
                    && e["status"] == "PRESENT"
                    && e["missing_gradient_filled_zero"] == false
                    && seen.insert(leaf.to_owned()),
                "retained raw family receipt invalid",
            )?;
            validate_inherited_leaf(leaf)?;
            let source = authority.root.join(leaf);
            replay_require(
                sha256_file(&source)?
                    == e["sha256"]
                        .as_str()
                        .ok_or_else(|| bad("raw gradient hash absent"))?,
                "retained raw gradient hash differs",
            )?;
            let values = shared::floats(&source)?;
            replay_require(
                values.len() == COUNT && values.iter().all(|v| v.is_finite()),
                "retained raw gradient shape/nonfinite",
            )?;
            replay_require(
                e["all_zero"] == values.iter().all(|v| *v == 0.),
                "retained zero gradient presence differs",
            )?;
            for (a, b) in sum.iter_mut().zip(values) {
                *a += b;
            }
            copied.push(copy_file(a, &source, leaf)?);
        }
    }
    let files = receipt["files"]
        .as_array()
        .ok_or_else(|| bad("retained aggregate/master files absent"))?;
    replay_require(
        files.len() == 4,
        "retained aggregate/master inventory differs",
    )?;
    let mut loaded: BTreeMap<(String, String), Vec<f32>> = BTreeMap::new();
    for e in files {
        let family = e["family"]
            .as_str()
            .ok_or_else(|| bad("retained aggregate family absent"))?;
        let kind = e["kind"]
            .as_str()
            .ok_or_else(|| bad("retained aggregate kind absent"))?;
        let leaf = e["file"]
            .as_str()
            .ok_or_else(|| bad("retained aggregate filename absent"))?;
        replay_require(
            [PREFIX, GENERATE].contains(&family)
                && ["gradient", "initial-master"].contains(&kind)
                && e["shape"] == json!([960])
                && e["bytes"] == 3840
                && seen.insert(leaf.to_owned()),
            "retained aggregate/master receipt invalid",
        )?;
        validate_inherited_leaf(leaf)?;
        let source = authority.root.join(leaf);
        replay_require(
            sha256_file(&source)?
                == e["sha256"]
                    .as_str()
                    .ok_or_else(|| bad("aggregate hash absent"))?,
            "retained aggregate/master hash differs",
        )?;
        let values = shared::floats(&source)?;
        replay_require(
            values.len() == COUNT && values.iter().all(|v| v.is_finite()),
            "retained aggregate/master shape/nonfinite",
        )?;
        replay_require(
            loaded
                .insert((family.into(), kind.into()), values)
                .is_none(),
            "retained duplicate aggregate family",
        )?;
        copied.push(copy_file(a, &source, leaf)?);
    }
    let take = |family: &str, kind: &str| {
        loaded
            .get(&(family.to_owned(), kind.to_owned()))
            .cloned()
            .ok_or_else(|| bad("retained aggregate/master family absent"))
    };
    let pm = take(PREFIX, "initial-master")?;
    let pg = take(PREFIX, "gradient")?;
    let gm = take(GENERATE, "initial-master")?;
    let gg = take(GENERATE, "gradient")?;
    replay_require(
        pg.iter().zip(&ps).all(|(a, b)| a.to_bits() == b.to_bits())
            && gg.iter().zip(&gs).all(|(a, b)| a.to_bits() == b.to_bits()),
        "retained ordered f32 perterm sums differ",
    )?;
    let pbytes = fs::read(a.checkpoint.join("prefix/prefix-source-f32.bin"))?;
    let gbytes = fs::read(a.checkpoint.join("generate-source/generate.unary.f32le"))?;
    replay_require(
        pm.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>() == pbytes
            && gm.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>() == gbytes,
        "retained gradient original fractional master bits differ",
    )?;

    let mut witness_relocations = Vec::new();
    for entry in fs::read_dir(&authority.root)? {
        let e = entry?;
        let leaf = e.file_name().to_string_lossy().into_owned();
        if leaf.starts_with("original-") && leaf.ends_with(".json") {
            let current_path = a.out.join(&leaf);
            let old_bytes = fs::read(e.path())?;
            let current_bytes = fs::read(&current_path)?;
            if old_bytes == current_bytes {
                copied.push(copy_file(a, &e.path(), &leaf)?);
            } else {
                let old_episode = expected
                    .episode
                    .as_ref()
                    .ok_or_else(|| bad("saved original episode absent"))?;
                let current_episode = current
                    .episode
                    .as_ref()
                    .ok_or_else(|| bad("current original episode absent"))?;
                let phase = old_episode
                    .phases
                    .iter()
                    .find(|p| {
                        p.original_prefix_inverse
                            && leaf == format!("original-joint-phase-{:02}.json", p.position)
                    })
                    .ok_or_else(|| {
                        bad("normalized witness mismatch is not a declared inverse phase")
                    })?;
                let current_phase = current_episode
                    .phases
                    .iter()
                    .find(|p| p.position == phase.position && p.original_prefix_inverse)
                    .ok_or_else(|| bad("relocated inverse phase authority missing"))?;
                verify_relocated_witness(
                    &serde_json::from_slice(&old_bytes)?,
                    &serde_json::from_slice(&current_bytes)?,
                    &phase.capture.root,
                    &current_phase.capture.root,
                )?;
                let preserved = format!("imported-producer-{leaf}");
                copied.push(copy_file(a, &e.path(), &preserved)?);
                // Keep the current preparer's witness at its normal filename.
                copied.push(copy_file(a, &current_path, &leaf)?);
                witness_relocations.push(json!({"file":leaf,"producer_file":preserved,"position":phase.position,"producer_capture_root":phase.capture.root,"current_capture_root":current_phase.capture.root,"producer_sha256":sha256_bytes(&old_bytes),"current_sha256":sha256_bytes(&current_bytes),"equivalence":"all parsed scientific/provenance values exact except declared original_derivation.capture_root"}));
            }
        }
    }
    let margins = read(&authority.root.join("protected-margin-receipt.json"))?;
    replay_require(
        margins["policy"] == protected_joint_vector::policy()
            && margins["backward_calls"] == 380
            && margins["all380_parity_before_any_backward"] == true
            && margins["protected_weight"] == 1
            && margins["objective_guard_CE_weight"] == 0
            && receipt["protected_margin_receipt_sha256"]
                == sha256_file(&authority.root.join("protected-margin-receipt.json"))?,
        "saved protected derivative policy differs",
    )?;
    let terms = margins["terms"]
        .as_array()
        .ok_or_else(|| bad("saved protected terms absent"))?;
    let population = read(&authority.root.join("coupled-population.json"))?;
    let rows = population["rows"]
        .as_array()
        .ok_or_else(|| bad("saved population absent"))?;
    replay_require(
        terms.len() == 380 && rows.len() == 391,
        "saved protected population shape differs",
    )?;
    for (i, t) in terms.iter().enumerate() {
        let row = &rows[i];
        replay_require(
            t["guard_index"] == i
                && ["input_index", "position", "id", "actual_prefix_ids"]
                    .iter()
                    .all(|k| t[*k] == row[*k])
                && t["winner"] == row["target_label_only"]
                && t["shape"] == json!([1920])
                && t["status"] == "PRESENT",
            "saved protected row identity differs",
        )?;
        let jl = format!("protected-margin-{i:03}.f32le");
        let ml = format!("protected-original-masses-{i:03}.u64le");
        let ul = format!("protected-donor-margin-{i:03}.json");
        for (leaf, fk, hk) in [
            (&jl, "file", "sha256"),
            (&ml, "original_masses_file", "original_masses_sha256"),
            (&ul, "utility_file", "utility_sha256"),
        ] {
            replay_require(
                t[fk] == leaf.as_str() && t[hk] == sha256_file(&authority.root.join(leaf))?,
                "saved protected file identity differs",
            )?;
            copied.push(copy_file(a, &authority.root.join(leaf), leaf)?);
        }
        let j = decode_jacobian(&fs::read(authority.root.join(&jl))?)?;
        replay_require(
            t["all_zero"] == j.iter().all(|v| *v == 0.),
            "saved protected zero/presence differs",
        )?;
        let bytes = fs::read(authority.root.join(&ml))?;
        replay_require(
            bytes.len() == 4096 * 8,
            "saved original mass array shape differs",
        )?;
        let masses = bytes
            .chunks_exact(8)
            .map(|b| u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]))
            .collect::<Vec<_>>();
        let w = t["winner"]
            .as_u64()
            .filter(|x| *x < 4096)
            .ok_or_else(|| bad("saved winner invalid"))? as usize;
        let r = t["rival"]
            .as_u64()
            .filter(|x| *x < 4096 && *x != w as u64)
            .ok_or_else(|| bad("saved rival invalid"))? as usize;
        let strongest = (0..4096)
            .filter(|x| *x != w)
            .min_by(|a, b| masses[*b].cmp(&masses[*a]).then(a.cmp(b)))
            .ok_or_else(|| bad("saved rival missing"))?;
        let winner = (0..4096)
            .min_by(|a, b| masses[*b].cmp(&masses[*a]).then(a.cmp(b)))
            .ok_or_else(|| bad("saved winner missing"))?;
        replay_require(
            winner == w
                && strongest == r
                && masses[w] > 0
                && masses[r] > 0
                && t["winner_mass"] == masses[w]
                && t["rival_mass"] == masses[r],
            "saved original winner/strongest-rival masses differ",
        )?;
    }
    replay_require(
        receipt["full_pool_utility_file"] == "coupled-full-pool-donor-utilities.json"
            && receipt["full_pool_utility_sha256"]
                == sha256_file(
                    &authority
                        .root
                        .join("coupled-full-pool-donor-utilities.json"),
                )?,
        "saved objective donor-utility receipt differs",
    )?;
    for leaf in [
        "coupled-gradient-receipt.json",
        "coupled-forward-parity.json",
        "protected-margin-receipt.json",
        "protected-forward-parity.json",
        "coupled-full-pool-donor-utilities.json",
    ] {
        copied.push(copy_file(a, &authority.root.join(leaf), leaf)?);
    }
    for (leaf, dest) in [
        ("report.json", "imported-producer-report.json"),
        ("manifest.json", "imported-producer-manifest.json"),
        ("config.json", "imported-producer-config.json"),
        (
            "coupled-population.json",
            "imported-producer-population.json",
        ),
        (
            "coupled-construction.json",
            "imported-producer-construction.json",
        ),
    ] {
        copied.push(copy_file(a, &authority.root.join(leaf), dest)?);
    }
    for (source, dest) in [
        (
            authority.runtime_root.join("runtime-identity.json"),
            "imported-producer-runtime-identity.json",
        ),
        (
            authority.observation_root.join("manifest.json"),
            "imported-producer-observation-manifest.json",
        ),
        (
            authority.observation_root.join("launch.json"),
            "imported-producer-launch.json",
        ),
        (
            authority.observation_root.join("execution.json"),
            "imported-producer-execution.json",
        ),
        (
            authority.root.join("attempt.json"),
            "imported-producer-attempt.json",
        ),
        (
            authority.root.join("external-config-binding.json"),
            "imported-producer-external-config-binding.json",
        ),
    ] {
        copied.push(copy_file(a, &source, dest)?);
    }
    let inheritance = json!({"schema":"uor-r4.saved-protected-credit-import/1","authority":authority,"source_commit":SOURCE,"binary_sha256":BINARY,"config_sha256":CONFIG,"report_sha256":REPORT,"manifest_sha256":MANIFEST,"inherited_objective_backward_calls":31,"inherited_protected_backward_calls":380,"new_training_graph_forwards":0,"new_backward_calls":0,"copied_files":copied,"witness_relocations":witness_relocations});
    verify_copies(a, &inheritance)?;
    write(a, "saved-protected-credit-import.json", &inheritance)?;
    Ok((pm, pg, gm, gg, inheritance))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inverse_witness_accepts_only_exact_declared_capture_relocation() {
        let old = json!({"original_derivation":{"capture_root":"/old/capture","method":"audited inverse","sha":"fixed"},"pool":{"mass":123},"copy":[4,4]});
        let mut current = old.clone();
        current["original_derivation"]["capture_root"] = json!("/new/capture");
        assert!(verify_relocated_witness(
            &old,
            &current,
            Path::new("/old/capture"),
            Path::new("/new/capture")
        )
        .is_ok());
        assert!(verify_relocated_witness(
            &old,
            &current,
            Path::new("/wrong"),
            Path::new("/new/capture")
        )
        .is_err());
        assert!(verify_relocated_witness(
            &old,
            &current,
            Path::new("/old/capture"),
            Path::new("/wrong")
        )
        .is_err());
        current["pool"]["mass"] = json!(124);
        assert!(verify_relocated_witness(
            &old,
            &current,
            Path::new("/old/capture"),
            Path::new("/new/capture")
        )
        .is_err());
        current["pool"]["mass"] = json!(123);
        current["original_derivation"]["sha"] = json!("changed");
        assert!(verify_relocated_witness(
            &old,
            &current,
            Path::new("/old/capture"),
            Path::new("/new/capture")
        )
        .is_err());
    }
    #[test]
    fn relocation_normalizes_only_named_locations_and_keeps_identity_fields() {
        let a: prefix::Config = serde_json::from_value(
            json!({"retained_intermediate_root":"old/i","retained_probe_root":"old/p"}),
        )
        .unwrap();
        let b: prefix::Config = serde_json::from_value(
            json!({"retained_intermediate_root":"new/i","retained_probe_root":"new/p"}),
        )
        .unwrap();
        assert_eq!(relocated_inputs(&a).unwrap(), relocated_inputs(&b).unwrap());
        assert!(serde_json::from_value::<prefix::Config>(json!({"retained_intermediate_root":"i","retained_probe_root":"p","unexpected_hash":"bad"})).is_err());
    }
    #[test]
    fn saved_jacobian_requires_exact_finite_shape() {
        assert!(decode_jacobian(&vec![0; 1920 * 4]).is_ok());
        assert!(decode_jacobian(&vec![0; 1920 * 4 - 1]).is_err());
        let mut bad = vec![0; 1920 * 4];
        bad[..4].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(decode_jacobian(&bad).is_err());
    }
    #[test]
    fn saved_credit_rejects_unknown_contract_and_invalid_pins() {
        let mut a = SavedProtectedCredit {
            contract_version: 1,
            root: PathBuf::from("r"),
            runtime_root: PathBuf::from("t"),
            observation_root: PathBuf::from("o"),
            expected_runtime_identity_sha256: "0".repeat(64),
            expected_observation_manifest_sha256: "1".repeat(64),
        };
        assert!(a.validate().is_ok());
        a.contract_version = 2;
        assert!(a.validate().is_err());
        a.contract_version = 1;
        a.expected_runtime_identity_sha256 = "z".repeat(64);
        assert!(a.validate().is_err());
    }
}
