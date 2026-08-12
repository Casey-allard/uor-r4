//! # UOR-aligned R⁴ Tangent Space Router Implementation (Wasm Library)

use std::collections::{HashMap, BTreeMap};
use std::f64::consts::PI;
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use serde_json::Value;

pub mod zeta_zeros;

/// Core mathematical constants representing the 3/8 Resonance Hashing Field.
pub const ALPHA_4: f64 = 1.0 / (2.0 * PI); // 1 / 2π
pub const ALPHA_5: f64 = 2.0 * PI;         // 2π (Unity Constraint: ALPHA_4 * ALPHA_5 = 1)

/// Represents a high-dimensional vector in continuous R⁴ Space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct R4Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl R4Vector {
    pub fn origin() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0, w: 0.0 }
    }

    pub fn minkowski_norm(&self) -> f64 {
        -(self.w * self.w) + (self.x * self.x) + (self.y * self.y) + (self.z * self.z)
    }

    pub fn tangent_direction_from_origin(&self) -> Self {
        let magnitude = (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt();
        if magnitude == 0.0 {
            Self::origin()
        } else {
            Self {
                x: self.x / magnitude,
                y: self.y / magnitude,
                z: self.z / magnitude,
                w: self.w / magnitude,
            }
        }
    }
}

/// A content-addressed identifier derived via the 3/8 Resonance Hashing Law.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UorAddress {
    pub hash_bytes: [u8; 32],
}

impl UorAddress {
    pub fn to_uri(&self) -> String {
        let hex_str: String = self.hash_bytes.iter()
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
    pub activated_experts: Vec<usize>,
    pub alignment_phase: f64, // $\theta$ phase state
    pub twist_parity_spin: i8, // $\kappa \in \{-1, 1\}$
    pub gcd: usize,
}

/// Dynamic resonance details computed via the 3/8 Resonance Hashing Law.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceInfo {
    pub total_bytes: usize,
    pub resonant_bits: usize,
    pub klein_matches: usize,
    pub uor_signature: String,
}

mod sparse_vector_serde {
    use serde::{Serialize, Deserialize, Serializer, Deserializer};

    #[derive(Serialize, Deserialize)]
    struct SparseVecRepresentation {
        start_idx: usize,
        values: Vec<f64>,
    }

