//! Typed primitive feature packing. The full model must construct these values
//! from committed session state; this module cannot certify that future caller.
use super::circuit::{CircuitError, Result, INPUT_BITS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Phase {
    QueryA,
    ExtentA,
    QueryB,
    ExtentB,
    Control,
    Emit,
    Observe,
    Key,
}

#[derive(Clone, Debug, Default)]
pub struct Selected {
    /// The 120 signed hemisphere predicates, with high eight bits required zero.
    pub signature: [u64; 2],
    pub first_byte: u8,
    pub length: u8,
    pub age: u8,
    pub numeric_valid: bool,
}
#[derive(Clone, Debug, Default)]
pub struct Active {
    pub byte: u8,
    pub kind: u8,
    pub cursor: u8,
    pub length: u8,
    pub acknowledged: bool,
    pub provisional: bool,
}
#[derive(Clone, Debug, Default)]
pub struct CausalInputs {
    pub state_signatures: [[u64; 2]; 4],
    pub last_observed: Option<u8>,
    pub selected_a: Option<Selected>,
    pub selected_b: Option<Selected>,
    pub active: Option<Active>,
    pub turn_position: u8,
    pub zeta_bins: [u8; 4],
    pub last_control: u8,
    pub publications: u8,
    pub pending_offer: bool,
}
fn put(out: &mut [bool; INPUT_BITS], offset: usize, bits: usize, value: u64) {
    for i in 0..bits {
        out[offset + i] = (value >> i) & 1 != 0;
    }
}
fn signature(out: &mut [bool; INPUT_BITS], offset: usize, value: [u64; 2]) -> Result<()> {
    if value[1] >> 56 != 0 {
        return Err(CircuitError::Domain);
    }
    put(out, offset, 64, value[0]);
    put(out, offset + 64, 56, value[1]);
    Ok(())
}
impl CausalInputs {
    /// No current target, unobserved byte, digest or unselected record input exists.
    /// Stage masks suppress selections that have not occurred in this tick.
    pub fn pack(&self, phase: Phase) -> Result<[bool; INPUT_BITS]> {
        if self.zeta_bins.iter().any(|&v| v > 15) || self.last_control > 6 || self.publications > 2
        {
            return Err(CircuitError::Domain);
        }
        let mut out = [false; INPUT_BITS];
        for (lane, &s) in self.state_signatures.iter().enumerate() {
            signature(&mut out, lane * 128, s)?;
        }
        let mut flags = 0;
        if let Some(b) = self.last_observed {
            flags |= 1;
            put(&mut out, 768, 8, u64::from(b));
        }
        let a = if matches!(phase, Phase::QueryA | Phase::Key) {
            None
        } else {
            self.selected_a.as_ref()
        };
        let b = if (phase as u8) < Phase::ExtentB as u8 || phase == Phase::Key {
            None
        } else {
            self.selected_b.as_ref()
        };
        for (i, selected) in [a, b].into_iter().enumerate() {
            if let Some(s) = selected {
                let pre_extent =
                    (i == 0 && phase == Phase::ExtentA) || (i == 1 && phase == Phase::ExtentB);
                if !pre_extent && (s.length == 0 || s.length > 64) {
                    return Err(CircuitError::Domain);
                }
                flags |= 2 << i;
                signature(&mut out, 512 + i * 128, s.signature)?;
                put(&mut out, 776 + i * 8, 8, u64::from(s.first_byte));
                if !pre_extent {
                    put(&mut out, 829 + i * 7, 7, u64::from(s.length));
                }
                if (phase as u8) >= Phase::Control as u8 && s.numeric_valid {
                    flags |= 16 << i;
                }
                put(&mut out, 872 + i * 8, 8, u64::from(s.age));
            }
        }
        if let Some(a) = self
            .active
            .as_ref()
            .filter(|a| phase != Phase::QueryA || a.acknowledged)
        {
            if a.length == 0 || a.length > 64 || a.cursor >= a.length || a.kind > 2 {
                return Err(CircuitError::Domain);
            }
            flags |= 8;
            if a.kind == 2 {
                flags |= 64;
            }
            put(&mut out, 792, 8, u64::from(a.byte));
            put(&mut out, 812, 2, u64::from(a.kind));
            put(&mut out, 816, 6, u64::from(a.cursor));
            put(&mut out, 822, 7, u64::from(a.length));
            out[843] = a.acknowledged;
            out[844] = a.provisional;
        }
        if self.pending_offer && matches!(phase, Phase::Emit | Phase::Observe) {
            flags |= 128;
        }
        put(&mut out, 800, 8, flags);
        put(&mut out, 808, 4, phase as u64);
        put(&mut out, 848, 8, u64::from(self.turn_position));
        for (i, &z) in self.zeta_bins.iter().enumerate() {
            put(&mut out, 856 + i * 4, 4, u64::from(z));
        }
        put(&mut out, 888, 3, u64::from(self.last_control));
        put(&mut out, 891, 2, u64::from(self.publications));
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn addressed_inputs_stage_masks_and_reserved_bits() {
        let base = CausalInputs::default();
        let mut selected = base.clone();
        selected.selected_a = Some(Selected {
            signature: [1, 1],
            first_byte: 42,
            length: 64,
            age: 255,
            numeric_valid: true,
        });
        selected.selected_b = Some(Selected {
            signature: [2, 3],
            first_byte: 99,
            length: 1,
            age: 10,
            numeric_valid: false,
        });
        assert_eq!(
            base.pack(Phase::QueryA).unwrap(),
            selected.pack(Phase::QueryA).unwrap()
        );
        let mut a_only = selected.clone();
        a_only.selected_b = None;
        assert_eq!(
            a_only.pack(Phase::QueryB).unwrap(),
            selected.pack(Phase::QueryB).unwrap()
        );
        assert_ne!(
            a_only.pack(Phase::ExtentB).unwrap(),
            selected.pack(Phase::ExtentB).unwrap()
        );
        assert_eq!(
            base.pack(Phase::Key).unwrap(),
            selected.pack(Phase::Key).unwrap()
        );
        let extent_a = selected.pack(Phase::ExtentA).unwrap();
        assert!(extent_a[829..836].iter().all(|&b| !b));
        assert!(!extent_a[804]);
        assert!(selected.pack(Phase::Emit).unwrap()[804]);
        let extent_b = selected.pack(Phase::ExtentB).unwrap();
        assert!(extent_b[836..843].iter().all(|&b| !b));
        let mut old = base.clone();
        old.active = Some(Active {
            byte: 99,
            kind: 0,
            cursor: 0,
            length: 1,
            acknowledged: false,
            provisional: false,
        });
        assert_eq!(
            old.pack(Phase::QueryA).unwrap(),
            base.pack(Phase::QueryA).unwrap()
        );
        old.active.as_mut().unwrap().acknowledged = true;
        assert_ne!(
            old.pack(Phase::QueryA).unwrap(),
            base.pack(Phase::QueryA).unwrap()
        );
        let frame = selected.pack(Phase::Emit).unwrap();
        assert!(frame[896..].iter().all(|&b| !b));
        for lane in 0..6 {
            assert!(frame[lane * 128 + 120..lane * 128 + 128]
                .iter()
                .all(|&b| !b));
        }
        selected.state_signatures[0][1] = 1 << 63;
        assert_eq!(selected.pack(Phase::QueryA), Err(CircuitError::Domain));
    }
}
