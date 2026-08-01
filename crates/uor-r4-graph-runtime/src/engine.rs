use crate::runtime_state::RuntimeState;
use crate::runtime_state::SemanticStateSlot;
use crate::status::ResolutionStatus;
use crate::vp_tree::VpTree;
use core::fmt;
use uor_r4_graph_format::ScoreQ;
use uor_r4_graph_format::{CODE_OP_HALT, OP_CLEAR_SLOT, OP_SHIFT_SLOTS, OP_UPDATE_SLOT};
use uor_r4_graph_format::{FormatError, GraphView, SectionId};

/// Errors during graph step execution.
#[derive(Debug)]
pub enum RuntimeError {
    Format(FormatError),
    InvalidNode,
    Patch(alloc::borrow::Cow<'static, str>),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Format(e) => write!(f, "Format error: {:?}", e),
            Self::InvalidNode => write!(f, "Invalid node reference in graph"),
            Self::Patch(msg) => write!(f, "Patch error: {}", msg),
        }
    }
}

/// Multiplication-free zero-allocation prediction runtime wrapping an R4G1 borrowed `PatchChain`.
///
/// **Accuracy status (issue #280):** on `convert_r4g1`-produced
/// artifacts this runtime has no readable per-node emission structure
/// to walk (the converter is a container-format demo), and its readout
/// measured 0.0% held-out top1/agreement. It is not part of the
/// serving path (`uor-r4-api::engine::R4Engine` is) and is excluded
/// from the certify measurement matrix; treat outputs on such
/// artifacts as exercising format traversal, not prediction.
#[derive(Debug, Clone)]
pub struct R4G1Runtime<'a> {
    chain: crate::patch_chain::PatchChain<'a>,
    route_index: Option<VpTree>,
}

fn signature_affinity_bonus(prototype: &[u8], mask: &[u8], signature: &[u8]) -> i32 {
    let mut distance = 0u32;
    for ((&prototype_byte, &mask_byte), &signature_byte) in
        prototype.iter().zip(mask).zip(signature)
    {
        distance += ((signature_byte ^ prototype_byte) & mask_byte).count_ones();
    }
    let x = 288i32.saturating_sub(distance as i32);
    // x * 10 == (x << 3) + (x << 1), preserving the integer-only kernel.
    (x << 3).saturating_add(x << 1)
}

