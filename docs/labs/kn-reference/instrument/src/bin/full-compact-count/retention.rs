//! Exact occupied-byte selection for whole-context UORKNC02 retention.

use std::collections::BTreeMap;

pub const HEADER: u64 = 44;
pub const ORDER_HEADER: u64 = 72;
pub const ALWAYS_KEEP_ORDER: u32 = 3;

type Fail = Box<dyn std::error::Error>;

pub fn key_width(order: u32) -> usize {
    (order as usize * 12).div_ceil(8)
}

pub fn follower_width(order: u32, highest: u32) -> u64 {
    if order == highest {
        6
    } else {
        10
    }
}

pub fn keep(order: u32, support: u64, threshold: u64) -> bool {
    order <= ALWAYS_KEEP_ORDER || support >= threshold
}

/// Record an order's retained raw and adjusted domains without recounting
/// adjusted predecessor multiplicities from the now-pruned higher orders.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub contexts: u64,
    pub followers: u64,
    pub totals: [u64; 2],
    pub spectra: [[u64; 4]; 2],
}

impl Stats {
    pub fn add(&mut self, group: Self) {
        self.contexts += group.contexts;
        self.followers += group.followers;
        for d in 0..2 {
            self.totals[d] += group.totals[d];
            for slot in 0..4 {
                self.spectra[d][slot] += group.spectra[d][slot];
            }
        }
    }

    pub fn observe(&mut self, raw: u32, adjusted: u32) {
        self.followers += 1;
        for (d, count) in [raw, adjusted].into_iter().enumerate() {
            self.totals[d] += u64::from(count);
            if (1..=4).contains(&count) {
                self.spectra[d][count as usize - 1] += 1;
            }
        }
    }

    pub fn bytes(self, order: u32, highest: u32) -> u64 {
        self.contexts * (key_width(order) as u64 + 4)
            + self.followers * follower_width(order, highest)
    }
}

pub struct Budget {
    highest: u32,
    floor: u64,
    /// Each occupied integer support contributes one bin, never one entry per
    /// context. Distinct supports require at least 1+...+bins raw occurrences.
    bytes_by_support: BTreeMap<u64, u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selection {
    pub threshold: u64,
    pub bytes: u64,
    pub floor: u64,
}

impl Budget {
    pub fn new(highest: u32) -> Self {
        Self {
            highest,
            floor: HEADER + ORDER_HEADER * u64::from(highest + 1),
            bytes_by_support: BTreeMap::new(),
        }
    }

    pub fn observe(&mut self, order: u32, support: u64, followers: u64) -> Result<(), Fail> {
        if order > self.highest || support == 0 || followers == 0 {
            return Err("invalid whole-context budget group".into());
        }
        let bytes = Stats {
            contexts: 1,
            followers,
            ..Stats::default()
        }
        .bytes(order, self.highest);
        let bin = if order <= ALWAYS_KEEP_ORDER {
            &mut self.floor
        } else {
            self.bytes_by_support.entry(support).or_default()
        };
        *bin = bin.checked_add(bytes).ok_or("compact budget overflow")?;
        Ok(())
    }

    pub fn select(&self, cap: u64) -> Result<Selection, Fail> {
        if cap < self.floor {
            return Err(
                format!("compact cap {cap} is below mandatory floor {}", self.floor).into(),
            );
        }
        let mut bytes = self.floor;
        for (&support, &cost) in self.bytes_by_support.iter().rev() {
            let next = bytes.checked_add(cost).ok_or("compact budget overflow")?;
            if next > cap {
                return Ok(Selection {
                    threshold: support.checked_add(1).ok_or("threshold overflow")?,
                    bytes,
                    floor: self.floor,
                });
            }
            bytes = next;
        }
        Ok(Selection {
            threshold: 1,
            bytes,
            floor: self.floor,
        })
    }
}
