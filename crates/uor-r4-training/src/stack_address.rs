//! The invariant address of a source position, and the route that uses it
//! ([`docs/mechanisms/invariant-addressed-read.md`]).
//!
//! The mechanism brief's first experiment routes the read to a small candidate
//! set by a **fixed** address instead of scoring the whole window, with a
//! **null control**: the same routing machinery with the address permuted, so
//! that a routing win cannot be credited to structure without evidence. This
//! module is the deterministic half of that: an address any position can be
//! given in O(1) with no learned parameters, a fixed bijection over the address
//! space for the null arm, and the route that selects the best-agreeing
//! positions.
//!
//! Nothing here reads the model: the address is a function of a token identity
//! and a coarse distance bucket, which is what makes it *invariant* — the same
//! object keeps its address wherever it appears, and the address can be
//! translated, compared and permuted.

/// The prime whose multiples scatter the token identity across the address
/// space. Fixed, and deliberately the smallest prime that is not a power of
/// two: the address must not be a bit slice of the identity.
use serde::{Deserialize, Serialize};

const ADDRESS_PRIME: u32 = 2_654_435_761;

/// The address of one source position: `token`'s identity scattered by
/// [`ADDRESS_PRIME`] and combined with its **family key**, the bucket of the
/// distance `back` (the position's age in tokens). `buckets` is the number of
/// distance buckets; `space` is the size of the address space (a power of two
/// in every configuration this lab runs, but not required to be).
///
/// O(1), deterministic, no learned parameters, and stable across processes: the
/// same token at the same age always lands on the same address.
pub fn position_address(token: u32, back: usize, buckets: u32, space: u32) -> u32 {
    if space == 0 || buckets == 0 {
        return 0;
    }
    let identity = token.wrapping_mul(ADDRESS_PRIME);
    let family = (back as u64 % u64::from(buckets)) as u32;
    // The family key rotates the identity by a bucket-dependent amount, so two
    // occurrences of one token at different ages are different addresses while
    // every occurrence at the same age agrees exactly.
    let rotation = family.wrapping_mul(ADDRESS_PRIME.rotate_left(13)) % space;
    identity.rotate_left(family % 32).wrapping_add(rotation) % space
}

/// The fixed bijection over `0..space` that the **null control** applies to
/// every address ([`permute_address`]'s inverse is itself with the inverse
/// step, so the mapping is a permutation rather than a scramble that could
/// collapse addresses onto each other).
///
/// `step` is forced odd and coprime to `space` when `space` is a power of two:
/// an affine map `x -> step * x + offset (mod space)` with `gcd(step, space) = 1`
/// is a bijection, and the test below pins exactly that.
pub fn permute_address(address: u32, space: u32, step: u32, offset: u32) -> u32 {
    if space == 0 {
        return 0;
    }
    address.wrapping_mul(step | 1).wrapping_add(offset) % space
}

/// The map that makes a permutation pair from a seed, so the null arm is a
/// fixed, recorded bijection rather than an unrecorded shuffle: returns
/// `(step, offset)` with `step` odd (hence coprime to a power-of-two space).
pub fn permutation_from_seed(seed: u64, space: u32) -> (u32, u32) {
    let mixed = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let step = ((mixed >> 32) as u32) | 1;
    let offset = (mixed as u32) % space.max(1);
    (step, offset)
}

/// The route: write into `out` the indices of the `k` positions whose addresses
/// agree with `query` **most closely**, best first, and return how many were
/// written. Agreement is the number of leading bits two addresses share
/// (equality first, then near-neighbours), with ties to the **earlier
/// position** so a route is deterministic.
///
/// `out` must hold at least `k` entries; positions beyond `k` are not written.
pub fn route(query: u32, addresses: &[u32], k: usize, out: &mut [usize]) -> usize {
    let k = k.min(out.len()).min(addresses.len());
    for slot in 0..k {
        let mut best: Option<usize> = None;
        for (index, &address) in addresses.iter().enumerate() {
            if out[..slot].contains(&index) {
                continue;
            }
            best = match best {
                None => Some(index),
                Some(current) => {
                    let agreed = agreement(query, address);
                    let held = agreement(query, addresses[current]);
                    if agreed > held || (agreed == held && index < current) {
                        Some(index)
                    } else {
                        Some(current)
                    }
                }
            };
        }
        match best {
            Some(index) => out[slot] = index,
            None => return slot,
        }
    }
    k
}

