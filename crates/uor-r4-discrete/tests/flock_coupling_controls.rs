//! Finite host controls for item `m4-flock-coupling-prereg`
//! (ORDER `docs/loop/tasks/m4-flock-coupling-prereg/antigravity-25/ORDER.md`).
//!
//! Verifies all hand-derived literals (`C0`, `X1`–`X12`, `F1`–`F7`, `F5b`, `F5c`)
//! against the build-time generated `geometry.rs` tables (`ROOTS`, `AXES`,
//! `COMPOSE`, `INVERSE`, `BFS`, `GENERATORS`, `IDENTITY`, `OPERATOR_KAPPA`,
//! `COMPOSITION_KAPPA`) and the integer Flock / writer semantics in
//! `src/discrete.cu` (`snap` at `:102-110`, `flock_vector` at `:125-267`,
//! `writer` at `:307-345`, `discrete_counterfactual` at `:384-407`, and
//! `discrete_choose` at `:413-430`).
//!
//! Pure integer and exact rational `(num, den)` arithmetic only; adds no library
//! path and no CPU fallback to any product path.

use uor_r4_discrete::metadata::BinderPlan;

mod geometry {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/geometry.rs"));
}

const ABSENT: u32 = u32::MAX;
const MAX_LOSS: u64 = u64::MAX;
const TRACE_ROW: usize = 4;
const TRACE_LEVEL: usize = 28;
const EDGE_TYPES: usize = 5;

const A: [i32; 8] = [2, 2, 0, 0, 0, 0, 0, 0];
const NEG_A: [i32; 8] = [-2, -2, 0, 0, 0, 0, 0, 0];
const NEG_B: [i32; 8] = [0, 0, -2, 0, -2, 0, 0, 0];
const R4: [i32; 8] = [0, 0, -2, 0, 2, 0, 0, 0];
const R13: [i32; 8] = [0, 0, 2, 0, -2, 0, 0, 0];
const B: [i32; 8] = [0, 0, 2, 0, 2, 0, 0, 0];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReadMode {
    Route,
    Flock { kept: u32 },
    FlockCoupled { kept: u32, k_max: i32 },
    RelationCoupled { kept: u32, sweeps: u32, k_max: i32 },
}

#[derive(Clone, Debug)]
struct CoupledFixture {
    blocks: usize,
    context: usize,
    kept_cells: u32,
    mode: ReadMode,
    tokens: Vec<Vec<[i32; 8]>>,
    gates: Vec<i32>,
}

fn root(r: u32) -> [i32; 8] {
    geometry::ROOTS[r as usize].map(i32::from)
}

fn root_id(v: &[i32; 8]) -> u32 {
    for r in 0..16u32 {
        if &root(r) == v {
            return r;
        }
    }
    ABSENT
}

fn action(a: u8, x: &[i32; 8]) -> [i32; 8] {
    let mut y = [0; 8];
    for (d, out) in y.iter_mut().enumerate() {
        let axis = geometry::AXES[usize::from(a) * 8 + d];
        let v = x[usize::from(axis & 7)];
        *out = if axis & 8 != 0 { -v } else { v };
    }
    y
}

fn root_cost(x: &[i32; 8], r: u32) -> (u32, u32) {
    let target = root(r);
    let mut distance = 0u32;
    let mut hamming = 0u32;
    for d in 0..8 {
        distance += x[d].abs_diff(target[d]);
        if x[d] != target[d] {
            hamming += 1;
        }
    }
    (distance, hamming)
}

/// `snap`, `discrete.cu:102-110`: builds `n * r` by repeated addition.
fn snap(x: &[i32; 8], n: u32) -> (u32, [u32; 16], [u32; 16]) {
    let mut best = ABSENT;
    let mut best_distance = ABSENT;
    let mut best_hamming = ABSENT;
    let mut distances = [0u32; 16];
    let mut hammings = [0u32; 16];
    for r in 0..16u32 {
        let rv = root(r);
        let mut distance = 0u32;
        let mut hamming = 0u32;
        for d in 0..8 {
            let mut y = 0i32;
            for _ in 0..n {
                y += rv[d];
            }
            distance += x[d].abs_diff(y);
            if x[d] != y {
                hamming += 1;
            }
        }
        distances[r as usize] = distance;
        hammings[r as usize] = hamming;
        if distance < best_distance || (distance == best_distance && hamming < best_hamming) {
            best = r;
            best_distance = distance;
            best_hamming = hamming;
        }
    }
    (best, distances, hammings)
}

fn stages_for_context(context: usize) -> Result<usize, u32> {
    BinderPlan::new(context).map(|p| p.stages()).map_err(|_| 10)
}

fn validate_coupled_fixture(f: &CoupledFixture) -> Result<(), u32> {
    let stages = stages_for_context(f.context)?;
    match f.mode {
        ReadMode::Route => {
            if f.gates.iter().any(|&g| !(-1..=1).contains(&g)) {
                return Err(10);
            }
        }
        ReadMode::Flock { kept } => {
            if !(1..=8).contains(&kept) || f.gates.len() != f.blocks * stages {
                return Err(10);
            }
            if f.gates.iter().any(|&g| !(-1..=1).contains(&g)) {
                return Err(10);
            }
        }
        ReadMode::FlockCoupled { kept, k_max } => {
            if !(1..=8).contains(&kept)
                || !(1..=3).contains(&k_max)
                || f.gates.len() != f.blocks * stages
            {
                return Err(10);
            }
            if f.gates.iter().any(|&g| g.abs() > k_max) {
                return Err(10);
            }
        }
        ReadMode::RelationCoupled {
            kept,
            sweeps,
            k_max,
        } => {
            if !(1..=8).contains(&kept)
                || !(1..=5).contains(&sweeps)
                || !(1..=3).contains(&k_max)
                || f.gates.len() != f.blocks * stages + EDGE_TYPES * stages
            {
                return Err(10);
            }
            let combine = &f.gates[..f.blocks * stages];
            let relation = &f.gates[f.blocks * stages..];
            if combine.iter().any(|&g| g.abs() > k_max)
                || relation.iter().any(|&g| !(-1..=1).contains(&g))
            {
                return Err(10);
            }
        }
    }
    Ok(())
}

