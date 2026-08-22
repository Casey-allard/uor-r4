//! Bounded semantic-planning primitives and frozen capacities (#843).
//!
//! Frozen contract: `docs/bounded_semantic_transitions_spec_843.md` §4. These
//! are the *deployed-form* shapes the offline compiler emits into and the
//! runtime reads out of, so they are `core`-only and allocation-free: fixed
//! arrays, no owned collections, no strings, no floating point. Every operation
//! defined here is P-4 legal — mask `AND`, integer comparison, and saturating
//! integer add or subtract. There is no multiply, no divide and no float
//! anywhere in this module.
//!
//! Capacities are maintainer sign-off values set at 4x headroom over the
//! measured fitting vocabulary (§4.1): 409–545 reachable states and 3–4
//! operators per task family at the maximum horizon. Exceeding any of them is a
//! deterministic `Decline(capacity)` at plan time, never a silent truncation.

/// Frozen maximum planning horizon (`H_max`, #844 §2.5).
pub const PLAN_HORIZON_MAX: usize = 16;
/// Frozen maximum frontier width (`W_max`, #844 §2.5). The measured maximum
/// breadth-first queue depth was exactly this, so the bound *binds*: which
/// candidates a bounded arm retains is decisive, not incidental.
pub const PLAN_FRONTIER_MAX: usize = 64;
/// Typed slots in one state valuation.
pub const PLAN_SLOTS_MAX: usize = 8;
/// Width of one bounded signed slot value.
pub const PLAN_SLOT_BITS: u32 = 16;
/// Distinct operators in the packed vocabulary.
pub const PLAN_ACTIONS_MAX: usize = 64;
/// Rules in the packed transition table.
pub const PLAN_RULES_MAX: usize = 256;
/// Forbidden-region predicates carried by one planning query.
pub const PLAN_CONSTRAINTS_MAX: usize = 64;
/// Goal predicates carried by one planning query.
pub const PLAN_GOALS_MAX: usize = 8;
/// Closed-set capacity — the real memory bound on a planning episode.
pub const PLAN_VISITED_MAX: usize = 2048;
/// Maximum encoded plan-witness size.
pub const PLAN_WITNESS_MAX_BYTES: usize = 4096;

/// A bounded typed slot valuation: up to [`PLAN_SLOTS_MAX`] signed
/// [`PLAN_SLOT_BITS`]-bit slots. `Copy`, so nothing here allocates.
///
/// The derived ordering is lexicographic over the slot array and then the
/// length, which is the canonical order the packed tables sort by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SlotVec {
    slots: [i16; PLAN_SLOTS_MAX],
    len: u8,
}

impl SlotVec {
    /// The empty valuation.
    pub const fn empty() -> Self {
        Self {
            slots: [0; PLAN_SLOTS_MAX],
            len: 0,
        }
    }

    /// A valuation over `values`. `None` when it exceeds [`PLAN_SLOTS_MAX`] —
    /// the capacity boundary is reported, never truncated to fit.
    pub fn from_slice(values: &[i16]) -> Option<Self> {
        if values.len() > PLAN_SLOTS_MAX {
            return None;
        }
        let mut slots = [0i16; PLAN_SLOTS_MAX];
        slots[..values.len()].copy_from_slice(values);
        Some(Self {
            slots,
            len: values.len() as u8,
        })
    }

    /// Number of slots in use.
    pub fn len(&self) -> usize {
        usize::from(self.len)
    }

    /// Whether no slot is in use.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The slots in use.
    pub fn as_slice(&self) -> &[i16] {
        &self.slots[..usize::from(self.len)]
    }

    /// The value of slot `index`, if it is in use.
    pub fn get(&self, index: usize) -> Option<i16> {
        self.as_slice().get(index).copied()
    }

    /// Apply a typed effect by saturating integer addition, slot by slot.
    /// `None` when the arities differ — a typed non-application, not a guess.
    /// Saturation is deliberate: a slot at its bound stays at its bound rather
    /// than wrapping into a different, valid-looking state.
    pub fn apply(&self, effect: &EffectDelta) -> Option<Self> {
        if effect.len() != self.len() {
            return None;
        }
        let mut out = *self;
        for i in 0..self.len() {
            out.slots[i] = self.slots[i].saturating_add(effect.delta[i]);
        }
        Some(out)
    }
}

/// A typed per-slot delta, applied by saturating integer addition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct EffectDelta {
    delta: [i16; PLAN_SLOTS_MAX],
    len: u8,
}

