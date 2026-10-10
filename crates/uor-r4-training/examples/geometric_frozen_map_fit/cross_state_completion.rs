//! One pre-registered cross-state forward mechanism; no constructor or selector.
use super::*;
pub(super) const ORIGINAL8: [usize; 8] = [0, 1, 4, 5, 8, 9, 12, 13];
pub(super) const RESUME_REPORT_SHA: &str =
    "738649541df34641ffe0eb6e45a151d612d485ff8902b2d9d8d1e812844a2cde";
pub(super) const RESUME_MANIFEST_SHA: &str =
    "e5623d996b60308d8ca1080f748c062388ff74c6b14b16f9956b4b792a9ca8c2";
pub(super) const RESUME_FIELD_SHA: &str =
    "f61249ae4032a92d452689a52add6c9b5a67099f55b1a9291ba6f6f66a0d9acf";
const RESUME145_REPORT_SHA: &str =
    "e3a7e3ebd93c38253afb67f77ad3bb6b64a5e0ac50561f0204e1b83d22b8f4a5";
const RESUME145_MANIFEST_SHA: &str =
    "172a2aacdce188bd5a37b131a294b877b2af60c87fc5520a9ea371c4b79460dd";
const RESUME145_FIELD_SHA: &str =
    "03781884edd6c2524c66e724c1fc65d9b9c62968d33b264d976282b49ce8094a";

const RESUME175_REPORT_SHA: &str =
    "9632d32651d0bcce82b2cda2ba70dee7d7a7878ff99577f88002442e23acb572";
const RESUME175_MANIFEST_SHA: &str =
    "1a6216a307a739a804f81bf94e93fd89638a896fda1904986dd3aee0a280fccc";
const RESUME175_FIELD_SHA: &str =
    "c82c376df10f14d5c1c9af970fd3b1fe239fb3560e2ccd0020dd2805d8fb773e";
const RESUME175_MASTER_SHA: &str =
    "c59f6eb8898a7b7f3ebef0a877a7a26fef3ab3eb681a55e8c6e02cf701d11f10";
const BOTTLENECK_OBJECTIVE: &str = "equal-episode-logmeanexp-unweighted-native-token-CE/1;temperature1;all-targets-including-EOS;existing-raw-identity-STE;no-phase-weighting";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ResumeProfile {
    report_sha: &'static str,
    manifest_sha: &'static str,
    field_sha: &'static str,
    pub checkpoint_step: usize,
    pub lineage_step: usize,
    pub complete: usize,
}
const SAVED22: ResumeProfile = ResumeProfile {
    report_sha: RESUME_REPORT_SHA,
    manifest_sha: RESUME_MANIFEST_SHA,
    field_sha: RESUME_FIELD_SHA,
    checkpoint_step: 64,
    lineage_step: 64,
    complete: 22,
};
const SAVED145: ResumeProfile = ResumeProfile {
    report_sha: RESUME145_REPORT_SHA,
    manifest_sha: RESUME145_MANIFEST_SHA,
    field_sha: RESUME145_FIELD_SHA,
    checkpoint_step: 256,
    lineage_step: 320,
    complete: 145,
};
const SAVED175: ResumeProfile = ResumeProfile {
    report_sha: RESUME175_REPORT_SHA,
    manifest_sha: RESUME175_MANIFEST_SHA,
    field_sha: RESUME175_FIELD_SHA,
    checkpoint_step: 256,
    lineage_step: 576,
    complete: 175,
};
impl ResumeProfile {
    pub fn lineage_after(self, local_step: usize) -> usize {
        self.lineage_step + local_step
    }
    fn validate_lineage(self, report: &Value, receipt: &Value) -> Result<()> {
        if report["updates"] != self.checkpoint_step || receipt["step"] != self.checkpoint_step {
            return Err(bad("cross-state resume local checkpoint step differs"));
        }
        if self == SAVED22 {
            // This exact sealed original run predates explicit lineage fields.
            if report.get("prior_updates").is_some()
                || report.get("lineage_step").is_some()
                || report.get("cross_state_resume").is_some()
                || receipt.get("lineage_step").is_some()
                || receipt.get("cross_state_resume").is_some()
            {
                return Err(bad("cross-state original64 lineage metadata differs"));
            }
        } else if self == SAVED145 {
            let prior = &report["cross_state_resume"];
            if report["prior_updates"] != SAVED22.lineage_step
                || report["lineage_step"] != self.lineage_step
                || receipt["lineage_step"] != self.lineage_step
                || report["fresh_adam"] != true
                || receipt["cross_state_resume"] != *prior
                || prior["report_sha256"] != SAVED22.report_sha
                || prior["manifest_sha256"] != SAVED22.manifest_sha
                || prior["field_sha256"] != SAVED22.field_sha
                || prior["prior_step"] != SAVED22.checkpoint_step
                || prior["prior_complete"] != SAVED22.complete
                || report["initial_receipt"]["step"] != 0
                || report["initial_receipt"]["lineage_step"] != SAVED22.lineage_step
            {
                return Err(bad("cross-state saved145 lineage/provenance differs"));
            }
        } else if self == SAVED175 {
            let prior = &report["cross_state_resume"];
            let initial = &report["initial_receipt"];
            if report["prior_updates"] != SAVED145.lineage_step
                || report["lineage_step"] != self.lineage_step
                || receipt["lineage_step"] != self.lineage_step
                || report["fresh_adam"] != true
                || receipt["cross_state_resume"] != *prior
                || initial["cross_state_resume"] != *prior
                || prior["report_sha256"] != SAVED145.report_sha
                || prior["manifest_sha256"] != SAVED145.manifest_sha
                || prior["field_sha256"] != SAVED145.field_sha
                || prior["prior_step"] != SAVED145.checkpoint_step
                || prior["prior_lineage_step"] != SAVED145.lineage_step
                || prior["prior_complete"] != SAVED145.complete
                || initial["step"] != 0
                || initial["lineage_step"] != SAVED145.lineage_step
                || initial["continuation_sha256"] != SAVED145.field_sha
                || initial["parameters"] != prior["source_master_inventory"]
                || report["initial"]["continuation_sha256"] != SAVED145.field_sha
                || report["initial"]["complete"] != SAVED145.complete
                || report["phase_policy"] != "none;all-target-token-logmeanexp"
                || receipt["parameters"]["continuation.cross_state"]["sha256"]
                    != RESUME175_MASTER_SHA
                || report["final_active_masters"]["continuation.cross_state"]
                    != RESUME175_MASTER_SHA
            {
                return Err(bad(
                    "cross-state saved175 lineage/ancestry/master identity differs",
                ));
            }
            for policy in [report, initial, receipt] {
                if policy["cross_state_bottleneck"] != true
                    || policy["training_objective"] != BOTTLENECK_OBJECTIVE
                    || policy["loss_scope"] != "all"
                    || policy["credit"] != "raw_identity"
                {
                    return Err(bad("cross-state saved175 bottleneck policy differs"));
                }
            }
        } else {
            return Err(bad("cross-state unsupported resume profile"));
        }
        Ok(())
    }
}
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ResumeConfig {
    pub root: PathBuf,
    pub expected_report_sha256: String,
    pub expected_manifest_sha256: String,
    pub expected_field_sha256: String,
}

