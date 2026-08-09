//! Bounded Future-State Optimization & Planning Over Graph Transitions
//!
//! Specification & Source: `docs/hologram_formal_analysis_direction.md` PDF §§3, 10, 12, 17;
//! `docs/formal_vocabulary.md` §5; GitHub Issue #131.
//!
//! This module replaces overloaded intent framing with an explicit bounded future-state optimizer:
//! - Finite action trajectory search $s_0 \xrightarrow{a_1} s_1 \xrightarrow{a_2} \dots \xrightarrow{a_k} s_k \in G$.
//! - Enforces forbidden region constraints $s_i \notin C$ at every intermediate step.
//! - Bounded planning horizon $H_{\max}$, frontier size $K_{\max}$, and deterministic tie-breaking.
//! - Emits `PlanWitness` recording selected transitions and rejected counterfactual alternatives.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Graph state node for graph transitions.
#[derive(Debug, Clone, PartialEq)]
pub struct PlannerStateNode {
    pub id: String,
    pub is_goal: bool,
    pub is_forbidden: bool,
    pub forbidden_region_id: Option<String>,
}

/// Typed graph edge transition.
#[derive(Debug, Clone, PartialEq)]
pub struct PlannerEdgeTransition {
    pub src_id: String,
    pub action: String,
    pub dst_id: String,
    pub cost: f32,
    pub confidence: f32,
}

/// Planning configuration and resource bounds.
#[derive(Debug, Clone, PartialEq)]
pub struct PlannerConfig {
    pub max_horizon: usize,
    pub max_frontier_size: usize,
    pub min_confidence_threshold: f32,
}

impl PlannerConfig {
    pub fn default_v1() -> Self {
        Self {
            max_horizon: 10,
            max_frontier_size: 100,
            min_confidence_threshold: 0.5,
        }
    }
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self::default_v1()
    }
}

/// Plan trajectory result.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanTrajectory {
    pub state_sequence: Vec<String>,
    pub action_sequence: Vec<String>,
    pub total_cost: f32,
    pub horizon_steps: usize,
    pub nodes_expanded: usize,
    pub witness: PlanWitness,
}

/// Audit witness recording selected transitions and rejected alternatives.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanWitness {
    pub plan_cid: String,
    pub accepted_edges: Vec<(String, String, String)>, // (src, action, dst)
    pub rejected_alternatives_count: usize,
}

#[derive(Clone)]
struct SearchNode {
    state_id: String,
    g_cost: f32,
    f_cost: f32,
    depth: usize,
    state_path: Vec<String>,
    action_path: Vec<String>,
}

impl PartialEq for SearchNode {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for SearchNode {}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap based on f_cost with deterministic tie-breaking on state_id
        other
            .f_cost
            .total_cmp(&self.f_cost)
            .then_with(|| self.state_id.cmp(&other.state_id))
    }
}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Bounded Future-State Graph Planner.
pub struct BoundedGraphPlanner;

