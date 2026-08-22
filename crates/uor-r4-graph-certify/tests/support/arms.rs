//! The #845 ordering arms and controls — one scorer per roster entry of
//! `docs/w33_geometry_qualification_spec_845.md` §4, each a frontier-retention
//! functional for the reference skeleton in [`super::ordering`].
//!
//! Pinned constants live here and are mirrored in the spec's §4-A appendix.
//! Every fitted control fits on the fitting half only and is deterministic;
//! every control reports its auxiliary table bytes for the byte-parity audit
//! (the geometry arm's byte count is the shared budget).
#![allow(dead_code)]

use super::ordering::Scorer;
use super::w33;
use uor_r4_graph_format::plan::SlotVec;

/// One fitted observation: (state, goal, remaining gold steps).
pub type RemainingObservation = ((i16, i16), (i16, i16), u8);
/// One fitted transition: (state, successor).
pub type Transition = ((i16, i16), (i16, i16));

/// Geometry auxiliary bytes: the two 40 x 40 tables (`d_W`, `phi`). This is
/// the byte budget every control is matched against.
pub const GEOMETRY_TABLE_BYTES: usize = 2 * w33::POINTS * w33::POINTS;

/// splitmix64 — the pinned deterministic generator for every seeded table.
pub fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

fn slots_of(state: &SlotVec) -> (i16, i16) {
    let slice = state.as_slice();
    (
        slice.first().copied().unwrap_or(0),
        slice.get(1).copied().unwrap_or(0),
    )
}

/// The shared W(3,3) context every geometry-shaped scorer borrows: the pinned
/// point list and the two tables.
pub struct W33Tables {
    pub points: Vec<w33::Vec4>,
    pub distance: Vec<[u8; w33::POINTS]>,
    pub phase: Vec<[u8; w33::POINTS]>,
}

impl W33Tables {
    pub fn build() -> Self {
        let points = w33::points();
        let distance = w33::distance_table(&points);
        let phase = w33::phase_table(&points);
        Self {
            points,
            distance,
            phase,
        }
    }
}

/// The W(3,3) geometry arm: retention score `-(4 * d_W + t(phi))` of the
/// successor's mapped point against the goal's mapped point (spec §3).
pub struct GeometryScorer<'t> {
    tables: &'t W33Tables,
    goal_point: usize,
    lookups: u64,
}

impl<'t> GeometryScorer<'t> {
    pub fn new(tables: &'t W33Tables, goal: (i16, i16)) -> Self {
        let goal_point = w33::map_state(&tables.points, goal.0, goal.1);
        Self {
            tables,
            goal_point,
            lookups: 0,
        }
    }
}

impl Scorer for GeometryScorer<'_> {
    fn score(&mut self, successor: &SlotVec) -> i32 {
        let (s0, s1) = slots_of(successor);
        let point = w33::map_state(&self.tables.points, s0, s1);
        self.lookups += 2;
        let d = i32::from(self.tables.distance[point][self.goal_point]);
        let t = i32::from(self.tables.phase[point][self.goal_point]);
        -(4 * d + t)
    }
    fn lookups(&self) -> u64 {
        self.lookups
    }
    fn table_bytes(&self) -> usize {
        GEOMETRY_TABLE_BYTES
    }
    fn name(&self) -> &'static str {
        "w33-geometry"
    }
}

/// Hamming/popcount control: retention score = negated popcount of the raw
/// 32-bit slot pair XORed against the goal's. No auxiliary tables.
pub struct HammingScorer {
    goal_bits: u32,
}

impl HammingScorer {
    pub fn new(goal: (i16, i16)) -> Self {
        Self {
            goal_bits: pack_bits(goal),
        }
    }
}

fn pack_bits(slots: (i16, i16)) -> u32 {
    (u32::from(slots.0 as u16)) | (u32::from(slots.1 as u16) << 16)
}

impl Scorer for HammingScorer {
    fn score(&mut self, successor: &SlotVec) -> i32 {
        let bits = pack_bits(slots_of(successor));
        -((bits ^ self.goal_bits).count_ones() as i32)
    }
    fn lookups(&self) -> u64 {
        0
    }
    fn table_bytes(&self) -> usize {
        0
    }
    fn name(&self) -> &'static str {
        "hamming-popcount"
    }
}

