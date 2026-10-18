//! Discrete geometric model substrate. Model operations have no host implementation.
//! Static geometry compilation is a build dependency, excluded from the runtime graph.
//! The routing atlas is one sixteen-root E8 orbit; this is not full-orbit learnability.
pub mod baseline_reuse;
#[cfg(test)]
mod baseline_reuse_source;
pub mod controller;
pub mod corpus;
pub mod coverage_census;
pub mod decomposition;
#[cfg(test)]
mod decomposition_source;
#[cfg(feature = "cuda")]
pub mod device;
#[cfg(feature = "cuda")]
pub mod driver_qualification;
#[cfg(feature = "cuda")]
pub mod measurement;
pub mod metadata;
pub mod objective;
pub mod proposal_timing;
pub mod rows_source;
pub mod rowsk_source;
pub use objective::ObjectivePolicy;
pub mod training;

pub const SCHEMA: &str = "uor-r4.discrete-spiralcore-fbs/2";
#[derive(Debug)]
pub enum Error {
    Invalid(String),
    Cuda(String),
    TrainingHeld,
    UnimplementedParameter { kind: u32, index: u32 },
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(s) => write!(f, "invalid discrete model: {s}"),
            Self::Cuda(s) => write!(f, "CUDA: {s}"),
            Self::TrainingHeld => f.write_str("training remains HELD; source delivery and finite construction controls do not release the owner hold"),
            Self::UnimplementedParameter { kind, index } => write!(f, "parameter kind {kind} index {index} is reserved but unimplemented"),
        }
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    pub vocabulary: u32,
    pub blocks: u32,
    pub context: u32,
    pub kept_cells: u32,
}
impl Spec {
    pub fn validate(&self) -> Result<()> {
        if !(2..=8192).contains(&self.vocabulary)
            || !(2..=16).contains(&self.blocks)
            || !(1..=243).contains(&self.context)
            || self.kept_cells == 0
            || self.kept_cells > 16
            || self.kept_cells > self.vocabulary
        {
            return Err(Error::Invalid("requires vocabulary2..8192, whole8D blocks2..16, context1..243, kept cells1..min(16,vocabulary)".into()));
        }
        Ok(())
    }
}
pub const RELATION_EDGE_TYPES: u32 = 5;

/// Initial read connectivity is a declared experiment setting. The historical
/// zero-gate default remains unchanged; active leaves carry no target-derived
/// information and must be matched between Relation and its frozen-zero control.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadInitialization {
    #[default]
    ZeroGatesV1,
    ActiveLeafV1,
}
impl ReadInitialization {
    pub fn is_zero_gates(&self) -> bool {
        *self == Self::ZeroGatesV1
    }
    pub fn validate(self, read_spec: ReadSpec) -> Result<()> {
        if self == Self::ActiveLeafV1 && read_spec == ReadSpec::Route {
            return Err(Error::Invalid(
                "active leaf initialization requires Flock or Relation".into(),
            ));
        }
        Ok(())
    }
}

/// Read configuration is an artifact identity, not an inference-time override.
/// Flock's frame pairing and rank order are fixed by this schema: frame L1,
/// coordinate Hamming, then recency/position, on caller-bounded valid prefixes.
/// Relation extends Flock with bounded S-sweep exact-relation reranking over
/// all causally eligible BinderPlan nodes using existing Gate-kind trits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReadSpec {
    Route,
    Flock { kept: u32 },
    Relation { kept: u32, sweeps: u32 },
}
impl<'de> serde::Deserialize<'de> for ReadSpec {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        // Serde's internally tagged unit variant accepts extra map fields even
        // with deny_unknown_fields. An empty struct variant makes Route strict.
        #[derive(serde::Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        enum Wire {
            Route {},
            Flock { kept: u32 },
            Relation { kept: u32, sweeps: u32 },
        }
        Ok(
            match <Wire as serde::Deserialize>::deserialize(deserializer)? {
                Wire::Route {} => Self::Route,
                Wire::Flock { kept } => Self::Flock { kept },
                Wire::Relation { kept, sweeps } => Self::Relation { kept, sweeps },
            },
        )
    }
}
impl ReadSpec {
    pub fn validate(&self, spec: Spec) -> Result<()> {
        spec.validate()?;
        match self {
            Self::Route => {}
            Self::Flock { kept } => {
                if !(1..=8).contains(kept) {
                    return Err(Error::Invalid(
                        "flock kept count requires 1..8; selection uses the actual eligible count"
                            .into(),
                    ));
                }
            }
            Self::Relation { kept, sweeps } => {
                if !(1..=8).contains(kept) {
                    return Err(Error::Invalid(
                        "relation kept count must be in 1..=8".into(),
                    ));
                }
                if !(1..=5).contains(sweeps) {
                    return Err(Error::Invalid(
                        "relation sweep count must be in 1..=5".into(),
                    ));
                }
            }
        }
        Ok(())
    }
    /// Fixed kernel metadata: mode 0 is route, mode 1 is flock, mode 2 is relation.
    pub(crate) fn words(self) -> [u32; 3] {
        match self {
            Self::Route => [0, 0, 0],
            Self::Flock { kept } => [1, kept, 0],
            Self::Relation { kept, sweeps } => [2, kept, sweeps],
        }
    }
    /// Gate trit inventory for the given block count and binder stage count.
    pub fn gate_count(self, blocks: u32, stages: u32) -> Result<u32> {
        let base = blocks
            .checked_mul(stages)
            .ok_or_else(|| Error::Invalid("gate inventory overflow".into()))?;
        match self {
            Self::Route | Self::Flock { .. } => Ok(base),
            Self::Relation { .. } => {
                let relation = RELATION_EDGE_TYPES
                    .checked_mul(stages)
                    .ok_or_else(|| Error::Invalid("gate inventory overflow".into()))?;
                base.checked_add(relation)
                    .ok_or_else(|| Error::Invalid("gate inventory overflow".into()))
            }
        }
    }
}
/// Explicit initialization configuration for the `init-read` CLI. No default
/// read mode is accepted in this packet; legacy `init` explicitly selects Route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InitConfig {
    pub spec: Spec,
    pub read_spec: ReadSpec,
    #[serde(
        default,
        skip_serializing_if = "ObjectivePolicy::is_nearest_address_margin"
    )]
    pub objective_policy: ObjectivePolicy,
}
/// Record version and occurrence are identities, not a geometric metric or model epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub token: u32,
    pub occurrence: u64,
    pub record: u64,
    pub version: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parameter {
    Route(u32),
    VectorBlock(u32),
    /// Flock trit at block*stages+level, including atom level zero. CUDA kind 3,
    /// never kind 2 (none); Route models explicitly refuse this parameter.
    Gate(u32),
}

