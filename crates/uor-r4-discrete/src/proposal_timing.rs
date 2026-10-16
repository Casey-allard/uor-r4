use crate::{Error, Result};

pub const HOST_SPANS: [&str; 9] = [
    "admission_epoch_read",
    "targets_spec_alloc_upload",
    "losses_status_alloc_zero",
    "counterfactual_submission",
    "status_read_check",
    "proposal_alloc",
    "chooser_submission",
    "final_sync",
    "cleanup",
];

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct HostTimings {
    pub spans_ns: [u128; 9],
    pub completed: [bool; 9],
    pub call_ns: u128,
}

impl HostTimings {
    pub fn validate_complete(&self) -> Result<()> {
        if !self.completed.iter().all(|completed| *completed)
            || self
                .spans_ns
                .iter()
                .try_fold(0u128, |total, span| total.checked_add(*span))
                != Some(self.call_ns)
        {
            return Err(Error::Invalid(
                "proposal host spans do not partition the completed call".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, serde::Serialize)]
pub struct EventDuration {
    pub milliseconds_ieee754_bits: u32,
    pub nanoseconds_floor: u128,
}

impl EventDuration {
    pub fn from_milliseconds_bits(bits: u32) -> Result<Self> {
        let exponent = (bits >> 23) & 255;
        let fraction = bits & 0x7fffff;
        if bits >> 31 != 0 || exponent == 255 {
            return Err(Error::Invalid(
                "CUDA clock duration is negative or non-finite".into(),
            ));
        }
        let significand = if exponent == 0 {
            fraction
        } else {
            fraction | 0x800000
        };
        let shift = if exponent == 0 {
            -149
        } else {
            exponent as i32 - 150
        };
        let scaled = u128::from(significand) * 1_000_000;
        let nanoseconds_floor = if shift >= 0 {
            let shift = shift as u32;
            if shift >= 128 || scaled > (u128::MAX >> shift) {
                return Err(Error::Invalid(
                    "CUDA clock duration exceeds integer nanoseconds".into(),
                ));
            }
            scaled << shift
        } else {
            scaled.checked_shr((-shift) as u32).unwrap_or(0)
        };
        Ok(Self {
            milliseconds_ieee754_bits: bits,
            nanoseconds_floor,
        })
    }
}

#[derive(Debug, serde::Serialize)]
pub struct ProposalTimings {
    pub schema: &'static str,
    pub host_span_names: [&'static str; 9],
    pub host: HostTimings,
    pub counterfactual: EventDuration,
    pub chooser: EventDuration,
    pub event_readback_ns: u128,
    pub accounting: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendor_clock_bits_decode_without_float_arithmetic() {
        for (bits, expected) in [
            (0, 0),
            (0x3f800000, 1_000_000),
            (0x3f000000, 500_000),
            (0x3fc00000, 1_500_000),
            (1, 0),
        ] {
            let duration = EventDuration::from_milliseconds_bits(bits).expect("finite clock");
            assert_eq!(duration.nanoseconds_floor, expected);
            assert_eq!(duration.milliseconds_ieee754_bits, bits);
        }
    }

    #[test]
    fn invalid_or_overflowing_clock_values_are_refused() {
        for bits in [0x80000000, 0xbf800000, 0x7f800000, 0x7fc00000, 0x7f7fffff] {
            assert!(EventDuration::from_milliseconds_bits(bits).is_err());
        }
    }

    #[test]
    fn host_spans_have_unique_names_and_start_incomplete() {
        for (index, name) in HOST_SPANS.iter().enumerate() {
            assert!(!HOST_SPANS[..index].contains(name));
        }
        let host = HostTimings::default();
        assert_eq!(host.spans_ns, [0; 9]);
        assert_eq!(host.completed, [false; 9]);
        assert!(host.validate_complete().is_err());
        let mut complete = HostTimings {
            spans_ns: [1; 9],
            completed: [true; 9],
            call_ns: 9,
        };
        assert!(complete.validate_complete().is_ok());
        complete.call_ns = 10;
        assert!(complete.validate_complete().is_err());
        complete.spans_ns = [u128::MAX; 9];
        assert!(complete.validate_complete().is_err());
    }
}