impl EffectDelta {
    /// The empty effect, covering no slot. Useful as a `const` array filler in
    /// caller-owned scratch, which must be constructible without allocating.
    pub const EMPTY: Self = Self {
        delta: [0; PLAN_SLOTS_MAX],
        len: 0,
    };

    /// The identity effect over `arity` slots.
    pub fn identity(arity: usize) -> Option<Self> {
        if arity > PLAN_SLOTS_MAX {
            return None;
        }
        Some(Self {
            delta: [0; PLAN_SLOTS_MAX],
            len: arity as u8,
        })
    }

    /// An effect over `values`. `None` when it exceeds [`PLAN_SLOTS_MAX`].
    pub fn from_slice(values: &[i16]) -> Option<Self> {
        if values.len() > PLAN_SLOTS_MAX {
            return None;
        }
        let mut delta = [0i16; PLAN_SLOTS_MAX];
        delta[..values.len()].copy_from_slice(values);
        Some(Self {
            delta,
            len: values.len() as u8,
        })
    }

    /// The effect observed between two valuations, by saturating subtraction.
    /// `None` when the arities differ.
    pub fn between(from: &SlotVec, to: &SlotVec) -> Option<Self> {
        if from.len() != to.len() {
            return None;
        }
        let mut delta = [0i16; PLAN_SLOTS_MAX];
        for (i, slot) in delta.iter_mut().enumerate().take(from.len()) {
            *slot = to.slots[i].saturating_sub(from.slots[i]);
        }
        Some(Self {
            delta,
            len: from.len,
        })
    }

    /// Number of slots this effect covers.
    pub fn len(&self) -> usize {
        usize::from(self.len)
    }

    /// Whether this effect covers no slot.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The per-slot deltas.
    pub fn as_slice(&self) -> &[i16] {
        &self.delta[..usize::from(self.len)]
    }
}

/// The comparison one slot of a precondition applies. `Any` reads nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(u8)]
pub enum CompareOp {
    /// The slot is not read.
    #[default]
    Any = 0,
    /// The slot equals the bound.
    Equal = 1,
    /// The slot differs from the bound.
    NotEqual = 2,
    /// The slot is at most the bound.
    AtMost = 3,
    /// The slot is at least the bound.
    AtLeast = 4,
}

impl CompareOp {
    /// The wire code for this comparison.
    pub fn code(self) -> u8 {
        self as u8
    }
}

/// A precondition over a bounded slot valuation: a read mask plus a per-slot
/// comparison against a bound. Evaluated with mask `AND` and integer compare
/// only, so it lowers directly to the deployed kernel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PreconditionMask {
    read_mask: u8,
    ops: [CompareOp; PLAN_SLOTS_MAX],
    bounds: [i16; PLAN_SLOTS_MAX],
}

impl PreconditionMask {
    /// The precondition that always holds and reads no slot.
    pub const fn unconditional() -> Self {
        Self {
            read_mask: 0,
            ops: [CompareOp::Any; PLAN_SLOTS_MAX],
            bounds: [0; PLAN_SLOTS_MAX],
        }
    }

    /// Add a comparison on `slot`. `None` when the slot is out of range.
    /// `CompareOp::Any` clears the slot's read bit rather than adding a
    /// vacuous test.
    pub fn reading(mut self, slot: usize, op: CompareOp, bound: i16) -> Option<Self> {
        if slot >= PLAN_SLOTS_MAX {
            return None;
        }
        self.ops[slot] = op;
        self.bounds[slot] = bound;
        let bit = 1u8 << slot;
        if op == CompareOp::Any {
            self.read_mask &= !bit;
        } else {
            self.read_mask |= bit;
        }
        Some(self)
    }

    /// Which slots this precondition reads — the basis for rule generalization
    /// and the field an observation records alongside its outcome.
    pub fn read_mask(&self) -> u8 {
        self.read_mask
    }

    /// The comparison applied to `slot`. `CompareOp::Any` beyond capacity.
    pub fn op(&self, slot: usize) -> CompareOp {
        self.ops.get(slot).copied().unwrap_or(CompareOp::Any)
    }

    /// The bound `slot` is compared against. Zero beyond capacity.
    pub fn bound(&self, slot: usize) -> i16 {
        self.bounds.get(slot).copied().unwrap_or(0)
    }

