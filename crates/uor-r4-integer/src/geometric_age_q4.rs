//! Four-bit independent temporal biases in fixed quarter-nat units.
//!
//! Admission regenerates head-major [head, lag] Q24 entries. Lag zero is
//! retained; no slope, bucketing, centering or semantic metric is introduced.
//! This codec changes coefficient storage, not causal support or normalization.
//!
//! The separately named residual codec retains the fixed Stack initialization
//! prior and uses independent eighth-nat q4 residuals. It does not reinterpret
//! the raw quarter-nat codec or learn a prior slope.
use crate::geometric_context_q4::{unpack_coefficients, ContextQ4Error};
use serde::{Deserialize, Serialize};
use std::fmt;

pub const POLICY: &str = "signed-q4[-7,7];reserved-minus8;independent-head-lag;fixed-quarter-nat;exact-shift22;lag0-retained;no-centering/1";
pub const SCHEMA: &str = "uor-r4.geometric-age-q4/1";
pub const RESIDUAL_SCHEMA: &str = "uor-r4.geometric-age-prior-residual-q4/1";
pub const RESIDUAL_POLICY: &str = "signed-q4[-7,7];reserved-minus8;independent-head-lag-residual;fixed-eighth-nat;exact-prior-plus-shift21;lag0-retained;no-clipping-or-centering/1";
pub const PRIOR_IDENTITY: &str = "stack-read-age-initialization:-lag*2^(-8*(head+1)/heads);H1-shift8;H2-ordered-shifts4,8;fixed-not-learned/1";
pub const PRIOR_PHASE: &str =
    "head-major;lag=query-position-minus-source-position;current-occurrence-lag0/1";
pub const RESIDUAL_EXPONENT: i32 = -3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgeQ4Config {
    pub heads: usize,
    pub context: usize,
}
impl AgeQ4Config {
    pub fn validate(self) -> Result<(), AgeQ4Error> {
        if !(1..=2).contains(&self.heads) || !(1..=128).contains(&self.context) {
            return Err(AgeQ4Error::Configuration);
        }
        Ok(())
    }
    pub fn coefficient_count(self) -> Result<usize, AgeQ4Error> {
        self.validate()?;
        Ok(self.heads * self.context)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgeQ4Error {
    Configuration,
    Packed(ContextQ4Error),
    ArithmeticOverflow,
}
impl fmt::Display for AgeQ4Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "geometric age q4: {self:?}")
    }
}
impl std::error::Error for AgeQ4Error {}

pub struct NativeAgeQ4 {
    config: AgeQ4Config,
    packed: Vec<u8>,
    age_q24: Vec<i64>,
}
impl NativeAgeQ4 {
    pub fn new(config: AgeQ4Config, packed: &[u8]) -> Result<Self, AgeQ4Error> {
        let coefficients =
            unpack_coefficients(config.coefficient_count()?, packed).map_err(AgeQ4Error::Packed)?;
        Ok(Self {
            config,
            packed: packed.to_vec(),
            age_q24: coefficients
                .into_iter()
                .map(|q| i64::from(q) << 22)
                .collect(),
        })
    }
    pub fn config(&self) -> AgeQ4Config {
        self.config
    }
    pub fn packed(&self) -> &[u8] {
        &self.packed
    }
    pub fn age_q24(&self) -> &[i64] {
        &self.age_q24
    }
}

/// Only exact dyadic specializations of the fixed initialization are admitted.
/// Head ordering, slope and phase cannot be supplied independently by a caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgeResidualQ4Config {
    pub heads: usize,
    pub context: usize,
}
impl AgeResidualQ4Config {
    pub fn validate(self) -> Result<(), AgeQ4Error> {
        AgeQ4Config {
            heads: self.heads,
            context: self.context,
        }
        .validate()
    }
    pub fn coefficient_count(self) -> Result<usize, AgeQ4Error> {
        self.validate()?;
        Ok(self.heads * self.context)
    }
    pub fn prior_head_shifts(self) -> Result<Vec<u32>, AgeQ4Error> {
        self.validate()?;
        Ok(if self.heads == 1 { vec![8] } else { vec![4, 8] })
    }
    /// Admission-time expansion; no floating-point prior reconstruction.
    pub fn prior_q24(self) -> Result<Vec<i64>, AgeQ4Error> {
        let shifts = self.prior_head_shifts()?;
        let mut values = Vec::with_capacity(self.coefficient_count()?);
        for shift in shifts {
            for lag in 0..self.context {
                // lag<=127, shift in {4,8}: this positive shift cannot lose bits.
                let scaled = (lag as i64) << (24 - shift);
                values.push(scaled.checked_neg().ok_or(AgeQ4Error::ArithmeticOverflow)?);
            }
        }
        Ok(values)
    }
}

