//! Admission of the two retained learned parents and their controlled replays.
use super::*;

#[derive(Clone, Deserialize)]
pub(super) struct LearnedRow {
    pub index: usize,
    pub episode: Episode,
    pub float_prediction: u32,
    pub native_prediction: u32,
    pub answer_logits: BTreeMap<String, Vec<f32>>,
}
#[derive(Clone, Deserialize)]
pub(super) struct OracleRow {
    pub index: usize,
    pub episode: Episode,
    pub baseline_prediction: u32,
    pub projected_prediction: u32,
    pub baseline_answer_logits: Vec<f32>,
}
#[derive(Clone)]
pub(super) struct PanelReference {
    pub learned: Vec<LearnedRow>,
    pub oracle: Vec<OracleRow>,
}
pub(super) struct Continuation {
    pub updates: usize,
    pub rng: u64,
    pub original: PanelReference,
    pub stress: PanelReference,
    pub value_source: PathBuf,
    pub native_values: PathBuf,
    pub metadata: Value,
}

fn pinned_json(path: &Path, expected: &str) -> Result<Value> {
    let bytes = fs::read(path)?;
    if sha256_bytes(&bytes) != expected {
        return Err(invalid(format!(
            "retained continuation identity differs: {}",
            path.display()
        )));
    }
    Ok(serde_json::from_slice(&bytes)?)
}
fn same_logits(actual: &(u32, Vec<f32>), prediction: u32, logits: &[f32]) -> bool {
    actual.0 == prediction
        && actual.1.len() == logits.len()
        && actual
            .1
            .iter()
            .map(|x| x.to_bits())
            .eq(logits.iter().map(|x| x.to_bits()))
}
impl PanelReference {
    pub fn validate_initial(
        &self,
        index: usize,
        float: &(u32, Vec<f32>),
        native: &(u32, Vec<f32>),
        donor: &(u32, Vec<f32>),
    ) -> Result<()> {
        let row = self
            .learned
            .get(index)
            .ok_or_else(|| invalid("parent row absent"))?;
        let oracle = self
            .oracle
            .get(index)
            .ok_or_else(|| invalid("teacher replay row absent"))?;
        if !same_logits(
            float,
            row.float_prediction,
            row.answer_logits
                .get("float")
                .ok_or_else(|| invalid("parent float logits absent"))?,
        ) || !same_logits(
            native,
            row.native_prediction,
            row.answer_logits
                .get("native")
                .ok_or_else(|| invalid("parent native logits absent"))?,
        ) || !same_logits(
            donor,
            oracle.baseline_prediction,
            &oracle.baseline_answer_logits,
        ) {
            return Err(invalid("continued float/native parent or new-address donor logits differ from retained rows"));
        }
        Ok(())
    }
}
impl Continuation {
    pub fn load(a: &Args, original: &[Episode], stress: &[Episode]) -> Result<Option<Self>> {
        let Some(root) = &a.continuation_parent else {
            return Ok(None);
        };
        let replay = a
            .teacher_replay
            .as_ref()
            .ok_or_else(|| invalid("teacher replay absent"))?;
        report_output::verify(root).map_err(|e| invalid(e.to_string()))?;
        report_output::verify(replay).map_err(|e| invalid(e.to_string()))?;
        // Exact accepted records, including their actual saved RNG state. The
        // state is read by serde_json as u64, never through a floating parser.
        const PINS: [[&str; 7]; 2] = [
            [
                "7daa9f3e6105f8e1d204c0c9b8d1085aba6386eb76fea157a158ae0cd5d17f5b",
                "170d893107b7185c5ec2c00b599a7ffe00930ff0801f692105dcf68a70b33214",
                "d00ccbb8c20d3e21f4663f445d9821a8e91f9e10aed7ef23414845499c86e6ff",
                "df188af7866bc0ae14762f7731d5bb1707fed1588c795acea053b6e9621a82a1",
                "bfc3df63dbae70c0ba4581c01f35f6331fc941c5a6a625cd470d9eee306056ca",
                "5fc3bc9c74408629d92de1a797fd84be3752a2d9d3723561fb04e858cae2239b",
                "4baa8a2a00f217e3c5ed38cef8101b74b72cbdd97ba96dadd55f93f894024b3c",
            ],
            [
                "dd15dec70edb39f2bf950662b05a362caf4364f24d3800926cc403a02947daea",
                "1bdb4c8dc495a6fb3b61fa0b1915479981500f40b0e4a564f7a2a696cb73131e",
                "65a4980b3e689b4113d380e4b44c7383a9eba7d4936ed2a63e56b69d0389297e",
                "3a20a5c33c7d4a9ca2c72a0cfec1c91412e1698a356ac6a32a57fa08c1135c51",
                "a7ac19cf92b6f1a55449edb7f7bdd8c3f09459fdd65da693c7d23c48654b36ef",
                "2082850cd1ed3ab51bbb2663078435ccfefd15e86f844e5d6671d50dc32545a6",
                "bd13b44eca4d6be6b9d97516c76e851a3fa8ec3c67c42617ab83619e6edf2f54",
            ],
        ];
        let pins = PINS[(a.seed - 1) as usize];
        let report = pinned_json(&root.join("report.json"), pins[0])?;
        let checkpoint = pinned_json(&root.join("trained/checkpoint.json"), pins[1])?;
        let replay_report = pinned_json(&replay.join("report.json"), pins[2])?;
        if report["complete"] != true
            || report["seed"] != a.seed
            || report["completed_updates"] != 640
            || checkpoint["completed_updates"] != 640
            || replay_report["complete"] != true
            || replay_report["representation"] != "k2-first-fixed-greedy-residual"
            || a.rate.to_bits() != 0.003f64.to_bits()
        {
            return Err(invalid(
                "continuation completed dose/seed/replay/rate contract differs",
            ));
        }
        for (actual, wanted) in [
            (&a.context_source, root.join("trained/context-source")),
            (&a.context_native, root.join("trained/native-context")),
        ] {
            if fs::canonicalize(actual)? != fs::canonicalize(wanted)? {
                return Err(invalid(
                    "context must be the selected learned parent's trained artifact",
                ));
            }
        }
        let mut panels = Vec::new();
        for (i, (name, episodes)) in [("original", original), ("stress", stress)]
            .into_iter()
            .enumerate()
        {
            let learned: Vec<LearnedRow> = serde_json::from_value(pinned_json(
                &root.join(name).join("rows.json"),
                pins[3 + i],
            )?)?;
            let oracle: Vec<OracleRow> = serde_json::from_value(pinned_json(
                &replay.join(name).join("rows.json"),
                pins[5 + i],
            )?)?;
            if learned.len() != episodes.len() || oracle.len() != episodes.len() {
                return Err(invalid("continuation panels incomplete"));
            }
            for (index, ((l, o), e)) in learned.iter().zip(&oracle).zip(episodes).enumerate() {
                if l.index != index || o.index != index || &l.episode != e || &o.episode != e {
                    return Err(invalid("continuation episode identity differs"));
                }
                for (prediction, logits) in [
                    (l.float_prediction, l.answer_logits.get("float")),
                    (l.native_prediction, l.answer_logits.get("native")),
                    (o.baseline_prediction, Some(&o.baseline_answer_logits)),
                ] {
                    let logits = logits.ok_or_else(|| invalid("parent logits missing"))?;
                    if logits.len() != 40 || argmax(logits)? != prediction {
                        return Err(invalid("parent logits/prediction invalid"));
                    }
                }
            }
            panels.push(PanelReference { learned, oracle });
        }
        let value_source = root.join("trained/value-source");
        let native_values = root.join("trained/native-values");
        let expected_value = [
            "eb4ea56648b1ec0d3ffb982fc4e65f957dcecbf318974ca7723ab4ebfe11a7ef",
            "a11472805275fd469ff38c826546f450246836d94840e43385a35a0c6c34d843",
        ][(a.seed - 1) as usize];
        if sha256_file(&value_source.join("value-parameters.safetensors"))? != expected_value {
            return Err(invalid("accepted learned value weights differ"));
        }
        let rng = checkpoint["next_generator_state"]
            .as_u64()
            .filter(|x| *x != 0)
            .ok_or_else(|| invalid("exact saved generator u64 missing"))?;
        let updates = checkpoint["completed_updates"]
            .as_u64()
            .and_then(|x| usize::try_from(x).ok())
            .ok_or_else(|| invalid("parent update count"))?;
        let metadata = json!({"schema":"uor-r4.query-credit-continuation-parent/1","root":root,"teacher_replay":replay,"report_sha256":pins[0],"checkpoint_sha256":pins[1],"replay_report_sha256":pins[2],"panel_sha256":&pins[3..],"value_source":file_inventory(&value_source)?,"native_values":file_inventory(&native_values)?,"parent_updates":updates,"next_generator_state":rng,"optimizer":"fresh AdamW moments in BOTH arms; saved moments unavailable; not exact optimizer resume","teacher":"fixed loaded parent compiled context; does not follow evolving student","joint_scope":"trainable transitions can change addresses; comparison is joint credit allocation, not isolated payload-head causality"});
        let stress = panels
            .pop()
            .ok_or_else(|| invalid("stress reference absent"))?;
        let original = panels
            .pop()
            .ok_or_else(|| invalid("original reference absent"))?;
        Ok(Some(Self {
            updates,
            rng,
            original,
            stress,
            value_source,
            native_values,
            metadata,
        }))
    }
    pub fn values(
        &self,
        a: &Args,
        f: &Frozen,
        c: &ContextWeights,
    ) -> Result<(ValueProducerWeights, CompiledValueProducer)> {
        let values = ValueProducerWeights::load_source(&self.value_source)?;
        let native = CompiledValueProducer::load(
            &self.native_values,
            ValueProducerSourcePaths {
                value_source: &self.value_source,
                context_source: &a.context_source,
                context_dependencies: a.dependencies(),
            },
            &f.tokenizer,
        )?;
        native.validate_for(&values, c)?;
        native.validate_native_context(&f.parent)?;
        Ok((values, native))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn continuation_rng_is_exact_u64_and_logit_binding_is_bitwise() -> Result<()> {
        let v: Value = serde_json::from_str("{\"next_generator_state\":13803231091573875691}")?;
        assert_eq!(
            v["next_generator_state"].as_u64(),
            Some(13803231091573875691)
        );
        let actual = (1, vec![0f32, 1.]);
        assert!(same_logits(&actual, 1, &[0., 1.]));
        assert!(!same_logits(&actual, 1, &[-0., 1.]));
        assert!(!same_logits(&actual, 0, &[0., 1.]));
        Ok(())
    }
}