/// How many leading bits two addresses share, plus one: equality scores
/// `32 + 1`, so an exact match always outranks a near one.
fn agreement(a: u32, b: u32) -> u32 {
    (a ^ b).leading_zeros() + u32::from(a == b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_address_is_deterministic_and_o1() {
        for token in [0u32, 1, 7, 4095] {
            for back in [0usize, 1, 17, 1024] {
                let once = position_address(token, back, 8, 1 << 16);
                let twice = position_address(token, back, 8, 1 << 16);
                assert_eq!(once, twice);
                assert!(once < (1 << 16));
            }
        }
        // The same token at different ages is a different address; the same
        // token at the same age is the same address wherever it appears.
        let young = position_address(7, 0, 8, 1 << 16);
        let old = position_address(7, 700, 8, 1 << 16);
        assert_ne!(young, old, "the family key must separate ages");
        assert_eq!(old, position_address(7, 700, 8, 1 << 16));
    }

    #[test]
    fn the_null_control_is_a_bijection() {
        let space = 1u32 << 12;
        let (step, offset) = permutation_from_seed(20261012, space);
        assert_eq!(step % 2, 1, "the step must be odd");
        let mut seen = vec![false; space as usize];
        for address in 0..space {
            let mapped = permute_address(address, space, step, offset);
            assert!(mapped < space);
            assert!(!seen[mapped as usize], "the map is not injective");
            seen[mapped as usize] = true;
        }
        assert!(seen.iter().all(|&s| s), "the map is not surjective");
    }

    #[test]
    fn a_route_takes_exact_matches_first_and_ties_to_the_earlier_position() {
        let addresses = [0b1000u32, 0b1001, 0b0000, 0b1000];
        let mut out = [usize::MAX; 4];
        // The query equals positions 0 and 3: both exact, earlier first.
        let written = route(0b1000, &addresses, 2, &mut out);
        assert_eq!(written, 2);
        assert_eq!(&out[..2], &[0, 3]);
        // A query agreeing with nothing exactly still routes to near
        // neighbours, earliest first on ties.
        let written = route(0b1111, &addresses, 3, &mut out);
        assert_eq!(written, 3);
        assert_eq!(out[0], 0);
        // A `k` of zero writes nothing.
        assert_eq!(route(0b1000, &addresses, 0, &mut out), 0);
    }

    #[test]
    fn the_permuted_address_of_a_real_address_is_not_itself() {
        // The null arm must actually move addresses, or it would be the same
        // arm as the real one.
        let space = 1u32 << 12;
        let (step, offset) = permutation_from_seed(20261012, space);
        let moved = (0..space)
            .filter(|&a| permute_address(a, space, step, offset) != a)
            .count();
        assert!(
            moved > (space as usize) * 9 / 10,
            "the permutation barely moves"
        );
    }
}

// ---------------------------------------------------------------------------
// The tangent and normal movements of the null control
// ---------------------------------------------------------------------------

/// An address with its structure made explicit: the **cell** it occupies (the
/// coarse invariant — the family key that routes) and the **representative**
/// inside that cell (the position's fine coordinate). A null control that
/// permutes addresses destroys both, which tests only whether *any* address
/// works; the control the mechanism needs is a movement that keeps the cell and
/// moves the representative — a **tangent** movement, along the manifold rather
/// than off it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Address {
    /// The coarse invariant: unchanged by a tangent movement, and what routing
    /// compares.
    pub cell: u32,
    /// The fine coordinate inside the cell: what a tangent movement changes.
    pub representative: u32,
}

impl Address {
    /// The address of one source position as `(cell, representative)`:
    /// the cell is the family key (the distance bucket, which is the coarse
    /// invariant the read routes on) and the representative is the scattered
    /// token identity inside it.
    pub fn of(token: u32, back: usize, buckets: u32, cell_space: u32, inner_space: u32) -> Self {
        let cell = if buckets == 0 {
            0
        } else {
            (back as u64 % u64::from(buckets)) as u32 % cell_space.max(1)
        };
        Address {
            cell,
            representative: position_address(token, back, buckets, inner_space.max(1)),
        }
    }

    /// The **token-cell configuration** of the DeepSeek invariant-addressed
    /// read (the router slice): the cell is the token identity itself,
    /// `token % cells` — not the distance bucket [`Self::of`] uses — and the
    /// representative is the same O(1) position address. Two occurrences of
    /// one token share their cell whatever their age, so the tree routes a
    /// query token to other occurrences of that token.
    ///
    /// [`Self::of`] is untouched: it is the bucket-cell configuration the
    /// earlier slices and their tests are pinned to. This constructor is the
    /// second, identity-cell configuration, and it is what the gated router
    /// asks the tree about.
    pub fn of_ident(token: u32, back: usize, buckets: u32, cells: u32, inner: u32) -> Self {
        let cell = if cells == 0 { 0 } else { token % cells };
        Address {
            cell,
            representative: position_address(token, back, buckets, inner.max(1)),
        }
    }
    /// The packed form the router compares: cell first, representative behind
    /// it, so agreement is coarse-to-fine and a cell match always outranks a
    /// representative match.
    pub fn packed(&self, inner_bits: u32) -> u32 {
        (self.cell << inner_bits.min(31))
            | (self.representative & ((1u32 << inner_bits.min(31)) - 1))
    }

    /// A **tangent** movement: the same cell, a different representative. This
    /// is the null that matters — if the model does as well with the
    /// representative moved, it is using the cell (the geometry) and not an
    /// arbitrary fine coordinate; if it degrades, the fine coordinate is
    /// load-bearing and the address is a lookup rather than an invariant.
    pub fn tangent(&self, step: u32, inner_space: u32) -> Self {
        Address {
            cell: self.cell,
            representative: self
                .representative
                .wrapping_mul(step | 1)
                .wrapping_add(step.rotate_left(7))
                % inner_space.max(1),
        }
    }

    /// A **normal** movement: a different cell, so the coarse invariant itself
    /// changes. This is the coarser null (a permutation of the whole address
    /// space is the extreme case of it): it tests whether *any* structure
    /// works, and a mechanism that survives it but dies under [`Self::tangent`]
    /// is keying on an arbitrary representative rather than on the geometry.
    pub fn normal(&self, cell_space: u32, offset: u32) -> Self {
        Address {
            cell: self.cell.wrapping_add(offset | 1) % cell_space.max(1),
            representative: self.representative,
        }
    }
}

#[cfg(test)]
mod movement_tests {
    use super::*;