pub struct NativeAgeResidualQ4 {
    config: AgeResidualQ4Config,
    packed: Vec<u8>,
    prior_q24: Vec<i64>,
    age_q24: Vec<i64>,
}
impl NativeAgeResidualQ4 {
    pub fn new(config: AgeResidualQ4Config, packed: &[u8]) -> Result<Self, AgeQ4Error> {
        let coefficients =
            unpack_coefficients(config.coefficient_count()?, packed).map_err(AgeQ4Error::Packed)?;
        let prior_q24 = config.prior_q24()?;
        let age_q24 = prior_q24
            .iter()
            .zip(coefficients)
            .map(|(&prior, q)| {
                prior
                    .checked_add(i64::from(q) << 21)
                    .ok_or(AgeQ4Error::ArithmeticOverflow)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            config,
            packed: packed.to_vec(),
            prior_q24,
            age_q24,
        })
    }
    pub fn config(&self) -> AgeResidualQ4Config {
        self.config
    }
    pub fn packed(&self) -> &[u8] {
        &self.packed
    }
    pub fn prior_head_shifts(&self) -> Result<Vec<u32>, AgeQ4Error> {
        self.config.prior_head_shifts()
    }
    pub fn prior_q24(&self) -> &[i64] {
        &self.prior_q24
    }
    pub fn age_q24(&self) -> &[i64] {
        &self.age_q24
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_no_read::pack_coefficients;
    type TestResult = Result<(), Box<dyn std::error::Error>>;
    #[test]
    fn age_q4_exact_units_head_lag_zero_and_bounds() -> TestResult {
        let coefficients = [-7, -1, 0, 1, 7, 2];
        let codec = NativeAgeQ4::new(
            AgeQ4Config {
                heads: 2,
                context: 3,
            },
            &pack_coefficients(&coefficients)?,
        )?;
        assert_eq!(
            codec.age_q24(),
            &[-29360128, -4194304, 0, 4194304, 29360128, 8388608]
        );
        assert_eq!(codec.config().coefficient_count()?, coefficients.len());
        assert_eq!(codec.age_q24()[0], -7i64 << 22);
        assert_eq!(codec.age_q24()[3], 1i64 << 22);
        Ok(())
    }
    #[test]
    fn age_q4_rejects_reserved_padding_shape_and_length() {
        let config = AgeQ4Config {
            heads: 1,
            context: 1,
        };
        assert!(NativeAgeQ4::new(config, &[8]).is_err());
        assert!(NativeAgeQ4::new(config, &[0x10]).is_err());
        assert!(NativeAgeQ4::new(config, &[]).is_err());
        for config in [
            AgeQ4Config {
                heads: 0,
                context: 1,
            },
            AgeQ4Config {
                heads: 3,
                context: 1,
            },
            AgeQ4Config {
                heads: 1,
                context: 0,
            },
            AgeQ4Config {
                heads: 1,
                context: 129,
            },
        ] {
            assert!(NativeAgeQ4::new(config, &[]).is_err());
        }
    }

    #[test]
    fn age_residual_q4_exact_prior_extrema_head_lag_and_single_head() -> TestResult {
        let config = AgeResidualQ4Config {
            heads: 2,
            context: 128,
        };
        let mut coefficients = vec![0i8; 256];
        coefficients[0] = 7;
        coefficients[127] = -7;
        coefficients[128] = -7;
        coefficients[255] = 7;
        let codec = NativeAgeResidualQ4::new(config, &pack_coefficients(&coefficients)?)?;
        assert_eq!(codec.prior_head_shifts()?, [4, 8]);
        for lag in 0..128 {
            assert_eq!(codec.prior_q24()[lag], -(lag as i64) << 20);
            assert_eq!(codec.prior_q24()[128 + lag], -(lag as i64) << 16);
        }
        assert_eq!(codec.age_q24()[0], 7 << 21); // A learned lag-zero residual survives.
        assert_eq!(codec.age_q24()[127], -(127i64 << 20) - (7 << 21));
        assert_eq!(codec.age_q24()[128], -(7 << 21));
        assert_eq!(codec.age_q24()[255], -(127i64 << 16) + (7 << 21));
        assert_eq!(codec.age_q24()[127], -147_849_216); // -8.8125 nats.
        let zeros = NativeAgeResidualQ4::new(config, &pack_coefficients(&[0; 256])?)?;
        assert_eq!(zeros.age_q24(), zeros.prior_q24());
        let single = NativeAgeResidualQ4::new(
            AgeResidualQ4Config {
                heads: 1,
                context: 128,
            },
            &pack_coefficients(&[0; 128])?,
        )?;
        assert_eq!(single.prior_head_shifts()?, [8]);
        assert_eq!(single.age_q24(), &zeros.age_q24()[128..]);
        Ok(())
    }

    #[test]
    fn age_residual_q4_admission_keeps_raw_quarter_policy_distinct() -> TestResult {
        let residual_config = AgeResidualQ4Config {
            heads: 2,
            context: 3,
        };
        let raw_config = AgeQ4Config {
            heads: 2,
            context: 3,
        };
        let packed = pack_coefficients(&[-7, 0, 7, 7, 0, -7])?;
        let residual = NativeAgeResidualQ4::new(residual_config, &packed)?;
        let raw = NativeAgeQ4::new(raw_config, &packed)?;
        assert_ne!(residual.age_q24(), raw.age_q24());
        assert_eq!(raw.age_q24()[0], -7 << 22);
        assert_eq!(residual.age_q24()[0], -7 << 21);
        let one = AgeResidualQ4Config {
            heads: 1,
            context: 1,
        };
        for bytes in [&[][..], &[8][..], &[0x10][..], &[0, 0][..]] {
            assert!(NativeAgeResidualQ4::new(one, bytes).is_err());
        }
        for config in [
            AgeResidualQ4Config {
                heads: 0,
                context: 1,
            },
            AgeResidualQ4Config {
                heads: 3,
                context: 1,
            },
            AgeResidualQ4Config {
                heads: 1,
                context: 0,
            },
            AgeResidualQ4Config {
                heads: 1,
                context: 129,
            },
        ] {
            assert!(NativeAgeResidualQ4::new(config, &[]).is_err());
            assert!(config.prior_head_shifts().is_err());
        }
        Ok(())
    }
}