impl<'a> R4G1Runtime<'a> {
    /// Create a new R4G1 runtime by running two-stage validation over `bytes`.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, FormatError> {
        let view = GraphView::parse(bytes)?;
        Ok(Self {
            chain: crate::patch_chain::PatchChain::new(view),
            route_index: VpTree::from_graph(&view),
        })
    }

    /// Appends a patch epoch to the runtime's chain.
    pub fn try_push_patch(&mut self, patch_bytes: &'a [u8]) -> Result<(), RuntimeError> {
        let view = GraphView::parse(patch_bytes).map_err(RuntimeError::Format)?;
        self.chain
            .try_push_patch(view)
            .map_err(|e| RuntimeError::Patch(alloc::borrow::Cow::Borrowed(e)))
    }

    pub fn view(&self) -> &GraphView<'a> {
        self.chain.base_graph()
    }

    pub fn node_count(&self) -> u32 {
        self.chain.base_graph().node_count().unwrap_or(0)
    }

    pub fn edge_count(&self) -> u32 {
        self.chain.base_graph().edge_count().unwrap_or(0)
    }

    /// Single deterministic, allocation-free step of the graph runtime.
    pub fn step(
        &self,
        state: &mut RuntimeState,
        token: u32,
        _witness: &mut [u8],
    ) -> Result<(u32, ResolutionStatus), RuntimeError> {
        state.record_token(token);

        let num_nodes = self.node_count();
        if num_nodes == 0 {
            return Ok((0, ResolutionStatus::BackedOff));
        }

        let context = state.token().as_slice();
        let mut node_scores = [ScoreQ::MIN; 64];
        let (pred_token, score) = self.predict_distribution(context, None, &mut node_scores);

        let status = if score.raw() > 50_000 {
            ResolutionStatus::Supported
        } else if score.raw() > 0 {
            ResolutionStatus::Boundary
        } else {
            ResolutionStatus::BackedOff
        };

        // Phase 8: Execute CODE section state update programs.
        if let Some(code_bytes) = self.view().section(SectionId::CODE) {
            Self::execute_state_updates(state, code_bytes);
        }

        Ok((pred_token, status))
    }

    /// Execute the bytecode program in the CODE section to update semantic states.
    fn execute_state_updates(state: &mut RuntimeState, code_bytes: &[u8]) {
        let mut cursor = 0;
        while cursor < code_bytes.len() {
            let opcode = code_bytes[cursor];
            match opcode {
                CODE_OP_HALT => break,
                OP_UPDATE_SLOT => {
                    if cursor + 16 > code_bytes.len() {
                        break;
                    }
                    let level = code_bytes[cursor + 1];
                    let region_id = u32::from_le_bytes([
                        code_bytes[cursor + 2],
                        code_bytes[cursor + 3],
                        code_bytes[cursor + 4],
                        code_bytes[cursor + 5],
                    ]);
                    let token = u32::from_le_bytes([
                        code_bytes[cursor + 6],
                        code_bytes[cursor + 7],
                        code_bytes[cursor + 8],
                        code_bytes[cursor + 9],
                    ]);
                    let score_q = ScoreQ::from_raw(i32::from_le_bytes([
                        code_bytes[cursor + 10],
                        code_bytes[cursor + 11],
                        code_bytes[cursor + 12],
                        code_bytes[cursor + 13],
                    ]));
                    let age =
                        u16::from_le_bytes([code_bytes[cursor + 14], code_bytes[cursor + 15]]);

                    let slot = SemanticStateSlot {
                        region_id,
                        token,
                        score_q,
                        age,
                    };
                    match level {
                        0 => state.local_mut().update_slot(slot),
                        1 => state.segment_mut().update_slot(slot),
                        2 => state.session_mut().update_slot(slot),
                        _ => {}
                    }
                    cursor += 16;
                }
                OP_CLEAR_SLOT => {
                    if cursor + 2 > code_bytes.len() {
                        break;
                    }
                    let level = code_bytes[cursor + 1];
                    match level {
                        0 => state.local_mut().clear(),
                        1 => state.segment_mut().clear(),
                        2 => state.session_mut().clear(),
                        _ => {}
                    }
                    cursor += 2;
                }
                OP_SHIFT_SLOTS => {
                    if cursor + 2 > code_bytes.len() {
                        break;
                    }
                    let level = code_bytes[cursor + 1];
                    match level {
                        0 => state.local_mut().shift_slots(),
                        1 => state.segment_mut().shift_slots(),
                        2 => state.session_mut().shift_slots(),
                        _ => {}
                    }
                    cursor += 2;
                }
                _ => break, // Unknown opcode, halt execution
            }
        }
    }
}

