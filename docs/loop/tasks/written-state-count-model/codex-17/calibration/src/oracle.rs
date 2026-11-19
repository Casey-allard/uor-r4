//! Independent scalar checker for the #4106 synthetic fixtures. Imports the
//! shared signed orbit table, but no production candidate scorer or fitter.

use serde::Serialize;
use std::{cmp::Ordering, collections::BTreeMap};
use uor_r4_integer::discrete_d11::spiralcore_orbit_roots;

const MAX_RECORDS: usize = 256;
const MAX_SLOTS: usize = 10;
const NONWORD: u8 = 0;
const START: u8 = 1;
const CONT: u8 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidDescriptor,
    TooManyRecords,
    OpposedDescriptor,
    InvalidCandidateOrder,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct Shape {
    pub connector_slots: u8,
    pub word_slots: u8,
    pub classes: [u8; MAX_SLOTS],
}

impl Shape {
    pub fn used_slots(self) -> u8 {
        self.connector_slots + self.word_slots
    }

    fn valid(self) -> bool {
        if !(1..=3).contains(&self.connector_slots) || !(1..=7).contains(&self.word_slots) {
            return false;
        }
        let connector = self.connector_slots as usize;
        let end = self.used_slots() as usize;
        self.classes[..connector].iter().all(|&x| x <= CONT)
            && self.classes[connector] == START
            && self.classes[connector + 1..end].iter().all(|&x| x == CONT)
            && self.classes[end..].iter().all(|&x| x == NONWORD)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct Descriptor {
    pub shape: Shape,
    pub roots: [u16; MAX_SLOTS],
}

impl Descriptor {
    fn valid(self) -> bool {
        self.shape.valid()
            && self.roots[self.shape.used_slots() as usize..]
                .iter()
                .all(|&x| x == 0)
    }

    /// Decompose the fixed-size root payload without defining the future ABI.
    pub fn little_endian_roots(self) -> [u8; 2 * MAX_SLOTS] {
        let mut result = [0; 2 * MAX_SLOTS];
        for (pair, root) in result.chunks_exact_mut(2).zip(self.roots) {
            pair.copy_from_slice(&root.to_le_bytes());
        }
        result
    }

    fn canonical(mut self, control: Control) -> Self {
        if control == Control::Ordered {
            return self;
        }
        let connector = self.shape.connector_slots as usize;
        let end = self.shape.used_slots() as usize;
        for segment in [0..connector, connector..end] {
            for class in [START, CONT] {
                let positions: Vec<_> = segment
                    .clone()
                    .filter(|&i| self.shape.classes[i] == class)
                    .collect();
                let mut roots: Vec<_> = positions.iter().map(|&i| self.roots[i]).collect();
                roots.sort_unstable();
                for (i, root) in positions.into_iter().zip(roots) {
                    self.roots[i] = root;
                }
            }
        }
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Control {
    Ordered,
    WithinSegmentClassMultiset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct Example {
    pub descriptor: Descriptor,
    pub onset: bool,
    pub wording_family: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct Candidate {
    pub original_start: u32,
    pub descriptor: Descriptor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct Radius {
    pub shape: Shape,
    pub leave_family_out: Option<u16>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Bank {
    control: Control,
    records: Vec<Example>,
    radii: Vec<Radius>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct Observation {
    pub original_start: u32,
    pub used_slots: u8,
    pub onset_distance: Option<u16>,
    pub non_onset_distance: Option<u16>,
    pub radius: Option<u16>,
    pub margin_numerator: Option<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "source")]
pub enum Choice {
    NoCandidates,
    Unsupported,
    Nonpositive,
    Tie,
    Source(u32),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Comparison {
    pub observations: Vec<Observation>,
    pub ranked_sources: Vec<u32>,
    /// Numerator of first times denominator of second, then the reciprocal.
    pub top_cross_products: Option<[i64; 2]>,
    pub choice: Choice,
}

/// Existing D11 order is first block in the low nibble. This independent
/// expansion uses signed coordinate arrays and powers of 16.
fn signed_coordinates(mut encoded: u16) -> [i16; 32] {
    let table = spiralcore_orbit_roots();
    let mut result = [0; 32];
    for block in result.chunks_exact_mut(8) {
        let original_root = (encoded % 16) as usize;
        for (out, &coordinate) in block.iter_mut().zip(table[original_root].iter()) {
            *out = i16::from(coordinate);
        }
        encoded /= 16;
    }
    result
}

fn distance(a: Descriptor, b: Descriptor) -> u16 {
    let used = a.shape.used_slots() as usize;
    (0..used)
        .map(|i| {
            let lhs = signed_coordinates(a.roots[i]);
            let rhs = signed_coordinates(b.roots[i]);
            lhs.into_iter()
                .zip(rhs)
                .map(|(a, b)| (a - b).unsigned_abs())
                .sum::<u16>()
        })
        .sum()
}

/// Supervised finite freeze only. Retain the input order and all same-label
/// duplicate records. For each usable shape, omit each onset exemplar's
/// entire family and take the maximum remaining nearest-neighbor distance.
pub fn fit_synthetic(examples: &[Example], control: Control) -> Result<Bank, Error> {
    if examples.len() > MAX_RECORDS {
        return Err(Error::TooManyRecords);
    }
    let mut seen = BTreeMap::new();
    let mut by_shape: BTreeMap<Shape, Vec<Example>> = BTreeMap::new();
    let mut records = Vec::with_capacity(examples.len());
    for &ex in examples {
        if !ex.descriptor.valid() {
            return Err(Error::InvalidDescriptor);
        }
        let record = Example {
            descriptor: ex.descriptor.canonical(control),
            ..ex
        };
        if seen
            .insert(record.descriptor, record.onset)
            .is_some_and(|old| old != record.onset)
        {
            return Err(Error::OpposedDescriptor);
        }
        records.push(record);
        by_shape
            .entry(record.descriptor.shape)
            .or_default()
            .push(record);
    }
    let radii = by_shape
        .into_iter()
        .map(|(shape, records)| {
            let onset: Vec<_> = records.iter().filter(|r| r.onset).collect();
            let leave_family_out = if onset.is_empty() {
                None
            } else {
                let observations: Option<Vec<u16>> = onset
                    .iter()
                    .map(|anchor| {
                        onset
                            .iter()
                            .filter(|r| r.wording_family != anchor.wording_family)
                            .map(|r| distance(r.descriptor, anchor.descriptor))
                            .min()
                    })
                    .collect();
                observations.and_then(|radii| radii.into_iter().max())
            };
            Radius {
                shape,
                leave_family_out,
            }
        })
        .collect();
    Ok(Bank {
        control,
        records,
        radii,
    })
}

pub fn compare_candidates(bank: &Bank, candidates: &[Candidate]) -> Result<Comparison, Error> {
    if candidates
        .windows(2)
        .any(|p| p[0].original_start >= p[1].original_start)
    {
        return Err(Error::InvalidCandidateOrder);
    }
    let mut observations = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        if !candidate.descriptor.valid() {
            return Err(Error::InvalidDescriptor);
        }
        let d = candidate.descriptor.canonical(bank.control);
        let class_minimum = |onset| {
            bank.records
                .iter()
                .filter(|ex| ex.onset == onset && ex.descriptor.shape == d.shape)
                .map(|ex| distance(ex.descriptor, d))
                .min()
        };
        let onset_distance = class_minimum(true);
        let non_onset_distance = class_minimum(false);
        let radius = bank
            .radii
            .iter()
            .find(|r| r.shape == d.shape)
            .and_then(|r| r.leave_family_out);
        let margin_numerator = match (onset_distance, non_onset_distance, radius) {
            (Some(onset), Some(other), Some(limit)) if onset <= limit => {
                Some(i32::from(other) - i32::from(onset))
            }
            _ => None,
        };
        observations.push(Observation {
            original_start: candidate.original_start,
            used_slots: d.shape.used_slots(),
            onset_distance,
            non_onset_distance,
            radius,
            margin_numerator,
        });
    }
    let mut ranked_sources = Vec::new();
    let mut top_cross_products = None;
    let choice = if observations.is_empty() {
        Choice::NoCandidates
    } else {
        let margins: Option<Vec<_>> = observations
            .iter()
            .map(|row| {
                row.margin_numerator
                    .map(|margin| (margin, row.used_slots, row.original_start))
            })
            .collect();
        if let Some(mut margins) = margins {
            let order = |&(p, n, _): &(i32, u8, u32), &(q, m, _): &(i32, u8, u32)| {
                (i64::from(p) * i64::from(m)).cmp(&(i64::from(q) * i64::from(n)))
            };
            margins.sort_by(|a, b| order(b, a));
            let best = margins[0];
            ranked_sources.extend(margins.iter().map(|r| r.2));
            top_cross_products = margins.get(1).map(|second| {
                [
                    i64::from(best.0) * i64::from(second.1),
                    i64::from(second.0) * i64::from(best.1),
                ]
            });
            if best.0 <= 0 {
                Choice::Nonpositive
            } else if margins
                .get(1)
                .is_some_and(|other| order(&best, other) == Ordering::Equal)
            {
                Choice::Tie
            } else {
                Choice::Source(best.2)
            }
        } else {
            Choice::Unsupported
        }
    };
    Ok(Comparison {
        observations,
        ranked_sources,
        top_cross_products,
        choice,
    })
}
