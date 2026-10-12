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
        assert!(moved > (space as usize) * 9 / 10, "the permutation barely moves");
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

    /// The packed form the router compares: cell first, representative behind
    /// it, so agreement is coarse-to-fine and a cell match always outranks a
    /// representative match.
    pub fn packed(&self, inner_bits: u32) -> u32 {
        (self.cell << inner_bits.min(31)) | (self.representative & ((1u32 << inner_bits.min(31)) - 1))
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
        assert_eq!(moved.cell, address.cell, "a tangent movement must keep the cell");
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
        assert_ne!(moved.cell, address.cell, "a normal movement must leave the cell");
        assert_eq!(moved.representative, address.representative);
    }

    #[test]
    fn packing_is_coarse_to_fine_so_a_cell_match_outranks_a_fine_one() {
        let inner_bits = 12;
        let a = Address { cell: 3, representative: 0 }.packed(inner_bits);
        let b = Address { cell: 2, representative: (1 << 12) - 1 }.packed(inner_bits);
        assert!(a > b, "the cell must dominate the packed order");
        // And the packed order equals the (cell, representative) order.
        assert_eq!(a, (3 << inner_bits) | 0);
    }
}