impl ResumeConfig {
    fn profile(&self) -> Result<ResumeProfile> {
        [SAVED22, SAVED145, SAVED175]
            .into_iter()
            .find(|p| {
                self.expected_report_sha256 == p.report_sha
                    && self.expected_manifest_sha256 == p.manifest_sha
                    && self.expected_field_sha256 == p.field_sha
            })
            .ok_or_else(|| {
                bad("cross-state resume requires one complete pinned profile hash triple")
            })
    }
}

pub(super) struct ResumeState {
    pub profile: ResumeProfile,
    root: PathBuf,
    field_bytes: Vec<u8>,
    receipt: Value,
    pub prior_final: Value,
    pub provenance: Value,
}

pub(super) fn load_resume(a: &Args, parent: &ContinuationParent) -> Result<Option<ResumeState>> {
    let Some(config) = &a.cross_state_resume else {
        return Ok(None);
    };
    settings(a)?;
    let profile = config.profile()?;
    report_output::verify(&config.root)?;
    if sha256_file(&config.root.join("report.json"))? != profile.report_sha
        || sha256_file(&config.root.join("manifest.json"))? != profile.manifest_sha
    {
        return Err(bad("cross-state resume sealed report identity differs"));
    }
    let report = read(&config.root.join("report.json"))?;
    let checkpoint = config
        .root
        .join(format!("checkpoint-{:04}", profile.checkpoint_step));
    let receipt = read(&checkpoint.join("receipt.json"))?;
    let field_bytes = fs::read(checkpoint.join("continuation-field.bin"))?;
    profile.validate_lineage(&report, &receipt)?;
    if report["status"] != "COMPLETED"
        || report["mode"] != "cross_state_continuation"
        || report["final_receipt"] != receipt
        || receipt["parent"] != serde_json::to_value(&parent.binding)?
        || receipt["generate_sha256"] != parent.generate_sha256
        || receipt["frozen_model_report_sha256"] != CONTINUATION_PARENT_REPORT_SHA
        || receipt["frozen_model_manifest_sha256"] != CONTINUATION_PARENT_MANIFEST_SHA
        || receipt["frozen_parent_receipt"] != parent.receipt
        || receipt["continuation_sha256"] != profile.field_sha
        || report["final"]["continuation_sha256"] != profile.field_sha
        || sha256_bytes(&field_bytes) != profile.field_sha
        || receipt["shared_coefficients"] != 115200
        || receipt["active_parameter_names"] != json!(["continuation.cross_state"])
        || read(&checkpoint.join("continuation-source/metadata.json"))? != receipt
    {
        return Err(bad(
            "cross-state resume checkpoint/upstream/field identity differs",
        ));
    }
    validate_rows(&report["final"], profile.complete)?;
    let provenance = json!({"root":fs::canonicalize(&config.root)?,
        "report_sha256":profile.report_sha,"manifest_sha256":profile.manifest_sha,
        "checkpoint_receipt_sha256":sha256_file(&checkpoint.join("receipt.json"))?,
        "field_sha256":profile.field_sha,"source_master_inventory":receipt["parameters"],
        "prior_step":profile.checkpoint_step,"prior_lineage_step":profile.lineage_step,
        "prior_complete":profile.complete,"optimizer":"reset fresh AdamW moments; parameter continuation only"});
    Ok(Some(ResumeState {
        profile,
        root: checkpoint,
        field_bytes,
        receipt,
        prior_final: report["final"].clone(),
        provenance,
    }))
}

impl ResumeState {
    pub fn restore_weights(
        &self,
        parent: &ContinuationParent,
        device: &Device,
    ) -> Result<ContinuationLearningWeights> {
        let generator = parent.generator()?;
        let field = NativeContinuationField::from_bytes(
            &self.field_bytes,
            generator.source_binding(),
            generator.generate_model(),
        )?;
        if !field.is_cross_state() || field.lanes() != 8 || field.vocab_size() != 4096 {
            return Err(bad("cross-state resume native schema/dimensions differ"));
        }
        let weights = ContinuationLearningWeights::from_native(
            &field,
            generator.generate_model(),
            parent.integer.binding(),
            device,
        )?;
        restore(
            &self.root.join("continuation-source"),
            &self.receipt["parameters"],
            &weights.parameters(),
            device,
        )?;
        if weights
            .export_native(generator.source_binding(), generator.generate_model())?
            .to_bytes()?
            != self.field_bytes
        {
            return Err(bad(
                "cross-state resumed fractional masters do not export exact saved field",
            ));
        }
        Ok(weights)
    }
}

pub(super) fn schedule_indices(schedule: &[usize], update: usize) -> Vec<usize> {
    (0..BATCH)
        .map(|i| schedule[(update * BATCH + i) % schedule.len()])
        .collect()
}

fn validate_rows(endpoint: &Value, expected: usize) -> Result<()> {
    let rows = endpoint["rows"]
        .as_array()
        .filter(|r| r.len() == 512)
        .ok_or_else(|| bad("cross-state endpoint coverage"))?;
    let mut seen = BTreeSet::new();
    let mut count = 0;
    for row in rows {
        let id = row["id"]
            .as_str()
            .ok_or_else(|| bad("cross-state row ID absent"))?;
        if !seen.insert(id)
            || row["generated_ids"]
                .as_array()
                .is_none_or(|v| v.iter().any(|id| id.as_u64().is_none()))
        {
            return Err(bad("cross-state row identity/generated IDs invalid"));
        }
        count += usize::from(
            row["complete"]
                .as_bool()
                .ok_or_else(|| bad("cross-state completion absent"))?,
        );
    }
    if count != expected || endpoint["complete"] != expected {
        return Err(bad("cross-state complete count differs"));
    }
    Ok(())
}

