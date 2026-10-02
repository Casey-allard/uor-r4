//! Authored grammar copied without semantic changes from the retained context
//! study. Private example helpers are kept local instead of widening library APIs.
//! Parent helper source SHA256: 758ef92bfaf2c1090e39b04ac7ee97f2424463959bb89fa57cc3ee70da97de7c.
use serde::{Deserialize, Serialize};
pub(super) struct Rng(pub(super) u64);
impl Rng {
    pub(super) fn next(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct Episode {
    pub(super) ids: Vec<u32>,
    pub(super) roles: Vec<String>,
    pub(super) source: usize,
    pub(super) query: usize,
    pub(super) answer: u32,
    pub(super) facts: usize,
    pub(super) pair: usize,
    pub(super) condition: String,
    pub(super) write_gaps: Vec<usize>,
    pub(super) query_gap: usize,
    pub(super) target_write_gap: usize,
    pub(super) query_span: Vec<u32>,
}
fn push(ids: &mut Vec<u32>, roles: &mut Vec<String>, id: u32, role: &str) {
    ids.push(id);
    roles.push(role.into());
}
pub(super) fn episode(
    facts: &[(Vec<u32>, u32)],
    write_noise: &[Vec<u32>],
    query_key: &[u32],
    query_noise: &[u32],
    pair: usize,
    condition: &str,
) -> Episode {
    let (mut ids, mut roles) = (Vec::new(), Vec::new());
    push(&mut ids, &mut roles, 36, "bos");
    let (mut source, mut answer) = (0, 0);
    let mut target_write_gap = 0;
    for (i, (key, value)) in facts.iter().enumerate() {
        push(&mut ids, &mut roles, 32, "write_open");
        for &word in key {
            push(&mut ids, &mut roles, word, "write_key");
        }
        push(&mut ids, &mut roles, 39, "write_commit");
        for &noise in &write_noise[i] {
            push(&mut ids, &mut roles, 37, "noise_marker");
            push(&mut ids, &mut roles, noise, "noise_key");
        }
        push(&mut ids, &mut roles, 35, "value_marker");
        push(&mut ids, &mut roles, *value, "value");
        if key.as_slice() == query_key {
            source = ids.len() - 1;
            answer = *value;
            target_write_gap = write_noise[i].len();
        }
    }
    push(&mut ids, &mut roles, 33, "query_open");
    for &word in query_key {
        push(&mut ids, &mut roles, word, "query_key");
    }
    push(&mut ids, &mut roles, 39, "query_commit");
    for &noise in query_noise {
        push(&mut ids, &mut roles, 37, "noise_marker");
        push(&mut ids, &mut roles, noise, "noise_key");
    }
    push(&mut ids, &mut roles, 34, "answer_marker");
    let query = ids.len() - 1;
    Episode {
        ids,
        roles,
        source,
        query,
        answer,
        facts: facts.len(),
        pair,
        condition: condition.into(),
        write_gaps: write_noise.iter().map(Vec::len).collect(),
        query_gap: query_noise.len(),
        target_write_gap,
        query_span: query_key.to_vec(),
    }
}
pub(super) fn draw(rng: &mut Rng, n: usize) -> Vec<(Vec<u32>, u32)> {
    let mut words: Vec<u32> = (0..16).collect();
    for i in (1..16).rev() {
        let j = rng.next(i + 1);
        words.swap(i, j);
    }
    let mut facts = Vec::new();
    for pair in 0..n / 2 {
        let (a, b) = (words[2 + 2 * pair], words[3 + 2 * pair]);
        let first = 16 + rng.next(16) as u32;
        let second = 16 + ((first - 16 + 1 + rng.next(15) as u32) % 16);
        facts.push((vec![words[0], a, b, words[1]], first));
        facts.push((vec![words[0], b, a, words[1]], second));
    }
    facts
}
pub(super) fn noise(rng: &mut Rng, max: usize) -> Vec<u32> {
    let n = rng.next(max + 1);
    (0..n).map(|_| rng.next(16) as u32).collect()
}
pub(super) fn batch(es: &[Episode]) -> (Vec<u32>, Vec<u32>, Vec<f32>, usize) {
    let time = es.iter().map(|e| e.ids.len()).max().unwrap_or(0);
    let (mut ids, mut targets, mut weights) = (
        vec![38; es.len() * time],
        vec![0; es.len() * time],
        vec![0.; es.len() * time],
    );
    for (b, e) in es.iter().enumerate() {
        ids[b * time..b * time + e.ids.len()].copy_from_slice(&e.ids);
        targets[b * time + e.query] = e.answer;
        weights[b * time + e.query] = 1.;
    }
    (ids, targets, weights, time)
}
