//! Deterministic, selection-blind Gate 0 class-map helpers for issue #983.
//!
//! This test support deliberately stops at construction-derived action lookup
//! and a post-label strict ceiling.  It does not admit candidates, execute a
//! selector, invert a payload, or read a validation label while compiling a
//! class map or measuring structural coverage.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::Serialize;
use uor_r4_core::construction_causal_return_attention::{
    ConstructionCausalReturnAction, ConstructionCausalReturnClassEvent,
    ConstructionCausalReturnControlledObservation, ConstructionCausalReturnControlledRawCandidate,
    ConstructionCausalReturnControlledRepresentation, ConstructionCausalReturnExactRecallKey,
    ConstructionCausalReturnFullWord, ConstructionCausalReturnNegativeControl,
    ConstructionCausalReturnObservation, ConstructionCausalReturnRawCandidate,
    ConstructionCausalReturnRepresentation, CONSTRUCTION_CAUSAL_RETURN_CONSTRUCTION_ROWS,
};

const GATE0_SCHEMA: u32 = 1;
const GEOMETRIC_MAP_DOMAIN: &str = "uor-r4.construction-causal-return-gate0-map/1";
const EXACT_RECALL_MAP_DOMAIN: &str = "uor-r4.construction-causal-return-gate0-exact-recall-map/1";
const UNAVAILABLE_MAP_DOMAIN: &str = "uor-r4.construction-causal-return-gate0-unavailable-map/1";
const COVERAGE_DOMAIN: &str = "uor-r4.construction-causal-return-gate0-coverage/1";
const CEILING_DOMAIN: &str = "uor-r4.construction-causal-return-gate0-ceiling/1";
const PERMUTATION_DOMAIN: &str = "uor-r4.construction-causal-return-gate0-permutation/1";
pub const GATE0_CLASS_LOOKUP_SHAPE_IDENTITY: &str =
    "uor-r4.gate0-two-unified-typed-btree-class-slot-probes/1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gate0ClassMapError {
    Invalid(String),
    Serialization(String),
    ArithmeticOverflow,
}

impl fmt::Display for Gate0ClassMapError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => write!(formatter, "invalid Gate 0 class map: {reason}"),
            Self::Serialization(reason) => {
                write!(formatter, "Gate 0 canonical serialization failed: {reason}")
            }
            Self::ArithmeticOverflow => formatter.write_str("Gate 0 arithmetic overflow"),
        }
    }
}

impl std::error::Error for Gate0ClassMapError {}

/// The complete key material exposed by a real or controlled representation.
/// A disabled geometric arm has `Unavailable`; it is still queried with the
/// same two typed class reads and therefore cannot silently gain a shortcut.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum Gate0RepresentationKeys {
    Geometric {
        r_min: ConstructionCausalReturnClassEvent,
        r_full: ConstructionCausalReturnFullWord,
    },
    ExactRecall {
        exact_recall: ConstructionCausalReturnExactRecallKey,
    },
    Unavailable,
}

impl Gate0RepresentationKeys {
    pub fn from_real(representation: &ConstructionCausalReturnRepresentation) -> Self {
        Self::Geometric {
            r_min: representation.r_min(),
            r_full: representation.r_full(),
        }
    }

    pub fn from_controlled(
        representation: &ConstructionCausalReturnControlledRepresentation,
    ) -> Self {
        if let Some(exact_recall) = representation.exact_recall_key() {
            return Self::ExactRecall {
                exact_recall: exact_recall.clone(),
            };
        }
        match representation.r_min() {
            Some(r_min) => Self::Geometric {
                r_min,
                r_full: representation.r_full(),
            },
            None => Self::Unavailable,
        }
    }

    pub fn geometric(
        &self,
    ) -> Option<(
        ConstructionCausalReturnClassEvent,
        ConstructionCausalReturnFullWord,
    )> {
        match self {
            Self::Geometric { r_min, r_full } => Some((*r_min, *r_full)),
            Self::ExactRecall { .. } | Self::Unavailable => None,
        }
    }

    pub fn exact_recall(&self) -> Option<&ConstructionCausalReturnExactRecallKey> {
        match self {
            Self::ExactRecall { exact_recall } => Some(exact_recall),
            Self::Geometric { .. } | Self::Unavailable => None,
        }
    }
}

/// Construction-label row.  This is the only helper type that carries a
/// construction action.  Validation query rows below are label-free.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Gate0ObservationRow {
    pub transition_id: String,
    pub candidate_address_kappa: String,
    pub keys: Gate0RepresentationKeys,
    pub action: ConstructionCausalReturnAction,
}

impl Gate0ObservationRow {
    pub fn from_real(observation: &ConstructionCausalReturnObservation) -> Self {
        Self {
            transition_id: observation.transition_id().to_owned(),
            candidate_address_kappa: observation.candidate_address_kappa().to_owned(),
            keys: Gate0RepresentationKeys::from_real(observation.representation()),
            action: observation.action(),
        }
    }

    pub fn from_controlled(observation: &ConstructionCausalReturnControlledObservation) -> Self {
        Self {
            transition_id: observation.transition_id().to_owned(),
            candidate_address_kappa: observation.candidate_address_kappa().to_owned(),
            keys: Gate0RepresentationKeys::from_controlled(observation.representation()),
            action: observation.action(),
        }
    }
}

/// One naturally admitted validation candidate with no expected continuation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Gate0QueryCandidateRow {
    pub decision_id: String,
    pub candidate_address_kappa: String,
    pub keys: Gate0RepresentationKeys,
}

impl Gate0QueryCandidateRow {
    pub fn from_real(
        decision_id: impl Into<String>,
        candidate: &ConstructionCausalReturnRawCandidate,
    ) -> Self {
        Self {
            decision_id: decision_id.into(),
            candidate_address_kappa: candidate.candidate_address_kappa().to_owned(),
            keys: Gate0RepresentationKeys::from_real(candidate.representation()),
        }
    }

