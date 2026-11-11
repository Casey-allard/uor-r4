//! Shared types for standalone one-model instruments.

pub mod reply_control_controller;

use serde::Deserialize;

pub const EOS: u32 = 1;
pub const REPLY_CAP: usize = 48;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct TraceRow {
    pub case: usize,
    pub reply_position: usize,
    pub token: u32,
    pub nearest: NearestTrace,
    pub eos: EosTrace,
    pub memory: MemoryTrace,
    pub reads: ReadTrace,
    pub short_cycle_period: Option<usize>,
    pub class: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct NearestTrace {
    pub margin: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct EosTrace {
    pub rank: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct MemoryTrace {
    pub movement: Option<MemoryMoveTrace>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct MemoryMoveTrace {
    pub stored: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct ReadTrace {
    pub copy_hits: usize,
    pub type_hits: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Keep,
    EmitEos,
}

pub fn short_cycle(tokens: &[u32]) -> Option<usize> {
    for period in 1..=4 {
        let span = period * 3;
        if tokens.len() >= span {
            let tail = &tokens[tokens.len() - span..];
            if tail[..period] == tail[period..period * 2]
                && tail[..period] == tail[period * 2..]
            {
                return Some(period);
            }
        }
    }
    None
}

pub fn apply_flow(
    rows: &[TraceRow],
    current_cycle_rule: bool,
    mut decide: impl FnMut(&TraceRow, &[u32]) -> Decision,
) -> Vec<u32> {
    let mut ids = Vec::new();
    for row in rows {
        if ids.len() == REPLY_CAP {
            break;
        }
        match decide(row, &ids) {
            Decision::EmitEos => {
                ids.push(EOS);
                break;
            }
            Decision::Keep => ids.push(row.token),
        }
        if row.token == EOS {
            break;
        }
        if current_cycle_rule
            && row.short_cycle_period.is_some()
            && short_cycle(&ids).is_some()
        {
            break;
        }
    }
    ids
}