/// All public corpus-training entry points retain the owner hold.
pub fn training_admission() -> Result<()> {
    Err(Error::TrainingHeld)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn apply_calls_are_confined_to_bounded_fixture_and_admitted_transition() -> Result<()> {
        use std::{collections::BTreeMap, fs, path::Path};
        fn inventory(
            root: &Path,
            directory: &Path,
            found: &mut BTreeMap<(String, String), usize>,
        ) -> Result<()> {
            let needle = [".", "apply", "("].concat();
            for entry in fs::read_dir(directory).map_err(|e| Error::Invalid(e.to_string()))? {
                let entry = entry.map_err(|e| Error::Invalid(e.to_string()))?;
                let path = entry.path();
                let kind = entry
                    .file_type()
                    .map_err(|e| Error::Invalid(e.to_string()))?;
                if kind.is_dir() {
                    inventory(root, &path, found)?;
                } else if path
                    .extension()
                    .is_some_and(|extension| extension == "rs" || extension == "cu")
                {
                    let text =
                        fs::read_to_string(&path).map_err(|e| Error::Invalid(e.to_string()))?;
                    let relative = path
                        .strip_prefix(root)
                        .map_err(|e| Error::Invalid(e.to_string()))?
                        .to_string_lossy()
                        .replace('\\', "/");
                    for line in text.lines() {
                        let count = line.matches(&needle).count();
                        if count != 0 {
                            *found
                                .entry((relative.clone(), line.trim().to_owned()))
                                .or_default() += count;
                        }
                    }
                }
            }
            Ok(())
        }
        // Read source bytes only, including CUDA-gated modules. No context,
        // model, fixture forward or kernel is instantiated by this test.
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut observed = BTreeMap::new();
        inventory(&root, &root, &mut observed)?;
        let expected = BTreeMap::from([
            (
                (
                    "driver_qualification.rs".into(),
                    ["let applied = model", ".", "apply", "(proposal)?;"].concat(),
                ),
                1,
            ),
            (
                (
                    "training.rs".into(),
                    // The existing sole transition now returns the guarded
                    // apply expression; frozen relation slots bypass it.
                    ["model", ".", "apply", "(&proposal)?"].concat(),
                ),
                1,
            ),
        ]);
        assert_eq!(
            observed, expected,
            "update reach changed; preserve the owner hold and fixture admission"
        );
        let main =
            fs::read_to_string(root.join("main.rs")).map_err(|e| Error::Invalid(e.to_string()))?;
        assert!(!main.contains(&[".", "apply", "("].concat()));
        let device = fs::read_to_string(root.join("device.rs"))
            .map_err(|e| Error::Invalid(e.to_string()))?;
        assert!(device.contains("pub(crate) fn apply("));
        let fixture = fs::read_to_string(root.join("driver_qualification.rs"))
            .map_err(|e| Error::Invalid(e.to_string()))?;
        let helper = fixture
            .split("fn apply(context:")
            .nth(1)
            .and_then(|text| text.split("fn isolated_apply_boundary(").next())
            .ok_or_else(|| Error::Invalid("bounded fixture helper source absent".into()))?;
        assert!(helper.contains("if context.apply_calls >= 11"));
        assert_eq!(helper.matches(&[".", "apply", "("].concat()).count(), 1);
        let training = fs::read_to_string(root.join("training.rs"))
            .map_err(|e| Error::Invalid(e.to_string()))?;
        let transition = training
            .split("pub fn transition(")
            .nth(1)
            .and_then(|text| text.split("pub fn run_transitions(").next())
            .ok_or_else(|| Error::Invalid("admission-gated transition source absent".into()))?;
        assert!(transition.contains("admission: &CorpusUpdateAdmission"));
        assert!(transition.contains("if admission.lineage_sha256 != self.lineage_sha256()?"));
        Ok(())
    }
    #[test]
    fn configuration_and_hold_are_metadata_only() {
        assert!(Spec {
            vocabulary: 256,
            blocks: 2,
            context: 27,
            kept_cells: 8
        }
        .validate()
        .is_ok());
        assert!(Spec {
            vocabulary: 256,
            blocks: 0,
            context: 27,
            kept_cells: 8
        }
        .validate()
        .is_err());
        assert!(matches!(training_admission(), Err(Error::TrainingHeld)));
    }
    #[test]
    fn read_configuration_is_explicit_bounded_metadata() {
        let spec = Spec {
            vocabulary: 4,
            blocks: 2,
            context: 3,
            kept_cells: 2,
        };
        assert!(ReadSpec::Route.validate(spec).is_ok());
        assert!(ReadSpec::Flock { kept: 3 }.validate(spec).is_ok());
        assert!(ReadSpec::Flock { kept: 0 }.validate(spec).is_err());
        assert!(ReadSpec::Flock { kept: 8 }.validate(spec).is_ok());
        assert!(ReadSpec::Flock { kept: 9 }.validate(spec).is_err());
        assert!(ReadSpec::Relation { kept: 1, sweeps: 1 }
            .validate(spec)
            .is_ok());
        assert!(ReadSpec::Relation { kept: 8, sweeps: 5 }
            .validate(spec)
            .is_ok());
        assert!(ReadSpec::Relation { kept: 0, sweeps: 1 }
            .validate(spec)
            .is_err());
        assert!(ReadSpec::Relation { kept: 9, sweeps: 1 }
            .validate(spec)
            .is_err());
        assert!(ReadSpec::Relation { kept: 2, sweeps: 0 }
            .validate(spec)
            .is_err());
        assert!(ReadSpec::Relation { kept: 2, sweeps: 6 }
            .validate(spec)
            .is_err());
        assert_eq!(ReadSpec::Route.words(), [0, 0, 0]);
        assert_eq!(ReadSpec::Flock { kept: 2 }.words(), [1, 2, 0]);
        assert_eq!(ReadSpec::Relation { kept: 2, sweeps: 1 }.words(), [2, 2, 1]);
        assert_eq!(ReadSpec::Route.gate_count(4, 5).unwrap(), 20);
        assert_eq!(ReadSpec::Flock { kept: 8 }.gate_count(4, 5).unwrap(), 20);
        assert_eq!(
            ReadSpec::Relation { kept: 8, sweeps: 1 }
                .gate_count(4, 5)
                .unwrap(),
            45
        );
        let packet = InitConfig {
            objective_policy: ObjectivePolicy::NearestAddressMarginV1,
            spec,
            read_spec: ReadSpec::Flock { kept: 2 },
        };
        let bytes = serde_json::to_vec(&packet).unwrap();
        let restored: InitConfig = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(restored, packet);
        let rel_packet = InitConfig {
            objective_policy: ObjectivePolicy::NearestAddressMarginV1,
            spec,
            read_spec: ReadSpec::Relation { kept: 2, sweeps: 1 },
        };
        let rel_bytes = serde_json::to_vec(&rel_packet).unwrap();
        let rel_restored: InitConfig = serde_json::from_slice(&rel_bytes).unwrap();
        assert_eq!(rel_restored, rel_packet);
        assert_eq!(
            serde_json::from_str::<ReadSpec>(r#"{"kind":"route"}"#).unwrap(),
            ReadSpec::Route
        );
        assert_eq!(
            serde_json::from_str::<ReadSpec>(r#"{"kind":"relation","kept":2,"sweeps":1}"#).unwrap(),
            ReadSpec::Relation { kept: 2, sweeps: 1 }
        );
        assert!(serde_json::from_str::<ReadSpec>(r#"{"kind":"route","kept":2}"#).is_err());
        assert!(serde_json::from_str::<ReadSpec>(r#"{"kind":"flock"}"#).is_err());
        assert!(serde_json::from_str::<ReadSpec>(r#"{"kind":"relation","kept":2}"#).is_err());
        assert!(serde_json::from_str::<InitConfig>(
            r#"{"spec":{"vocabulary":4,"blocks":2,"context":3,"kept_cells":2}}"#
        )
        .is_err());
    }
}

#[cfg(feature = "cuda")]
pub mod flock_qualification;
#[cfg(feature = "cuda")]
pub mod qualification;
#[cfg(feature = "cuda")]
pub mod relation_qualification;
#[cfg(feature = "cuda")]
pub mod rows_qualification;
#[cfg(feature = "cuda")]
pub mod rowsk_qualification;