/// Learned control over the *same* quantization as the geometry arm: a
/// 40 x 40 table of mean remaining-gold-steps between mapped point classes,
/// fitted on the fitting half (u8 saturating; 255 = never observed), plus the
/// 40 x 16-bit binary codes derived from it by row-median thresholding. The
/// retention score is the learned-table distance with the code Hamming
/// distance as refinement — the sharpest non-geometric test of whether the
/// quadrangle *structure*, rather than any table over the same 40 classes,
/// carries the signal.
pub struct LearnedTable {
    table: Vec<[u8; w33::POINTS]>,
    codes: Vec<u16>,
}

impl LearnedTable {
    /// `observations` are (state, goal, remaining-gold-steps) triples drawn
    /// from fitting-half gold paths.
    pub fn fit(tables: &W33Tables, observations: &[RemainingObservation]) -> Self {
        let mut sum = vec![[0u32; w33::POINTS]; w33::POINTS];
        let mut count = vec![[0u32; w33::POINTS]; w33::POINTS];
        for ((s0, s1), (g0, g1), remaining) in observations {
            let s = w33::map_state(&tables.points, *s0, *s1);
            let g = w33::map_state(&tables.points, *g0, *g1);
            sum[s][g] += u32::from(*remaining);
            count[s][g] += 1;
        }
        let mut table = vec![[255u8; w33::POINTS]; w33::POINTS];
        for s in 0..w33::POINTS {
            for g in 0..w33::POINTS {
                if let Some(mean) = sum[s][g].checked_div(count[s][g]) {
                    table[s][g] = mean.min(254) as u8;
                }
            }
        }
        // Row codes: bit b set when the row entry at column-block b sits at or
        // below the row's observed median (16 four-column blocks).
        let codes = table
            .iter()
            .map(|row| {
                let mut observed: Vec<u8> = row.iter().copied().filter(|v| *v != 255).collect();
                observed.sort_unstable();
                let median = observed.get(observed.len() / 2).copied().unwrap_or(255);
                let mut code = 0u16;
                for block in 0..16 {
                    let start = block * w33::POINTS / 16;
                    let end = (block + 1) * w33::POINTS / 16;
                    let hit = row[start..end].iter().any(|v| *v <= median);
                    if hit {
                        code |= 1 << block;
                    }
                }
                code
            })
            .collect();
        Self { table, codes }
    }
}

/// The per-episode learned-control scorer.
pub struct LearnedScorer<'t> {
    tables: &'t W33Tables,
    fitted: &'t LearnedTable,
    goal_point: usize,
    lookups: u64,
}

impl<'t> LearnedScorer<'t> {
    pub fn new(tables: &'t W33Tables, fitted: &'t LearnedTable, goal: (i16, i16)) -> Self {
        let goal_point = w33::map_state(&tables.points, goal.0, goal.1);
        Self {
            tables,
            fitted,
            goal_point,
            lookups: 0,
        }
    }
}

impl Scorer for LearnedScorer<'_> {
    fn score(&mut self, successor: &SlotVec) -> i32 {
        let (s0, s1) = slots_of(successor);
        let point = w33::map_state(&self.tables.points, s0, s1);
        self.lookups += 2;
        let learned = i32::from(self.fitted.table[point][self.goal_point]);
        let code = self.fitted.codes[point] ^ self.fitted.codes[self.goal_point];
        -(learned * 32 + code.count_ones() as i32)
    }
    fn lookups(&self) -> u64 {
        self.lookups
    }
    fn table_bytes(&self) -> usize {
        w33::POINTS * w33::POINTS + 2 * w33::POINTS
    }
    fn name(&self) -> &'static str {
        "learned-table-codes"
    }
}

/// VSA/binding control: per-slot role tables of 128-bit fillers over the nine
/// slot residues (pinned seed), bound by XOR; retention score = negated
/// Hamming distance of the bound state and goal hypervectors.
pub struct VsaTables {
    fillers: [[u128; 9]; 2],
}

impl VsaTables {
    pub fn build() -> Self {
        let mut state = 0x845a_0001_u64;
        let mut fillers = [[0u128; 9]; 2];
        for role in &mut fillers {
            for filler in role.iter_mut() {
                let high = splitmix64(&mut state) as u128;
                let low = splitmix64(&mut state) as u128;
                *filler = (high << 64) | low;
            }
        }
        Self { fillers }
    }

    fn hyper(&self, slots: (i16, i16)) -> u128 {
        let r0 = slots.0.rem_euclid(9) as usize;
        let r1 = slots.1.rem_euclid(9) as usize;
        self.fillers[0][r0] ^ self.fillers[1][r1]
    }
}