fn check_node_emits(
    base_graph: &uor_r4_graph_format::GraphView,
    node_id: u32,
    target_token: u32,
    emit_remainder: Option<&[u8]>,
    exct_remainder: Option<&[u8]>,
) -> (bool, ScoreQ) {
    let mut stack = [0u32; 128];
    let mut visited = [0u32; 128];
    let mut stack_len = 1usize;
    let mut visited_len = 1usize;
    stack[0] = node_id;
    visited[0] = node_id;

    while stack_len > 0 {
        stack_len -= 1;
        let current_id = stack[stack_len];
        let node = match base_graph.node(current_id) {
            Some(n) => n,
            None => continue,
        };

        if let Some(emit_bytes) = emit_remainder {
            let start = node.emission_start as usize;
            let len = node.emission_len as usize;
            if len > 0 && start + len <= emit_bytes.len() {
                let sl = &emit_bytes[start..start + len];
                for i in 0..(sl.len() >> 3) {
                    let offset = i << 3;
                    let cand = u32::from_le_bytes([
                        sl[offset],
                        sl[offset + 1],
                        sl[offset + 2],
                        sl[offset + 3],
                    ]);
                    if cand == target_token {
                        let raw = i32::from_le_bytes([
                            sl[offset + 4],
                            sl[offset + 5],
                            sl[offset + 6],
                            sl[offset + 7],
                        ]);
                        return (
                            true,
                            if raw > 0 {
                                ScoreQ::from_raw(raw)
                            } else {
                                ScoreQ::from_raw(1)
                            },
                        );
                    }
                }
            }
        }

        if node.child_len > 0 {
            let start = node.child_start as usize;
            let count = (node.child_len as usize).min(16);
            for i in (0..count).rev() {
                if let Some(edge) = base_graph.edge((start + i) as u32) {
                    let dst = edge.dst.0;
                    if stack_len < stack.len()
                        && visited_len < visited.len()
                        && !visited[..visited_len].contains(&dst)
                    {
                        stack[stack_len] = dst;
                        stack_len += 1;
                        visited[visited_len] = dst;
                        visited_len += 1;
                    }
                }
            }
        }
    }

    if let Some(exct_bytes) = exct_remainder {
        for i in 0..(exct_bytes.len() >> 3) {
            let offset = i << 3;
            let cand = u32::from_le_bytes([
                exct_bytes[offset],
                exct_bytes[offset + 1],
                exct_bytes[offset + 2],
                exct_bytes[offset + 3],
            ]);
            if cand == target_token {
                let raw = i32::from_le_bytes([
                    exct_bytes[offset + 4],
                    exct_bytes[offset + 5],
                    exct_bytes[offset + 6],
                    exct_bytes[offset + 7],
                ]);
                return (
                    true,
                    if raw > 0 {
                        ScoreQ::from_raw(raw)
                    } else {
                        ScoreQ::from_raw(1)
                    },
                );
            }
        }
    }

    (false, ScoreQ::ZERO)
}

fn syntactic_morphism_score(prev_token: u32, cand_token: u32, tokens_since_period: usize) -> i32 {
    let mut score = 0i32;

    let endo_op =
        crate::cayley_dickson::EndomorphismOperator::from_token_transition(prev_token, cand_token);
    let state = crate::cayley_dickson::CayleyDicksonVector::from_u32(cand_token);
    // x * 5 == (x << 2) + x (shift/add only, no multiply on the hot path).
    let cs = endo_op.centralizer_score(&state);
    let cd_score = (cs << 2).saturating_add(cs);
    score += cd_score;

    let is_cand_period = cand_token == 29889 || cand_token == 13 || cand_token == 2;
    let is_cand_comma = cand_token == 29892 || cand_token == 11;
    let is_prev_period = prev_token == 29889 || prev_token == 13 || prev_token == 2;
    let is_prev_comma = prev_token == 29892 || prev_token == 11;

    let is_prev_prep = matches!(
        prev_token,
        304 | 311 | 310 | 315 | 297 | 393 | 449 | 322 | 323 | 363 | 368 | 378 | 527 | 550
    );
    let is_cand_prep = matches!(
        cand_token,
        304 | 311 | 310 | 315 | 297 | 393 | 449 | 322 | 323 | 363 | 368 | 378 | 527 | 550
    );

    let is_prev_det = matches!(
        prev_token,
        278 | 262
            | 263
            | 257
            | 385
            | 459
            | 1357
            | 856
            | 860
            | 1079
            | 1072
            | 1189
            | 415
            | 264
            | 407
            | 450
            | 414
            | 1722
    );
    let is_cand_det = matches!(
        cand_token,
        278 | 262
            | 263
            | 257
            | 385
            | 459
            | 1357
            | 856
            | 860
            | 1079
            | 1072
            | 1189
            | 415
            | 264
            | 407
            | 450
            | 414
            | 1722
    );

    let is_prev_noun = matches!(prev_token, 638 | 3108 | 7695 | 1211);

    if is_prev_noun {
        if is_cand_period || is_cand_comma {
            score -= 1200;
        } else if cand_token == 7695 || cand_token == 471 || cand_token == 18012 {
            score += 600;
        }
    }

    if is_prev_period {
        if is_cand_period || is_cand_comma || is_cand_prep {
            score -= 1000;
        } else if is_cand_det || cand_token == 7695 || cand_token == 1211 || cand_token == 3108 {
            score += 400;
        }
    }

    if is_prev_comma {
        if is_cand_period || is_cand_comma || is_cand_prep {
            score -= 900;
        } else if is_cand_det || cand_token == 7695 || cand_token == 1211 {
            score += 350;
        } else {
            score += 100;
        }
    }

    if is_prev_prep {
        if is_cand_prep || is_cand_period || is_cand_comma {
            score -= 800;
        } else if is_cand_det {
            score += 300;
        }
    }

    if is_prev_det {
        if is_cand_period || is_cand_comma || is_cand_prep || is_cand_det {
            score -= 800;
        } else {
            score += 200;
        }
    }

    if tokens_since_period < 8 {
        if is_cand_period {
            score -= 2500;
        } else if is_cand_comma && tokens_since_period < 4 {
            score -= 1500;
        }
    } else if (8..=20).contains(&tokens_since_period) && is_cand_period {
        score += 350;
    }

    score
}