    #[test]
    fn a_tangent_movement_keeps_the_cell_and_moves_the_representative() {
        let inner = 1u32 << 12;
        let address = Address::of(11, 300, 8, 8, inner);
        let moved = address.tangent(0x9E37_79B9, inner);
        assert_eq!(
            moved.cell, address.cell,
            "a tangent movement must keep the cell"
        );
        assert_ne!(
            moved.representative, address.representative,
            "a tangent movement must actually move"
        );
        assert!(moved.representative < inner);
    }

    #[test]
    fn a_normal_movement_changes_the_cell_and_keeps_the_representative() {
        let address = Address::of(11, 300, 8, 8, 1 << 12);
        let moved = address.normal(8, 0x9E37_79B9);
        assert_ne!(
            moved.cell, address.cell,
            "a normal movement must leave the cell"
        );
        assert_eq!(moved.representative, address.representative);
    }

    #[test]
    fn packing_is_coarse_to_fine_so_a_cell_match_outranks_a_fine_one() {
        let inner_bits = 12;
        let a = Address {
            cell: 3,
            representative: 0,
        }
        .packed(inner_bits);
        let b = Address {
            cell: 2,
            representative: (1 << 12) - 1,
        }
        .packed(inner_bits);
        assert!(a > b, "the cell must dominate the packed order");
        // And the packed order equals the (cell, representative) order.
        assert_eq!(a, (3 << inner_bits) | 0);
    }
}

// ---------------------------------------------------------------------------
// The spanning tree the route walks (Matthew, Spiralcore: "layer-2 spanning tree")
// ---------------------------------------------------------------------------

/// A spanning tree over the cells. The cells are the layer-2 objects — the
/// coarse invariants a source position belongs to — and the tree is what turns
/// routing from a *search* into a *path*: two cells have exactly one path
/// between them, so addresses are canonical (one node, one path from the root),
/// comparable by shared ancestry, and the route is the path through the nearest
/// common ancestor rather than a top-k over the window.
///
/// Built deterministically by breadth-first traversal from `root`, with the
/// first edge that reaches a cell winning, so the tree depends only on the cell
/// count, the edge list and the root — all recorded with the run.
#[derive(Clone, Debug)]
pub struct CellTree {
    parent: Vec<u32>,
    depth: Vec<u32>,
    children: Vec<Vec<u32>>,
    root: u32,
    reached: usize,
}

impl CellTree {
    /// Build the tree over `cells` nodes from undirected `edges`, rooted at
    /// `root`. Unreached cells keep themselves as parent (a forest is allowed
    /// and reported through [`Self::reached`]); a tree needs every cell reached.
    pub fn build(cells: u32, edges: &[(u32, u32)], root: u32) -> Self {
        let n = cells as usize;
        let root = if root < cells { root } else { 0 };
        let mut adjacent: Vec<Vec<u32>> = vec![Vec::new(); n];
        for &(a, b) in edges {
            if a < cells && b < cells && a != b {
                adjacent[a as usize].push(b);
                adjacent[b as usize].push(a);
            }
        }
        let mut parent: Vec<u32> = (0..cells).collect();
        let mut depth = vec![u32::MAX; n];
        let mut children: Vec<Vec<u32>> = vec![Vec::new(); n];
        let mut queue = std::collections::VecDeque::new();
        depth[root as usize] = 0;
        queue.push_back(root);
        let mut reached = 0usize;
        while let Some(node) = queue.pop_front() {
            reached += 1;
            let mut next = adjacent[node as usize].clone();
            next.sort_unstable();
            for neighbour in next {
                if depth[neighbour as usize] == u32::MAX {
                    depth[neighbour as usize] = depth[node as usize] + 1;
                    parent[neighbour as usize] = node;
                    children[node as usize].push(neighbour);
                    queue.push_back(neighbour);
                }
            }
        }
        CellTree {
            parent,
            depth,
            children,
            root,
            reached,
        }
    }

    /// How many cells the traversal reached; a value below the cell count means
    /// the edge list does not span the cells and the tree is a forest.
    pub fn reached(&self) -> usize {
        self.reached
    }

    /// The root.
    pub fn root(&self) -> u32 {
        self.root
    }

    /// The canonical address of a cell: its **path from the root**, which is
    /// what makes re-rooting a re-encoding (Spiralcore's context translation)
    /// rather than a renumbering.
    pub fn path(&self, cell: u32) -> Vec<u32> {
        let mut out = Vec::new();
        if cell as usize >= self.parent.len() || self.depth[cell as usize] == u32::MAX {
            return out;
        }
        let mut node = cell;
        loop {
            out.push(node);
            if node == self.root {
                break;
            }
            node = self.parent[node as usize];
        }
        out.reverse();
        out
    }

    /// The nearest common ancestor of two cells — the node the route passes
    /// through, and the only one every path between them shares.
    pub fn common_ancestor(&self, a: u32, b: u32) -> Option<u32> {
        if a as usize >= self.parent.len() || b as usize >= self.parent.len() {
            return None;
        }
        if self.depth[a as usize] == u32::MAX || self.depth[b as usize] == u32::MAX {
            return None;
        }
        let (mut x, mut y) = (a, b);
        while self.depth[x as usize] > self.depth[y as usize] {
            x = self.parent[x as usize];
        }
        while self.depth[y as usize] > self.depth[x as usize] {
            y = self.parent[y as usize];
        }
        while x != y {
            x = self.parent[x as usize];
            y = self.parent[y as usize];
        }
        Some(x)
    }

    /// How many ancestors two cells share: prefix agreement on the tree, which
    /// replaces the hash agreement the first slice used.
    pub fn shared_ancestors(&self, a: u32, b: u32) -> usize {
        let (pa, pb) = (self.path(a), self.path(b));
        pa.iter().zip(pb.iter()).take_while(|(x, y)| x == y).count()
    }

