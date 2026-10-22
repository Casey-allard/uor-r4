//! Finite host controls for item `m1-faster-writer`
//! (ORDER `docs/loop/tasks/m1-faster-writer/claude-34/ORDER.md`).
//!
//! `reference_writer` mirrors the prediction part of today's full-scan
//! writer in `src/discrete.cu`: `action`/`move_action`/`table_block`
//! (:20-28), `root_cost` (:37-40), `address_distance`, `token_distance`,
//! `root_id`, `token_cell` (:274-290), `cell_precedes` (:301-303) and
//! `writer` (:307-325). The loss terms after the pick are out of scope.
//!
//! `bucketed_writer` is the proposed CSR bucket index over the 256 root-pair
//! address cells (design note on #2840). It is test-only: this file adds no
//! library writer and no CPU fallback to any product path. Integer only.

mod geometry {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/geometry.rs"));
}

const ABSENT: u32 = u32::MAX;
const CELLS: usize = 256;
const WIDTH: usize = 8;

/// The kernel's `(kind, index, candidate)` proposal. Only `kind == 1` with a
/// nonzero candidate moves a block; `index` is `token * blocks + block`.
#[derive(Clone, Copy, Debug)]
struct Move {
    kind: u32,
    index: u32,
    candidate: u32,
}

const NO_MOVE: Move = Move {
    kind: 0,
    index: 0,
    candidate: 0,
};

#[derive(Clone, Debug)]
struct Vocab {
    blocks: usize,
    table: Vec<i32>,
}

impl Vocab {
    fn new(blocks: usize, tokens: &[Vec<[i32; 8]>]) -> Self {
        let mut table = Vec::new();
        for token in tokens {
            assert_eq!(token.len(), blocks);
            token
                .iter()
                .for_each(|block| table.extend_from_slice(block));
        }
        Vocab { blocks, table }
    }

    fn len(&self) -> usize {
        self.table.len() / (self.blocks * WIDTH)
    }
}

#[derive(Default, Debug)]
struct Counters {
    token_cell_calls: u64,
    cell_distance_evals: u64,
    token_distance_evals: u64,
}

/// The writer's prediction and its admitted cells in admission order.
/// `None` is the kernel's `~(W)0` (MAX) return.
#[derive(Debug, PartialEq, Eq)]
struct Pick {
    token: u32,
    admitted: Vec<u32>,
}

// ---------------------------------------------------------------------------
// Host reference of discrete.cu
// ---------------------------------------------------------------------------

fn root(r: u32) -> [i32; 8] {
    geometry::ROOTS[r as usize].map(i32::from)
}

/// `action`, discrete.cu:20-22.
fn action(a: u8, x: &[i32]) -> [i32; 8] {
    let mut y = [0; 8];
    for (d, out) in y.iter_mut().enumerate() {
        let axis = geometry::AXES[usize::from(a) * 8 + d];
        let v = x[usize::from(axis & 7)];
        *out = if axis & 8 != 0 { -v } else { v };
    }
    y
}

/// `move_action`, discrete.cu:23.
fn move_action(candidate: u32) -> u8 {
    if candidate != 0 {
        geometry::GENERATORS[candidate as usize - 1]
    } else {
        geometry::IDENTITY
    }
}

/// `table_block`, discrete.cu:24-28.
fn table_block(v: &Vocab, token: u32, block: usize, mv: Move) -> [i32; 8] {
    let at = token as usize * v.blocks + block;
    let p = &v.table[at * WIDTH..(at + 1) * WIDTH];
    if mv.kind == 1 && mv.index as usize == at && mv.candidate != 0 {
        action(move_action(mv.candidate), p)
    } else {
        p.try_into().expect("eight coordinates")
    }
}

/// Distance part of `root_cost`, discrete.cu:37-40.
fn root_cost(x: &[i32], r: u32) -> u32 {
    x.iter().zip(root(r)).map(|(&a, b)| a.abs_diff(b)).sum()
}