impl BoundedGraphPlanner {
    /// Plan finite action trajectory from start state to goal region avoiding
    /// forbidden states. `None` when no trajectory is a product of these inputs:
    /// the start state is unknown or forbidden, or no plan reaches the goal
    /// region within the horizon and frontier bounds (R5 — the absence of a
    /// product rather than a raised error; the search either yields a plan or
    /// there is none).
    pub fn plan(
        start_state_id: &str,
        nodes: &[PlannerStateNode],
        edges: &[PlannerEdgeTransition],
        config: &PlannerConfig,
    ) -> Option<PlanTrajectory> {
        let node_map: HashMap<&str, &PlannerStateNode> =
            nodes.iter().map(|n| (n.id.as_str(), n)).collect();

        let start_node = node_map.get(start_state_id)?;

        if start_node.is_forbidden {
            return None;
        }

        let mut open_set = BinaryHeap::new();
        let mut visited = HashSet::new();
        let mut rejected_count = 0;
        let mut nodes_expanded = 0;

        open_set.push(SearchNode {
            state_id: start_state_id.to_string(),
            g_cost: 0.0,
            f_cost: 0.0,
            depth: 0,
            state_path: vec![start_state_id.to_string()],
            action_path: Vec::new(),
        });

        while let Some(current) = open_set.pop() {
            nodes_expanded += 1;

            let curr_node = match node_map.get(current.state_id.as_str()) {
                Some(n) => n,
                None => continue,
            };
            if curr_node.is_forbidden {
                continue;
            }
            if curr_node.is_goal {
                let mut accepted_edges = Vec::new();
                for i in 0..current.action_path.len() {
                    accepted_edges.push((
                        current.state_path[i].clone(),
                        current.action_path[i].clone(),
                        current.state_path[i + 1].clone(),
                    ));
                }

                let plan_cid = format!(
                    "blake3:plan_{}",
                    blake3::hash(current.state_path.join("->").as_bytes()).to_hex()
                );
                return Some(PlanTrajectory {
                    state_sequence: current.state_path,
                    action_sequence: current.action_path,
                    total_cost: current.g_cost,
                    horizon_steps: current.depth,
                    nodes_expanded,
                    witness: PlanWitness {
                        plan_cid,
                        accepted_edges,
                        rejected_alternatives_count: rejected_count,
                    },
                });
            }

            if current.depth >= config.max_horizon {
                continue;
            }

            if visited.contains(&current.state_id) {
                continue;
            }
            visited.insert(current.state_id.clone());

            let outgoing: Vec<&PlannerEdgeTransition> = edges
                .iter()
                .filter(|e| e.src_id == current.state_id)
                .collect();

            for edge in outgoing {
                if edge.confidence < config.min_confidence_threshold {
                    rejected_count += 1;
                    continue;
                }

                let dst_node = match node_map.get(edge.dst_id.as_str()) {
                    Some(n) => n,
                    None => {
                        rejected_count += 1;
                        continue;
                    }
                };

                if dst_node.is_forbidden {
                    rejected_count += 1;
                    continue;
                }

                let new_g = current.g_cost + edge.cost;
                let new_h = if dst_node.is_goal { 0.0 } else { 1.0 };
                let new_f = new_g + new_h;

                let mut new_state_path = current.state_path.clone();
                new_state_path.push(edge.dst_id.clone());

                let mut new_action_path = current.action_path.clone();
                new_action_path.push(edge.action.clone());

                open_set.push(SearchNode {
                    state_id: edge.dst_id.clone(),
                    g_cost: new_g,
                    f_cost: new_f,
                    depth: current.depth + 1,
                    state_path: new_state_path,
                    action_path: new_action_path,
                });
            }

            if open_set.len() > config.max_frontier_size {
                return None;
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_planner_successful_goal_reaching() {
        let nodes = vec![
            PlannerStateNode {
                id: "s0".to_string(),
                is_goal: false,
                is_forbidden: false,
                forbidden_region_id: None,
            },
            PlannerStateNode {
                id: "s1".to_string(),
                is_goal: false,
                is_forbidden: false,
                forbidden_region_id: None,
            },
            PlannerStateNode {
                id: "s2".to_string(),
                is_goal: true,
                is_forbidden: false,
                forbidden_region_id: None,
            },
        ];
        let edges = vec![
            PlannerEdgeTransition {
                src_id: "s0".to_string(),
                action: "step1".to_string(),
                dst_id: "s1".to_string(),
                cost: 1.0,
                confidence: 0.9,
            },
            PlannerEdgeTransition {
                src_id: "s1".to_string(),
                action: "step2".to_string(),
                dst_id: "s2".to_string(),
                cost: 1.0,
                confidence: 0.95,
            },
        ];

        let config = PlannerConfig::default_v1();
        let plan = BoundedGraphPlanner::plan("s0", &nodes, &edges, &config).unwrap();

        assert_eq!(plan.state_sequence, vec!["s0", "s1", "s2"]);
        assert_eq!(plan.action_sequence, vec!["step1", "step2"]);
        assert_eq!(plan.horizon_steps, 2);
        assert!(plan.witness.plan_cid.starts_with("blake3:plan_"));
    }

    #[test]
    fn test_bounded_planner_forbidden_constraint_rejection() {
        let nodes = vec![
            PlannerStateNode {
                id: "s0".to_string(),
                is_goal: false,
                is_forbidden: false,
                forbidden_region_id: None,
            },
            PlannerStateNode {
                id: "s1".to_string(),
                is_goal: false,
                is_forbidden: true,
                forbidden_region_id: Some("C_hazard".to_string()),
            },
            PlannerStateNode {
                id: "s2".to_string(),
                is_goal: true,
                is_forbidden: false,
                forbidden_region_id: None,
            },
        ];
        let edges = vec![PlannerEdgeTransition {
            src_id: "s0".to_string(),
            action: "step1".to_string(),
            dst_id: "s1".to_string(),
            cost: 1.0,
            confidence: 0.9,
        }];

        let config = PlannerConfig::default_v1();
        // The only path runs through the forbidden intermediate state, so no
        // trajectory reaches the goal: the planner yields no plan.
        let plan = BoundedGraphPlanner::plan("s0", &nodes, &edges, &config);
        assert!(plan.is_none());
    }
}
