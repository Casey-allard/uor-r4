use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uor_r4_core::semantic::{KappaLabel, WeightedRoute};
use uor_r4_core::zeta_projection::{project_state, window_ranges, NUM_WINDOWS};
use uor_r4_core::{get_word_vector, identity_to_qimc_prime};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TypedObject {
    pub object_type: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroundedSemantics {
    pub vsa_vector: Vec<f32>,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FacetCoordinates {
    pub coordinates: HashMap<String, Vec<u32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Operator {
    pub name: String,
    pub space_cid: KappaLabel,
}

pub trait SemanticGeometry {
    fn space_manifest(&self) -> KappaLabel;
    /// Ground a typed object into semantics. `None` when the object carries no
    /// content to ground: not a limitation of the geometry, but the absence of
    /// an object to place (R5 — the only reportable condition is "no product").
    fn ground(&self, object: &TypedObject) -> Option<GroundedSemantics>;
    /// Encode grounded semantics into facet coordinates. Total: every grounded
    /// vector has coordinates.
    fn encode(&self, grounded: &GroundedSemantics) -> FacetCoordinates;
    /// Soft-route coordinates to weighted routes. Total: the empty coordinate
    /// set routes to the empty (or single fallback) route list, never an error.
    fn soft_route(&self, coordinates: &FacetCoordinates, max_routes: usize) -> Vec<WeightedRoute>;
    /// Apply an operator to a route. `None` when this geometry does not carry
    /// the requested operator — the (route, operator) product does not exist
    /// here, not a fault.
    fn apply_operator(
        &self,
        route: &WeightedRoute,
        operator: &Operator,
    ) -> Option<Vec<WeightedRoute>>;
}

// 1. Spectral Geometry (Heuristic Baseline)
pub struct SpectralGeometry<'a> {
    pub space_cid: KappaLabel,
    pub active_state: Option<&'a [f64]>,
    pub identity: Option<&'a str>,
}

impl<'a> SemanticGeometry for SpectralGeometry<'a> {
    fn space_manifest(&self) -> KappaLabel {
        self.space_cid.clone()
    }

    fn ground(&self, object: &TypedObject) -> Option<GroundedSemantics> {
        if object.content.is_empty() {
            return None;
        }
        let mut v = vec![0.0; 1024];
        if let Some(state) = self.active_state {
            v[..512].copy_from_slice(state);

            // Keep the session state as context, but make the spectral query
            // itself observable. This is the content-reconnection path: two
            // queries with an identical session state get distinct,
            // deterministic zeta signals before QR window selection.
            let mut query_signal = vec![0.0; 512];
            let mut known_words = 0usize;
            for word in object.content.split_whitespace() {
                let normalized = word
                    .trim_matches(|character: char| !character.is_alphanumeric())
                    .to_lowercase();
                if normalized.is_empty() {
                    continue;
                }
                let (prime, _, _) = identity_to_qimc_prime(&normalized);
                let word_vector = get_word_vector(prime);
                for (target, source) in query_signal.iter_mut().zip(word_vector) {
                    *target += source;
                }
                known_words += 1;
            }
            let query_norm = query_signal
                .iter()
                .map(|value| value * value)
                .sum::<f64>()
                .sqrt();
            if known_words > 0 && query_norm > 1.0e-12 {
                for (target, source) in v[..512].iter_mut().zip(query_signal) {
                    *target = 0.25 * *target + 0.75 * source / query_norm;
                }
            }
        } else {
            v[..512].copy_from_slice(&vec![1.0 / (512.0f64).sqrt(); 512]);
        }
        Some(GroundedSemantics {
            vsa_vector: v.iter().map(|&x| x as f32).collect(),
            roles: vec!["spectral-role".to_string()],
        })
    }

    fn encode(&self, grounded: &GroundedSemantics) -> FacetCoordinates {
        let active_state: Vec<f64> = grounded.vsa_vector[..512]
            .iter()
            .map(|&x| x as f64)
            .collect();
        // Window choice is a property of the query signal, not the identity
        // control plane. Keeping these biases at zero prevents a near-tie
        // from moving identical content to a different sparse range when the
        // same sentence is indexed under another identity.
        let biases = vec![0.0; NUM_WINDOWS];
        let projection = project_state(&active_state, &biases);
        let routed_idx = projection.window_index;

        let mut coords = HashMap::new();
        coords.insert("window".to_string(), vec![routed_idx as u32]);

        let mut score_bits = Vec::with_capacity(NUM_WINDOWS);
        for &score in &projection.scores {
            score_bits.push((score as f32).to_bits());
        }
        coords.insert("scores".to_string(), score_bits);
        coords.insert(
            "ranges".to_string(),
            window_ranges()
                .into_iter()
                .flat_map(|(start, end)| [start as u32, end as u32])
                .collect(),
        );
        coords.insert(
            "eigenvalues".to_string(),
            projection
                .eigenvalues
                .iter()
                .map(|value| (*value as f32).to_bits())
                .collect(),
        );
        coords.insert(
            "projected_state".to_string(),
            projection
                .projected_state
                .iter()
                .map(|value| (*value as f32).to_bits())
                .collect(),
        );

        FacetCoordinates {
            coordinates: coords,
        }
    }

    fn soft_route(&self, coordinates: &FacetCoordinates, _max_routes: usize) -> Vec<WeightedRoute> {
        let window_idx = coordinates
            .coordinates
            .get("window")
            .and_then(|w| w.first())
            .copied()
            .unwrap_or(1);

        let mut routes = Vec::new();
        if let Some(scores) = coordinates.coordinates.get("scores") {
            let ranges = coordinates.coordinates.get("ranges");
            for (idx, &bits) in scores.iter().enumerate() {
                let score = f32::from_bits(bits);
                let path = ranges
                    .and_then(|ranges| {
                        let start = ranges.get(idx * 2)?;
                        let end = ranges.get(idx * 2 + 1)?;
                        Some(vec![(idx + 1) as u32, *start, *end])
                    })
                    .unwrap_or_else(|| vec![(idx + 1) as u32]);
                routes.push(WeightedRoute {
                    axis: (idx + 1) as u32,
                    path,
                    score,
                });
            }
        } else {
            routes.push(WeightedRoute {
                axis: window_idx,
                path: vec![window_idx],
                score: 1.0,
            });
        }
        routes
    }

    fn apply_operator(
        &self,
        route: &WeightedRoute,
        operator: &Operator,
    ) -> Option<Vec<WeightedRoute>> {
        if operator.name == "identity" {
            Some(vec![route.clone()])
        } else {
            None
        }
    }
}

// 2. VsaGeometry (Proof-Carrying)
pub struct VsaGeometry {
    pub space_cid: KappaLabel,
}

impl SemanticGeometry for VsaGeometry {
    fn space_manifest(&self) -> KappaLabel {
        self.space_cid.clone()
    }

    fn ground(&self, object: &TypedObject) -> Option<GroundedSemantics> {
        if object.content.is_empty() {
            return None;
        }
        // #496: ground the VSA hypervector from the zeta word-sum CONTENT signal
        // (the same multiplication-free construction `SpectralGeometry::ground`
        // uses, no transformer/GPU), instead of `expand_atom`'s content HASH.
        // The hash gave related sentences unrelated random ±1 vectors, so even a
        // commensurable comparison ranked at chance (#493). Summing each word's
        // seeded zeta vector makes the base semantic: related content → similar
        // vectors, and the facet multi-index (`encode`) rides on top unchanged.
        let mut content = vec![0.0f64; 512];
        let mut known_words = 0usize;
        for word in object.content.split_whitespace() {
            let normalized = word
                .trim_matches(|character: char| !character.is_alphanumeric())
                .to_lowercase();
            if normalized.is_empty() {
                continue;
            }
            let (prime, _, _) = identity_to_qimc_prime(&normalized);
            let word_vector = get_word_vector(prime);
            for (target, source) in content.iter_mut().zip(word_vector) {
                *target += source;
            }
            known_words += 1;
        }
        let norm = content
            .iter()
            .map(|value| value * value)
            .sum::<f64>()
            .sqrt();
        let mut float_vec = vec![0.0f32; 1024];
        if known_words > 0 && norm > 1.0e-12 {
            // Normalized content signal in the first 512 dims; the second half is
            // held at zero so the cosine of two grounded vectors IS their content
            // cosine (both queries and stored candidates ground identically).
            for (target, source) in float_vec[..512].iter_mut().zip(content.iter()) {
                *target = (source / norm) as f32;
            }
        } else {
            // No vocabulary word at all: a content-free uniform base. It cannot
            // rank by content, but it is never a random hash of the string.
            let uniform = (1.0 / (512.0f64).sqrt()) as f32;
            for target in float_vec[..512].iter_mut() {
                *target = uniform;
            }
        }
        Some(GroundedSemantics {
            vsa_vector: float_vec,
            roles: vec!["grounded-vsa-role".to_string()],
        })
    }

    fn encode(&self, grounded: &GroundedSemantics) -> FacetCoordinates {
        let mut coords = HashMap::new();
        // Induce simple path codes by binning the grounded VSA dimensions
        let mut type_path = Vec::new();
        if !grounded.vsa_vector.is_empty() {
            let sum_first_half: f32 = grounded.vsa_vector[0..512].iter().sum();
            type_path.push((sum_first_half.abs() as u32) % 100);
        }
        coords.insert("type".to_string(), type_path);
        coords.insert("entity".to_string(), vec![100, 200]);
        FacetCoordinates {
            coordinates: coords,
        }
    }

    fn soft_route(&self, coordinates: &FacetCoordinates, max_routes: usize) -> Vec<WeightedRoute> {
        let mut routes = Vec::new();
        for (facet, path) in &coordinates.coordinates {
            let axis = match facet.as_str() {
                "type" => 1,
                "entity" => 2,
                _ => 0,
            };
            routes.push(WeightedRoute {
                axis,
                path: path.clone(),
                score: 0.95,
            });
        }
        routes.into_iter().take(max_routes).collect()
    }

    fn apply_operator(
        &self,
        route: &WeightedRoute,
        operator: &Operator,
    ) -> Option<Vec<WeightedRoute>> {
        if operator.name == "vsa-identity" {
            Some(vec![route.clone()])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_session_state_can_select_different_query_windows() {
        let state = vec![1.0 / (512.0_f64).sqrt(); 512];
        let geometry = SpectralGeometry {
            space_cid: "blake3:spectral_space".to_string(),
            active_state: Some(&state),
            identity: Some("test-session"),
        };
        let first = geometry.encode(
            &geometry
                .ground(&TypedObject {
                    object_type: "query".to_string(),
                    content: "prime factorization and zeta spectrum".to_string(),
                })
                .expect("first query grounds"),
        );
        let second = geometry.encode(
            &geometry
                .ground(&TypedObject {
                    object_type: "query".to_string(),
                    content: "oceanic climate satellites and rainfall".to_string(),
                })
                .expect("second query grounds"),
        );

        assert_ne!(
            first.coordinates.get("window"),
            second.coordinates.get("window"),
            "query content must affect window selection with a fixed session state"
        );
    }
}