/// `address_distance`, discrete.cu:274-276.
fn address_distance(q: &[i32], blocks: usize, cell: u32, c: &mut Counters) -> u32 {
    c.cell_distance_evals += 1;
    root_cost(&q[..WIDTH], cell / 16)
        + root_cost(&q[(blocks - 1) * WIDTH..blocks * WIDTH], cell % 16)
}

/// `token_distance`, discrete.cu:277-281: full L1 over every block.
fn token_distance(q: &[i32], v: &Vocab, token: u32, mv: Move, c: &mut Counters) -> u32 {
    c.token_distance_evals += 1;
    (0..v.blocks)
        .map(|b| {
            table_block(v, token, b, mv)
                .iter()
                .zip(&q[b * WIDTH..(b + 1) * WIDTH])
                .map(|(&x, &y)| y.abs_diff(x))
                .sum::<u32>()
        })
        .sum()
}

/// `root_id`, discrete.cu:282-285: exact match against the 16 roots.
fn root_id(x: &[i32; 8]) -> u32 {
    (0..16).find(|&r| root(r) == *x).unwrap_or(ABSENT)
}

/// `token_cell`, discrete.cu:286-290.
fn token_cell(v: &Vocab, token: u32, mv: Move, c: &mut Counters) -> u32 {
    c.token_cell_calls += 1;
    let a = root_id(&table_block(v, token, 0, mv));
    let b = root_id(&table_block(v, token, v.blocks - 1, mv));
    if a == ABSENT || b == ABSENT {
        ABSENT
    } else {
        a * 16 + b
    }
}

/// `cell_precedes`, discrete.cu:301-303.
fn cell_precedes(distance: u32, cell: u32, other_distance: u32, other_cell: u32) -> bool {
    distance < other_distance || (distance == other_distance && cell < other_cell)
}

/// Prediction part of `writer`, discrete.cu:307-325. The kernel's extra
/// `token_cell(target)` call feeds only the loss and is not mirrored.
fn reference_writer(q: &[i32], v: &Vocab, k: usize, mv: Move, c: &mut Counters) -> Option<Pick> {
    let n = v.len() as u32;
    let mut occupied = [false; CELLS];
    let mut used = [false; CELLS];
    for token in 0..n {
        let cell = token_cell(v, token, mv, c);
        if cell == ABSENT {
            return None;
        }
        occupied[cell as usize] = true;
    }
    let (mut best_token, mut best_distance) = (ABSENT, ABSENT);
    let mut admitted = Vec::new();
    for _ in 0..k {
        let (mut cell, mut cost) = (ABSENT, ABSENT);
        for other in 0..CELLS as u32 {
            if occupied[other as usize] && !used[other as usize] {
                let d = address_distance(q, v.blocks, other, c);
                if cell_precedes(d, other, cost, cell) {
                    cell = other;
                    cost = d;
                }
            }
        }
        if cell == ABSENT {
            break;
        }
        used[cell as usize] = true;
        admitted.push(cell);
        for token in 0..n {
            if token_cell(v, token, mv, c) == cell {
                let d = token_distance(q, v, token, mv, c);
                if d < best_distance || (d == best_distance && token < best_token) {
                    best_token = token;
                    best_distance = d;
                }
            }
        }
    }
    (best_token != ABSENT).then_some(Pick {
        token: best_token,
        admitted,
    })
}

// ---------------------------------------------------------------------------
// Host bucketed writer
// ---------------------------------------------------------------------------

/// CSR index over the 256 address cells, built once per codebook.
/// `cells` keeps each token's base cell and `absent` counts ABSENT tokens,
/// so a virtual move can update occupancy and the ABSENT state exactly.
struct BucketIndex {
    offsets: [u32; CELLS + 1],
    members: Vec<u32>,
    cells: Vec<u32>,
    absent: u32,
}