    /// The route: write the indices of the source cells the read should look at,
    /// **nearest first** — the query's own cell, then its ancestors up to the
    /// root, then its children — and return how many were written. A cell that
    /// is on the query's path is at most one edge from it, so the candidate set
    /// is the adjacent subtree rather than the window, and its size is bounded
    /// by the branching factor times the depth rather than by the context.
    ///
    /// Cost: the walk is O((depth + branching) * window) per query row -- the
    /// path is visited once and each cell on it is scanned against the window's
    /// positions -- not O(1).
    pub fn route(&self, query: u32, candidates: &[u32], out: &mut [usize]) -> usize {
        if query as usize >= self.parent.len() {
            return 0;
        }
        let mut order: Vec<u32> = Vec::new();
        let mut node = Some(query);
        while let Some(current) = node {
            order.push(current);
            node = if current == self.root {
                None
            } else {
                Some(self.parent[current as usize])
            };
        }
        order.extend(self.children[query as usize].iter().copied());
        // order is the query's path (query -> root) plus its children: each
        // cell appears exactly once, so a candidate (one cell) matches at most
        // one cell and no duplicate check is needed. The linear contains scan
        // over the written prefix was O(k) per candidate, which made the walk
        // O(|order| * window * k); without it the walk is O(|order| * window),
        // still not O(1).
        let mut written = 0usize;
        for cell in order {
            for (index, &candidate) in candidates.iter().enumerate() {
                if candidate == cell {
                    if written == out.len() {
                        return written;
                    }
                    out[written] = index;
                    written += 1;
                }
            }
        }
        written
    }

    /// A **tangent** movement of a cell: stay at the node, change only the
    /// representative — which is [`Address::tangent`]'s job, so this is the
    /// identity on the tree and exists to make the attribution clause uniform
    /// (the tree does not move under a tangent movement; only the fine
    /// coordinate does).
    pub fn tangent_cell(&self, cell: u32) -> u32 {
        cell
    }

    /// A **normal** movement of a cell: leave the node — the first child, or the
    /// parent when there is none — so the cell, and therefore its path, changes.
    pub fn normal_cell(&self, cell: u32) -> u32 {
        if cell as usize >= self.parent.len() {
            return cell;
        }
        match self.children[cell as usize].first() {
            Some(&child) => child,
            None => self.parent[cell as usize],
        }
    }
}

#[cfg(test)]
mod tree_tests {
    use super::*;

    fn chain() -> CellTree {
        // 0 - 1 - 2 - 3, rooted at 0.
        CellTree::build(4, &[(0, 1), (1, 2), (2, 3)], 0)
    }

    #[test]
    fn the_tree_spans_and_paths_are_canonical() {
        let tree = chain();
        assert_eq!(tree.reached(), 4, "the edge list must span the cells");
        assert_eq!(tree.path(0), vec![0]);
        assert_eq!(tree.path(3), vec![0, 1, 2, 3]);
        // The same cell has one path, whatever order it is asked for.
        assert_eq!(tree.path(2), tree.path(2));
        // A forest is allowed and reported rather than hidden.
        let forest = CellTree::build(4, &[(0, 1)], 0);
        assert_eq!(forest.reached(), 2);
        assert!(forest.path(3).is_empty());
    }

    #[test]
    fn two_cells_have_exactly_one_path_so_there_are_no_ties() {
        let tree = chain();
        assert_eq!(tree.common_ancestor(3, 2), Some(2));
        assert_eq!(tree.common_ancestor(3, 1), Some(1));
        assert_eq!(tree.common_ancestor(3, 0), Some(0));
        assert_eq!(tree.common_ancestor(2, 2), Some(2));
        // Prefix agreement is shared ancestry, counted on the path.
        assert_eq!(tree.shared_ancestors(3, 2), 3); // 0,1,2
        assert_eq!(tree.shared_ancestors(3, 0), 1); // 0
        assert_eq!(tree.shared_ancestors(1, 3), 2); // 0,1
    }

    #[test]
    fn the_route_walks_the_adjacent_subtree_nearest_first() {
        let tree = chain();
        // Candidates out of order; the query is cell 2.
        let candidates = [3u32, 0, 2, 1];
        let mut out = [usize::MAX; 4];
        let written = tree.route(2, &candidates, &mut out);
        // Order: the query's cell (index 2), then its parent, then the root, then
        // its child — the path, not a score.
        assert_eq!(written, 4);
        assert_eq!(&out[..4], &[2, 3, 1, 0]);
        // A bounded `out` writes only what fits.
        let mut small = [usize::MAX; 2];
        assert_eq!(tree.route(2, &candidates, &mut small), 2);
        assert_eq!(&small[..2], &[2, 3]);
    }

    #[test]
    fn tangent_movements_stay_at_the_node_and_normal_movements_leave_it() {
        let tree = chain();
        assert_eq!(
            tree.tangent_cell(2),
            2,
            "a tangent movement must not move the node"
        );
        assert_ne!(
            tree.normal_cell(2),
            2,
            "a normal movement must leave the node"
        );
        assert_eq!(tree.normal_cell(2), 3, "the first child");
        assert_eq!(tree.normal_cell(3), 2, "a leaf falls back to its parent");
    }
}

// ---------------------------------------------------------------------------
// The routing decision, as a pure function
// ---------------------------------------------------------------------------

