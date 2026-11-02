//! Exact CPU execution of the integer artifact's turn memory and point key.

use uor_r4_integer::stack::{
    MemoryImage, MemoryTableImage, PointKeyImage, MEMORY_PAD, MEMORY_WEIGHT_EXP,
};

const BOS: u32 = 0;
const EOS: u32 = 1;
const NORM_FINE_EXP: i32 = -24;
const WRITER_EXP: i32 = -12;
const MEMORY_WEIGHT_DROP: u32 = (-MEMORY_WEIGHT_EXP) as u32;
const MEMORY_POINT_RAISE: i32 = WRITER_EXP - NORM_FINE_EXP;

#[derive(Clone)]
struct Table {
    keys: Vec<Vec<u32>>,
    values: Vec<u32>,
    weight: Vec<i32>,
}

impl Table {
    fn new(image: &MemoryTableImage, width: usize) -> Result<Self, String> {
        if image.weight.len() != width {
            return Err("turn-memory weight width does not match the trunk".to_owned());
        }
        Ok(Self {
            keys: image.keys.clone(),
            values: image.values.clone(),
            weight: image.weight.clone(),
        })
    }

    fn stored(&self, key: &[u32]) -> Option<u32> {
        self.keys
            .binary_search_by(|probe| probe.as_slice().cmp(key))
            .ok()
            .map(|index| self.values[index])
    }
}

#[derive(Clone)]
struct PointKey {
    width: usize,
    points: Vec<i16>,
    targets: Vec<u32>,
}

impl PointKey {
    fn new(image: PointKeyImage) -> Result<Self, String> {
        if image.width == 0 || image.points.len() != image.targets.len().saturating_mul(image.width)
        {
            return Err("point-key table dimensions are inconsistent".to_owned());
        }
        Ok(Self {
            width: image.width,
            points: image.points,
            targets: image.targets,
        })
    }