fn build_index(v: &Vocab) -> BucketIndex {
    // Codebook-time cost, outside the per-row counters.
    let mut build = Counters::default();
    let cells: Vec<u32> = (0..v.len() as u32)
        .map(|t| token_cell(v, t, NO_MOVE, &mut build))
        .collect();
    let mut offsets = [0u32; CELLS + 1];
    for &cell in cells.iter().filter(|&&cell| cell != ABSENT) {
        offsets[cell as usize + 1] += 1;
    }
    let mut running = 0;
    for offset in offsets.iter_mut() {
        running += *offset;
        *offset = running;
    }
    let mut fill = offsets;
    let mut members = vec![0; offsets[CELLS] as usize];
    for (token, &cell) in cells.iter().enumerate() {
        if cell != ABSENT {
            let slot = &mut fill[cell as usize];
            members[*slot as usize] = token as u32;
            *slot += 1;
        }
    }
    let absent = cells.iter().filter(|&&cell| cell == ABSENT).count() as u32;
    BucketIndex {
        offsets,
        members,
        cells,
        absent,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Walk {
    /// Every admitted bucket, in cell-id order.
    IdOrder,
    /// Admitted buckets in admission order, skipping a bucket whose address
    /// distance (a lower bound on every member's token distance) is strictly
    /// greater than the best token distance so far.
    ServingPruned,
}

fn bucketed_writer(
    q: &[i32],
    v: &Vocab,
    idx: &BucketIndex,
    k: usize,
    mv: Move,
    walk: Walk,
    c: &mut Counters,
) -> Option<Pick> {
    let moved = (mv.kind == 1 && mv.candidate != 0 && (mv.index as usize) < v.len() * v.blocks)
        .then(|| mv.index / v.blocks as u32);
    let mut count: [u32; CELLS] =
        std::array::from_fn(|cell| idx.offsets[cell + 1] - idx.offsets[cell]);
    let mut absent = idx.absent;
    // Only the moved token's cell is recomputed.
    let moved_cell = moved.map(|token| {
        let base = idx.cells[token as usize];
        let now = token_cell(v, token, mv, c);
        if base == ABSENT {
            absent -= 1;
        } else {
            count[base as usize] -= 1;
        }
        if now == ABSENT {
            absent += 1;
        } else {
            count[now as usize] += 1;
        }
        now
    });
    if absent > 0 {
        return None;
    }

    // Admission: one pass computing each cell's address distance once, then a
    // counting sort on the distance key (ids ascend within a key).
    let mut distance = [0u32; CELLS];
    for (cell, d) in distance.iter_mut().enumerate() {
        *d = address_distance(q, v.blocks, cell as u32, c);
    }
    let mut keys: Vec<u32> = Vec::new();
    for (&members, &d) in count.iter().zip(&distance) {
        if members > 0 {
            if let Err(at) = keys.binary_search(&d) {
                keys.insert(at, d);
            }
        }
    }
    let mut admitted = Vec::with_capacity(k);
    'admit: for &key in &keys {
        for (cell, (&members, &d)) in count.iter().zip(&distance).enumerate() {
            if admitted.len() == k {
                break 'admit;
            }
            if members > 0 && d == key {
                admitted.push(cell as u32);
            }
        }
    }

    let mut order = admitted.clone();
    if walk == Walk::IdOrder {
        order.sort_unstable();
    }
    let (mut best_token, mut best_distance) = (ABSENT, ABSENT);
    for &cell in &order {
        if walk == Walk::ServingPruned && distance[cell as usize] > best_distance {
            continue;
        }
        let span = idx.offsets[cell as usize] as usize..idx.offsets[cell as usize + 1] as usize;
        let mover = moved.filter(|_| moved_cell == Some(cell));
        for &t in idx.members[span]
            .iter()
            .filter(|&&t| Some(t) != moved)
            .chain(mover.iter())
        {
            let d = token_distance(q, v, t, mv, c);
            if d < best_distance || (d == best_distance && t < best_token) {
                best_token = t;
                best_distance = d;
            }
        }
    }
    (best_token != ABSENT).then_some(Pick {
        token: best_token,
        admitted,
    })
}

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Runs the reference and both bucketed walks; asserts they agree exactly.
fn all_agree(q: &[i32], v: &Vocab, idx: &BucketIndex, k: usize, mv: Move) -> Option<Pick> {
    let reference = reference_writer(q, v, k, mv, &mut Counters::default());
    for walk in [Walk::IdOrder, Walk::ServingPruned] {
        let got = bucketed_writer(q, v, idx, k, mv, walk, &mut Counters::default());
        assert_eq!(got, reference, "{walk:?} k={k} move={mv:?} q={q:?}");
    }
    reference
}

fn query(blocks: &[[i32; 8]]) -> Vec<i32> {
    blocks.iter().flatten().copied().collect()
}

fn non_root() -> [i32; 8] {
    let x = [9; 8];
    assert_eq!(root_id(&x), ABSENT);
    x
}

/// Two distinct roots `x < y`, both unequal to `from`, at equal L1 from it.
fn equal_distance_pair(from: u32) -> (u32, u32) {
    for x in 0..16 {
        for y in x + 1..16 {
            if x != from && y != from && root_cost(&root(from), x) == root_cost(&root(from), y) {
                return (x, y);
            }
        }
    }
    panic!("no equal-distance root pair from root {from}");
}

/// A root `r` and a candidate whose action takes `root(r)` to another root.
fn crossing_move() -> (u32, u32, u32) {
    for r in 0..16 {
        for candidate in 1..16 {
            let to = root_id(&action(move_action(candidate), &root(r)));
            if to != ABSENT && to != r {
                return (r, candidate, to);
            }
        }
    }
    panic!("no generator moves a root to another root");
}

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    fn below(&mut self, n: usize) -> usize {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x % n as u64) as usize
    }
}