/// Which movement the null arm applies, if any.
///
/// The two movements test different things, and the distinction is the whole
/// point of the control (**owner correction: a null requires a tangent
/// movement**):
///
/// - [`RouteNull::Tangent`] moves the **representative** inside each cell and
///   leaves the cells alone. The **candidate set is unchanged**; what changes is
///   the fine coordinate the read scores. If the model does as well under it,
///   the read depends only on the cell — the geometry — and the representative
///   is arbitrary; if it degrades, that fine coordinate is load-bearing and the
///   address is a lookup rather than an invariant.
/// - [`RouteNull::Normal`] moves the **cells** themselves, so the candidate set
///   changes too. This is the coarser null: it destroys the structure and
///   bounds how much *any* addressing could buy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteNull {
    /// The real arm: no movement.
    #[default]
    None,
    /// Hold the cell, move the representative (the attribution clause).
    Tangent {
        /// The odd multiplier the movement applies to the representative.
        step: u32,
    },
    /// Change the cell (the scale clause).
    Normal {
        /// The odd offset the movement applies to the cell.
        offset: u32,
    },
}

/// The routing configuration of one run: the tree over the cells, the address
/// geometry, how many candidates to take, and which null arm is active. Every
/// field is recorded with the run, and the default is **no routing at all**, so
/// a run that does not ask for this mechanism executes the model it had.
///
/// Deserialization is tolerant: every field defaults, so a serialized form may
/// carry only what the run set — and the *absent* form is the default-off
/// state, which the read must reproduce bit for bit
/// ([`RouteConfig::default`]).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RouteConfig {
    /// Cells in the spanning tree.
    pub cells: u32,
    /// Undirected cell edges; the tree is built from these and the root below.
    pub edges: Vec<(u32, u32)>,
    /// The root of the spanning tree.
    pub root: u32,
    /// Distance buckets — the family key in the address.
    pub buckets: u32,
    /// Bits of the address the representative occupies (the rest is the cell).
    pub inner_bits: u32,
    /// How many candidates to route to.
    pub k: usize,
    /// Which null arm is active.
    pub null: RouteNull,
}

impl Default for RouteConfig {
    fn default() -> Self {
        // No cells, no edges, k = 0: the "off" state the read must reproduce
        // bit for bit when the mechanism is not asked for.
        RouteConfig {
            cells: 0,
            edges: Vec::new(),
            root: 0,
            buckets: 1,
            inner_bits: 12,
            k: 0,
            null: RouteNull::None,
        }
    }
}

impl RouteConfig {
    /// Whether routing is active at all. A run with `k = 0` or a single cell
    /// routes nowhere and must behave exactly as the unrouted model.
    pub fn active(&self) -> bool {
        self.k > 0 && self.cells > 1
    }

    /// A balanced binary spanning tree over the cells (node i's children are
    /// 2i + 1 and 2i + 2, root 0): depth ceil(log2 cells), so the route is
    /// O(log cells) per query rather than O(cells). The default tree the
    /// examples build when a run names only the cell count.
    pub fn balanced(cells: u32, buckets: u32, inner_bits: u32, k: usize, null: RouteNull) -> Self {
        let mut edges = Vec::new();
        for i in 0..cells {
            let left = 2 * i + 1;
            let right = 2 * i + 2;
            if left < cells {
                edges.push((i, left));
            }
            if right < cells {
                edges.push((i, right));
            }
        }
        RouteConfig {
            cells,
            edges,
            root: 0,
            buckets,
            inner_bits,
            k,
            null,
        }
    }

    /// Whether this is exactly the default-off state: what an absent
    /// `read_router` deserializes to, and the only form a saved configuration
    /// may omit. (An asked-for routing that happens to be inactive is still
    /// recorded, so the record of the run is exact.)
    pub fn is_default_off(&self) -> bool {
        self == &RouteConfig::default()
    }

    /// The tree this configuration describes.
    pub fn tree(&self) -> CellTree {
        CellTree::build(self.cells, &self.edges, self.root)
    }

    /// Structural validation of an **active** configuration (inactive ones
    /// route nowhere and need none): the cell count, the candidate count, the
    /// packed-address bit budget, the buckets and a spanning tree over every
    /// cell. This is a configuration check, not a model check — the model's
    /// own combinations (flocks, other read mechanisms) refuse the router at
    /// their setters.
    pub fn validate(&self) -> crate::Result<()> {
        if !self.active() {
            return Ok(());
        }
        let invalid = |message: String| crate::TrainingError::Invalid(message);
        if self.cells < 2 {
            return Err(invalid(format!(
                "a routed read needs at least two cells (got {})",
                self.cells
            )));
        }
        if self.k == 0 || self.k > 32 {
            return Err(invalid(format!(
                "a routed read keeps 1..=32 candidates per query (got {})",
                self.k
            )));
        }
        // The packed form is cell << inner_bits | representative: the cell
        // must fit above the representative's bits in one u32.
        if self.inner_bits == 0 || self.inner_bits > 20 {
            return Err(invalid(format!(
                "a routed read's representative is 1..=20 bits (got {})",
                self.inner_bits
            )));
        }
        if self.cells >= 2u64.checked_shl(32 - self.inner_bits).unwrap_or(0) as u32 {
            return Err(invalid(format!(
                "{} cells do not fit above {} inner bits in a packed address",
                self.cells, self.inner_bits
            )));
        }
        if self.buckets == 0 {
            return Err(invalid(
                "a routed read needs at least one distance bucket".to_string(),
            ));
        }
        if self.root >= self.cells {
            return Err(invalid(format!(
                "the tree's root {} is not a cell of {}",
                self.root, self.cells
            )));
        }
        let tree = self.tree();
        if tree.reached() != self.cells as usize {
            return Err(invalid(format!(
                "the edges must span every cell: reached {} of {}",
                tree.reached(),
                self.cells
            )));
        }
        Ok(())
    }
}