pub(super) fn settings(a: &Args) -> Result<()> {
    if a.mode != Mode::CrossStateContinuation {
        return if a.cross_state_resume.is_some() || a.cross_state_bottleneck {
            Err(bad(
                "cross-state resume/objective requires cross-state mode",
            ))
        } else {
            Ok(())
        };
    }
    let profile = a
        .cross_state_resume
        .as_ref()
        .map(ResumeConfig::profile)
        .transpose()?;
    // The stopped phase-balanced continuation stays unavailable. These two
    // strict profiles are admitted only under the retained bottleneck objective.
    if a.cross_state_bottleneck != matches!(profile, Some(SAVED145 | SAVED175)) {
        return Err(bad("bottleneck objective requires pinned saved145 or saved175; phase-CE continuation from either is unavailable"));
    }
    if a.updates
        != if a.cross_state_resume.is_some() {
            256
        } else {
            64
        }
        || a.seed != 1001
        || a.credit != Credit::RawIdentity
        || a.read_state_pullback != ReadStatePullback::Legacy
        || a.loss_scope != LossScope::All
        || a.ceiling_scorer
        || a.ceiling_float
        || a.query_conditioned_read
        || a.panel_inputs.is_some()
        || a.panel_labels.is_some()
        || a.baseline.is_some()
        || a.prediction_control_updates.is_some()
        || a.prediction_control_rates.is_some()
        || a.prediction_control_resume.is_some()
        || a.prediction_control_resume_source_step.is_some()
        || a.prediction_control_resume_generate_step.is_some()
        || a.prediction_control_trainable != ControlTrainable::Joint
        || a.continuation
            .as_ref()
            .is_none_or(|c| c.learning_rate != 0.03)
        || a.joint_continuation.is_some()
        || a.reference_replay.is_some()
        || a.native_code_proposals
        || a.reached_frontier_objective
        || a.categorical_action_learning
        || a.categorical_action_only
        || a.constrained_context_learning
        || a.constrained_emission_learning
        || a.retained_context_root.is_some()
        || a.prototype_compensation.is_some()
        || a.reached_u.is_some()
        || a.readout_coadaptation.is_some()
        || a.prefix_context_credit.is_some()
        || a.context_path_credit.is_some()
        || a.context_cue_coadapt.is_some()
        || a.prefix_fragment_learning.is_some()
        || a.generate_episode_learning.is_some()
        || a.coupled_episode_learning.is_some()
        || a.generate_episode_completion.is_some()
        || a.prefix_artifact_check.is_some()
    {
        return Err(bad("cross-state continuation requires fresh64 or pinned-resume256/seed1001/lr.03/raw_identity/all-answer fitting with frozen upstream and no proposal/diagnostic options"));
    }
    Ok(())
}

pub(super) fn baseline(initial: &Value, prior: Option<(&Value, ResumeProfile)>) -> Result<()> {
    if let Some((prior, profile)) = prior {
        if prior["continuation_sha256"] != profile.field_sha
            || initial["continuation_sha256"] != profile.field_sha
        {
            return Err(bad("cross-state resume baseline field identity differs"));
        }
        validate_rows(prior, profile.complete)?;
        validate_rows(initial, profile.complete)?;
        let old = prior["rows"]
            .as_array()
            .ok_or_else(|| bad("resume prior rows absent"))?;
        let current = initial["rows"]
            .as_array()
            .ok_or_else(|| bad("resume baseline rows absent"))?;
        for (a, b) in old.iter().zip(current) {
            for key in ["id", "generated_ids", "complete"] {
                if a[key] != b[key] {
                    return Err(bad(
                        "cross-state resumed baseline differs from saved512 rows",
                    ));
                }
            }
        }
        return Ok(());
    }
    validate_rows(initial, 8)?;
    let rows = initial["rows"]
        .as_array()
        .filter(|r| r.len() == 512)
        .ok_or_else(|| bad("cross-state baseline coverage"))?;
    let complete = rows
        .iter()
        .enumerate()
        .filter(|(_, r)| r["complete"] == true)
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    if complete != ORIGINAL8 || initial["complete"] != 8 {
        return Err(bad(
            "cross-state zero field did not reproduce exact original eight",
        ));
    }
    Ok(())
}

pub(super) fn outcomes(
    initial: &Value,
    final_eval: &Value,
    prior: Option<(&Value, ResumeProfile)>,
) -> Result<Value> {
    baseline(initial, prior)?;
    let initial_count = prior.map_or(8, |(_, profile)| profile.complete);
    let before = initial["rows"]
        .as_array()
        .ok_or_else(|| bad("initial rows absent"))?;
    let after = final_eval["rows"]
        .as_array()
        .filter(|r| r.len() == 512)
        .ok_or_else(|| bad("final rows coverage"))?;
    let mut seen = BTreeSet::new();
    let mut gained = Vec::new();
    let mut lost = Vec::new();
    let mut retained = Vec::new();
    let mut changed = Vec::new();
    let mut strata = BTreeMap::<String, (usize, usize, usize)>::new();
    for (old, new) in before.iter().zip(after) {
        let id = old["id"]
            .as_str()
            .ok_or_else(|| bad("row identity absent"))?;
        if new["id"] != id || !seen.insert(id) {
            return Err(bad("row identity/uniqueness differs"));
        }
        let a = old["complete"]
            .as_bool()
            .ok_or_else(|| bad("initial completion absent"))?;
        let b = new["complete"]
            .as_bool()
            .ok_or_else(|| bad("final completion absent"))?;
        match (a, b) {
            (false, true) => gained.push(id),
            (true, false) => lost.push(id),
            (true, true) => retained.push(id),
            _ => {}
        }
        if old["generated_ids"] != new["generated_ids"] {
            changed.push(id);
        }
        let stratum = id
            .split('-')
            .nth(2)
            .ok_or_else(|| bad("stratum absent"))?
            .to_string();
        let counts = strata.entry(stratum).or_default();
        counts.0 += 1;
        counts.1 += usize::from(a);
        counts.2 += usize::from(b);
    }
    let final_count = gained.len() + retained.len();
    if final_eval["complete"] != final_count {
        return Err(bad("final complete summary differs"));
    }
    Ok(
        json!({"initial_complete":initial_count,"final_complete":final_count,
        "gained_complete_ids":gained,"lost_complete_ids":lost,"retained_complete_ids":retained,
        "changed_output_ids":changed,"by_stratum_cases_initial_final":strata,
        "keep":final_count>initial_count,"decision":if final_count>initial_count {"KEEP"} else {"REJECT"},
        "bar":format!("prospective developmental netcomplete>{initial_count} on unchanged512 own-prefix; exact loss/gain IDs reported, not endpoint retention veto; M2 remains256 thenfresh40%"),
        "prior_runs":"prior all-original8 local KEEP decisions unchanged; no retrospective reclassification"}),
    )
}

/// Training-only objective; native scoring and evaluation CE remain unchanged.
pub(super) fn objective_policy(a: &Args) -> &'static str {
    if a.cross_state_bottleneck {
        BOTTLENECK_OBJECTIVE
    } else {
        loss_weight_policy(true, LossScope::All)
    }
}