fn perturb(rng: &mut Rng, mut x: [i32; 8]) -> [i32; 8] {
    x[rng.below(8)] += if rng.below(2) == 0 { 1 } else { -1 };
    x
}

/// Tokens built from a small root palette so cells crowd and ties are common;
/// some tokens repeat earlier ones; about one vocabulary in ten has one
/// ABSENT token.
fn random_vocab(rng: &mut Rng, palette: &[u32], n: usize, blocks: usize) -> Vocab {
    let mut tokens: Vec<Vec<[i32; 8]>> = Vec::with_capacity(n);
    for _ in 0..n {
        if !tokens.is_empty() && rng.below(4) == 0 {
            let copy = tokens[rng.below(tokens.len())].clone();
            tokens.push(copy);
            continue;
        }
        let token = (0..blocks)
            .map(|b| {
                let x = root(palette[rng.below(palette.len())]);
                let interior = b != 0 && b != blocks - 1;
                if interior && rng.below(3) == 0 {
                    perturb(rng, x)
                } else {
                    x
                }
            })
            .collect();
        tokens.push(token);
    }
    if rng.below(10) == 0 {
        let t = rng.below(n);
        let b = if rng.below(2) == 0 { 0 } else { blocks - 1 };
        tokens[t][b] = non_root();
    }
    Vocab::new(blocks, &tokens)
}

fn random_query(rng: &mut Rng, palette: &[u32], blocks: usize) -> Vec<i32> {
    let q: Vec<[i32; 8]> = (0..blocks)
        .map(|_| {
            let x = root(palette[rng.below(palette.len())]);
            if rng.below(3) == 0 {
                perturb(rng, x)
            } else {
                x
            }
        })
        .collect();
    query(&q)
}

fn random_move(rng: &mut Rng, n: usize, blocks: usize) -> Move {
    if rng.below(4) == 0 {
        // Inert proposal kinds (0 = plain forward, 2 = export).
        return Move {
            kind: [0, 2][rng.below(2)],
            index: rng.below(n * blocks) as u32,
            candidate: rng.below(16) as u32,
        };
    }
    let token = rng.below(n);
    let block = match rng.below(3) {
        0 => 0,
        1 => blocks - 1,
        _ => rng.below(blocks),
    };
    Move {
        kind: 1,
        index: (token * blocks + block) as u32,
        candidate: rng.below(16) as u32,
    }
}

// ---------------------------------------------------------------------------
// (a) Seeded sweep
// ---------------------------------------------------------------------------

