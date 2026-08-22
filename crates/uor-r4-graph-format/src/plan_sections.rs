//! Borrowed packed bounded-planning sections — the optional `PSCH`, `PTRN`,
//! `PGOL` and `PWIT` sections (#843, lowering the design contract in
//! `docs/bounded_semantic_transitions_spec_843.md` §5 and §7).
//!
//! Four sections, read with P-4-legal operations only (mask `AND`, integer
//! comparison, saturating integer add and subtract, and fixed-offset table
//! reads — no multiply, no divide, no float):
//!
//! - [`PlanSchema`] (`PSCH`) — the slot shape, the operator **effect**
//!   vocabulary, the frozen capacities as recorded values, and the ordinal band
//!   thresholds. Small and fixed, so it is read by direct offset.
//! - [`RuleTable`] (`PTRN`) — the induced rule table, rows canonically ordered
//!   by `(operator, precondition, effect)`, plus a fixed-width **operator
//!   index** so `operator -> its rule slice` is a fixed-offset table read.
//!   Lookup cost is never a function of how many rules the table holds.
//! - [`PredicateSet`] (`PGOL`) — the goal and forbidden-region predicates one
//!   planning query carries, canonically ordered.
//! - [`PlanWitnessBytes`] (`PWIT`) — a versioned, self-contained plan witness,
//!   replayable from its own bytes with no model output and no other section.
//!
//! All four are **optional** ([`crate::types::SectionId::OPTIONAL_BIT`]): an
//! artifact without them, or a reader that does not consume them, behaves
//! exactly as before (absent-section identity), and every artifact produced
//! before they existed stays valid.
//!
//! Parsing is two-stage and never allocates: a header check (magic, version,
//! reserved-zero, slot shape, recorded capacities against the frozen ones),
//! then a structural check (exact byte coverage, bounded counts, canonical
//! ordering, in-range references). Every rejection is a typed
//! [`crate::NotAProduct`]; nothing is best-effort.

use crate::error::FormatError;
use crate::plan::{
    CompareOp, EffectDelta, PreconditionMask, SlotVec, PLAN_ACTIONS_MAX, PLAN_CONSTRAINTS_MAX,
    PLAN_FRONTIER_MAX, PLAN_GOALS_MAX, PLAN_HORIZON_MAX, PLAN_RULES_MAX, PLAN_SLOTS_MAX,
    PLAN_SLOT_BITS, PLAN_VISITED_MAX, PLAN_WITNESS_MAX_BYTES,
};
use crate::sanctioned::{NotAProduct, ObjectKind};

/// `PSCH` magic.
pub const PSCH_MAGIC: [u8; 4] = *b"PSC1";
/// `PTRN` magic.
pub const PTRN_MAGIC: [u8; 4] = *b"PTR1";
/// `PGOL` magic.
pub const PGOL_MAGIC: [u8; 4] = *b"PGL1";
/// `PWIT` magic.
pub const PWIT_MAGIC: [u8; 4] = *b"PWT1";

/// Schema version shared by all four sections. A reader that does not know a
/// version refuses the section rather than reading it best-effort.
pub const PLAN_SECTION_VERSION: u16 = 1;

/// `PSCH` fixed header length.
pub const PSCH_HEADER_LEN: usize = 48;
/// One operator-vocabulary entry: [`PLAN_SLOTS_MAX`] little-endian `i16`.
pub const PSCH_OPERATOR_LEN: usize = PLAN_SLOTS_MAX * 2;
/// `PTRN` fixed header length.
pub const PTRN_HEADER_LEN: usize = 16;
/// One `PTRN` rule row.
pub const PTRN_ROW_LEN: usize = 52;
/// One `PTRN` operator-index entry: first row and row count.
pub const PTRN_INDEX_LEN: usize = 4;
/// `PGOL` fixed header length.
pub const PGOL_HEADER_LEN: usize = 12;
/// One packed predicate row.
pub const PREDICATE_LEN: usize = 28;
/// `PWIT` fixed header length.
pub const PWIT_HEADER_LEN: usize = 16;
/// One `PWIT` step row: applied effect, resulting slots, chosen index, rule row.
pub const PWIT_STEP_LEN: usize = 36;
/// One `PWIT` considered-candidate row (informational).
pub const PWIT_CONSIDERED_LEN: usize = 16;

fn bad(reason: FormatError) -> NotAProduct {
    NotAProduct::new(ObjectKind::PlanSchema, reason)
}

fn bad_rules(reason: FormatError) -> NotAProduct {
    NotAProduct::new(ObjectKind::PlanRuleTable, reason)
}

fn bad_predicates(reason: FormatError) -> NotAProduct {
    NotAProduct::new(ObjectKind::PlanPredicateSet, reason)
}

fn bad_witness(reason: FormatError) -> NotAProduct {
    NotAProduct::new(ObjectKind::PlanWitness, reason)
}

fn read_u16(bytes: &[u8]) -> u16 {
    u16::from(bytes[0]) | (u16::from(bytes[1]) << 8)
}

fn read_u32(bytes: &[u8]) -> u32 {
    u32::from(bytes[0])
        | (u32::from(bytes[1]) << 8)
        | (u32::from(bytes[2]) << 16)
        | (u32::from(bytes[3]) << 24)
}

fn read_i16(bytes: &[u8]) -> i16 {
    read_u16(bytes) as i16
}

fn read_i32(bytes: &[u8]) -> i32 {
    read_u32(bytes) as i32
}

fn compare_op(code: u8) -> Option<CompareOp> {
    match code {
        0 => Some(CompareOp::Any),
        1 => Some(CompareOp::Equal),
        2 => Some(CompareOp::NotEqual),
        3 => Some(CompareOp::AtMost),
        4 => Some(CompareOp::AtLeast),
        _ => None,
    }
}