/// The identity cells of one window's tokens under `config` —
/// `token % cells` after the null movement's cell part. Causal by
/// construction: the cell of position `j` depends only on the token there, so
/// prefix `cells[..=t]` is the window every query position `t` routes over,
/// and nothing has to be recomputed as the window grows. Inactive
/// configurations return an empty vector.
pub fn row_cells(config: &RouteConfig, tokens: &[u32]) -> Vec<u32> {
    if !config.active() {
        return Vec::new();
    }
    tokens
        .iter()
        .map(|&token| source_address_ident(config, Source { token, back: 0 }).cell)
        .collect()
}

/// One source position as the router sees it: the token it carries and how far
/// back it is (its age, which is the family key).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Source {
    /// The token identity.
    pub token: u32,
    /// The position's age in tokens.
    pub back: usize,
}

/// The address of a source position under `config`, after the null movement (if
/// any). For [`RouteNull::Tangent`] the **cell is untouched** and only the
/// representative moves; for [`RouteNull::Normal`] the cell moves.
pub fn source_address(config: &RouteConfig, source: Source) -> Address {
    let inner = 1u32 << config.inner_bits.min(31);
    let address = Address::of(
        source.token,
        source.back,
        config.buckets,
        config.cells.max(1),
        inner,
    );
    match config.null {
        RouteNull::None => address,
        RouteNull::Tangent { step } => address.tangent(step, inner),
        RouteNull::Normal { offset } => address.normal(config.cells.max(1), offset),
    }
}

/// The address of a source position under this configuration's **identity-cell**
/// variant ([`Address::of_ident`], the DeepSeek invariant-addressed read's own
/// configuration), after the null movement (if any). Same rule as
/// [`source_address`], except the cell is `token % cells` rather than the
/// distance bucket, so the null arms move exactly what they move on the bucket
/// arm: [`RouteNull::Tangent`] keeps the token cell and moves the fine
/// coordinate; [`RouteNull::Normal`] moves the cell.
pub fn source_address_ident(config: &RouteConfig, source: Source) -> Address {
    let inner = 1u32 << config.inner_bits.min(31);
    let address = Address::of_ident(
        source.token,
        source.back,
        config.buckets,
        config.cells.max(1),
        inner,
    );
    match config.null {
        RouteNull::None => address,
        RouteNull::Tangent { step } => address.tangent(step, inner),
        RouteNull::Normal { offset } => address.normal(config.cells.max(1), offset),
    }
}

/// The packed addresses of the sources a gated read keeps, in the kept-list
/// order [`crate::flock::flock_select`] gives — the sink (position 0) first,
/// then the ascending local positions, then the routed candidates — passed to
/// the read's dynamics in that order so the kept values, scores and addresses
/// never have to be reordered. `sources[j]` is the window's `j`-th position;
/// `kept` are the kept indices of that window in exactly kept order (indices
/// outside `sources` are dropped).
pub fn kept_addresses_packed(config: &RouteConfig, sources: &[Source], kept: &[usize]) -> Vec<u32> {
    kept.iter()
        .filter(|&&j| j < sources.len())
        .map(|&j| source_address_ident(config, sources[j]).packed(config.inner_bits))
        .collect()
}
/// The routing decision: write the indices of the sources the read should look
/// at, nearest first, and return how many were written. Pure — it takes sources
/// and their ages and returns indices, so the read's own change is a call and
/// not a rewrite, and the decision can be tested without a model.
///
/// The **cells** decide the route (that is the coarse invariant); the
/// **representatives** travel with the candidates and are what the read scores,
/// which is why the tangent null changes the read's input without changing its
/// candidate set.
pub fn route_sources(
    config: &RouteConfig,
    query: Source,
    sources: &[Source],
    out: &mut [usize],
) -> usize {
    if !config.active() {
        return 0;
    }
    let tree = config.tree();
    let query_cell = source_address(config, query).cell;
    let cells: Vec<u32> = sources
        .iter()
        .map(|&source| source_address(config, source).cell)
        .collect();
    tree.route(query_cell, &cells, out)
}

/// The kept positions of one query row under the identity-cell configuration
/// (the router slice's token-modulo-cells address, see Address::of_ident): the
/// query's own position and the routed candidates on the spanning tree, at most
/// config.k positions, in a deterministic order (the query first, then the
/// nearest-cell candidates). When config.k covers the whole visible prefix
/// (k >= query + 1) every visible position is kept, so the routed read
/// reproduces the full read bit for bit -- routing is a bound, and a bound at
/// least as wide as the window routes nothing away. An inactive configuration
/// returns the query position alone (the caller never routes it).
/// tokens[j] is the token of visible position j (tokens.len() > query).
///
/// Cost: the tree walk is O((depth + branching) * window) per query row -- the
/// path is visited once and each cell on it is scanned against the window's
/// positions -- not O(1).
pub fn routed_keep(config: &RouteConfig, query: usize, tokens: &[u32]) -> Vec<usize> {
    if query >= tokens.len() {
        return Vec::new();
    }
    if config.k >= query + 1 {
        return (0..=query).collect();
    }
    let mut kept = vec![query];
    if !config.active() || config.k <= 1 {
        return kept;
    }
    let cells: Vec<u32> = tokens[..=query]
        .iter()
        .map(|&token| source_address_ident(config, Source { token, back: 0 }).cell)
        .collect();
    let tree = config.tree();
    let mut out = vec![usize::MAX; config.k - 1];
    let written = tree.route(cells[query], &cells, &mut out);
    for &j in &out[..written] {
        if j != query {
            kept.push(j);
        }
    }
    kept.truncate(config.k);
    kept
}

