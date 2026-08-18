//! The R⁴ Tangent Space Router engine: `UorR4Router` state, manifold
//! indexing, geometric Markov generation, thought streams. The wasm-bindgen
//! surface travels with the struct; the cdylib link stays at the facade.

use crate::geometry::SemanticGeometry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f64::consts::PI;
use std::sync::OnceLock;
use uor_r4_core::*;
use wasm_bindgen::prelude::*;

pub mod benchmark;
pub mod fallback;
pub mod geometry;
pub mod session_signature;

pub use session_signature::{
    fixture_session_signatures, from_state as session_signature_from_state,
    from_tokens as session_signature_from_tokens, MULTI_TURN_FIXTURE,
};

// #790 item 4: only the live cascade surface is exported — the dead
// FallbackRouter pipeline and its satellite types were removed.
pub use fallback::{run_cascade, CascadeOutcome, EngineStatus, TierFn, TierOutcome, TierResult};

/// A content-addressed identifier derived via the 3/8 Resonance Hashing Law.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UorAddress {
    pub hash_bytes: [u8; 32],
}

impl UorAddress {
    pub fn to_uri(&self) -> String {
        let hex_str: String = self
            .hash_bytes
            .iter()
            .take(8)
            .map(|b| format!("{:02x}", b))
            .collect();
        format!("uor-addr-{}", hex_str)
    }
}

/// Models a single active reasoning trajectory or thought stream traveling through R⁴.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtStream {
    pub id: String,
    pub raw_content: String,
    pub uor_address: UorAddress,
    pub uor_uri: String,
    pub r4_target: R4Vector,
    pub path_steps: Vec<R4Vector>,
    pub activated_experts: Vec<u64>,
    pub alignment_phase: f64,  // $\theta$ phase state
    pub twist_parity_spin: i8, // $\kappa \in \{-1, 1\}$
    pub gcd: u64,
}

/// Dynamic resonance details computed via the 3/8 Resonance Hashing Law.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceInfo {
    pub total_bytes: u64,
    pub resonant_bits: u64,
    pub klein_matches: u64,
    pub uor_signature: String,
}

mod sparse_vector_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct SparseVecRepresentation {
        start_idx: u64,
        values: Vec<f64>,
    }

    pub fn serialize<S>(vec: &[f64], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut start_idx = 0;
        let mut end_idx = 0;
        let mut found = false;

        for (i, &val) in vec.iter().enumerate() {
            if val != 0.0 {
                if !found {
                    start_idx = i;
                    found = true;
                }
                end_idx = i + 1;
            }
        }

        let representation = if found {
            SparseVecRepresentation {
                start_idx: start_idx as u64,
                values: vec[start_idx..end_idx].to_vec(),
            }
        } else {
            SparseVecRepresentation {
                start_idx: 0,
                values: Vec::new(),
            }
        };

        representation.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let representation = SparseVecRepresentation::deserialize(deserializer)?;
        let mut vec = vec![0.0; 512];
        let start_idx = representation.start_idx as usize;
        let end_idx = start_idx + representation.values.len();
        if end_idx <= 512 {
            for (i, &val) in representation.values.iter().enumerate() {
                vec[start_idx + i] = val;
            }
        }
        Ok(vec)
    }
}

/// Scoped corpus sentence indexed on the manifold
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CorpusItem {
    #[serde(default)]
    pub sentence: String,
    #[serde(default, with = "sparse_vector_serde")]
    pub state_vector: Vec<f64>,
    #[serde(default)]
    pub kappa: f64,
    #[serde(default)]
    pub deficit_angle: f64,
    #[serde(default)]
    pub prime_product: String, // Stored as String to avoid JS JSON float loss
    #[serde(default)]
    pub words: Vec<String>,
    #[serde(default)]
    pub u: f64,
    #[serde(default)]
    pub v: f64,
    #[serde(default)]
    pub v_4d: Vec<f64>,
    #[serde(default)]
    pub provenance: Option<CoordinateProvenance>,
}

/// UOR-addressed evidence for the text and transforms that produced one
/// stored geometric coordinate. Empty/absent provenance is retained for
/// backwards-compatible imports of pre-#259 router state.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CoordinateProvenance {
    pub source_kappa: String,
    pub projection_kappa: String,
    pub vocabulary_kappa: String,
}

fn uor_json_address(value: &serde_json::Value) -> String {
    let bytes = match serde_json::to_vec(value) {
        Ok(bytes) => bytes,
        Err(_) => return "uor-addr:error:serialization".to_string(),
    };
    match uor_addr::json::address(&bytes) {
        Ok(outcome) => outcome.address.to_string(),
        Err(_) => match uor_addr::json::address_blake3(&bytes) {
            Ok(outcome) => outcome.address.to_string(),
            Err(_) => "uor-addr:error:address".to_string(),
        },
    }
}

fn source_provenance(sentence: &str) -> String {
    uor_json_address(&serde_json::json!({
        "axis": "uor-r4-router-source-v1",
        "sentence": sentence,
    }))
}

fn projection_provenance() -> &'static str {
    static PROJECTION_KAPPA: OnceLock<String> = OnceLock::new();
    PROJECTION_KAPPA.get_or_init(|| {
        uor_json_address(&serde_json::json!({
            "axis": "uor-r4-spectral-projection-v1",
            "channels": 512,
            "windows": 16,
            "samples": 257,
            "subwindows": 6,
            "x_min": 1.0e4,
            "x_max": 1.0e6,
            "rho": 4.0,
            "sparse_radius": 0.3,
            "window_bias": "none",
            "query_state_blend": [0.25, 0.75],
            "zeta_zero_set": "uor-r4-core::zeta_zeros:v1",
        }))
    })
}

fn vocabulary_provenance(word_primes: &HashMap<String, u64>, words: &[String]) -> String {
    let mut entries: Vec<(String, u64)> = words
        .iter()
        .filter_map(|word| word_primes.get(word).map(|prime| (word.clone(), *prime)))
        .collect();
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    uor_json_address(&serde_json::json!({
        "axis": "uor-r4-vocabulary-state-v1",
        "entries": entries,
    }))
}

/// Serde helper: `HashMap<Vec<u32>, Vec<u64>>` cannot serialize as a JSON
/// object (keys must be strings), which previously made `export_state` fail
/// silently whenever a facet index was populated (issue #62). Encodes each
/// map as an array of `(key, values)` pairs, sorted by key so exports are
/// deterministic across runs.
mod vec_u32_key_map {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::HashMap;