fn collect_target_leaf_nodes<'a>(
    base_graph: &GraphView<'a>,
    start_id: u32,
    out: &mut [u32; 128],
    out_len: &mut usize,
) {
    if *out_len >= out.len() {
        return;
    }

    let mut stack = [0u32; 256];
    let mut visited = [0u32; 256];
    let mut stack_len = 1usize;
    let mut visited_len = 1usize;
    stack[0] = start_id;
    visited[0] = start_id;

    while stack_len > 0 && *out_len < out.len() {
        stack_len -= 1;
        let node_id = stack[stack_len];
        let Some(node) = base_graph.node(node_id) else {
            continue;
        };

        if (node.emission_len > 0 || node_id == 0) && !out[..*out_len].contains(&node_id) {
            out[*out_len] = node_id;
            *out_len += 1;
            if *out_len >= out.len() {
                break;
            }
        }

        if node.child_len > 0 {
            let start = node.child_start as usize;
            let count = node.child_len as usize;
            for i in (0..count).rev() {
                if let Some(edge) = base_graph.edge((start + i) as u32) {
                    let dst = edge.dst.0;
                    if stack_len < stack.len()
                        && visited_len < visited.len()
                        && !visited[..visited_len].contains(&dst)
                    {
                        stack[stack_len] = dst;
                        stack_len += 1;
                        visited[visited_len] = dst;
                        visited_len += 1;
                    }
                }
            }
        }
    }
}

impl<'a> R4G1Runtime<'a> {
    pub fn predict_token(
        &self,
        context_tokens: &[u32],
        signature: Option<&[u8]>,
        node_scores: &mut [ScoreQ],
    ) -> u32 {
        let (token, _) = self.predict_distribution(context_tokens, signature, node_scores);
        token
    }

    pub fn predict_distribution(
        &self,
        context_tokens: &[u32],
        signature: Option<&[u8]>,
        _node_scores: &mut [ScoreQ],
    ) -> (u32, ScoreQ) {
        self.predict_distribution_with_signature_lanes(
            context_tokens,
            signature,
            None,
            _node_scores,
        )
    }

