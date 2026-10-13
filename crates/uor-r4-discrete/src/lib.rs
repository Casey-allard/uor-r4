//! Discrete geometric model substrate. Model operations have no host implementation.
//! Static geometry compilation is a build dependency, excluded from the runtime graph.
//! The routing atlas is one sixteen-root E8 orbit; this is not full-orbit learnability.
#[cfg(feature = "cuda")]
pub mod device;
pub mod metadata;

pub const SCHEMA: &str = "uor-r4.discrete-spiralcore-fbs/1";
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
    /// Reserved for the independent discrete murmuration read; no gate law
    /// is implemented by this library yet. CUDA kind 3, never kind 2 (none).
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
}

#[cfg(feature = "cuda")]
pub mod qualification;