#[cfg(test)]
mod decision_tests {
    use super::*;

    fn config(null: RouteNull) -> RouteConfig {
        // A 4-cell chain, 0-1-2-3, rooted at 0, with k = 4.
        RouteConfig {
            cells: 4,
            edges: vec![(0, 1), (1, 2), (2, 3)],
            root: 0,
            buckets: 4,
            inner_bits: 12,
            k: 4,
            null,
        }
    }

    #[test]
    fn an_inactive_configuration_routes_nothing() {
        let off = RouteConfig::default();
        assert!(!off.active());
        let sources = [Source { token: 1, back: 0 }, Source { token: 2, back: 1 }];
        let mut out = [usize::MAX; 4];
        assert_eq!(route_sources(&off, sources[0], &sources, &mut out), 0);
        // And `k = 0` with cells present is still inactive.
        let mut k0 = config(RouteNull::None);
        k0.k = 0;
        assert!(!k0.active());
        assert_eq!(route_sources(&k0, sources[0], &sources, &mut out), 0);
    }

    #[test]
    fn the_route_follows_the_cells_and_returns_indices() {
        let cfg = config(RouteNull::None);
        // Ages chosen so the four sources land in four different cells.
        let sources = [
            Source { token: 5, back: 0 },
            Source { token: 5, back: 1 },
            Source { token: 5, back: 2 },
            Source { token: 5, back: 3 },
        ];
        let mut out = [usize::MAX; 4];
        let written = route_sources(&cfg, sources[0], &sources, &mut out);
        assert!(written >= 1);
        // The query's own cell is routed first.
        assert_eq!(out[0], 0);
        // Every written index is a real source, and none repeats.
        for &index in &out[..written] {
            assert!(index < sources.len());
            assert_eq!(out[..written].iter().filter(|&&i| i == index).count(), 1);
        }
    }

    #[test]
    fn a_tangent_null_keeps_the_candidate_set_and_a_normal_null_moves_it() {
        let sources = [
            Source { token: 5, back: 0 },
            Source { token: 5, back: 1 },
            Source { token: 5, back: 2 },
            Source { token: 5, back: 3 },
        ];
        let mut real = [usize::MAX; 4];
        let mut tangent = [usize::MAX; 4];
        let mut normal = [usize::MAX; 4];
        let real_written = route_sources(&config(RouteNull::None), sources[0], &sources, &mut real);
        let tangent_written = route_sources(
            &config(RouteNull::Tangent { step: 0x9E37_79B9 }),
            sources[0],
            &sources,
            &mut tangent,
        );
        let normal_written = route_sources(
            &config(RouteNull::Normal { offset: 1 }),
            sources[0],
            &sources,
            &mut normal,
        );
        // The tangent arm routes to the same candidates: it changes the fine
        // coordinate the read scores, not the cell the route compares.
        assert_eq!(real_written, tangent_written);
        assert_eq!(&real[..real_written], &tangent[..tangent_written]);
        // The tangent arm does move the representatives, which is what the read
        // scores - so the arm is not the real arm.
        let cfg_t = config(RouteNull::Tangent { step: 0x9E37_79B9 });
        assert_ne!(
            source_address(&config(RouteNull::None), sources[0]).representative,
            source_address(&cfg_t, sources[0]).representative
        );
        // The normal arm changes the cells, so the route itself may differ.
        let cfg_n = config(RouteNull::Normal { offset: 1 });
        assert_ne!(
            source_address(&config(RouteNull::None), sources[0]).cell,
            source_address(&cfg_n, sources[0]).cell
        );
        assert!(normal_written <= sources.len());
    }
}

#[cfg(test)]
mod ident_address_tests {
    use super::*;

    /// The plan's configuration, small: 4 cells, 4 distance buckets.
    fn ident_config(null: RouteNull) -> RouteConfig {
        RouteConfig {
            cells: 4,
            edges: vec![(0, 1), (1, 2), (2, 3)],
            root: 0,
            buckets: 4,
            inner_bits: 12,
            k: 4,
            null,
        }
    }

    #[test]
    fn the_identity_cell_is_the_token_modulo_the_cells() {
        // `cell = token % cells`, whatever the age.
        let inner = 1u32 << 12;
        for (token, back) in [(0u32, 0usize), (5, 300), (9, 1024), (4095, 17)] {
            let address = Address::of_ident(token, back, 8, 4, inner);
            assert_eq!(address.cell, token % 4);
            assert_eq!(
                address.representative,
                position_address(token, back, 8, inner),
                "the representative is the same position address as the bucket configuration"
            );
        }
        // Two occurrences of one token share a cell whatever their age; the
        // bucket configuration would separate them.
        let young = Address::of_ident(7, 0, 8, 4, inner);
        let old = Address::of_ident(7, 700, 8, 4, inner);
        assert_eq!(young.cell, old.cell);
        assert_ne!(
            young.representative, old.representative,
            "ages still split the fine coordinate"
        );
        // And a zero cell count degrades to cell 0 rather than panicking.
        assert_eq!(Address::of_ident(5, 0, 8, 0, inner).cell, 0);
    }