    /// Predict with separate context and session signatures.
    ///
    /// The context signature remains the only input to ROUT fallback, so
    /// enabling the session lane starts as a bias-only experiment. The
    /// optional session signature participates in the existing emission
    /// affinity bonus below; callers can later opt it into ROUT after an
    /// evaluation establishes that the fallback prototypes are calibrated
    /// for session-space inputs.
    pub fn predict_distribution_with_signature_lanes(
        &self,
        context_tokens: &[u32],
        context_signature: Option<&[u8]>,
        session_signature: Option<&[u8]>,
        _node_scores: &mut [ScoreQ],
    ) -> (u32, ScoreQ) {
        let num_nodes = self.node_count();
        if num_nodes == 0 || context_tokens.is_empty() {
            return (0, ScoreQ::ZERO);
        }

        let base_graph = self.chain.base_graph();
        let emit_remainder = base_graph
            .section(SectionId::EMIT)
            .and_then(|b| if b.len() >= 4 { Some(&b[4..]) } else { None });
        let exct_remainder = base_graph
            .section(SectionId::EXCT)
            .and_then(|b| if b.len() >= 12 { Some(&b[4..]) } else { None });

        let mut active_nodes = [0u32; 64];
        let mut active_len = 0usize;

        let tokens_slice = if !context_tokens.is_empty() && context_tokens[0] <= 1 {
            &context_tokens[1..]
        } else {
            context_tokens
        };
        if tokens_slice.is_empty() {
            return (0, ScoreQ::ZERO);
        }

        let max_suffix = core::cmp::min(10, tokens_slice.len());

        for suffix_len in (1..=max_suffix).rev() {
            let suffix = &tokens_slice[tokens_slice.len() - suffix_len..];

            let mut current = [0u32; 64];
            let mut current_len = 0usize;
            for n in 0..num_nodes {
                if check_node_emits(base_graph, n, suffix[0], emit_remainder, exct_remainder).0
                    && current_len < 64
                {
                    current[current_len] = n;
                    current_len += 1;
                }
            }

            let mut failed = false;
            for &t in &suffix[1..] {
                let mut next_current = [0u32; 64];
                let mut next_len = 0usize;
                for &node_id in &current[..current_len] {
                    if let Some(node) = base_graph.node(node_id) {
                        let start = node.forward_start;
                        let len = node.forward_len as u32;
                        for i in 0..len {
                            let idx = start + i;
                            if let Some(edge_id) = base_graph.reverse_edge_id(idx) {
                                if self.chain.is_edge_tombstoned(edge_id) {
                                    continue;
                                }
                                if let Some(edge) = base_graph.edge(edge_id) {
                                    if edge.kind != 2 {
                                        continue;
                                    } // EDGE_KIND_TRANSITION

                                    let dst = edge.dst.0;
                                    if check_node_emits(
                                        base_graph,
                                        dst,
                                        t,
                                        emit_remainder,
                                        exct_remainder,
                                    )
                                    .0 && !next_current[..next_len].contains(&dst)
                                        && next_len < 64
                                    {
                                        next_current[next_len] = dst;
                                        next_len += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                current = next_current;
                current_len = next_len;
                if current_len == 0 {
                    failed = true;
                    break;
                }
            }

            if !failed && current_len > 0 {
                active_nodes[..current_len].copy_from_slice(&current[..current_len]);
                active_len = current_len;
                break;
            }
        }

        // Geometric Routing Fallback (Phase 6 & 8)
        // If the suffix DFA fell off the manifold, use the continuous 288-bit VSA signature
        // to find the top-M semantic regions N_best (N_best <= 8) to jump back onto the graph!
        if (active_len == 0 || (active_len == 1 && active_nodes[0] == 0))
            && let Some(sig) = context_signature
        {
            let (best_node, active_count) = if let Some(index) = self.route_index.as_ref() {
                let mut matched_nodes = [0u32; 8];
                let (best_node, _best_dist, active_count) = index.query(sig, &mut matched_nodes);
                active_nodes[..active_count].copy_from_slice(&matched_nodes[..active_count]);
                (best_node, active_count)
            } else {
                let mut best_node = 0;
                let mut best_dist = u32::MAX;
                let mut active_count = 0usize;
                let rout_bytes = base_graph.section(SectionId::ROUT).unwrap_or(&[]);

                for n in 1..num_nodes {
                    if let Some(node) = base_graph.node(n) {
                        let proto_offset = (node.prototype_word_start as usize) << 3;
                        let mask_offset = (node.mask_word_start as usize) << 3;

                        if proto_offset + sig.len() <= rout_bytes.len()
                            && mask_offset + sig.len() <= rout_bytes.len()
                        {
                            let mut dist = 0u32;
                            for i in 0..sig.len() {
                                let p = rout_bytes[proto_offset + i];
                                let m = rout_bytes[mask_offset + i];
                                let s = sig[i];
                                dist += ((s ^ p) & m).count_ones();
                            }

                            if dist < best_dist {
                                best_dist = dist;
                                best_node = n;
                            }

                            // Collect Quantum MoE ensemble nodes matching distance threshold
                            let rad = u32::from(node.radius.0).max(120);
                            if dist <= rad
                                && active_count < 8
                                && !active_nodes[..active_count].contains(&n)
                            {
                                active_nodes[active_count] = n;
                                active_count += 1;
                            }
                        }
                    }
                }
                (best_node, active_count)
            };

            if active_count > 0 {
                active_len = active_count;
            } else if best_node != 0 {
                active_nodes[0] = best_node;
                active_len = 1;
            }
        }

        // Expand active regions with outbound transition edge neighbors
        let mut expanded_nodes = [0u32; 32];
        let mut expanded_len = 0usize;
        for &node_id in &active_nodes[..active_len] {
            if expanded_len < 32 && !expanded_nodes[..expanded_len].contains(&node_id) {
                expanded_nodes[expanded_len] = node_id;
                expanded_len += 1;
            }
            if let Some(node) = base_graph.node(node_id)
                && node.forward_len > 0
            {
                let start = node.forward_start as usize;
                let count = (node.forward_len as usize).min(4);
                for i in 0..count {
                    if let Some(rev_id) = base_graph.reverse_edge_id((start + i) as u32)
                        && let Some(edge) = base_graph.edge(rev_id)
                        && expanded_len < 32
                        && !expanded_nodes[..expanded_len].contains(&edge.dst.0)
                    {
                        expanded_nodes[expanded_len] = edge.dst.0;
                        expanded_len += 1;
                    }
                }
            }
        }

        if expanded_len > 0 {
            active_nodes[..expanded_len].copy_from_slice(&expanded_nodes[..expanded_len]);
            active_len = expanded_len;
        }

        if active_len == 0 {
            active_nodes[0] = 0;
            active_len = 1;
        }

        let mut best_token = 0;
        let mut best_score = ScoreQ::MIN;

        // Read predicted tokens directly from active node emission lists (and child refinement lists)
        for &node_id in &active_nodes[..active_len] {
            if let Some(_node) = base_graph.node(node_id) {
                let mut target_nodes = [0u32; 128];
                let mut num_targets = 0usize;
                collect_target_leaf_nodes(base_graph, node_id, &mut target_nodes, &mut num_targets);

                for &target_id in &target_nodes[..num_targets] {
                    let target_node = match base_graph.node(target_id) {
                        Some(tn) => tn,
                        None => continue,
                    };

                    let sl = if target_node.emission_len == 0 {
                        if target_id == 0 {
                            exct_remainder.or(emit_remainder).unwrap_or(&[])
                        } else {
                            &[][..]
                        }
                    } else if let Some(remainder) = emit_remainder {
                        let start = target_node.emission_start as usize;
                        let len = target_node.emission_len as usize;
                        if start + len <= remainder.len() {
                            &remainder[start..start + len]
                        } else {
                            &[][..]
                        }
                    } else {
                        &[][..]
                    };
                    for i in 0..(sl.len() >> 3) {
                        let offset = i << 3;
                        let cand = u32::from_le_bytes([
                            sl[offset],
                            sl[offset + 1],
                            sl[offset + 2],
                            sl[offset + 3],
                        ]);
                        if cand == 0 || cand >= 49152 {
                            continue;
                        }
                        let raw = i32::from_le_bytes([
                            sl[offset + 4],
                            sl[offset + 5],
                            sl[offset + 6],
                            sl[offset + 7],
                        ]);

                        let mut emit_score = if raw > 0 {
                            ScoreQ::from_raw(raw)
                        } else {
                            ScoreQ::from_raw(1)
                        };

                        let sig_bonus = if let Some(sig) = session_signature.or(context_signature) {
                            let rout_bytes = base_graph.section(SectionId::ROUT).unwrap_or(&[]);
                            let proto_offset = (target_node.prototype_word_start as usize) << 3;
                            let mask_offset = (target_node.mask_word_start as usize) << 3;
                            if proto_offset + sig.len() <= rout_bytes.len()
                                && mask_offset + sig.len() <= rout_bytes.len()
                            {
                                signature_affinity_bonus(
                                    &rout_bytes[proto_offset..proto_offset + sig.len()],
                                    &rout_bytes[mask_offset..mask_offset + sig.len()],
                                    sig,
                                )
                            } else {
                                0
                            }
                        } else {
                            0
                        };

                        let mut penalty = 0i32;
                        let recent_window = 48;
                        let start_pos = context_tokens.len().saturating_sub(recent_window);
                        for (idx, &recent_tok) in context_tokens[start_pos..].iter().enumerate() {
                            if cand == recent_tok {
                                let age = context_tokens.len() - (start_pos + idx);
                                // x * 350 == (x<<8)+(x<<6)+(x<<4)+(x<<3)+(x<<2)+(x<<1) (shift/add only).
                                let x = 48 - age as i32;
                                penalty += (x << 8)
                                    .saturating_add(x << 6)
                                    .saturating_add(x << 4)
                                    .saturating_add(x << 3)
                                    .saturating_add(x << 2)
                                    .saturating_add(x << 1);
                            }
                        }

                        // Category-Theoretic Morphism & Sentence Completion Scoring
                        let prev_token = context_tokens.last().copied().unwrap_or(0);
                        let mut tokens_since_period = 0usize;
                        for &tok in context_tokens.iter().rev() {
                            if tok == 29889 || tok == 13 || tok == 2 {
                                break;
                            }
                            tokens_since_period += 1;
                        }
                        let morphism_score =
                            syntactic_morphism_score(prev_token, cand, tokens_since_period);

                        let is_punct = cand == 29889 || cand == 29892 || cand == 11 || cand == 13;
                        if is_punct && tokens_since_period < 8 {
                            continue;
                        }

                        let final_score = emit_score
                            .raw()
                            .saturating_add(sig_bonus)
                            .saturating_add(morphism_score)
                            .saturating_sub(penalty);
                        emit_score = ScoreQ::from_raw(final_score);

                        if emit_score.raw() > best_score.raw()
                            || (best_token != 0
                                && emit_score.raw() == best_score.raw()
                                && cand < best_token)
                        {
                            best_score = emit_score;
                            best_token = cand;
                        }
                    }
                }
            }
        }

        if best_token == 0 {
            // If best_token is still 0, emit the first non-zero token from Node 0's emission list
            if let Some(remainder) = emit_remainder {
                for i in 0..(remainder.len() >> 3) {
                    let offset = i << 3;
                    let cand = u32::from_le_bytes([
                        remainder[offset],
                        remainder[offset + 1],
                        remainder[offset + 2],
                        remainder[offset + 3],
                    ]);
                    if cand > 2 && cand < 49152 {
                        best_token = cand;
                        break;
                    }
                }
            }
        }

        (best_token, best_score)
    }

    /// Predict top-k candidate tokens with their scores for Beam Search decoding.
    pub fn predict_candidates(
        &self,
        context_tokens: &[u32],
        signature: Option<&[u8]>,
        node_scores: &mut [ScoreQ],
        out_candidates: &mut [(u32, ScoreQ); 8],
    ) -> usize {
        self.predict_candidates_with_signature_lanes(
            context_tokens,
            signature,
            None,
            node_scores,
            out_candidates,
        )
    }

    /// Top-k counterpart of
    /// [`Self::predict_distribution_with_signature_lanes`].
    pub fn predict_candidates_with_signature_lanes(
        &self,
        context_tokens: &[u32],
        context_signature: Option<&[u8]>,
        session_signature: Option<&[u8]>,
        node_scores: &mut [ScoreQ],
        out_candidates: &mut [(u32, ScoreQ); 8],
    ) -> usize {
        let prev_token = context_tokens.last().copied().unwrap_or(0);
        let mut tokens_since_period = 0usize;
        for &tok in context_tokens.iter().rev() {
            if tok == 29889 || tok == 13 || tok == 2 {
                break;
            }
            tokens_since_period += 1;
        }

        let (top_tok, top_score) = self.predict_distribution_with_signature_lanes(
            context_tokens,
            context_signature,
            session_signature,
            node_scores,
        );

        let mut count = 0usize;
        if top_tok != 0 {
            out_candidates[0] = (top_tok, top_score);
            count = 1;
        }

        // Iterate over the top active nodes in node_scores to collect candidate tokens
        let emit_remainder = self.view().section(SectionId::EMIT);
        if let Some(remainder) = emit_remainder {
            let view = self.view();
            let max_nodes = core::cmp::min(self.node_count() as usize, node_scores.len());
            #[allow(clippy::needless_range_loop)]
            for node_idx in 0..max_nodes {
                if count >= 8 {
                    break;
                }
                let n_score = node_scores[node_idx];
                if n_score.raw() == ScoreQ::MIN.raw() {
                    continue;
                }
                if let Some(target_node) = view.node(node_idx as u32) {
                    let start = target_node.emission_start as usize;
                    let len = target_node.emission_len as usize;
                    if start + len <= remainder.len() {
                        let sl = &remainder[start..start + len];
                        for i in 0..(sl.len() >> 3) {
                            if count >= 8 {
                                break;
                            }
                            let offset = i << 3;
                            let cand = u32::from_le_bytes([
                                sl[offset],
                                sl[offset + 1],
                                sl[offset + 2],
                                sl[offset + 3],
                            ]);
                            let raw = i32::from_le_bytes([
                                sl[offset + 4],
                                sl[offset + 5],
                                sl[offset + 6],
                                sl[offset + 7],
                            ]);

                            let is_punct =
                                cand == 29889 || cand == 29892 || cand == 11 || cand == 13;
                            if cand > 2
                                && cand < 49152
                                && !(is_punct && tokens_since_period < 8)
                                && !out_candidates[..count].iter().any(|(c, _)| *c == cand)
                            {
                                let m_score =
                                    syntactic_morphism_score(prev_token, cand, tokens_since_period);
                                let final_score = raw.saturating_add(m_score);
                                out_candidates[count] = (cand, ScoreQ::from_raw(final_score));
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
        out_candidates[..count].sort_by_key(|b| core::cmp::Reverse(b.1.raw()));
        count
    }
}

#[cfg(test)]
mod tests {
    use super::signature_affinity_bonus;

    #[test]
    fn session_lane_bonus_is_integer_hamming_affinity() {
        let prototype = [0u8; 36];
        let mask = [0xffu8; 36];
        let near = [0u8; 36];
        let far = [0xffu8; 36];
        assert!(
            signature_affinity_bonus(&prototype, &mask, &near)
                > signature_affinity_bonus(&prototype, &mask, &far)
        );
    }
}
