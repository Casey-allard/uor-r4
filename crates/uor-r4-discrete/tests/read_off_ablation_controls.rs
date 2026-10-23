//! Finite integer host controls for item `r-read-off-vs-read-on` (#2885)
//! (ORDER `docs/loop/tasks/r-read-off-vs-read-on/antigravity-45/ORDER.md`).
//!
//! Mirrors `route_vector`, `flock_vector`, `snap`, `read_vector`, and the
//! prediction/admission prefix of `writer` in `src/discrete.cu` (`:18-325`)
//! against the compiled `OUT_DIR/geometry.rs` tables (`ROOTS`, `AXES`,
//! `COMPOSE`, `BFS`, `BRANCH`, `GENERATORS`, `IDENTITY`), and verifies:
//!   C1 (`route_read_on_preserved_bit_for_bit`)
//!   C2 (`route_read_off_equals_query_last_position_bypass`)
//!   C3 (`flock_zero_gates_is_exact_read_off_and_matches_route_bypass`)
//!   C4 (`flock_active_leaf_differs_from_zero_gates_when_context_shifts_consensus`)
//!   C5 (`mode_and_proposal_refusal_guards`)
//!   C6 (`geometric_permuted_read_control_breaks_route_alignment_without_leaving_group`)

mod geometry {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/geometry.rs"));
}

const ABSENT: u32 = u32::MAX;
const WIDTH: usize = 8;
const CELLS: usize = 256;
const EDGE_TYPES: usize = 5;
const MODE_ROUTE: u32 = 0;
const MODE_FLOCK: u32 = 1;
const MODE_RELATION: u32 = 2;
const GATE_KIND: u32 = 3;
const STATUS_ROUTE_BYPASS_MODE_REFUSED: u32 = 11;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
struct RowPlan {
    length: usize,
    nodes: usize,
    frontier: Vec<u32>,
    node_words: Vec<[u32; 4]>,
}

impl RowPlan {
    fn build(length: usize) -> Self {
        assert!((1..=243).contains(&length));
        let mut node_words: Vec<[u32; 4]> = Vec::new();
        let mut node_levels: Vec<usize> = Vec::new();
        let mut frontier: Vec<u32> = Vec::new();
        for atom in 0..length {
            frontier.push(node_words.len() as u32);
            node_words.push([ABSENT, ABSENT, ABSENT, atom as u32]);
            node_levels.push(0);
            loop {
                let n = frontier.len();
                if n < 3 {
                    break;
                }
                let children = [frontier[n - 3], frontier[n - 2], frontier[n - 1]];
                let lv = node_levels[children[0] as usize];
                if children.iter().any(|&c| node_levels[c as usize] != lv) {
                    break;
                }
                let parent = node_words.len() as u32;
                node_words.push([children[0], children[1], children[2], ABSENT]);
                node_levels.push(lv + 1);
                frontier.truncate(n - 3);
                frontier.push(parent);
            }
        }
        Self {
            length,
            nodes: node_words.len(),
            frontier,
            node_words,
        }
    }
}

#[derive(Clone, Debug)]
struct Vocab {
    blocks: usize,
    table: Vec<[i32; WIDTH]>,
}

impl Vocab {
    fn from_root_ids(blocks: usize, token_roots: &[Vec<u32>]) -> Self {
        let mut table = Vec::with_capacity(token_roots.len() * blocks);
        for tok in token_roots {
            assert_eq!(tok.len(), blocks);
            for &r in tok {
                table.push(root(r));
            }
        }
        Self { blocks, table }
    }