/// Read one packed [`PreconditionMask`] from a [`PREDICATE_LEN`] row:
/// `read_mask u8`, `reserved u8`, `ops[8] u8`, `bounds[8] i16`.
fn read_predicate(row: &[u8]) -> Result<PreconditionMask, NotAProduct> {
    if row.len() < PREDICATE_LEN {
        return Err(bad(FormatError::PlanBounds));
    }
    if row[1] != 0 {
        return Err(bad(FormatError::PlanNonZeroReserved));
    }
    let declared_mask = row[0];
    let mut predicate = PreconditionMask::unconditional();
    let mut rebuilt_mask = 0u8;
    for slot in 0..PLAN_SLOTS_MAX {
        let op = compare_op(row[2 + slot]).ok_or_else(|| bad(FormatError::PlanInvalidRow))?;
        let bound = read_i16(&row[10 + slot * 2..12 + slot * 2]);
        if op != CompareOp::Any {
            rebuilt_mask |= 1u8 << slot;
        }
        predicate = predicate
            .reading(slot, op, bound)
            .ok_or_else(|| bad(FormatError::PlanBounds))?;
    }
    // The declared mask must agree with the per-slot operations: a header that
    // claims to read a slot the row does not test is a malformed predicate, not
    // a predicate to be read charitably.
    if declared_mask != rebuilt_mask {
        return Err(bad(FormatError::PlanInvalidRow));
    }
    Ok(predicate)
}

/// Read [`PLAN_SLOTS_MAX`] little-endian `i16` as a [`SlotVec`] of `arity`.
fn read_slots(bytes: &[u8], arity: usize) -> Result<SlotVec, NotAProduct> {
    if bytes.len() < PLAN_SLOTS_MAX * 2 || arity > PLAN_SLOTS_MAX {
        return Err(bad(FormatError::PlanBounds));
    }
    let mut values = [0i16; PLAN_SLOTS_MAX];
    for (slot, value) in values.iter_mut().enumerate() {
        *value = read_i16(&bytes[slot * 2..slot * 2 + 2]);
    }
    // Slots beyond the declared arity must be zero: a non-zero tail is unread
    // state that two otherwise-identical artifacts could disagree on.
    if values[arity..].iter().any(|v| *v != 0) {
        return Err(bad(FormatError::PlanInvalidRow));
    }
    SlotVec::from_slice(&values[..arity]).ok_or_else(|| bad(FormatError::PlanBounds))
}

fn read_effect(bytes: &[u8], arity: usize) -> Result<EffectDelta, NotAProduct> {
    let slots = read_slots(bytes, arity)?;
    EffectDelta::from_slice(slots.as_slice()).ok_or_else(|| bad(FormatError::PlanBounds))
}

// ---------------------------------------------------------------------------
// PSCH — the planning schema
// ---------------------------------------------------------------------------

/// A borrowed, validated `PSCH` planning schema.
///
/// Header layout, little-endian, in fixed field order: magic, version, slot
/// count, slot bits, operator count, reserved; then the frozen capacities as
/// *recorded* values (horizon, frontier, actions, rules, constraints, goals,
/// visited, witness bytes); then the three ordinal band thresholds; then
/// reserved. The operator effect vocabulary follows, canonically ascending.
///
/// The recorded capacities are checked against the frozen ones at parse time.
/// An artifact that claims a larger capacity than this build supports is
/// refused, not honoured: a capacity header is a promise about bounded work,
/// and reading one larger than the scratch that will be provided is exactly the
/// silent overflow the bound exists to prevent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanSchema<'a> {
    operators: &'a [u8],
    slot_count: u8,
    operator_count: u16,
    band_low: u32,
    band_medium: u32,
    band_high: u32,
}

impl<'a> PlanSchema<'a> {
    /// Two-stage validation of a `PSCH` section.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, NotAProduct> {
        if bytes.len() < PSCH_HEADER_LEN {
            return Err(bad(FormatError::PlanTooShort));
        }
        if bytes[0..4] != PSCH_MAGIC {
            return Err(bad(FormatError::PlanBadMagic));
        }
        if read_u16(&bytes[4..6]) != PLAN_SECTION_VERSION {
            return Err(bad(FormatError::PlanUnsupportedVersion));
        }
        let slot_count = bytes[6];
        let slot_bits = bytes[7];
        if usize::from(slot_count) > PLAN_SLOTS_MAX || slot_count == 0 {
            return Err(bad(FormatError::PlanBounds));
        }
        if u32::from(slot_bits) != PLAN_SLOT_BITS {
            return Err(bad(FormatError::PlanUnsupportedVersion));
        }
        let operator_count = read_u16(&bytes[8..10]);
        if read_u16(&bytes[10..12]) != 0 || read_u32(&bytes[44..48]) != 0 {
            return Err(bad(FormatError::PlanNonZeroReserved));
        }
        if usize::from(operator_count) > PLAN_ACTIONS_MAX {
            return Err(bad(FormatError::PlanBounds));
        }
        // Recorded capacities must equal the frozen ones this build enforces.
        let recorded: [(u32, u32); 8] = [
            (u32::from(read_u16(&bytes[12..14])), PLAN_HORIZON_MAX as u32),
            (
                u32::from(read_u16(&bytes[14..16])),
                PLAN_FRONTIER_MAX as u32,
            ),
            (u32::from(read_u16(&bytes[16..18])), PLAN_ACTIONS_MAX as u32),
            (u32::from(read_u16(&bytes[18..20])), PLAN_RULES_MAX as u32),
            (
                u32::from(read_u16(&bytes[20..22])),
                PLAN_CONSTRAINTS_MAX as u32,
            ),
            (u32::from(read_u16(&bytes[22..24])), PLAN_GOALS_MAX as u32),
            (read_u32(&bytes[24..28]), PLAN_VISITED_MAX as u32),
            (read_u32(&bytes[28..32]), PLAN_WITNESS_MAX_BYTES as u32),
        ];
        if recorded.iter().any(|(got, frozen)| got != frozen) {
            return Err(bad(FormatError::PlanCapacityMismatch));
        }
        let band_low = read_u32(&bytes[32..36]);
        let band_medium = read_u32(&bytes[36..40]);
        let band_high = read_u32(&bytes[40..44]);
        if !(band_low < band_medium && band_medium < band_high) {
            return Err(bad(FormatError::PlanNotCanonical));
        }