pub struct VsaScorer<'t> {
    tables: &'t VsaTables,
    goal: u128,
    lookups: u64,
}

impl<'t> VsaScorer<'t> {
    pub fn new(tables: &'t VsaTables, goal: (i16, i16)) -> Self {
        Self {
            tables,
            goal: tables.hyper(goal),
            lookups: 0,
        }
    }
}

impl Scorer for VsaScorer<'_> {
    fn score(&mut self, successor: &SlotVec) -> i32 {
        self.lookups += 2;
        let bound = self.tables.hyper(slots_of(successor));
        -((bound ^ self.goal).count_ones() as i32)
    }
    fn lookups(&self) -> u64 {
        self.lookups
    }
    fn table_bytes(&self) -> usize {
        2 * 9 * 16
    }
    fn name(&self) -> &'static str {
        "vsa-binding"
    }
}

/// Random-embedding null: seed-pinned tables of the same shape and value
/// range as the geometry tables (d in {0,1,2}, t in {0,1,2}), scored with the
/// same functional. Matched in bytes and form; devoid of quadrangle structure.
pub struct RandomTables {
    d: Vec<[u8; w33::POINTS]>,
    t: Vec<[u8; w33::POINTS]>,
}

impl RandomTables {
    pub fn build() -> Self {
        let mut state = 0x845a_0002_u64;
        let mut fill = || {
            (0..w33::POINTS)
                .map(|_| {
                    let mut row = [0u8; w33::POINTS];
                    for slot in row.iter_mut() {
                        *slot = (splitmix64(&mut state) % 3) as u8;
                    }
                    row
                })
                .collect::<Vec<_>>()
        };
        let d = fill();
        let t = fill();
        Self { d, t }
    }
}

pub struct RandomScorer<'t> {
    tables: &'t W33Tables,
    random: &'t RandomTables,
    goal_point: usize,
    lookups: u64,
}

impl<'t> RandomScorer<'t> {
    pub fn new(tables: &'t W33Tables, random: &'t RandomTables, goal: (i16, i16)) -> Self {
        let goal_point = w33::map_state(&tables.points, goal.0, goal.1);
        Self {
            tables,
            random,
            goal_point,
            lookups: 0,
        }
    }
}

impl Scorer for RandomScorer<'_> {
    fn score(&mut self, successor: &SlotVec) -> i32 {
        let (s0, s1) = slots_of(successor);
        let point = w33::map_state(&self.tables.points, s0, s1);
        self.lookups += 2;
        let d = i32::from(self.random.d[point][self.goal_point]);
        let t = i32::from(self.random.t[point][self.goal_point]);
        -(4 * d + t)
    }
    fn lookups(&self) -> u64 {
        self.lookups
    }
    fn table_bytes(&self) -> usize {
        GEOMETRY_TABLE_BYTES
    }
    fn name(&self) -> &'static str {
        "random-embedding"
    }
}

/// Isomorphic-relabel control: the true geometry tables conjugated by a
/// pinned *scrambling* permutation of the 40 points (Fisher–Yates, seeded) —
/// the quadrangle's internal structure survives, its alignment with the state
/// space does not. The permutation is asserted NOT to be a collinearity
/// automorphism (else the control would be vacuous). Tables are conjugated at
/// build time, so the byte budget equals the geometry arm's exactly.
pub struct RelabeledTables {
    d: Vec<[u8; w33::POINTS]>,
    t: Vec<[u8; w33::POINTS]>,
    pub rho: Vec<usize>,
}

impl RelabeledTables {
    pub fn build(tables: &W33Tables) -> Self {
        let mut state = 0x845a_0003_u64;
        let mut rho: Vec<usize> = (0..w33::POINTS).collect();
        for i in (1..w33::POINTS).rev() {
            let j = (splitmix64(&mut state) % (i as u64 + 1)) as usize;
            rho.swap(i, j);
        }
        let mut d = vec![[0u8; w33::POINTS]; w33::POINTS];
        let mut t = vec![[0u8; w33::POINTS]; w33::POINTS];
        for p in 0..w33::POINTS {
            for q in 0..w33::POINTS {
                d[p][q] = tables.distance[rho[p]][rho[q]];
                t[p][q] = tables.phase[rho[p]][rho[q]];
            }
        }
        Self { d, t, rho }
    }