    fn name(&self, query: &[i32]) -> Result<(u32, i64), String> {
        if query.len() != self.width {
            return Err("point-key query width does not match its table".to_owned());
        }
        let mut least = i64::MAX;
        let mut tied = Vec::new();
        for (row, &target) in self.points.chunks_exact(self.width).zip(&self.targets) {
            let distance = query.iter().zip(row).fold(0i64, |sum, (&value, &point)| {
                sum.wrapping_add(i64::from(value.abs_diff(i32::from(point))))
            });
            if distance < least {
                least = distance;
                tied.clear();
            }
            if distance == least {
                tied.push(target);
            }
        }
        tied.sort_unstable();
        let mut named = u32::MAX;
        let mut count = 0usize;
        let mut at = 0usize;
        while at < tied.len() {
            let target = tied[at];
            let run = tied[at..]
                .iter()
                .take_while(|&&candidate| candidate == target)
                .count();
            if run > count {
                named = target;
                count = run;
            }
            at += run;
        }
        if count == 0 || named == BOS {
            return Err("the point key named no token, or BOS".to_owned());
        }
        Ok((named, least))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MemoryAnchors {
    pub(crate) opening: Vec<u32>,
    pub(crate) reply: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MemoryMove {
    pub(crate) stored: Option<u32>,
    pub(crate) opening: bool,
}

#[derive(Clone)]
pub(crate) struct CpuTurnMemory {
    k: usize,
    key_window: usize,
    reply_turn: Option<usize>,
    opening: Table,
    reply: Option<Table>,
    point: Option<PointKey>,
}

impl CpuTurnMemory {
    pub(crate) fn new(image: MemoryImage, width: usize) -> Result<Self, String> {
        if image.record.key_window == 0 {
            return Err("turn-memory key window is zero".to_owned());
        }
        Ok(Self {
            k: image.record.k,
            key_window: image.record.key_window,
            reply_turn: image.record.reply_turn,
            opening: Table::new(&image.opening, width)?,
            reply: image
                .reply
                .as_ref()
                .map(|table| Table::new(table, width))
                .transpose()?,
            point: image.point.map(PointKey::new).transpose()?,
        })
    }

    pub(crate) fn apply(
        &self,
        tokens: &[u32],
        row: usize,
        reply_start: usize,
        anchors: &mut MemoryAnchors,
        writer: &[i16],
        norm: &mut [i64],
    ) -> Option<MemoryMove> {
        if row + 1 == reply_start {
            anchors.opening = self.turn_ids(tokens, row, self.k);
            anchors.reply = self
                .reply_turn
                .map(|turn| self.turn_ids(tokens, row, turn))
                .unwrap_or_default();
            let stored = self.opening.stored(&anchors.opening);
            if let Some(id) = stored {
                move_point(&self.opening, id, writer, norm);
            }
            return Some(MemoryMove {
                stored,
                opening: true,
            });
        }
        if row < reply_start {
            return None;
        }
        let reply = self.reply.as_ref()?;
        let key = self.reply_key(tokens, row, reply_start, &anchors.reply);
        let stored = reply.stored(&key);
        if let Some(id) = stored {
            move_point(reply, id, writer, norm);
        }
        Some(MemoryMove {
            stored,
            opening: false,
        })
    }

    pub(crate) fn point_name(&self, query: &[i32]) -> Option<Result<(u32, i64), String>> {
        self.point.as_ref().map(|point| point.name(query))
    }

    pub(crate) fn anchor_limits(&self) -> (usize, usize) {
        (self.k, self.reply_turn.unwrap_or(0))
    }

    pub(crate) fn anchors_at(&self, tokens: &[u32], reply_start: usize) -> MemoryAnchors {
        let row = reply_start.saturating_sub(1);
        MemoryAnchors {
            opening: self.turn_ids(tokens, row, self.k),
            reply: self
                .reply_turn
                .map(|turn| self.turn_ids(tokens, row, turn))
                .unwrap_or_default(),
        }
    }

    fn block_start(&self, row: usize) -> usize {
        row / self.key_window * self.key_window
    }

    fn turn_ids(&self, tokens: &[u32], at: usize, limit: usize) -> Vec<u32> {
        let block_start = self.block_start(at);
        let mut start = block_start;
        for position in (block_start..=at).rev() {
            let token = tokens[position];
            if token == BOS || token == EOS {
                start = position + 1;
                break;
            }
        }
        if start > at {
            return Vec::new();
        }
        tokens[start..=at].iter().copied().take(limit).collect()
    }

    fn reply_key(&self, tokens: &[u32], row: usize, reply_start: usize, turn: &[u32]) -> Vec<u32> {
        let block_start = self.block_start(row);
        let prompt_last = reply_start.saturating_sub(1);
        let mut key = Vec::with_capacity(turn.len() + 4);
        if reply_start > 0 && prompt_last >= block_start {
            key.extend_from_slice(turn);
        }
        let visible = block_start.max(reply_start);
        let length = row + 1 - visible;
        key.extend([MEMORY_PAD, length as u32, MEMORY_PAD, tokens[row]]);
        key
    }
}

fn move_point(table: &Table, stored: u32, writer: &[i16], norm: &mut [i64]) {
    let at = stored as usize * norm.len();
    let Some(row) = writer.get(at..at + norm.len()) else {
        return;
    };
    for ((slot, &target), &weight) in norm.iter_mut().zip(row).zip(&table.weight) {
        let point = i64::from(target) << MEMORY_POINT_RAISE;
        let sum = (i128::from(*slot) << MEMORY_WEIGHT_DROP)
            + i128::from(weight) * i128::from(point - *slot);
        let moved = round_shift(sum, MEMORY_WEIGHT_DROP);
        *slot = moved.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64;
    }
}

fn round_shift(value: i128, shift: u32) -> i128 {
    let half = 1i128 << (shift - 1);
    if value >= 0 {
        (value + half) >> shift
    } else {
        -((half - value) >> shift)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_moves_round_half_away_from_zero() {
        let positive = Table {
            keys: Vec::new(),
            values: Vec::new(),
            weight: vec![1 << 11],
        };
        let negative = Table {
            weight: vec![-(1 << 11); 1],
            ..positive.clone()
        };
        let mut norm = [0i64];
        move_point(&positive, 0, &[1], &mut norm);
        assert_eq!(norm, [1]);
        norm[0] = 0;
        move_point(&negative, 0, &[1], &mut norm);
        assert_eq!(norm, [-1]);
    }

    #[test]
    fn malformed_memory_and_point_dimensions_are_refused() {
        assert!(Table::new(
            &MemoryTableImage {
                keys: Vec::new(),
                values: Vec::new(),
                weight: Vec::new(),
            },
            1,
        )
        .is_err());
        assert!(PointKey::new(PointKeyImage {
            width: 2,
            points: vec![0],
            targets: vec![1],
        })
        .is_err());
    }
}