/// Stable streaming chain rule for log(mean(exp(token CE))). The loss values
/// are hard-forward anchors and each supplied gradient is the existing token
/// STE. The coefficients here are detached chain-rule multipliers, not a new
/// differentiable weighted-mean objective. Only one token graph stays live.
#[derive(Default)]
pub(super) struct EpisodeBottleneck {
    maximum: f64,
    normalization: f64,
    positions: usize,
    numerator: BTreeMap<String, Tensor>,
}
impl EpisodeBottleneck {
    pub fn push(&mut self, loss: f64, gradients: BTreeMap<String, Tensor>) -> Result<()> {
        if !loss.is_finite() || gradients.is_empty() || self.positions >= 32 {
            return Err(bad(
                "bottleneck requires finite loss, active gradients and at most32 targets",
            ));
        }
        if self.positions != 0 && self.numerator.keys().ne(gradients.keys()) {
            return Err(bad(
                "bottleneck active gradient inventory changed within episode",
            ));
        }
        let maximum = if self.positions == 0 {
            loss
        } else {
            self.maximum.max(loss)
        };
        let old_scale = if self.positions == 0 {
            0.
        } else {
            (self.maximum - maximum).exp()
        };
        let token_scale = (loss - maximum).exp();
        let normalization = self.normalization * old_scale + token_scale;
        for (name, gradient) in gradients {
            let weighted = (&gradient * token_scale)?;
            let next = if let Some(old) = self.numerator.remove(&name) {
                ((old * old_scale)? + weighted)?
            } else {
                weighted
            };
            self.numerator.insert(name, next.detach());
        }
        self.maximum = maximum;
        self.normalization = normalization;
        self.positions += 1;
        Ok(())
    }

