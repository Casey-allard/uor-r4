//! Integer reference serving from the same sealed bank that CUDA fitted.
use crate::packet::{open, validate_raw, Result, Row, Rows, NIL, NO_RADIUS, OUT};
use uor_r4_integer::discrete_d11::spiralcore_orbit_roots;

pub struct Bank {
    words: Vec<u32>,
}
impl Bank {
    pub fn load(rows: &Rows, sealed: &[u8]) -> Result<Self> {
        Ok(Self {
            words: validate_raw(open(rows, sealed)?)?,
        })
    }
    pub fn mode(&self) -> u32 {
        self.words[1]
    }
    fn records(&self) -> impl Iterator<Item = &[u32]> {
        self.words[4..].chunks_exact(8).take(self.words[2] as usize)
    }

    pub fn select(&self, row: &Row) -> Result<[u32; OUT]> {
        row.validate()?;
        let mut trace = [NIL; OUT];
        trace[..2].copy_from_slice(&row.0[..2]);
        let mut margins = [Margin::ABSENT; 3];
        let mut unseen = false;
        for (i, margin) in margins.iter_mut().enumerate() {
            let c = row.candidate(i);
            if c[0] == NIL {
                continue;
            }
            let out = &mut trace[8 + 20 * i..28 + 20 * i];
            out[..3].copy_from_slice(&[c[0], c[1], 1]);
            if c[2] == 0 {
                continue;
            }
            out[2] = 2;
            let depth = i + 1;
            let word_len = if self.mode() == 2 {
                row.0[5] - c[0]
            } else {
                c[1] - c[0]
            };
            let slots = depth + word_len as usize;
            let shape = depth as u32 | (word_len << 8);
            let sig = if self.mode() == 2 { 0 } else { c[3] };
            let mut roots = [0u16; 10];
            if self.mode() != 2 {
                for (slot, to) in roots[..slots].iter_mut().enumerate() {
                    *to = root_at(&c[4..], slot);
                }
                if self.mode() == 1 {
                    canonical(&mut roots[..slots], depth, sig);
                }
            }
            out[3..5].copy_from_slice(&[shape, sig]);
            for (to, &value) in out[10..10 + slots].iter_mut().zip(&roots) {
                *to = u32::from(value);
            }
            let hits = self
                .records()
                .filter(|record| record[0] & 65535 == shape && record[1] == sig);
            let score = if self.mode() == 2 {
                let mut votes = [0u32; 2];
                for record in hits {
                    votes[(record[0] >> 16) as usize] += 1;
                }
                out[5..8].copy_from_slice(&[votes[0], votes[1], NO_RADIUS]);
                (votes != [0, 0]).then_some((votes[0] as i32 - votes[1] as i32, 1))
            } else {
                let mut closest = [NIL; 2];
                let mut radius = NO_RADIUS;
                for record in hits {
                    let d = roots[..slots]
                        .iter()
                        .enumerate()
                        .map(|(j, &r)| l1(r, root_at(&record[3..], j)))
                        .sum();
                    let index = (record[0] >> 16) as usize;
                    closest[index] = closest[index].min(d);
                    radius = record[2];
                }
                out[5..8].copy_from_slice(&[closest[0], closest[1], radius]);
                if closest.contains(&NIL) || radius == NO_RADIUS || closest[0] > radius {
                    None
                } else {
                    Some((closest[1] as i32 - closest[0] as i32, slots as i32))
                }
            };
            if let Some((num, den)) = score {
                out[2] = 3;
                out[8..10].copy_from_slice(&[num as u32, den as u32]);
                *margin = Margin {
                    num,
                    den,
                    source: c[0],
                };
            } else {
                unseen = true;
            }
        }
        margins.sort_by(|a, b| b.compare(*a));
        let [best, next, _] = margins;
        for (score, dest) in [best, next]
            .into_iter()
            .zip(trace[4..8].chunks_exact_mut(2))
        {
            if score.den != 0 {
                dest.copy_from_slice(&[score.num as u32, score.den as u32]);
            }
        }
        trace[2] = if unseen {
            2
        } else if best.den == 0 {
            1
        } else if best.num <= 0 {
            4
        } else if next.den != 0 && best.compare(next).is_eq() {
            3
        } else {
            0
        };
        if trace[2] == 0 {
            trace[3] = best.source;
        }
        Ok(trace)
    }
}

#[derive(Clone, Copy)]
struct Margin {
    num: i32,
    den: i32,
    source: u32,
}
impl Margin {
    const ABSENT: Self = Self {
        num: 0,
        den: 0,
        source: NIL,
    };
    fn compare(self, other: Self) -> std::cmp::Ordering {
        match (self.den == 0, other.den == 0) {
            (false, false) => (self.num * other.den).cmp(&(other.num * self.den)),
            (a, b) => b.cmp(&a),
        }
    }
}

fn root_at(data: &[u32], slot: usize) -> u16 {
    (data[slot / 2] >> (16 * (slot % 2))) as u16
}

fn l1(a: u16, b: u16) -> u32 {
    let orbit = spiralcore_orbit_roots();
    (0..4)
        .map(|i| {
            let x = &orbit[((a >> (4 * i)) & 15) as usize];
            let y = &orbit[((b >> (4 * i)) & 15) as usize];
            x.iter()
                .zip(y)
                .map(|(&x, &y)| u32::from(x.abs_diff(y)))
                .sum::<u32>()
        })
        .sum()
}

pub(crate) fn canonical(geometry: &mut [u16], depth: usize, signature: u32) {
    for range in [0..depth, depth..geometry.len()] {
        for class in [1, 2] {
            let mut values = [0u16; 10];
            let mut count = 0;
            let positions = range
                .clone()
                .filter(|&i| (signature >> (2 * i)) & 3 == class);
            for i in positions.clone() {
                values[count] = geometry[i];
                count += 1;
            }
            values[..count].sort_unstable();
            for (i, word) in positions.zip(values) {
                geometry[i] = word;
            }
        }
    }
}