    #[test]
    fn the_ident_source_address_carries_the_null_movements() {
        let inner = 1u32 << 12;
        let source = Source {
            token: 11,
            back: 300,
        };
        let real = source_address_ident(&ident_config(RouteNull::None), source);
        assert_eq!(real, Address::of_ident(11, 300, 4, 4, inner));
        // A tangent movement keeps the token cell and moves the representative.
        let tangent = source_address_ident(
            &ident_config(RouteNull::Tangent { step: 0x9E37_79B9 }),
            source,
        );
        assert_eq!(tangent.cell, real.cell);
        assert_ne!(tangent.representative, real.representative);
        // A normal movement moves the cell.
        let normal = source_address_ident(&ident_config(RouteNull::Normal { offset: 1 }), source);
        assert_ne!(normal.cell, real.cell);
        assert_eq!(normal.representative, real.representative);
    }

    #[test]
    fn the_kept_addresses_follow_the_flock_kept_list_order() {
        // flock_select order: the sink (position 0) first, then the ascending
        // local positions, then the routed candidates — the caller passes that
        // exact order and the addresses come back one-for-one.
        let cfg = ident_config(RouteNull::None);
        let sources = [
            Source { token: 3, back: 4 }, // sink
            Source { token: 5, back: 3 },
            Source { token: 5, back: 2 }, // local
            Source { token: 9, back: 1 }, // local
            Source {
                token: 3,
                back: 500,
            }, // routed candidate, later in the window
        ];
        let kept = [0usize, 2, 3, 4]; // sink, window ascending, routed
        let packed = kept_addresses_packed(&cfg, &sources, &kept);
        assert_eq!(packed.len(), kept.len());
        for (slot, &j) in kept.iter().enumerate() {
            let want = source_address_ident(&cfg, sources[j]).packed(cfg.inner_bits);
            assert_eq!(
                packed[slot], want,
                "slot {slot} must carry kept index {j}'s address in kept order"
            );
        }
        // Out-of-window indices are dropped rather than panicking.
        let packed = kept_addresses_packed(&cfg, &sources, &[0, 99, 2]);
        assert_eq!(packed.len(), 2);
        // And the null arm carries through: a tangent movement changes the
        // packed fine coordinate, and nothing else.
        let moved = kept_addresses_packed(
            &ident_config(RouteNull::Tangent { step: 0x9E37_79B9 }),
            &sources,
            &kept,
        );
        assert_ne!(moved, packed);
    }

    #[test]
    fn the_route_config_serializes_and_an_absent_form_defaults_off() {
        // Round-trip a full configuration.
        let cfg = ident_config(RouteNull::Tangent { step: 0x9E37_79B9 });
        let text = serde_json::to_string(&cfg).unwrap();
        let read: RouteConfig = serde_json::from_str(&text).unwrap();
        let re = serde_json::to_string(&read).unwrap();
        assert_eq!(text, re, "a round trip must not change the serialized form");
        // An absent form is the default-off state: no cells, k = 0, no null.
        let off: RouteConfig = serde_json::from_str("{}").unwrap();
        assert!(!off.active());
        assert_eq!(off.cells, 0);
        assert_eq!(off.k, 0);
        assert_eq!(off.null, RouteNull::None);
        // The null arms have names.
        assert_eq!(
            serde_json::to_string(&RouteNull::Tangent { step: 3 }).unwrap(),
            "{\"tangent\":{\"step\":3}}"
        );
        let back: RouteNull = serde_json::from_str("{\"normal\":{\"offset\":1}}").unwrap();
        assert_eq!(back, RouteNull::Normal { offset: 1 });
    }
}

#[cfg(test)]
mod routed_keep_tests {
    use super::*;

    #[test]
    fn routed_keep_at_k_eq_window_keeps_every_visible_position() {
        // A balanced tree spans every cell; at k >= window the bound covers the
        // whole causal prefix, so the routed read reproduces the full read.
        let cfg = RouteConfig::balanced(4, 4, 12, 8, RouteNull::None);
        let tokens = [3u32, 7, 9, 15, 2, 8, 5, 11];
        for query in 0..tokens.len() {
            assert_eq!(
                routed_keep(&cfg, query, &tokens),
                (0..=query).collect::<Vec<usize>>()
            );
        }
    }

    #[test]
    fn routed_keep_tangent_holds_cells_and_normal_moves_them() {
        let real = RouteConfig::balanced(4, 4, 12, 2, RouteNull::None);
        let tangent = RouteConfig::balanced(4, 4, 12, 2, RouteNull::Tangent { step: 0x9E37_79B9 });
        let tokens = [0u32, 5, 9, 13, 17, 1, 6, 10, 14];
        for query in 4..tokens.len() {
            // A tangent null moves only the representative, never the cell, so
            // the kept set is identical to the real address.
            assert_eq!(
                routed_keep(&real, query, &tokens),
                routed_keep(&tangent, query, &tokens)
            );
        }
        // A normal null moves the cell, so the route may change; it must stay
        // within the visible prefix.
        let normal = RouteConfig::balanced(4, 4, 12, 2, RouteNull::Normal { offset: 1 });
        let kept = routed_keep(&normal, 8, &tokens);
        assert!(kept.iter().all(|&j| j < tokens.len()));
        assert!(!kept.is_empty());
    }

    #[test]
    fn a_balanced_tree_spans_every_cell() {
        let cfg = RouteConfig::balanced(256, 64, 12, 16, RouteNull::None);
        let tree = cfg.tree();
        assert_eq!(tree.reached(), 256);
        assert!(cfg.validate().is_ok());
    }
}