#[test]
fn bucketed_and_pruned_writers_match_full_scan_on_seeded_vocabularies() {
    let (mut picks, mut max_returns, mut cell_changing_moves) = (0u32, 0u32, 0u32);
    for seed in 1..=300u64 {
        let mut rng = Rng::new(seed);
        let n = 8 + rng.below(57); // V in 8..=64
        let blocks = 2 + rng.below(3); // 2..=4
        let palette: Vec<u32> = (0..2 + rng.below(15))
            .map(|_| rng.below(16) as u32)
            .collect();
        let v = random_vocab(&mut rng, &palette, n, blocks);
        let idx = build_index(&v);
        for _ in 0..24 {
            let k = 1 + rng.below(16); // K in 1..=16
            let q = random_query(&mut rng, &palette, blocks);
            let mv = random_move(&mut rng, n, blocks);
            match all_agree(&q, &v, &idx, k, mv) {
                Some(_) => picks += 1,
                None => max_returns += 1,
            }
            if mv.kind == 1 {
                let token = mv.index / blocks as u32;
                let mut scratch = Counters::default();
                if token_cell(&v, token, mv, &mut scratch) != idx.cells[token as usize] {
                    cell_changing_moves += 1;
                }
            }
        }
    }
    assert!(picks > 1000, "sweep made only {picks} picks");
    assert!(max_returns > 0, "sweep never hit an ABSENT token");
    assert!(
        cell_changing_moves > 100,
        "only {cell_changing_moves} moves changed a cell"
    );
}

// ---------------------------------------------------------------------------
// (b) Constructed ties
// ---------------------------------------------------------------------------

/// Two cells at equal address distance: the smaller cell id is admitted
/// first. At K = 2 the two tokens also tie on token distance across cells,
/// so the smaller token id wins and the serving prune must not skip the
/// second cell (its bound equals, but does not exceed, the best distance).
#[test]
fn admission_tie_prefers_smaller_cell_id_and_cross_cell_token_tie_prefers_smaller_token_id() {
    let (x, y) = equal_distance_pair(0);
    let (cell_x, cell_y) = (x, y); // row-root 0: cell = 0 * 16 + root
    let v = Vocab::new(2, &[vec![root(0), root(y)], vec![root(0), root(x)]]);
    let idx = build_index(&v);
    let q = query(&[root(0), root(0)]);
    assert_eq!(
        all_agree(&q, &v, &idx, 1, NO_MOVE),
        Some(Pick {
            token: 1,
            admitted: vec![cell_x],
        })
    );
    assert_eq!(
        all_agree(&q, &v, &idx, 2, NO_MOVE),
        Some(Pick {
            token: 0,
            admitted: vec![cell_x, cell_y],
        })
    );
}

/// Two identical tokens in one cell: the smaller token id wins.
#[test]
fn same_cell_token_tie_prefers_smaller_token_id() {
    let near = vec![root(3), root(4), root(5)];
    let far = vec![root(9), root(10), root(11)];
    let other = vec![root(12), root(13), root(14)];
    let tokens = [far.clone(), far.clone(), near.clone(), far, other, near];
    let v = Vocab::new(3, &tokens);
    let idx = build_index(&v);
    let q = query(&[root(3), root(4), root(5)]);
    for k in [1, 2, 16] {
        assert_eq!(
            all_agree(&q, &v, &idx, k, NO_MOVE).map(|p| p.token),
            Some(2)
        );
    }
}

// ---------------------------------------------------------------------------
// (c) ABSENT
// ---------------------------------------------------------------------------

#[test]
fn absent_token_returns_max_for_every_writer() {
    let v = Vocab::new(
        3,
        &[
            vec![root(1), root(2), root(3)],
            vec![root(4), root(5), root(6)],
            vec![root(1), root(2), root(3)],
            vec![root(7), root(8), non_root()],
        ],
    );
    let idx = build_index(&v);
    assert_eq!(idx.absent, 1);
    let q = query(&[root(1), root(2), root(3)]);
    let (_, candidate, _) = crossing_move();
    let moves = [
        NO_MOVE,
        // Token 0 interior block, token 1 first block.
        Move {
            kind: 1,
            index: 1,
            candidate,
        },
        Move {
            kind: 1,
            index: 3,
            candidate,
        },
        // The ABSENT token's interior block, then its ABSENT block itself.
        Move {
            kind: 1,
            index: 3 * 3 + 1,
            candidate,
        },
        Move {
            kind: 1,
            index: 3 * 3 + 2,
            candidate,
        },
    ];
    for mv in moves {
        for k in [1, 4, 16] {
            assert_eq!(all_agree(&q, &v, &idx, k, mv), None, "move {mv:?}");
        }
    }
}