    pub fn finish(
        self,
        expected_positions: usize,
        batch_episodes: usize,
    ) -> Result<(f64, BTreeMap<String, Tensor>)> {
        if self.positions == 0
            || self.positions != expected_positions
            || batch_episodes == 0
            || !self.normalization.is_finite()
            || self.normalization <= 0.
        {
            return Err(bad(
                "bottleneck requires every target including EOS and nonempty batch",
            ));
        }
        let objective = (self.maximum + self.normalization.ln() - (self.positions as f64).ln())
            / batch_episodes as f64;
        if !objective.is_finite() {
            return Err(bad("nonfinite bottleneck objective"));
        }
        // No additional 1/T or phase factor: log-mean-exp already supplies its
        // softmax token derivative; only equal episode/batch averaging remains.
        let scale = 1. / self.normalization / batch_episodes as f64;
        let gradients = self
            .numerator
            .into_iter()
            .map(|(name, g)| Ok((name, (g * scale)?.detach())))
            .collect::<Result<BTreeMap<_, _>>>()?;
        Ok((objective, gradients))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn endpoint(indices: &[usize]) -> Value {
        json!({"complete":indices.len(),"continuation_sha256":RESUME_FIELD_SHA,"rows":(0..512).map(|i|json!({
   "id":format!("development-diverse-length2-{i}"),"complete":indices.contains(&i),
   "generated_ids":[i]})).collect::<Vec<_>>()})
    }
    #[test]
    fn cross_state_net_gain_records_original_loss_without_old_local_veto() -> Result<()> {
        let initial = endpoint(&ORIGINAL8);
        let next = endpoint(&[1, 4, 5, 8, 9, 12, 13, 100, 101]);
        let result = outcomes(&initial, &next, None)?;
        assert_eq!(result["keep"], true);
        assert_eq!(result["final_complete"], 9);
        assert_eq!(
            result["lost_complete_ids"].as_array().map(Vec::len),
            Some(1)
        );
        assert_eq!(
            result["gained_complete_ids"].as_array().map(Vec::len),
            Some(2)
        );
        assert_eq!(outcomes(&initial, &initial, None)?["keep"], false);
        Ok(())
    }
    #[test]
    fn cross_state_rejects_wrong_zero_baseline_or_duplicate_rows() -> Result<()> {
        assert!(baseline(&endpoint(&[0, 1, 4, 5, 8, 9, 12]), None).is_err());
        let initial = endpoint(&ORIGINAL8);
        let mut next = initial.clone();
        next["rows"][1]["id"] = next["rows"][0]["id"].clone();
        assert!(outcomes(&initial, &next, None).is_err());
        Ok(())
    }
    fn config() -> Value {
        json!({"mode":"cross_state_continuation","credit":"raw_identity","seed":1001,
   "updates":64,"checkpoint":"cp","saved_fit":"fit","categorical":"cat","parent_config":"p",
   "training_inputs":"i","training_labels":"l","development_inputs":"i","development_labels":"l",
   "maximum_seconds":7200,"maximum_report_bytes":1073741824,"out":"new",
   "continuation":{"expected_model_report_sha256":CONTINUATION_PARENT_REPORT_SHA,
   "expected_model_manifest_sha256":CONTINUATION_PARENT_MANIFEST_SHA,
   "learning_rate":0.03,"maximum_cache_tensor_bytes":536870912}})
    }
    #[test]
    fn cross_state_admission_rejects_recipe_or_credit_variations() -> Result<()> {
        let base = config();
        let parsed: Args = serde_json::from_value(base.clone())?;
        settings(&parsed)?;
        continuation_settings(&parsed)?;
        for (key, value) in [
            ("updates", json!(96)),
            ("seed", json!(1002)),
            ("native_code_proposals", json!(true)),
            ("read_state_pullback", json!("categorical")),
            ("loss_scope", json!("entry_only")),
        ] {
            let mut bad = base.clone();
            bad[key] = value;
            assert!(settings(&serde_json::from_value::<Args>(bad)?).is_err());
        }
        Ok(())
    }
    #[test]
    fn cross_state_resume_requires_exact_all512_outputs_not_only_complete22() -> Result<()> {
        let prior = endpoint(&(0..22).collect::<Vec<_>>());
        baseline(&prior, Some((&prior, SAVED22)))?;
        for (row, key, value) in [
            (300, "generated_ids", json!([999])),
            (300, "id", json!("development-diverse-length2-changed")),
            (300, "complete", json!(true)),
        ] {
            let mut changed = prior.clone();
            changed["rows"][row][key] = value;
            assert!(
                baseline(&changed, Some((&prior, SAVED22))).is_err(),
                "{key}"
            );
        }
        let mut swapped = prior.clone();
        swapped["rows"]
            .as_array_mut()
            .ok_or_else(|| bad("fixture rows"))?
            .swap(300, 301);
        assert!(baseline(&swapped, Some((&prior, SAVED22))).is_err());
        let mut duplicate = prior.clone();
        duplicate["rows"][300]["id"] = duplicate["rows"][301]["id"].clone();
        assert!(baseline(&duplicate, Some((&duplicate, SAVED22))).is_err());
        let mut wrong_field = prior.clone();
        wrong_field["continuation_sha256"] = json!("wrong");
        assert!(baseline(&wrong_field, Some((&prior, SAVED22))).is_err());
        Ok(())
    }

    #[test]
    fn cross_state_resume_net_gain_uses_saved22_and_reports_losses() -> Result<()> {
        let prior = endpoint(&(0..22).collect::<Vec<_>>());
        let next = endpoint(&(1..24).collect::<Vec<_>>());
        let result = outcomes(&prior, &next, Some((&prior, SAVED22)))?;
        assert_eq!(result["initial_complete"], 22);
        assert_eq!(result["final_complete"], 23);
        assert_eq!(result["keep"], true);
        assert_eq!(
            result["lost_complete_ids"].as_array().map(Vec::len),
            Some(1)
        );
        assert_eq!(
            result["gained_complete_ids"].as_array().map(Vec::len),
            Some(2)
        );
        assert_eq!(
            outcomes(&prior, &prior, Some((&prior, SAVED22)))?["keep"],
            false
        );
        assert_eq!(
            outcomes(
                &prior,
                &endpoint(&(0..20).collect::<Vec<_>>()),
                Some((&prior, SAVED22))
            )?["keep"],
            false
        );
        Ok(())
    }

    #[test]
    fn cross_state_resume_schedule_is_four_identical_full_passes() {
        let schedule = order(1001, 512);
        let fresh = (0..64)
            .flat_map(|u| schedule_indices(&schedule, u))
            .collect::<Vec<_>>();
        let resumed = (0..256)
            .flat_map(|u| schedule_indices(&schedule, u))
            .collect::<Vec<_>>();
        assert_eq!(fresh, schedule);
        assert_eq!(
            fresh.iter().copied().collect::<BTreeSet<_>>(),
            (0..512).collect()
        );
        assert_eq!(resumed.len(), 2048);
        for pass in resumed.chunks_exact(512) {
            assert_eq!(pass, fresh.as_slice());
        }
        for index in 0..512 {
            assert_eq!(resumed.iter().filter(|&&i| i == index).count(), 4);
        }
    }

    #[test]
    fn cross_state_resume_admission_pins_parent_and_exact_local256() -> Result<()> {
        let mut valid = config();
        valid["updates"] = json!(256);
        valid["cross_state_resume"] = json!({"root":"saved22",
            "expected_report_sha256":RESUME_REPORT_SHA,
            "expected_manifest_sha256":RESUME_MANIFEST_SHA,
            "expected_field_sha256":RESUME_FIELD_SHA});
        let parsed: Args = serde_json::from_value(valid.clone())?;
        settings(&parsed)?;
        continuation_settings(&parsed)?;
        for key in [
            "expected_report_sha256",
            "expected_manifest_sha256",
            "expected_field_sha256",
        ] {
            let mut changed = valid.clone();
            changed["cross_state_resume"][key] = json!("wrong");
            assert!(
                settings(&serde_json::from_value(changed)?).is_err(),
                "{key}"
            );
        }
        for updates in [0, 64, 128, 320] {
            let mut changed = valid.clone();
            changed["updates"] = json!(updates);
            assert!(settings(&serde_json::from_value(changed)?).is_err());
        }
        let mut wrong_mode = valid.clone();
        wrong_mode["mode"] = json!("continuation_only");
        assert!(settings(&serde_json::from_value(wrong_mode)?).is_err());
        let mut fresh = config();
        fresh["updates"] = json!(256);
        assert!(settings(&serde_json::from_value(fresh)?).is_err());
        valid["cross_state_resume"]["unknown"] = json!(true);
        assert!(serde_json::from_value::<Args>(valid).is_err());
        Ok(())
    }

    #[test]
    fn cross_state_resume_restores_fractional_master_bits_and_rejects_tampering() -> Result<()> {
        let owned = owned_fixture()?;
        let values = vec![0.031f32, -0.124, 0.376, 1.749];
        let name = "continuation.cross_state".to_string();
        let source = BTreeMap::from([(
            name.clone(),
            Var::from_vec(values.clone(), (1, 4), &Device::Cpu)?,
        )]);
        let inventory = save_masters(&owned.0.join("masters"), &source)?;
        let restored = BTreeMap::from([(
            name.clone(),
            Var::zeros((1, 4), candle_core::DType::F32, &Device::Cpu)?,
        )]);
        restore(
            &owned.0.join("masters"),
            &inventory,
            &restored,
            &Device::Cpu,
        )?;
        assert_eq!(identities(&source)?, identities(&restored)?);
        let observed = restored[&name].flatten_all()?.to_vec1::<f32>()?;
        assert_eq!(
            observed.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
            values.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
        );
        assert!(observed.iter().any(|x| *x != (x * 4.).round() * 0.25));
        let path = owned.0.join("masters").join(format!("{name}.f32le"));
        let original = fs::read(&path)?;
        let mut altered = original.clone();
        altered[0] ^= 1;
        fs::write(&path, &altered)?;
        assert!(restore(
            &owned.0.join("masters"),
            &inventory,
            &restored,
            &Device::Cpu
        )
        .is_err());
        fs::write(&path, &original)?;
        let mut wrong_shape = inventory.clone();
        wrong_shape[&name]["shape"] = json!([2, 2]);
        assert!(restore(
            &owned.0.join("masters"),
            &wrong_shape,
            &restored,
            &Device::Cpu
        )
        .is_err());
        let mut missing = inventory;
        missing
            .as_object_mut()
            .ok_or_else(|| bad("fixture inventory"))?
            .remove(&name);
        assert!(restore(&owned.0.join("masters"), &missing, &restored, &Device::Cpu).is_err());
        Ok(())
    }
    struct OwnedFixture(PathBuf);
    impl Drop for OwnedFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn owned_fixture() -> Result<OwnedFixture> {
        // Keep test artifacts in the owned worktree, never system temp or a
        // shared cache. Exclusive creation prevents replacing another fixture.
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let root = PathBuf::from("local").join(format!(
            "cross-state-resume-test-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all("local")?;
        fs::create_dir(&root)?;
        Ok(OwnedFixture(root))
    }
    #[test]
    fn cross_state_resume_rejects_output_overlap_before_claim() -> Result<()> {
        let owned = owned_fixture()?;
        let safe_input = owned.0.join("inputs");
        let retained = owned.0.join("saved22");
        fs::create_dir(&safe_input)?;
        fs::create_dir(&retained)?;
        let mut config = config();
        for key in [
            "checkpoint",
            "saved_fit",
            "categorical",
            "parent_config",
            "training_inputs",
            "training_labels",
            "development_inputs",
            "development_labels",
        ] {
            config[key] = json!(safe_input);
        }
        config["cross_state_resume"] = json!({"root":retained,
            "expected_report_sha256":RESUME_REPORT_SHA,
            "expected_manifest_sha256":RESUME_MANIFEST_SHA,
            "expected_field_sha256":RESUME_FIELD_SHA});
        for output in [retained.clone(), retained.join("new-run"), owned.0.clone()] {
            config["out"] = json!(output);
            assert!(validate_input_output_paths(&serde_json::from_value(config.clone())?).is_err());
        }
        assert!(!retained.join("new-run").exists());
        config["out"] = json!(owned.0.join("new-run"));
        validate_input_output_paths(&serde_json::from_value(config)?)?;
        assert!(!owned.0.join("new-run").exists());
        Ok(())
    }
    fn profile_config(profile: ResumeProfile) -> ResumeConfig {
        ResumeConfig {
            root: PathBuf::from("sealed-parent"),
            expected_report_sha256: profile.report_sha.to_string(),
            expected_manifest_sha256: profile.manifest_sha.to_string(),
            expected_field_sha256: profile.field_sha.to_string(),
        }
    }

    #[test]
    fn cross_state_saved145_admission_requires_complete_profile_triple() -> Result<()> {
        for profile in [SAVED22, SAVED145, SAVED175] {
            assert_eq!(profile_config(profile).profile()?, profile);
            let mut admitted = config();
            admitted["updates"] = json!(256);
            admitted["cross_state_bottleneck"] = json!(matches!(profile, SAVED145 | SAVED175));
            admitted["cross_state_resume"] = serde_json::to_value(profile_config(profile))?;
            let args: Args = serde_json::from_value(admitted.clone())?;
            settings(&args)?;
            continuation_settings(&args)?;
            for other in [SAVED22, SAVED145, SAVED175]
                .into_iter()
                .filter(|p| *p != profile)
            {
                for key in [
                    "expected_report_sha256",
                    "expected_manifest_sha256",
                    "expected_field_sha256",
                ] {
                    let mut mixed = admitted.clone();
                    mixed["cross_state_resume"][key] =
                        serde_json::to_value(profile_config(other))?[key].clone();
                    assert!(settings(&serde_json::from_value(mixed)?).is_err(), "{key}");
                }
            }
        }
        let mut unsupported = profile_config(SAVED145);
        unsupported.expected_field_sha256 = "0".repeat(64);
        assert!(unsupported.profile().is_err());
        Ok(())
    }

    #[test]
    fn cross_state_saved145_validates_local_and_cumulative_lineage() -> Result<()> {
        let original_report = json!({"updates":64});
        let original_receipt = json!({"step":64});
        SAVED22.validate_lineage(&original_report, &original_receipt)?;
        assert_eq!(SAVED22.lineage_after(0), 64);
        assert_eq!(SAVED22.lineage_after(256), 320);
        assert_eq!(SAVED145.lineage_after(0), 320);
        assert_eq!(SAVED145.lineage_after(256), 576);
        let provenance = json!({"report_sha256":SAVED22.report_sha,
            "manifest_sha256":SAVED22.manifest_sha,"field_sha256":SAVED22.field_sha,
            "prior_step":64,"prior_complete":22});
        let report = json!({"updates":256,"prior_updates":64,"lineage_step":320,
            "fresh_adam":true,"cross_state_resume":provenance,
            "initial_receipt":{"step":0,"lineage_step":64}});
        let receipt = json!({"step":256,"lineage_step":320,"cross_state_resume":provenance});
        SAVED145.validate_lineage(&report, &receipt)?;
        for (key, wrong) in [
            ("updates", json!(320)),
            ("prior_updates", json!(256)),
            ("lineage_step", json!(256)),
            ("fresh_adam", json!(false)),
        ] {
            let mut altered = report.clone();
            altered[key] = wrong;
            assert!(
                SAVED145.validate_lineage(&altered, &receipt).is_err(),
                "{key}"
            );
        }
        for key in ["step", "lineage_step"] {
            let mut altered = receipt.clone();
            altered[key] = json!(64);
            assert!(
                SAVED145.validate_lineage(&report, &altered).is_err(),
                "{key}"
            );
        }
        let mut altered = report.clone();
        altered["cross_state_resume"]["field_sha256"] = json!(SAVED145.field_sha);
        let mut matched_receipt = receipt.clone();
        matched_receipt["cross_state_resume"] = altered["cross_state_resume"].clone();
        assert!(SAVED145
            .validate_lineage(&altered, &matched_receipt)
            .is_err());
        assert!(SAVED145
            .validate_lineage(&original_report, &original_receipt)
            .is_err());
        assert!(SAVED22.validate_lineage(&report, &receipt).is_err());
        let mut fabricated_legacy = original_report;
        fabricated_legacy["lineage_step"] = json!(64);
        assert!(SAVED22
            .validate_lineage(&fabricated_legacy, &original_receipt)
            .is_err());
        Ok(())
    }

    #[test]
    fn cross_state_saved145_baseline_and_net_bar_are_profile_specific() -> Result<()> {
        let mut prior = endpoint(&(0..145).collect::<Vec<_>>());
        prior["continuation_sha256"] = json!(SAVED145.field_sha);
        baseline(&prior, Some((&prior, SAVED145)))?;
        assert!(baseline(&prior, Some((&prior, SAVED22))).is_err());
        let mut changed_failed_row = prior.clone();
        changed_failed_row["rows"][400]["generated_ids"] = json!([999]);
        assert!(baseline(&changed_failed_row, Some((&prior, SAVED145))).is_err());
        let next = endpoint(&(1..147).collect::<Vec<_>>());
        let result = outcomes(&prior, &next, Some((&prior, SAVED145)))?;
        assert_eq!(result["initial_complete"], 145);
        assert_eq!(result["final_complete"], 146);
        assert_eq!(result["keep"], true);
        assert_eq!(
            result["lost_complete_ids"].as_array().map(Vec::len),
            Some(1)
        );
        assert_eq!(
            result["gained_complete_ids"].as_array().map(Vec::len),
            Some(2)
        );
        assert_eq!(
            outcomes(&prior, &prior, Some((&prior, SAVED145)))?["keep"],
            false
        );
        assert_eq!(
            outcomes(
                &prior,
                &endpoint(&(0..144).collect::<Vec<_>>()),
                Some((&prior, SAVED145))
            )?["keep"],
            false
        );
        let mut wrong_count = endpoint(&(0..22).collect::<Vec<_>>());
        wrong_count["continuation_sha256"] = json!(SAVED145.field_sha);
        assert!(baseline(&wrong_count, Some((&wrong_count, SAVED145))).is_err());
        Ok(())
    }
    #[test]
    fn bottleneck_admission_rejects_stopped_objective_and_other_parents() -> Result<()> {
        let mut admitted = config();
        admitted["updates"] = json!(256);
        for profile in [SAVED145, SAVED175] {
            admitted["cross_state_resume"] = serde_json::to_value(profile_config(profile))?;
            admitted["cross_state_bottleneck"] = json!(false);
            assert!(settings(&serde_json::from_value(admitted.clone())?).is_err());
            admitted["cross_state_bottleneck"] = json!(true);
            settings(&serde_json::from_value(admitted.clone())?)?;
        }
        let mut changed = admitted.clone();
        changed["cross_state_resume"] = serde_json::to_value(profile_config(SAVED22))?;
        assert!(settings(&serde_json::from_value(changed)?).is_err());
        let mut fresh = config();
        fresh["cross_state_bottleneck"] = json!(true);
        assert!(settings(&serde_json::from_value(fresh)?).is_err());
        admitted["mode"] = json!("continuation_only");
        assert!(settings(&serde_json::from_value(admitted)?).is_err());
        Ok(())
    }

    /// Hard-forward anchors deliberately differ from the differentiable slopes.
    /// This mirrors the existing native-probability/STE composition rather than
    /// pretending finite differences of quantized serving equal its adjoint.
    fn check_bottleneck_reference(episodes: &[Vec<(f32, [f32; 2])>]) -> Result<(f64, Vec<f32>)> {
        let device = &Device::Cpu;
        let variable = Var::zeros(2, candle_core::DType::F32, device)?;
        let mut reference = Tensor::zeros((), candle_core::DType::F32, device)?;
        let mut streamed_value = 0.;
        let mut streamed_gradient = Tensor::zeros(2, candle_core::DType::F32, device)?;
        for episode in episodes {
            let mut accumulator = EpisodeBottleneck::default();
            let mut losses = Vec::new();
            for &(hard, slope) in episode {
                let coefficient = Tensor::from_vec(slope.to_vec(), 2, device)?;
                let soft = (variable.as_tensor() * coefficient)?.sum_all()?;
                let loss = ((&soft - soft.detach())? + hard as f64)?;
                let gradients = loss.backward()?;
                let gradient = gradients
                    .get(variable.as_tensor())
                    .ok_or_else(|| bad("fixture gradient absent"))?;
                accumulator.push(
                    loss.to_scalar::<f32>()? as f64,
                    BTreeMap::from([("field".to_string(), gradient.detach())]),
                )?;
                losses.push(loss);
            }
            let (value, gradients) = accumulator.finish(episode.len(), episodes.len())?;
            streamed_value += value;
            streamed_gradient = (&streamed_gradient + &gradients["field"])?;
            let maximum = episode
                .iter()
                .map(|(hard, _)| *hard as f64)
                .fold(f64::NEG_INFINITY, f64::max);
            let direct = (Tensor::stack(&losses, 0)? - maximum)?
                .exp()?
                .sum_all()?
                .log()?
                .affine(1., maximum - (episode.len() as f64).ln())?;
            reference = (reference + (direct / episodes.len() as f64)?)?;
        }
        let reference_value = reference.to_scalar::<f32>()? as f64;
        assert!(
            (streamed_value - reference_value).abs() < 2e-4,
            "streamed={streamed_value}, reference={reference_value}"
        );
        let gradients = reference.backward()?;
        let expected = gradients
            .get(variable.as_tensor())
            .ok_or_else(|| bad("reference gradient absent"))?
            .to_vec1::<f32>()?;
        let observed = streamed_gradient.to_vec1::<f32>()?;
        for (actual, expected) in observed.iter().zip(expected) {
            assert!((actual - expected).abs() < 2e-5, "{actual} != {expected}");
        }
        Ok((streamed_value, observed))
    }

    #[test]
    fn bottleneck_streaming_matches_analytic_and_differentiated_ste_reference() -> Result<()> {
        let (value, gradient) =
            check_bottleneck_reference(&[vec![(1., [1., 0.]), (4., [0., 2.]), (2., [-1., 3.])]])?;
        let z = 1f64.exp() + 4f64.exp() + 2f64.exp();
        assert!((value - (z / 3.).ln()).abs() < 1e-12);
        assert!((gradient[0] as f64 - (1f64.exp() - 2f64.exp()) / z).abs() < 1e-6);
        assert!((gradient[1] as f64 - (2. * 4f64.exp() + 3. * 2f64.exp()) / z).abs() < 1e-6);
        Ok(())
    }

    #[test]
    fn bottleneck_equal_episodes_include_eos_without_second_length_factor() -> Result<()> {
        // Last token represents EOS and is deliberately the hardest. Two
        // episodes of different length must each contribute half the objective.
        let (value, gradient) = check_bottleneck_reference(&[
            vec![(0., [1., 0.]), (4., [0., 1.])],
            vec![
                (2., [1., 0.]),
                (2., [1., 0.]),
                (2., [1., 0.]),
                (2., [1., 0.]),
            ],
        ])?;
        let eos_weight = 4f64.exp() / (1. + 4f64.exp());
        assert!((gradient[1] as f64 - 0.5 * eos_weight).abs() < 1e-6);
        assert!((gradient[0] as f64 - (0.5 + 0.5 * (1. - eos_weight))).abs() < 1e-6);
        assert!((value - 0.5 * (((1. + 4f64.exp()) / 2.).ln() + 2.)).abs() < 1e-12);
        Ok(())
    }

    #[test]
    fn bottleneck_max_shift_is_stable_and_uniform_losses_are_uniform() -> Result<()> {
        let base = vec![(0., [1., 0.]), (-1000., [0., 1.])];
        let shifted = base
            .iter()
            .map(|(l, g)| (l + 1000., *g))
            .collect::<Vec<_>>();
        let (a, ga) = check_bottleneck_reference(&[base])?;
        let (b, gb) = check_bottleneck_reference(&[shifted])?;
        assert!((b - a - 1000.).abs() < 1e-10);
        assert_eq!(ga, gb);
        assert_eq!(ga, vec![1., 0.]);
        let (_, uniform) = check_bottleneck_reference(&[vec![(3., [1., 0.]), (3., [0., 1.])]])?;
        assert_eq!(uniform, vec![0.5, 0.5]);
        Ok(())
    }

    #[test]
    fn bottleneck_rejects_nonfinite_empty_missing_eos_and_changed_inventory() -> Result<()> {
        fn gradients() -> Result<BTreeMap<String, Tensor>> {
            Ok(BTreeMap::from([(
                "field".to_string(),
                Tensor::new(1f32, &Device::Cpu)?,
            )]))
        }
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(EpisodeBottleneck::default()
                .push(value, gradients()?)
                .is_err());
        }
        assert!(EpisodeBottleneck::default()
            .push(1., BTreeMap::new())
            .is_err());
        assert!(EpisodeBottleneck::default().finish(0, 8).is_err());
        let mut omitted_eos = EpisodeBottleneck::default();
        omitted_eos.push(1., gradients()?)?;
        assert!(omitted_eos.finish(2, 8).is_err());
        let mut empty_batch = EpisodeBottleneck::default();
        empty_batch.push(1., gradients()?)?;
        assert!(empty_batch.finish(1, 0).is_err());
        let mut changed = EpisodeBottleneck::default();
        changed.push(1., gradients()?)?;
        assert!(changed
            .push(
                1.,
                BTreeMap::from([("other".to_string(), Tensor::new(1f32, &Device::Cpu)?)])
            )
            .is_err());
        let mut oversized = EpisodeBottleneck::default();
        for _ in 0..32 {
            oversized.push(1., gradients()?)?;
        }
        assert!(oversized.push(1., gradients()?).is_err());
        let mut nonfinite_gradient = EpisodeBottleneck::default();
        nonfinite_gradient.push(
            1.,
            BTreeMap::from([("field".to_string(), Tensor::new(f32::NAN, &Device::Cpu)?)]),
        )?;
        let (_, bad_gradient) = nonfinite_gradient.finish(1, 1)?;
        // The existing batch clip admission is the production finite-gradient gate.
        assert!(clip_denominator(&bad_gradient, &Device::Cpu).is_err());
        Ok(())
    }
    fn saved175_lineage_fixture() -> Result<(Value, Value)> {
        // Exercise the actual published checkpoint receipts, including their
        // original fractional-master inventory and objective metadata.
        let artifacts: Value = serde_json::from_str(include_str!(
            "../../../../docs/labs/m2-bottleneck-2026-10-10/artifact-receipts.json"
        ))?;
        let receipt = artifacts["final"].clone();
        let report = json!({
            "updates":receipt["step"],"prior_updates":320,"lineage_step":receipt["lineage_step"],
            "fresh_adam":true,"cross_state_bottleneck":receipt["cross_state_bottleneck"],
            "training_objective":receipt["training_objective"],"loss_scope":"all","credit":"raw_identity",
            "phase_policy":"none;all-target-token-logmeanexp",
            "cross_state_resume":receipt["cross_state_resume"],"initial_receipt":artifacts["initial"],
            "initial":{"continuation_sha256":artifacts["initial"]["continuation_sha256"],"complete":145},
            "final_active_masters":{"continuation.cross_state":receipt["parameters"]["continuation.cross_state"]["sha256"]}
        });
        Ok((report, receipt))
    }

    #[test]
    fn cross_state_saved175_authenticates_bottleneck_ancestry_and_master() -> Result<()> {
        let (report, receipt) = saved175_lineage_fixture()?;
        SAVED175.validate_lineage(&report, &receipt)?;
        for path in [
            "/updates",
            "/prior_updates",
            "/lineage_step",
            "/fresh_adam",
            "/initial_receipt/step",
            "/initial_receipt/lineage_step",
            "/initial_receipt/continuation_sha256",
            "/initial_receipt/parameters",
            "/initial/continuation_sha256",
            "/initial/complete",
            "/phase_policy",
            "/final_active_masters/continuation.cross_state",
            "/cross_state_bottleneck",
            "/training_objective",
            "/loss_scope",
            "/credit",
            "/initial_receipt/cross_state_bottleneck",
            "/initial_receipt/training_objective",
            "/initial_receipt/loss_scope",
            "/initial_receipt/credit",
        ] {
            let mut altered = report.clone();
            *altered
                .pointer_mut(path)
                .ok_or_else(|| bad("fixture report field absent"))? = Value::Null;
            assert!(
                SAVED175.validate_lineage(&altered, &receipt).is_err(),
                "{path}"
            );
        }
        for path in [
            "/step",
            "/lineage_step",
            "/parameters/continuation.cross_state/sha256",
            "/cross_state_bottleneck",
            "/training_objective",
            "/loss_scope",
            "/credit",
        ] {
            let mut altered = receipt.clone();
            *altered
                .pointer_mut(path)
                .ok_or_else(|| bad("fixture receipt field absent"))? = Value::Null;
            assert!(
                SAVED175.validate_lineage(&report, &altered).is_err(),
                "{path}"
            );
        }
        // Keep all three provenance copies consistent: the fixed ancestor pins,
        // not just agreement between report and receipt, must reject changes.
        for key in [
            "report_sha256",
            "manifest_sha256",
            "field_sha256",
            "prior_step",
            "prior_lineage_step",
            "prior_complete",
        ] {
            let mut altered_report = report.clone();
            let mut altered_receipt = receipt.clone();
            let mut prior = report["cross_state_resume"].clone();
            prior[key] = Value::Null;
            altered_report["cross_state_resume"] = prior.clone();
            altered_report["initial_receipt"]["cross_state_resume"] = prior.clone();
            altered_receipt["cross_state_resume"] = prior;
            assert!(
                SAVED175
                    .validate_lineage(&altered_report, &altered_receipt)
                    .is_err(),
                "{key}"
            );
        }
        assert!(SAVED145.validate_lineage(&report, &receipt).is_err());
        Ok(())
    }

    #[test]
    fn cross_state_saved175_baseline_lineage_and_net_bar() -> Result<()> {
        assert_eq!(SAVED175.checkpoint_step, 256);
        assert_eq!(SAVED175.lineage_after(0), 576);
        assert_eq!(SAVED175.lineage_after(256), 832);
        let mut prior = endpoint(&(0..175).collect::<Vec<_>>());
        prior["continuation_sha256"] = json!(SAVED175.field_sha);
        baseline(&prior, Some((&prior, SAVED175)))?;
        assert!(baseline(&prior, Some((&prior, SAVED145))).is_err());
        let mut altered = prior.clone();
        altered["rows"][400]["generated_ids"] = json!([999]);
        assert!(baseline(&altered, Some((&prior, SAVED175))).is_err());
        let next = endpoint(&(1..177).collect::<Vec<_>>());
        let result = outcomes(&prior, &next, Some((&prior, SAVED175)))?;
        assert_eq!(result["initial_complete"], 175);
        assert_eq!(result["final_complete"], 176);
        assert_eq!(result["keep"], true);
        assert_eq!(
            result["lost_complete_ids"].as_array().map(Vec::len),
            Some(1)
        );
        assert_eq!(
            result["gained_complete_ids"].as_array().map(Vec::len),
            Some(2)
        );
        assert_eq!(
            outcomes(&prior, &prior, Some((&prior, SAVED175)))?["keep"],
            false
        );
        assert_eq!(
            outcomes(
                &prior,
                &endpoint(&(0..174).collect::<Vec<_>>()),
                Some((&prior, SAVED175))
            )?["keep"],
            false
        );
        let mut wrong_count = endpoint(&(0..145).collect::<Vec<_>>());
        wrong_count["continuation_sha256"] = json!(SAVED175.field_sha);
        assert!(baseline(&wrong_count, Some((&wrong_count, SAVED175))).is_err());
        Ok(())
    }
}