/// Widened counterfactual gate value (`X7`):
/// Ordinal 0 returns `current`. Ordinals `1..=2*k_max` visit
/// `[0, -1, +1, -2, +2, -3, +3]` (bounded by `k_max`) skipping `current`.
fn coupled_gate_value(
    gates: &[i32],
    at: usize,
    k_max: i32,
    kind: u32,
    index: usize,
    candidate: u32,
) -> i32 {
    let current = gates[at];
    if kind != 3 || index != at || candidate == 0 {
        return current;
    }
    let order = [0, -1, 1, -2, 2, -3, 3];
    let active_len = (2 * k_max + 1) as usize;
    let mut k = 0u32;
    for &val in &order[..active_len] {
        if val == current {
            continue;
        }
        k += 1;
        if k == candidate {
            return val;
        }
    }
    current
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ForwardOut {
    vectors: Vec<[i32; 8]>,
    trace: Vec<u32>,
}

/// Exact integer `flock_vector` / `FlockCoupled` forward for one prefix window.
fn flock_coupled_forward(
    f: &CoupledFixture,
    window: &[usize],
    kind: u32,
    index: usize,
    candidate: u32,
) -> Result<ForwardOut, u32> {
    validate_coupled_fixture(f)?;
    let (kept, k_max) = match f.mode {
        ReadMode::Flock { kept } => (kept as usize, 1i32),
        ReadMode::FlockCoupled { kept, k_max } => (kept as usize, k_max),
        ReadMode::RelationCoupled { kept, k_max, .. } => (kept as usize, k_max),
        ReadMode::Route => return Err(9),
    };
    let stages = stages_for_context(f.context)?;
    let length = window.len();
    if length == 0 || length > f.context || f.blocks < 2 {
        return Err(2);
    }
    let binder = BinderPlan::new(length).map_err(|_| 2u32)?;
    let plan = binder.node_words();
    let nodes = binder.nodes_count();
    let mut level = vec![0usize; nodes];
    let mut first = vec![0usize; nodes];
    let mut last = vec![0usize; nodes];
    for node in 0..nodes {
        let atom = plan[node * 4 + 3];
        if atom != ABSENT {
            let atom = atom as usize;
            level[node] = 0;
            first[node] = atom;
            last[node] = atom;
        } else {
            let c0 = plan[node * 4] as usize;
            let c2 = plan[node * 4 + 2] as usize;
            level[node] = level[c0] + 1;
            first[node] = first[c0];
            last[node] = last[c2];
        }
    }

    let mut vectors = Vec::with_capacity(f.blocks);
    let mut trace = vec![ABSENT; f.blocks * (TRACE_ROW + stages * TRACE_LEVEL)];

    for b in 0..f.blocks {
        let frame_blk = if b == 0 { f.blocks - 1 } else { 0 };
        let query_tok = window[length - 1];
        let query_frame = f.tokens[query_tok][frame_blk];
        let mut u = f.tokens[query_tok][b];
        let fq = root_id(&query_frame);
        if fq == ABSENT {
            return Err(2);
        }
        let mut frame = vec![0u32; nodes];
        let mut payload = vec![[0i32; 8]; nodes];
        for node in 0..nodes {
            let atom = plan[node * 4 + 3];
            if atom != ABSENT {
                let atom = atom as usize;
                let fv = f.tokens[window[atom]][frame_blk];
                let fr = root_id(&fv);
                if fr == ABSENT {
                    return Err(2);
                }
                frame[node] = fr;
                payload[node] = f.tokens[window[atom]][b];
            } else {
                let c0 = plan[node * 4] as usize;
                frame[node] = frame[c0];
                let mut sum = [0i32; 8];
                for c in 0..3 {
                    let child = plan[node * 4 + c] as usize;
                    let act = geometry::BFS[frame[child] as usize * 16 + frame[c0] as usize];
                    let carried = action(act, &payload[child]);
                    for d in 0..8 {
                        sum[d] += carried[d];
                    }
                }
                let (r, _, _) = snap(&sum, 3);
                payload[node] = root(r);
            }
        }

        let rec_base = b * (TRACE_ROW + stages * TRACE_LEVEL);
        trace[rec_base] = fq;
        trace[rec_base + 3] = root_id(&u);

        let mut terms = 1u32;
        for l in 0..stages {
            let gate = coupled_gate_value(&f.gates, b * stages + l, k_max, kind, index, candidate);
            if gate.abs() > k_max {
                return Err(10);
            }
            let lv_base = rec_base + TRACE_ROW + l * TRACE_LEVEL;
            // X12: level word 0 is `gate + k_max` under FlockCoupled (equals `gate + 1` at k_max = 1).
            trace[lv_base] = (gate + k_max) as u32;

            let mut top = [0usize; 8];
            let mut key_d = [0u32; 8];
            let mut key_h = [0u32; 8];
            let mut found = 0usize;
            let mut eligible = 0u32;
            for node in 0..nodes {
                if level[node] != l || last[node] + 1 >= length {
                    continue;
                }
                eligible += 1;
                let (distance, hamming) = root_cost(&query_frame, frame[node]);
                let mut at = found;
                while at > 0 {
                    let o = top[at - 1];
                    let better = distance < key_d[at - 1]
                        || (distance == key_d[at - 1]
                            && (hamming < key_h[at - 1]
                                || (hamming == key_h[at - 1]
                                    && (last[node] > last[o]
                                        || (last[node] == last[o] && node < o)))));
                    if !better {
                        break;
                    }
                    if at < kept {
                        top[at] = top[at - 1];
                        key_d[at] = key_d[at - 1];
                        key_h[at] = key_h[at - 1];
                    }
                    at -= 1;
                }
                if at < kept {
                    top[at] = node;
                    key_d[at] = distance;
                    key_h[at] = hamming;
                    if found < kept {
                        found += 1;
                    }
                }
            }
            trace[lv_base + 1] = found as u32;
            trace[lv_base + 3] = eligible;
            if found == 0 {
                continue;
            }
            let mut sum = [0i32; 8];
            for k in 0..found {
                let node = top[k];
                let a = geometry::BFS[frame[node] as usize * 16 + fq as usize];
                let carried = action(a, &payload[node]);
                for d in 0..8 {
                    sum[d] += carried[d];
                }
                trace[lv_base + 4 + k] = node as u32;
                trace[lv_base + 12 + k] = u32::from(a);
                trace[lv_base + 20 + k] = root_id(&carried);
            }
            let (m, _, _) = snap(&sum, found as u32);
            trace[lv_base + 2] = m;
            if gate == 0 {
                continue;
            }
            // Repeated addition of +-m by |gate| (`discrete.cu` C1 extension).
            let mv = root(m);
            let mag = gate.unsigned_abs();
            for _ in 0..mag {
                for d in 0..8 {
                    u[d] += if gate > 0 { mv[d] } else { -mv[d] };
                }
                terms += 1;
            }
        }
        let (out, _, _) = snap(&u, terms);
        trace[rec_base + 1] = out;
        trace[rec_base + 2] = terms;
        vectors.push(root(out));
    }

    Ok(ForwardOut { vectors, trace })
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WriterOut {
    loss: u64,
    prediction: u32,
    token_rivals: u32,
    address_rivals: u32,
    token_hinge: u64,
    address_hinge: u64,
}

fn token_cell(blocks: &[[i32; 8]]) -> u32 {
    let a = root_id(&blocks[0]);
    let b = root_id(&blocks[blocks.len() - 1]);
    if a == ABSENT || b == ABSENT {
        ABSENT
    } else {
        a * 16 + b
    }
}

fn address_distance(q: &[[i32; 8]], cell: u32) -> u32 {
    let (a, _) = root_cost(&q[0], cell / 16);
    let (b, _) = root_cost(&q[q.len() - 1], cell % 16);
    a + b
}

fn token_distance(q: &[[i32; 8]], tok_blocks: &[[i32; 8]]) -> u32 {
    let mut sum = 0u32;
    for (qb, tb) in q.iter().zip(tok_blocks.iter()) {
        for d in 0..8 {
            sum += qb[d].abs_diff(tb[d]);
        }
    }
    sum
}

/// `writer`, `discrete.cu:307-345` (`NearestAddressMarginV1`).
fn writer(q: &[[i32; 8]], tokens: &[Vec<[i32; 8]>], kept_cells: u32, target: usize) -> WriterOut {
    let mut occupied = [false; 256];
    let mut used = [false; 256];
    for tok in tokens {
        let c = token_cell(tok);
        assert_ne!(c, ABSENT);
        occupied[c as usize] = true;
    }
    let target_cell = token_cell(&tokens[target]);
    let mut best_token = ABSENT;
    let mut best_distance = ABSENT;
    let mut rival_distance = ABSENT;
    let mut rival_address = ABSENT;
    let mut token_rivals = 0u32;
    let mut address_rivals = 0u32;

    for _ in 0..kept_cells {
        let mut cell = ABSENT;
        let mut cost = ABSENT;
        for c in 0..256u32 {
            if occupied[c as usize] && !used[c as usize] {
                let d = address_distance(q, c);
                if d < cost || (d == cost && c < cell) {
                    cell = c;
                    cost = d;
                }
            }
        }
        if cell == ABSENT {
            break;
        }
        used[cell as usize] = true;
        if cell != target_cell {
            address_rivals += 1;
            if cost < rival_address {
                rival_address = cost;
            }
        }
        for (tok_id, tok) in tokens.iter().enumerate() {
            if token_cell(tok) == cell {
                let d = token_distance(q, tok);
                if d < best_distance || (d == best_distance && (tok_id as u32) < best_token) {
                    best_token = tok_id as u32;
                    best_distance = d;
                }
                if tok_id != target {
                    token_rivals += 1;
                    if d < rival_distance {
                        rival_distance = d;
                    }
                }
            }
        }
    }

    let width = (q.len() * 8) as u32;
    let td = token_distance(q, &tokens[target]);
    let ad = address_distance(q, target_cell);
    let tm = if rival_distance == ABSENT || width + td <= rival_distance {
        0
    } else {
        u64::from(width + td - rival_distance)
    };
    let am = if rival_address == ABSENT || 8 + ad <= rival_address {
        0
    } else {
        u64::from(8 + ad - rival_address)
    };
    let loss = 8 * tm + u64::from(width) * am;
    WriterOut {
        loss,
        prediction: best_token,
        token_rivals,
        address_rivals,
        token_hinge: tm,
        address_hinge: am,
    }
}

/// Cached tuple extracted from `forward_traced` (`C0`, §1 / §4.0).
#[derive(Clone, Debug, PartialEq, Eq)]
struct CachedRowTuple {
    u_roots: Vec<u32>,
    found_per_level: Vec<Vec<u32>>,
    m_roots: Vec<Vec<u32>>,
}

fn extract_cached_tuple(trace: &[u32], blocks: usize, stages: usize) -> Result<CachedRowTuple, u32> {
    let mut u_roots = Vec::with_capacity(blocks);
    let mut found_per_level = Vec::with_capacity(blocks);
    let mut m_roots = Vec::with_capacity(blocks);
    for b in 0..blocks {
        let rec_base = b * (TRACE_ROW + stages * TRACE_LEVEL);
        let u_root = trace[rec_base + 3];
        if u_root == ABSENT || u_root >= 16 {
            return Err(8);
        }
        u_roots.push(u_root);
        let mut founds = Vec::with_capacity(stages);
        let mut ms = Vec::with_capacity(stages);
        for l in 0..stages {
            let lv_base = rec_base + TRACE_ROW + l * TRACE_LEVEL;
            let found = trace[lv_base + 1];
            let m = trace[lv_base + 2];
            if found > 0 && (m == ABSENT || m >= 16) {
                return Err(8);
            }
            founds.push(found);
            ms.push(m);
        }
        found_per_level.push(founds);
        m_roots.push(ms);
    }
    Ok(CachedRowTuple {
        u_roots,
        found_per_level,
        m_roots,
    })
}

fn replay_cached_tuple(tuple: &CachedRowTuple, gates: &[i32], stages: usize) -> Vec<[i32; 8]> {
    let blocks = tuple.u_roots.len();
    let mut out_vectors = Vec::with_capacity(blocks);
    for b in 0..blocks {
        let mut u = root(tuple.u_roots[b]);
        let mut terms = 1u32;
        for l in 0..stages {
            let found = tuple.found_per_level[b][l];
            if found == 0 {
                continue;
            }
            let gate = gates[b * stages + l];
            if gate == 0 {
                continue;
            }
            let mv = root(tuple.m_roots[b][l]);
            for _ in 0..gate.unsigned_abs() {
                for d in 0..8 {
                    u[d] += if gate > 0 { mv[d] } else { -mv[d] };
                }
                terms += 1;
            }
        }
        let (out_r, _, _) = snap(&u, terms);
        out_vectors.push(root(out_r));
    }
    out_vectors
}

fn fixture_t_prime(k_max: i32, gates: &[i32]) -> CoupledFixture {
    CoupledFixture {
        blocks: 2,
        context: 2,
        kept_cells: 2,
        mode: ReadMode::FlockCoupled { kept: 1, k_max },
        tokens: vec![vec![A, A], vec![A, B]],
        gates: gates.to_vec(),
    }
}

#[test]
fn geometry_roots_and_bfs_match_rederived_prereg_literals() {
    assert_eq!(root(0), A);
    assert_eq!(root(1), NEG_A);
    assert_eq!(root(3), NEG_B);
    assert_eq!(root(4), R4);
    assert_eq!(root(13), R13);
    assert_eq!(root(14), B);
    assert_eq!(geometry::IDENTITY, 0);
    assert_eq!(geometry::BFS[0], 0);
    assert_eq!(geometry::BFS[14], 1);
    for r in 0..16u32 {
        let rv = root(r);
        let l1: i32 = rv.iter().map(|x| x.abs()).sum();
        let nz = rv.iter().filter(|&&x| x != 0).count();
        assert_eq!(l1, 4);
        assert_eq!(nz, 2);
    }
}

#[test]
fn c0_cached_tuple_replay_equals_live_forward_and_refuses_absent_roots() {
    let base = fixture_t_prime(1, &[0, 0]);
    let window = [1usize, 0usize];
    let traced = flock_coupled_forward(&base, &window, 2, 0, 0).expect("live forward");
    let tuple = extract_cached_tuple(&traced.trace, 2, 1).expect("valid tuple");
    for k0 in -3..=3 {
        for k1 in -3..=3 {
            let live = flock_coupled_forward(
                &fixture_t_prime(3, &[k0, k1]),
                &window,
                2,
                0,
                0,
            )
            .expect("live coupled forward");
            let replayed = replay_cached_tuple(&tuple, &[k0, k1], 1);
            assert_eq!(replayed, live.vectors);
            let live_w = writer(&live.vectors, &base.tokens, base.kept_cells, 1);
            let replay_w = writer(&replayed, &base.tokens, base.kept_cells, 1);
            assert_eq!(replay_w, live_w);
        }
    }

    // Mutate trace so `u_root = ABSENT` or `m_root = ABSENT` with `found_l > 0`: must refuse.
    let mut bad_u = traced.trace.clone();
    bad_u[3] = ABSENT;
    assert_eq!(extract_cached_tuple(&bad_u, 2, 1), Err(8));

    let mut bad_m = traced.trace.clone();
    bad_m[TRACE_ROW + 2] = ABSENT;
    assert_eq!(extract_cached_tuple(&bad_m, 2, 1), Err(8));
}

#[test]
fn x1_to_x5_and_x10_to_x12_integer_export_kernel_controls_match_literals() {
    let window = [1usize, 0usize];

    // X1: Zero coupling is inert (`K = 0` -> out `[A, A]`, `terms = 1`, bitwise equal to Flock).
    let flock_zero = CoupledFixture {
        mode: ReadMode::Flock { kept: 1 },
        ..fixture_t_prime(1, &[0, 0])
    };
    let coupled_zero = fixture_t_prime(1, &[0, 0]);
    let f0 = flock_coupled_forward(&flock_zero, &window, 2, 0, 0).unwrap();
    let c0 = flock_coupled_forward(&coupled_zero, &window, 2, 0, 0).unwrap();
    assert_eq!(c0, f0);
    assert_eq!(c0.vectors, vec![A, A]);
    assert_eq!(c0.trace[2], 1);
    assert_eq!(c0.trace[TRACE_ROW + TRACE_LEVEL + 2], 1);

    // X2: Trit embedding (`k_max = 1 == Flock` bitwise across K in {-1, 0, +1}).
    for k in [-1, 0, 1] {
        let flock_k = CoupledFixture {
            mode: ReadMode::Flock { kept: 1 },
            ..fixture_t_prime(1, &[0, k])
        };
        let coupled_k = fixture_t_prime(1, &[0, k]);
        let fk = flock_coupled_forward(&flock_k, &window, 2, 0, 0).unwrap();
        let ck = flock_coupled_forward(&coupled_k, &window, 2, 0, 0).unwrap();
        assert_eq!(ck, fk);
        assert_eq!(ck.vectors, vec![A, A]);
        let expected_terms = if k == 0 { 1 } else { 2 };
        assert_eq!(ck.trace[TRACE_ROW + TRACE_LEVEL + 2], expected_terms);
    }
    // Check the exact tie structure for K = +1 (`a + b`) and K = -1 (`a - b`) at n = 2.
    let (best_pos1, dist_pos1, ham_pos1) = snap(&[2, 2, 2, 0, 2, 0, 0, 0], 2);
    assert_eq!((best_pos1, dist_pos1[0], dist_pos1[14], ham_pos1[0], ham_pos1[14]), (0, 8, 8, 4, 4));
    let (best_neg1, dist_neg1, ham_neg1) = snap(&[2, 2, -2, 0, -2, 0, 0, 0], 2);
    assert_eq!((best_neg1, dist_neg1[0], dist_neg1[3], ham_neg1[0], ham_neg1[3]), (0, 8, 8, 4, 4));

    // X3: Magnitude flips the read (`K = +2` -> residual `[2,2,4,0,4,0,0,0]`, `n = 3`, `D(b) = 8`).
    let (best_p2, dist_p2, _) = snap(&[2, 2, 4, 0, 4, 0, 0, 0], 3);
    assert_eq!(best_p2, 14);
    assert_eq!(dist_p2[14], 8);
    assert_eq!(dist_p2[0], 16);
    assert_eq!(dist_p2[4], 16);
    assert_eq!(dist_p2[13], 16);
    for (r, &d) in dist_p2.iter().enumerate() {
        if !matches!(r, 0 | 4 | 13 | 14) {
            assert!(d >= 20, "root {r} had distance {d} < 20");
        }
    }
    let c_p2 = flock_coupled_forward(&fixture_t_prime(3, &[0, 2]), &window, 2, 0, 0).unwrap();
    assert_eq!(c_p2.vectors, vec![A, B]);
    assert_eq!(c_p2.trace[TRACE_ROW + TRACE_LEVEL + 2], 3);

    // X4: Signed magnitude (`K = -2` -> residual `[2,2,-4,0,-4,0,0,0]`, `n = 3`, `D(r3) = 8`).
    let (best_m2, dist_m2, _) = snap(&[2, 2, -4, 0, -4, 0, 0, 0], 3);
    assert_eq!(best_m2, 3);
    assert_eq!(dist_m2[3], 8);
    let c_m2 = flock_coupled_forward(&fixture_t_prime(3, &[0, -2]), &window, 2, 0, 0).unwrap();
    assert_eq!(c_m2.vectors, vec![A, NEG_B]);
    assert_eq!(c_m2.trace[TRACE_ROW + TRACE_LEVEL + 2], 3);

    // X5: Saturation (`K = +3` -> residual `[2,2,6,0,6,0,0,0]`, `n = 4`, `D(b) = 8`, `D(a) = 24`, `D(r4) = D(r13) = 20`).
    let (best_p3, dist_p3, _) = snap(&[2, 2, 6, 0, 6, 0, 0, 0], 4);
    assert_eq!(best_p3, 14);
    assert_eq!(dist_p3[14], 8);
    assert_eq!(dist_p3[0], 24);
    assert_eq!(dist_p3[4], 20);
    assert_eq!(dist_p3[13], 20);
    let c_p3 = flock_coupled_forward(&fixture_t_prime(3, &[0, 3]), &window, 2, 0, 0).unwrap();
    assert_eq!(c_p3.vectors, vec![A, B]);
    assert_eq!(c_p3.trace[TRACE_ROW + TRACE_LEVEL + 2], 4);

    // X10: `n = 16` bound (`k_max = 3`, 5 stages -> `terms = 16`, `|16 * 2| = 32`).
    let (best_16, dist_16, _) = snap(&[2, 2, 30, 0, 30, 0, 0, 0], 16);
    assert_eq!(best_16, 14);
    assert_eq!(dist_16[14], 8);
    assert_eq!(dist_16[4], 68);
    assert_eq!(dist_16[13], 68);
    assert_eq!(dist_16[0], 120);

    // X12: Trace encoding (`gate + k_max` avoids ABSENT at `K = -2` or `-3`).
    assert_eq!(
        c_m2.trace[TRACE_ROW + TRACE_LEVEL + TRACE_ROW],
        (-2i32 + 3i32) as u32
    );
    assert_ne!(c_m2.trace[TRACE_ROW + TRACE_LEVEL + TRACE_ROW], ABSENT);
}

#[test]
fn x6_and_x11_refusals_and_relation_trit_bounds_hold() {
    // X6: |K| > k_max refused with status 10.
    assert_eq!(
        validate_coupled_fixture(&fixture_t_prime(2, &[0, 3])),
        Err(10)
    );
    // k_max outside {1, 2, 3} refused with status 10.
    assert_eq!(
        validate_coupled_fixture(&fixture_t_prime(0, &[0, 0])),
        Err(10)
    );
    assert_eq!(
        validate_coupled_fixture(&fixture_t_prime(4, &[0, 0])),
        Err(10)
    );
    // Gate count mismatch refused with status 10.
    assert_eq!(
        validate_coupled_fixture(&fixture_t_prime(3, &[0])),
        Err(10)
    );
    // Flock and Route reject |g| > 1 with status 10.
    let bad_flock = CoupledFixture {
        mode: ReadMode::Flock { kept: 1 },
        ..fixture_t_prime(1, &[0, 2])
    };
    assert_eq!(validate_coupled_fixture(&bad_flock), Err(10));
    let bad_route = CoupledFixture {
        mode: ReadMode::Route,
        ..fixture_t_prime(1, &[0, 2])
    };
    assert_eq!(validate_coupled_fixture(&bad_route), Err(10));

    // X11: Relation variant widens combine gates to [-k_max, k_max] while keeping relation trits in [-1, 1].
    let mut rel_gates = vec![0i32; 4 * 5 + EDGE_TYPES * 5];
    rel_gates[0] = 3;
    rel_gates[19] = -3;
    rel_gates[20] = 1;
    let rel_ok = CoupledFixture {
        blocks: 4,
        context: 192,
        kept_cells: 8,
        mode: ReadMode::RelationCoupled {
            kept: 8,
            sweeps: 1,
            k_max: 3,
        },
        tokens: vec![vec![A; 4], vec![B; 4]],
        gates: rel_gates.clone(),
    };
    assert_eq!(validate_coupled_fixture(&rel_ok), Ok(()));

    rel_gates[20] = 2; // Relation gate outside [-1, 1] -> status 10.
    let rel_bad = CoupledFixture {
        gates: rel_gates,
        ..rel_ok
    };
    assert_eq!(validate_coupled_fixture(&rel_bad), Err(10));
}

#[test]
fn x7_widened_counterfactual_lanes_match_derived_writer_losses_on_t_prime_and_c8() {
    let window = [1usize, 0usize];

    // First verify C8 (`tokens = [[A, B], [A, A]]`, `k_max = 1`) reproduces `[448, 256, 64]`.
    let c8 = CoupledFixture {
        tokens: vec![vec![A, B], vec![A, A]],
        ..fixture_t_prime(1, &[0, 0])
    };
    let mut c8_losses = [MAX_LOSS; 16];
    for cand in 0..3u32 {
        let fwd = flock_coupled_forward(&c8, &window, 3, 1, cand).unwrap();
        c8_losses[cand as usize] = writer(&fwd.vectors, &c8.tokens, c8.kept_cells, 1).loss;
    }
    assert_eq!(&c8_losses[..3], &[448, 256, 64]);
    assert!(c8_losses[3..].iter().all(|&l| l == MAX_LOSS));

    // Now verify X7 on T' (`tokens = [[A, A], [A, B]]`, `Gate(1) = 0`, `k_max = 3`, target = 1 `[A, B]`):
    // Ordinals `0..7` correspond to `K = [0, -1, +1, -2, +2, -3, +3]`.
    // Exact derived losses: `[448, 448, 448, 256, 64, 256, 64]`, and `MAX_LOSS` on `7..16`.
    let t_prime = fixture_t_prime(3, &[0, 0]);
    let mut x7_losses = [MAX_LOSS; 16];
    let mut x7_gates = [0i32; 7];
    for cand in 0..7u32 {
        x7_gates[cand as usize] = coupled_gate_value(&t_prime.gates, 1, 3, 3, 1, cand);
        let fwd = flock_coupled_forward(&t_prime, &window, 3, 1, cand).unwrap();
        let w = writer(&fwd.vectors, &t_prime.tokens, t_prime.kept_cells, 1);
        x7_losses[cand as usize] = w.loss;
    }
    assert_eq!(x7_gates, [0, -1, 1, -2, 2, -3, 3]);
    assert_eq!(&x7_losses[..7], &[448, 448, 448, 256, 64, 256, 64]);
    assert!(x7_losses[7..].iter().all(|&l| l == MAX_LOSS));

    // `discrete_choose` (`discrete.cu:420-422`): strict improvement only; ties keep lower ordinal.
    let mut best = 0usize;
    for i in 1..7 {
        if x7_losses[i] < x7_losses[best] {
            best = i;
        }
    }
    assert_eq!(best, 4); // Ordinal 4 (`K = +2`, loss 64) beats ordinal 6 (`K = +3`, loss 64) by tie-break.
    assert_eq!(x7_gates[best], 2);
}

#[test]
fn x8_empty_upper_level_and_x9_two_levels_at_once_match_derived_literals() {
    // X8: Two-stage C7 fixture (`context = 3`, `length = 3`, `tokens = [[A,A],[NEG_A,A],[A,B]]`).
    // Binder node 3 covers leaves `0..=2` (`last[3] = 2 == length - 1`), so level 1 has `found_1 = 0`.
    // Setting `K_{b,1} = +2` (`gates = [0, 2, 0, 2]`) leaves `terms = 1` and output `[A, B]` unchanged.
    let x8_fixture = CoupledFixture {
        blocks: 2,
        context: 3,
        kept_cells: 3,
        mode: ReadMode::FlockCoupled { kept: 2, k_max: 3 },
        tokens: vec![vec![A, A], vec![NEG_A, A], vec![A, B]],
        gates: vec![0, 2, 0, 2],
    };
    let x8_out = flock_coupled_forward(&x8_fixture, &[0, 1, 2], 2, 0, 0).unwrap();
    assert_eq!(x8_out.vectors, vec![A, B]);
    let stride = TRACE_ROW + 2 * TRACE_LEVEL;
    assert_eq!(&x8_out.trace[0..4], &[14, 0, 1, 0]);
    assert_eq!(&x8_out.trace[stride..stride + 4], &[0, 14, 1, 14]);
    // Level 1 trace shows `found = 0`, `consensus = ABSENT`, `eligible = 0`.
    let lv1_b1 = &x8_out.trace[stride + TRACE_ROW + TRACE_LEVEL..stride + TRACE_ROW + 2 * TRACE_LEVEL];
    assert_eq!(&lv1_b1[0..4], &[5, 0, ABSENT, 0]);

    // X9: Two-stage length-4 fixture (`context = 4, kept = 1`, `tokens = [[A,A],[A,B]]`,
    // prefix `[0, 0, 1, 0]` = `[[A,A],[A,A],[A,B],[A,A]]`).
    // In block 1: `u = A = r0`; level 0 keeps atom 2 by recency (`m_0 = B = r14`);
    // level 1 admits binder node 3 covering atoms `0..=2` (`snap_3(a + a + b) = A = r0`, so `m_1 = A = r0`).
    // With `K = (+2, -1)` on block 1 (`gates = [0, 0, 2, -1]`):
    // `terms = 1 + 2 + 1 = 4`, residual `u + 2*m_0 - m_1 = a + 2b - a = [0,0,4,0,4,0,0,0]`,
    // which snaps uniquely to `B = r14` (`D(B) = 8`, `D(r4) = D(r13) = 16`, `D(A) = 24`).
    let x9_fixture = CoupledFixture {
        blocks: 2,
        context: 4,
        kept_cells: 2,
        mode: ReadMode::FlockCoupled { kept: 1, k_max: 3 },
        tokens: vec![vec![A, A], vec![A, B]],
        gates: vec![0, 0, 2, -1],
    };
    let x9_out = flock_coupled_forward(&x9_fixture, &[0, 0, 1, 0], 2, 0, 0).unwrap();
    assert_eq!(x9_out.vectors, vec![A, B]);
    assert_eq!(&x9_out.trace[stride..stride + 4], &[0, 14, 4, 0]);
    let lv0_b1 = &x9_out.trace[stride + TRACE_ROW..stride + TRACE_ROW + TRACE_LEVEL];
    let lv1_b1 = &x9_out.trace[stride + TRACE_ROW + TRACE_LEVEL..stride + TRACE_ROW + 2 * TRACE_LEVEL];
    assert_eq!(lv0_b1[1], 1); // found_0 = 1
    assert_eq!(lv0_b1[2], 14); // m_0 = B (14)
    assert_eq!(lv1_b1[1], 1); // found_1 = 1
    assert_eq!(lv1_b1[2], 0); // m_1 = A (0)
}

/// Exact rational `(num, den)` (`den > 0`) for testing F1–F5b without floating-point types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rat {
    num: i64,
    den: i64,
}

impl Rat {
    fn new(num: i128, den: i128) -> Self {
        assert!(den > 0);
        let g = gcd(num.unsigned_abs(), den as u128) as i128;
        Self {
            num: (num / g) as i64,
            den: (den / g) as i64,
        }
    }
    fn abs(self) -> Self {
        Self::new(i128::from(self.num).abs(), i128::from(self.den))
    }
    fn add(self, other: Self) -> Self {
        Self::new(
            i128::from(self.num) * i128::from(other.den)
                + i128::from(other.num) * i128::from(self.den),
            i128::from(self.den) * i128::from(other.den),
        )
    }
    fn sub(self, other: Self) -> Self {
        Self::new(
            i128::from(self.num) * i128::from(other.den)
                - i128::from(other.num) * i128::from(self.den),
            i128::from(self.den) * i128::from(other.den),
        )
    }
    fn mul(self, other: Self) -> Self {
        Self::new(
            i128::from(self.num) * i128::from(other.num),
            i128::from(self.den) * i128::from(other.den),
        )
    }
    fn div(self, other: Self) -> Self {
        assert_ne!(other.num, 0);
        let sign = if other.num < 0 { -1i128 } else { 1i128 };
        Self::new(
            i128::from(self.num) * i128::from(other.den) * sign,
            i128::from(self.den) * i128::from(other.num).abs(),
        )
    }
    fn lt(self, other: Self) -> bool {
        i128::from(self.num) * i128::from(other.den) < i128::from(other.num) * i128::from(self.den)
    }
    fn clamp(self, lo: i64, hi: i64) -> Self {
        if self.lt(Self::new(i128::from(lo), 1)) {
            Self::new(i128::from(lo), 1)
        } else if Self::new(i128::from(hi), 1).lt(self) {
            Self::new(i128::from(hi), 1)
        } else {
            self
        }
    }
    /// `round_half_away_from_zero(K)` clamped to `[-k_max, +k_max]`.
    fn export_int(self, k_max: i32) -> i32 {
        let sign = if self.num < 0 { -1i128 } else { 1i128 };
        let abs_num = i128::from(self.num).abs();
        let den = i128::from(self.den);
        let rounded_abs = (abs_num + den / 2) / den;
        let val = (sign * rounded_abs) as i32;
        val.clamp(-k_max, k_max)
    }
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a.max(1)
}

/// Mean-L1 distance `D(r) = L1(y_bar(K) - root(r))` on T' block 1 (`u = a`, `m = b`).
fn mean_l1_distance_t_prime(k: Rat, r: u32) -> Rat {
    let n = Rat::new(1, 1).add(k.abs());
    let u = A;
    let m = B;
    let rv = root(r);
    let mut sum = Rat::new(0, 1);
    for d in 0..8 {
        let yd = Rat::new(i128::from(u[d]), 1).add(k.mul(Rat::new(i128::from(m[d]), 1)));
        let y_bar_d = yd.div(n);
        let diff = y_bar_d.sub(Rat::new(i128::from(rv[d]), 1)).abs();
        sum = sum.add(diff);
    }
    sum
}

fn float_snap_t_prime(k: Rat) -> u32 {
    let mut best = 0u32;
    let mut best_d = mean_l1_distance_t_prime(k, 0);
    for r in 1..16u32 {
        let d = mean_l1_distance_t_prime(k, r);
        if d.lt(best_d) {
            best = r;
            best_d = d;
        }
    }
    best
}

fn address_hinge_loss_and_subgrad(k: Rat, target_root: u32) -> (Rat, Rat) {
    let d_target = mean_l1_distance_t_prime(k, target_root);
    let mut min_rival = None::<Rat>;
    for r in 0..16u32 {
        if r == target_root {
            continue;
        }
        let d = mean_l1_distance_t_prime(k, r);
        if min_rival.is_none_or(|cur| d.lt(cur)) {
            min_rival = Some(d);
        }
    }
    let rival = min_rival.unwrap();
    let raw = Rat::new(8, 1).add(d_target).sub(rival);
    if raw.lt(Rat::new(0, 1)) || raw == Rat::new(0, 1) {
        return (Rat::new(0, 1), Rat::new(0, 1));
    }
    let n = Rat::new(1, 1).add(k.abs());
    let n_sq = n.mul(n);
    // Closed-form subgradient on T' block 1 for target B (14) or -B (3).
    let grad = if target_root == 14 {
        if k.lt(Rat::new(2, 1)) {
            Rat::new(-16, 1).div(n_sq)
        } else {
            Rat::new(-4, 1).div(n_sq)
        }
    } else if target_root == 3 {
        if Rat::new(-2, 1).lt(k) {
            Rat::new(16, 1).div(n_sq)
        } else {
            Rat::new(4, 1).div(n_sq)
        }
    } else {
        unreachable!()
    };
    (raw, grad)
}

/// D29 banned-token scanner for the offline float module (`F6`, §2).
fn scan_offline_float_module_tokens(source: &str) -> Vec<String> {
    let banned = [
        concat!("ex", "p"),
        concat!("ex", "pf"),
        concat!("__ex", "pf"),
        concat!("lo", "g"),
        concat!("lo", "gf"),
        concat!("soft", "max"),
        concat!("ta", "nh"),
        concat!("sig", "moid"),
        concat!("er", "f"),
        concat!("po", "w"),
        concat!("wm", "ma"),
        concat!("m", "ma"),
        concat!("cub", "las"),
        concat!("cut", "lass"),
        concat!("cud", "nn"),
    ];
    let mut hits = Vec::new();
    for token in source.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
        if token.is_empty() {
            continue;
        }
        let lower = token.to_ascii_lowercase();
        if banned.iter().any(|&b| lower == b) {
            hits.push(lower);
        }
    }
    hits
}