        let vocabulary_len = usize::from(operator_count) * PSCH_OPERATOR_LEN;
        let operators = bytes
            .get(PSCH_HEADER_LEN..PSCH_HEADER_LEN + vocabulary_len)
            .ok_or_else(|| bad(FormatError::PlanBounds))?;
        if bytes.len() != PSCH_HEADER_LEN + vocabulary_len {
            return Err(bad(FormatError::PlanTrailingBytes));
        }
        // Canonical, strictly ascending, so the vocabulary has one
        // representation and no duplicate operator.
        let arity = usize::from(slot_count);
        let mut previous: Option<EffectDelta> = None;
        for index in 0..usize::from(operator_count) {
            let effect = read_effect(&operators[index * PSCH_OPERATOR_LEN..], arity)?;
            if previous.is_some_and(|p| p >= effect) {
                return Err(bad(FormatError::PlanNotCanonical));
            }
            previous = Some(effect);
        }
        Ok(Self {
            operators,
            slot_count,
            operator_count,
            band_low,
            band_medium,
            band_high,
        })
    }

    /// Typed slots per state valuation.
    pub fn slot_count(&self) -> usize {
        usize::from(self.slot_count)
    }

    /// Operators in the packed vocabulary.
    pub fn operator_count(&self) -> usize {
        usize::from(self.operator_count)
    }

    /// The effect of operator `index`, by fixed-offset table read.
    pub fn operator(&self, index: usize) -> Option<EffectDelta> {
        if index >= self.operator_count() {
            return None;
        }
        read_effect(
            &self.operators[index * PSCH_OPERATOR_LEN..],
            self.slot_count(),
        )
        .ok()
    }

    /// The three ordinal band thresholds, ascending.
    pub fn band_thresholds(&self) -> (u32, u32, u32) {
        (self.band_low, self.band_medium, self.band_high)
    }
}

// ---------------------------------------------------------------------------
// PTRN — the transition rule table
// ---------------------------------------------------------------------------

/// One packed transition rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackedRule {
    /// Index into the `PSCH` operator vocabulary.
    pub operator: u16,
    /// When the rule applies.
    pub precondition: PreconditionMask,
    /// What it does, by saturating integer addition.
    pub effect: EffectDelta,
    /// Distinct observations behind it.
    pub support: u32,
    /// Ordinal band, `0..=3` — never a calibrated probability.
    pub band: u8,
}

/// A borrowed, validated `PTRN` rule table with its operator index.
///
/// Rows are canonically ordered by `(operator, precondition, effect)` and the
/// index gives each operator's contiguous row slice, so resolving
/// `operator -> rules` is a fixed-offset table read whose cost does not depend
/// on how many rules the table holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleTable<'a> {
    rows: &'a [u8],
    index: &'a [u8],
    rule_count: u16,
    operator_count: u16,
    slot_count: u8,
}

impl<'a> RuleTable<'a> {
    /// Two-stage validation of a `PTRN` section against its schema.
    pub fn parse(bytes: &'a [u8], schema: &PlanSchema<'_>) -> Result<Self, NotAProduct> {
        if bytes.len() < PTRN_HEADER_LEN {
            return Err(bad_rules(FormatError::PlanTooShort));
        }
        if bytes[0..4] != PTRN_MAGIC {
            return Err(bad_rules(FormatError::PlanBadMagic));
        }
        if read_u16(&bytes[4..6]) != PLAN_SECTION_VERSION {
            return Err(bad_rules(FormatError::PlanUnsupportedVersion));
        }
        let rule_count = read_u16(&bytes[6..8]);
        let operator_count = read_u16(&bytes[8..10]);
        let slot_count = bytes[10];
        if bytes[11] != 0 || read_u32(&bytes[12..16]) != 0 {
            return Err(bad_rules(FormatError::PlanNonZeroReserved));
        }
        if usize::from(rule_count) > PLAN_RULES_MAX {
            return Err(bad_rules(FormatError::PlanBounds));
        }
        if usize::from(operator_count) != schema.operator_count()
            || usize::from(slot_count) != schema.slot_count()
        {
            return Err(bad_rules(FormatError::PlanUnsupportedVersion));
        }

        let rows_len = usize::from(rule_count) * PTRN_ROW_LEN;
        let index_len = usize::from(operator_count) * PTRN_INDEX_LEN;
        if bytes.len() != PTRN_HEADER_LEN + rows_len + index_len {
            return Err(bad_rules(FormatError::PlanTrailingBytes));
        }
        let rows = &bytes[PTRN_HEADER_LEN..PTRN_HEADER_LEN + rows_len];
        let index = &bytes[PTRN_HEADER_LEN + rows_len..];

        let table = Self {
            rows,
            index,
            rule_count,
            operator_count,
            slot_count,
        };

        // Structural stage: every row decodes, references a real operator, and
        // the rows are strictly ascending in the canonical key — so the table
        // has one representation and carries no duplicate rule.
        let mut previous: Option<(u16, &[u8])> = None;
        for row in 0..usize::from(rule_count) {
            let rule = table
                .rule(row)
                .ok_or_else(|| bad_rules(FormatError::PlanBounds))?;
            if usize::from(rule.operator) >= schema.operator_count() {
                return Err(bad_rules(FormatError::PlanBounds));
            }
            if rule.band > 3 {
                return Err(bad_rules(FormatError::PlanInvalidRow));
            }
            let raw = &rows[row * PTRN_ROW_LEN..row * PTRN_ROW_LEN + PTRN_ROW_LEN];
            if read_u32(&raw[48..52]) != 0 {
                return Err(bad_rules(FormatError::PlanNonZeroReserved));
            }
            let key = (rule.operator, &raw[2..48]);
            if previous.is_some_and(|p| p >= key) {
                return Err(bad_rules(FormatError::PlanNotCanonical));
            }
            previous = Some(key);
        }

        // The operator index must tile the row array in order, so every rule is
        // reachable through exactly one operator slice.
        let mut expected_first = 0usize;
        for operator in 0..usize::from(operator_count) {
            let (first, count) = table
                .index_entry(operator)
                .ok_or_else(|| bad_rules(FormatError::PlanBounds))?;
            if first != expected_first {
                return Err(bad_rules(FormatError::PlanNotCanonical));
            }
            let end = first
                .checked_add(count)
                .ok_or_else(|| bad_rules(FormatError::PlanBounds))?;
            if end > usize::from(rule_count) {
                return Err(bad_rules(FormatError::PlanBounds));
            }
            for row in first..end {
                let rule = table
                    .rule(row)
                    .ok_or_else(|| bad_rules(FormatError::PlanBounds))?;
                if usize::from(rule.operator) != operator {
                    return Err(bad_rules(FormatError::PlanNotCanonical));
                }
            }
            expected_first = end;
        }
        if expected_first != usize::from(rule_count) {
            return Err(bad_rules(FormatError::PlanNotCanonical));
        }
        Ok(table)
    }

