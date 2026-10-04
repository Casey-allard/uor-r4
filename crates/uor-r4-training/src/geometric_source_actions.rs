//! Compatibility export of the shared checked integer source runtime.

pub use uor_r4_integer::geometric_source_actions::*;

impl From<uor_r4_integer::geometric_source_realizer::SourceRuntimeError> for crate::TrainingError {
    fn from(error: uor_r4_integer::geometric_source_realizer::SourceRuntimeError) -> Self {
        crate::invalid(error.to_string())
    }
}