    /// Whether the precondition holds in `state`. Total: a slot the mask reads
    /// but the valuation does not carry fails closed.
    pub fn holds(&self, state: &SlotVec) -> bool {
        for slot in 0..PLAN_SLOTS_MAX {
            if self.read_mask & (1u8 << slot) == 0 {
                continue;
            }
            let Some(value) = state.get(slot) else {
                return false;
            };
            let bound = self.bounds[slot];
            let ok = match self.ops[slot] {
                CompareOp::Any => true,
                CompareOp::Equal => value == bound,
                CompareOp::NotEqual => value != bound,
                CompareOp::AtMost => value <= bound,
                CompareOp::AtLeast => value >= bound,
            };
            if !ok {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacities_carry_the_frozen_horizon_and_frontier() {
        assert_eq!(PLAN_HORIZON_MAX, 16);
        assert_eq!(PLAN_FRONTIER_MAX, 64);
    }

    #[test]
    fn slot_capacity_is_reported_not_truncated() {
        let too_many = [0i16; PLAN_SLOTS_MAX + 1];
        assert!(SlotVec::from_slice(&too_many).is_none());
        assert!(EffectDelta::from_slice(&too_many).is_none());
        assert!(SlotVec::from_slice(&[1, 2, 3]).is_some());
    }

    #[test]
    fn effects_apply_by_saturating_addition() {
        let state = SlotVec::from_slice(&[1, 2]).unwrap();
        let effect = EffectDelta::from_slice(&[3, -1]).unwrap();
        assert_eq!(state.apply(&effect).unwrap().as_slice(), &[4, 1]);
    }

    #[test]
    fn slot_arithmetic_saturates_rather_than_wrapping() {
        let state = SlotVec::from_slice(&[i16::MAX]).unwrap();
        let effect = EffectDelta::from_slice(&[1]).unwrap();
        assert_eq!(state.apply(&effect).unwrap().as_slice(), &[i16::MAX]);
        let low = SlotVec::from_slice(&[i16::MIN]).unwrap();
        let down = EffectDelta::from_slice(&[-1]).unwrap();
        assert_eq!(low.apply(&down).unwrap().as_slice(), &[i16::MIN]);
    }

    #[test]
    fn an_arity_mismatch_is_a_typed_non_application() {
        let state = SlotVec::from_slice(&[1, 2]).unwrap();
        let effect = EffectDelta::from_slice(&[1]).unwrap();
        assert!(state.apply(&effect).is_none());
    }

    #[test]
    fn an_effect_round_trips_between_two_valuations() {
        let from = SlotVec::from_slice(&[2, -3]).unwrap();
        let to = SlotVec::from_slice(&[5, -1]).unwrap();
        let effect = EffectDelta::between(&from, &to).unwrap();
        assert_eq!(effect.as_slice(), &[3, 2]);
        assert_eq!(from.apply(&effect).unwrap(), to);
    }

    #[test]
    fn an_unconditional_precondition_reads_nothing_and_always_holds() {
        let pre = PreconditionMask::unconditional();
        assert_eq!(pre.read_mask(), 0);
        assert!(pre.holds(&SlotVec::empty()));
        assert!(pre.holds(&SlotVec::from_slice(&[7, 9]).unwrap()));
    }

    #[test]
    fn a_precondition_records_the_slots_it_reads() {
        let pre = PreconditionMask::unconditional()
            .reading(0, CompareOp::AtLeast, 2)
            .unwrap()
            .reading(2, CompareOp::Equal, -1)
            .unwrap();
        assert_eq!(pre.read_mask(), 0b0000_0101);
        assert!(pre.holds(&SlotVec::from_slice(&[2, 0, -1]).unwrap()));
        assert!(!pre.holds(&SlotVec::from_slice(&[1, 0, -1]).unwrap()));
        assert!(!pre.holds(&SlotVec::from_slice(&[9, 0, 0]).unwrap()));
    }

    #[test]
    fn a_read_slot_the_valuation_does_not_carry_fails_closed() {
        let pre = PreconditionMask::unconditional()
            .reading(5, CompareOp::Equal, 0)
            .unwrap();
        assert!(!pre.holds(&SlotVec::from_slice(&[0, 0]).unwrap()));
    }

    #[test]
    fn clearing_a_slot_to_any_drops_its_read_bit() {
        let pre = PreconditionMask::unconditional()
            .reading(1, CompareOp::Equal, 4)
            .unwrap();
        assert_eq!(pre.read_mask(), 0b0000_0010);
        let cleared = pre.reading(1, CompareOp::Any, 0).unwrap();
        assert_eq!(cleared.read_mask(), 0);
        assert!(cleared.holds(&SlotVec::from_slice(&[0, 0]).unwrap()));
    }

    #[test]
    fn a_slot_beyond_capacity_is_refused() {
        assert!(PreconditionMask::unconditional()
            .reading(PLAN_SLOTS_MAX, CompareOp::Equal, 0)
            .is_none());
    }
}