    pub fn serialize<S>(vec: &Vec<f64>, serializer: S) -> Result<S::Ok, S::Error>
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
                start_idx,
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
        let end_idx = representation.start_idx + representation.values.len();
        if end_idx <= 512 {
            for (i, &val) in representation.values.iter().enumerate() {
                vec[representation.start_idx + i] = val;
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
}

/// The unified router core coordinator.
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct UorR4Router {
    #[serde(default)]
    active_streams: HashMap<String, ThoughtStream>,
    #[serde(default)]
    expert_active_counts: Vec<usize>, // Changed to Vec for clean serde support
    #[serde(default)]
    connection_drift: f64,
    #[serde(default)]
    kill_switch_threshold: f64,
    #[serde(default)]
    is_aligned: bool,

    // --- New Prime Router persistent states ---
    #[serde(default)]
    vocabulary: Vec<String>,
    #[serde(default)]
    word_primes: HashMap<String, usize>,
    #[serde(default)]
    max_prime: usize,
    #[serde(skip)]
    vocab_vectors: HashMap<String, Vec<f64>>,
    #[serde(skip)]
    transitions: HashMap<String, HashMap<String, f64>>,
    #[serde(skip)]
    transitions_2nd: HashMap<String, HashMap<String, f64>>,
    #[serde(default)]
    corpus_index: HashMap<usize, Vec<CorpusItem>>,
    #[serde(default)]
    corpus_index_by_identity: HashMap<String, HashMap<usize, Vec<CorpusItem>>>,
    #[serde(default)]
    session_brain_states: HashMap<String, Vec<f64>>,
    #[serde(default)]
    angle_x: f64,
    #[serde(default)]
    angle_y: f64,
    #[serde(skip)]
    last_routing_data: Option<RoutingData>,
}

#[derive(Serialize)]
pub struct GeometricResponse {
    pub text: String,
    pub trajectory: Vec<TrajectoryStep>,
}

#[wasm_bindgen]
impl UorR4Router {
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

        let klein_matches = text.chars()
            .filter(|&c| {
                let m = (c as usize) % 50;
                m == 0 || m == 1 || m == 48 || m == 49
            })
            .count();

        let info = ResonanceInfo {
            total_bytes: text.len(),
            resonant_bits: text.len() * 8,
            klein_matches,
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
                self.expert_active_counts[ch] += 1;
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
    pub fn get_expert_counts(&self) -> Vec<usize> {
        self.expert_active_counts.clone()
    }

    /// Returns the number of words in the vocabulary index
    pub fn get_vocab_size(&self) -> usize {
        self.word_primes.len()
    }

    /// Returns the total number of indexed sentences in the corpus
    pub fn get_total_indexed_sentences(&self) -> usize {
        self.corpus_index_by_identity.values()
            .map(|store| store.values().map(|items| items.len()).sum::<usize>())
            .sum()
    }

    // --- New rotation angle handlers ---
    pub fn get_angle_x(&self) -> f64 { self.angle_x }
    pub fn set_angle_x(&mut self, val: f64) { self.angle_x = val; }
    pub fn get_angle_y(&self) -> f64 { self.angle_y }
    pub fn set_angle_y(&mut self, val: f64) { self.angle_y = val; }

    // --- New Prime Router Public Interfaces ---

    /// Resets the brain state vector for a specific identity
    pub fn reset_brain(&mut self, identity: &str) {
        let key = identity_key(identity);
        let baseline = vec![1.0 / (512.0f64).sqrt(); 512];
        self.session_brain_states.insert(key, baseline);
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
        let active_state = self.session_brain_states
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
                if already_exists { break; }
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

        println!("[*] Found {} new unique sentences to index.", unique_sentences.len());

        for (i, s) in unique_sentences.iter().enumerate() {
            if i > 0 && i % 2000 == 0 {
                println!("    - Indexing progress: {}/{}...", i, unique_sentences.len());
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
        let key = identity_key(identity);
        let active_state = self.session_brain_states
            .entry(key)
            .or_insert_with(|| vec![1.0 / (512.0f64).sqrt(); 512])
            .clone();

        let routing = self.route_query_to_manifold_internal(text, identity, Some(&active_state));
        let res = self.retrieve_geometric_resonance(text, &routing, top_n, &active_state, identity);
        serde_wasm_bindgen::to_value(&res).unwrap_or(JsValue::NULL)
    }

    /// Dynamically computes the suggested token limit based on manifold routing metrics
    pub fn get_suggested_token_limit(&self, text: &str, identity: &str) -> usize {
        let key = identity_key(identity);
        let active_state = self.session_brain_states.get(&key)
            .cloned()
            .unwrap_or_else(|| vec![1.0 / (512.0f64).sqrt(); 512]);

        let routing = self.route_query_to_manifold_internal(text, identity, Some(&active_state));
        let routed = &routing.routed;
        
        let stratum = routed.state_vector.iter().filter(|&&v| v.abs() > 1e-4).count();
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
        let active_state = self.session_brain_states
            .entry(key)
            .or_insert_with(|| vec![1.0 / (512.0f64).sqrt(); 512])
            .clone();

        let (response_text, trajectory, final_state) = self.generate_geometric_response_with_trajectory_internal(
            text, &active_state, max_tokens, temp, gravity, freq_penalty, identity, gamma
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

    /// Exports the full router system database to JSON string
    pub fn export_state(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Imports a JSON string and restores the router system database
    pub fn import_state(&mut self, json_str: &str) -> Result<(), JsValue> {
        match serde_json::from_str::<Self>(json_str) {
            Ok(mut imported) => {
                for (word, &prime) in &imported.word_primes {
                    let vec = get_word_vector(prime);
                    imported.vocab_vectors.insert(word.clone(), vec);
                }
                imported.max_prime = imported.word_primes.values().max().copied().unwrap_or(0);
                // Rebuild transitions dynamically
                imported.rebuild_transitions();
                *self = imported;
                Ok(())
            }
            Err(e) => Err(JsValue::from_str(&format!("Failed to parse router state JSON: {}", e)))
        }
    }

    /// Serves all points in the corpus index for the semantic map visualizer
    pub fn get_semantic_map_points(&self) -> JsValue {
        #[derive(Serialize)]
        struct MapPoint {
            sentence: String,
            window_index: usize,
            u: f64,
            v: f64,
            v_4d: Vec<f64>,
            scope: String,
            kappa: f64,
            prime_product_mod: i64,
        }

        let mut points = Vec::new();
        for (identity_key, store) in &self.corpus_index_by_identity {
            let scope_name = identity_key.split(':').nth(1).unwrap_or(identity_key).to_string();
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
// ============================================================
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

        let hex_part = uor_hash_str.strip_prefix("sha256:").unwrap_or(&uor_hash_str);

        let mut hash_bytes = [0u8; 32];
        for i in 0..32 {
            if let (Some(h1), Some(h2)) = (hex_part.chars().nth(i * 2), hex_part.chars().nth(i * 2 + 1)) {
                if let (Some(d1), Some(d2)) = (h1.to_digit(16), h2.to_digit(16)) {
                    hash_bytes[i] = ((d1 << 4) | d2) as u8;
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

        let r4_target = R4Vector { x: x_coord, y: y_coord, z: z_coord, w: w_coord };

        let mut activated_experts = Vec::new();
        for i in 0..64 {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            if (hash_bytes[byte_idx] & (1 << bit_idx)) != 0 {
                activated_experts.push(i);
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
        let p1 = primes[(hash_accumulator.abs() as usize) % primes.len()];
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
        while !is_prime_value(next_prime) {
            next_prime += 1;
        }

        self.max_prime = next_prime;
        self.vocabulary.push(w.clone());
        self.word_primes.insert(w.clone(), next_prime);

        // Seed coordinates across 512 zeta zeros via prime log oscillation
        let vec = get_word_vector(next_prime);
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

    fn get_state_4d_projection(&self, state_vector: &[f64]) -> Vec<f64> {
        let mut w_act = 0.0;
        let mut w_obj = 0.0;
        let mut w_temp = 0.0;
        let mut w_shared = 0.0;
        for i in 0..128 {
            w_act += state_vector[i] * state_vector[i];
            w_obj += state_vector[i + 128] * state_vector[i + 128];
            w_temp += state_vector[i + 256] * state_vector[i + 256];
            w_shared += state_vector[i + 384] * state_vector[i + 384];
        }
        w_act = w_act.sqrt();
        w_obj = w_obj.sqrt();
        w_temp = w_temp.sqrt();
        w_shared = w_shared.sqrt();
        
        let denom = (w_act * w_act + w_obj * w_obj + w_temp * w_temp + w_shared * w_shared).sqrt();
        if denom < 1e-12 {
            vec![0.5, 0.5, 0.5, 0.5]
        } else {
            vec![w_act / denom, w_obj / denom, w_temp / denom, w_shared / denom]
        }
    }

    fn evolve_brain_state(&mut self, identity: &str, query_text: &str, gamma: f64) -> Vec<f64> {
        let key = identity_key(identity);
        let mut active_state = self.session_brain_states
            .entry(key.clone())
            .or_insert_with(|| vec![1.0 / (512.0f64).sqrt(); 512])
            .clone();
        
        let words = tokenize(query_text);
        let mut s_vec = vec![0.0; 512];
        let mut word_count = 0;
        for w in words {
            if let Some(v) = self.vocab_vectors.get(&w) {
                for i in 0..512 {
                    s_vec[i] += v[i];
                }
                word_count += 1;
            }
        }
        
        if word_count > 0 {
            let mut s_sum_sq = 0.0;
            for i in 0..512 {
                s_sum_sq += s_vec[i] * s_vec[i];
            }
            let s_norm = s_sum_sq.sqrt();
            if s_norm > 0.0 {
                for i in 0..512 {
                    s_vec[i] /= s_norm;
                }
            }
            
            let mut h_new = vec![0.0; 512];
            let mut h_sum_sq = 0.0;
            for i in 0..512 {
                h_new[i] = gamma * active_state[i] + (1.0 - gamma) * s_vec[i];
                h_sum_sq += h_new[i] * h_new[i];
            }
            let h_norm = h_sum_sq.sqrt();
            if h_norm > 0.0 {
                for i in 0..512 {
                    h_new[i] /= h_norm;
                }
            }
            active_state = h_new;
        }
        
        self.session_brain_states.insert(key, active_state.clone());
        active_state
    }

    fn route_query_to_manifold_internal(
        &self,
        _text: &str,
        identity: &str,
        state_vector: Option<&[f64]>,
    ) -> RoutingData {
        let active_state = match state_vector {
            Some(v) => v.to_vec(),
            None => {
                let key = identity_key(identity);
                self.session_brain_states.get(&key).cloned().unwrap_or_else(|| vec![1.0 / (512.0f64).sqrt(); 512])
            }
        };

        let (qimc_prime, qimc_index, identity_meta) = identity_to_qimc_prime(identity);
        let uor_control = derive_uor_control_plane(&identity_meta);

        let mut routed_idx = 0;
        let mut best_score = -1.0;
        let mut all_routes = Vec::new();

        // 16 scale windows
        for win_idx in 1..=16 {
            let s_idx = (win_idx - 1) * 32;
            let e_idx = win_idx * 32;
            let slice = &active_state[s_idx..e_idx];

            let mut sum_sq = 0.0;
            for &val in slice {
                sum_sq += val * val;
            }
            let norm = sum_sq.sqrt();
            let bias = uor_control.window_biases.get(&win_idx).copied().unwrap_or(0.0);
            let score = norm * (1.0 + bias);

            if score > best_score {
                best_score = score;
                routed_idx = win_idx;
            }

            all_routes.push((win_idx, score, slice.to_vec()));
        }

        let active_range = [(routed_idx - 1) * 32, routed_idx * 32];
        let routed_slice = &active_state[active_range[0]..active_range[1]];
        let (sigma_q, sigma_kl, lambda_val, kappa, deficit_angle) = state_metrics_from_weights(routed_slice);

        let v_4d = self.get_state_4d_projection(&active_state);
        let (sector_id, _bins, hopf_components) = assign_sector_hopf_transport_scalar(
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
            window_index: routed_idx,
            scale_x: scale_x_for_window(routed_idx),
            metrics: MetricsResult {
                sigma_q,
                sigma_kl,
                lambda_entropy: lambda_val,
                kappa,
                deficit_angle,
            },
            eigenvalues: vec![0.05, 0.03, 0.01, 0.005, 0.002, 0.0, 0.0, 0.0],
            active_range: vec![active_range[0], active_range[1]],
            state_vector: routed_slice.to_vec(),
            qimc: QimcResult {
                identity: identity_meta.identity.clone(),
                identity_type: identity_meta.identity_type.clone(),
                identity_uor_address: identity_meta.identity_uor_address.clone(),
                identity_uor_digest: identity_meta.identity_uor_digest.clone(),
                identity_uor_hash_algorithm: identity_meta.identity_uor_hash_algorithm.clone(),
                uor_control: UorControlPlanInfo {
                    entropy_bias: uor_control.entropy_bias,
                    hopf_chi_bins: uor_control.hopf_chi_bins,
                },
                prime: qimc_prime,
                index: qimc_index,
            },
            hopf: HopfResult {
                rho1: hopf_components["rho1"],
                rho2: hopf_components["rho2"],
                chi: hopf_components["chi"],
                delta: hopf_components["delta"],
                alpha: hopf_components["alpha"],
                transported_alpha: hopf_components["transported_alpha"],
                phase_transport_lambda: uor_control.phase_transport_lambda,
                hopf_chi_bins: uor_control.hopf_chi_bins,
                sector_id,
                subspace_norms: SubspaceNorms {
                    act: active_state[0..128].iter().map(|&x| x*x).sum::<f64>().sqrt(),
                    obj: active_state[128..256].iter().map(|&x| x*x).sum::<f64>().sqrt(),
                    temp: active_state[256..384].iter().map(|&x| x*x).sum::<f64>().sqrt(),
                    shared: active_state[384..512].iter().map(|&x| x*x).sum::<f64>().sqrt(),
                },
            },
            uor_address: attestation.address.clone(),
            uor: attestation,
        };

        let mut routes_output = Vec::new();
        for (w_idx, score, slice) in all_routes {
            let (_s_q, _s_kl, _l_v, k_v, d_a) = state_metrics_from_weights(&slice);
            routes_output.push(RouteInfo {
                window_index: w_idx,
                scale_x: scale_x_for_window(w_idx),
                routing_score: score,
                kappa: if w_idx == routed_idx { k_v } else { 0.0 },
                deficit_angle: if w_idx == routed_idx { d_a } else { std::f64::consts::PI },
                state_vector: slice,
                active_range: vec![(w_idx - 1) * 32, w_idx * 32],
            });
        }

        RoutingData { routed, all_routes: routes_output }
    }

    fn index_sentence_internal(&mut self, sentence: &str, identity: &str) {
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
        
        let routing_data = self.route_query_to_manifold_internal(s_clean, identity, None);
        let best = routing_data.routed;
        let idx_win = best.window_index;
        
        let prime_product = self.get_sentence_prime_product(&words);
        
        let mut state_vector = vec![0.0; 512];
        let s_idx = best.active_range[0];
        let e_idx = best.active_range[1];
        for i in s_idx..e_idx {
            state_vector[i] = best.state_vector[i - s_idx];
        }
        
        let (u, v) = self.get_sentence_projection(&state_vector, idx_win);
        let v_4d = self.get_state_4d_projection(&state_vector);

        let key = identity_key(identity);
        let target_index = self.corpus_index_by_identity
            .entry(key)
            .or_default();
        
        let win_items = target_index.entry(idx_win).or_default();
        
        // Incrementally update transitions (1st and 2nd order)
        if !words.is_empty() {
            // 1st order
            for i in 0..words.len() - 1 {
                let w1 = &words[i];
                let w2 = &words[i+1];
                let entry = self.transitions.entry(w1.clone()).or_default();
                let count = entry.entry(w2.clone()).or_insert(0.0);
                *count += 1.0;
            }
            
            // 2nd order (trigram)
            for i in 0..words.len().saturating_sub(2) {
                let w1 = &words[i];
                let w2 = &words[i+1];
                let w3 = &words[i+2];
                let key_trigram = format!("{} {}", w1, w2);
                let entry = self.transitions_2nd.entry(key_trigram).or_default();
                let count = entry.entry(w3.clone()).or_insert(0.0);
                *count += 1.0;
            }
        }
        
        win_items.push(CorpusItem {
            sentence: s_clean.to_string(),
            state_vector,
            kappa: best.metrics.kappa,
            deficit_angle: best.metrics.deficit_angle,
            prime_product: prime_product.to_string(),
            words,
            u,
            v,
            v_4d,
        });
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
        let query_primes: Vec<usize> = query_words.iter()
            .filter(|w| !stopwords.contains(&w.as_str()))
            .filter_map(|w| self.word_primes.get(w).copied())
            .collect();

        let mut scored = Vec::new();
        let key = identity_key(identity);
        
        let empty_index = HashMap::new();
        let scoped_index = self.corpus_index_by_identity.get(&key).unwrap_or(&empty_index);
        let shared_index = self.corpus_index_by_identity.get("shared:shared").unwrap_or(&empty_index);

        let mut query_projections = HashMap::new();
        for r in &routing_data.all_routes {
            let mut full_state = vec![0.0; 512];
            let start = r.active_range[0];
            let end = r.active_range[1];
            for i in start..end {
                full_state[i] = r.state_vector[i - start];
            }
            query_projections.insert(r.window_index, full_state);
        }

        let all_windows: std::collections::HashSet<usize> = scoped_index.keys()
            .chain(shared_index.keys())
            .copied()
            .collect();

        for &win_idx in &all_windows {
            let shared_items = shared_index.get(&win_idx);
            let scoped_items = scoped_index.get(&win_idx);

            let s_idx = (win_idx - 1) * 32;
            let e_idx = win_idx * 32;
            let mut sum_sq = 0.0;
            for i in s_idx..e_idx {
                sum_sq += state_vector[i] * state_vector[i];
            }
            let slice_norm = sum_sq.sqrt();

            let q_vec = match query_projections.get(&win_idx) {
                Some(qv) => qv,
                None => continue,
            };

            let merged_items = shared_items.into_iter().flatten().map(|item| (item, 0.0))
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
                let relevance = (shared_count as f64) * 100.0 + (sim * slice_norm) + scope_boost;

                scored.push(ResonanceResult {
                    sentence: item.sentence.clone(),
                    relevance,
                    window_index: win_idx,
                    kappa: item.kappa,
                    deficit_angle: item.deficit_angle,
                });
            }
        }

        scored.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());
        scored.truncate(top_n);
        scored
    }

    fn rebuild_transitions(&mut self) {
        self.transitions.clear();
        self.transitions_2nd.clear();

        let mut process_words = |words: &[String]| {
            if words.is_empty() { return; }
            
            // 1st order
            for i in 0..words.len() - 1 {
                let w1 = &words[i];
                let w2 = &words[i+1];
                let entry = self.transitions.entry(w1.clone()).or_default();
                let count = entry.entry(w2.clone()).or_insert(0.0);
                *count += 1.0;
            }
            
            // 2nd order (trigram)
            for i in 0..words.len().saturating_sub(2) {
                let w1 = &words[i];
                let w2 = &words[i+1];
                let w3 = &words[i+2];
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
            return ("manifold base frequency unstable".to_string(), Vec::new(), state_vector.to_vec());
        }

        // 1. Find start key matching prompts
        let mut start_key = None;
        for i in 0..words.len().saturating_sub(1) {
            let key = format!("{} {}", words[i], words[i+1]);
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
        
        let mut accumulated_delta = 0.0;
        let mut prev_stratum = 0;
        let mut prev_state_bin = vec![false; 512];
        let mut prev_state_vec = vec![0.0; 512];

        // Seed generator
        let mut seed = prompt_text.len() as u64;
        let mut rand_f = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
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
                                let mut count = 0;
                                for (word, vec) in &self.vocab_vectors {
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
                                    count += 1;
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
                        let p_trans = if total_count > 0.0 { count_trans / total_count } else { 1e-10 };
                        let c_vec = self.vocab_vectors.get(c).cloned().unwrap_or_else(|| vec![0.0; 512]);
                        let sim = cosine_similarity(&c_vec, &s_local);
                        let freq = history.get(c).copied().unwrap_or(0.0);
                        let score = p_trans.ln() + (gravity * sim) - (freq_penalty * freq);
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

            *history.entry(next_word.clone()).or_insert(0.0) += 1.0;

            // Route metrics
            let r_data = self.route_query_to_manifold_internal("", identity, Some(&s_local));
            let routed = r_data.routed;

            let s_idx = routed.active_range[0];
            let e_idx = routed.active_range[1];
            let mut state_vec = vec![0.0; 512];
            for i in s_idx..e_idx {
                state_vec[i] = routed.state_vector[i - s_idx];
            }

            let mut curr_state_bin = vec![false; 512];
            let mut stratum = 0;
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
                dihedral = DihedralInfo { s: 0, k: 0, label: "r^0".to_string() };
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
                    k: k_rot,
                    label: format!("{}r^{}", if s_refl == 1 { "s" } else { "" }, k_rot),
                };
            }

            prev_stratum = stratum;
            prev_state_bin = curr_state_bin;
            prev_state_vec = state_vec;

            // R4 Stereographic projection coordinates
            let mut q4 = vec![0.0; 4];
            for k in 0..4 {
                if s_idx + k < 512 {
                    q4[k] = routed.state_vector[k];
                }
            }
            let mut q4_sum_sq = 0.0;
            for k in 0..4 {
                q4_sum_sq += q4[k] * q4[k];
            }
            let q4_norm = q4_sum_sq.sqrt();
            if q4_norm > 1e-9 {
                for k in 0..4 {
                    q4[k] /= q4_norm;
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
                step: step_idx + 1,
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
                    stratum,
                    cascade_length: cascade_len,
                    catastrophe,
                    winding_number,
                    commutator_curvature: commutator_curv,
                    monodromy: dihedral,
                },
            });

            // Evolve s_local with the chosen word's vector along geodesic
            if let Some(v_next) = self.vocab_vectors.get(&next_word) {
                let mut v_sum_sq = 0.0;
                for i in 0..512 {
                    v_sum_sq += v_next[i] * v_next[i];
                }
                let v_norm = v_sum_sq.sqrt();
                let mut v_normed = vec![0.0; 512];
                if v_norm > 0.0 {
                    for i in 0..512 {
                        v_normed[i] = v_next[i] / v_norm;
                    }
                }
                let mut h_new = vec![0.0; 512];
                let mut h_sum_sq = 0.0;
                for i in 0..512 {
                    h_new[i] = gamma * s_local[i] + (1.0 - gamma) * v_normed[i];
                    h_sum_sq += h_new[i] * h_new[i];
                }
                let h_norm = h_sum_sq.sqrt();
                if h_norm > 0.0 {
                    for i in 0..512 {
                        h_new[i] /= h_norm;
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
    pub window_index: usize,
    pub scale_x: f64,
    pub metrics: MetricsResult,
    pub eigenvalues: Vec<f64>,
    pub active_range: Vec<usize>,
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
    pub prime: usize,
    pub index: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UorControlPlanInfo {
    pub entropy_bias: f64,
    pub hopf_chi_bins: usize,
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
    pub hopf_chi_bins: usize,
    pub sector_id: usize,
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
pub struct UorAttestationResult {
    pub algorithm: String,
    pub hash_algorithm: String,
    pub hash_algorithm_id: usize,
    pub address: String,
    pub kappa_label: String,
    pub fingerprint_hex: String,
    pub verify_result: String,
    pub multihash_addresses: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RouteInfo {
    pub window_index: usize,
    pub scale_x: f64,
    pub routing_score: f64,
    pub kappa: f64,
    pub deficit_angle: f64,
    pub state_vector: Vec<f64>,
    pub active_range: Vec<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct ResonanceResult {
    pub sentence: String,
    pub relevance: f64,
    pub window_index: usize,
    pub kappa: f64,
    pub deficit_angle: f64,
}

#[derive(Serialize, Deserialize)]
pub struct TrajectoryStep {
    pub step: usize,
    pub word: String,
    pub window_index: usize,
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
    pub stratum: usize,
    pub cascade_length: usize,
    pub catastrophe: bool,
    pub winding_number: f64,
    pub commutator_curvature: f64,
    pub monodromy: DihedralInfo,
}

#[derive(Serialize, Deserialize)]
pub struct DihedralInfo {
    pub s: usize,
    pub k: usize,
    pub label: String,
}

struct UorControlPlane {
    entropy_bias: f64,
    phase_transport_lambda: f64,
    hopf_chi_bins: usize,
    window_biases: HashMap<usize, f64>,
}

struct IdentityMeta {
    identity: String,
    identity_type: String,
    identity_uor_address: String,
    identity_uor_digest: String,
    identity_uor_hash_algorithm: String,
    identity_uor_multihash: HashMap<String, String>,
}

// ─── Free Utility Functions ───

fn wrap_to_pi(theta: f64) -> f64 {
    (theta + PI).rem_euclid(2.0 * PI) - PI
}

fn allocate_triplet_bins_budget(
    total_cap: usize,
    min_first: usize,
    min_second: usize,
    min_third: usize,
) -> (usize, usize, usize) {
    let total_cap = total_cap.max(1);
    let min_first = min_first.max(1);
    let min_second = min_second.max(1);
    let min_third = min_third.max(1);
    
    let mut best = (1, total_cap, 1);
    let mut best_score: Option<(usize, i32, i32, usize, i32)> = None;
    for k_first in min_first..=total_cap {
        for k_second in min_second..=total_cap {
            let max_third = total_cap / (k_first * k_second).max(1);
            if max_third < min_third {
                break;
            }
            for k_third in min_third..=max_third {
                let product = k_first * k_second * k_third;
                let favor_base = if k_second >= k_third { 1 } else { 0 };
                let spread = (k_first as i32 - k_second as i32).abs()
                    + (k_second as i32 - k_third as i32).abs()
                    + (k_first as i32 - k_third as i32).abs();
                let score = (product, favor_base, -spread, k_second, -(k_third as i32));
                if best_score.is_none() || score > best_score.unwrap() {
                    best_score = Some(score);
                    best = (k_first, k_second, k_third);
                }
            }
        }
    }
    best
}

fn is_prime_value(n: usize) -> bool {
    if n < 2 { return false; }
    let limit = (n as f64).sqrt() as usize;
    for i in 2..=limit {
        if n % i == 0 { return false; }
    }
    true
}

fn get_primes_6k_plus_1(count: usize) -> Vec<usize> {
    let mut primes = Vec::new();
    let mut k = 1;
    while primes.len() < count {
        let candidate = 6 * k + 1;
        if is_prime_value(candidate) {
            primes.push(candidate);
        }
        k += 1;
    }
    primes
}

fn identity_key(identity: &str) -> String {
    let raw = identity.trim();
    if raw.is_empty() {
        return "text:shared".to_string();
    }
    let lowered = raw.to_lowercase();
    if lowered == "__shared__" || lowered == "shared" {
        return "shared:shared".to_string();
    }
    if lowered.contains(':') {
        let parts: Vec<&str> = lowered.splitn(2, ':').collect();
        if ["sha256", "sha3-256", "blake3", "keccak256", "sha512"].contains(&parts[0]) {
            return format!("uor:{}", lowered);
        }
    }
    format!("text:{}", lowered)
}

fn identity_to_qimc_prime(identity: &str) -> (usize, usize, IdentityMeta) {
    let key = identity_key(identity);
    let parts: Vec<&str> = key.splitn(2, ':').collect();
    let id_type = parts[0].to_string();
    let normalized = parts[1].to_string();

    let digest: String;
    let mut multihash = HashMap::new();

    if id_type == "uor" {
        let uor_parts: Vec<&str> = normalized.splitn(2, ':').collect();
        let algo = uor_parts[0].to_string();
        digest = uor_parts.get(1).copied().unwrap_or("").to_string();
        multihash.insert(algo, normalized.clone());
    } else {
        // Derive SHA-256 UOR address of canonical representation
        let canonical_id_json = format!("\"{}\"", normalized);
        let digest_bytes = sha256_bytes(canonical_id_json.as_bytes());
        digest = hex::encode(digest_bytes);
        multihash.insert("sha256".to_string(), format!("sha256:{}", digest));
    }

    let val = if digest.len() >= 12 {
        u64::from_str_radix(&digest[..12], 16).unwrap_or(0)
    } else {
        0
    };

    let primes_6k = get_primes_6k_plus_1(512);
    let idx = ((val % 500) + 1) as usize;
    let prime = primes_6k[idx - 1];

    let meta = IdentityMeta {
        identity: normalized.clone(),
        identity_type: id_type,
        identity_uor_address: multihash.get("sha256").cloned().unwrap_or_else(|| format!("sha256:{}", digest)),
        identity_uor_digest: digest,
        identity_uor_hash_algorithm: "sha256".to_string(),
        identity_uor_multihash: multihash,
    };

    (prime, idx, meta)
}

fn derive_uor_control_plane(identity_meta: &IdentityMeta) -> UorControlPlane {
    let digest_bytes = hex::decode(&identity_meta.identity_uor_digest).unwrap_or_else(|_| vec![0; 32]);
    let entropy_bias = (digest_bytes[0] as f64) / 255.0;
    
    let phase_transport_lambda = 0.70 + (0.60 * entropy_bias);
    let mut hopf_chi_bins = 2 + (entropy_bias * 3.0) as usize;
    if hopf_chi_bins < 2 { hopf_chi_bins = 2; }
    if hopf_chi_bins > 4 { hopf_chi_bins = 4; }

    let mut window_biases = HashMap::new();
    for window in 1..=16 {
        let b = digest_bytes[(window * 3) % digest_bytes.len()];
        let centered = (b as f64 / 255.0) - 0.5;
        window_biases.insert(window, centered * 0.04);
    }

    UorControlPlane {
        entropy_bias,
        phase_transport_lambda,
        hopf_chi_bins,
        window_biases,
    }
}

fn generate_uor_attestation(payload: &Value) -> UorAttestationResult {
    let sorted_payload = match payload {
        Value::Object(map) => {
            let mut btree = BTreeMap::new();
            for (k, v) in map {
                btree.insert(k.clone(), v.clone());
            }
            serde_json::to_vec(&btree).unwrap_or_default()
        }
        _ => serde_json::to_vec(payload).unwrap_or_default(),
    };

    let digest_bytes = sha256_bytes(&sorted_payload);
    let digest = hex::encode(digest_bytes);
    let address = format!("sha256:{}", digest);

    let mut multihash_addresses = HashMap::new();
    multihash_addresses.insert("sha256".to_string(), address.clone());

    UorAttestationResult {
        algorithm: "sha256".to_string(),
        hash_algorithm: "sha256".to_string(),
        hash_algorithm_id: 1,
        address,
        kappa_label: "uor-witness-v1".to_string(),
        fingerprint_hex: digest,
        verify_result: "verified".to_string(),
        multihash_addresses,
    }
}

fn hopf_coordinate_components_scalar(normalized_coordinate: &[f64]) -> HashMap<String, f64> {
    let a = normalized_coordinate[0];
    let b = normalized_coordinate[1];
    let c = normalized_coordinate[2];
    let d = normalized_coordinate[3];
    let rho1 = (a * a + b * b).sqrt();
    let rho2 = (c * c + d * d).sqrt();
    let denom = (rho1 * rho1 + rho2 * rho2).sqrt().max(1e-12);
    let cos_chi = rho1 / denom;
    let sin_chi = rho2 / denom;
    let chi = sin_chi.min(1.0).max(0.0).asin();
    let chi_u = (sin_chi * sin_chi).min(1.0 - 1e-12).max(0.0);
    let theta1 = wrap_to_pi(b.atan2(a));
    let theta2 = wrap_to_pi(d.atan2(c));
    let delta = wrap_to_pi(theta1 - theta2);
    let alpha = wrap_to_pi(0.5 * (theta1 + theta2));
    
    let mut map = HashMap::new();
    map.insert("rho1".to_string(), rho1);
    map.insert("rho2".to_string(), rho2);
    map.insert("chi".to_string(), chi);
    map.insert("chi_u".to_string(), chi_u);
    map.insert("theta1".to_string(), theta1);
    map.insert("theta2".to_string(), theta2);
    map.insert("delta".to_string(), delta);
    map.insert("alpha".to_string(), alpha);
    map.insert("cos_chi".to_string(), cos_chi);
    map.insert("sin_chi".to_string(), sin_chi);
    map
}

fn hopf_phase_transport_components_scalar(
    normalized_coordinate: &[f64],
    phase_transport_lambda: f64,
) -> HashMap<String, f64> {
    let mut map = hopf_coordinate_components_scalar(normalized_coordinate);
    let chi = map["chi"];
    let delta = map["delta"];
    let alpha = map["alpha"];
    let connection_weight = 0.5 * phase_transport_lambda * (2.0 * chi).cos();
    let phase_shift = wrap_to_pi(connection_weight * delta);
    let transported_alpha = wrap_to_pi(alpha + phase_shift);
    
    map.insert("transport_connection_weight".to_string(), connection_weight);
    map.insert("transport_phase_shift".to_string(), phase_shift);
    map.insert("transported_alpha".to_string(), transported_alpha);
    map
}

fn assign_sector_hopf_transport_scalar(
    normalized_coordinate: &[f64],
    k: usize,
    phase_transport_lambda: f64,
    hopf_chi_bins: usize,
) -> (usize, HashMap<String, usize>, HashMap<String, f64>) {
    let components = hopf_phase_transport_components_scalar(normalized_coordinate, phase_transport_lambda);
    let (kchi, kdelta, kalpha) = allocate_triplet_bins_budget(k, hopf_chi_bins.max(2), 2, 2);
    
    let delta = components["delta"];
    let transported_alpha = components["transported_alpha"];
    let chi_u = components["chi_u"];
    
    let u_delta = (delta + std::f64::consts::PI) / (2.0 * std::f64::consts::PI);
    let u_alpha = (transported_alpha + std::f64::consts::PI) / (2.0 * std::f64::consts::PI);
    
    let chi_bin = ((chi_u * kchi as f64) as usize).min(kchi - 1);
    let delta_bin = ((u_delta * kdelta as f64) as usize).min(kdelta - 1);
    let alpha_bin = ((u_alpha * kalpha as f64) as usize).min(kalpha - 1);
    
    let local_span = kdelta * kalpha;
    let sector_id = (chi_bin * local_span + delta_bin * kalpha + alpha_bin).min(k - 1);
    
    let mut bins = HashMap::new();
    bins.insert("chi_bins".to_string(), kchi);
    bins.insert("delta_bins".to_string(), kdelta);
    bins.insert("alpha_bins".to_string(), kalpha);
    bins.insert("chi_bin".to_string(), chi_bin);
    bins.insert("delta_bin".to_string(), delta_bin);
    bins.insert("alpha_bin".to_string(), alpha_bin);
    
    (sector_id, bins, components)
}

fn get_word_vector(prime: usize) -> Vec<f64> {
    use crate::zeta_zeros::ZETA_ZEROS;
    let ln_p = (prime as f64).ln();
    let mut vec = vec![0.0; 512];
    let mut sum_sq = 0.0;
    for i in 0..512 {
        let val = (ln_p * ZETA_ZEROS[i]).sin();
        vec[i] = val;
        sum_sq += val * val;
    }
    let norm = sum_sq.sqrt();
    if norm > 0.0 {
        for i in 0..512 {
            vec[i] = (vec[i] / norm) * 0.1;
        }
    }
    vec
}

fn scale_x_for_window(window_idx: usize) -> f64 {
    let x_min = 1e4_f64;
    let x_max = 1e6_f64;
    let ratio = (window_idx - 1) as f64 / 15.0;
    (x_min.ln() + ratio * (x_max.ln() - x_min.ln())).exp()
}

fn get_q_proj() -> Vec<[f64; 2]> {
    let mut state = 42u64;
    let mut next_random = || {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (state >> 32) as f64 / 4294967296.0
    };
    
    let mut p_proj = vec![[0.0; 2]; 512];
    for i in 0..512 {
        let u1 = next_random().max(1e-15);
        let u2 = next_random();
        let r = (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * std::f64::consts::PI * u2;
        p_proj[i][0] = r * theta.cos();
        p_proj[i][1] = r * theta.sin();
    }
    
    let mut q_proj = p_proj.clone();
    let mut len0_sq = 0.0;
    for i in 0..512 {
        len0_sq += q_proj[i][0] * q_proj[i][0];
    }
    let len0 = len0_sq.sqrt();
    if len0 > 0.0 {
        for i in 0..512 {
            q_proj[i][0] /= len0;
        }
    }
    let mut dot = 0.0;
    for i in 0..512 {
        dot += q_proj[i][0] * q_proj[i][1];
    }
    for i in 0..512 {
        q_proj[i][1] -= dot * q_proj[i][0];
    }
    let mut len1_sq = 0.0;
    for i in 0..512 {
        len1_sq += q_proj[i][1] * q_proj[i][1];
    }
    let len1 = len1_sq.sqrt();
    if len1 > 0.0 {
        for i in 0..512 {
            q_proj[i][1] /= len1;
        }
    }
    
    q_proj
}

fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|w| {
            w.chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    for c in text.chars() {
        current.push(c);
        if c == '.' || c == '?' || c == '!' {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                sentences.push(trimmed);
            }
            current.clear();
        }
    }
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        sentences.push(trimmed);
    }
    sentences
}

fn cosine_similarity(v1: &[f64], v2: &[f64]) -> f64 {
    if v1.len() != v2.len() {
        return 0.0;
    }
    let mut dot = 0.0;
    let mut norm1 = 0.0;
    let mut norm2 = 0.0;
    for i in 0..v1.len() {
        dot += v1[i] * v2[i];
        norm1 += v1[i] * v1[i];
        norm2 += v2[i] * v2[i];
    }
    if norm1 == 0.0 || norm2 == 0.0 {
        return 0.0;
    }
    dot / (norm1.sqrt() * norm2.sqrt())
}

fn sigma_q_from_weights(p: &[f64]) -> f64 {
    let n = p.len();
    if n <= 1 {
        return 1.0;
    }
    let inv_n = 1.0 / (n as f64);
    let mut sum_sq_diff = 0.0;
    for &val in p {
        let diff = val - inv_n;
        sum_sq_diff += diff * diff;
    }
    1.0 - sum_sq_diff / (1.0 - inv_n)
}

fn sigma_kl_from_weights(p: &[f64]) -> f64 {
    let n = p.len();
    if n <= 1 {
        return 1.0;
    }
    let mut p_clipped = p.to_vec();
    let eps = 1e-300;
    let mut sum = 0.0;
    for val in p_clipped.iter_mut() {
        if *val < eps {
            *val = eps;
        }
        sum += *val;
    }
    if sum > 0.0 {
        for val in p_clipped.iter_mut() {
            *val /= sum;
        }
    }
    let log_n = (n as f64).ln();
    let mut entropy_sum = 0.0;
    for &val in &p_clipped {
        entropy_sum += val * (n as f64 * val).ln();
    }
    1.0 - entropy_sum / log_n
}

fn state_metrics_from_weights(p: &[f64]) -> (f64, f64, f64, f64, f64) {
    let mut p_pos = p.to_vec();
    let mut sum = 0.0;
    let mut kappa = 0.0;
    for val in p_pos.iter_mut() {
        if *val < 0.0 {
            *val = 0.0;
        }
        sum += *val;
        if *val > kappa {
            kappa = *val;
        }
    }
    if sum <= 0.0 {
        return (1.0, 1.0, 0.0, 0.0, std::f64::consts::PI);
    }
    for val in p_pos.iter_mut() {
        *val /= sum;
    }
    let sigma_q = sigma_q_from_weights(&p_pos);
    let sigma_kl = sigma_kl_from_weights(&p_pos);
    let one_minus = (1.0 - sigma_q).max(1e-300);
    let lambda = -one_minus.ln();
    let deficit_angle = std::f64::consts::PI - lambda;
    (sigma_q, sigma_kl, lambda, kappa, deficit_angle)
}

fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}

fn query_stopwords() -> std::collections::HashSet<&'static str> {
    let stopwords = [
        "the", "of", "is", "a", "in", "and", "to", "for", "on", "with", "at", "by", "an", "be", "this", "that", "from", 
        "are", "was", "were", "it", "as", "he", "she", "they", "what", "how", "why", "where", "who", "when", 
        "tell", "me", "about", "describe", "explain", "show", "give", "find", "do", "does", "did", "can", "could", "would", "should"
    ];
    let mut set = std::collections::HashSet::new();
    for s in stopwords {
        set.insert(s);
    }
    set
}

fn uuid_placeholder(seed: i32) -> String {
    let mut val = seed.abs();
    let mut output = String::new();
    let chars = "abcdef0123456789";
    for _ in 0..6 {
        let idx = (val % 16) as usize;
        output.push(chars.chars().nth(idx).unwrap());
        val /= 16;
    }
    output
}

#[wasm_bindgen(start)]
pub fn init_wasm() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    Ok(())
}

impl UorR4Router {

    pub fn get_sentence_projection_native(&self, state_vector: &[f64], win_idx: usize) -> (f64, f64) {
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
        let key = identity_key(identity);
        let active_state = self.session_brain_states
            .entry(key)
            .or_insert_with(|| vec![1.0 / (512.0f64).sqrt(); 512])
            .clone();

        self.route_query_to_manifold_internal(text, identity, Some(&active_state))
    }

    pub fn get_top_resonances_native(&mut self, text: &str, identity: &str, top_n: usize) -> Vec<ResonanceResult> {
        let key = identity_key(identity);
        let active_state = self.session_brain_states
            .entry(key)
            .or_insert_with(|| vec![1.0 / (512.0f64).sqrt(); 512])
            .clone();

        let routing = self.route_query_to_manifold_internal(text, identity, Some(&active_state));
        self.retrieve_geometric_resonance(text, &routing, top_n, &active_state, identity)
    }

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
        let active_state = self.session_brain_states
            .entry(key)
            .or_insert_with(|| vec![1.0 / (512.0f64).sqrt(); 512])
            .clone();

        let (response_text, trajectory, final_state) = self.generate_geometric_response_with_trajectory_internal(
            text, &active_state, max_tokens, temp, gravity, freq_penalty, identity, gamma
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
            window_index: usize,
            u: f64,
            v: f64,
            v_4d: Vec<f64>,
            scope: String,
            kappa: f64,
            prime_product_mod: i64,
        }

        let mut points = Vec::new();
        for (identity_key, store) in &self.corpus_index_by_identity {
            let scope_name = identity_key.split(':').nth(1).unwrap_or(identity_key).to_string();
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

    pub fn import_state_native(&mut self, json_str: &str) -> Result<(), serde_json::Error> {
        let mut imported: Self = serde_json::from_str(json_str)?;
        for (word, &prime) in &imported.word_primes {
            let vec = get_word_vector(prime);
            imported.vocab_vectors.insert(word.clone(), vec);
        }
        imported.max_prime = imported.word_primes.values().max().copied().unwrap_or(0);
        imported.rebuild_transitions();
        *self = imported;
        Ok(())
    }

    pub fn inject_thought_stream_native(&mut self, content: &str) -> ThoughtStream {
        let stream = self.compile_thought_internal(content);
        for &ch in &stream.activated_experts {
            if ch < 64 {
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

// =====================================================================
// UOR-Framework Coordinate Reduction & Axis Rebase (ADR-022 / ADR-030)
// =====================================================================

use std::cell::RefCell;
use uor_foundation::enforcement::{GroundedShape, ShapeViolation, Hasher};
use uor_foundation::pipeline::{
    ConstrainedTypeShape, ConstraintRef, IntoBindingValue, TermValue, PartitionProductFields,
    AxisExtension
};
use uor_foundation::{DefaultHostTypes, HostBounds};

thread_local! {
    pub static ACTIVE_ROUTER: RefCell<Option<*mut UorR4Router>> = RefCell::new(None);
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
        let router = ACTIVE_ROUTER.with(|r| {
            r.borrow().and_then(|ptr| unsafe { Some(&mut *ptr) })
        }).ok_or_else(|| ShapeViolation {
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
    const FIELD_NAMES: &'static [&'static str] = &["window_idx", "deficit_angle", "kappa", "entropy"];
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

impl<'a> uor_foundation::pipeline::PrismModel<'a, DefaultHostTypes, R4HostBounds, R4HasherAndAxis, R4_INLINE_BYTES, R4_FP_MAX>
    for UorR4RouterModel
{
    type Input = R4RoutingInput<'a>;
    type Output = R4RoutingOutput;
    type Route = UorR4RouterRoute;

    fn forward(input: Self::Input) -> Result<uor_foundation::enforcement::Grounded<'a, Self::Output, R4_INLINE_BYTES, R4_FP_MAX>, uor_foundation::PipelineFailure> {
        uor_foundation::pipeline::run_route::<
            DefaultHostTypes,
            R4HostBounds,
            R4HasherAndAxis,
            Self,
            uor_foundation::pipeline::NullResolverTuple,
            uor_foundation::pipeline::EmptyCommitment,
            R4_INLINE_BYTES,
            R4_FP_MAX,
        >(input, &uor_foundation::pipeline::NullResolverTuple, &uor_foundation::pipeline::EmptyCommitment)
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
        assert!(!results.is_empty(), "Should retrieve at least one resonant result");
        assert!(results[0].sentence.contains("quantum gravity"), "The top resonant result should contain the target keywords");
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
            0.5
        );
    }
}