    /// Whether the pinned permutation preserves d_W everywhere — the control
    /// requires this to be FALSE (asserted by test).
    pub fn is_automorphism(&self, tables: &W33Tables) -> bool {
        (0..w33::POINTS).all(|p| (0..w33::POINTS).all(|q| self.d[p][q] == tables.distance[p][q]))
    }
}

pub struct RelabeledScorer<'t> {
    tables: &'t W33Tables,
    relabeled: &'t RelabeledTables,
    goal_point: usize,
    lookups: u64,
}

impl<'t> RelabeledScorer<'t> {
    pub fn new(tables: &'t W33Tables, relabeled: &'t RelabeledTables, goal: (i16, i16)) -> Self {
        let goal_point = w33::map_state(&tables.points, goal.0, goal.1);
        Self {
            tables,
            relabeled,
            goal_point,
            lookups: 0,
        }
    }
}

impl Scorer for RelabeledScorer<'_> {
    fn score(&mut self, successor: &SlotVec) -> i32 {
        let (s0, s1) = slots_of(successor);
        let point = w33::map_state(&self.tables.points, s0, s1);
        self.lookups += 2;
        let d = i32::from(self.relabeled.d[point][self.goal_point]);
        let t = i32::from(self.relabeled.t[point][self.goal_point]);
        -(4 * d + t)
    }
    fn lookups(&self) -> u64 {
        self.lookups
    }
    fn table_bytes(&self) -> usize {
        GEOMETRY_TABLE_BYTES
    }
    fn name(&self) -> &'static str {
        "isomorphic-relabel"
    }
}

/// Adversarial phase-permutation control: the true d_W with the phase values
/// swapped (1 <-> 2) — isolates whether the pinned phase convention
/// specifically carries signal beyond the collinearity metric.
pub struct PhasePermutedScorer<'t> {
    tables: &'t W33Tables,
    goal_point: usize,
    lookups: u64,
}

impl<'t> PhasePermutedScorer<'t> {
    pub fn new(tables: &'t W33Tables, goal: (i16, i16)) -> Self {
        let goal_point = w33::map_state(&tables.points, goal.0, goal.1);
        Self {
            tables,
            goal_point,
            lookups: 0,
        }
    }
}

impl Scorer for PhasePermutedScorer<'_> {
    fn score(&mut self, successor: &SlotVec) -> i32 {
        let (s0, s1) = slots_of(successor);
        let point = w33::map_state(&self.tables.points, s0, s1);
        self.lookups += 2;
        let d = i32::from(self.tables.distance[point][self.goal_point]);
        let t = i32::from(w33::permute_phase(
            self.tables.phase[point][self.goal_point],
        ));
        -(4 * d + t)
    }
    fn lookups(&self) -> u64 {
        self.lookups
    }
    fn table_bytes(&self) -> usize {
        GEOMETRY_TABLE_BYTES
    }
    fn name(&self) -> &'static str {
        "phase-permuted"
    }
}

/// Spectral control: an offline f64 Laplacian eigen-embedding of the observed
/// class-transition graph (fitting half only), quantized to i16 tables. The
/// eigensolver is a fixed-sweep cyclic Jacobi rotation — deterministic by
/// construction (fixed sweep order and count, no convergence branching on
/// accumulated error). f64 is compiler/certifier scope, per the spec.
pub struct SpectralEmbedding {
    embed: Vec<[i16; SPECTRAL_DIMS]>,
}

/// Embedding width. 40 x 8 x 2 bytes = 640 auxiliary bytes.
pub const SPECTRAL_DIMS: usize = 8;
/// Fixed Jacobi sweep count (each sweep visits every upper-triangle pair).
pub const JACOBI_SWEEPS: usize = 32;

