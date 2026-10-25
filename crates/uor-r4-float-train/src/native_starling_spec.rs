//! Canonical native Starling configuration metadata only.
//! The CUDA implementation must re-export these types, not define a shadow copy.
//! Parsed valid configuration and serialized identities match the retained Starling source.
use crate::{invalid, Result};
use serde::{Deserialize, Serialize};

/// The `read_route=` prefix of the native read.
pub const ROUTE_PREFIX: &str = "starling_native";
/// An unfilled slot in K1's output.
pub const ABSENT: u32 = u32::MAX;
/// K + E may not exceed this (the kernel's register list).
pub const MAX_SLOTS: usize = 16;
/// Vertices of the 600-cell (the icosian codebook K1 addresses).
pub const CELLS: usize = 120;

/// The declared surrogate constants (A1 section B). The fixture values are the
/// card's; a training run registers its own in its sheet.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Credit {
    /// Score margin `mu` of the pairwise hinge.
    pub margin: f32,
    /// Utility gap `delta` a pair must exceed (strictly).
    pub delta: f32,
    /// Address margin `mu_a`.
    pub address_margin: f32,
    /// Coefficient of the rank hinge.
    pub rank: f32,
    /// Coefficient of the address margin.
    pub address: f32,
}

impl Credit {
    /// The constants of the card's finite fixtures (A1 section B, with the
    /// FIT review's binding correction 6148141029: delta = 0.125, exactly
    /// representable, so the strict boundary case is exact in F32).
    pub const FIXTURE: Credit = Credit {
        margin: 1.0,
        delta: 0.125,
        address_margin: 0.1,
        rank: 1.0,
        address: 1.0,
    };

    fn validate(&self) -> Result<()> {
        let values = [
            self.margin,
            self.delta,
            self.address_margin,
            self.rank,
            self.address,
        ];
        if values.iter().any(|v| !v.is_finite() || *v < 0.0) {
            return Err(invalid(
                "starling credit constants must be finite and non-negative",
            ));
        }
        Ok(())
    }
}

/// The native read's configuration, bound into the artifact's `config.json`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NativeStarling {
    /// Ring depth `M` per vertex.
    pub depth: usize,
    /// Kept slots `K`.
    pub kept: usize,
    /// Exploration slots `E` (zero forward weight).
    pub explore: usize,
    /// The tokenizer's BOS identity: every ring clears at a document start.
    pub bos_id: u32,
    pub credit: Credit,
}

impl NativeStarling {
    /// `starling_native:M:K:E` with the given BOS identity and credit.
    pub fn parse(text: &str, bos_id: u32, credit: Credit) -> Result<Self> {
        let parts: Vec<&str> = text.split(':').collect();
        let numbers = match parts.as_slice() {
            [ROUTE_PREFIX, m, k, e] => [m, k, e]
                .iter()
                .map(|v| v.parse::<usize>())
                .collect::<std::result::Result<Vec<_>, _>>()
                .ok(),
            _ => None,
        };
        let Some(numbers) = numbers else {
            return Err(invalid(format!(
                "invalid native read route {text} (starling_native:M:K:E)"
            )));
        };
        let spec = Self {
            depth: numbers[0],
            kept: numbers[1],
            explore: numbers[2],
            bos_id,
            credit,
        };
        spec.validate()?;
        Ok(spec)
    }

    pub fn validate(&self) -> Result<()> {
        let slots = self
            .kept
            .checked_add(self.explore)
            .ok_or_else(|| invalid("starling_native K + E overflows the slot inventory"))?;
        if self.depth == 0 || self.kept == 0 || slots > MAX_SLOTS {
            return Err(invalid(format!(
                "starling_native needs M >= 1, K >= 1 and K + E <= {MAX_SLOTS}"
            )));
        }
        self.credit.validate()
    }

    pub fn name(&self) -> String {
        format!(
            "{ROUTE_PREFIX}:{}:{}:{}",
            self.depth, self.kept, self.explore
        )
    }

    /// Kept plus exploration slots K1 returns per query.
    pub fn slots(&self) -> usize {
        self.kept + self.explore
    }
}