    fn token_count(&self) -> usize {
        self.table.len() / self.blocks
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WriterOut {
    prediction: u32,
    admitted_cells: Vec<u32>,
    token_rivals: u32,
    address_rivals: u32,
}

fn root(r: u32) -> [i32; WIDTH] {
    geometry::ROOTS[r as usize].map(i32::from)
}

/// `compose`, discrete.cu:19.
fn compose(a: u8, b: u8) -> u8 {
    geometry::COMPOSE[usize::from(a) * 64 + usize::from(b)]
}

/// `action`, discrete.cu:20-22.
fn action(a: u8, x: &[i32; WIDTH]) -> [i32; WIDTH] {
    let mut y = [0i32; WIDTH];
    for (d, out) in y.iter_mut().enumerate() {
        let axis = geometry::AXES[usize::from(a) * WIDTH + d];
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
fn table_block(v: &Vocab, token: u32, block: usize, mv: Move) -> [i32; WIDTH] {
    let at = token as usize * v.blocks + block;
    let base = &v.table[at];
    if mv.kind == 1 && mv.index as usize == at && mv.candidate != 0 {
        action(move_action(mv.candidate), base)
    } else {
        *base
    }
}

/// `input_block`, discrete.cu:29-34.
fn input_block(v: &Vocab, ids: &[u32], pos: usize, block: usize, mv: Move) -> [i32; WIDTH] {
    table_block(v, ids[pos], block, mv)
}

/// `root_cost`, discrete.cu:37-40.
fn root_cost(x: &[i32; WIDTH], r: u32) -> (u32, u32) {
    let target = root(r);
    let mut distance = 0u32;
    let mut hamming = 0u32;
    for d in 0..WIDTH {
        distance += x[d].abs_diff(target[d]);
        if x[d] != target[d] {
            hamming += 1;
        }
    }
    (distance, hamming)
}

/// `root_id`, discrete.cu:285-288.
fn root_id(x: &[i32; WIDTH]) -> u32 {
    for r in 0..16u32 {
        if *x == root(r) {
            return r;
        }
    }
    ABSENT
}

/// `snap`, discrete.cu:104-112.
fn snap(x: &[i32; WIDTH], n: u32) -> u32 {
    let mut best = ABSENT;
    let mut best_distance = ABSENT;
    let mut best_hamming = ABSENT;
    for r in 0..16u32 {
        let base = root(r);
        let mut distance = 0u32;
        let mut hamming = 0u32;
        for d in 0..WIDTH {
            let mut y = 0i32;
            for _ in 0..n {
                y += base[d];
            }
            distance += x[d].abs_diff(y);
            if x[d] != y {
                hamming += 1;
            }
        }
        if distance < best_distance || (distance == best_distance && hamming < best_hamming) {
            best = r;
            best_distance = distance;
            best_hamming = hamming;
        }
    }
    best
}

/// Baseline `route_vector`, discrete.cu:41-77.
fn route_vector_baseline(
    v: &Vocab,
    ids: &[u32],
    routes: &[u8],
    plan: &RowPlan,
    stages: usize,
    mv: Move,
) -> Option<Vec<[i32; WIDTH]>> {
    if plan.length == 0 || plan.frontier.is_empty() || plan.frontier.len() > 16 {
        return None;
    }
    let mut output = vec![[0i32; WIDTH]; v.blocks];
    for (block, out_block) in output.iter_mut().enumerate() {
        let query = input_block(v, ids, plan.length - 1, block, mv);
        let mut node = ABSENT;
        let mut current_root = 0u32;
        let mut total = geometry::IDENTITY;
        let mut reached_leaf = false;
        for stage in 0..stages {
            let parameter = block * stages + stage;
            let mut frame = *routes.get(parameter)?;
            if frame >= 64 {
                return None;
            }
            if mv.kind == 0 && mv.index as usize == parameter && mv.candidate != 0 {
                frame = compose(move_action(mv.candidate), frame);
            }
            let q = action(frame, &query);
            let count = if stage == 0 { plan.frontier.len() } else { 3 };
            let mut best = ABSENT;
            let mut best_distance = ABSENT;
            let mut best_hamming = ABSENT;
            let mut goal = 0u32;
            for port in 0..count {
                let r = if stage == 0 {
                    port as u32
                } else {
                    u32::from(geometry::BRANCH[current_root as usize * 3 + port])
                };
                let (distance, hamming) = root_cost(&q, r);
                if distance < best_distance || (distance == best_distance && hamming < best_hamming)
                {
                    best = port as u32;
                    best_distance = distance;
                    best_hamming = hamming;
                    goal = r;
                }
            }
            if best == ABSENT {
                return None;
            }
            let bfs = geometry::BFS[current_root as usize * 16 + goal as usize];
            total = compose(bfs, total);
            current_root = goal;
            node = if stage == 0 {
                plan.frontier[best as usize]
            } else {
                plan.node_words[node as usize][best as usize]
            };
            if node as usize >= plan.nodes {
                return None;
            }
            let atom = plan.node_words[node as usize][3];
            if atom != ABSENT {
                if atom as usize >= plan.length {
                    return None;
                }
                let carrier = input_block(v, ids, atom as usize, block, mv);
                *out_block = action(total, &carrier);
                reached_leaf = true;
                break;
            }
            if stage + 1 == stages {
                return None;
            }
        }
        if !reached_leaf {
            return None;
        }
    }
    Some(output)
}

/// `route_vector` with the `route_bypass` (`s[13]`) switch:
/// - `route_bypass == 0`: baseline `route_vector`.
/// - `route_bypass == 1`: returns last-position query blocks `pos = length - 1` directly.
fn route_vector_with_switch(
    v: &Vocab,
    ids: &[u32],
    routes: &[u8],
    plan: &RowPlan,
    stages: usize,
    route_bypass: u32,
    mv: Move,
) -> Result<Vec<[i32; WIDTH]>, u32> {
    if route_bypass > 1 {
        return Err(STATUS_ROUTE_BYPASS_MODE_REFUSED);
    }
    if route_bypass == 1 && mv.kind == 0 && mv.candidate != 0 {
        return Err(STATUS_ROUTE_BYPASS_MODE_REFUSED);
    }
    if plan.length == 0 || plan.frontier.is_empty() || plan.frontier.len() > 16 {
        return Err(1);
    }
    if route_bypass == 1 {
        let mut output = Vec::with_capacity(v.blocks);
        for block in 0..v.blocks {
            output.push(input_block(v, ids, plan.length - 1, block, mv));
        }
        return Ok(output);
    }
    route_vector_baseline(v, ids, routes, plan, stages, mv).ok_or(1)
}

fn gate_value(gates: &[i32], at: usize, mv: Move) -> i32 {
    let current = gates[at];
    if mv.kind != GATE_KIND || mv.index as usize != at || mv.candidate == 0 {
        return current;
    }
    let order = [0i32, -1, 1];
    let mut k = 0u32;
    for candidate_trit in order {
        if candidate_trit == current {
            continue;
        }
        k += 1;
        if k == mv.candidate {
            return candidate_trit;
        }
    }
    current
}

fn frame_block(b: usize, blocks: usize) -> usize {
    if b == 0 {
        blocks - 1
    } else {
        0
    }
}

/// `flock_vector`, discrete.cu:127-270 (for `MODE_FLOCK` and `MODE_RELATION`, default `s[12] == 0`).
#[allow(clippy::too_many_arguments)]
fn flock_vector_reference(
    v: &Vocab,
    ids: &[u32],
    gates: &[i32],
    plan: &RowPlan,
    stages: usize,
    mode: u32,
    kept: usize,
    sweeps: usize,
    route_bypass: u32,
    mv: Move,
) -> Result<(Vec<[i32; WIDTH]>, Vec<u32>), u32> {
    if route_bypass != 0 {
        return Err(STATUS_ROUTE_BYPASS_MODE_REFUSED);
    }
    if mode != MODE_FLOCK && mode != MODE_RELATION {
        return Err(STATUS_ROUTE_BYPASS_MODE_REFUSED);
    }
    if plan.length == 0 || kept == 0 || kept > 8 || v.blocks < 2 {
        return Err(1);
    }
    if mode == MODE_RELATION && (sweeps == 0 || sweeps > 5) {
        return Err(1);
    }
    let mut level = vec![0usize; plan.nodes];
    let mut first = vec![0usize; plan.nodes];
    let mut last = vec![0usize; plan.nodes];
    let mut frame = vec![0u32; plan.nodes];
    for node in 0..plan.nodes {
        let words = plan.node_words[node];
        let atom = words[3];
        if atom != ABSENT {
            if atom as usize >= plan.length {
                return Err(1);
            }
            level[node] = 0;
            first[node] = atom as usize;
            last[node] = atom as usize;
        } else {
            let c0 = words[0] as usize;
            let c1 = words[1] as usize;
            let c2 = words[2] as usize;
            if c0 >= node || c1 >= node || c2 >= node {
                return Err(1);
            }
            level[node] = level[c0] + 1;
            first[node] = first[c0];
            last[node] = last[c2];
        }
    }
    let mut output = vec![[0i32; WIDTH]; v.blocks];
    let mut terms_per_block = vec![0u32; v.blocks];
    for b in 0..v.blocks {
        let f = frame_block(b, v.blocks);
        let query_frame = input_block(v, ids, plan.length - 1, f, mv);
        let mut u = input_block(v, ids, plan.length - 1, b, mv);
        let fq = root_id(&query_frame);
        if fq == ABSENT {
            return Err(6);
        }
        let mut payload = vec![[0i32; WIDTH]; plan.nodes];
        for node in 0..plan.nodes {
            let words = plan.node_words[node];
            let atom = words[3];
            if atom != ABSENT {
                let fv = input_block(v, ids, atom as usize, f, mv);
                frame[node] = root_id(&fv);
                if frame[node] == ABSENT {
                    return Err(6);
                }
                payload[node] = input_block(v, ids, atom as usize, b, mv);
            } else {
                let c0 = words[0] as usize;
                frame[node] = frame[c0];
                let mut sum = [0i32; WIDTH];
                for k in 0..3 {
                    let child = words[k] as usize;
                    let act = geometry::BFS[frame[child] as usize * 16 + frame[c0] as usize];
                    let carried = action(act, &payload[child]);
                    for d in 0..WIDTH {
                        sum[d] += carried[d];
                    }
                }
                let r = snap(&sum, 3);
                payload[node] = root(r);
            }
        }
        let mut rel = vec![0i32; plan.nodes];
        if mode == MODE_RELATION {
            for rg in 0..(EDGE_TYPES * stages) {
                let gt = gate_value(gates, v.blocks * stages + rg, mv);
                if !(-1..=1).contains(&gt) {
                    return Err(1);
                }
            }
            let mut prev = vec![0i32; plan.nodes];
            for node in 0..plan.nodes {
                let (dist, _) = root_cost(&query_frame, frame[node]);
                prev[node] = 8 - dist as i32;
            }
            for _ in 0..sweeps {
                for i in 0..plan.nodes {
                    if last[i] >= plan.length {
                        continue;
                    }
                    let li = level[i];
                    for e in 0..EDGE_TYPES {
                        let gt = gate_value(gates, v.blocks * stages + li * EDGE_TYPES + e, mv);
                        if gt == 0 {
                            continue;
                        }
                        let mut esum = 0i32;
                        if e == 0 {
                            if plan.node_words[i][3] == ABSENT {
                                for c in 0..3 {
                                    let ch = plan.node_words[i][c] as usize;
                                    if last[ch] < plan.length {
                                        esum += prev[ch];
                                    }
                                }
                            }
                            for p in (i + 1)..plan.nodes {
                                let pw = plan.node_words[p];
                                if pw[3] == ABSENT
                                    && last[p] < plan.length
                                    && (pw[0] as usize == i
                                        || pw[1] as usize == i
                                        || pw[2] as usize == i)
                                {
                                    esum += prev[p];
                                }
                            }
                        } else if e == 1 {
                            for p in (i + 1)..plan.nodes {
                                let pw = plan.node_words[p];
                                if pw[3] != ABSENT || last[p] >= plan.length {
                                    continue;
                                }
                                let (c0, c1, c2) = (pw[0] as usize, pw[1] as usize, pw[2] as usize);
                                if i == c0 && last[c1] < plan.length {
                                    esum += prev[c1];
                                } else if i == c1 {
                                    if last[c0] < plan.length {
                                        esum += prev[c0];
                                    }
                                    if last[c2] < plan.length {
                                        esum += prev[c2];
                                    }
                                } else if i == c2 && last[c1] < plan.length {
                                    esum += prev[c1];
                                }
                            }
                        } else if e == 2 {
                            for j in 0..i {
                                if last[j] < plan.length
                                    && level[j] == li
                                    && last[j] + 1 == first[i]
                                {
                                    esum += prev[j];
                                }
                            }
                        } else if e == 3 {
                            if li == 0 {
                                let tid = ids[last[i]];
                                for j in 0..i {
                                    if last[j] < plan.length && level[j] == 0 && ids[last[j]] == tid
                                    {
                                        esum = prev[j];
                                    }
                                }
                            }
                        } else if e == 4 {
                            for j in 0..i {
                                if last[j] < plan.length && level[j] == li && frame[j] == frame[i] {
                                    esum += prev[j];
                                }
                            }
                        }
                        rel[i] += if gt > 0 { esum } else { -esum };
                    }
                }
                prev.clone_from(&rel);
            }
        }
        let mut terms = 1u32;
        for l in 0..stages {
            let gate = gate_value(gates, b * stages + l, mv);
            if !(-1..=1).contains(&gate) {
                return Err(1);
            }
            if gate == 0 {
                continue;
            }
            let mut top: Vec<usize> = Vec::with_capacity(kept);
            let mut key_d: Vec<u32> = Vec::with_capacity(kept);
            let mut key_h: Vec<u32> = Vec::with_capacity(kept);
            let mut key_r: Vec<i32> = Vec::with_capacity(kept);
            for node in 0..plan.nodes {
                if level[node] != l || last[node] + 1 >= plan.length {
                    continue;
                }
                let (distance, hamming) = root_cost(&query_frame, frame[node]);
                let rscore = rel[node];
                let mut at = top.len();
                while at > 0 {
                    let o = top[at - 1];
                    let better = rscore > key_r[at - 1]
                        || (rscore == key_r[at - 1]
                            && (distance < key_d[at - 1]
                                || (distance == key_d[at - 1]
                                    && (hamming < key_h[at - 1]
                                        || (hamming == key_h[at - 1]
                                            && (last[node] > last[o]
                                                || (last[node] == last[o] && node < o)))))));
                    if !better {
                        break;
                    }
                    at -= 1;
                }
                if at < kept {
                    top.insert(at, node);
                    key_d.insert(at, distance);
                    key_h.insert(at, hamming);
                    key_r.insert(at, rscore);
                    if top.len() > kept {
                        top.pop();
                        key_d.pop();
                        key_h.pop();
                        key_r.pop();
                    }
                }
            }
            if top.is_empty() {
                continue;
            }
            let mut sum = [0i32; WIDTH];
            for &node in &top {
                let a = geometry::BFS[frame[node] as usize * 16 + fq as usize];
                let carried = action(a, &payload[node]);
                for d in 0..WIDTH {
                    sum[d] += carried[d];
                }
            }
            let m = snap(&sum, top.len() as u32);
            let m_root = root(m);
            for d in 0..WIDTH {
                u[d] += if gate > 0 { m_root[d] } else { -m_root[d] };
            }
            terms += 1;
        }
        let out_root = snap(&u, terms);
        output[b] = root(out_root);
        terms_per_block[b] = terms;
    }
    Ok((output, terms_per_block))
}

/// `writer`, discrete.cu:277-360 (prediction, admitted cells, and rival counts).
fn writer_reference(
    q: &[[i32; WIDTH]],
    v: &Vocab,
    admitted_k: usize,
    target: u32,
    mv: Move,
) -> Option<WriterOut> {
    let blocks = v.blocks;
    assert_eq!(q.len(), blocks);
    let mut occupied = [false; CELLS];
    let mut tok_cells = vec![0u32; v.token_count()];
    for (tok, slot) in tok_cells.iter_mut().enumerate() {
        let first = table_block(v, tok as u32, 0, mv);
        let last = table_block(v, tok as u32, blocks - 1, mv);
        let a = root_id(&first);
        let b = root_id(&last);
        if a == ABSENT || b == ABSENT {
            return None;
        }
        let c = a * 16 + b;
        occupied[c as usize] = true;
        *slot = c;
    }
    let address_distance = |cell: u32| -> u32 {
        let (a, _) = root_cost(&q[0], cell / 16);
        let (b, _) = root_cost(&q[blocks - 1], cell % 16);
        a + b
    };
    let token_distance = |tok: u32| -> u32 {
        let mut sum = 0u32;
        for (b, qb) in q.iter().enumerate() {
            let tb = table_block(v, tok, b, mv);
            for d in 0..WIDTH {
                sum += qb[d].abs_diff(tb[d]);
            }
        }
        sum
    };
    let mut occupied_cells: Vec<(u32, u32)> = (0..CELLS as u32)
        .filter(|&c| occupied[c as usize])
        .map(|c| (address_distance(c), c))
        .collect();
    occupied_cells.sort_by_key(|&(dist, cell)| (dist, cell));
    let admitted_cells: Vec<u32> = occupied_cells
        .iter()
        .take(admitted_k)
        .map(|&(_, cell)| cell)
        .collect();
    let is_admitted = |cell: u32| admitted_cells.contains(&cell);
    let mut best_token = ABSENT;
    let mut best_dist = ABSENT;
    for tok in 0..v.token_count() as u32 {
        if !is_admitted(tok_cells[tok as usize]) {
            continue;
        }
        let d = token_distance(tok);
        if d < best_dist || (d == best_dist && tok < best_token) {
            best_dist = d;
            best_token = tok;
        }
    }
    if best_token == ABSENT {
        return None;
    }
    let target_cell = tok_cells[target as usize];
    let target_cell_dist = address_distance(target_cell);
    let mut address_rivals = 0u32;
    for &c in &admitted_cells {
        if c == target_cell {
            continue;
        }
        let d = address_distance(c);
        if d < target_cell_dist || (d == target_cell_dist && c < target_cell) {
            address_rivals += 1;
        }
    }
    let target_tok_dist = token_distance(target);
    let mut token_rivals = 0u32;
    for tok in 0..v.token_count() as u32 {
        if tok == target || !is_admitted(tok_cells[tok as usize]) {
            continue;
        }
        let d = token_distance(tok);
        if d < target_tok_dist || (d == target_tok_dist && tok < target) {
            token_rivals += 1;
        }
    }
    Some(WriterOut {
        prediction: best_token,
        admitted_cells,
        token_rivals,
        address_rivals,
    })
}

/// Deterministic geometric derangement of the 64 signed-axis actions in `G`:
/// `perm(a) = (5 * a + 1) mod 64` (since `gcd(5, 64) == 1` and `4 * a + 1 != 0 mod 64`).
fn permute_route_actions(routes: &[u8]) -> Vec<u8> {
    routes
        .iter()
        .map(|&a| ((usize::from(a) * 5 + 1) % 64) as u8)
        .collect()
}

fn sample_vocab_and_plan() -> (Vocab, RowPlan, Vec<u32>) {
    // 16 tokens over 4 blocks, each with a distinct outer-cell address `(r, (r * 3 + 1) % 16)`
    // and distinct interior blocks so all 16 cells and tokens are separated.
    let token_roots: Vec<Vec<u32>> = (0..16u32)
        .map(|t| vec![t, (t + 5) % 16, (t * 7 + 2) % 16, (t * 3 + 1) % 16])
        .collect();
    let vocab = Vocab::from_root_ids(4, &token_roots);
    // Length-9 causal prefix produces a complete 2-level ternary tree (13 nodes, 3 stages).
    let plan = RowPlan::build(9);
    let ids = vec![0, 3, 7, 2, 11, 5, 14, 9, 4];
    (vocab, plan, ids)
}

#[test]
fn route_read_on_preserved_bit_for_bit() {
    let (vocab, plan, ids) = sample_vocab_and_plan();
    let stages = 3usize;
    for action_seed in 0..64u8 {
        let routes: Vec<u8> = (0..(vocab.blocks * stages))
            .map(|i| (action_seed.wrapping_add((i * 11) as u8)) % 64)
            .collect();
        let baseline =
            route_vector_baseline(&vocab, &ids, &routes, &plan, stages, NO_MOVE).unwrap();
        let switched_on =
            route_vector_with_switch(&vocab, &ids, &routes, &plan, stages, 0, NO_MOVE).unwrap();
        assert_eq!(baseline, switched_on);
        let w_base = writer_reference(&baseline, &vocab, 4, 6, NO_MOVE).unwrap();
        let w_on = writer_reference(&switched_on, &vocab, 4, 6, NO_MOVE).unwrap();
        assert_eq!(w_base, w_on);
    }
}

#[test]
fn route_read_off_equals_query_last_position_bypass() {
    let (vocab, plan, ids) = sample_vocab_and_plan();
    let stages = 3usize;
    let last_token = ids[plan.length - 1];
    let expected_unmoved: Vec<[i32; WIDTH]> = (0..vocab.blocks)
        .map(|b| table_block(&vocab, last_token, b, NO_MOVE))
        .collect();

    // Across arbitrary route tables and earlier prefix tokens, `route_bypass == 1`
    // returns the exact last-position query blocks.
    for action_seed in [0u8, 7, 19, 42, 63] {
        let routes: Vec<u8> = (0..(vocab.blocks * stages))
            .map(|i| (action_seed.wrapping_add((i * 17) as u8)) % 64)
            .collect();
        let mut alt_ids = ids.clone();
        alt_ids[0] = (alt_ids[0] + u32::from(action_seed)) % 16;
        let bypassed =
            route_vector_with_switch(&vocab, &alt_ids, &routes, &plan, stages, 1, NO_MOVE).unwrap();
        assert_eq!(bypassed, expected_unmoved);
    }

    // Virtual VectorBlock moves (`kind == 1`) on the last token still apply under `route_bypass == 1`.
    let mv = Move {
        kind: 1,
        index: last_token * vocab.blocks as u32 + 2,
        candidate: 5,
    };
    let routes = vec![geometry::IDENTITY; vocab.blocks * stages];
    let bypassed_moved =
        route_vector_with_switch(&vocab, &ids, &routes, &plan, stages, 1, mv).unwrap();
    let expected_moved: Vec<[i32; WIDTH]> = (0..vocab.blocks)
        .map(|b| table_block(&vocab, last_token, b, mv))
        .collect();
    assert_eq!(bypassed_moved, expected_moved);
    assert_ne!(bypassed_moved[2], expected_unmoved[2]);
}

#[test]
fn flock_zero_gates_is_exact_read_off_and_matches_route_bypass() {
    // 1. Exhaustively prove `snap(root(r), 1) == r` across all 16 orbit roots.
    for r in 0..16u32 {
        assert_eq!(snap(&root(r), 1), r);
    }

    // 2. Prove `Flock` and `Relation` with `ZeroGatesV1` (`gates == 0`) match `RouteBypassV1`
    // bit-for-bit on read vectors, residual terms (`terms == 1`), and writer outputs.
    let (vocab, plan, ids) = sample_vocab_and_plan();
    let stages = 3usize;
    let routes = vec![13u8; vocab.blocks * stages];
    let zero_gates_flock = vec![0i32; vocab.blocks * stages];
    let zero_gates_relation = vec![0i32; vocab.blocks * stages + EDGE_TYPES * stages];

    let route_off =
        route_vector_with_switch(&vocab, &ids, &routes, &plan, stages, 1, NO_MOVE).unwrap();
    let (flock_off, flock_terms) = flock_vector_reference(
        &vocab,
        &ids,
        &zero_gates_flock,
        &plan,
        stages,
        MODE_FLOCK,
        8,
        0,
        0,
        NO_MOVE,
    )
    .unwrap();
    let (relation_off, relation_terms) = flock_vector_reference(
        &vocab,
        &ids,
        &zero_gates_relation,
        &plan,
        stages,
        MODE_RELATION,
        8,
        1,
        0,
        NO_MOVE,
    )
    .unwrap();

    assert_eq!(flock_terms, vec![1u32; vocab.blocks]);
    assert_eq!(relation_terms, vec![1u32; vocab.blocks]);
    assert_eq!(flock_off, route_off);
    assert_eq!(relation_off, route_off);

    let w_route_off = writer_reference(&route_off, &vocab, 4, 6, NO_MOVE).unwrap();
    let w_flock_off = writer_reference(&flock_off, &vocab, 4, 6, NO_MOVE).unwrap();
    let w_rel_off = writer_reference(&relation_off, &vocab, 4, 6, NO_MOVE).unwrap();
    assert_eq!(w_route_off, w_flock_off);
    assert_eq!(w_route_off, w_rel_off);
}

#[test]
fn flock_active_leaf_differs_from_zero_gates_when_context_shifts_consensus() {
    let (vocab, plan, ids) = sample_vocab_and_plan();
    let stages = 3usize;
    let zero_gates = vec![0i32; vocab.blocks * stages];
    // `ActiveLeafV1` sets level-0 combine gates (`b * stages + 0`) to `+1`.
    let mut active_leaf_gates = vec![0i32; vocab.blocks * stages];
    for b in 0..vocab.blocks {
        active_leaf_gates[b * stages] = 1;
    }

    let (flock_off, terms_off) = flock_vector_reference(
        &vocab,
        &ids,
        &zero_gates,
        &plan,
        stages,
        MODE_FLOCK,
        8,
        0,
        0,
        NO_MOVE,
    )
    .unwrap();
    let (flock_on, terms_on) = flock_vector_reference(
        &vocab,
        &ids,
        &active_leaf_gates,
        &plan,
        stages,
        MODE_FLOCK,
        8,
        0,
        0,
        NO_MOVE,
    )
    .unwrap();

    assert_eq!(terms_off, vec![1u32; vocab.blocks]);
    assert_eq!(terms_on, vec![2u32; vocab.blocks]);
    assert_ne!(flock_on, flock_off);

    let w_off = writer_reference(&flock_off, &vocab, 4, 6, NO_MOVE).unwrap();
    let w_on = writer_reference(&flock_on, &vocab, 4, 6, NO_MOVE).unwrap();
    assert_ne!(w_on.prediction, w_off.prediction);
}

#[test]
fn mode_and_proposal_refusal_guards() {
    let (vocab, plan, ids) = sample_vocab_and_plan();
    let stages = 3usize;
    let routes = vec![geometry::IDENTITY; vocab.blocks * stages];
    let gates = vec![0i32; vocab.blocks * stages + EDGE_TYPES * stages];

    // 1. Out-of-range `route_bypass > 1` in Route mode is refused with status 11.
    assert_eq!(
        route_vector_with_switch(&vocab, &ids, &routes, &plan, stages, 2, NO_MOVE),
        Err(STATUS_ROUTE_BYPASS_MODE_REFUSED)
    );

    // 2. Proposing a Route move (`kind == 0`, `candidate != 0`) while `route_bypass == 1` is refused.
    let route_move = Move {
        kind: 0,
        index: 0,
        candidate: 1,
    };
    assert_eq!(
        route_vector_with_switch(&vocab, &ids, &routes, &plan, stages, 1, route_move),
        Err(STATUS_ROUTE_BYPASS_MODE_REFUSED)
    );

    // 3. Setting `route_bypass != 0` in `MODE_FLOCK` or `MODE_RELATION` is refused with status 11.
    assert_eq!(
        flock_vector_reference(&vocab, &ids, &gates, &plan, stages, MODE_FLOCK, 8, 0, 1, NO_MOVE),
        Err(STATUS_ROUTE_BYPASS_MODE_REFUSED)
    );
    assert_eq!(
        flock_vector_reference(
            &vocab,
            &ids,
            &gates,
            &plan,
            stages,
            MODE_RELATION,
            8,
            1,
            1,
            NO_MOVE
        ),
        Err(STATUS_ROUTE_BYPASS_MODE_REFUSED)
    );
}

#[test]
fn geometric_permuted_read_control_breaks_route_alignment_without_leaving_group() {
    // Verify the odd-stride affine map `a -> (5 * a + 1) mod 64` is a bijection on `0..64`
    // with zero fixed points (a complete derangement of the 64 signed-axis group elements).
    let all_actions: Vec<u8> = (0..64u8).collect();
    let permuted = permute_route_actions(&all_actions);
    let mut sorted = permuted.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, all_actions);
    for (orig, &perm) in all_actions.iter().zip(&permuted) {
        assert_ne!(*orig, perm);
        assert!(perm < 64);
    }

    // Verify that applying `permute_route_actions` to a Route model changes the routed
    // carrier output and writer prediction while remaining strictly inside `G`.
    let (vocab, plan, ids) = sample_vocab_and_plan();
    let stages = 3usize;
    let routes = vec![geometry::IDENTITY; vocab.blocks * stages];
    let permuted_routes = permute_route_actions(&routes);

    let out_on =
        route_vector_with_switch(&vocab, &ids, &routes, &plan, stages, 0, NO_MOVE).unwrap();
    let out_perm =
        route_vector_with_switch(&vocab, &ids, &permuted_routes, &plan, stages, 0, NO_MOVE)
            .unwrap();
    let out_off =
        route_vector_with_switch(&vocab, &ids, &routes, &plan, stages, 1, NO_MOVE).unwrap();

    assert_ne!(out_on, out_perm);
    assert_ne!(out_perm, out_off);

    let w_on = writer_reference(&out_on, &vocab, 4, 6, NO_MOVE).unwrap();
    let w_perm = writer_reference(&out_perm, &vocab, 4, 6, NO_MOVE).unwrap();
    assert_ne!(w_on.prediction, w_perm.prediction);
}
