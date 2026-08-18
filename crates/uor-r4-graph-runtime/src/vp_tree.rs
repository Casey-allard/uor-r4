//! Deterministic exact nearest-neighbor search for ROUT signatures.
//!
//! A VP-tree is valid here only when every indexed point uses the same mask:
//! masked Hamming with a per-point mask is not a single metric and cannot be
//! safely pruned by triangle-inequality bounds. Artifacts with varying masks
//! therefore keep the engine's linear fallback.

use alloc::vec::Vec;

use uor_r4_graph_format::{GraphView, SectionId};

const NONE: u32 = u32::MAX;

/// The checked-in 363-node graph is faster with the compact linear scan.
/// Keep the exact tree for larger graphs, but avoid its measured small-graph
/// regression. This is a serving heuristic, not a universal crossover claim.
pub(crate) const MIN_ROUTE_INDEX_NODES: u32 = 512;

#[derive(Debug, Clone)]
struct Point {
    node_id: u32,
    prototype: Vec<u8>,
    radius: u32,
}

#[derive(Debug, Clone, Copy)]
struct TreeNode {
    point: u32,
    threshold: u32,
    left: u32,
    right: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct VpTree {
    points: Vec<Point>,
    nodes: Vec<TreeNode>,
    mask: Vec<u8>,
    max_radius: u32,
}

impl VpTree {
    /// Build an index when ROUT contains a single shared mask.
    pub(crate) fn from_graph(view: &GraphView<'_>) -> Option<Self> {
        let head = view.head()?;
        let signature_bytes = usize::from(head.signature_bytes());
        if signature_bytes == 0 {
            return None;
        }
        let rout = view.section(SectionId::ROUT)?;
        let mut mask: Option<Vec<u8>> = None;
        let mut points = Vec::new();

        for (node_id, node) in view.nodes().enumerate() {
            let node_id = node_id as u32;
            if node_id == 0 {
                continue;
            }
            let proto_start = (node.prototype_word_start as usize).checked_mul(8)?; // p4-allow(load-time): overflow-checked word-to-byte conversion during VP-tree construction; the query path uses shifts
            let mask_start = (node.mask_word_start as usize).checked_mul(8)?; // p4-allow(load-time): same load-time conversion as above
            let proto_end = proto_start.checked_add(signature_bytes)?;
            let mask_end = mask_start.checked_add(signature_bytes)?;
            let prototype = rout.get(proto_start..proto_end)?;
            let node_mask = rout.get(mask_start..mask_end)?;

            match &mask {
                Some(shared) if shared.as_slice() != node_mask => return None,
                None => mask = Some(node_mask.to_vec()),
                Some(_) => {}
            }

            let shared_mask = mask.as_deref()?;
            let masked_prototype = prototype
                .iter()
                .zip(shared_mask)
                .map(|(&byte, &bit_mask)| byte & bit_mask)
                .collect();
            points.push(Point {
                node_id,
                prototype: masked_prototype,
                radius: u32::from(node.radius.0).max(120),
            });
        }

        Self::from_points(points, mask?)
    }

    fn from_points(points: Vec<Point>, mask: Vec<u8>) -> Option<Self> {
        if points.len() < 2
            || points
                .iter()
                .any(|point| point.prototype.len() != mask.len())
        {
            return None;
        }
        let max_radius = points.iter().map(|point| point.radius).max().unwrap_or(0);
        let mut indices: Vec<usize> = (0..points.len()).collect();
        let mut nodes = Vec::with_capacity(points.len());
        Self::build(&mut indices, &points, &mask, &mut nodes);
        Some(Self {
            points,
            nodes,
            mask,
            max_radius,
        })
    }

    fn build(
        indices: &mut [usize],
        points: &[Point],
        mask: &[u8],
        nodes: &mut Vec<TreeNode>,
    ) -> u32 {
        let point = indices[0];
        let tree_index = nodes.len() as u32;
        nodes.push(TreeNode {
            point: point as u32,
            threshold: 0,
            left: NONE,
            right: NONE,
        });

        if indices.len() == 1 {
            return tree_index;
        }

        let mut distances: Vec<(usize, u32)> = indices[1..]
            .iter()
            .map(|&candidate| {
                (
                    candidate,
                    Self::distance(&points[point].prototype, &points[candidate].prototype, mask),
                )
            })
            .collect();
        distances.sort_unstable_by(|(left_id, left_distance), (right_id, right_distance)| {
            left_distance
                .cmp(right_distance)
                .then_with(|| points[*left_id].node_id.cmp(&points[*right_id].node_id))
        });

        let split = distances.len() / 2; // p4-allow(load-time): VP-tree construction split; query path is division-free
        let threshold = distances[split].1;
        for (slot, &(candidate, _)) in distances.iter().enumerate() {
            indices[slot + 1] = candidate;
        }
        let (left_indices, right_indices) = indices[1..].split_at_mut(split);
        let left = if left_indices.is_empty() {
            NONE
        } else {
            Self::build(left_indices, points, mask, nodes)
        };
        let right = if right_indices.is_empty() {
            NONE
        } else {
            Self::build(right_indices, points, mask, nodes)
        };
        nodes[tree_index as usize] = TreeNode {
            point: point as u32,
            threshold,
            left,
            right,
        };
        tree_index
    }

