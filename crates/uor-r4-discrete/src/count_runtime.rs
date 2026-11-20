//! Serving-only view of the same native count and mixture operators used in
//! the discrete trainer. The separate Cargo target has no GPU build pipeline.
#![forbid(unsafe_code)]

include!("kn_ceiling/runtime/code_pick.rs");

pub mod ngram_count {
    use crate::{invalid, Result};
    include!("kn_ceiling/runtime/key.rs");
}
pub mod kn_score {
    use crate::{
        invalid,
        ngram_count::{NgramContext, NGRAM_VOCABULARY},
        Result,
    };
    include!("kn_ceiling/runtime/interpolation.rs");
}
mod kn_ceiling {
    use crate::{invalid, kn_score::KN_BASE_MASS, ngram_count::NGRAM_MAX_ORDER, Result};
    include!("kn_ceiling/runtime/mixture.rs");
    pub mod compact {
        use super::{
            FrozenPickMixture, PickTarget, PICK_FIRED_EXACT, PICK_FIRED_NEAR, PICK_NOT_FIRED,
        };
        use crate::{
            invalid,
            kn_score::{interpolate_order, KnDiscounts, KN_BASE_MASS},
            ngram_count::{
                NgramContext, PrimeRouteKey, NGRAM_BOS, NGRAM_MAX_ORDER, NGRAM_VOCABULARY,
            },
            Result,
        };
        use sha2::{Digest, Sha256};
        include!("kn_ceiling/runtime/compact.rs");
        include!("kn_ceiling/runtime/rank.rs");
    }
    pub mod frozen_pick;
    pub use frozen_pick::FrozenPickMixture;
}
pub use kn_ceiling::{compact::CompactCountLookup, frozen_pick::FrozenPickMixture, PickTarget};

#[derive(Debug)]
pub struct Error(String);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
fn invalid(message: impl Into<String>) -> Error {
    Error(message.into())
}
pub type Result<T> = std::result::Result<T, Error>;