    /// Rules in the table.
    pub fn rule_count(&self) -> usize {
        usize::from(self.rule_count)
    }

    /// Operators the index covers.
    pub fn operator_count(&self) -> usize {
        usize::from(self.operator_count)
    }

    fn index_entry(&self, operator: usize) -> Option<(usize, usize)> {
        let at = operator.checked_mul(PTRN_INDEX_LEN)?;
        let entry = self.index.get(at..at + PTRN_INDEX_LEN)?;
        Some((
            usize::from(read_u16(&entry[0..2])),
            usize::from(read_u16(&entry[2..4])),
        ))
    }

    /// The rule at `row`, by fixed-offset table read.
    pub fn rule(&self, row: usize) -> Option<PackedRule> {
        let at = row.checked_mul(PTRN_ROW_LEN)?;
        let raw = self.rows.get(at..at + PTRN_ROW_LEN)?;
        let arity = usize::from(self.slot_count);
        let mut precondition = PreconditionMask::unconditional();
        let mut rebuilt_mask = 0u8;
        for slot in 0..PLAN_SLOTS_MAX {
            let op = compare_op(raw[8 + slot])?;
            let bound = read_i16(&raw[16 + slot * 2..18 + slot * 2]);
            if op != CompareOp::Any {
                rebuilt_mask |= 1u8 << slot;
            }
            precondition = precondition.reading(slot, op, bound)?;
        }
        if raw[2] != rebuilt_mask {
            return None;
        }
        let effect = read_effect(&raw[32..48], arity).ok()?;
        Some(PackedRule {
            operator: read_u16(&raw[0..2]),
            precondition,
            effect,
            support: read_u32(&raw[4..8]),
            band: raw[3],
        })
    }

    /// The half-open row range holding `operator`'s rules — one fixed-offset
    /// read, independent of the table's size.
    pub fn rules_for(&self, operator: usize) -> Option<(usize, usize)> {
        let (first, count) = self.index_entry(operator)?;
        Some((first, first + count))
    }
}

// ---------------------------------------------------------------------------
// PGOL — goal and forbidden-region predicates
// ---------------------------------------------------------------------------

/// A borrowed, validated `PGOL` predicate set: the goals a planning query must
/// reach and the forbidden regions it must never enter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PredicateSet<'a> {
    rows: &'a [u8],
    goal_count: u8,
    constraint_count: u8,
}

impl<'a> PredicateSet<'a> {
    /// Two-stage validation of a `PGOL` section.
    pub fn parse(bytes: &'a [u8], schema: &PlanSchema<'_>) -> Result<Self, NotAProduct> {
        if bytes.len() < PGOL_HEADER_LEN {
            return Err(bad_predicates(FormatError::PlanTooShort));
        }
        if bytes[0..4] != PGOL_MAGIC {
            return Err(bad_predicates(FormatError::PlanBadMagic));
        }
        if read_u16(&bytes[4..6]) != PLAN_SECTION_VERSION {
            return Err(bad_predicates(FormatError::PlanUnsupportedVersion));
        }
        let goal_count = bytes[6];
        let constraint_count = bytes[7];
        if usize::from(bytes[8]) != schema.slot_count() {
            return Err(bad_predicates(FormatError::PlanUnsupportedVersion));
        }
        if bytes[9] != 0 || bytes[10] != 0 || bytes[11] != 0 {
            return Err(bad_predicates(FormatError::PlanNonZeroReserved));
        }
        if usize::from(goal_count) > PLAN_GOALS_MAX
            || usize::from(constraint_count) > PLAN_CONSTRAINTS_MAX
        {
            return Err(bad_predicates(FormatError::PlanBounds));
        }
        let total = usize::from(goal_count) + usize::from(constraint_count);
        if bytes.len() != PGOL_HEADER_LEN + total * PREDICATE_LEN {
            return Err(bad_predicates(FormatError::PlanTrailingBytes));
        }
        let rows = &bytes[PGOL_HEADER_LEN..];
        // Each group is canonical and strictly ascending in its raw bytes.
        for group in [
            (0usize, usize::from(goal_count)),
            (usize::from(goal_count), total),
        ] {
            let mut previous: Option<&[u8]> = None;
            for row in group.0..group.1 {
                let raw = &rows[row * PREDICATE_LEN..row * PREDICATE_LEN + PREDICATE_LEN];
                read_predicate(raw)?;
                if previous.is_some_and(|p| p >= raw) {
                    return Err(bad_predicates(FormatError::PlanNotCanonical));
                }
                previous = Some(raw);
            }
        }
        Ok(Self {
            rows,
            goal_count,
            constraint_count,
        })
    }

    /// Goal predicates in the set.
    pub fn goal_count(&self) -> usize {
        usize::from(self.goal_count)
    }

    /// Forbidden-region predicates in the set.
    pub fn constraint_count(&self) -> usize {
        usize::from(self.constraint_count)
    }

    /// Goal predicate `index`.
    pub fn goal(&self, index: usize) -> Option<PreconditionMask> {
        if index >= self.goal_count() {
            return None;
        }
        read_predicate(&self.rows[index * PREDICATE_LEN..]).ok()
    }

    /// Forbidden-region predicate `index`.
    pub fn constraint(&self, index: usize) -> Option<PreconditionMask> {
        if index >= self.constraint_count() {
            return None;
        }
        let row = self.goal_count() + index;
        read_predicate(&self.rows[row * PREDICATE_LEN..]).ok()
    }