    fn distance(left: &[u8], right: &[u8], mask: &[u8]) -> u32 {
        left.iter()
            .zip(right)
            .zip(mask)
            .map(|((&a, &b), &bit_mask)| ((a ^ b) & bit_mask).count_ones())
            .sum()
    }

    fn insert_active(active: &mut [u32; 8], active_len: &mut usize, node_id: u32) {
        if active[..*active_len].contains(&node_id) {
            return;
        }
        let position = active[..*active_len]
            .iter()
            .position(|&existing| existing > node_id)
            .unwrap_or(*active_len);
        if position >= active.len() {
            return;
        }
        if *active_len < active.len() {
            *active_len += 1;
        }
        for index in (position + 1..*active_len).rev() {
            active[index] = active[index - 1];
        }
        active[position] = node_id;
    }

    fn search_node(
        &self,
        tree_index: u32,
        signature: &[u8],
        best_node: &mut u32,
        best_distance: &mut u32,
        active: &mut [u32; 8],
        active_len: &mut usize,
    ) {
        if tree_index == NONE {
            return;
        }
        let tree_node = self.nodes[tree_index as usize];
        let point = &self.points[tree_node.point as usize];
        let distance = Self::distance(signature, &point.prototype, &self.mask);
        if distance < *best_distance || (distance == *best_distance && point.node_id < *best_node) {
            *best_distance = distance;
            *best_node = point.node_id;
        }
        if distance <= point.radius {
            Self::insert_active(active, active_len, point.node_id);
        }

        let bound = (*best_distance).max(self.max_radius);
        if distance < tree_node.threshold {
            self.search_node(
                tree_node.left,
                signature,
                best_node,
                best_distance,
                active,
                active_len,
            );
            if distance.saturating_add(bound) >= tree_node.threshold {
                self.search_node(
                    tree_node.right,
                    signature,
                    best_node,
                    best_distance,
                    active,
                    active_len,
                );
            }
        } else {
            self.search_node(
                tree_node.right,
                signature,
                best_node,
                best_distance,
                active,
                active_len,
            );
            if distance.saturating_sub(bound) <= tree_node.threshold {
                self.search_node(
                    tree_node.left,
                    signature,
                    best_node,
                    best_distance,
                    active,
                    active_len,
                );
            }
        }
    }

    /// Return the exact nearest node and the first eight matching node IDs in
    /// canonical node-ID order. The matching set is filtered by each node's
    /// calibrated radius after the shared-metric tree search.
    pub(crate) fn query(&self, signature: &[u8], active: &mut [u32; 8]) -> (u32, u32, usize) {
        if signature.len() != self.mask.len() {
            return (0, u32::MAX, 0);
        }
        let mut best_node = 0;
        let mut best_distance = u32::MAX;
        let mut active_len = 0;
        self.search_node(
            0,
            signature,
            &mut best_node,
            &mut best_distance,
            active,
            &mut active_len,
        );
        (best_node, best_distance, active_len)
    }
}

#[cfg(test)]
#[allow(dead_code, unused_imports)]
mod tests {
    use super::{Point, VpTree};
    use alloc::vec;

    fn point(node_id: u32, value: u8, radius: u32) -> Point {
        Point {
            node_id,
            prototype: vec![value],
            radius,
        }
    }

    #[test]
    fn exact_nearest_and_radius_matches_are_deterministic() {
        let tree = VpTree::from_points(
            vec![
                point(1, 0b0000_0000, 2),
                point(2, 0b0000_0011, 2),
                point(3, 0b1111_0000, 2),
            ],
            vec![0xff],
        )
        .expect("tree");
        let mut active = [0u32; 8];
        let (node, distance, active_len) = tree.query(&[0b0000_0001], &mut active);
        assert_eq!((node, distance), (1, 1));
        assert_eq!(&active[..active_len], &[1, 2]);
    }

    #[test]
    fn ties_choose_smallest_node_id() {
        let tree = VpTree::from_points(
            vec![point(9, 0, 1), point(2, 2, 1), point(4, 4, 1)],
            vec![0xff],
        )
        .expect("tree");
        let mut active = [0u32; 8];
        let (node, distance, _) = tree.query(&[3], &mut active);
        assert_eq!((node, distance), (2, 1));
    }
}
