//! Discrete geometric model substrate. Model operations have no host implementation.
//! Static geometry compilation is a build dependency, excluded from the runtime graph.
//! The routing atlas is one sixteen-root E8 orbit; this is not full-orbit learnability.
#[cfg(feature = "cuda")]
pub mod device;
pub mod metadata;

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
/// Read configuration is an artifact identity, not an inference-time override.
/// Flock's frame pairing and rank order are fixed by this schema: frame L1,
/// coordinate Hamming, then recency/position, on caller-bounded valid prefixes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReadSpec {
    Route,
    Flock { kept: u32 },
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
        }
        Ok(
            match <Wire as serde::Deserialize>::deserialize(deserializer)? {
                Wire::Route {} => Self::Route,
                Wire::Flock { kept } => Self::Flock { kept },
            },
        )
    }
}
impl ReadSpec {
    pub fn validate(&self, spec: Spec) -> Result<()> {
        spec.validate()?;
        if let Self::Flock { kept } = self {
            if !(1..=8).contains(kept) {
                return Err(Error::Invalid(
                    "flock kept count requires 1..8; selection uses the actual eligible count"
                        .into(),
                ));
            }
        }
        Ok(())
    }
    /// Fixed kernel metadata: mode 0 is route, mode 1 is flock.
    pub(crate) fn words(self) -> [u32; 2] {
        match self {
            Self::Route => [0, 0],
            Self::Flock { kept } => [1, kept],
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
        assert_eq!(ReadSpec::Route.words(), [0, 0]);
        assert_eq!(ReadSpec::Flock { kept: 2 }.words(), [1, 2]);
        let packet = InitConfig {
            spec,
            read_spec: ReadSpec::Flock { kept: 2 },
        };
        let bytes = serde_json::to_vec(&packet).unwrap();
        let restored: InitConfig = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(restored, packet);
        assert_eq!(
            serde_json::from_str::<ReadSpec>(r#"{"kind":"route"}"#).unwrap(),
            ReadSpec::Route
        );
        assert!(serde_json::from_str::<ReadSpec>(r#"{"kind":"route","kept":2}"#).is_err());
        assert!(serde_json::from_str::<ReadSpec>(r#"{"kind":"flock"}"#).is_err());
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
