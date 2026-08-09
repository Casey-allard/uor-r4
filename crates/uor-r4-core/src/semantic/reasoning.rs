use super::reference::KappaLabel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Constraint {
    pub facet: String,
    pub path: Vec<u32>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FacetPolicy {
    pub priority_order: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackoffPolicy {
    pub max_backoff_steps: u32,
    pub allow_any_facet: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidencePolicy {
    pub min_evidence_count: u32,
    pub require_provenance: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Limits {
    pub max_probes: u32,
    pub max_operators: u32,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeightedRoute {
    pub axis: u32,
    pub path: Vec<u32>,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegionProbe {
    pub region_id: String,
    pub depth: u32,
    pub matched: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OperatorExecution {
    pub operator_cid: KappaLabel,
    pub input_route: Vec<u32>,
    pub output_routes: Vec<Vec<u32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateScore {
    pub candidate_cid: KappaLabel,
    pub raw_score: f32,
    pub breakdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OperationCensus {
    pub total_probes: u64,
    pub total_operator_steps: u64,
    pub total_joins: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReasoningPlanV1 {
    pub query_cid: KappaLabel,
    pub semantic_space_cid: KappaLabel,
    pub clauses: Vec<Constraint>,
    pub operators: Vec<KappaLabel>,
    pub facet_policy: FacetPolicy,
    pub backoff_policy: BackoffPolicy,
    pub required_evidence: EvidencePolicy,
    pub deterministic_limits: Limits,
}

impl ReasoningPlanV1 {
    pub fn execute(
        &self,
        registry: &OperatorRegistry,
        start_route: &WeightedRoute,
    ) -> Option<SemanticInferenceWitnessV1> {
        // Total: `None` when the plan yields no valid witness — an exhausted
        // operator/probe budget, a missing operator, or an unsatisfied required
        // clause / evidence floor (all properties of the plan + registry the
        // caller supplied).
        // 1. Budget Enforcement
        if self.deterministic_limits.max_operators == 0 {
            return None;
        }

        let mut current_routes = vec![start_route.clone()];
        let mut probed_regions = Vec::new();
        let mut applied_operators = Vec::new();
        let mut op_steps = 0;
        let mut probe_count = 0;
        let mut join_count = 0;

        for op_cid in &self.operators {
            if op_steps >= self.deterministic_limits.max_operators as u64 {
                return None;
            }

            let mut next_routes = Vec::new();
            for route in &current_routes {
                if probe_count >= self.deterministic_limits.max_probes as u64 {
                    return None;
                }

                probed_regions.push(RegionProbe {
                    region_id: format!("axis_{}:{:?}", route.axis, route.path),
                    depth: route.path.len() as u32,
                    matched: true,
                });
                probe_count += 1;

                let outputs = registry.evaluate(op_cid, route)?;

                if let Some(op) = registry.operators.get(op_cid) {
                    if op.op_type == OperatorType::Conjunction {
                        join_count += 1;
                    }
                }

                applied_operators.push(OperatorExecution {
                    operator_cid: op_cid.clone(),
                    input_route: route.path.clone(),
                    output_routes: outputs.iter().map(|r| r.path.clone()).collect(),
                });

                next_routes.extend(outputs);
            }

            current_routes = next_routes;
            op_steps += 1;
        }

        // 2. Plan Validation / Constraint checking
        let mut evidence_cids = Vec::new();
        let mut contradiction_cids = Vec::new();

        for (idx, clause) in self.clauses.iter().enumerate() {
            let mut satisfied = false;
            for route in &current_routes {
                if route.path.starts_with(&clause.path) {
                    satisfied = true;
                    break;
                }
            }

            let proof_cid = format!("blake3:clause_proof_{}_{}", idx, clause.facet);
            if satisfied {
                evidence_cids.push(proof_cid);
            } else {
                if clause.required {
                    return None;
                } else {
                    contradiction_cids.push(proof_cid);
                }
            }
        }

        if evidence_cids.len() < self.required_evidence.min_evidence_count as usize {
            return None;
        }

        // Compute deterministic plan CID by hashing its fields
        let plan_json = serde_json::to_string(self).unwrap_or_default();
        let plan_cid = format!(
            "blake3:plan_{}",
            blake3::hash(plan_json.as_bytes()).to_hex()
        );

        // Compute deterministic result CID by hashing the final routes
        let routes_json = serde_json::to_string(&current_routes).unwrap_or_default();
        let result_cid = format!(
            "blake3:result_{}",
            blake3::hash(routes_json.as_bytes()).to_hex()
        );

        let score_components = current_routes
            .iter()
            .enumerate()
            .map(|(i, r)| CandidateScore {
                candidate_cid: format!("blake3:candidate_{}", i),
                raw_score: r.score,
                breakdown: format!("route_score_axis_{}", r.axis),
            })
            .collect();

        Some(SemanticInferenceWitnessV1 {
            query_cid: self.query_cid.clone(),
            plan_cid,
            semantic_space_cid: self.semantic_space_cid.clone(),
            store_root_cids: vec!["blake3:store_root".to_string()],
            generated_routes: current_routes.clone(),
            probed_regions,
            applied_operators,
            evidence_cids,
            contradiction_cids,
            score_components,
            result_cid,
            operation_census: OperationCensus {
                total_probes: probe_count,
                total_operator_steps: op_steps,
                total_joins: join_count,
            },
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticInferenceWitnessV1 {
    pub query_cid: KappaLabel,
    pub plan_cid: KappaLabel,
    pub semantic_space_cid: KappaLabel,
    pub store_root_cids: Vec<KappaLabel>,
    pub generated_routes: Vec<WeightedRoute>,
    pub probed_regions: Vec<RegionProbe>,
    pub applied_operators: Vec<OperatorExecution>,
    pub evidence_cids: Vec<KappaLabel>,
    pub contradiction_cids: Vec<KappaLabel>,
    pub score_components: Vec<CandidateScore>,
    pub result_cid: KappaLabel,
    pub operation_census: OperationCensus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperatorType {
    RelationTraversal,
    Conjunction,
    Disjunction,
    Negation,
    Projection,
    TemporalOrdering,
    Backoff,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TypedOperator {
    pub cid: KappaLabel,
    pub name: String,
    pub op_type: OperatorType,
    pub input_type: String,
    pub output_type: String,
    pub transition_table: std::collections::HashMap<Vec<u32>, Vec<Vec<u32>>>, // input -> outputs
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct OperatorRegistry {
    pub space_cid: KappaLabel,
    pub operators: std::collections::HashMap<KappaLabel, TypedOperator>,
}

impl OperatorRegistry {
    pub fn new(space_cid: KappaLabel) -> Self {
        Self {
            space_cid,
            operators: std::collections::HashMap::new(),
        }
    }

    pub fn register_operator(&mut self, op: TypedOperator) {
        self.operators.insert(op.cid.clone(), op);
    }

    pub fn evaluate(
        &self,
        op_cid: &str,
        input_route: &WeightedRoute,
    ) -> Option<Vec<WeightedRoute>> {
        // Total: `None` only when the named operator is absent from the
        // registry; a present operator with no matching transition yields an
        // empty route set, not an error.
        let op = self.operators.get(op_cid)?;

        match op.op_type {
            OperatorType::RelationTraversal => {
                if let Some(transitions) = op.transition_table.get(&input_route.path) {
                    let mut results = Vec::new();
                    for path in transitions {
                        results.push(WeightedRoute {
                            axis: input_route.axis,
                            path: path.clone(),
                            score: input_route.score * 0.95,
                        });
                    }
                    Some(results)
                } else {
                    Some(vec![])
                }
            }
            OperatorType::Conjunction => {
                if let Some(transitions) = op.transition_table.get(&input_route.path) {
                    let mut results = Vec::new();
                    for path in transitions {
                        results.push(WeightedRoute {
                            axis: input_route.axis,
                            path: path.clone(),
                            score: input_route.score * 0.90, // Conjunction decay
                        });
                    }
                    Some(results)
                } else {
                    Some(vec![])
                }
            }
            OperatorType::Disjunction => {
                if let Some(transitions) = op.transition_table.get(&input_route.path) {
                    let mut results = Vec::new();
                    for path in transitions {
                        results.push(WeightedRoute {
                            axis: input_route.axis,
                            path: path.clone(),
                            score: input_route.score * 0.85, // Disjunction decay
                        });
                    }
                    Some(results)
                } else {
                    Some(vec![])
                }
            }
            OperatorType::Negation => {
                if let Some(transitions) = op.transition_table.get(&input_route.path) {
                    let mut results = Vec::new();
                    for path in transitions {
                        results.push(WeightedRoute {
                            axis: input_route.axis,
                            path: path.clone(),
                            score: -input_route.score, // Negation exclusion
                        });
                    }
                    Some(results)
                } else {
                    Some(vec![])
                }
            }
            OperatorType::Projection => {
                if let Some(transitions) = op.transition_table.get(&input_route.path) {
                    let mut results = Vec::new();
                    for path in transitions {
                        results.push(WeightedRoute {
                            axis: input_route.axis,
                            path: path.clone(),
                            score: input_route.score * 0.98,
                        });
                    }
                    Some(results)
                } else if input_route.path.len() > 1 {
                    Some(vec![WeightedRoute {
                        axis: input_route.axis,
                        path: vec![input_route.path[0]],
                        score: input_route.score * 0.90,
                    }])
                } else {
                    Some(vec![])
                }
            }
            OperatorType::TemporalOrdering => {
                if let Some(transitions) = op.transition_table.get(&input_route.path) {
                    let mut results = Vec::new();
                    for path in transitions {
                        results.push(WeightedRoute {
                            axis: input_route.axis,
                            path: path.clone(),
                            score: input_route.score * 0.92,
                        });
                    }
                    Some(results)
                } else {
                    let mut timed_path = input_route.path.clone();
                    timed_path.push(999);
                    Some(vec![WeightedRoute {
                        axis: input_route.axis,
                        path: timed_path,
                        score: input_route.score * 0.95,
                    }])
                }
            }
            OperatorType::Backoff => {
                if let Some(transitions) = op.transition_table.get(&input_route.path) {
                    let mut results = Vec::new();
                    for path in transitions {
                        results.push(WeightedRoute {
                            axis: input_route.axis,
                            path: path.clone(),
                            score: input_route.score * 0.8,
                        });
                    }
                    Some(results)
                } else if input_route.path.len() > 1 {
                    let mut backed_off = input_route.path.clone();
                    backed_off.pop();
                    Some(vec![WeightedRoute {
                        axis: input_route.axis,
                        path: backed_off,
                        score: input_route.score * 0.8,
                    }])
                } else {
                    Some(vec![])
                }
            }
        }
    }
}
