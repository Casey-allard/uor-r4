//! Explicit-address bridge from retained scoped memory to the shared native lexical decoder.
//!
//! Address/operation intent is supplied by the caller. This module does not parse a user request
//! or learn memory admission; the selected owned occurrence reaches the *loaded* TlModel's
//! Generate/Copy/Stop path without a host-authored answer.
#![forbid(unsafe_code)]

use std::fmt;

use super::scoped_memory::{encode_key, HistoryView, Lookup, Memory, Record};
use super::state_lexical::SlFacts;
use super::transferable_lexical::{TlModel, TlRollout};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadStatus {
    Found,
    Disabled,
    Absent,
    NoHistory,
    Evicted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BridgeError {
    Request(&'static str),
    Model(String),
    Memory(String),
}

impl fmt::Display for BridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Request(message) => write!(f, "invalid addressed lexical request: {message}"),
            Self::Model(message) => write!(f, "invalid lexical model: {message}"),
            Self::Memory(message) => write!(f, "invalid scoped memory: {message}"),
        }
    }
}

impl std::error::Error for BridgeError {}

/// Typed request. The caller owns parsing, scope authorization and source admission.
#[derive(Clone, Copy)]
pub struct AddressedRequest<'a> {
    pub scope: &'a [u8],
    pub entity: &'a [u8],
    pub relation: u8,
    pub view: u64,
    pub history: HistoryView,
    pub observed: &'a [u32],
    pub max_new: usize,
    pub read_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddressedGeneration {
    pub status: ReadStatus,
    pub record_id: Option<u64>,
    pub source_id: Option<u64>,
    pub selected_commit: Option<u64>,
    pub selected_payload: Option<Vec<u32>>,
    pub chain_len: usize,
    /// None is a typed grounded-answer abstention. Ordinary prose is a separate request.
    pub rollout: Option<TlRollout>,
}

fn history_code(history: HistoryView) -> u8 {
    match history {
        HistoryView::Current => 0,
        HistoryView::PreviousAssertion => 1,
        HistoryView::PreviousDistinctValue => 2,
        HistoryView::Initial => 3,
    }
}

fn facts(memory: &Memory, record: &Record, history: HistoryView) -> SlFacts {
    let predecessor = memory.record_ref(record.predecessor);
    SlFacts {
        history: history_code(history),
        derived: false,
        prior_differs: predecessor.is_some_and(|p| p.value != record.value),
        committed: predecessor.is_some(),
        key_changed: false,
    }
}

/// Select an exact owned record and execute the unchanged native lexical policy. The disabled
/// arm never calls lookup or makes Copy legal. Missing, evicted or disabled evidence returns a
/// typed status with no grounded generation; ordinary prose is a separate request.
pub fn generate(
    model: &TlModel,
    memory: &Memory,
    request: AddressedRequest<'_>,
) -> Result<AddressedGeneration, BridgeError> {
    if request.scope.is_empty() || request.entity.is_empty() {
        return Err(BridgeError::Request("scope and entity must be nonempty"));
    }
    if !(1..=64).contains(&request.max_new) {
        return Err(BridgeError::Request("max_new must be 1..=64"));
    }
    if request.view > memory.commit {
        return Err(BridgeError::Request("view exceeds committed history"));
    }
    model.validate().map_err(BridgeError::Model)?;
    memory
        .validate()
        .map_err(|error| BridgeError::Memory(error.to_string()))?;
    let key = encode_key(request.scope, request.entity, request.relation);
    let chain_len = if request.read_enabled {
        memory.chain(&key).len()
    } else {
        0
    };
    let selected = if request.read_enabled {
        match memory.lookup(&key, request.view, request.history) {
            Lookup::Found(record) => (ReadStatus::Found, Some(record)),
            Lookup::Absent => (ReadStatus::Absent, None),
            Lookup::NoHistory => (ReadStatus::NoHistory, None),
            Lookup::Evicted => (ReadStatus::Evicted, None),
        }
    } else {
        (ReadStatus::Disabled, None)
    };
    let (status, record) = selected;
    let sel = record.map_or(&[][..], |record| record.payload.as_slice());
    let rollout = record.map(|record| {
        model.rollout(
            sel,
            &[],
            facts(memory, record, request.history),
            request.observed,
            sel,
            request.max_new,
            false,
            false,
        )
    });
    Ok(AddressedGeneration {
        status,
        record_id: record.map(|r| r.id),
        source_id: record.map(|r| r.source),
        selected_commit: record.map(|r| r.commit),
        selected_payload: record.map(|r| r.payload.clone()),
        chain_len,
        rollout,
    })
}

#[cfg(test)]
mod tests {
    use super::super::scoped_memory::Update;
    use super::super::transferable_lexical::{TlConfig, TlTrainConfig, TlTrainer};
    use super::*;

    fn model() -> TlModel {
        let mut cfg = TlConfig::new(16);
        cfg.h_dim = 8;
        TlTrainer::new(cfg, TlTrainConfig::default())
            .unwrap()
            .model()
            .unwrap()
    }

    #[test]
    fn exact_source_selection_and_disabled_control() {
        let mut memory = Memory::new(17, 3);
        let model = model();
        memory
            .write(b"s", b"a", 1, b"red", &[3], 11, Update::Assert, false, 0)
            .unwrap();
        let view = memory.commit;
        memory
            .write(b"s", b"a", 1, b"green", &[4], 12, Update::Correct, false, 0)
            .unwrap();
        let request = |view, read_enabled| AddressedRequest {
            scope: b"s",
            entity: b"a",
            relation: 1,
            view,
            history: HistoryView::Current,
            observed: &[],
            max_new: 3,
            read_enabled,
        };
        let prior = generate(&model, &memory, request(view, true)).unwrap();
        let current = generate(&model, &memory, request(memory.commit, true)).unwrap();
        let disabled = generate(&model, &memory, request(memory.commit, false)).unwrap();
        assert_eq!(prior.selected_payload, Some(vec![3]));
        assert_eq!(current.selected_payload, Some(vec![4]));
        assert_eq!(current.source_id, Some(12));
        assert_eq!(disabled.status, ReadStatus::Disabled);
        assert_eq!(disabled.selected_payload, None);
        assert!(disabled.rollout.is_none());
    }
}