impl SpectralEmbedding {
    /// `transitions` are (state, successor) pairs from fitting-half gold
    /// paths, mapped to point classes.
    pub fn fit(tables: &W33Tables, transitions: &[Transition]) -> Self {
        let n = w33::POINTS;
        let mut weight = vec![[0f64; w33::POINTS]; w33::POINTS];
        for ((s0, s1), (t0, t1)) in transitions {
            let a = w33::map_state(&tables.points, *s0, *s1);
            let b = w33::map_state(&tables.points, *t0, *t1);
            if a != b {
                weight[a][b] += 1.0;
                weight[b][a] += 1.0;
            }
        }
        // L = D - W.
        let mut matrix = vec![[0f64; w33::POINTS]; w33::POINTS];
        for i in 0..n {
            let degree: f64 = weight[i].iter().sum();
            for j in 0..n {
                matrix[i][j] = if i == j { degree } else { -weight[i][j] };
            }
        }
        let (values, vectors) = jacobi_eigen(&mut matrix);
        // Ascending eigenvalues; skip the constant kernel vector; take the
        // next SPECTRAL_DIMS. Ties break by original index (stable sort).
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|a, b| {
            values[*a]
                .partial_cmp(&values[*b])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let chosen: Vec<usize> = order.into_iter().skip(1).take(SPECTRAL_DIMS).collect();
        let embed = (0..n)
            .map(|point| {
                let mut row = [0i16; SPECTRAL_DIMS];
                for (dim, source) in chosen.iter().enumerate() {
                    let value = vectors[point][*source] * 1024.0;
                    row[dim] = value.clamp(-32768.0, 32767.0) as i16;
                }
                row
            })
            .collect();
        Self { embed }
    }
}

/// Cyclic Jacobi over a symmetric matrix: returns (eigenvalues on the
/// diagonal, eigenvector columns). Fixed sweeps in a fixed pair order.
fn jacobi_eigen(matrix: &mut [[f64; w33::POINTS]]) -> (Vec<f64>, Vec<[f64; w33::POINTS]>) {
    let n = w33::POINTS;
    let mut vectors = vec![[0f64; w33::POINTS]; w33::POINTS];
    for (i, row) in vectors.iter_mut().enumerate() {
        row[i] = 1.0;
    }
    for _sweep in 0..JACOBI_SWEEPS {
        for p in 0..n {
            for q in (p + 1)..n {
                let apq = matrix[p][q];
                if apq == 0.0 {
                    continue;
                }
                let app = matrix[p][p];
                let aqq = matrix[q][q];
                let theta = 0.5 * (aqq - app) / apq;
                let t = if theta >= 0.0 {
                    1.0 / (theta + (1.0 + theta * theta).sqrt())
                } else {
                    1.0 / (theta - (1.0 + theta * theta).sqrt())
                };
                let c = 1.0 / (1.0 + t * t).sqrt();
                let s = t * c;
                for row in matrix.iter_mut() {
                    let akp = row[p];
                    let akq = row[q];
                    row[p] = c * akp - s * akq;
                    row[q] = s * akp + c * akq;
                }
                let (head, tail) = matrix.split_at_mut(q);
                let row_p = &mut head[p];
                let row_q = &mut tail[0];
                for (apk, aqk) in row_p.iter_mut().zip(row_q.iter_mut()) {
                    let a = *apk;
                    let b = *aqk;
                    *apk = c * a - s * b;
                    *aqk = s * a + c * b;
                }
                for row in vectors.iter_mut() {
                    let vp = row[p];
                    let vq = row[q];
                    row[p] = c * vp - s * vq;
                    row[q] = s * vp + c * vq;
                }
            }
        }
    }
    let values = (0..n).map(|i| matrix[i][i]).collect();
    (values, vectors)
}

pub struct SpectralScorer<'t> {
    tables: &'t W33Tables,
    embedding: &'t SpectralEmbedding,
    goal_point: usize,
    lookups: u64,
}

impl<'t> SpectralScorer<'t> {
    pub fn new(tables: &'t W33Tables, embedding: &'t SpectralEmbedding, goal: (i16, i16)) -> Self {
        let goal_point = w33::map_state(&tables.points, goal.0, goal.1);
        Self {
            tables,
            embedding,
            goal_point,
            lookups: 0,
        }
    }
}

impl Scorer for SpectralScorer<'_> {
    fn score(&mut self, successor: &SlotVec) -> i32 {
        let (s0, s1) = slots_of(successor);
        let point = w33::map_state(&self.tables.points, s0, s1);
        self.lookups += 2;
        let a = &self.embedding.embed[point];
        let b = &self.embedding.embed[self.goal_point];
        let l1: i32 = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| (i32::from(*x) - i32::from(*y)).abs())
            .sum();
        -l1
    }
    fn lookups(&self) -> u64 {
        self.lookups
    }
    fn table_bytes(&self) -> usize {
        w33::POINTS * SPECTRAL_DIMS * 2
    }
    fn name(&self) -> &'static str {
        "spectral-embedding"
    }
}