// ---------------------------------------------------------------------------
// (d) Virtual moves across cells
// ---------------------------------------------------------------------------

#[test]
fn virtual_move_into_and_out_of_admitted_cell_matches_full_scan() {
    let (r, candidate, to) = crossing_move();
    let other = (0..16).find(|&o| o != r && o != to).expect("third root");
    let base_cell = r * 16 + other;
    let moved_cell = to * 16 + other;

    // Into: token 1 moves from the base cell to a cell at address distance 0.
    let v = Vocab::new(2, &[vec![root(r), root(other)], vec![root(r), root(other)]]);
    let idx = build_index(&v);
    let into = Move {
        kind: 1,
        index: 2,
        candidate,
    };
    let q = query(&[root(to), root(other)]);
    assert_eq!(
        all_agree(&q, &v, &idx, 1, NO_MOVE),
        Some(Pick {
            token: 0,
            admitted: vec![base_cell],
        })
    );
    assert_eq!(
        all_agree(&q, &v, &idx, 1, into),
        Some(Pick {
            token: 1,
            admitted: vec![moved_cell],
        })
    );

    // Out, base cell keeps a member: token 0 leaves; token 2 is picked there.
    let v = Vocab::new(
        2,
        &[
            vec![root(r), root(other)],
            vec![root(other), root(other)],
            vec![root(r), root(other)],
        ],
    );
    let idx = build_index(&v);
    let q = query(&[root(r), root(other)]);
    let out = Move {
        kind: 1,
        index: 0,
        candidate,
    };
    assert_eq!(
        all_agree(&q, &v, &idx, 1, NO_MOVE).map(|p| p.token),
        Some(0)
    );
    assert_eq!(
        all_agree(&q, &v, &idx, 1, out),
        Some(Pick {
            token: 2,
            admitted: vec![base_cell],
        })
    );

    // Out, base cell empties: admission moves on to another occupied cell.
    let v = Vocab::new(
        2,
        &[vec![root(r), root(other)], vec![root(other), root(other)]],
    );
    let idx = build_index(&v);
    let emptied = all_agree(&q, &v, &idx, 1, out).expect("pick");
    assert_ne!(emptied.admitted, vec![base_cell]);
    for k in [2, 3] {
        all_agree(&q, &v, &idx, k, out);
    }
}

// ---------------------------------------------------------------------------
// (e) Counters
// ---------------------------------------------------------------------------

#[test]
fn bucketed_writer_makes_no_token_cell_calls_and_256_cell_evaluations_per_row() {
    let mut rng = Rng::new(7);
    let palette: Vec<u32> = (0..16).collect();
    let n = 48;
    let blocks = 3;
    let v = loop {
        let v = random_vocab(&mut rng, &palette, n, blocks);
        if build_index(&v).absent == 0 {
            break v;
        }
    };
    let idx = build_index(&v);
    let (_, candidate, _) = crossing_move();
    for (mv, calls) in [
        (NO_MOVE, 0),
        (
            Move {
                kind: 1,
                index: 5 * 3,
                candidate,
            },
            1,
        ),
    ] {
        for k in [1, 8, 16] {
            let q = random_query(&mut rng, &palette, blocks);
            let mut full = Counters::default();
            let pick = reference_writer(&q, &v, k, mv, &mut full).expect("pick");
            let admitted = pick.admitted.len() as u64;
            assert_eq!(full.token_cell_calls, n as u64 * (1 + admitted));
            for walk in [Walk::IdOrder, Walk::ServingPruned] {
                let mut c = Counters::default();
                bucketed_writer(&q, &v, &idx, k, mv, walk, &mut c);
                assert_eq!(c.token_cell_calls, calls, "{walk:?} move={mv:?}");
                assert_eq!(c.cell_distance_evals, CELLS as u64, "{walk:?} move={mv:?}");
            }
        }
    }
}