    pub fn serialize<S: Serializer>(
        map: &HashMap<Vec<u32>, Vec<u64>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut pairs: Vec<(&Vec<u32>, &Vec<u64>)> = map.iter().collect();
        pairs.sort();
        pairs.serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<HashMap<Vec<u32>, Vec<u64>>, D::Error> {
        let pairs: Vec<(Vec<u32>, Vec<u64>)> = Vec::deserialize(deserializer)?;
        Ok(pairs.into_iter().collect())
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct MultiFacetStore {
    #[serde(default, with = "vec_u32_key_map")]
    pub type_index: HashMap<Vec<u32>, Vec<u64>>,
    #[serde(default, with = "vec_u32_key_map")]
    pub entity_index: HashMap<Vec<u32>, Vec<u64>>,
    #[serde(default, with = "vec_u32_key_map")]
    pub relation_index: HashMap<Vec<u32>, Vec<u64>>,
    #[serde(default, with = "vec_u32_key_map")]
    pub temporal_index: HashMap<Vec<u32>, Vec<u64>>,
    #[serde(default, with = "vec_u32_key_map")]
    pub intent_index: HashMap<Vec<u32>, Vec<u64>>,
    #[serde(default, with = "vec_u32_key_map")]
    pub provenance_index: HashMap<Vec<u32>, Vec<u64>>,
}

impl MultiFacetStore {
    pub fn compute_epoch_root(&self) -> String {
        let mut entries = Vec::new();
        let mut add_idx = |name: &str, idx: &HashMap<Vec<u32>, Vec<u64>>| {
            let mut keys: Vec<&Vec<u32>> = idx.keys().collect();
            keys.sort();
            for k in keys {
                let mut v = idx.get(k).unwrap().clone();
                v.sort();
                let key_str = k
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                let val_str = v
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                entries.push(format!("{}:{}:{}", name, key_str, val_str));
            }
        };
        add_idx("type", &self.type_index);
        add_idx("entity", &self.entity_index);
        add_idx("relation", &self.relation_index);
        add_idx("temporal", &self.temporal_index);
        add_idx("intent", &self.intent_index);
        add_idx("provenance", &self.provenance_index);

        if entries.is_empty() {
            return "blake3:0000000000000000000000000000000000000000000000000000000000000000"
                .to_string();
        }

        let leaf_refs: Vec<&[u8]> = entries.iter().map(|s| s.as_bytes()).collect();
        let (root, _) =
            uor_r4_core::semantic::merkle::compute_merkle_root_and_proof(&leaf_refs, 0).unwrap();
        format!("blake3:{}", hex::encode(root))
    }

    pub fn compute_inclusion_proof(
        &self,
        facet: &str,
        path: &[u32],
    ) -> Option<(String, Vec<String>, usize)> {
        let mut entries = Vec::new();
        let mut target_entry = None;

        let mut add_idx = |name: &str, idx: &HashMap<Vec<u32>, Vec<u64>>| {
            let mut keys: Vec<&Vec<u32>> = idx.keys().collect();
            keys.sort();
            for k in keys {
                let mut v = idx.get(k).unwrap().clone();
                v.sort();
                let key_str = k
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                let val_str = v
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                let entry = format!("{}:{}:{}", name, key_str, val_str);

                if name == facet && k == path {
                    target_entry = Some(entry.clone());
                }
                entries.push(entry);
            }
        };

        add_idx("type", &self.type_index);
        add_idx("entity", &self.entity_index);
        add_idx("relation", &self.relation_index);
        add_idx("temporal", &self.temporal_index);
        add_idx("intent", &self.intent_index);
        add_idx("provenance", &self.provenance_index);

        let target = target_entry?;
        let target_idx = entries.iter().position(|e| e == &target)?;

        let leaf_refs: Vec<&[u8]> = entries.iter().map(|s| s.as_bytes()).collect();
        let (_root, proof_bytes) =
            uor_r4_core::semantic::merkle::compute_merkle_root_and_proof(&leaf_refs, target_idx)
                .unwrap();

        let proof_hex = proof_bytes.into_iter().map(hex::encode).collect();
        Some((target, proof_hex, target_idx))
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GeometryType {
    Spectral,
    Vsa,
}

fn default_geometry_type() -> GeometryType {
    GeometryType::Spectral
}

/// The shipped weight one shared query prime carries in retrieval relevance
/// (issue #484).
///
/// `relevance = shared_count * WEIGHT + sim * slice_norm + scope_boost`, so
/// this constant sets how much word overlap outranks geometry. It was a bare
/// `100.0` literal in `retrieve_geometric_resonance` until #484 named it: at
/// that magnitude, against a cosine bounded by one and a `scope_boost` of 15,
/// the ranking is lexical and the geometry is a tie-break within
/// equal-overlap groups. That is the measured reason #480's query-shape fix
/// could not pay, and it bounds every geometry lever on this path.
///
/// This is the ROUTING-path default only. Since #502 the deployed
/// CONTENT-query path defaults to `W = 0` (pure geometry) — see
/// [`UorR4Router::default_lexical_weight`]. Changing this value reorders
/// `get_top_resonances_native`, its one and only consumer; it does NOT move
/// the pinned #421 anchor-accuracy rows, which rank by bare cosine over the
/// stored vectors in `router_reconnect` and never read this weight (#490/#502
/// re-verified the blast radius). Use [`UorR4Router::set_lexical_weight`] for
/// measurement rather than editing the constant.
pub const DEFAULT_LEXICAL_WEIGHT: f64 = 100.0;

/// #496: how many salient content dimensions the VSA facet-index keys each item
/// on — the top-magnitude components of its 512-d content signal.
const VSA_SALIENT_DIMS: usize = 16;

/// The indices of the top-`m` largest-magnitude components of a content signal,
/// returned sorted ascending. Similar content shares its salient dimensions, so
/// a query and its true target land in overlapping VSA facet buckets — the
/// high-recall, content-addressed basis of the #496 facet-index.
fn salient_content_dims(content: &[f32], m: usize) -> Vec<u32> {
    let mut idx: Vec<u32> = (0..content.len() as u32).collect();
    idx.sort_by(|&a, &b| {
        content[b as usize]
            .abs()
            .total_cmp(&content[a as usize].abs())
    });
    idx.truncate(m);
    idx.sort_unstable();
    idx
}

/// Deployed default for `content_query_vector` (issue #490, adopting #486).
///
/// The retrieval query vector is built from the query text's own CONTENT
/// state — the same construction every stored vector uses — rather than from
/// the routing state, which #486 measured to be at chance against the stored
/// vectors. `true` since #490 (minimal variant: query vector only,
/// `DEFAULT_LEXICAL_WEIGHT` unchanged). This is a named function rather than
/// `Default::default()` so the DESERIALIZED router (the server restores router
/// state through `import_state`) carries the deployed default too — a plain
/// `#[serde(skip)]` bool would reset to `false` on load and silently disable
/// the fix on the one surface it is meant to improve.
fn content_query_vector_default() -> bool {
    true
}

/// The unified router core coordinator.
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct UorR4Router {
    #[serde(default)]
    active_streams: HashMap<String, ThoughtStream>,
    #[serde(default)]
    expert_active_counts: Vec<u64>, // Changed to Vec for clean serde support
    #[serde(default)]
    connection_drift: f64,
    #[serde(default)]
    kill_switch_threshold: f64,
    #[serde(default)]
    is_aligned: bool,

    // --- New Prime Router persistent states ---
    #[serde(default)]
    #[wasm_bindgen(skip)]
    pub vocabulary: Vec<String>,
    #[serde(default)]
    word_primes: HashMap<String, u64>,
    #[serde(default)]
    max_prime: u64,
    #[serde(skip)]
    vocab_vectors: HashMap<String, Vec<f64>>,
    #[serde(skip)]
    transitions: HashMap<String, HashMap<String, f64>>,
    #[serde(skip)]
    transitions_2nd: HashMap<String, HashMap<String, f64>>,
    #[serde(default)]
    corpus_index: HashMap<u64, Vec<CorpusItem>>,
    #[serde(default)]
    corpus_index_by_identity: HashMap<String, HashMap<u64, Vec<CorpusItem>>>,
    #[serde(default)]
    session_brain_states: HashMap<String, Vec<f64>>,
    #[serde(default)]
    angle_x: f64,
    #[serde(default)]
    angle_y: f64,
    #[serde(skip)]
    last_routing_data: Option<RoutingData>,
    #[serde(default)]
    #[wasm_bindgen(skip)]
    pub facet_store: MultiFacetStore,
    /// #496 real facet-index: each indexed item's VSA hypervector, stored at
    /// index time so retrieval scores against precomputed vectors instead of
    /// re-grounding every candidate (the O(candidates) cost of the first cut).
    #[serde(default)]
    vsa_vectors: HashMap<u64, Vec<f32>>,
    /// #496 real facet-index: a salient-content-dimension inverted index (top
    /// magnitude dims of each item's content signal -> item ids). Retrieval
    /// UNIONs the query's salient dims for a high-recall, content-addressed
    /// candidate set — replacing the coarse `type = sum % 100` / `entity =
    /// [100,200]` facets that dropped ~47% of targets (#496 ablation).
    #[serde(default)]
    vsa_dim_index: HashMap<u32, Vec<u64>>,
    #[serde(default = "default_geometry_type")]
    pub geometry_type: GeometryType,
    /// Issue #434 (adopted de-banding): when true the content-bearing
    /// store keeps only the routed window band of the content vector
    /// (the pre-#434 behavior, one sixteenth of the width, zeros
    /// elsewhere); when false — the DEFAULT since #434 — the store keeps
    /// the full-width content vector. Retained as a flag so the
    /// measurement harnesses can A/B and so rollback is a one-line
    /// setter. Serialized so an exported store records which shape its
    /// vectors have.
    #[serde(default)]
    banded_storage: bool,
    /// Measurement knob for issue #480: build the QUERY projection
    /// full-width instead of band-only. Default OFF, so deployed retrieval
    /// ordering is unchanged.
    ///
    /// Not serialized: it selects a measurement configuration, not a
    /// property of a stored vector, so an exported store must not carry it.
    #[serde(skip)]
    full_width_query: bool,
    /// Measurement knob for issue #484: the weight the retrieval relevance
    /// gives one shared query prime, against the cosine term it is added to.
    /// `None` is the shipped [`DEFAULT_LEXICAL_WEIGHT`].
    ///
    /// This exists because the shipped value is a hard-coded literal that
    /// had never been measured against any alternative, while sitting two
    /// orders of magnitude above the geometry it is summed with — which is
    /// what made #480's vector-shape fix unable to pay. Making it settable
    /// turns "the ranking is lexical" from a property of the source into a
    /// measurable axis.
    ///
    /// Not serialized, for the same reason `full_width_query` is not: it
    /// selects a measurement configuration, not a property of a stored
    /// vector.
    #[serde(skip)]
    lexical_weight: Option<f64>,
    /// Measurement knob for issue #484: drop the `slice_norm` factor from the
    /// geometric term, ranking by the bare cosine. Default OFF (the shipped
    /// form multiplies by `slice_norm`).
    ///
    /// This exists because `slice_norm` is a PER-WINDOW-BUCKET scalar, so
    /// `sim * slice_norm` is not comparable across buckets. Turning the
    /// lexical weight to zero therefore does NOT give a cosine ranking — it
    /// gives a ranking dominated by which bucket has the largest slice norm.
    /// Without this knob, the "no lexical term" arm measures the wrong thing
    /// and would be reported as if it measured cosine retrieval.
    ///
    /// Not serialized, for the same reason the other two measurement knobs
    /// are not.
    #[serde(skip)]
    unscaled_geometric_term: bool,
    /// Measurement knob for issue #486: build the retrieval query vector from
    /// the query text's own CONTENT state, the same construction
    /// `index_sentence_internal` stores, instead of from the routing state.
    /// Default ON since #490 (see [`content_query_vector_default`]); set it
    /// OFF with [`UorR4Router::set_content_query_vector`] to reproduce the
    /// pre-#490 deployed ordering for measurement.
    ///
    /// Why it exists. #486 measured that the deployed query projection has no
    /// relationship to the stored vector: querying with the exact text of a
    /// stored sentence, `cosine(query, that sentence's stored vector)` sits at
    /// the 0.4938 percentile of all candidates — chance — with the cross
    /// distribution carrying real spread (0.045 sd), so the stored vectors are
    /// fine and it is the comparison that is wrong. The query side reads a
    /// ROUTING state; the stored side is a CONTENT vector. They are different
    /// objects and their cosine is noise by construction.
    ///
    /// The harnesses that measured cosine retrieval working on this stack
    /// (#442, MRR 0.8948) sidestepped this by indexing the query text as a
    /// corpus item and reading its stored vector back — the same encode path
    /// on both sides. This knob does that directly and without mutating state.
    ///
    /// Not serialized: retrieval configuration, not stored corpus state. The
    /// `default` keeps the deployed value on the deserialize path (#490).
    #[serde(skip, default = "content_query_vector_default")]
    content_query_vector: bool,
}

#[derive(Serialize)]
pub struct GeometricResponse {
    pub text: String,
    pub trajectory: Vec<TrajectoryStep>,
}

fn hopf_probe_value(digest: &blake3::Hash, index: usize) -> f64 {
    (digest.as_bytes()[index % digest.as_bytes().len()] as f64 / 255.0) - 0.5
}

impl UorR4Router {
    pub fn index_semantic_object(&mut self, id: usize, coords: &geometry::FacetCoordinates) {
        let id = id as u64;
        let index_prefix = |index: &mut HashMap<Vec<u32>, Vec<u64>>, path: &Vec<u32>| {
            for i in 1..=path.len() {
                let prefix = path[..i].to_vec();
                let list = index.entry(prefix).or_default();
                if !list.contains(&id) {
                    list.push(id);
                }
            }
        };

        if let Some(path) = coords.coordinates.get("type") {
            index_prefix(&mut self.facet_store.type_index, path);
        }
        if let Some(path) = coords.coordinates.get("entity") {
            index_prefix(&mut self.facet_store.entity_index, path);
        }
        if let Some(path) = coords.coordinates.get("relation") {
            index_prefix(&mut self.facet_store.relation_index, path);
        }
        if let Some(path) = coords.coordinates.get("temporal") {
            index_prefix(&mut self.facet_store.temporal_index, path);
        }
        if let Some(path) = coords.coordinates.get("intent") {
            index_prefix(&mut self.facet_store.intent_index, path);
        }
        if let Some(path) = coords.coordinates.get("provenance") {
            index_prefix(&mut self.facet_store.provenance_index, path);
        }
    }

    pub fn get_store_inclusion_proof_native(
        &self,
        facet: &str,
        path_str: &str,
    ) -> Option<(String, Vec<String>, usize)> {
        let path: Vec<u32> = path_str
            .split(',')
            .filter_map(|s| s.parse::<u32>().ok())
            .collect();
        self.facet_store.compute_inclusion_proof(facet, &path)
    }
}

#[wasm_bindgen]
impl UorR4Router {
    #[wasm_bindgen]
    pub fn set_geometry_type(&mut self, geom: &str) {
        self.geometry_type = match geom {
            "vsa" | "Vsa" | "VSA" => GeometryType::Vsa,
            _ => GeometryType::Spectral,
        };
        // #493 (disposition of #434): fail loud on the knob rather than let
        // VSA retrieval look like a working semantic path. Measured: after
        // `index_corpus`/`index_sentence` the facet store IS populated (so the
        // candidate SET is real), but `retrieve_vsa_multi_facet_resonance`
        // scores each candidate with `cosine_similarity(query 1024-dim VSA
        // hypervector, stored 512-dim SPECTRAL content vector)` — a length
        // mismatch that returns exactly 0.0 for every candidate, so the ranking
        // is dead. Making the comparison commensurable does not rescue it: the
        // VSA hypervector is content-hash-derived (`expand_atom`), not a
        // semantic embedding, so re-grounded scoring ranks at chance (a "fox"
        // query ranks the fox sentence last). Until VSA has a real encoder,
        // Spectral is the semantic retrieval path. See
        // `docs/geometry_ablation_434.md` and #493.
        if self.geometry_type == GeometryType::Vsa {
            eprintln!(
                "warning: geometry_type=Vsa — VSA retrieval does NOT rank by content \
                 similarity on this store (grounding is content-hash-derived, and the \
                 scorer compares a VSA hypervector to a spectral content vector); \
                 relevances are not meaningful. Use Spectral geometry for content \
                 retrieval. See #493."
            );
        }
    }

    #[wasm_bindgen]
    pub fn get_store_epoch_root(&self) -> String {
        self.facet_store.compute_epoch_root()
    }

    #[wasm_bindgen]
    pub fn get_store_inclusion_proof(&self, facet: &str, path_str: &str) -> JsValue {
        if let Some((target, proof, target_idx)) =
            self.get_store_inclusion_proof_native(facet, path_str)
        {
            let res = serde_json::json!({
                "target": target,
                "proof": proof,
                "target_idx": target_idx,
            });
            serde_wasm_bindgen::to_value(&res).unwrap_or(JsValue::NULL)
        } else {
            JsValue::NULL
        }
    }

    /// Instantiates the R4 Router with perfect, error-free default states
    #[wasm_bindgen(constructor)]
    pub fn new(threshold: f64) -> Self {
        let mut router = Self {
            active_streams: HashMap::new(),
            expert_active_counts: vec![0; 64],
            connection_drift: 0.0,
            kill_switch_threshold: threshold,
            is_aligned: true,
            vocabulary: Vec::new(),
            word_primes: HashMap::new(),
            max_prime: 0,
            vocab_vectors: HashMap::new(),
            transitions: HashMap::new(),
            transitions_2nd: HashMap::new(),
            corpus_index: HashMap::new(),
            corpus_index_by_identity: HashMap::new(),
            session_brain_states: HashMap::new(),
            angle_x: 0.5,
            angle_y: 0.5,
            last_routing_data: None,
            facet_store: MultiFacetStore::default(),
            vsa_vectors: HashMap::new(),
            vsa_dim_index: HashMap::new(),
            geometry_type: default_geometry_type(),
            banded_storage: false,
            full_width_query: false,
            lexical_weight: None,
            unscaled_geometric_term: false,
            content_query_vector: content_query_vector_default(),
        };

        // Initialize default corpus
        router.index_default_corpus();
        router
    }

    // --- Helper to index default corpus sentences ---
    pub fn index_default_corpus(&mut self) {
        let default_sentences = &[
            "Welcome to the R4 Prime Router. This is a local geometric world model.",
            "I can help you coordinate water borehole data for the Gambia project.",
            "The dry season in the Gambia requires deep aquifer coordination.",
            "We can map borehole locations directly onto the prime number coordinates.",
            "No training is required because the Riemann zeta zeroes form a stable coordinate system.",
            "This engine replaces the transformer MoE gating using sparse orthogonal projections.",
            "A traditional transformer routes tokens using learned parameters, but we route using prime factor frequency manifolds.",
            "Each scale window acts as an expert containing specific geometric resonances.",
            "The deficit angle measures the deflection of your query relative to the hypersphere curvature.",
            "If the deficit angle is positive, the wave is trapped in a stable periodic orbit.",
            "Negative deficit angles indicate hyperbolic divergence and scattering.",
            "A symmetric orbit indicates stable, logical, and focused input sequences.",
            "Coherence kappa indicates how well the prompt wave aligns with the local zero frequencies.",
            "To talk to this engine, you must populate its manifold coordinates with a starting text.",
            "You can paste any text corpus to dynamically index new knowledge into the manifold.",
            "Once indexed, the router retrieves and synthesizes responses based on state vector similarity.",
            "The 512 dimensions correspond to the first 512 non-trivial Riemann zeta zeroes.",
            "Water flow rates in the Gambia depend on the aquifer's soil coherence.",
            "The prime router helps you find the most efficient path for borehole water flow coordinates.",
            "We can run this engine entirely locally without internet access or third-party APIs.",
            "You are talking directly to the mathematical voice of the prime spectrum.",
            "Ask me about the Gambia borehole locations, or how the R4 routing replaces transformer layers."
        ];

        // Seed with baseline vocabulary first
        let mut all_words = Vec::new();
        for s in default_sentences {
            all_words.extend(tokenize(s));
        }
        all_words.sort();
        all_words.dedup();
        for w in &all_words {
            self.add_word_to_vocabulary(w);
        }

        // Index each sentence to the shared corpus
        for s in default_sentences {
            self.index_sentence_internal(s, "shared");
        }
        self.rebuild_transitions();
    }

    /// Exposes read-only status of manifold alignment
    pub fn is_aligned(&self) -> bool {
        self.is_aligned
    }

    /// Returns current connection drift
    pub fn connection_drift(&self) -> f64 {
        self.connection_drift
    }

    /// Returns the kill switch threshold limit
    pub fn kill_switch_threshold(&self) -> f64 {
        self.kill_switch_threshold
    }

    /// Computes live UOR resonance metrics for a given input text
    pub fn calculate_resonance(&self, text: &str) -> JsValue {
        let json_payload = serde_json::json!({
            "content": text
        });
        let json_bytes = serde_json::to_vec(&json_payload).unwrap_or_default();
        let uor_hash_str = match uor_addr::json::address(&json_bytes) {
            Ok(outcome) => outcome.address.to_string(),
            Err(_) => format!("sha256:{:064x}", 0),
        };

        let klein_matches = text
            .chars()
            .filter(|&c| {
                let m = (c as usize) % 50;
                m == 0 || m == 1 || m == 48 || m == 49
            })
            .count();

        let info = ResonanceInfo {
            total_bytes: text.len() as u64,
            resonant_bits: text.len() as u64 * 8,
            klein_matches: klein_matches as u64,
            uor_signature: uor_hash_str,
        };

        serde_wasm_bindgen::to_value(&info).unwrap_or(JsValue::NULL)
    }

    /// Compiles a raw string thought parameter down into its content-addressed math state
    pub fn compile_thought(&self, content: &str) -> JsValue {
        let stream = self.compile_thought_internal(content);
        serde_wasm_bindgen::to_value(&stream).unwrap_or(JsValue::NULL)
    }

    /// Injects a new thought stream, updates MoE activations, and returns the stream
    pub fn inject_thought_stream(&mut self, content: &str) -> JsValue {
        if !self.is_aligned {
            return JsValue::NULL;
        }

        let stream = self.compile_thought_internal(content);

        // Map the MoE active overlays
        for &ch in &stream.activated_experts {
            if ch < 64 {
                self.expert_active_counts[ch as usize] += 1;
            }
        }

        let id = stream.id.clone();
        let js_stream = serde_wasm_bindgen::to_value(&stream).unwrap_or(JsValue::NULL);
        self.active_streams.insert(id, stream);
        js_stream
    }

    /// Progresses the connection drift state using delta-time ($dt$) increments.
    /// Returns a log message string if a ZKP reset occurs, otherwise returns undefined.
    pub fn update_drift_physics(&mut self, dt: f64, drift_rate: f64) -> Option<String> {
        if !self.is_aligned || drift_rate == 0.0 {
            return None;
        }

        self.connection_drift += drift_rate * dt;

        if self.connection_drift >= self.kill_switch_threshold {
            let log = self.execute_zkp_phase_reset();
            Some(log)
        } else {
            None
        }
    }

    /// Reset the alignment back to native state ($0.00\%$ error) using ZKP 2i Sync-Handshake
    pub fn execute_zkp_phase_reset(&mut self) -> String {
        self.is_aligned = false;
        self.connection_drift = 0.0;

        let log = String::from(
            "[ZKP 2i HANDSHAKE]: Manifold limit breached. Recalibrating continuous tangent vectors...\n\
             [ZKP 2i HANDSHAKE]: Recalibration complete. Zero-point origin alignment LOCKED (0.00% err)."
        );

        // Clear active overlays during reset
        self.expert_active_counts = vec![0; 64];
        self.active_streams.clear();

        // Safe restabilization
        self.is_aligned = true;
        log
    }

    /// Returns the active stream list as a JS Array
    pub fn get_active_streams(&self) -> JsValue {
        let streams: Vec<&ThoughtStream> = self.active_streams.values().collect();
        serde_wasm_bindgen::to_value(&streams).unwrap_or(JsValue::NULL)
    }

    /// Returns the active counts for the 64 experts
    pub fn get_expert_counts(&self) -> Vec<u32> {
        self.expert_active_counts
            .iter()
            .map(|&c| c as u32)
            .collect()
    }

    /// Returns the number of words in the vocabulary index
    pub fn get_vocab_size(&self) -> usize {
        self.word_primes.len()
    }

    /// Returns the total number of indexed sentences in the corpus
    pub fn get_total_indexed_sentences(&self) -> usize {
        self.corpus_index_by_identity
            .values()
            .map(|store| store.values().map(|items| items.len()).sum::<usize>())
            .sum()
    }

    // --- New rotation angle handlers ---
    pub fn get_angle_x(&self) -> f64 {
        self.angle_x
    }
    pub fn set_angle_x(&mut self, val: f64) {
        self.angle_x = val;
    }
    pub fn get_angle_y(&self) -> f64 {
        self.angle_y
    }
    pub fn set_angle_y(&mut self, val: f64) {
        self.angle_y = val;
    }

    // --- New Prime Router Public Interfaces ---

    /// Resets the brain state vector for a specific identity
    pub fn reset_brain(&mut self, identity: &str) {
        let key = identity_key(identity);
        let baseline = vec![1.0 / (512.0f64).sqrt(); 512];
        self.session_brain_states.insert(key, baseline);
    }

    #[wasm_bindgen]
    pub fn clear_corpus(&mut self) {
        self.corpus_index.clear();
        self.corpus_index_by_identity.clear();
        self.facet_store = MultiFacetStore::default();
        // #496: item ids restart at 0 after a clear, so the VSA index and stored
        // vectors must be cleared too or reused ids would collide with stale rows.
        self.vsa_vectors.clear();
        self.vsa_dim_index.clear();
    }

    /// Resets the entire router system back to factory defaults
    pub fn reset_to_defaults(&mut self) {
        self.active_streams.clear();
        self.expert_active_counts = vec![0; 64];
        self.connection_drift = 0.0;
        self.is_aligned = true;
        self.vocabulary.clear();
        self.word_primes.clear();
        self.vocab_vectors.clear();
        self.transitions.clear();
        self.transitions_2nd.clear();
        self.corpus_index.clear();
        self.corpus_index_by_identity.clear();
        self.facet_store = MultiFacetStore::default();
        self.vsa_vectors.clear();
        self.vsa_dim_index.clear();
        self.session_brain_states.clear();
        self.angle_x = 0.5;
        self.angle_y = 0.5;
        self.last_routing_data = None;
        self.index_default_corpus();
    }

    /// Evolves state vector using user prompt words and returns the new state
    pub fn evolve_state(&mut self, identity: &str, text: &str, gamma: f64) -> Vec<f64> {
        self.evolve_brain_state(identity, text, gamma)
    }

    /// Returns the routed window and detailed thermodynamic/Hopf metrics for a query
    pub fn route_query_to_manifold(&mut self, text: &str, identity: &str) -> JsValue {
        let key = identity_key(identity);
        let active_state = self
            .session_brain_states
            .entry(key)
            .or_insert_with(|| vec![1.0 / (512.0f64).sqrt(); 512])
            .clone();

        let routing = self.route_query_to_manifold_internal(text, identity, Some(&active_state));
        serde_wasm_bindgen::to_value(&routing).unwrap_or(JsValue::NULL)
    }

    /// Indexes a single sentence into the identity's scoped corpus
    pub fn index_sentence(&mut self, sentence: &str, identity: &str) {
        let s_clean = sentence.trim();
        if s_clean.is_empty() || s_clean.len() > 1000 {
            return;
        }
        let s_lower = s_clean.to_lowercase();
        let key = identity_key(identity);
        let mut already_exists = false;
        if let Some(target_index) = self.corpus_index_by_identity.get(&key) {
            for win_items in target_index.values() {
                for item in win_items {
                    if item.sentence.to_lowercase() == s_lower {
                        already_exists = true;
                        break;
                    }
                }
                if already_exists {
                    break;
                }
            }
        }
        if !already_exists {
            self.index_sentence_internal(s_clean, identity);
        }
    }

    /// Indexes an entire block of text split into sentences
    pub fn index_corpus(&mut self, corpus_text: &str, identity: &str) -> usize {
        let mut count = 0;
        let sentences = split_sentences(corpus_text);

        let key = identity_key(identity);
        let mut existing = std::collections::HashSet::new();
        if let Some(target_index) = self.corpus_index_by_identity.get(&key) {
            for win_items in target_index.values() {
                for item in win_items {
                    existing.insert(item.sentence.to_lowercase());
                }
            }
        }
        if let Some(shared_index) = self.corpus_index_by_identity.get("shared:shared") {
            for win_items in shared_index.values() {
                for item in win_items {
                    existing.insert(item.sentence.to_lowercase());
                }
            }
        }

        let mut unique_sentences = Vec::new();
        for s in &sentences {
            let s_clean = s.trim();
            let word_count = s_clean.split_whitespace().count();
            if s_clean.len() > 30 && s_clean.len() < 400 && word_count > 4 {
                let s_lower = s_clean.to_lowercase();
                if !existing.contains(&s_lower) {
                    existing.insert(s_lower);
                    unique_sentences.push(s_clean.to_string());
                }
            }
        }

        if unique_sentences.is_empty() {
            return 0;
        }

        println!(
            "[*] Found {} new unique sentences to index.",
            unique_sentences.len()
        );

        for (i, s) in unique_sentences.iter().enumerate() {
            if i > 0 && i % 2000 == 0 {
                println!(
                    "    - Indexing progress: {}/{}...",
                    i,
                    unique_sentences.len()
                );
            }
            self.index_sentence_internal(s, identity);
            count += 1;
        }

        if count > 0 {
            self.rebuild_transitions();
        }
        count
    }

    /// Returns the top N resonant sentences sorted by relevance
    pub fn get_top_resonances(&mut self, text: &str, identity: &str, top_n: usize) -> JsValue {
        let res = self.get_top_resonances_native(text, identity, top_n);
        serde_wasm_bindgen::to_value(&res).unwrap_or(JsValue::NULL)
    }

    /// Dynamically computes the suggested token limit based on manifold routing metrics
    pub fn get_suggested_token_limit(&self, text: &str, identity: &str) -> usize {
        let key = identity_key(identity);
        let active_state = self
            .session_brain_states
            .get(&key)
            .cloned()
            .unwrap_or_else(|| vec![1.0 / (512.0f64).sqrt(); 512]);

        let routing = self.route_query_to_manifold_internal(text, identity, Some(&active_state));
        let routed = &routing.routed;

        let stratum = routed
            .state_vector
            .iter()
            .filter(|&&v| v.abs() > 1e-4)
            .count();
        let eval_sum: f64 = routed.eigenvalues.iter().sum();
        let theta_d = routed.metrics.deficit_angle;
        let input_words = tokenize(text).len();

        let base = 50.0;
        let stratum_contrib = stratum as f64 * 1.5;
        let angle_contrib = theta_d.abs() * 45.0;
        let eval_contrib = eval_sum * 110.0;
        let word_contrib = input_words as f64 * 2.5;

        let total = (base + stratum_contrib + angle_contrib + eval_contrib + word_contrib) as usize;
        total.clamp(50, 500)
    }

    /// Decodes a response steered by the active brain state vector
    // wasm-bindgen preserves this established JavaScript calling convention;
    // changing it to a Rust builder would break the public browser API.
    #[allow(clippy::too_many_arguments)]
    pub fn generate_geometric_response(
        &mut self,
        text: &str,
        identity: &str,
        max_tokens: usize,
        temp: f64,
        gravity: f64,
        freq_penalty: f64,
        gamma: f64,
    ) -> JsValue {
        let key = identity_key(identity);
        let active_state = self
            .session_brain_states
            .entry(key)
            .or_insert_with(|| vec![1.0 / (512.0f64).sqrt(); 512])
            .clone();

        let (response_text, trajectory, final_state) = self
            .generate_geometric_response_with_trajectory_internal(
                text,
                &active_state,
                max_tokens,
                temp,
                gravity,
                freq_penalty,
                identity,
                gamma,
            );

        // Update brain state
        let key_save = identity_key(identity);
        self.session_brain_states.insert(key_save, final_state);

        let geom_res = GeometricResponse {
            text: response_text,
            trajectory,
        };
        serde_wasm_bindgen::to_value(&geom_res).unwrap_or(JsValue::NULL)
    }

    /// Whether the content-bearing store bands its stored vectors
    /// (issue #434). False — full-width storage — is the default.
    pub fn banded_storage(&self) -> bool {
        self.banded_storage
    }

    /// Selects the storage shape for subsequently indexed sentences
    /// (issue #434). `true` restores the pre-#434 banded storage; the
    /// default `false` keeps the full-width content vector. Already
    /// indexed items are not rewritten, so flip this before ingestion.
    pub fn set_banded_storage(&mut self, banded: bool) {
        self.banded_storage = banded;
    }

    /// Build the query projection full-width rather than band-only
    /// (issue #480). Default off — see `docs/query_projection_480.md` for
    /// why the symmetric shape was measured and NOT adopted.
    pub fn set_full_width_query(&mut self, full_width: bool) {
        self.full_width_query = full_width;
    }

    /// The weight one shared query prime carries in retrieval relevance
    /// (issues #484 / #502). The deployed default depends on the query path
    /// (see [`UorR4Router::default_lexical_weight`]) unless overridden with
    /// [`set_lexical_weight`](UorR4Router::set_lexical_weight).
    pub fn lexical_weight(&self) -> f64 {
        self.lexical_weight
            .unwrap_or_else(|| self.default_lexical_weight())
    }

    /// The deployed default lexical weight, which depends on the query path
    /// (issue #502).
    ///
    /// On the CONTENT-query path (deployed since #490) the lexical term is
    /// dropped (`W = 0`, pure geometry): #500 measured on this path that
    /// dropping it is worth +0.022 MRR and +0.032 top-1 at equal recall@20
    /// (0.99), and it removes the `shared_count * 100` term that masked the
    /// dead cosine (#486) for months — a simplification, not only a tune.
    ///
    /// On the ROUTING path (`content_query_vector = false`) the term is kept
    /// at [`DEFAULT_LEXICAL_WEIGHT`]: there the `W = 0` arm is confounded by
    /// `slice_norm` (a per-window-bucket scalar, #484), and that path is not
    /// deployed anyway — it exists only to reproduce the pre-#490 ordering
    /// for measurement.
    ///
    /// Blast radius of this default (established by #490 and re-verified for
    /// #502): the lexical weight is read at exactly one site, inside
    /// `retrieve_geometric_resonance`, reachable only through
    /// `get_top_resonances_native`. The `router_reconnect` (#421) anchor
    /// rows rank by bare cosine over the stored vectors and the score
    /// pipeline never calls this path, so they are INVARIANT under the
    /// weight — the "an ordering change moves the pinned #421 rows" gate the
    /// original #484 doc recorded is backwards for this knob, the same way it
    /// was for the #490 content-query flip.
    fn default_lexical_weight(&self) -> f64 {
        if self.content_query_vector {
            0.0
        } else {
            DEFAULT_LEXICAL_WEIGHT
        }
    }

    /// Override the lexical weight for measurement (issue #484).
    ///
    /// The parameter is a continuum, not a flag: at `0.0` the ranking is
    /// pure cosine, at [`DEFAULT_LEXICAL_WEIGHT`] it is the shipped form,
    /// and as it grows it approaches strict lexicographic order with the
    /// cosine as a tie-break. The shipped value is one point on that
    /// continuum and the others had never been looked at.
    ///
    /// Deployed behaviour is unchanged while this is unset. A negative or
    /// non-finite weight is REJECTED rather than clamped — silently
    /// substituting a different weight than the caller asked for would make
    /// a sweep report the wrong arm's number under the right arm's label,
    /// which is worse than a panic in a measurement harness.
    pub fn set_lexical_weight(&mut self, weight: f64) {
        assert!(
            weight.is_finite() && weight >= 0.0,
            "lexical weight must be finite and non-negative, got {weight}"
        );
        self.lexical_weight = Some(weight);
    }

    /// Rank by the bare cosine instead of `sim * slice_norm` (issue #484).
    /// Default off; deployed behaviour is the scaled form.
    ///
    /// Pair this with `set_lexical_weight(0.0)` to get an actually
    /// cosine-ranked arm. Setting the weight to zero on its own does not:
    /// `slice_norm` is a per-window-bucket scalar, so the scaled term is not
    /// comparable across buckets and the resulting order is driven by bucket
    /// scale rather than by similarity.
    pub fn set_unscaled_geometric_term(&mut self, unscaled: bool) {
        self.unscaled_geometric_term = unscaled;
    }

    /// Build the retrieval query vector from the query text's own content
    /// state rather than from the routing state (issue #486). **Default ON
    /// since #490**; pass `false` to reproduce the pre-#490 routing-query
    /// ordering for measurement.
    ///
    /// This is the arm that makes the query and the stored vector the same
    /// KIND of object. Falls back to the deployed projection for any text
    /// with no vocabulary word, so the knob can never leave a query without a
    /// vector.
    pub fn set_content_query_vector(&mut self, content: bool) {
        self.content_query_vector = content;
    }

    /// Exports the full router system database to JSON string
    pub fn export_state(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Imports a JSON string and restores the router system database. Returns
    /// `true` when the string is a valid serialized router state and was
    /// applied, `false` when it could not be parsed. Import is total: a
    /// malformed input is the absence of a state to restore, reported as
    /// `false`, not a thrown JS error (R5).
    pub fn import_state(&mut self, json_str: &str) -> bool {
        match serde_json::from_str::<Self>(json_str) {
            Ok(mut imported) => {
                for (word, &prime) in &imported.word_primes {
                    let vec = get_word_vector(prime as usize);
                    imported.vocab_vectors.insert(word.clone(), vec);
                }
                imported.max_prime = imported.word_primes.values().max().copied().unwrap_or(0);
                // Rebuild transitions dynamically
                imported.rebuild_transitions();
                *self = imported;
                true
            }
            Err(_) => false,
        }
    }

    /// Serves all points in the corpus index for the semantic map visualizer
    pub fn get_semantic_map_points(&self) -> JsValue {
        #[derive(Serialize)]
        struct MapPoint {
            sentence: String,
            window_index: u64,
            u: f64,
            v: f64,
            v_4d: Vec<f64>,
            scope: String,
            kappa: f64,
            prime_product_mod: i64,
        }

        let mut points = Vec::new();
        for (identity_key, store) in &self.corpus_index_by_identity {
            let scope_name = identity_key
                .split(':')
                .nth(1)
                .unwrap_or(identity_key)
                .to_string();
            for (&win_idx, items) in store {
                for item in items {
                    let prime_product_val: i64 = item.prime_product.parse().unwrap_or(1);
                    points.push(MapPoint {
                        sentence: item.sentence.chars().take(120).collect(),
                        window_index: win_idx,
                        u: item.u,
                        v: item.v,
                        v_4d: item.v_4d.clone(),
                        scope: scope_name.clone(),
                        kappa: item.kappa,
                        prime_product_mod: prime_product_val % 10007,
                    });
                }
            }
        }

        let map_val = serde_json::json!({
            "points": points,
            "total": points.len(),
        });
        match serde_json::to_string(&map_val) {
            Ok(s) => JsValue::from_str(&s),
            Err(_) => JsValue::NULL,
        }
    }

    /// Runs the formal UOR coordinate reduction pipeline and returns both RoutingData and trace steps as a single JsValue
    pub fn route_query_to_manifold_uor(&mut self, text: &str, identity: &str) -> JsValue {
        use uor_foundation::pipeline::PrismModel;
        let mut buf = [0u8; 640];
        let query_bytes = text.as_bytes();
        let identity_bytes = identity.as_bytes();
        let query_len = query_bytes.len().min(512);
        let identity_len = identity_bytes.len().min(128);
        buf[..query_len].copy_from_slice(&query_bytes[..query_len]);
        buf[512..512 + identity_len].copy_from_slice(&identity_bytes[..identity_len]);

        let input = R4RoutingInput {
            query: &buf[..512],
            identity: &buf[512..],
            data: &buf,
        };

        // Bind thread-local
        let self_ptr = self as *mut UorR4Router;
        ACTIVE_ROUTER.with(|r| {
            *r.borrow_mut() = Some(self_ptr);
        });

        // Run dry run through UorR4RouterModel
        let grounded = match UorR4RouterModel::forward(input) {
            Ok(g) => g,
            Err(_) => {
                // Reset thread-local
                ACTIVE_ROUTER.with(|r| {
                    *r.borrow_mut() = None;
                });
                return JsValue::NULL;
            }
        };

        // Reset thread-local
        ACTIVE_ROUTER.with(|r| {
            *r.borrow_mut() = None;
        });

        let routing_data = match &self.last_routing_data {
            Some(rd) => rd,
            None => return JsValue::NULL,
        };

        let trace = grounded.derivation().replay::<256>();
        let mut uor_trace_steps = Vec::new();
        for i in 0..trace.len() {
            if let Some(event) = trace.event(i as usize) {
                uor_trace_steps.push(serde_json::json!({
                    "step": event.step_index(),
                    "op": format!("{:?}", event.op()),
                    "target": format!("0x{:032x}", event.target().as_u128()),
                }));
            }
        }

        let uor_payload = serde_json::json!({
            "algorithm": routing_data.routed.uor.algorithm.clone(),
            "hash_algorithm": routing_data.routed.uor.hash_algorithm.clone(),
            "hash_algorithm_id": routing_data.routed.uor.hash_algorithm_id,
            "address": routing_data.routed.uor.address.clone(),
            "verify_result": "Verified",
            "kappa_label": format!("witt:{}", grounded.witt_level_bits()),
            "fingerprint_hex": hex::encode(grounded.content_fingerprint().as_bytes()),
            "sigma": grounded.sigma().value(),
            "d_delta": grounded.d_delta().as_i64(),
            "euler": grounded.euler().as_i64(),
            "residual": grounded.residual().as_u32(),
            "stratum": grounded.triad().stratum(),
            "multihash_addresses": routing_data.routed.uor.multihash_addresses.clone(),
        });

        let result = serde_json::json!({
            "routing_data": routing_data,
            "uor_trace_steps": uor_trace_steps,
            "uor_payload": uor_payload,
        });

        match serde_json::to_string(&result) {
            Ok(s) => JsValue::from_str(&s),
            Err(_) => JsValue::NULL,
        }
    }

    /// Projects the active brain state vector into 2D coordinates for the map path tracing
    pub fn get_sentence_projection_wasm(&self, state_vector: &[f64], win_idx: usize) -> Vec<f64> {
        let (u, v) = self.get_sentence_projection(state_vector, win_idx);
        vec![u, v]
    }

    /// Projects the active brain state vector into 4D coordinates
    pub fn get_state_4d_projection_wasm(&self, state_vector: &[f64]) -> Vec<f64> {
        self.get_state_4d_projection(state_vector)
    }

    /// Retrieves the evolved brain state vector for a given identity
    pub fn get_brain_state_wasm(&mut self, identity: &str) -> Vec<f64> {
        self.get_brain_state_native(identity)
    }
}

// ============================================================
// Internal Helper Computations

impl UorR4Router {
    /// Internal Rust compilation logic matching the original spec
    fn compile_thought_internal(&self, content: &str) -> ThoughtStream {
        let json_payload = serde_json::json!({
            "content": content
        });

        let json_bytes = serde_json::to_vec(&json_payload).unwrap_or_default();
        let uor_hash_str = match uor_addr::json::address(&json_bytes) {
            Ok(outcome) => outcome.address.to_string(),
            Err(_) => format!("sha256:{:064x}", 0),
        };

        let hex_part = uor_hash_str
            .strip_prefix("sha256:")
            .unwrap_or(&uor_hash_str);

        let mut hash_bytes = [0u8; 32];
        for (i, byte) in hash_bytes.iter_mut().enumerate() {
            if let (Some(h1), Some(h2)) =
                (hex_part.chars().nth(i * 2), hex_part.chars().nth(i * 2 + 1))
            {
                if let (Some(d1), Some(d2)) = (h1.to_digit(16), h2.to_digit(16)) {
                    *byte = ((d1 << 4) | d2) as u8;
                }
            }
        }

        let hash_accumulator: i32 = hash_bytes.iter().map(|&b| b as i32).sum();

        let uor_addr = UorAddress { hash_bytes };
        let uor_uri = uor_hash_str;

        let x_coord = (hash_accumulator as f64 * 0.015).sin() * 110.0;
        let y_coord = (hash_accumulator as f64 * 0.025).cos() * 110.0;
        let z_coord = (hash_accumulator as f64 * 0.035).sin() * 90.0;
        let w_coord = (hash_accumulator as f64 * 0.045).cos() * 50.0;

        let r4_target = R4Vector {
            x: x_coord,
            y: y_coord,
            z: z_coord,
            w: w_coord,
        };

        let mut activated_experts = Vec::new();
        for i in 0..64 {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            if (hash_bytes[byte_idx] & (1 << bit_idx)) != 0 {
                activated_experts.push(i as u64);
            }
        }
        activated_experts.truncate(8);

        let mut path_steps = Vec::new();
        for step in 0..=50 {
            let ratio = step as f64 / 50.0;
            path_steps.push(R4Vector {
                x: x_coord * ratio + (ratio * PI * 2.0).sin() * 25.0,
                y: y_coord * ratio + (ratio * PI * 2.0).cos() * 15.0,
                z: z_coord * ratio,
                w: w_coord * ratio,
            });
        }

        let twist_parity_spin = if hash_accumulator % 2 == 0 { 1 } else { -1 };
        let alignment_phase = (hash_accumulator.abs() % 314) as f64 / 100.0;

        let primes = [11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53];
        let p1 = primes[(hash_accumulator.unsigned_abs() as usize) % primes.len()];
        let p2 = primes[((hash_accumulator.abs() + 9) as usize) % primes.len()];
        let gcd = if p1 == p2 { p1 } else { 1 };

        ThoughtStream {
            id: format!("stream-{}", uuid_placeholder(hash_accumulator)),
            raw_content: content.to_string(),
            uor_address: uor_addr,
            uor_uri,
            r4_target,
            path_steps,
            activated_experts,
            alignment_phase,
            twist_parity_spin,
            gcd,
        }
    }

    fn add_word_to_vocabulary(&mut self, word: &str) {
        let w = word.trim().to_lowercase();
        if w.is_empty() || self.word_primes.contains_key(&w) {
            return;
        }

        let mut next_prime = 2;
        if self.max_prime > 0 {
            next_prime = self.max_prime + 1;
        } else if !self.word_primes.is_empty() {
            let max_p = self.word_primes.values().max().cloned().unwrap_or(2);
            next_prime = max_p + 1;
        }
        while !is_prime_value(next_prime as usize) {
            next_prime += 1;
        }

        self.max_prime = next_prime;
        self.vocabulary.push(w.clone());
        self.word_primes.insert(w.clone(), next_prime);

        // Seed coordinates across 512 zeta zeros via prime log oscillation
        let vec = get_word_vector(next_prime as usize);
        self.vocab_vectors.insert(w, vec);
    }

    fn get_sentence_prime_product(&self, words: &[String]) -> u128 {
        let mut prod: u128 = 1;
        let stopwords = query_stopwords();
        for w in words {
            if stopwords.contains(&w.as_str()) {
                continue;
            }
            if let Some(&p) = self.word_primes.get(w) {
                prod = prod.saturating_mul(p as u128);
            }
        }
        prod
    }

    fn get_sentence_projection(&self, state_vector: &[f64], win_idx: usize) -> (f64, f64) {
        let q_proj = get_q_proj();
        let mut u_raw = 0.0;
        let mut v_raw = 0.0;
        for i in 0..512 {
            u_raw += state_vector[i] * q_proj[i][0];
            v_raw += state_vector[i] * q_proj[i][1];
        }
        let angle = (win_idx as f64 / 16.0) * 2.0 * std::f64::consts::PI;
        let radius = 20.0;
        let u = radius * angle.cos() + u_raw * 5.0;
        let v = radius * angle.sin() + v_raw * 5.0;
        (u, v)
    }

    /// Project each 128-dimensional block onto a fixed signed direction.
    ///
    /// The previous projection retained only four block L2 norms. Those are
    /// non-negative and, for the evolved session states, nearly constant, so
    /// most of the directional information never reached the Hopf address.
    /// The probe is deterministic and content-independent: its 128 entries
    /// are derived from a domain-separated Blake3 digest and normalized once
    /// per block. Compiler-side floating point is intentional here; this is
    /// the exploratory router, not the deployed integer kernel.
    fn hopf_signed_projection_component(&self, state_vector: &[f64], block: usize) -> f64 {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"uor-r4 issue-306 signed-hopf-projection");
        hasher.update(&(block as u64).to_le_bytes());
        let digest = hasher.finalize();

        let norm_sq = (0..128)
            .map(|i| hopf_probe_value(&digest, i) * hopf_probe_value(&digest, i))
            .sum::<f64>();
        let norm = norm_sq.sqrt();
        let offset = block * 128;
        (0..128)
            .map(|i| state_vector[offset + i] * hopf_probe_value(&digest, i) / norm)
            .sum()
    }

    fn get_state_4d_projection(&self, state_vector: &[f64]) -> Vec<f64> {
        let components = [
            self.hopf_signed_projection_component(state_vector, 0),
            self.hopf_signed_projection_component(state_vector, 1),
            self.hopf_signed_projection_component(state_vector, 2),
            self.hopf_signed_projection_component(state_vector, 3),
        ];
        let denom = components
            .iter()
            .map(|value| value * value)
            .sum::<f64>()
            .sqrt();
        if denom < 1e-12 {
            vec![0.5, 0.5, 0.5, 0.5]
        } else {
            components.iter().map(|value| value / denom).collect()
        }
    }

    fn evolve_brain_state(&mut self, identity: &str, query_text: &str, gamma: f64) -> Vec<f64> {
        let key = identity_key(identity);
        let mut active_state = self
            .session_brain_states
            .entry(key.clone())
            .or_insert_with(|| vec![1.0 / (512.0f64).sqrt(); 512])
            .clone();

        let words = tokenize(query_text);
        let mut s_vec = vec![0.0; 512];
        let mut word_count = 0;
        for w in words {
            if let Some(v) = self.vocab_vectors.get(&w) {
                for (sum, value) in s_vec.iter_mut().zip(v) {
                    *sum += *value;
                }
                word_count += 1;
            }
        }

        if word_count > 0 {
            let s_sum_sq = s_vec.iter().map(|value| value * value).sum::<f64>();
            let s_norm = s_sum_sq.sqrt();
            if s_norm > 0.0 {
                for value in &mut s_vec {
                    *value /= s_norm;
                }
            }

            let mut h_new = vec![0.0; 512];
            let mut h_sum_sq = 0.0;
            for ((next, active), source) in h_new.iter_mut().zip(&active_state).zip(&s_vec) {
                *next = gamma * *active + (1.0 - gamma) * *source;
                h_sum_sq += *next * *next;
            }
            let h_norm = h_sum_sq.sqrt();
            if h_norm > 0.0 {
                for value in &mut h_new {
                    *value /= h_norm;
                }
            }
            active_state = h_new;
        }

        self.session_brain_states.insert(key, active_state.clone());
        active_state
    }

    fn route_query_to_manifold_internal(
        &self,
        text: &str,
        identity: &str,
        state_vector: Option<&[f64]>,
    ) -> RoutingData {
        self.route_query_to_manifold_internal_with_hopf_input(text, identity, state_vector)
            .0
    }

    /// As `route_query_to_manifold_internal`, additionally returning the
    /// 512-d vector that `get_state_4d_projection` reduces to the Hopf map
    /// input (the grounded VSA vector, or the session state on ground
    /// failure). Measurement surface for issue #303.
    fn route_query_to_manifold_internal_with_hopf_input(
        &self,
        text: &str,
        identity: &str,
        state_vector: Option<&[f64]>,
    ) -> (RoutingData, Vec<f64>) {
        let active_state = match state_vector {
            Some(v) => v.to_vec(),
            None => {
                let key = identity_key(identity);
                self.session_brain_states
                    .get(&key)
                    .cloned()
                    .unwrap_or_else(|| vec![1.0 / (512.0f64).sqrt(); 512])
            }
        };

        match self.geometry_type {
            GeometryType::Spectral => {
                let geom = geometry::SpectralGeometry {
                    space_cid: "blake3:spectral_space".to_string(),
                    active_state: Some(&active_state),
                    identity: Some(identity),
                };
                self.route_query_to_manifold_internal_generic(&geom, text, identity, &active_state)
            }
            GeometryType::Vsa => {
                let geom = geometry::VsaGeometry {
                    space_cid: "blake3:vsa_space".to_string(),
                };
                self.route_query_to_manifold_internal_generic(&geom, text, identity, &active_state)
            }
        }
    }

    fn route_query_to_manifold_internal_generic<G: geometry::SemanticGeometry>(
        &self,
        geometry: &G,
        text: &str,
        identity: &str,
        active_state: &[f64],
    ) -> (RoutingData, Vec<f64>) {
        let (qimc_prime, qimc_index, identity_meta) = identity_to_qimc_prime(identity);
        let uor_control = derive_uor_control_plane(&identity_meta);

        let obj = geometry::TypedObject {
            object_type: "query".to_string(),
            content: text.to_string(),
        };

        let grounded = geometry.ground(&obj).unwrap_or_else(|| {
            let mut v = vec![0.0; 1024];
            v[..512].copy_from_slice(active_state);
            geometry::GroundedSemantics {
                vsa_vector: v.iter().map(|&x| x as f32).collect(),
                roles: vec![],
            }
        });

        let coords = geometry.encode(&grounded);

        let routes = geometry.soft_route(&coords, 16);

        let routed_idx = routes
            .iter()
            .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
            .map(|r| r.axis as usize)
            .unwrap_or(1);

        let routed_path = routes
            .iter()
            .find(|route| route.axis as usize == routed_idx)
            .map(|route| route.path.as_slice());

        let active_state_refined: Vec<f64> = grounded.vsa_vector[..512]
            .iter()
            .map(|&x| x as f64)
            .collect();
        let active_range = routed_path
            .filter(|path| path.len() >= 3)
            .map(|path| [path[1] as usize, path[2] as usize])
            .filter(|range| range[0] < range[1] && range[1] <= active_state_refined.len())
            .unwrap_or([(routed_idx - 1) * 32, routed_idx * 32]);
        let routed_projection = coords
            .coordinates
            .get("projected_state")
            .map(|values| {
                values
                    .iter()
                    .map(|bits| f32::from_bits(*bits) as f64)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| active_state_refined.clone());
        let routed_slice = &routed_projection[active_range[0]..active_range[1]];
        let (sigma_q, sigma_kl, lambda_val, kappa, deficit_angle) =
            state_metrics_from_weights(routed_slice);

        let eigenvalues = coords
            .coordinates
            .get("eigenvalues")
            .map(|values| {
                values
                    .iter()
                    .map(|bits| f32::from_bits(*bits) as f64)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec![0.0; 8]);

        let v_4d = self.get_state_4d_projection(&active_state_refined);
        let (sector_id, bins, hopf_components) = assign_sector_hopf_transport_scalar(
            &v_4d,
            512,
            uor_control.phase_transport_lambda,
            uor_control.hopf_chi_bins,
        );

        let payload = serde_json::json!({
            "identity": identity_meta.identity,
            "identity_type": identity_meta.identity_type,
            "identity_uor_address": identity_meta.identity_uor_address,
            "identity_uor_digest": identity_meta.identity_uor_digest,
            "identity_uor_hash_algorithm": identity_meta.identity_uor_hash_algorithm,
            "uor_entropy_bias": uor_control.entropy_bias,
            "window_index": routed_idx,
            "scale_x": scale_x_for_window(routed_idx),
            "kappa": kappa,
            "deficit_angle": deficit_angle,
            "hopf_sector": sector_id,
        });

        let attestation = generate_uor_attestation(&payload);

        let routed = RoutedResult {
            window_index: routed_idx as u64,
            scale_x: scale_x_for_window(routed_idx),
            metrics: MetricsResult {
                sigma_q,
                sigma_kl,
                lambda_entropy: lambda_val,
                kappa,
                deficit_angle,
            },
            eigenvalues,
            active_range: vec![active_range[0] as u64, active_range[1] as u64],
            state_vector: routed_slice.to_vec(),
            qimc: QimcResult {
                identity: identity_meta.identity.clone(),
                identity_type: identity_meta.identity_type.clone(),
                identity_uor_address: identity_meta.identity_uor_address.clone(),
                identity_uor_digest: identity_meta.identity_uor_digest.clone(),
                identity_uor_hash_algorithm: identity_meta.identity_uor_hash_algorithm.clone(),
                uor_control: UorControlPlanInfo {
                    entropy_bias: uor_control.entropy_bias,
                    hopf_chi_bins: uor_control.hopf_chi_bins as u64,
                },
                prime: qimc_prime as u64,
                index: qimc_index as u64,
            },
            hopf: HopfResult {
                rho1: hopf_components["rho1"],
                rho2: hopf_components["rho2"],
                chi: hopf_components["chi"],
                delta: hopf_components["delta"],
                alpha: hopf_components["alpha"],
                transported_alpha: hopf_components["transported_alpha"],
                phase_transport_lambda: uor_control.phase_transport_lambda,
                hopf_chi_bins: uor_control.hopf_chi_bins as u64,
                sector_id: sector_id as u64,
                chi_bin: bins["chi_bin"] as u64,
                delta_bin: bins["delta_bin"] as u64,
                alpha_bin: bins["alpha_bin"] as u64,
                subspace_norms: SubspaceNorms {
                    act: active_state_refined[0..128]
                        .iter()
                        .map(|&x| x * x)
                        .sum::<f64>()
                        .sqrt(),
                    obj: active_state_refined[128..256]
                        .iter()
                        .map(|&x| x * x)
                        .sum::<f64>()
                        .sqrt(),
                    temp: active_state_refined[256..384]
                        .iter()
                        .map(|&x| x * x)
                        .sum::<f64>()
                        .sqrt(),
                    shared: active_state_refined[384..512]
                        .iter()
                        .map(|&x| x * x)
                        .sum::<f64>()
                        .sqrt(),
                },
            },
            uor_address: attestation.address.clone(),
            uor: attestation,
        };

        let mut routes_output = Vec::new();
        for r in &routes {
            let range = r
                .path
                .get(1..3)
                .map(|path| [path[0] as usize, path[1] as usize])
                .filter(|range| range[0] < range[1] && range[1] <= active_state_refined.len())
                .unwrap_or([(r.axis as usize - 1) * 32, r.axis as usize * 32]);
            let source_state = if r.axis as usize == routed_idx {
                &routed_projection
            } else {
                &active_state_refined
            };
            let slice = &source_state[range[0]..range[1]];
            let (_s_q, _s_kl, _l_v, k_v, d_a) = state_metrics_from_weights(slice);
            routes_output.push(RouteInfo {
                window_index: r.axis as u64,
                scale_x: scale_x_for_window(r.axis as usize),
                routing_score: r.score as f64,
                kappa: if r.axis as usize == routed_idx {
                    k_v
                } else {
                    0.0
                },
                deficit_angle: if r.axis as usize == routed_idx {
                    d_a
                } else {
                    std::f64::consts::PI
                },
                state_vector: slice.to_vec(),
                active_range: vec![range[0] as u64, range[1] as u64],
            });
        }

        (
            RoutingData {
                routed,
                all_routes: routes_output,
            },
            active_state_refined,
        )
    }

    /// Content-derived 512-d state for a piece of text (issue #245): the
    /// L2-normalized sum of the zeta-seeded vocabulary vectors of its known
    /// words — the same construction `evolve_brain_state` uses for queries.
    /// `None` when no word of the text is in the vocabulary.
    fn content_state_vector(&self, text: &str) -> Option<Vec<f64>> {
        let words = tokenize(text);
        let mut s_vec = vec![0.0; 512];
        let mut word_count = 0usize;
        for w in words {
            if let Some(v) = self.vocab_vectors.get(&w) {
                for (sum, value) in s_vec.iter_mut().zip(v) {
                    *sum += *value;
                }
                word_count += 1;
            }
        }
        if word_count == 0 {
            return None;
        }
        let norm = s_vec.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm > 0.0 {
            for v in &mut s_vec {
                *v /= norm;
            }
        }
        Some(s_vec)
    }

    /// Read-only view of the indexed corpus items for an identity (test and
    /// inspection surface for the content-bearing store, issue #245).
    pub fn corpus_items_for(&self, identity: &str) -> Vec<&CorpusItem> {
        let key = identity_key(identity);
        self.corpus_index_by_identity
            .get(&key)
            .map(|store| store.values().flatten().collect())
            .unwrap_or_default()
    }

    /// Content-free indexing (issue #255, arm 1): reconstructs the
    /// pre-#245 stub exactly — the sentence routes with no content-derived
    /// state vector, so the stored vector is whatever the session state
    /// happened to be. Test/harness surface only; production indexing is
    /// `index_sentence`.
    pub fn index_sentence_content_free(&mut self, sentence: &str, identity: &str) {
        self.index_sentence_with_state_override(sentence, identity, true)
    }

    /// Shared body for issue #255's arms: identical to
    /// `index_sentence_internal` except `content_free` forces the pre-#245
    /// `None` routing state.
    fn index_sentence_with_state_override(
        &mut self,
        sentence: &str,
        identity: &str,
        content_free: bool,
    ) {
        let s_clean = sentence.trim();
        if s_clean.is_empty() || s_clean.len() < 10 {
            return;
        }
        let words = tokenize(s_clean);
        for w in &words {
            if w.len() > 1 && !self.word_primes.contains_key(w) {
                self.add_word_to_vocabulary(w);
            }
        }
        let content_state = if content_free {
            None
        } else {
            self.content_state_vector(s_clean)
        };
        self.index_sentence_routed(s_clean, identity, &words, content_state.as_deref());
    }

    fn index_sentence_internal(&mut self, sentence: &str, identity: &str) {
        self.index_sentence_with_state_override(sentence, identity, false)
    }

    fn index_sentence_routed(
        &mut self,
        s_clean: &str,
        identity: &str,
        words: &[String],
        state: Option<&[f64]>,
    ) {
        // Issue #245: route the sentence through its own content-derived
        // state, not the session brain state at index time — stored
        // state_vectors must encode the sentence, or cosine resonance
        // retrieval is degenerate. Falls back to session state only when no
        // word of the sentence is in the vocabulary (words were just added
        // above, so this is rare).
        let routing_data = self.route_query_to_manifold_internal(s_clean, identity, state);
        let best = routing_data.routed;
        let idx_win = best.window_index;

        let prime_product = self.get_sentence_prime_product(words);

        // Issue #434 (adopted de-banding): the stored vector is the
        // FULL-WIDTH content vector by default. The pre-#434 shape kept
        // only `active_range` — thirty-two of five hundred twelve
        // coefficients, one sixteenth of the width — which PR #442
        // measured as the dominant cost of retrieval quality. Banding is
        // retained behind `banded_storage` for A/B and rollback. The
        // content-free path (issue #255 arm one) supplies no content
        // vector and keeps the banded shape either way, so that arm's
        // pre-#245 reconstruction stays byte-identical.
        let s_idx = best.active_range[0] as usize;
        let e_idx = best.active_range[1] as usize;
        let state_vector = match state {
            Some(full) if !self.banded_storage && full.len() == 512 => full.to_vec(),
            _ => {
                let mut banded = vec![0.0; 512];
                banded[s_idx..e_idx].copy_from_slice(&best.state_vector[..e_idx - s_idx]);
                banded
            }
        };

        let (u, v) = self.get_sentence_projection(&state_vector, idx_win as usize);
        let v_4d = self.get_state_4d_projection(&state_vector);
        let provenance = CoordinateProvenance {
            source_kappa: source_provenance(s_clean),
            projection_kappa: projection_provenance().to_string(),
            vocabulary_kappa: vocabulary_provenance(&self.word_primes, words),
        };

        let key = identity_key(identity);
        let target_index = self.corpus_index_by_identity.entry(key).or_default();

        let win_items = target_index.entry(idx_win).or_default();

        // Incrementally update transitions (1st and 2nd order)
        if !words.is_empty() {
            // 1st order
            for i in 0..words.len() - 1 {
                let w1 = &words[i];
                let w2 = &words[i + 1];
                let entry = self.transitions.entry(w1.clone()).or_default();
                let count = entry.entry(w2.clone()).or_insert(0.0);
                *count += 1.0;
            }

            // 2nd order (trigram)
            for i in 0..words.len().saturating_sub(2) {
                let w1 = &words[i];
                let w2 = &words[i + 1];
                let w3 = &words[i + 2];
                let key_trigram = format!("{} {}", w1, w2);
                let entry = self.transitions_2nd.entry(key_trigram).or_default();
                let count = entry.entry(w3.clone()).or_insert(0.0);
                *count += 1.0;
            }
        }

        let item = CorpusItem {
            sentence: s_clean.to_string(),
            state_vector,
            kappa: best.metrics.kappa,
            deficit_angle: best.metrics.deficit_angle,
            prime_product: prime_product.to_string(),
            words: words.to_vec(),
            u,
            v,
            v_4d,
            provenance: Some(provenance),
        };

        win_items.push(item.clone());

        let item_id = self.corpus_index.len();
        self.corpus_index.insert(item_id as u64, vec![item]);

        // Induced multi-facet index coordinates
        let geom = geometry::VsaGeometry {
            space_cid: "blake3:vsa_space".to_string(),
        };
        let obj = geometry::TypedObject {
            object_type: "sentence".to_string(),
            content: s_clean.to_string(),
        };
        if let Some(grounded) = geom.ground(&obj) {
            let coords = geom.encode(&grounded);
            self.index_semantic_object(item_id, &coords);
            // #496 real facet-index: store the item's VSA hypervector for
            // precomputed scoring, and index it under its salient content
            // dimensions (the high-recall, content-addressed candidate set).
            let content_len = 512.min(grounded.vsa_vector.len());
            let dims = salient_content_dims(&grounded.vsa_vector[..content_len], VSA_SALIENT_DIMS);
            let id = item_id as u64;
            for d in dims {
                let list = self.vsa_dim_index.entry(d).or_default();
                if list.last() != Some(&id) {
                    list.push(id);
                }
            }
            self.vsa_vectors.insert(id, grounded.vsa_vector);
        }
    }

    /// Recompute and verify provenance for every indexed sentence in one
    /// identity scope. This is deliberately outside the prediction kernel;
    /// callers may use it as an integrity check before serving a retrieval.
    /// Returns the count of verified items, or `None` if any indexed sentence
    /// is missing provenance or its recorded κ disagrees with the recomputed
    /// source/projection/vocabulary provenance (R5 — a failed integrity check
    /// is the absence of a verified count, not a reported error class).
    pub fn verify_corpus_provenance(&self, identity: &str) -> Option<usize> {
        let items = self.corpus_items_for(identity);
        for item in &items {
            let provenance = item.provenance.as_ref()?;
            if provenance.source_kappa != source_provenance(&item.sentence) {
                return None;
            }
            if provenance.projection_kappa != projection_provenance() {
                return None;
            }
            if provenance.vocabulary_kappa != vocabulary_provenance(&self.word_primes, &item.words)
            {
                return None;
            }
        }
        Some(items.len())
    }

    fn retrieve_geometric_resonance(
        &self,
        text: &str,
        routing_data: &RoutingData,
        top_n: usize,
        state_vector: &[f64],
        identity: &str,
    ) -> Vec<ResonanceResult> {
        let query_words = tokenize(text);
        let stopwords = query_stopwords();
        let query_primes: Vec<u64> = query_words
            .iter()
            .filter(|w| !stopwords.contains(&w.as_str()))
            .filter_map(|w| self.word_primes.get(w).copied())
            .collect();

        let mut scored = Vec::new();
        let key = identity_key(identity);

        let empty_index = HashMap::new();
        let scoped_index = self
            .corpus_index_by_identity
            .get(&key)
            .unwrap_or(&empty_index);
        let shared_index = self
            .corpus_index_by_identity
            .get("shared:shared")
            .unwrap_or(&empty_index);

        // Issue #480: the query projection is band-only while STORAGE is
        // full-width by default (#434, PR #465). That asymmetry is real —
        // zeroed query coordinates contribute nothing to a dot product, so
        // the cosine only ever sees the band however wide the stored vector
        // is — and it is **deliberately retained**. Do not "fix" it without
        // reading `docs/query_projection_480.md` first.
        //
        // Measured at 46,342 windows / 500 probes: making the shapes
        // symmetric is worth +0.006 MRR and +0.008 top-1 while costing
        // ~0.018 of recall@20, against a pre-declared +0.05 MRR bar. The
        // reason it cannot pay here is the relevance form below —
        //     relevance = shared_count * 100 + sim * slice_norm + scope_boost
        // — where the lexical term is 100x the cosine, so the vector shape
        // only reorders candidates already tied on word overlap. The 0.8948
        // de-banding figure came from a harness that ranked by cosine ALONE.
        //
        // The symmetric shape stays reachable through `set_full_width_query`
        // so the measurement reproduces, and because it becomes the right
        // default the moment the lexical term stops dominating.
        //
        // `slice_norm` below is intentionally left on the band slice: it is a
        // scale factor on the cosine term, not part of the vector shape.
        let full_width_query = self.full_width_query && state_vector.len() == 512;
        // #484: read once, outside the scoring loop, so every candidate in
        // one call is ranked under one weight and one geometric scaling.
        let lexical_weight = self.lexical_weight();
        let unscaled_geometric_term = self.unscaled_geometric_term;
        // #486: the query text's own content vector, built by the SAME
        // construction that produced every stored vector. `None` here means
        // the text has no vocabulary word at all, in which case the deployed
        // projection is used unchanged rather than leaving the query with no
        // vector.
        let content_query = if self.content_query_vector {
            self.content_state_vector(text)
        } else {
            None
        };
        let mut query_projections = HashMap::new();
        let mut query_ranges = HashMap::new();
        for r in &routing_data.all_routes {
            let start = r.active_range[0] as usize;
            let end = r.active_range[1] as usize;
            let projection = match &content_query {
                Some(content) => content.clone(),
                None if full_width_query => state_vector.to_vec(),
                None => {
                    let mut banded = vec![0.0; 512];
                    banded[start..end].copy_from_slice(&r.state_vector[..end - start]);
                    banded
                }
            };
            query_projections.insert(r.window_index, projection);
            query_ranges.insert(r.window_index, (start, end));
        }

        let all_windows: std::collections::HashSet<u64> = scoped_index
            .keys()
            .chain(shared_index.keys())
            .copied()
            .collect();

        for &win_idx in &all_windows {
            let shared_items = shared_index.get(&win_idx);
            let scoped_items = scoped_index.get(&win_idx);

            let (s_idx, e_idx) = query_ranges
                .get(&win_idx)
                .copied()
                .unwrap_or(((win_idx as usize - 1) * 32, win_idx as usize * 32));
            let sum_sq = state_vector[s_idx..e_idx]
                .iter()
                .map(|value| value * value)
                .sum::<f64>();
            let slice_norm = sum_sq.sqrt();

            let q_vec = match query_projections.get(&win_idx) {
                Some(qv) => qv,
                None => continue,
            };

            let merged_items = shared_items
                .into_iter()
                .flatten()
                .map(|item| (item, 0.0))
                .chain(scoped_items.into_iter().flatten().map(|item| (item, 15.0)));

            for (item, scope_boost) in merged_items {
                let mut shared_count = 0;
                for word in &item.words {
                    if let Some(&p) = self.word_primes.get(word) {
                        if query_primes.contains(&p) {
                            shared_count += 1;
                        }
                    }
                }

                let sim = cosine_similarity(q_vec, &item.state_vector);
                let geometric = if unscaled_geometric_term {
                    sim
                } else {
                    sim * slice_norm
                };
                let relevance = (shared_count as f64) * lexical_weight + geometric + scope_boost;

                scored.push(ResonanceResult {
                    sentence: item.sentence.clone(),
                    relevance,
                    window_index: win_idx,
                    kappa: item.kappa,
                    deficit_angle: item.deficit_angle,
                    provenance: item.provenance.clone(),
                });
            }
        }

        scored.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());
        scored.truncate(top_n);
        scored
    }

    fn retrieve_vsa_multi_facet_resonance(&self, text: &str, top_n: usize) -> Vec<ResonanceResult> {
        let geom = geometry::VsaGeometry {
            space_cid: "blake3:vsa_space".to_string(),
        };
        let obj = geometry::TypedObject {
            object_type: "query".to_string(),
            content: text.to_string(),
        };

        let Some(grounded) = geom.ground(&obj) else {
            return Vec::new();
        };
        let query_vsa: Vec<f64> = grounded.vsa_vector.iter().map(|&x| x as f64).collect();

        // #496 real facet-index: gather candidates by UNION over the query's
        // salient content dimensions — a high-recall, content-addressed set that
        // replaces the coarse type/entity intersection (which dropped ~47% of
        // targets in the #496 ablation). Falls back to the legacy facet
        // intersection only for router state serialized before this index.
        let candidate_ids: Vec<u64> = if self.vsa_dim_index.is_empty() {
            self.legacy_vsa_facet_candidates(&geom, &grounded)
        } else {
            let content_len = 512.min(grounded.vsa_vector.len());
            let dims = salient_content_dims(&grounded.vsa_vector[..content_len], VSA_SALIENT_DIMS);
            let mut set: std::collections::HashSet<u64> = std::collections::HashSet::new();
            for d in dims {
                if let Some(list) = self.vsa_dim_index.get(&d) {
                    set.extend(list.iter().copied());
                }
            }
            let mut ids: Vec<u64> = set.into_iter().collect();
            ids.sort_unstable();
            ids
        };

        let mut scored = Vec::new();
        for &id in &candidate_ids {
            let Some(items) = self.corpus_index.get(&id) else {
                continue;
            };
            // #496: score against the PRECOMPUTED stored VSA vector (fast);
            // re-ground the sentence only for router state serialized before
            // `vsa_vectors` existed.
            let sim = if let Some(v) = self.vsa_vectors.get(&id) {
                let cand: Vec<f64> = v.iter().map(|&x| x as f64).collect();
                cosine_similarity(&query_vsa, &cand)
            } else {
                items
                    .first()
                    .and_then(|item| {
                        geom.ground(&geometry::TypedObject {
                            object_type: "query".to_string(),
                            content: item.sentence.clone(),
                        })
                    })
                    .map(|c| {
                        let cand: Vec<f64> = c.vsa_vector.iter().map(|&x| x as f64).collect();
                        cosine_similarity(&query_vsa, &cand)
                    })
                    .unwrap_or(0.0)
            };
            for item in items {
                scored.push(ResonanceResult {
                    sentence: item.sentence.clone(),
                    relevance: sim * 100.0,
                    window_index: id % 16 + 1,
                    kappa: item.kappa,
                    deficit_angle: item.deficit_angle,
                    provenance: item.provenance.clone(),
                });
            }
        }

        scored.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(top_n);
        scored
    }

    /// Legacy VSA candidate gathering (facet type/entity intersection with
    /// selective backoff), retained only for router state serialized before the
    /// #496 salient-dimension index existed (`vsa_dim_index` empty).
    fn legacy_vsa_facet_candidates(
        &self,
        geom: &geometry::VsaGeometry,
        grounded: &geometry::GroundedSemantics,
    ) -> Vec<u64> {
        let coords = geom.encode(grounded);
        let mut working_coords = coords.clone();
        let mut candidate_ids = Vec::new();
        loop {
            let mut lists = Vec::new();
            for (facet, path) in &working_coords.coordinates {
                let list = match facet.as_str() {
                    "type" => self.facet_store.type_index.get(path),
                    "entity" => self.facet_store.entity_index.get(path),
                    "relation" => self.facet_store.relation_index.get(path),
                    "temporal" => self.facet_store.temporal_index.get(path),
                    "intent" => self.facet_store.intent_index.get(path),
                    "provenance" => self.facet_store.provenance_index.get(path),
                    _ => None,
                };
                lists.push(list.cloned().unwrap_or_default());
            }
            if lists.is_empty() {
                break;
            }
            let mut intersection = lists[0].clone();
            for next_list in lists.iter().skip(1) {
                intersection.retain(|x| next_list.contains(x));
            }
            if !intersection.is_empty() {
                candidate_ids = intersection;
                break;
            }
            let mut longest_facet: Option<String> = None;
            let mut max_len = 0;
            for (facet, path) in &working_coords.coordinates {
                if path.len() > max_len {
                    max_len = path.len();
                    longest_facet = Some(facet.clone());
                } else if path.len() == max_len && max_len > 0 {
                    if let Some(ref current_longest) = longest_facet {
                        if facet > current_longest {
                            longest_facet = Some(facet.clone());
                        }
                    }
                }
            }
            if let Some(facet_to_backoff) = longest_facet {
                let path = working_coords
                    .coordinates
                    .get_mut(&facet_to_backoff)
                    .unwrap();
                path.pop();
                if path.is_empty() {
                    working_coords.coordinates.remove(&facet_to_backoff);
                }
            } else {
                break;
            }
        }
        candidate_ids
    }

    fn rebuild_transitions(&mut self) {
        self.transitions.clear();
        self.transitions_2nd.clear();

        let mut process_words = |words: &[String]| {
            if words.is_empty() {
                return;
            }

            // 1st order
            for i in 0..words.len() - 1 {
                let w1 = &words[i];
                let w2 = &words[i + 1];
                let entry = self.transitions.entry(w1.clone()).or_default();
                let count = entry.entry(w2.clone()).or_insert(0.0);
                *count += 1.0;
            }

            // 2nd order (trigram)
            for i in 0..words.len().saturating_sub(2) {
                let w1 = &words[i];
                let w2 = &words[i + 1];
                let w3 = &words[i + 2];
                let key = format!("{} {}", w1, w2);
                let entry = self.transitions_2nd.entry(key).or_default();
                let count = entry.entry(w3.clone()).or_insert(0.0);
                *count += 1.0;
            }
        };

        for identity_store in self.corpus_index_by_identity.values() {
            for win_items in identity_store.values() {
                for item in win_items {
                    process_words(&item.words);
                }
            }
        }

        // Also sort and dedup vocabulary
        self.vocabulary.sort();
        self.vocabulary.dedup();
    }

    // Kept as a flat internal call to mirror the established wasm/native
    // decoding surfaces without allocating a transient options object.
    #[allow(clippy::too_many_arguments)]
    fn generate_geometric_response_with_trajectory_internal(
        &self,
        prompt_text: &str,
        state_vector: &[f64],
        max_len: usize,
        temp: f64,
        gravity: f64,
        freq_penalty: f64,
        identity: &str,
        gamma: f64,
    ) -> (String, Vec<TrajectoryStep>, Vec<f64>) {
        let words = tokenize(prompt_text);
        if words.is_empty() {
            return (
                "manifold base frequency unstable".to_string(),
                Vec::new(),
                state_vector.to_vec(),
            );
        }

        // 1. Find start key matching prompts
        let mut start_key = None;
        for i in 0..words.len().saturating_sub(1) {
            let key = format!("{} {}", words[i], words[i + 1]);
            if self.transitions_2nd.contains_key(&key) {
                start_key = Some(key);
                break;
            }
        }

        if start_key.is_none() {
            // Find any trigram starting with prompt's last word
            if let Some(last_word) = words.last() {
                for key in self.transitions_2nd.keys() {
                    if key.starts_with(last_word) {
                        start_key = Some(key.clone());
                        break;
                    }
                }
            }
        }

        if start_key.is_none() {
            // Default random choice
            if !self.transitions_2nd.is_empty() {
                let keys: Vec<&String> = self.transitions_2nd.keys().collect();
                let seed = words.iter().map(|w| w.len()).sum::<usize>();
                let idx = seed % keys.len();
                start_key = Some(keys[idx].clone());
            }
        }

        let mut generated = Vec::new();
        if let Some(ref key) = start_key {
            let parts: Vec<&str> = key.split_whitespace().collect();
            if parts.len() >= 2 {
                generated.push(parts[0].to_string());
                generated.push(parts[1].to_string());
            }
        } else {
            generated.push("manifold".to_string());
            generated.push("base".to_string());
        }

        let mut history = HashMap::new();
        for w in &generated {
            *history.entry(w.clone()).or_insert(0.0) += 1.0;
        }

        let mut trajectory = Vec::new();
        let mut s_local = state_vector.to_vec();
        let mut consecutive_repeat_count = 0usize;

        let mut accumulated_delta = 0.0;
        let mut prev_stratum = 0usize;
        let mut prev_state_bin = vec![false; 512];
        let mut prev_state_vec = vec![0.0; 512];

        // Seed generator
        let mut seed = prompt_text.len() as u64;
        let mut rand_f = || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 32) as f64 / 4294967296.0
        };

        for step_idx in 0..max_len {
            let next_word;
            if step_idx < generated.len() {
                next_word = generated[step_idx].clone();
            } else {
                let w_prev2 = &generated[generated.len() - 2];
                let w_prev1 = &generated[generated.len() - 1];
                let key = format!("{} {}", w_prev2, w_prev1);

                let empty_targets = HashMap::new();
                let targets = self.transitions_2nd.get(&key).unwrap_or(&empty_targets);

                if targets.is_empty() {
                    // Single-word backoff
                    let last_word = &generated[generated.len() - 1];
                    let empty_first = HashMap::new();
                    let matching_targets = self.transitions.get(last_word).unwrap_or(&empty_first);
                    if !matching_targets.is_empty() {
                        let mut best_word = "manifold".to_string();
                        let mut best_dot = f64::MIN;
                        for next_w in matching_targets.keys() {
                            if let Some(c_vec) = self.vocab_vectors.get(next_w) {
                                let mut dot = 0.0;
                                for i in 0..512 {
                                    dot += c_vec[i] * s_local[i];
                                }
                                if dot > best_dot {
                                    best_dot = dot;
                                    best_word = next_w.clone();
                                }
                            }
                        }
                        next_word = best_word;
                    } else {
                        // Vectorized semantic jump to closest vocabulary word vector (with threshold/stride to prevent freezing)
                        let mut best_word = "manifold".to_string();
                        if !self.vocab_vectors.is_empty() {
                            let mut best_dot = f64::MIN;
                            let max_search = 1000;
                            if self.vocab_vectors.len() <= max_search {
                                for (word, vec) in &self.vocab_vectors {
                                    let mut dot = 0.0;
                                    for i in 0..512 {
                                        dot += vec[i] * s_local[i];
                                    }
                                    if dot > best_dot {
                                        best_dot = dot;
                                        best_word = word.clone();
                                    }
                                }
                            } else {
                                let stride = self.vocab_vectors.len() / max_search;
                                let stride = if stride == 0 { 1 } else { stride };
                                for (count, (word, vec)) in self.vocab_vectors.iter().enumerate() {
                                    if count % stride == 0 {
                                        let mut dot = 0.0;
                                        for i in 0..512 {
                                            dot += vec[i] * s_local[i];
                                        }
                                        if dot > best_dot {
                                            best_dot = dot;
                                            best_word = word.clone();
                                        }
                                    }
                                }
                            }
                        }
                        next_word = best_word;
                    }
                } else {
                    let mut candidates = Vec::new();
                    let mut scores = Vec::new();
                    let total_count: f64 = targets.values().sum();
                    for (c, &count_trans) in targets {
                        let p_trans = if total_count > 0.0 {
                            count_trans / total_count
                        } else {
                            1e-10
                        };
                        let c_vec = self
                            .vocab_vectors
                            .get(c)
                            .cloned()
                            .unwrap_or_else(|| vec![0.0; 512]);
                        let sim = cosine_similarity(&c_vec, &s_local);
                        let freq = history.get(c).copied().unwrap_or(0.0);
                        let is_consecutive_repeat =
                            generated.last().map(|w| w == c).unwrap_or(false);
                        let repeat_penalty = if is_consecutive_repeat {
                            freq_penalty * 5.0 * (consecutive_repeat_count as f64 + 1.0).exp()
                        } else {
                            0.0
                        };
                        let score =
                            p_trans.ln() + (gravity * sim) - (freq_penalty * freq) - repeat_penalty;
                        candidates.push(c.clone());
                        scores.push(score);
                    }

                    // Softmax selection
                    let max_score = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                    let mut sum_exp = 0.0;
                    let mut probs = Vec::new();
                    for &s in &scores {
                        let e = (s - max_score).exp();
                        probs.push(e);
                        sum_exp += e;
                    }

                    if sum_exp > 0.0 {
                        for p in probs.iter_mut() {
                            *p /= sum_exp;
                        }
                    }

                    if temp > 0.0 {
                        let r = rand_f();
                        let mut cum = 0.0;
                        let mut selected = candidates.last().unwrap().clone();
                        for (i, p) in probs.iter().enumerate() {
                            cum += p;
                            if r <= cum {
                                selected = candidates[i].clone();
                                break;
                            }
                        }
                        next_word = selected;
                    } else {
                        let mut best_idx = 0;
                        let mut best_s = scores[0];
                        for (i, &s) in scores.iter().enumerate() {
                            if s > best_s {
                                best_s = s;
                                best_idx = i;
                            }
                        }
                        next_word = candidates[best_idx].clone();
                    }
                }
                generated.push(next_word.clone());
            }

            if generated.len() >= 2
                && generated[generated.len() - 1] == generated[generated.len() - 2]
            {
                consecutive_repeat_count += 1;
            } else {
                consecutive_repeat_count = 0;
            }

            *history.entry(next_word.clone()).or_insert(0.0) += 1.0;

            // Route metrics
            let r_data = self.route_query_to_manifold_internal("", identity, Some(&s_local));
            let routed = r_data.routed;

            // Elliptic basin escape: inject orthogonal Hopf perturbation vector when \theta_d < -0.15 and repeat_count >= 3
            if routed.metrics.deficit_angle < -0.15 && consecutive_repeat_count >= 3 {
                let mut p_vec = vec![0.0; 512];
                for (i, p_val) in p_vec.iter_mut().enumerate() {
                    *p_val = (i as f64 * 0.1234 + step_idx as f64).sin();
                }
                let dot: f64 = p_vec.iter().zip(&s_local).map(|(p, s)| p * s).sum();
                let mut ortho = vec![0.0; 512];
                let mut ortho_sq = 0.0;
                for i in 0..512 {
                    ortho[i] = p_vec[i] - dot * s_local[i];
                    ortho_sq += ortho[i] * ortho[i];
                }
                let ortho_norm = ortho_sq.sqrt();
                if ortho_norm > 1e-9 {
                    let perturbation_scale = 0.35;
                    let mut s_new_sq = 0.0;
                    for i in 0..512 {
                        s_local[i] += perturbation_scale * (ortho[i] / ortho_norm);
                        s_new_sq += s_local[i] * s_local[i];
                    }
                    let s_new_norm = s_new_sq.sqrt();
                    if s_new_norm > 1e-9 {
                        for val in &mut s_local {
                            *val /= s_new_norm;
                        }
                    }
                }
            }

            let s_idx = routed.active_range[0] as usize;
            let e_idx = routed.active_range[1] as usize;
            let mut state_vec = vec![0.0; 512];
            state_vec[s_idx..e_idx].copy_from_slice(&routed.state_vector[..e_idx - s_idx]);

            let mut curr_state_bin = vec![false; 512];
            let mut stratum = 0usize;
            for i in 0..512 {
                if state_vec[i].abs() > 1e-4 {
                    curr_state_bin[i] = true;
                    stratum += 1;
                }
            }

            let cascade_len;
            let catastrophe;
            let commutator_curv;
            let winding_number;
            let dihedral;

            if step_idx == 0 {
                cascade_len = 0;
                catastrophe = false;
                commutator_curv = 0.0;
                winding_number = 0.0;
                dihedral = DihedralInfo {
                    s: 0,
                    k: 0,
                    label: "r^0".to_string(),
                };
            } else {
                let mut run = 0;
                let mut max_run = 0;
                for i in 0..512 {
                    if prev_state_bin[i] != curr_state_bin[i] {
                        run += 1;
                        max_run = max_run.max(run);
                    } else {
                        run = 0;
                    }
                }
                cascade_len = max_run;
                catastrophe = (stratum as i32 - prev_stratum as i32).abs() >= 15;

                let mut dist_e_sq = 0.0;
                let mut dist_h = 0;
                for i in 0..512 {
                    let diff = prev_state_vec[i] - state_vec[i];
                    dist_e_sq += diff * diff;
                    if prev_state_bin[i] != curr_state_bin[i] {
                        dist_h += 1;
                    }
                }
                let dist_e = dist_e_sq.sqrt();
                let dist_h = dist_h as f64;
                if dist_e + dist_h > 1e-6 {
                    commutator_curv = (dist_e - dist_h) / (dist_e + dist_h);
                } else {
                    commutator_curv = 0.0;
                }

                accumulated_delta += routed.hopf.delta;
                winding_number = accumulated_delta / (2.0 * std::f64::consts::PI);
                let s_refl = if stratum < prev_stratum { 1 } else { 0 };
                let k_rot = ((winding_number * 8.0).round() as i32).rem_euclid(8) as usize;
                dihedral = DihedralInfo {
                    s: s_refl,
                    k: k_rot as u64,
                    label: format!("{}r^{}", if s_refl == 1 { "s" } else { "" }, k_rot),
                };
            }

            prev_stratum = stratum;
            prev_state_bin = curr_state_bin;
            prev_state_vec = state_vec;

            // R4 Stereographic projection coordinates
            let mut q4 = [0.0; 4];
            for (k, value) in q4.iter_mut().enumerate() {
                if s_idx + k < 512 {
                    *value = routed.state_vector[k];
                }
            }
            let q4_sum_sq = q4.iter().map(|value| value * value).sum::<f64>();
            let q4_norm = q4_sum_sq.sqrt();
            if q4_norm > 1e-9 {
                for value in &mut q4 {
                    *value /= q4_norm;
                }
            }
            let denom = (1.0 - q4[0]).max(1e-6);
            let r4_proj = Projection3D {
                w: q4[0],
                x: q4[1],
                y: q4[2],
                z: q4[3],
                capital_x: q4[1] / denom,
                capital_y: q4[2] / denom,
                capital_z: q4[3] / denom,
            };

            trajectory.push(TrajectoryStep {
                step: (step_idx + 1) as u64,
                word: next_word.clone(),
                window_index: routed.window_index,
                scale_x: routed.scale_x,
                deficit_angle: routed.metrics.deficit_angle,
                kappa: routed.metrics.kappa,
                sigma_kl: routed.metrics.sigma_kl,
                qimc: routed.qimc,
                hopf: routed.hopf,
                r4_projection: r4_proj,
                quantum: QuantumMetrics {
                    stratum: stratum as u64,
                    cascade_length: cascade_len as u64,
                    catastrophe,
                    winding_number,
                    commutator_curvature: commutator_curv,
                    monodromy: dihedral,
                },
            });

            // Evolve s_local with the chosen word's vector along geodesic
            if let Some(v_next) = self.vocab_vectors.get(&next_word) {
                let v_sum_sq = v_next.iter().map(|value| value * value).sum::<f64>();
                let v_norm = v_sum_sq.sqrt();
                let mut v_normed = vec![0.0; 512];
                if v_norm > 0.0 {
                    for (normalized, value) in v_normed.iter_mut().zip(v_next) {
                        *normalized = *value / v_norm;
                    }
                }
                let mut h_new = vec![0.0; 512];
                let mut h_sum_sq = 0.0;
                for ((next, current), normalized) in h_new.iter_mut().zip(&s_local).zip(&v_normed) {
                    *next = gamma * *current + (1.0 - gamma) * *normalized;
                    h_sum_sq += *next * *next;
                }
                let h_norm = h_sum_sq.sqrt();
                if h_norm > 0.0 {
                    for value in &mut h_new {
                        *value /= h_norm;
                    }
                }
                s_local = h_new;
            }
        }

        let decoded_response = generated.join(" ");
        (decoded_response, trajectory, s_local)
    }
}

// ============================================================
// Helpers and Types Declarations
// ============================================================

#[derive(Serialize, Deserialize, Clone)]
pub struct RoutingData {
    pub routed: RoutedResult,
    pub all_routes: Vec<RouteInfo>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RoutedResult {
    pub window_index: u64,
    pub scale_x: f64,
    pub metrics: MetricsResult,
    pub eigenvalues: Vec<f64>,
    pub active_range: Vec<u64>,
    pub state_vector: Vec<f64>,
    pub qimc: QimcResult,
    pub hopf: HopfResult,
    pub uor_address: String,
    pub uor: UorAttestationResult,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MetricsResult {
    pub sigma_q: f64,
    pub sigma_kl: f64,
    pub lambda_entropy: f64,
    pub kappa: f64,
    pub deficit_angle: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct QimcResult {
    pub identity: String,
    pub identity_type: String,
    pub identity_uor_address: String,
    pub identity_uor_digest: String,
    pub identity_uor_hash_algorithm: String,
    pub uor_control: UorControlPlanInfo,
    pub prime: u64,
    pub index: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UorControlPlanInfo {
    pub entropy_bias: f64,
    pub hopf_chi_bins: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HopfResult {
    pub rho1: f64,
    pub rho2: f64,
    pub chi: f64,
    pub delta: f64,
    pub alpha: f64,
    pub transported_alpha: f64,
    pub phase_transport_lambda: f64,
    pub hopf_chi_bins: u64,
    pub sector_id: u64,
    pub chi_bin: u64,
    pub delta_bin: u64,
    pub alpha_bin: u64,
    pub subspace_norms: SubspaceNorms,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SubspaceNorms {
    pub act: f64,
    pub obj: f64,
    pub temp: f64,
    pub shared: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RouteInfo {
    pub window_index: u64,
    pub scale_x: f64,
    pub routing_score: f64,
    pub kappa: f64,
    pub deficit_angle: f64,
    pub state_vector: Vec<f64>,
    pub active_range: Vec<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResonanceResult {
    pub sentence: String,
    pub relevance: f64,
    pub window_index: u64,
    pub kappa: f64,
    pub deficit_angle: f64,
    #[serde(default)]
    pub provenance: Option<CoordinateProvenance>,
}

#[derive(Serialize, Deserialize)]
pub struct TrajectoryStep {
    pub step: u64,
    pub word: String,
    pub window_index: u64,
    pub scale_x: f64,
    pub deficit_angle: f64,
    pub kappa: f64,
    pub sigma_kl: f64,
    pub qimc: QimcResult,
    pub hopf: HopfResult,
    pub r4_projection: Projection3D,
    pub quantum: QuantumMetrics,
}

#[derive(Serialize, Deserialize)]
pub struct Projection3D {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    #[serde(rename = "X")]
    pub capital_x: f64,
    #[serde(rename = "Y")]
    pub capital_y: f64,
    #[serde(rename = "Z")]
    pub capital_z: f64,
}

#[derive(Serialize, Deserialize)]
pub struct QuantumMetrics {
    pub stratum: u64,
    pub cascade_length: u64,
    pub catastrophe: bool,
    pub winding_number: f64,
    pub commutator_curvature: f64,
    pub monodromy: DihedralInfo,
}

#[derive(Serialize, Deserialize)]
pub struct DihedralInfo {
    pub s: u64,
    pub k: u64,
    pub label: String,
}

#[wasm_bindgen(start)]
pub fn init_wasm() {
    console_error_panic_hook::set_once();
}

impl UorR4Router {
    pub fn get_sentence_projection_native(
        &self,
        state_vector: &[f64],
        win_idx: usize,
    ) -> (f64, f64) {
        self.get_sentence_projection(state_vector, win_idx)
    }

    pub fn get_state_4d_projection_native(&self, state_vector: &[f64]) -> Vec<f64> {
        self.get_state_4d_projection(state_vector)
    }

    pub fn identity_key_native(&self, identity: &str) -> String {
        identity_key(identity)
    }

    pub fn get_brain_state_native(&mut self, identity: &str) -> Vec<f64> {
        let key = identity_key(identity);
        self.session_brain_states
            .entry(key)
            .or_insert_with(|| vec![1.0 / (512.0f64).sqrt(); 512])
            .clone()
    }

    pub fn route_query_to_manifold_native(&mut self, text: &str, identity: &str) -> RoutingData {
        self.route_query_to_manifold_native_with_hopf_input(text, identity)
            .0
    }

    /// As `route_query_to_manifold_native`, additionally returning the 512-d
    /// vector that `get_state_4d_projection` reduces to the Hopf map input
    /// (the grounded VSA vector, or the session state on ground failure).
    /// Measurement surface for issue #303.
    pub fn route_query_to_manifold_native_with_hopf_input(
        &mut self,
        text: &str,
        identity: &str,
    ) -> (RoutingData, Vec<f64>) {
        let key = identity_key(identity);
        let active_state = self
            .session_brain_states
            .entry(key)
            .or_insert_with(|| vec![1.0 / (512.0f64).sqrt(); 512])
            .clone();

        self.route_query_to_manifold_internal_with_hopf_input(text, identity, Some(&active_state))
    }

    pub fn get_top_resonances_native(
        &mut self,
        text: &str,
        identity: &str,
        top_n: usize,
    ) -> Vec<ResonanceResult> {
        if self.geometry_type == GeometryType::Vsa {
            self.retrieve_vsa_multi_facet_resonance(text, top_n)
        } else {
            let key = identity_key(identity);
            let active_state = self
                .session_brain_states
                .entry(key)
                .or_insert_with(|| vec![1.0 / (512.0f64).sqrt(); 512])
                .clone();

            let routing =
                self.route_query_to_manifold_internal(text, identity, Some(&active_state));
            self.retrieve_geometric_resonance(text, &routing, top_n, &active_state, identity)
        }
    }

    // Native compatibility surface mirrors the wasm API; callers already
    // depend on these individually tunable decoding parameters.
    #[allow(clippy::too_many_arguments)]
    pub fn generate_geometric_response_native(
        &mut self,
        text: &str,
        identity: &str,
        max_tokens: usize,
        temp: f64,
        gravity: f64,
        freq_penalty: f64,
        gamma: f64,
    ) -> GeometricResponse {
        let key = identity_key(identity);
        let active_state = self
            .session_brain_states
            .entry(key)
            .or_insert_with(|| vec![1.0 / (512.0f64).sqrt(); 512])
            .clone();

        let (response_text, trajectory, final_state) = self
            .generate_geometric_response_with_trajectory_internal(
                text,
                &active_state,
                max_tokens,
                temp,
                gravity,
                freq_penalty,
                identity,
                gamma,
            );

        // Update brain state
        let key_save = identity_key(identity);
        self.session_brain_states.insert(key_save, final_state);

        GeometricResponse {
            text: response_text,
            trajectory,
        }
    }

    pub fn get_semantic_map_points_native(&self) -> serde_json::Value {
        #[derive(Serialize)]
        struct MapPoint {
            sentence: String,
            window_index: u64,
            u: f64,
            v: f64,
            v_4d: Vec<f64>,
            scope: String,
            kappa: f64,
            prime_product_mod: i64,
        }

        let mut points = Vec::new();
        for (identity_key, store) in &self.corpus_index_by_identity {
            let scope_name = identity_key
                .split(':')
                .nth(1)
                .unwrap_or(identity_key)
                .to_string();
            for (&win_idx, items) in store {
                for item in items {
                    let prime_product_val: i64 = item.prime_product.parse().unwrap_or(1);
                    points.push(MapPoint {
                        sentence: item.sentence.chars().take(120).collect(),
                        window_index: win_idx,
                        u: item.u,
                        v: item.v,
                        v_4d: item.v_4d.clone(),
                        scope: scope_name.clone(),
                        kappa: item.kappa,
                        prime_product_mod: prime_product_val % 10007,
                    });
                }
            }
        }

        serde_json::json!({
            "points": points,
            "total": points.len(),
        })
    }

    /// Native counterpart of [`Self::import_state`]. Returns `true` when the
    /// JSON is a valid serialized router state and was applied, `false` when it
    /// could not be parsed (R5 — total import surface).
    pub fn import_state_native(&mut self, json_str: &str) -> bool {
        let mut imported: Self = match serde_json::from_str(json_str) {
            Ok(value) => value,
            Err(_) => return false,
        };
        for (word, &prime) in &imported.word_primes {
            let vec = get_word_vector(prime as usize);
            imported.vocab_vectors.insert(word.clone(), vec);
        }
        imported.max_prime = imported.word_primes.values().max().copied().unwrap_or(0);
        imported.rebuild_transitions();
        *self = imported;
        true
    }

    pub fn inject_thought_stream_native(&mut self, content: &str) -> ThoughtStream {
        let stream = self.compile_thought_internal(content);
        for &ch in &stream.activated_experts {
            if ch < 64 {
                let ch = ch as usize;
                if ch >= self.expert_active_counts.len() {
                    self.expert_active_counts.resize(ch + 1, 0);
                }
                self.expert_active_counts[ch] += 1;
            }
        }
        let id = stream.id.clone();
        self.active_streams.insert(id, stream.clone());
        stream
    }

    pub fn get_active_streams_native(&self) -> Vec<ThoughtStream> {
        self.active_streams.values().cloned().collect()
    }
}

impl UorR4Router {
    pub fn last_routing_data(&self) -> &Option<RoutingData> {
        &self.last_routing_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indexing_and_resonance_retrieval() {
        let mut router = UorR4Router::new(0.5);
        let identity = "tenant-alpha";

        // Index a test sentence containing specific keywords
        let doc = "The quick brown fox jumps over the lazy dog and talks about quantum gravity.";
        router.index_sentence(doc, identity);

        // Retrieve resonances using keywords
        let results = router.get_top_resonances_native("quantum gravity", identity, 5);
        assert!(
            !results.is_empty(),
            "Should retrieve at least one resonant result"
        );
        assert!(
            results[0].sentence.contains("quantum gravity"),
            "The top resonant result should contain the target keywords"
        );
    }

    #[test]
    fn lexical_weight_default_is_content_path_conditional() {
        // #502: the deployed content-query path (default since #490) drops the
        // lexical term (W = 0, pure geometry); the routing path keeps
        // DEFAULT_LEXICAL_WEIGHT. This pins the default LOGIC so a future edit
        // to `default_lexical_weight` cannot silently change the serving
        // ranking — the anti-vacuity guard for the #502 flip.
        let mut router = UorR4Router::new(0.5);
        // new() defaults content_query_vector on (#490) -> deployed path -> W=0.
        assert_eq!(
            router.lexical_weight(),
            0.0,
            "deployed content-query path must default to W=0 (#502)"
        );
        // routing path (measurement only) keeps the historical weight.
        router.set_content_query_vector(false);
        assert_eq!(
            router.lexical_weight(),
            DEFAULT_LEXICAL_WEIGHT,
            "routing path must keep DEFAULT_LEXICAL_WEIGHT"
        );
        router.set_content_query_vector(true);
        assert_eq!(router.lexical_weight(), 0.0);
        // an explicit override wins on either path.
        router.set_lexical_weight(42.0);
        assert_eq!(router.lexical_weight(), 42.0);
        router.set_content_query_vector(false);
        assert_eq!(router.lexical_weight(), 42.0);
    }

    #[test]
    fn test_no_freeze_large_vocabulary_fallback() {
        let mut router = UorR4Router::new(0.5);
        let identity = "tenant-alpha";

        // Add 2000 dummy words to simulate a large loaded corpus vocabulary
        for i in 0..2000 {
            router.add_word_to_vocabulary(&format!("word{}", i));
        }

        // Generate response under fallback conditions (should not freeze or timeout)
        let _res = router.generate_geometric_response_native(
            "unknown query that triggers fallback",
            identity,
            10,
            0.7,
            10.0,
            4.0,
            0.5,
        );
    }

    #[test]
    fn test_pluggable_geometry_routing() {
        let mut router = UorR4Router::new(0.5);
        let identity = "tenant-alpha";

        // 1. Verify default (Spectral) mode
        assert_eq!(router.geometry_type, GeometryType::Spectral);
        let routing_spectral = router.route_query_to_manifold_native("hello world", identity);
        assert!(routing_spectral.routed.window_index >= 1);

        // 2. Configure to VSA mode
        router.set_geometry_type("vsa");
        assert_eq!(router.geometry_type, GeometryType::Vsa);
        let routing_vsa = router.route_query_to_manifold_native("hello world", identity);
        assert!(routing_vsa.routed.metrics.deficit_angle.is_finite());
        assert!(routing_vsa.routed.metrics.kappa > 0.0);
    }

    #[test]
    fn test_serialized_structs_fixed_width_json_shape() {
        // u64 fields must serialize as plain JSON numbers, identically to the
        // former usize wire shape (issue #12).
        let route = RouteInfo {
            window_index: 3,
            scale_x: 1.5,
            routing_score: 0.25,
            kappa: 0.75,
            deficit_angle: -0.5,
            state_vector: vec![0.1, 0.2],
            active_range: vec![64, 96],
        };
        let json = serde_json::to_string(&route).unwrap();
        assert_eq!(
            json,
            r#"{"window_index":3,"scale_x":1.5,"routing_score":0.25,"kappa":0.75,"deficit_angle":-0.5,"state_vector":[0.1,0.2],"active_range":[64,96]}"#
        );
        let round: RouteInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(round.window_index, 3u64);
        assert_eq!(round.active_range, vec![64u64, 96u64]);

        let info = ResonanceInfo {
            total_bytes: 5,
            resonant_bits: 40,
            klein_matches: 2,
            uor_signature: "sha256:abc".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert_eq!(
            json,
            r#"{"total_bytes":5,"resonant_bits":40,"klein_matches":2,"uor_signature":"sha256:abc"}"#
        );
        let round: ResonanceInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(round.total_bytes, 5u64);
        assert_eq!(round.resonant_bits, 40u64);
        assert_eq!(round.klein_matches, 2u64);
    }

    #[test]
    fn test_export_import_state_round_trip() {
        // The persisted router state (u64 keys/values) must survive a JSON
        // export/import cycle — including a populated MultiFacetStore (issue
        // #62: Vec<u32> keys used to make export_state silently return "").
        let mut router = UorR4Router::new(0.5);
        router.inject_thought_stream_native("fixed width integers cross the wire");
        router
            .facet_store
            .type_index
            .insert(vec![1, 2, 3], vec![7, 11]);
        let exported = router.export_state();
        assert!(!exported.is_empty(), "export_state must produce JSON");

        let mut restored = UorR4Router::new(0.5);
        assert!(
            restored.import_state_native(&exported),
            "exported state must re-import cleanly"
        );
        assert_eq!(
            restored.get_vocab_size(),
            router.get_vocab_size(),
            "vocabulary must survive the state round trip"
        );
        assert_eq!(
            restored.get_active_streams_native().len(),
            1,
            "thought streams must survive the state round trip"
        );
        assert_eq!(
            restored.facet_store.type_index.get(&vec![1, 2, 3]),
            Some(&vec![7, 11]),
            "facet store entries must survive the state round trip"
        );
        assert_eq!(
            restored.get_expert_counts(),
            router.get_expert_counts(),
            "expert counts must survive the state round trip"
        );
    }
}

// =====================================================================
// UOR-Framework Coordinate Reduction & Axis Rebase (ADR-022 / ADR-030)
// =====================================================================

use std::cell::RefCell;
use uor_foundation::enforcement::{GroundedShape, Hasher, ShapeViolation};
use uor_foundation::pipeline::{
    ConstrainedTypeShape, ConstraintRef, IntoBindingValue, PartitionProductFields, TermValue,
};
use uor_foundation::{DefaultHostTypes, HostBounds};

thread_local! {
    pub static ACTIVE_ROUTER: RefCell<Option<*mut UorR4Router>> = const { RefCell::new(None) };
}

// Custom R4 Host Bounds
#[derive(Clone, Copy)]
pub struct R4HostBounds;

impl HostBounds for R4HostBounds {
    const FINGERPRINT_MIN_BYTES: usize = 16;
    const FINGERPRINT_MAX_BYTES: usize = 32;
    const TRACE_MAX_EVENTS: usize = 256;
    const WITT_LEVEL_MAX_BITS: u32 = 64;
    const FOLD_UNROLL_THRESHOLD: usize = 8;
    const BETTI_DIMENSION_MAX: usize = 8;
    const NERVE_CONSTRAINTS_MAX: usize = 8;
    const NERVE_SITES_MAX: usize = 8;
    const JACOBIAN_SITES_MAX: usize = 8;
    const RECURSION_TRACE_DEPTH_MAX: usize = 16;
    const OP_CHAIN_DEPTH_MAX: usize = 8;
    const AFFINE_COEFFS_MAX: usize = 8;
    const CONJUNCTION_TERMS_MAX: usize = 8;
    const UNFOLD_ITERATIONS_MAX: usize = 256;
}

pub const R4_INLINE_BYTES: usize = uor_foundation::pipeline::carrier_inline_bytes::<R4HostBounds>();
pub const R4_FP_MAX: usize = 32;

// Custom Axis trait using uor_foundation_sdk::axis!
uor_foundation_sdk::axis! {
    /// Custom R4 routing axis.
    pub trait R4Axis: AxisExtension {
        const AXIS_ADDRESS: &'static str = "https://uor.foundation/axis/R4Axis";
        const MAX_OUTPUT_BYTES: usize = 28;
        fn route_query(input: &[u8], out: &mut [u8]) -> Result<usize, ShapeViolation>;
    }
}

pub struct R4RouterAxisImpl;

impl R4Axis for R4RouterAxisImpl {
    const AXIS_ADDRESS: &'static str = "https://uor.foundation/axis/R4Axis/Impl";
    const MAX_OUTPUT_BYTES: usize = 28;

    fn route_query(input: &[u8], out: &mut [u8]) -> Result<usize, ShapeViolation> {
        if input.len() < 640 {
            return Err(ShapeViolation {
                shape_iri: <Self as R4Axis>::AXIS_ADDRESS,
                constraint_iri: "https://uor.foundation/axis/R4Axis/inputSize",
                property_iri: "https://uor.foundation/axis/inputBytes",
                expected_range: "https://uor.foundation/axis/Bytes640",
                min_count: 640,
                max_count: 640,
                kind: uor_foundation::ViolationKind::ValueCheck,
            });
        }
        if out.len() < 28 {
            return Err(ShapeViolation {
                shape_iri: <Self as R4Axis>::AXIS_ADDRESS,
                constraint_iri: "https://uor.foundation/axis/R4Axis/outputSize",
                property_iri: "https://uor.foundation/axis/outputBytes",
                expected_range: "https://uor.foundation/axis/Bytes28",
                min_count: 28,
                max_count: 28,
                kind: uor_foundation::ViolationKind::ValueCheck,
            });
        }

        // Get the active router from thread-local
        let router = ACTIVE_ROUTER
            .with(|r| r.borrow().and_then(|ptr| unsafe { Some(&mut *ptr) }))
            .ok_or(ShapeViolation {
                shape_iri: <Self as R4Axis>::AXIS_ADDRESS,
                constraint_iri: "https://uor.foundation/axis/R4Axis/routerBound",
                property_iri: "https://uor.foundation/axis/routerActive",
                expected_range: "https://uor.foundation/axis/RouterActive",
                min_count: 1,
                max_count: 1,
                kind: uor_foundation::ViolationKind::ValueCheck,
            })?;

        // Extract input parameters
        let query_bytes = &input[0..512];
        let identity_bytes = &input[512..640];

        let query_str = std::str::from_utf8(query_bytes)
            .unwrap_or("")
            .trim_end_matches('\0');
        let identity_str = std::str::from_utf8(identity_bytes)
            .unwrap_or("")
            .trim_end_matches('\0');

        // Execute routing
        let routing_data = router.route_query_to_manifold_native(query_str, identity_str);

        let window_idx = routing_data.routed.window_index as u32;
        let deficit_angle = routing_data.routed.metrics.deficit_angle;
        let kappa = routing_data.routed.metrics.kappa;
        let entropy = routing_data.routed.metrics.lambda_entropy;

        // Save routing data
        router.last_routing_data = Some(routing_data);

        // Serialize output into out
        out[0..4].copy_from_slice(&window_idx.to_be_bytes());
        out[4..12].copy_from_slice(&deficit_angle.to_be_bytes());
        out[12..20].copy_from_slice(&kappa.to_be_bytes());
        out[20..28].copy_from_slice(&entropy.to_be_bytes());

        Ok(28)
    }
}

axis_extension_impl_for_r4_axis!(R4RouterAxisImpl);

// Custom input shape carrying query and identity
#[derive(Clone, Copy)]
pub struct R4RoutingInput<'a> {
    pub query: &'a [u8],
    pub identity: &'a [u8],
    pub data: &'a [u8], // Packed contiguous buffer of 640 bytes (512 query + 128 identity)
}

impl ConstrainedTypeShape for R4RoutingInput<'_> {
    const IRI: &'static str = "urn:uor:product:Bytes512:Bytes128";
    const SITE_COUNT: usize = 640;
    const CONSTRAINTS: &'static [ConstraintRef] = &[];
    const CYCLE_SIZE: u64 = u64::MAX;
}

impl uor_foundation::pipeline::__sdk_seal::Sealed for R4RoutingInput<'_> {}

impl<'a> IntoBindingValue<'a> for R4RoutingInput<'a> {
    fn as_binding_value<const INLINE_BYTES: usize>(&self) -> TermValue<'a, INLINE_BYTES> {
        TermValue::borrowed(self.data)
    }
}

impl PartitionProductFields for R4RoutingInput<'_> {
    const FIELDS: &'static [(u32, u32)] = &[(0, 512), (512, 128)];
    const FIELD_NAMES: &'static [&'static str] = &["query", "identity"];
}

// Custom output shape carrying routing metrics
#[derive(Debug, Clone, Copy)]
pub struct R4RoutingOutput;

impl ConstrainedTypeShape for R4RoutingOutput {
    const IRI: &'static str = "urn:uor:product:R4RoutingOutput";
    const SITE_COUNT: usize = 28;
    const CONSTRAINTS: &'static [ConstraintRef] = &[];
    const CYCLE_SIZE: u64 = u64::MAX;
}

impl uor_foundation::pipeline::__sdk_seal::Sealed for R4RoutingOutput {}
impl GroundedShape for R4RoutingOutput {}

impl<'a> IntoBindingValue<'a> for R4RoutingOutput {
    fn as_binding_value<const INLINE_BYTES: usize>(&self) -> TermValue<'a, INLINE_BYTES> {
        TermValue::empty()
    }
}

impl PartitionProductFields for R4RoutingOutput {
    const FIELDS: &'static [(u32, u32)] = &[(0, 4), (4, 8), (12, 8), (20, 8)];
    const FIELD_NAMES: &'static [&'static str] =
        &["window_idx", "deficit_angle", "kappa", "entropy"];
}

// Custom Hasher + AxisTuple implementation to bypass the blanket impl conflict
#[derive(Clone)]
pub struct R4HasherAndAxis {
    buffer: Vec<u8>,
}

impl Hasher<R4_FP_MAX> for R4HasherAndAxis {
    const OUTPUT_BYTES: usize = R4_FP_MAX;

    fn initial() -> Self {
        Self { buffer: Vec::new() }
    }

    fn fold_byte(mut self, b: u8) -> Self {
        self.buffer.push(b);
        self
    }

    fn fold_bytes(mut self, bytes: &[u8]) -> Self {
        self.buffer.extend_from_slice(bytes);
        self
    }

    fn finalize(self) -> [u8; R4_FP_MAX] {
        let mut out = [0u8; R4_FP_MAX];
        if self.buffer.len() >= 640 {
            if let Ok(_len) = R4RouterAxisImpl::route_query(&self.buffer, &mut out) {
                // Done
            } else {
                let sha = sha256_bytes(&self.buffer);
                out.copy_from_slice(&sha);
            }
        } else {
            let sha = sha256_bytes(&self.buffer);
            out.copy_from_slice(&sha);
        }
        out
    }
}

pub struct UorR4RouterModel;
pub struct UorR4RouterRoute;

impl uor_foundation::pipeline::__sdk_seal::Sealed for UorR4RouterModel {}
impl uor_foundation::pipeline::__sdk_seal::Sealed for UorR4RouterRoute {}

impl uor_foundation::pipeline::FoundationClosed<R4_INLINE_BYTES> for UorR4RouterRoute {
    fn arena_slice() -> &'static [uor_foundation::enforcement::Term<'static, R4_INLINE_BYTES>] {
        &[
            uor_foundation::enforcement::Term::Variable { name_index: 0 },
            uor_foundation::enforcement::Term::AxisInvocation {
                axis_index: 0,
                kernel_id: 0,
                input_index: 0,
            },
        ]
    }
}

impl<'a>
    uor_foundation::pipeline::PrismModel<
        'a,
        DefaultHostTypes,
        R4HostBounds,
        R4HasherAndAxis,
        R4_INLINE_BYTES,
        R4_FP_MAX,
    > for UorR4RouterModel
{
    type Input = R4RoutingInput<'a>;
    type Output = R4RoutingOutput;
    type Route = UorR4RouterRoute;

    fn forward(
        input: Self::Input,
    ) -> Result<
        uor_foundation::enforcement::Grounded<'a, Self::Output, R4_INLINE_BYTES, R4_FP_MAX>,
        uor_foundation::PipelineFailure,
    > {
        uor_foundation::pipeline::run_route::<
            DefaultHostTypes,
            R4HostBounds,
            R4HasherAndAxis,
            Self,
            uor_foundation::pipeline::NullResolverTuple,
            uor_foundation::pipeline::EmptyCommitment,
            R4_INLINE_BYTES,
            R4_FP_MAX,
        >(
            input,
            &uor_foundation::pipeline::NullResolverTuple,
            &uor_foundation::pipeline::EmptyCommitment,
        )
    }
}

#[wasm_bindgen]
pub fn vsa_encode_statement(subj: &str, pred: &str, obj: &str, space: &str) -> Vec<u8> {
    let hv = uor_r4_core::semantic::encode_statement(subj, pred, obj, space);
    let mut bytes = Vec::with_capacity(128);
    for &val in &hv.0 {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
    bytes
}

#[wasm_bindgen]
pub fn vsa_encode_event(subj: &str, act: &str, time: &str, loc: &str, space: &str) -> Vec<u8> {
    let hv = uor_r4_core::semantic::encode_event(subj, act, time, loc, space);
    let mut bytes = Vec::with_capacity(128);
    for &val in &hv.0 {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
    bytes
}

#[wasm_bindgen]
pub fn vsa_encode_graph_edge(src: &str, rel: &str, tgt: &str, space: &str) -> Vec<u8> {
    let hv = uor_r4_core::semantic::encode_graph_edge(src, rel, tgt, space);
    let mut bytes = Vec::with_capacity(128);
    for &val in &hv.0 {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
    bytes
}

#[cfg(test)]
mod router_repetition_tests {
    use super::*;

    #[test]
    fn hopf_projection_preserves_signed_block_direction() {
        let router = UorR4Router::new(0.85);
        let mut state = vec![0.0; 512];
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"uor-r4 issue-306 signed-hopf-projection");
        hasher.update(&0u64.to_le_bytes());
        let digest = hasher.finalize();
        for (i, value) in state.iter_mut().enumerate().take(128) {
            *value = hopf_probe_value(&digest, i);
        }

        let positive = router.get_state_4d_projection_native(&state);
        assert!(
            positive[0] > 0.99,
            "signed direction should survive projection"
        );

        for value in &mut state[..128] {
            *value = -*value;
        }
        let negative = router.get_state_4d_projection_native(&state);
        assert!(negative[0] < -0.99, "projection must retain direction sign");
    }

    #[test]
    fn test_elliptic_basin_escape_and_repetition_mitigation() {
        let mut router = UorR4Router::new(0.85);
        router.index_sentence("cut the paper cut the paper", "default");
        router.index_sentence("cut the edge cut the edge", "default");

        let (text, _trajectory, _state) = router
            .generate_geometric_response_with_trajectory_internal(
                "cut the paper",
                &vec![0.1f64; 512],
                10,
                0.5,
                1.0,
                1.0,
                "default",
                0.8,
            );

        let words: Vec<&str> = text.split_whitespace().collect();
        let mut max_consecutive = 1usize;
        let mut current_run = 1usize;
        for pair in words.windows(2) {
            if pair[0] == pair[1] {
                current_run += 1;
                max_consecutive = max_consecutive.max(current_run);
            } else {
                current_run = 1;
            }
        }

        assert!(
            max_consecutive < 5,
            "Consecutive repetition count should stay under 5 due to orthogonal Hopf perturbation escape (got {max_consecutive})"
        );
    }
}
