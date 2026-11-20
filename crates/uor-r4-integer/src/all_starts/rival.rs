//! Independently fitted negative radii, keyed by the unchanged ordered bank.
use super::{word, Bank, OnsetMode, CAP, NO_RADIUS};
use crate::{invalid, Result};
use std::collections::BTreeMap;

const MAGIC: u32 = 0x39324e52;
const HEADER: usize = 4;
pub const NEGATIVE_CALIBRATION_BYTES: usize = (HEADER + CAP) * 4;

#[derive(Clone, Debug)]
pub struct NegativeCalibration {
    radii: Box<[u32; CAP]>,
}

impl NegativeCalibration {
    pub(super) fn radius(&self, index: usize) -> u32 {
        self.radii[index]
    }
}

impl Bank {
    /// Attach a signed radius table without changing any exemplar or positive radius.
    pub fn with_negative_calibration(mut self, bytes: &[u8]) -> Result<Self> {
        if self.mode != OnsetMode::OrderedWholeWord
            || self.negative_calibration.is_some()
            || bytes.len() != NEGATIVE_CALIBRATION_BYTES
            || word(bytes, 0) != MAGIC
            || word(bytes, 1) != self.words[2]
            || word(bytes, 2) != CAP as u32
            || word(bytes, 3) != 0
        {
            return Err(invalid("invalid negative calibration envelope"));
        }
        let mut radii = Box::new([0; CAP]);
        let mut shapes = BTreeMap::<(u32, u32), (u32, usize)>::new();
        for (index, record) in self.records().enumerate() {
            let radius = word(bytes, HEADER + index);
            if radius > 1280 && radius != NO_RADIUS {
                return Err(invalid("invalid negative calibration radius"));
            }
            let key = (record[0] & 65535, record[1]);
            let (expected, negatives) = shapes.entry(key).or_insert((radius, 0));
            if *expected != radius {
                return Err(invalid("conflicting negative radii for one signature"));
            }
            *negatives += (record[0] >> 16) as usize;
            radii[index] = radius;
        }
        if shapes
            .values()
            .any(|&(radius, negatives)| radius != NO_RADIUS && negatives < 2)
            || bytes[(HEADER + self.words[2] as usize) * 4..]
                .iter()
                .any(|&byte| byte != 0)
        {
            return Err(invalid(
                "negative calibration lacks support or has nonzero padding",
            ));
        }
        self.negative_calibration = Some(NegativeCalibration { radii });
        Ok(self)
    }

    pub fn has_negative_calibration(&self) -> bool {
        self.negative_calibration.is_some()
    }

    /// Rotate only the finite radii over sorted native shape/signature keys.
    /// Returns the control bytes and the number of signatures actually changed.
    pub fn shuffle_negative_calibration(&self) -> Result<(Vec<u8>, usize)> {
        let table = self
            .negative_calibration
            .as_ref()
            .ok_or_else(|| invalid("missing fitted negative calibration"))?;
        let mut supported = BTreeMap::new();
        for (index, record) in self.records().enumerate() {
            let radius = table.radius(index);
            if radius != NO_RADIUS {
                supported.insert((record[0] & 65535, record[1]), radius);
            }
        }
        let radii: Vec<_> = supported.values().copied().collect();
        let mut changed = 0;
        if !radii.is_empty() {
            for (radius, rotated) in supported.values_mut().zip(radii.iter().cycle().skip(1)) {
                changed += usize::from(*radius != *rotated);
                *radius = *rotated;
            }
        }
        let mut words = [0; HEADER + CAP];
        words[..HEADER].copy_from_slice(&[MAGIC, self.words[2], CAP as u32, 0]);
        for (index, record) in self.records().enumerate() {
            words[HEADER + index] = supported
                .get(&(record[0] & 65535, record[1]))
                .copied()
                .unwrap_or(NO_RADIUS);
        }
        Ok((
            words.into_iter().flat_map(u32::to_le_bytes).collect(),
            changed,
        ))
    }
}