    /// Whether `state` satisfies every goal predicate.
    pub fn satisfies_goal(&self, state: &SlotVec) -> bool {
        (0..self.goal_count()).all(|i| self.goal(i).is_some_and(|g| g.holds(state)))
    }

    /// Whether `state` enters any forbidden region.
    pub fn is_forbidden(&self, state: &SlotVec) -> bool {
        (0..self.constraint_count()).any(|i| self.constraint(i).is_some_and(|c| c.holds(state)))
    }
}

// ---------------------------------------------------------------------------
// PWIT — the plan witness
// ---------------------------------------------------------------------------

/// Why a planning episode declined, as encoded in a witness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackedDecline {
    /// No valid plan exists within the representational ceiling.
    NoPlan,
    /// A fixed capacity was exceeded.
    Capacity,
    /// A conflicted rule or an unknown slot was reached.
    Unknown,
    /// The ordinal band fell below the decline threshold.
    LowConfidence,
}

impl PackedDecline {
    fn from_code(code: u8) -> Option<Option<Self>> {
        match code {
            0 => Some(None),
            1 => Some(Some(PackedDecline::NoPlan)),
            2 => Some(Some(PackedDecline::Capacity)),
            3 => Some(Some(PackedDecline::Unknown)),
            4 => Some(Some(PackedDecline::LowConfidence)),
            _ => None,
        }
    }

    /// The wire code for this decline.
    pub fn code(self) -> u8 {
        match self {
            PackedDecline::NoPlan => 1,
            PackedDecline::Capacity => 2,
            PackedDecline::Unknown => 3,
            PackedDecline::LowConfidence => 4,
        }
    }
}

/// The verdict of independently replaying a witness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayVerdict {
    /// Terminal goal and every intermediate transition check out.
    Valid,
    /// A step is invalid — at `step == step_count` the terminal goal failed.
    Invalid {
        /// The offending step.
        step: usize,
        /// Why.
        reason: ReplayFault,
    },
    /// The witness is an honest decline.
    Declined(PackedDecline),
}

/// What was wrong with a step during replay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayFault {
    /// The recorded effect does not carry the recorded predecessor to the
    /// recorded successor.
    EffectDoesNotProduceState,
    /// An intermediate state enters a forbidden region.
    EntersForbiddenRegion,
    /// The terminal state does not satisfy the goal.
    GoalNotSatisfied,
}

/// One recorded step of a witness: the applied effect, the resulting state, the
/// chosen candidate slot, and the `PTRN` row the rule came from.
pub type WitnessStep = (EffectDelta, SlotVec, u16, u16);

/// One considered candidate at a planning step. Informational: replay does not
/// depend on it, exactly as the reference model's witness does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsideredCandidate {
    /// Operator index in the `PSCH` vocabulary.
    pub operator: u16,
    /// Row in the `PTRN` table.
    pub rule_row: u16,
    /// Deterministic integer score — no float in the witness ordering.
    pub score: i32,
    /// Canonical deterministic tie-break rank; lower wins.
    pub tie_rank: u16,
    /// Distinct observations behind the rule.
    pub support: u32,
    /// Ordinal band, `0..=3`.
    pub band: u8,
    /// Bit 0 set when this candidate was the chosen one.
    pub flags: u8,
}

/// A borrowed, validated `PWIT` plan witness.
///
/// **Self-contained by construction.** The witness carries its own initial
/// state, its goal and forbidden predicates *inline*, and per step both the
/// applied effect and the resulting state. [`PlanWitnessBytes::replay`]
/// therefore re-verifies it from these bytes alone: no model output, no
/// planner, and no other section. Replaying a right answer reached through an
/// invalid intermediate step still returns `Invalid`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanWitnessBytes<'a> {
    bytes: &'a [u8],
    slot_count: u8,
    step_count: u8,
    considered_per_step: u8,
    constraint_count: u8,
    decline: Option<PackedDecline>,
    recorded_verdict: u8,
    recorded_step: u16,
}

impl<'a> PlanWitnessBytes<'a> {
    fn steps_at(&self) -> usize {
        PWIT_HEADER_LEN
            + PLAN_SLOTS_MAX * 2
            + PREDICATE_LEN
            + usize::from(self.constraint_count) * PREDICATE_LEN
    }

    fn considered_at(&self) -> usize {
        self.steps_at() + usize::from(self.step_count) * PWIT_STEP_LEN
    }

    /// Two-stage validation of a `PWIT` section.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, NotAProduct> {
        if bytes.len() < PWIT_HEADER_LEN {
            return Err(bad_witness(FormatError::PlanTooShort));
        }
        if bytes[0..4] != PWIT_MAGIC {
            return Err(bad_witness(FormatError::PlanBadMagic));
        }
        if read_u16(&bytes[4..6]) != PLAN_SECTION_VERSION {
            return Err(bad_witness(FormatError::PlanUnsupportedVersion));
        }
        let slot_count = bytes[6];
        let step_count = bytes[7];
        let considered_per_step = bytes[8];
        let constraint_count = bytes[9];
        let decline = PackedDecline::from_code(bytes[10])
            .ok_or_else(|| bad_witness(FormatError::PlanInvalidRow))?;
        let recorded_verdict = bytes[11];
        if recorded_verdict > 2 {
            return Err(bad_witness(FormatError::PlanInvalidRow));
        }
        let recorded_step = read_u16(&bytes[12..14]);
        if read_u16(&bytes[14..16]) != 0 {
            return Err(bad_witness(FormatError::PlanNonZeroReserved));
        }
        if usize::from(slot_count) > PLAN_SLOTS_MAX || slot_count == 0 {
            return Err(bad_witness(FormatError::PlanBounds));
        }
        if usize::from(step_count) > PLAN_HORIZON_MAX
            || usize::from(considered_per_step) > PLAN_ACTIONS_MAX
            || usize::from(constraint_count) > PLAN_CONSTRAINTS_MAX
        {
            return Err(bad_witness(FormatError::PlanBounds));
        }

