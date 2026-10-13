//! Read-geometry enum metadata only; historical executors remain excluded.
use serde::{Deserialize, Serialize};
/// Read-score geometry. `Dot` is the retained scaled dot product. `Lorentz`
/// lifts query and key to the hyperboloid x -> (sqrt(1+|x|^2), x) and scores
/// the scaled geodesic distance below a learned radius. Offline F32 training
/// implements both; this runtime serves a quantized Lorentz model through the
/// integer kernel in `crate::lorentz` under [`packed_numerical_contract`].
/// `LorentzAffine` shares the lift but is an offline-only spacing control.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadGeometry {
    #[default]
    Dot,
    Lorentz,
    /// Offline radius-preserving control with affine distance spacing.
    LorentzAffine,
}

impl ReadGeometry {
    pub const fn is_dot(&self) -> bool {
        matches!(self, Self::Dot)
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Dot => "dot",
            Self::Lorentz => "lorentz",
            Self::LorentzAffine => "lorentz_affine",
        }
    }
}
