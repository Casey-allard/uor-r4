use uor_r4_graph_format::GraphView;

/// A chain of patch epochs overlaying a base graph.
#[derive(Debug, Clone)]
pub struct PatchChain<'a> {
    base_graph: GraphView<'a>,
    patches: alloc::vec::Vec<GraphView<'a>>,
}

impl<'a> PatchChain<'a> {
    /// Creates a new patch chain from a validated base graph.
    pub fn new(base_graph: GraphView<'a>) -> Self {
        Self {
            base_graph,
            patches: alloc::vec::Vec::new(),
        }
    }

    /// Base graph view.
    pub fn base_graph(&self) -> &GraphView<'a> {
        &self.base_graph
    }

    /// Tries to append a patch epoch to the chain.
    ///
    /// Validates deterministic newest-valid precedence and rejects forks
    /// (the patch's parent CID must match the current chain tip). Total
    /// verdict: `None` when the patch is appended, `Some(reason)` when it is
    /// rejected.
    pub fn try_push_patch(&mut self, patch: GraphView<'a>) -> Option<&'static str> {
        let Some(parent_cid) = patch.patch_parent_cid() else {
            return Some("Patch has no PTCH section or parent CID");
        };

        let expected_parent = if let Some(last) = self.patches.last() {
            last.header().artifact_cid
        } else {
            self.base_graph.header().artifact_cid
        };

        // Chain validation: fork rejection unless compacted.
        if parent_cid != expected_parent {
            return Some("Fork rejection: parent CID does not match tip of the chain");
        }

        // Bounded layer limits: force compaction if we exceed 8 layers.
        if self.patches.len() >= 8 {
            return Some("Compaction required: chain length exceeds 8 uncompacted layers");
        }

        self.patches.push(patch);
        None
    }

    /// Checks whether a node ID is tombstoned by any patch in the chain.
    pub fn is_node_tombstoned(&self, id: u32) -> bool {
        if self.patches.is_empty() {
            return false;
        }
        // Newest-valid precedence (reverse order).
        for patch in self.patches.iter().rev() {
            for ts in patch.patch_tombstones() {
                if ts.kind == 0 && ts.id == id {
                    return true;
                }
            }
        }
        false
    }

    /// Checks whether an edge ID is tombstoned by any patch in the chain.
    pub fn is_edge_tombstoned(&self, id: u32) -> bool {
        if self.patches.is_empty() {
            return false;
        }
        // Newest-valid precedence (reverse order).
        for patch in self.patches.iter().rev() {
            for ts in patch.patch_tombstones() {
                if ts.kind == 1 && ts.id == id {
                    return true;
                }
            }
        }
        false
    }
}