#[test]
fn f1_to_f7_offline_proxy_and_export_gap_controls_match_literals() {
    let eta = Rat::new(1, 8);

    // F1: At `K = 1` (`n = 2`), `L = 8`, `dL/dK = -4`; one SGD step with `eta = 1/8` gives `K = 3/2`.
    let k1 = Rat::new(1, 1);
    let (l1, g1) = address_hinge_loss_and_subgrad(k1, 14);
    assert_eq!(l1, Rat::new(8, 1));
    assert_eq!(g1, Rat::new(-4, 1));
    let k1_next = k1.sub(eta.mul(g1));
    assert_eq!(k1_next, Rat::new(3, 2));

    // F2: At `K = 3` (`k_max = 3`), `L = 5`, `dL/dK = -1/4`, step + box projection returns `K = 3`.
    // At `k_max = 1`, starting from `K = 1`, step + projection returns `K = 1`.
    let k3 = Rat::new(3, 1);
    let (l3, g3) = address_hinge_loss_and_subgrad(k3, 14);
    assert_eq!(l3, Rat::new(5, 1));
    assert_eq!(g3, Rat::new(-1, 4));
    assert_eq!(k3.sub(eta.mul(g3)).clamp(-3, 3), Rat::new(3, 1));
    assert_eq!(k1_next.clamp(-1, 1), Rat::new(1, 1));

    // F3: Sign symmetry (`r* = -B = r3`, `K = -1`, `n = 2` -> `L = 8`, `dL/dK = +4`, step -> `-3/2`).
    let km1 = Rat::new(-1, 1);
    let (lm1, gm1) = address_hinge_loss_and_subgrad(km1, 3);
    assert_eq!(lm1, Rat::new(8, 1));
    assert_eq!(gm1, Rat::new(4, 1));
    let km1_next = km1.sub(eta.mul(gm1));
    assert_eq!(km1_next, Rat::new(-3, 2));

    // F4: Float/integer agreement at `K = 2` (`D(B) = 8/3` vs `16/3` -> float `B`, integer `K_int = 2` -> `B`).
    let k2 = Rat::new(2, 1);
    assert_eq!(mean_l1_distance_t_prime(k2, 14), Rat::new(8, 3));
    assert_eq!(mean_l1_distance_t_prime(k2, 0), Rat::new(16, 3));
    assert_eq!(float_snap_t_prime(k2), 14);
    let k2_int = k2.export_int(3);
    assert_eq!(k2_int, 2);
    let int_out_k2 = flock_coupled_forward(&fixture_t_prime(3, &[0, k2_int]), &[1, 0], 2, 0, 0).unwrap();
    assert_eq!(root_id(&int_out_k2.vectors[1]), 14);

    // F5: Export-gap witness at `K = 7/5` (`1.4`), `n = 12/5` (`2.4`):
    // `D(B) = 10/3`, `D(a) = 14/3`, `D(r4) = D(r13) = 17/3` -> float output `B (14)`.
    // Export `K_int = 1` -> integer output `A (0)` (X2 tie). Block 1 disagrees (1), block 0 agrees (0).
    let k_gap = Rat::new(7, 5);
    assert_eq!(mean_l1_distance_t_prime(k_gap, 14), Rat::new(10, 3));
    assert_eq!(mean_l1_distance_t_prime(k_gap, 0), Rat::new(14, 3));
    assert_eq!(mean_l1_distance_t_prime(k_gap, 4), Rat::new(17, 3));
    assert_eq!(mean_l1_distance_t_prime(k_gap, 13), Rat::new(17, 3));
    let float_b1 = float_snap_t_prime(k_gap);
    assert_eq!(float_b1, 14);
    let k_gap_int = k_gap.export_int(3);
    assert_eq!(k_gap_int, 1);
    let int_out_gap =
        flock_coupled_forward(&fixture_t_prime(3, &[0, k_gap_int]), &[1, 0], 2, 0, 0).unwrap();
    let int_b0 = root_id(&int_out_gap.vectors[0]);
    let int_b1 = root_id(&int_out_gap.vectors[1]);
    assert_eq!(int_b0, 0);
    assert_eq!(int_b1, 0);
    let block0_disagreement = usize::from(0 != int_b0);
    let block1_disagreement = usize::from(float_b1 != int_b1);
    assert_eq!((block0_disagreement, block1_disagreement), (0, 1));

    // F5b: Rounding half away from zero (`+3/2 -> +2`, `-3/2 -> -2`).
    assert_eq!(k1_next.export_int(3), 2);
    assert_eq!(km1_next.export_int(3), -2);

    // F5c: Determinism across repeated runs.
    let run_trajectory = || {
        let mut k = Rat::new(1, 1);
        for _ in 0..4 {
            let (_, g) = address_hinge_loss_and_subgrad(k, 14);
            k = k.sub(eta.mul(g)).clamp(-3, 3);
        }
        k
    };
    assert_eq!(run_trajectory(), run_trajectory());

    // F6: D29 banned-operation scanner passes clean subgradient source and rejects seeded `expf(`.
    let clean_kernel_stub = "void c1_step(int n, int k_max) { int y = u + k * m; }";
    assert!(scan_offline_float_module_tokens(clean_kernel_stub).is_empty());
    let seeded_bad = [ "int bad = ", concat!("ex", "pf"), "(x);" ].concat();
    assert_eq!(
        scan_offline_float_module_tokens(&seeded_bad),
        vec![concat!("ex", "pf").to_string()]
    );

    // F7: Exported integer snapshot passes discrete validation under `FlockCoupled`.
    let exported = fixture_t_prime(3, &[0, k1_next.export_int(3)]);
    assert_eq!(validate_coupled_fixture(&exported), Ok(()));
}