        let witness = Self {
            bytes,
            slot_count,
            step_count,
            considered_per_step,
            constraint_count,
            decline,
            recorded_verdict,
            recorded_step,
        };
        let expected = witness.considered_at()
            + usize::from(step_count) * usize::from(considered_per_step) * PWIT_CONSIDERED_LEN;
        if bytes.len() != expected {
            return Err(bad_witness(FormatError::PlanTrailingBytes));
        }
        // A witness that would exceed the frozen envelope is refused here: the
        // producer must decline with `Capacity`, never truncate what it emits.
        if bytes.len() > PLAN_WITNESS_MAX_BYTES {
            return Err(bad_witness(FormatError::PlanCapacityMismatch));
        }
        // Structural stage: every packed predicate, slot vector and step row
        // decodes, and the goal/constraint predicates are canonical.
        witness.initial_state()?;
        read_predicate(&bytes[PWIT_HEADER_LEN + PLAN_SLOTS_MAX * 2..])?;
        let mut previous: Option<&[u8]> = None;
        for index in 0..usize::from(constraint_count) {
            let at = PWIT_HEADER_LEN + PLAN_SLOTS_MAX * 2 + (index + 1) * PREDICATE_LEN;
            let raw = &bytes[at..at + PREDICATE_LEN];
            read_predicate(raw)?;
            if previous.is_some_and(|p| p >= raw) {
                return Err(bad_witness(FormatError::PlanNotCanonical));
            }
            previous = Some(raw);
        }
        for step in 0..usize::from(step_count) {
            witness
                .step(step)
                .ok_or_else(|| bad_witness(FormatError::PlanBounds))?;
        }
        for slot in 0..usize::from(step_count) * usize::from(considered_per_step) {
            witness
                .considered(slot)
                .ok_or_else(|| bad_witness(FormatError::PlanBounds))?;
        }
        Ok(witness)
    }

    /// Typed slots per valuation.
    pub fn slot_count(&self) -> usize {
        usize::from(self.slot_count)
    }

    /// Steps in the chosen path.
    pub fn step_count(&self) -> usize {
        usize::from(self.step_count)
    }

    /// The decline this witness records, if it is an honest abstention.
    pub fn decline(&self) -> Option<PackedDecline> {
        self.decline
    }

    /// The verdict the producer recorded. [`PlanWitnessBytes::replay`] does not
    /// trust it; a built test asserts the two agree.
    pub fn recorded_verdict(&self) -> (u8, u16) {
        (self.recorded_verdict, self.recorded_step)
    }

    /// The initial packed state.
    pub fn initial_state(&self) -> Result<SlotVec, NotAProduct> {
        read_slots(&self.bytes[PWIT_HEADER_LEN..], self.slot_count())
    }

    /// The inline goal predicate.
    pub fn goal(&self) -> Result<PreconditionMask, NotAProduct> {
        read_predicate(&self.bytes[PWIT_HEADER_LEN + PLAN_SLOTS_MAX * 2..])
    }

    /// The inline forbidden-region predicate `index`.
    pub fn constraint(&self, index: usize) -> Option<PreconditionMask> {
        if index >= usize::from(self.constraint_count) {
            return None;
        }
        let at = PWIT_HEADER_LEN + PLAN_SLOTS_MAX * 2 + (index + 1) * PREDICATE_LEN;
        read_predicate(self.bytes.get(at..)?).ok()
    }

    /// Step `index`: the effect applied, the resulting state, the chosen
    /// candidate slot, and the rule row it came from.
    pub fn step(&self, index: usize) -> Option<WitnessStep> {
        if index >= self.step_count() {
            return None;
        }
        let at = self.steps_at() + index * PWIT_STEP_LEN;
        let raw = self.bytes.get(at..at + PWIT_STEP_LEN)?;
        let arity = self.slot_count();
        let effect = read_effect(&raw[0..16], arity).ok()?;
        let resulting = read_slots(&raw[16..32], arity).ok()?;
        Some((
            effect,
            resulting,
            read_u16(&raw[32..34]),
            read_u16(&raw[34..36]),
        ))
    }

    /// Considered candidate `slot`, counted across all steps.
    pub fn considered(&self, slot: usize) -> Option<ConsideredCandidate> {
        let total = self.step_count() * usize::from(self.considered_per_step);
        if slot >= total {
            return None;
        }
        let at = self.considered_at() + slot * PWIT_CONSIDERED_LEN;
        let raw = self.bytes.get(at..at + PWIT_CONSIDERED_LEN)?;
        let band = raw[14];
        if band > 3 {
            return None;
        }
        Some(ConsideredCandidate {
            operator: read_u16(&raw[0..2]),
            rule_row: read_u16(&raw[2..4]),
            score: read_i32(&raw[4..8]),
            tie_rank: read_u16(&raw[8..10]),
            support: read_u32(&raw[10..14]),
            band,
            flags: raw[15],
        })
    }

    /// Independently re-verify this witness from its own bytes.
    ///
    /// Replays every recorded transition — asserting the recorded effect really
    /// carries the predecessor to the recorded successor — checks no
    /// intermediate state enters a forbidden region, and checks the terminal
    /// state satisfies the goal. Shares no code path with any planner. A right
    /// answer reached through an invalid intermediate step is rejected.
    pub fn replay(&self) -> ReplayVerdict {
        if let Some(decline) = self.decline {
            return ReplayVerdict::Declined(decline);
        }
        let Ok(mut state) = self.initial_state() else {
            return ReplayVerdict::Invalid {
                step: 0,
                reason: ReplayFault::EffectDoesNotProduceState,
            };
        };
        let Ok(goal) = self.goal() else {
            return ReplayVerdict::Invalid {
                step: self.step_count(),
                reason: ReplayFault::GoalNotSatisfied,
            };
        };
        for index in 0..self.step_count() {
            let Some((effect, recorded, _, _)) = self.step(index) else {
                return ReplayVerdict::Invalid {
                    step: index,
                    reason: ReplayFault::EffectDoesNotProduceState,
                };
            };
            match state.apply(&effect) {
                Some(next) if next == recorded => state = next,
                _ => {
                    return ReplayVerdict::Invalid {
                        step: index,
                        reason: ReplayFault::EffectDoesNotProduceState,
                    }
                }
            }
            for c in 0..usize::from(self.constraint_count) {
                if self.constraint(c).is_some_and(|p| p.holds(&state)) {
                    return ReplayVerdict::Invalid {
                        step: index,
                        reason: ReplayFault::EntersForbiddenRegion,
                    };
                }
            }
        }
        if goal.holds(&state) {
            ReplayVerdict::Valid
        } else {
            ReplayVerdict::Invalid {
                step: self.step_count(),
                reason: ReplayFault::GoalNotSatisfied,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Builders — offline only, so they are `alloc`-gated and never on a hot path
// ---------------------------------------------------------------------------

#[cfg(feature = "alloc")]
mod build {
    use super::*;
    use alloc::vec::Vec;

    fn put_u16(out: &mut Vec<u8>, value: u16) {
        out.extend_from_slice(&value.to_le_bytes());
    }

    fn put_u32(out: &mut Vec<u8>, value: u32) {
        out.extend_from_slice(&value.to_le_bytes());
    }

    fn put_slots(out: &mut Vec<u8>, values: &[i16]) {
        for slot in 0..PLAN_SLOTS_MAX {
            let value = values.get(slot).copied().unwrap_or(0);
            out.extend_from_slice(&value.to_le_bytes());
        }
    }

    fn encode_predicate(predicate: &PreconditionMask) -> [u8; PREDICATE_LEN] {
        let mut row = [0u8; PREDICATE_LEN];
        row[0] = predicate.read_mask();
        for slot in 0..PLAN_SLOTS_MAX {
            row[2 + slot] = predicate.op(slot).code();
            let bound = predicate.bound(slot).to_le_bytes();
            row[10 + slot * 2] = bound[0];
            row[11 + slot * 2] = bound[1];
        }
        row
    }

    /// Encode a `PSCH` planning schema. `None` when a bound is exceeded or the
    /// operator vocabulary is not strictly ascending after sorting (a
    /// duplicate effect), because the section has exactly one canonical form.
    pub fn build_schema(
        slot_count: u8,
        operators: &[EffectDelta],
        bands: (u32, u32, u32),
    ) -> Option<Vec<u8>> {
        if slot_count == 0 || usize::from(slot_count) > PLAN_SLOTS_MAX {
            return None;
        }
        if operators.len() > PLAN_ACTIONS_MAX {
            return None;
        }
        if !(bands.0 < bands.1 && bands.1 < bands.2) {
            return None;
        }
        let mut sorted = operators.to_vec();
        sorted.sort_unstable();
        if sorted.windows(2).any(|w| w[0] == w[1]) {
            return None;
        }
        if sorted.iter().any(|e| e.len() != usize::from(slot_count)) {
            return None;
        }
        let mut out = Vec::with_capacity(PSCH_HEADER_LEN + sorted.len() * PSCH_OPERATOR_LEN);
        out.extend_from_slice(&PSCH_MAGIC);
        put_u16(&mut out, PLAN_SECTION_VERSION);
        out.push(slot_count);
        out.push(PLAN_SLOT_BITS as u8);
        put_u16(&mut out, sorted.len() as u16);
        put_u16(&mut out, 0);
        put_u16(&mut out, PLAN_HORIZON_MAX as u16);
        put_u16(&mut out, PLAN_FRONTIER_MAX as u16);
        put_u16(&mut out, PLAN_ACTIONS_MAX as u16);
        put_u16(&mut out, PLAN_RULES_MAX as u16);
        put_u16(&mut out, PLAN_CONSTRAINTS_MAX as u16);
        put_u16(&mut out, PLAN_GOALS_MAX as u16);
        put_u32(&mut out, PLAN_VISITED_MAX as u32);
        put_u32(&mut out, PLAN_WITNESS_MAX_BYTES as u32);
        put_u32(&mut out, bands.0);
        put_u32(&mut out, bands.1);
        put_u32(&mut out, bands.2);
        put_u32(&mut out, 0);
        debug_assert_eq!(out.len(), PSCH_HEADER_LEN);
        for effect in &sorted {
            put_slots(&mut out, effect.as_slice());
        }
        Some(out)
    }

    fn encode_rule(rule: &PackedRule) -> [u8; PTRN_ROW_LEN] {
        let mut row = [0u8; PTRN_ROW_LEN];
        row[0..2].copy_from_slice(&rule.operator.to_le_bytes());
        row[2] = rule.precondition.read_mask();
        row[3] = rule.band;
        row[4..8].copy_from_slice(&rule.support.to_le_bytes());
        for slot in 0..PLAN_SLOTS_MAX {
            row[8 + slot] = rule.precondition.op(slot).code();
            let bound = rule.precondition.bound(slot).to_le_bytes();
            row[16 + slot * 2] = bound[0];
            row[17 + slot * 2] = bound[1];
            let delta = rule.effect.as_slice().get(slot).copied().unwrap_or(0);
            let delta = delta.to_le_bytes();
            row[32 + slot * 2] = delta[0];
            row[33 + slot * 2] = delta[1];
        }
        row
    }

    /// Encode a `PTRN` rule table and its operator index. Rows are sorted into
    /// the canonical `(operator, precondition, effect)` order here, so a caller
    /// cannot produce a non-canonical section by accident.
    pub fn build_rule_table(
        slot_count: u8,
        operator_count: u16,
        rules: &[PackedRule],
    ) -> Option<Vec<u8>> {
        if rules.len() > PLAN_RULES_MAX || usize::from(operator_count) > PLAN_ACTIONS_MAX {
            return None;
        }
        if rules
            .iter()
            .any(|r| r.operator >= operator_count || r.band > 3)
        {
            return None;
        }
        let mut rows: Vec<[u8; PTRN_ROW_LEN]> = rules.iter().map(encode_rule).collect();
        rows.sort_unstable_by(|a, b| {
            let left = (read_u16(&a[0..2]), &a[2..48]);
            let right = (read_u16(&b[0..2]), &b[2..48]);
            left.cmp(&right)
        });
        if rows
            .windows(2)
            .any(|w| (read_u16(&w[0][0..2]), &w[0][2..48]) == (read_u16(&w[1][0..2]), &w[1][2..48]))
        {
            return None;
        }
        let mut out = Vec::new();
        out.extend_from_slice(&PTRN_MAGIC);
        put_u16(&mut out, PLAN_SECTION_VERSION);
        put_u16(&mut out, rows.len() as u16);
        put_u16(&mut out, operator_count);
        out.push(slot_count);
        out.push(0);
        put_u32(&mut out, 0);
        debug_assert_eq!(out.len(), PTRN_HEADER_LEN);
        for row in &rows {
            out.extend_from_slice(row);
        }
        // The index tiles the row array in operator order.
        let mut cursor = 0usize;
        for operator in 0..usize::from(operator_count) {
            let first = cursor;
            while cursor < rows.len() && usize::from(read_u16(&rows[cursor][0..2])) == operator {
                cursor += 1;
            }
            put_u16(&mut out, first as u16);
            put_u16(&mut out, (cursor - first) as u16);
        }
        if cursor != rows.len() {
            return None;
        }
        Some(out)
    }

    /// Encode a `PGOL` predicate set. Each group is sorted into canonical order
    /// here; a duplicate predicate within a group is refused.
    pub fn build_predicate_set(
        slot_count: u8,
        goals: &[PreconditionMask],
        constraints: &[PreconditionMask],
    ) -> Option<Vec<u8>> {
        if goals.len() > PLAN_GOALS_MAX || constraints.len() > PLAN_CONSTRAINTS_MAX {
            return None;
        }
        let mut goal_rows: Vec<[u8; PREDICATE_LEN]> = goals.iter().map(encode_predicate).collect();
        let mut constraint_rows: Vec<[u8; PREDICATE_LEN]> =
            constraints.iter().map(encode_predicate).collect();
        goal_rows.sort_unstable();
        constraint_rows.sort_unstable();
        if goal_rows.windows(2).any(|w| w[0] == w[1])
            || constraint_rows.windows(2).any(|w| w[0] == w[1])
        {
            return None;
        }
        let mut out = Vec::new();
        out.extend_from_slice(&PGOL_MAGIC);
        put_u16(&mut out, PLAN_SECTION_VERSION);
        out.push(goal_rows.len() as u8);
        out.push(constraint_rows.len() as u8);
        out.push(slot_count);
        out.extend_from_slice(&[0, 0, 0]);
        debug_assert_eq!(out.len(), PGOL_HEADER_LEN);
        for row in goal_rows.iter().chain(constraint_rows.iter()) {
            out.extend_from_slice(row);
        }
        Some(out)
    }

    /// Everything one `PWIT` witness records.
    pub struct WitnessDraft<'a> {
        /// Typed slots per valuation.
        pub slot_count: u8,
        /// The initial packed state.
        pub initial: SlotVec,
        /// The goal predicate, carried inline so replay needs nothing else.
        pub goal: PreconditionMask,
        /// The forbidden-region predicates, carried inline.
        pub constraints: &'a [PreconditionMask],
        /// Per step: applied effect, resulting state, chosen candidate slot,
        /// and the rule row it came from.
        pub steps: &'a [WitnessStep],
        /// Considered candidates, row-major over steps.
        pub considered: &'a [ConsideredCandidate],
        /// Candidates recorded per step.
        pub considered_per_step: u8,
        /// Set when the episode is an honest decline rather than a plan.
        pub decline: Option<PackedDecline>,
        /// The verdict the producer recorded, and the step it names.
        pub verdict: (u8, u16),
    }

    /// Encode a `PWIT` witness. `None` when a bound is exceeded — including the
    /// frozen witness-byte envelope, which a producer must turn into
    /// `Decline(capacity)` rather than a truncated record.
    pub fn build_witness(draft: &WitnessDraft<'_>) -> Option<Vec<u8>> {
        if draft.slot_count == 0 || usize::from(draft.slot_count) > PLAN_SLOTS_MAX {
            return None;
        }
        if draft.steps.len() > PLAN_HORIZON_MAX
            || draft.constraints.len() > PLAN_CONSTRAINTS_MAX
            || usize::from(draft.considered_per_step) > PLAN_ACTIONS_MAX
        {
            return None;
        }
        if draft.considered.len() != draft.steps.len() * usize::from(draft.considered_per_step) {
            return None;
        }
        let mut constraint_rows: Vec<[u8; PREDICATE_LEN]> =
            draft.constraints.iter().map(encode_predicate).collect();
        constraint_rows.sort_unstable();
        if constraint_rows.windows(2).any(|w| w[0] == w[1]) {
            return None;
        }

        let mut out = Vec::new();
        out.extend_from_slice(&PWIT_MAGIC);
        put_u16(&mut out, PLAN_SECTION_VERSION);
        out.push(draft.slot_count);
        out.push(draft.steps.len() as u8);
        out.push(draft.considered_per_step);
        out.push(constraint_rows.len() as u8);
        out.push(draft.decline.map_or(0, PackedDecline::code));
        out.push(draft.verdict.0);
        put_u16(&mut out, draft.verdict.1);
        put_u16(&mut out, 0);
        debug_assert_eq!(out.len(), PWIT_HEADER_LEN);
        put_slots(&mut out, draft.initial.as_slice());
        out.extend_from_slice(&encode_predicate(&draft.goal));
        for row in &constraint_rows {
            out.extend_from_slice(row);
        }
        for (effect, resulting, chosen, rule_row) in draft.steps {
            put_slots(&mut out, effect.as_slice());
            put_slots(&mut out, resulting.as_slice());
            put_u16(&mut out, *chosen);
            put_u16(&mut out, *rule_row);
        }
        for candidate in draft.considered {
            if candidate.band > 3 {
                return None;
            }
            put_u16(&mut out, candidate.operator);
            put_u16(&mut out, candidate.rule_row);
            out.extend_from_slice(&candidate.score.to_le_bytes());
            put_u16(&mut out, candidate.tie_rank);
            put_u32(&mut out, candidate.support);
            out.push(candidate.band);
            out.push(candidate.flags);
        }
        if out.len() > PLAN_WITNESS_MAX_BYTES {
            return None;
        }
        Some(out)
    }
}

#[cfg(feature = "alloc")]
pub use build::{build_predicate_set, build_rule_table, build_schema, build_witness, WitnessDraft};