    pub fn from_controlled(
        decision_id: impl Into<String>,
        candidate: &ConstructionCausalReturnControlledRawCandidate,
    ) -> Self {
        Self {
            decision_id: decision_id.into(),
            candidate_address_kappa: candidate.candidate_address_kappa().to_owned(),
            keys: Gate0RepresentationKeys::from_controlled(candidate.representation()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Gate0RepresentationLevel {
    RMin,
    RFull,
    ExactRecall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Gate0LookupAbstention {
    RepresentationUnavailable,
    RepresentationKindMismatch,
    UnseenMinimumClass,
    UnseenRichClass,
    MultiplyMappedRichClass,
    UnseenExactRecallClass,
    MultiplyMappedExactRecallClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Gate0ActionLookup {
    Resolved {
        action: ConstructionCausalReturnAction,
        representation: Gate0RepresentationLevel,
    },
    Abstain {
        reason: Gate0LookupAbstention,
    },
}

/// Every lookup performs the minimum/exact slot and the rich/typed-no-op slot.
/// A direct `R_min` hit does not erase the second read from the work ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Gate0TwoClassReadAccounting {
    pub lookup_shape_identity: &'static str,
    pub unified_typed_slot_table: bool,
    pub declared_class_reads: usize,
    pub performed_class_reads: usize,
    pub minimum_or_exact_reads: usize,
    pub rich_or_typed_noop_reads: usize,
}

impl Gate0TwoClassReadAccounting {
    pub const fn exact() -> Self {
        Self {
            lookup_shape_identity: GATE0_CLASS_LOOKUP_SHAPE_IDENTITY,
            unified_typed_slot_table: true,
            declared_class_reads: 2,
            performed_class_reads: 2,
            minimum_or_exact_reads: 1,
            rich_or_typed_noop_reads: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gate0ActionLookupReport {
    pub keys: Gate0RepresentationKeys,
    pub class_reads: Gate0TwoClassReadAccounting,
    pub lookup: Gate0ActionLookup,
}

pub trait Gate0ClassLookup {
    fn map_kappa(&self) -> &str;

    fn lookup(&self, keys: &Gate0RepresentationKeys) -> Gate0ActionLookupReport;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gate0RichClassInventory {
    pub r_full: ConstructionCausalReturnFullWord,
    pub construction_rows: usize,
    pub select_rows: usize,
    pub reject_rows: usize,
    pub pure_action: Option<ConstructionCausalReturnAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gate0MinimumClassInventory {
    pub r_min: ConstructionCausalReturnClassEvent,
    pub construction_rows: usize,
    pub select_rows: usize,
    pub reject_rows: usize,
    pub pure_at_r_min: bool,
    pub direct_action: Option<ConstructionCausalReturnAction>,
    pub promoted_to_r_full: bool,
    pub all_promoted_rich_classes_pure: bool,
    pub rich_classes: Vec<Gate0RichClassInventory>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MinimumResolution {
    Direct(ConstructionCausalReturnAction),
    Promoted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RichResolution {
    Pure(ConstructionCausalReturnAction),
    MultiplyMapped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExactRecallResolution {
    Pure(ConstructionCausalReturnAction),
    MultiplyMapped,
}

/// One common typed key domain is used by every real and control arm.  The
/// two required reads therefore execute the same `BTreeMap::get` operation
/// shape even when a representation is unavailable or a second slot is a
/// typed no-op.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(clippy::large_enum_variant)]
enum Gate0ClassSlotKey {
    Minimum(ConstructionCausalReturnClassEvent),
    Rich {
        r_min: ConstructionCausalReturnClassEvent,
        r_full: ConstructionCausalReturnFullWord,
    },
    ExactRecall(ConstructionCausalReturnExactRecallKey),
    TypedNoop(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Gate0ClassSlotValue {
    Minimum(MinimumResolution),
    Rich(RichResolution),
    ExactRecall(ExactRecallResolution),
    TypedNoop,
}

fn typed_class_slot_table() -> BTreeMap<Gate0ClassSlotKey, Gate0ClassSlotValue> {
    BTreeMap::from([
        (
            Gate0ClassSlotKey::TypedNoop(0),
            Gate0ClassSlotValue::TypedNoop,
        ),
        (
            Gate0ClassSlotKey::TypedNoop(1),
            Gate0ClassSlotValue::TypedNoop,
        ),
    ])
}

fn perform_two_class_slot_reads<'a>(
    slots: &'a BTreeMap<Gate0ClassSlotKey, Gate0ClassSlotValue>,
    first_key: &Gate0ClassSlotKey,
    second_key: &Gate0ClassSlotKey,
) -> (
    Option<&'a Gate0ClassSlotValue>,
    Option<&'a Gate0ClassSlotValue>,
) {
    let slots = std::hint::black_box(slots);
    let first = std::hint::black_box(slots.get(std::hint::black_box(first_key)));
    let second = std::hint::black_box(slots.get(std::hint::black_box(second_key)));
    (first, second)
}

#[derive(Debug, Serialize)]
struct GeometricMapSeed<'a> {
    schema: u32,
    domain: &'static str,
    observation_rows: usize,
    minimum_class_count: usize,
    rich_class_count: usize,
    promoted_minimum_classes: usize,
    promoted_rows: usize,
    all_selection_classes_pure: bool,
    inventory: &'a [Gate0MinimumClassInventory],
}

/// Construction-only two-level map.  Its lookup tables are skipped during
/// serialization because JSON object keys cannot carry the exact typed H4
/// tuples; the complete, ordered inventory is the canonical serialized form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gate0GeometricClassMap {
    pub schema: u32,
    pub domain: &'static str,
    pub observation_rows: usize,
    pub minimum_class_count: usize,
    pub rich_class_count: usize,
    pub promoted_minimum_classes: usize,
    pub promoted_rows: usize,
    pub all_selection_classes_pure: bool,
    pub inventory: Vec<Gate0MinimumClassInventory>,
    pub artifact_kappa: String,
    #[serde(skip)]
    source_rows: Vec<Gate0ObservationRow>,
    #[serde(skip)]
    class_slots: BTreeMap<Gate0ClassSlotKey, Gate0ClassSlotValue>,
}

impl Gate0GeometricClassMap {
    pub fn compile(rows: &[Gate0ObservationRow]) -> Result<Self, Gate0ClassMapError> {
        let source_rows = canonical_observation_rows(rows)?;
        let mut grouped = BTreeMap::<
            ConstructionCausalReturnClassEvent,
            BTreeMap<ConstructionCausalReturnFullWord, Vec<ConstructionCausalReturnAction>>,
        >::new();
        for row in &source_rows {
            let Some((r_min, r_full)) = row.keys.geometric() else {
                return Err(Gate0ClassMapError::Invalid(
                    "geometric construction map received a non-geometric row".to_owned(),
                ));
            };
            grouped
                .entry(r_min)
                .or_default()
                .entry(r_full)
                .or_default()
                .push(row.action);
        }

        let mut class_slots = typed_class_slot_table();
        let mut inventory = Vec::with_capacity(grouped.len());
        let mut promoted_minimum_classes = 0usize;
        let mut promoted_rows = 0usize;
        let mut all_selection_classes_pure = true;

        for (r_min, rich_groups) in grouped {
            let all_actions = rich_groups.values().flatten().copied().collect::<Vec<_>>();
            let (select_rows, reject_rows) = action_counts(&all_actions)?;
            let distinct = all_actions.iter().copied().collect::<BTreeSet<_>>();
            let complete_rich_inventory = rich_groups
                .iter()
                .map(|(r_full, actions)| {
                    let (rich_select_rows, rich_reject_rows) = action_counts(actions)?;
                    let rich_distinct = actions.iter().copied().collect::<BTreeSet<_>>();
                    Ok(Gate0RichClassInventory {
                        r_full: *r_full,
                        construction_rows: actions.len(),
                        select_rows: rich_select_rows,
                        reject_rows: rich_reject_rows,
                        pure_action: (rich_distinct.len() == 1)
                            .then(|| rich_distinct.iter().next().copied())
                            .flatten(),
                    })
                })
                .collect::<Result<Vec<_>, Gate0ClassMapError>>()?;
            if distinct.len() == 1 {
                let action = *distinct
                    .iter()
                    .next()
                    .ok_or_else(|| Gate0ClassMapError::Invalid("empty minimum class".to_owned()))?;
                class_slots.insert(
                    Gate0ClassSlotKey::Minimum(r_min),
                    Gate0ClassSlotValue::Minimum(MinimumResolution::Direct(action)),
                );
                for r_full in rich_groups.keys() {
                    class_slots.insert(
                        Gate0ClassSlotKey::Rich {
                            r_min,
                            r_full: *r_full,
                        },
                        Gate0ClassSlotValue::TypedNoop,
                    );
                }
                inventory.push(Gate0MinimumClassInventory {
                    r_min,
                    construction_rows: all_actions.len(),
                    select_rows,
                    reject_rows,
                    pure_at_r_min: true,
                    direct_action: Some(action),
                    promoted_to_r_full: false,
                    all_promoted_rich_classes_pure: true,
                    rich_classes: complete_rich_inventory,
                });
                continue;
            }

            promoted_minimum_classes = checked_add(promoted_minimum_classes, 1)?;
            promoted_rows = checked_add(promoted_rows, all_actions.len())?;
            class_slots.insert(
                Gate0ClassSlotKey::Minimum(r_min),
                Gate0ClassSlotValue::Minimum(MinimumResolution::Promoted),
            );
            let mut all_rich_pure = true;
            for (r_full, actions) in rich_groups {
                let rich_distinct = actions.iter().copied().collect::<BTreeSet<_>>();
                let pure_action = if rich_distinct.len() == 1 {
                    rich_distinct.iter().next().copied()
                } else {
                    None
                };
                let resolution =
                    pure_action.map_or(RichResolution::MultiplyMapped, RichResolution::Pure);
                if pure_action.is_none() {
                    all_rich_pure = false;
                    all_selection_classes_pure = false;
                }
                class_slots.insert(
                    Gate0ClassSlotKey::Rich { r_min, r_full },
                    Gate0ClassSlotValue::Rich(resolution),
                );
            }
            inventory.push(Gate0MinimumClassInventory {
                r_min,
                construction_rows: all_actions.len(),
                select_rows,
                reject_rows,
                pure_at_r_min: false,
                direct_action: None,
                promoted_to_r_full: true,
                all_promoted_rich_classes_pure: all_rich_pure,
                rich_classes: complete_rich_inventory,
            });
        }

        let rich_class_count = inventory
            .iter()
            .map(|record| record.rich_classes.len())
            .try_fold(0usize, checked_add)?;
        let mut map = Self {
            schema: GATE0_SCHEMA,
            domain: GEOMETRIC_MAP_DOMAIN,
            observation_rows: source_rows.len(),
            minimum_class_count: inventory.len(),
            rich_class_count,
            promoted_minimum_classes,
            promoted_rows,
            all_selection_classes_pure,
            inventory,
            artifact_kappa: String::new(),
            source_rows,
            class_slots,
        };
        map.artifact_kappa = record_kappa(&map.seed())?;
        Ok(map)
    }

    pub fn source_rows(&self) -> &[Gate0ObservationRow] {
        &self.source_rows
    }

    pub fn promoted_rate_numerator(&self) -> usize {
        self.promoted_rows
    }

    pub fn promoted_rate_denominator(&self) -> usize {
        self.observation_rows
    }

    pub fn reproduce_artifact_kappa(&self) -> Result<String, Gate0ClassMapError> {
        record_kappa(&self.seed())
    }

    fn seed(&self) -> GeometricMapSeed<'_> {
        GeometricMapSeed {
            schema: self.schema,
            domain: self.domain,
            observation_rows: self.observation_rows,
            minimum_class_count: self.minimum_class_count,
            rich_class_count: self.rich_class_count,
            promoted_minimum_classes: self.promoted_minimum_classes,
            promoted_rows: self.promoted_rows,
            all_selection_classes_pure: self.all_selection_classes_pure,
            inventory: &self.inventory,
        }
    }
}

impl Gate0ClassLookup for Gate0GeometricClassMap {
    fn map_kappa(&self) -> &str {
        &self.artifact_kappa
    }

    fn lookup(&self, keys: &Gate0RepresentationKeys) -> Gate0ActionLookupReport {
        let class_reads = Gate0TwoClassReadAccounting::exact();
        let lookup = match keys {
            Gate0RepresentationKeys::Geometric { r_min, r_full } => {
                // Both probes are intentionally evaluated before either result
                // is interpreted, preserving the exact two-read contract.
                let minimum_key = Gate0ClassSlotKey::Minimum(*r_min);
                let rich_key = Gate0ClassSlotKey::Rich {
                    r_min: *r_min,
                    r_full: *r_full,
                };
                let (minimum_slot, rich_slot) =
                    perform_two_class_slot_reads(&self.class_slots, &minimum_key, &rich_key);
                let minimum = match minimum_slot {
                    Some(Gate0ClassSlotValue::Minimum(resolution)) => Some(resolution),
                    _ => None,
                };
                match minimum {
                    None => Gate0ActionLookup::Abstain {
                        reason: Gate0LookupAbstention::UnseenMinimumClass,
                    },
                    Some(MinimumResolution::Direct(action)) => Gate0ActionLookup::Resolved {
                        action: *action,
                        representation: Gate0RepresentationLevel::RMin,
                    },
                    Some(MinimumResolution::Promoted) => match rich_slot {
                        None => Gate0ActionLookup::Abstain {
                            reason: Gate0LookupAbstention::UnseenRichClass,
                        },
                        Some(Gate0ClassSlotValue::Rich(RichResolution::Pure(action))) => {
                            Gate0ActionLookup::Resolved {
                                action: *action,
                                representation: Gate0RepresentationLevel::RFull,
                            }
                        }
                        Some(Gate0ClassSlotValue::Rich(RichResolution::MultiplyMapped)) => {
                            Gate0ActionLookup::Abstain {
                                reason: Gate0LookupAbstention::MultiplyMappedRichClass,
                            }
                        }
                        Some(
                            Gate0ClassSlotValue::Minimum(_)
                            | Gate0ClassSlotValue::ExactRecall(_)
                            | Gate0ClassSlotValue::TypedNoop,
                        ) => Gate0ActionLookup::Abstain {
                            reason: Gate0LookupAbstention::UnseenRichClass,
                        },
                    },
                }
            }
            Gate0RepresentationKeys::ExactRecall { .. } => Gate0ActionLookup::Abstain {
                reason: {
                    let _ = perform_two_class_slot_reads(
                        &self.class_slots,
                        &Gate0ClassSlotKey::TypedNoop(0),
                        &Gate0ClassSlotKey::TypedNoop(1),
                    );
                    Gate0LookupAbstention::RepresentationKindMismatch
                },
            },
            Gate0RepresentationKeys::Unavailable => Gate0ActionLookup::Abstain {
                reason: {
                    let _ = perform_two_class_slot_reads(
                        &self.class_slots,
                        &Gate0ClassSlotKey::TypedNoop(0),
                        &Gate0ClassSlotKey::TypedNoop(1),
                    );
                    Gate0LookupAbstention::RepresentationUnavailable
                },
            },
        };
        Gate0ActionLookupReport {
            keys: keys.clone(),
            class_reads,
            lookup,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gate0ExactRecallClassInventory {
    pub exact_recall: ConstructionCausalReturnExactRecallKey,
    pub construction_rows: usize,
    pub select_rows: usize,
    pub reject_rows: usize,
    pub pure_action: Option<ConstructionCausalReturnAction>,
}

#[derive(Debug, Serialize)]
struct ExactRecallMapSeed<'a> {
    schema: u32,
    domain: &'static str,
    observation_rows: usize,
    class_count: usize,
    all_classes_pure: bool,
    inventory: &'a [Gate0ExactRecallClassInventory],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gate0ExactRecallMap {
    pub schema: u32,
    pub domain: &'static str,
    pub observation_rows: usize,
    pub class_count: usize,
    pub all_classes_pure: bool,
    pub inventory: Vec<Gate0ExactRecallClassInventory>,
    pub artifact_kappa: String,
    #[serde(skip)]
    source_rows: Vec<Gate0ObservationRow>,
    #[serde(skip)]
    class_slots: BTreeMap<Gate0ClassSlotKey, Gate0ClassSlotValue>,
}

impl Gate0ExactRecallMap {
    pub fn compile(rows: &[Gate0ObservationRow]) -> Result<Self, Gate0ClassMapError> {
        let source_rows = canonical_observation_rows(rows)?;
        let mut grouped = BTreeMap::<
            ConstructionCausalReturnExactRecallKey,
            Vec<ConstructionCausalReturnAction>,
        >::new();
        for row in &source_rows {
            let Some(key) = row.keys.exact_recall() else {
                return Err(Gate0ClassMapError::Invalid(
                    "exact-recall construction map received a non-exact row".to_owned(),
                ));
            };
            grouped.entry(key.clone()).or_default().push(row.action);
        }

        let mut class_slots = typed_class_slot_table();
        let mut inventory = Vec::with_capacity(grouped.len());
        let mut all_classes_pure = true;
        for (exact_recall, actions) in grouped {
            let (select_rows, reject_rows) = action_counts(&actions)?;
            let distinct = actions.iter().copied().collect::<BTreeSet<_>>();
            let pure_action = if distinct.len() == 1 {
                distinct.iter().next().copied()
            } else {
                None
            };
            if pure_action.is_none() {
                all_classes_pure = false;
            }
            let resolution = pure_action.map_or(
                ExactRecallResolution::MultiplyMapped,
                ExactRecallResolution::Pure,
            );
            class_slots.insert(
                Gate0ClassSlotKey::ExactRecall(exact_recall.clone()),
                Gate0ClassSlotValue::ExactRecall(resolution),
            );
            inventory.push(Gate0ExactRecallClassInventory {
                exact_recall,
                construction_rows: actions.len(),
                select_rows,
                reject_rows,
                pure_action,
            });
        }

        let mut map = Self {
            schema: GATE0_SCHEMA,
            domain: EXACT_RECALL_MAP_DOMAIN,
            observation_rows: source_rows.len(),
            class_count: inventory.len(),
            all_classes_pure,
            inventory,
            artifact_kappa: String::new(),
            source_rows,
            class_slots,
        };
        map.artifact_kappa = record_kappa(&map.seed())?;
        Ok(map)
    }

    pub fn source_rows(&self) -> &[Gate0ObservationRow] {
        &self.source_rows
    }

    pub fn reproduce_artifact_kappa(&self) -> Result<String, Gate0ClassMapError> {
        record_kappa(&self.seed())
    }

    fn seed(&self) -> ExactRecallMapSeed<'_> {
        ExactRecallMapSeed {
            schema: self.schema,
            domain: self.domain,
            observation_rows: self.observation_rows,
            class_count: self.class_count,
            all_classes_pure: self.all_classes_pure,
            inventory: &self.inventory,
        }
    }
}

impl Gate0ClassLookup for Gate0ExactRecallMap {
    fn map_kappa(&self) -> &str {
        &self.artifact_kappa
    }

    fn lookup(&self, keys: &Gate0RepresentationKeys) -> Gate0ActionLookupReport {
        let class_reads = Gate0TwoClassReadAccounting::exact();
        let lookup = match keys {
            Gate0RepresentationKeys::ExactRecall { exact_recall } => {
                let exact_key = Gate0ClassSlotKey::ExactRecall(exact_recall.clone());
                let (exact, _) = perform_two_class_slot_reads(
                    &self.class_slots,
                    &exact_key,
                    &Gate0ClassSlotKey::TypedNoop(1),
                );
                match exact {
                    None => Gate0ActionLookup::Abstain {
                        reason: Gate0LookupAbstention::UnseenExactRecallClass,
                    },
                    Some(Gate0ClassSlotValue::ExactRecall(ExactRecallResolution::Pure(action))) => {
                        Gate0ActionLookup::Resolved {
                            action: *action,
                            representation: Gate0RepresentationLevel::ExactRecall,
                        }
                    }
                    Some(Gate0ClassSlotValue::ExactRecall(
                        ExactRecallResolution::MultiplyMapped,
                    )) => Gate0ActionLookup::Abstain {
                        reason: Gate0LookupAbstention::MultiplyMappedExactRecallClass,
                    },
                    Some(
                        Gate0ClassSlotValue::Minimum(_)
                        | Gate0ClassSlotValue::Rich(_)
                        | Gate0ClassSlotValue::TypedNoop,
                    ) => Gate0ActionLookup::Abstain {
                        reason: Gate0LookupAbstention::UnseenExactRecallClass,
                    },
                }
            }
            Gate0RepresentationKeys::Geometric { .. } => Gate0ActionLookup::Abstain {
                reason: {
                    let _ = perform_two_class_slot_reads(
                        &self.class_slots,
                        &Gate0ClassSlotKey::TypedNoop(0),
                        &Gate0ClassSlotKey::TypedNoop(1),
                    );
                    Gate0LookupAbstention::RepresentationKindMismatch
                },
            },
            Gate0RepresentationKeys::Unavailable => Gate0ActionLookup::Abstain {
                reason: {
                    let _ = perform_two_class_slot_reads(
                        &self.class_slots,
                        &Gate0ClassSlotKey::TypedNoop(0),
                        &Gate0ClassSlotKey::TypedNoop(1),
                    );
                    Gate0LookupAbstention::RepresentationUnavailable
                },
            },
        };
        Gate0ActionLookupReport {
            keys: keys.clone(),
            class_reads,
            lookup,
        }
    }
}

/// Full provenance for one controlled construction row in an arm whose
/// representation is deliberately unavailable.  Retaining the control/frame
/// binding prevents a synthetic empty map from standing in for the actual
/// twenty-four equal-support construction rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gate0UnavailableObservationBinding {
    pub frame_kappa: String,
    pub control: ConstructionCausalReturnNegativeControl,
    pub control_input_kappa: String,
    pub observation: Gate0ObservationRow,
}

#[derive(Debug, Serialize)]
struct UnavailableMapSeed<'a> {
    schema: u32,
    domain: &'static str,
    control: ConstructionCausalReturnNegativeControl,
    frame_kappa: &'a str,
    observation_rows: usize,
    construction_rows: &'a [Gate0UnavailableObservationBinding],
}

/// Content-bound inventory for a disabled representation arm.  It compiles
/// only from the typed controlled observations, verifies that every one of the
/// exact 24 rows is unavailable, and always performs two abstaining reads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gate0UnavailableClassMap {
    pub schema: u32,
    pub domain: &'static str,
    pub control: ConstructionCausalReturnNegativeControl,
    pub frame_kappa: String,
    pub observation_rows: usize,
    pub construction_rows: Vec<Gate0UnavailableObservationBinding>,
    pub artifact_kappa: String,
    #[serde(skip)]
    class_slots: BTreeMap<Gate0ClassSlotKey, Gate0ClassSlotValue>,
}

impl Gate0UnavailableClassMap {
    pub fn compile(
        observations: &[ConstructionCausalReturnControlledObservation],
    ) -> Result<Self, Gate0ClassMapError> {
        if observations.len() != CONSTRUCTION_CAUSAL_RETURN_CONSTRUCTION_ROWS {
            return Err(Gate0ClassMapError::Invalid(format!(
                "unavailable control map requires exactly {CONSTRUCTION_CAUSAL_RETURN_CONSTRUCTION_ROWS} controlled construction rows"
            )));
        }
        let canonical_rows = canonical_observation_rows(
            &observations
                .iter()
                .map(Gate0ObservationRow::from_controlled)
                .collect::<Vec<_>>(),
        )?;
        if canonical_rows
            .iter()
            .any(|row| row.keys != Gate0RepresentationKeys::Unavailable)
        {
            return Err(Gate0ClassMapError::Invalid(
                "unavailable control map received an operative representation".to_owned(),
            ));
        }

        let first = observations.first().ok_or_else(|| {
            Gate0ClassMapError::Invalid("unavailable control map lost its rows".to_owned())
        })?;
        let control = first.control();
        let frame_kappa = first.frame_kappa().to_owned();
        if observations.iter().any(|observation| {
            observation.control() != control || observation.frame_kappa() != frame_kappa.as_str()
        }) {
            return Err(Gate0ClassMapError::Invalid(
                "unavailable control rows do not share one frame/control binding".to_owned(),
            ));
        }

        let provenance = observations
            .iter()
            .map(|observation| {
                (
                    (
                        observation.transition_id(),
                        observation.candidate_address_kappa(),
                    ),
                    observation,
                )
            })
            .collect::<BTreeMap<_, _>>();
        if provenance.len() != canonical_rows.len() {
            return Err(Gate0ClassMapError::Invalid(
                "unavailable control provenance contains duplicate construction identities"
                    .to_owned(),
            ));
        }
        let construction_rows = canonical_rows
            .into_iter()
            .map(|observation| {
                let source = provenance
                    .get(&(
                        observation.transition_id.as_str(),
                        observation.candidate_address_kappa.as_str(),
                    ))
                    .ok_or_else(|| {
                        Gate0ClassMapError::Invalid(
                            "unavailable control provenance lost a canonical row".to_owned(),
                        )
                    })?;
                Ok(Gate0UnavailableObservationBinding {
                    frame_kappa: source.frame_kappa().to_owned(),
                    control: source.control(),
                    control_input_kappa: source.control_input_kappa().to_owned(),
                    observation,
                })
            })
            .collect::<Result<Vec<_>, Gate0ClassMapError>>()?;

        let mut map = Self {
            schema: GATE0_SCHEMA,
            domain: UNAVAILABLE_MAP_DOMAIN,
            control,
            frame_kappa,
            observation_rows: construction_rows.len(),
            construction_rows,
            artifact_kappa: String::new(),
            class_slots: typed_class_slot_table(),
        };
        map.artifact_kappa = record_kappa(&map.seed())?;
        Ok(map)
    }

    pub fn reproduce_artifact_kappa(&self) -> Result<String, Gate0ClassMapError> {
        record_kappa(&self.seed())
    }

    fn seed(&self) -> UnavailableMapSeed<'_> {
        UnavailableMapSeed {
            schema: self.schema,
            domain: self.domain,
            control: self.control,
            frame_kappa: &self.frame_kappa,
            observation_rows: self.observation_rows,
            construction_rows: &self.construction_rows,
        }
    }
}

impl Gate0ClassLookup for Gate0UnavailableClassMap {
    fn map_kappa(&self) -> &str {
        &self.artifact_kappa
    }

    fn lookup(&self, keys: &Gate0RepresentationKeys) -> Gate0ActionLookupReport {
        let _ = perform_two_class_slot_reads(
            &self.class_slots,
            &Gate0ClassSlotKey::TypedNoop(0),
            &Gate0ClassSlotKey::TypedNoop(1),
        );
        let reason = if keys == &Gate0RepresentationKeys::Unavailable {
            Gate0LookupAbstention::RepresentationUnavailable
        } else {
            Gate0LookupAbstention::RepresentationKindMismatch
        };
        Gate0ActionLookupReport {
            keys: keys.clone(),
            class_reads: Gate0TwoClassReadAccounting::exact(),
            lookup: Gate0ActionLookup::Abstain { reason },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gate0StructuralCandidateCoverage {
    pub candidate_address_kappa: String,
    pub covered: bool,
    pub representation: Option<Gate0RepresentationLevel>,
    pub abstention: Option<Gate0LookupAbstention>,
    pub class_reads: Gate0TwoClassReadAccounting,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gate0StructuralDecisionCoverage {
    pub decision_id: String,
    pub candidate_count: usize,
    pub all_candidates_covered: bool,
    pub single_select_single_reject_shape: bool,
    pub structurally_covered: bool,
    pub candidates: Vec<Gate0StructuralCandidateCoverage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gate0StructuralCoverageReport {
    pub schema: u32,
    pub domain: &'static str,
    pub map_kappa: String,
    pub decision_count: usize,
    pub candidate_count: usize,
    pub covered_decisions: usize,
    pub covered_candidates: usize,
    pub resolved_at_r_min: usize,
    pub resolved_at_r_full: usize,
    pub resolved_by_exact_recall: usize,
    pub declared_class_reads: usize,
    pub performed_class_reads: usize,
    pub decisions: Vec<Gate0StructuralDecisionCoverage>,
    pub coverage_kappa: String,
}

#[derive(Debug, Serialize)]
struct StructuralCoverageSeed<'a> {
    schema: u32,
    domain: &'static str,
    map_kappa: &'a str,
    decision_count: usize,
    candidate_count: usize,
    covered_decisions: usize,
    covered_candidates: usize,
    resolved_at_r_min: usize,
    resolved_at_r_full: usize,
    resolved_by_exact_recall: usize,
    declared_class_reads: usize,
    performed_class_reads: usize,
    decisions: &'a [Gate0StructuralDecisionCoverage],
}

/// Selection-blind structural coverage.  The output records whether the
/// construction map can produce the frozen one-select/one-reject shape, but
/// it accepts no validation expectation and reports no winning candidate.
pub fn label_free_structural_coverage<M: Gate0ClassLookup>(
    map: &M,
    rows: &[Gate0QueryCandidateRow],
) -> Result<Gate0StructuralCoverageReport, Gate0ClassMapError> {
    let grouped = canonical_query_groups(rows)?;
    let mut decisions = Vec::with_capacity(grouped.len());
    let mut candidate_count = 0usize;
    let mut covered_decisions = 0usize;
    let mut covered_candidates = 0usize;
    let mut resolved_at_r_min = 0usize;
    let mut resolved_at_r_full = 0usize;
    let mut resolved_by_exact_recall = 0usize;
    let mut declared_class_reads = 0usize;
    let mut performed_class_reads = 0usize;

    for (decision_id, candidates) in grouped {
        let mut structural_candidates = Vec::with_capacity(candidates.len());
        let mut actions = Vec::with_capacity(candidates.len());
        let mut all_candidates_covered = true;
        for candidate in candidates {
            let report = map.lookup(&candidate.keys);
            candidate_count = checked_add(candidate_count, 1)?;
            declared_class_reads = checked_add(
                declared_class_reads,
                report.class_reads.declared_class_reads,
            )?;
            performed_class_reads = checked_add(
                performed_class_reads,
                report.class_reads.performed_class_reads,
            )?;
            let (covered, representation, abstention) = match report.lookup {
                Gate0ActionLookup::Resolved {
                    action,
                    representation,
                } => {
                    covered_candidates = checked_add(covered_candidates, 1)?;
                    match representation {
                        Gate0RepresentationLevel::RMin => {
                            resolved_at_r_min = checked_add(resolved_at_r_min, 1)?;
                        }
                        Gate0RepresentationLevel::RFull => {
                            resolved_at_r_full = checked_add(resolved_at_r_full, 1)?;
                        }
                        Gate0RepresentationLevel::ExactRecall => {
                            resolved_by_exact_recall = checked_add(resolved_by_exact_recall, 1)?;
                        }
                    }
                    actions.push(action);
                    (true, Some(representation), None)
                }
                Gate0ActionLookup::Abstain { reason } => {
                    all_candidates_covered = false;
                    (false, None, Some(reason))
                }
            };
            structural_candidates.push(Gate0StructuralCandidateCoverage {
                candidate_address_kappa: candidate.candidate_address_kappa,
                covered,
                representation,
                abstention,
                class_reads: report.class_reads,
            });
        }
        let single_select_single_reject_shape = actions.len() == 2
            && actions
                .iter()
                .filter(|action| **action == ConstructionCausalReturnAction::Select)
                .count()
                == 1
            && actions
                .iter()
                .filter(|action| **action == ConstructionCausalReturnAction::Reject)
                .count()
                == 1;
        let structurally_covered = all_candidates_covered && single_select_single_reject_shape;
        if structurally_covered {
            covered_decisions = checked_add(covered_decisions, 1)?;
        }
        decisions.push(Gate0StructuralDecisionCoverage {
            decision_id,
            candidate_count: structural_candidates.len(),
            all_candidates_covered,
            single_select_single_reject_shape,
            structurally_covered,
            candidates: structural_candidates,
        });
    }

    let mut report = Gate0StructuralCoverageReport {
        schema: GATE0_SCHEMA,
        domain: COVERAGE_DOMAIN,
        map_kappa: map.map_kappa().to_owned(),
        decision_count: decisions.len(),
        candidate_count,
        covered_decisions,
        covered_candidates,
        resolved_at_r_min,
        resolved_at_r_full,
        resolved_by_exact_recall,
        declared_class_reads,
        performed_class_reads,
        decisions,
        coverage_kappa: String::new(),
    };
    report.coverage_kappa = record_kappa(&StructuralCoverageSeed {
        schema: report.schema,
        domain: report.domain,
        map_kappa: &report.map_kappa,
        decision_count: report.decision_count,
        candidate_count: report.candidate_count,
        covered_decisions: report.covered_decisions,
        covered_candidates: report.covered_candidates,
        resolved_at_r_min: report.resolved_at_r_min,
        resolved_at_r_full: report.resolved_at_r_full,
        resolved_by_exact_recall: report.resolved_by_exact_recall,
        declared_class_reads: report.declared_class_reads,
        performed_class_reads: report.performed_class_reads,
        decisions: &report.decisions,
    })?;
    Ok(report)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Gate0ValidationLabel {
    pub decision_id: String,
    pub expected_candidate_address_kappa: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gate0CeilingCandidateLookup {
    pub candidate_address_kappa: String,
    pub lookup: Gate0ActionLookupReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gate0CeilingDecision {
    pub decision_id: String,
    pub expected_candidate_address_kappa: String,
    pub selected_candidate_address_kappa: Option<String>,
    pub strict_ceiling_hit: bool,
    pub abstained: bool,
    pub tied_or_multiply_selected: bool,
    pub candidates: Vec<Gate0CeilingCandidateLookup>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gate0StrictCeilingReport {
    pub schema: u32,
    pub domain: &'static str,
    pub map_kappa: String,
    pub decision_count: usize,
    pub strict_ceiling_hits: usize,
    pub abstentions: usize,
    pub ties_or_multiply_selected: usize,
    pub declared_class_reads: usize,
    pub performed_class_reads: usize,
    pub decisions: Vec<Gate0CeilingDecision>,
    pub ceiling_kappa: String,
}

#[derive(Debug, Serialize)]
struct StrictCeilingSeed<'a> {
    schema: u32,
    domain: &'static str,
    map_kappa: &'a str,
    decision_count: usize,
    strict_ceiling_hits: usize,
    abstentions: usize,
    ties_or_multiply_selected: usize,
    declared_class_reads: usize,
    performed_class_reads: usize,
    decisions: &'a [Gate0CeilingDecision],
}

/// Attaches only the sealed validation expectation after the label-free map
/// and query-key census exist.  This computes an upper bound; it does not run
/// the payload selector.
pub fn strict_post_label_ceiling<M: Gate0ClassLookup>(
    map: &M,
    rows: &[Gate0QueryCandidateRow],
    labels: &[Gate0ValidationLabel],
) -> Result<Gate0StrictCeilingReport, Gate0ClassMapError> {
    let grouped = canonical_query_groups(rows)?;
    let mut label_by_decision = BTreeMap::new();
    for label in labels {
        if label.decision_id.trim().is_empty()
            || label.expected_candidate_address_kappa.trim().is_empty()
            || label_by_decision
                .insert(
                    label.decision_id.clone(),
                    label.expected_candidate_address_kappa.clone(),
                )
                .is_some()
        {
            return Err(Gate0ClassMapError::Invalid(
                "validation labels require one non-empty expectation per decision".to_owned(),
            ));
        }
    }
    if label_by_decision.len() != grouped.len() || label_by_decision.keys().ne(grouped.keys()) {
        return Err(Gate0ClassMapError::Invalid(
            "validation labels do not exactly cover the label-free decisions".to_owned(),
        ));
    }

    let mut decisions = Vec::with_capacity(grouped.len());
    let mut strict_ceiling_hits = 0usize;
    let mut abstentions = 0usize;
    let mut ties_or_multiply_selected = 0usize;
    let mut declared_class_reads = 0usize;
    let mut performed_class_reads = 0usize;
    for (decision_id, candidates) in grouped {
        let expected = label_by_decision
            .get(&decision_id)
            .ok_or_else(|| Gate0ClassMapError::Invalid("missing validation label".to_owned()))?
            .clone();
        if !candidates
            .iter()
            .any(|candidate| candidate.candidate_address_kappa == expected)
        {
            return Err(Gate0ClassMapError::Invalid(format!(
                "expected candidate is outside natural support for {decision_id}"
            )));
        }

        let mut candidate_lookups = Vec::with_capacity(candidates.len());
        let mut selected = Vec::new();
        let mut rejected = 0usize;
        let mut abstained = false;
        for candidate in candidates {
            let lookup = map.lookup(&candidate.keys);
            declared_class_reads = checked_add(
                declared_class_reads,
                lookup.class_reads.declared_class_reads,
            )?;
            performed_class_reads = checked_add(
                performed_class_reads,
                lookup.class_reads.performed_class_reads,
            )?;
            match lookup.lookup {
                Gate0ActionLookup::Resolved {
                    action: ConstructionCausalReturnAction::Select,
                    ..
                } => selected.push(candidate.candidate_address_kappa.clone()),
                Gate0ActionLookup::Resolved {
                    action: ConstructionCausalReturnAction::Reject,
                    ..
                } => rejected = checked_add(rejected, 1)?,
                Gate0ActionLookup::Abstain { .. } => abstained = true,
            }
            candidate_lookups.push(Gate0CeilingCandidateLookup {
                candidate_address_kappa: candidate.candidate_address_kappa,
                lookup,
            });
        }
        let tied_or_multiply_selected = !abstained && (selected.len() != 1 || rejected != 1);
        let selected_candidate_address_kappa =
            (selected.len() == 1 && rejected == 1 && !abstained).then(|| selected[0].clone());
        let strict_ceiling_hit =
            selected_candidate_address_kappa.as_deref() == Some(expected.as_str());
        if strict_ceiling_hit {
            strict_ceiling_hits = checked_add(strict_ceiling_hits, 1)?;
        }
        if abstained {
            abstentions = checked_add(abstentions, 1)?;
        }
        if tied_or_multiply_selected {
            ties_or_multiply_selected = checked_add(ties_or_multiply_selected, 1)?;
        }
        decisions.push(Gate0CeilingDecision {
            decision_id,
            expected_candidate_address_kappa: expected,
            selected_candidate_address_kappa,
            strict_ceiling_hit,
            abstained,
            tied_or_multiply_selected,
            candidates: candidate_lookups,
        });
    }

    let mut report = Gate0StrictCeilingReport {
        schema: GATE0_SCHEMA,
        domain: CEILING_DOMAIN,
        map_kappa: map.map_kappa().to_owned(),
        decision_count: decisions.len(),
        strict_ceiling_hits,
        abstentions,
        ties_or_multiply_selected,
        declared_class_reads,
        performed_class_reads,
        decisions,
        ceiling_kappa: String::new(),
    };
    report.ceiling_kappa = record_kappa(&StrictCeilingSeed {
        schema: report.schema,
        domain: report.domain,
        map_kappa: &report.map_kappa,
        decision_count: report.decision_count,
        strict_ceiling_hits: report.strict_ceiling_hits,
        abstentions: report.abstentions,
        ties_or_multiply_selected: report.ties_or_multiply_selected,
        declared_class_reads: report.declared_class_reads,
        performed_class_reads: report.performed_class_reads,
        decisions: &report.decisions,
    })?;
    Ok(report)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Gate0PermutationKind {
    CyclicConstructionLabelPairing,
    CyclicCompiledKeyShuffle,
    IncoherentCandidateRepresentationSwap,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gate0ContentBinding {
    pub target_scope_id: String,
    pub target_candidate_address_kappa: String,
    pub source_scope_id: String,
    pub source_candidate_address_kappa: String,
    pub target_representation_kappa: String,
    pub source_representation_kappa: String,
    pub target_action_before: Option<ConstructionCausalReturnAction>,
    pub source_action: Option<ConstructionCausalReturnAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gate0ContentBoundPermutationReport {
    pub schema: u32,
    pub domain: &'static str,
    pub kind: Gate0PermutationKind,
    pub direction: &'static str,
    pub input_kappa: String,
    pub output_kappa: String,
    pub bindings: Vec<Gate0ContentBinding>,
    pub permutation_kappa: String,
}

#[derive(Debug, Serialize)]
struct PermutationSeed<'a> {
    schema: u32,
    domain: &'static str,
    kind: Gate0PermutationKind,
    direction: &'static str,
    input_kappa: &'a str,
    output_kappa: &'a str,
    bindings: &'a [Gate0ContentBinding],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gate0Transformation<T> {
    pub value: T,
    pub report: Gate0ContentBoundPermutationReport,
}

/// Cycles the construction current/label binding within each explicit cycle.
/// All transitions in a cycle must have the same candidate union.  Keys remain
/// attached to target content; only the construction action comes from the
/// same candidate in the next transition.
pub fn cyclic_construction_label_pairing(
    rows: &[Gate0ObservationRow],
    transition_cycles: &[Vec<String>],
) -> Result<Gate0Transformation<Vec<Gate0ObservationRow>>, Gate0ClassMapError> {
    let input = canonical_observation_rows(rows)?;
    let by_transition = observation_groups(&input)?;
    let all_transitions = by_transition.keys().cloned().collect::<BTreeSet<_>>();
    let mut assigned = BTreeSet::new();
    let mut source_for_target = BTreeMap::<String, String>::new();
    for cycle in transition_cycles {
        if cycle.len() < 2 {
            return Err(Gate0ClassMapError::Invalid(
                "construction label cycles require at least two transitions".to_owned(),
            ));
        }
        let mut canonical_cycle = cycle.clone();
        canonical_cycle.sort();
        canonical_cycle.dedup();
        if canonical_cycle.len() != cycle.len() {
            return Err(Gate0ClassMapError::Invalid(
                "construction label cycle repeats a transition".to_owned(),
            ));
        }
        let first_union =
            candidate_union(by_transition.get(&canonical_cycle[0]).ok_or_else(|| {
                Gate0ClassMapError::Invalid("unknown transition cycle".to_owned())
            })?);
        for transition in &canonical_cycle {
            if !assigned.insert(transition.clone())
                || candidate_union(by_transition.get(transition).ok_or_else(|| {
                    Gate0ClassMapError::Invalid("unknown transition cycle".to_owned())
                })?) != first_union
            {
                return Err(Gate0ClassMapError::Invalid(
                    "label cycles must partition transitions with equal candidate unions"
                        .to_owned(),
                ));
            }
        }
        for index in 0..canonical_cycle.len() {
            source_for_target.insert(
                canonical_cycle[index].clone(),
                canonical_cycle[(index + 1) % canonical_cycle.len()].clone(),
            );
        }
    }
    if assigned != all_transitions {
        return Err(Gate0ClassMapError::Invalid(
            "construction label cycles must cover every transition exactly once".to_owned(),
        ));
    }

    let mut output = Vec::with_capacity(input.len());
    let mut bindings = Vec::with_capacity(input.len());
    for target in &input {
        let source_transition = source_for_target
            .get(&target.transition_id)
            .ok_or_else(|| Gate0ClassMapError::Invalid("unassigned transition".to_owned()))?;
        let source = by_transition
            .get(source_transition)
            .and_then(|candidates| {
                candidates.iter().find(|candidate| {
                    candidate.candidate_address_kappa == target.candidate_address_kappa
                })
            })
            .ok_or_else(|| {
                Gate0ClassMapError::Invalid(
                    "cyclic label source lost a target candidate".to_owned(),
                )
            })?;
        let mut transformed = target.clone();
        transformed.action = source.action;
        bindings.push(Gate0ContentBinding {
            target_scope_id: target.transition_id.clone(),
            target_candidate_address_kappa: target.candidate_address_kappa.clone(),
            source_scope_id: source.transition_id.clone(),
            source_candidate_address_kappa: source.candidate_address_kappa.clone(),
            target_representation_kappa: record_kappa(&target.keys)?,
            source_representation_kappa: record_kappa(&source.keys)?,
            target_action_before: Some(target.action),
            source_action: Some(source.action),
        });
        output.push(transformed);
    }
    output.sort();
    let report = permutation_report(
        Gate0PermutationKind::CyclicConstructionLabelPairing,
        "target construction content receives the next transition's same-candidate action",
        &input,
        &output,
        bindings,
    )?;
    Ok(Gate0Transformation {
        value: output,
        report,
    })
}

/// Applies a one-step cycle to the complete compiled `(R_min, R_full)` key,
/// retaining construction actions and content identities.  The returned map is
/// recompiled only from the transformed construction rows.
pub fn cyclic_compiled_key_shuffle(
    map: &Gate0GeometricClassMap,
) -> Result<Gate0Transformation<Gate0GeometricClassMap>, Gate0ClassMapError> {
    let transformed = cyclic_key_rows(map.source_rows())?;
    let output_map = Gate0GeometricClassMap::compile(&transformed.value)?;
    let report = permutation_report(
        Gate0PermutationKind::CyclicCompiledKeyShuffle,
        "target construction row receives the next canonical compiled geometric key",
        map,
        &output_map,
        transformed.report.bindings,
    )?;
    Ok(Gate0Transformation {
        value: output_map,
        report,
    })
}

/// Exact-recall counterpart to `cyclic_compiled_key_shuffle`.
pub fn cyclic_exact_recall_key_shuffle(
    map: &Gate0ExactRecallMap,
) -> Result<Gate0Transformation<Gate0ExactRecallMap>, Gate0ClassMapError> {
    let transformed = cyclic_key_rows(map.source_rows())?;
    let output_map = Gate0ExactRecallMap::compile(&transformed.value)?;
    let report = permutation_report(
        Gate0PermutationKind::CyclicCompiledKeyShuffle,
        "target construction row receives the next canonical compiled exact-recall key",
        map,
        &output_map,
        transformed.report.bindings,
    )?;
    Ok(Gate0Transformation {
        value: output_map,
        report,
    })
}

/// Swaps the complete candidate-conditioned representation between the two
/// naturally admitted query candidates while keeping candidate identities and
/// support rows fixed.  No validation expectation is accepted.
pub fn incoherent_candidate_representation_swap(
    rows: &[Gate0QueryCandidateRow],
) -> Result<Gate0Transformation<Vec<Gate0QueryCandidateRow>>, Gate0ClassMapError> {
    let grouped = canonical_query_groups(rows)?;
    let input = grouped.values().flatten().cloned().collect::<Vec<_>>();
    let mut output = Vec::with_capacity(input.len());
    let mut bindings = Vec::with_capacity(input.len());
    for (decision_id, candidates) in grouped {
        let left = &candidates[0];
        let right = &candidates[1];
        for (target, source) in [(left, right), (right, left)] {
            output.push(Gate0QueryCandidateRow {
                decision_id: target.decision_id.clone(),
                candidate_address_kappa: target.candidate_address_kappa.clone(),
                keys: source.keys.clone(),
            });
            bindings.push(Gate0ContentBinding {
                target_scope_id: decision_id.clone(),
                target_candidate_address_kappa: target.candidate_address_kappa.clone(),
                source_scope_id: decision_id.clone(),
                source_candidate_address_kappa: source.candidate_address_kappa.clone(),
                target_representation_kappa: record_kappa(&target.keys)?,
                source_representation_kappa: record_kappa(&source.keys)?,
                target_action_before: None,
                source_action: None,
            });
        }
    }
    output.sort();
    let report = permutation_report(
        Gate0PermutationKind::IncoherentCandidateRepresentationSwap,
        "target natural candidate receives the other admitted candidate's representation",
        &input,
        &output,
        bindings,
    )?;
    Ok(Gate0Transformation {
        value: output,
        report,
    })
}

fn cyclic_key_rows(
    rows: &[Gate0ObservationRow],
) -> Result<Gate0Transformation<Vec<Gate0ObservationRow>>, Gate0ClassMapError> {
    let input = canonical_observation_rows(rows)?;
    let keys = input
        .iter()
        .map(|row| row.keys.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if keys.len() < 2 {
        return Err(Gate0ClassMapError::Invalid(
            "compiled-key shuffle requires at least two distinct keys".to_owned(),
        ));
    }
    let mut next_key = BTreeMap::new();
    for index in 0..keys.len() {
        next_key.insert(keys[index].clone(), keys[(index + 1) % keys.len()].clone());
    }
    let mut output = Vec::with_capacity(input.len());
    let mut bindings = Vec::with_capacity(input.len());
    for target in &input {
        let source_keys = next_key
            .get(&target.keys)
            .ok_or_else(|| Gate0ClassMapError::Invalid("missing cyclic key".to_owned()))?;
        let mut transformed = target.clone();
        transformed.keys = source_keys.clone();
        bindings.push(Gate0ContentBinding {
            target_scope_id: target.transition_id.clone(),
            target_candidate_address_kappa: target.candidate_address_kappa.clone(),
            source_scope_id: format!("compiled-key:{}", record_kappa(source_keys)?),
            source_candidate_address_kappa: String::new(),
            target_representation_kappa: record_kappa(&target.keys)?,
            source_representation_kappa: record_kappa(source_keys)?,
            target_action_before: Some(target.action),
            source_action: Some(target.action),
        });
        output.push(transformed);
    }
    output.sort();
    let report = permutation_report(
        Gate0PermutationKind::CyclicCompiledKeyShuffle,
        "target construction row receives the next canonical compiled key",
        &input,
        &output,
        bindings,
    )?;
    Ok(Gate0Transformation {
        value: output,
        report,
    })
}

fn canonical_observation_rows(
    rows: &[Gate0ObservationRow],
) -> Result<Vec<Gate0ObservationRow>, Gate0ClassMapError> {
    if rows.is_empty() {
        return Err(Gate0ClassMapError::Invalid(
            "construction map requires at least one row".to_owned(),
        ));
    }
    let mut canonical = rows.to_vec();
    canonical.sort();
    let mut identities = BTreeSet::new();
    for row in &canonical {
        if row.transition_id.trim().is_empty()
            || row.candidate_address_kappa.trim().is_empty()
            || !identities.insert((
                row.transition_id.clone(),
                row.candidate_address_kappa.clone(),
            ))
        {
            return Err(Gate0ClassMapError::Invalid(
                "construction rows require unique non-empty transition/candidate identities"
                    .to_owned(),
            ));
        }
    }
    Ok(canonical)
}

fn observation_groups(
    rows: &[Gate0ObservationRow],
) -> Result<BTreeMap<String, Vec<Gate0ObservationRow>>, Gate0ClassMapError> {
    let mut grouped = BTreeMap::<String, Vec<Gate0ObservationRow>>::new();
    for row in rows {
        grouped
            .entry(row.transition_id.clone())
            .or_default()
            .push(row.clone());
    }
    for candidates in grouped.values_mut() {
        candidates.sort_by(|left, right| {
            left.candidate_address_kappa
                .cmp(&right.candidate_address_kappa)
        });
        if candidates.len() != 2 {
            return Err(Gate0ClassMapError::Invalid(
                "each construction transition must have exactly two candidate rows".to_owned(),
            ));
        }
    }
    Ok(grouped)
}

fn candidate_union(rows: &[Gate0ObservationRow]) -> Vec<String> {
    rows.iter()
        .map(|row| row.candidate_address_kappa.clone())
        .collect()
}

fn canonical_query_groups(
    rows: &[Gate0QueryCandidateRow],
) -> Result<BTreeMap<String, Vec<Gate0QueryCandidateRow>>, Gate0ClassMapError> {
    if rows.is_empty() {
        return Err(Gate0ClassMapError::Invalid(
            "structural coverage requires at least one decision".to_owned(),
        ));
    }
    let mut grouped = BTreeMap::<String, Vec<Gate0QueryCandidateRow>>::new();
    let mut identities = BTreeSet::new();
    for row in rows {
        if row.decision_id.trim().is_empty()
            || row.candidate_address_kappa.trim().is_empty()
            || !identities.insert((row.decision_id.clone(), row.candidate_address_kappa.clone()))
        {
            return Err(Gate0ClassMapError::Invalid(
                "query rows require unique non-empty decision/candidate identities".to_owned(),
            ));
        }
        grouped
            .entry(row.decision_id.clone())
            .or_default()
            .push(row.clone());
    }
    for candidates in grouped.values_mut() {
        candidates.sort_by(|left, right| {
            left.candidate_address_kappa
                .cmp(&right.candidate_address_kappa)
        });
        if candidates.len() != 2 {
            return Err(Gate0ClassMapError::Invalid(
                "each Gate 0 decision must have exactly two natural candidates".to_owned(),
            ));
        }
    }
    Ok(grouped)
}

fn action_counts(
    actions: &[ConstructionCausalReturnAction],
) -> Result<(usize, usize), Gate0ClassMapError> {
    let mut select_rows = 0usize;
    let mut reject_rows = 0usize;
    for action in actions {
        match action {
            ConstructionCausalReturnAction::Select => {
                select_rows = checked_add(select_rows, 1)?;
            }
            ConstructionCausalReturnAction::Reject => {
                reject_rows = checked_add(reject_rows, 1)?;
            }
        }
    }
    Ok((select_rows, reject_rows))
}

fn checked_add(left: usize, right: usize) -> Result<usize, Gate0ClassMapError> {
    left.checked_add(right)
        .ok_or(Gate0ClassMapError::ArithmeticOverflow)
}

fn record_kappa<T: Serialize + ?Sized>(value: &T) -> Result<String, Gate0ClassMapError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| Gate0ClassMapError::Serialization(error.to_string()))?;
    Ok(format!("blake3:{}", blake3::hash(&bytes).to_hex()))
}

fn permutation_report<I: Serialize + ?Sized, O: Serialize + ?Sized>(
    kind: Gate0PermutationKind,
    direction: &'static str,
    input: &I,
    output: &O,
    mut bindings: Vec<Gate0ContentBinding>,
) -> Result<Gate0ContentBoundPermutationReport, Gate0ClassMapError> {
    bindings.sort_by(|left, right| {
        (
            left.target_scope_id.as_str(),
            left.target_candidate_address_kappa.as_str(),
            left.source_scope_id.as_str(),
            left.source_candidate_address_kappa.as_str(),
        )
            .cmp(&(
                right.target_scope_id.as_str(),
                right.target_candidate_address_kappa.as_str(),
                right.source_scope_id.as_str(),
                right.source_candidate_address_kappa.as_str(),
            ))
    });
    let input_kappa = record_kappa(input)?;
    let output_kappa = record_kappa(output)?;
    let mut report = Gate0ContentBoundPermutationReport {
        schema: GATE0_SCHEMA,
        domain: PERMUTATION_DOMAIN,
        kind,
        direction,
        input_kappa,
        output_kappa,
        bindings,
        permutation_kappa: String::new(),
    };
    report.permutation_kappa = record_kappa(&PermutationSeed {
        schema: report.schema,
        domain: report.domain,
        kind: report.kind,
        direction: report.direction,
        input_kappa: &report.input_kappa,
        output_kappa: &report.output_kappa,
        bindings: &report.bindings,
    })?;
    Ok(report)
}
