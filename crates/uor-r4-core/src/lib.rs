//! Core implementation of R⁴: zeta-zero embeddings, Hopf coordinates,
//! prime/QIMC identity, state metrics, and integrated local transformerless
//! compilation and inference. No WASM or UOR-framework dependencies.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::f64::consts::PI;

pub mod bounded_global_exact_spin_attention;
pub mod canonical_lexical_ingestion;
pub mod cayley_dickson;
pub mod construction_causal_return_attention;
pub mod conversation_entity_spin_path_attention;
pub mod corpus_induced_spin_placement;
pub mod direct_causal_geometric_attention;
pub mod geometric_gated_delta_retention;
pub mod higher_scope_geometric_attention;
pub mod local_geometric_generation;
pub mod paragraph_entity_spin_path_attention;
pub mod prime_route_attention;
pub mod prime_route_geometric_attention;
pub mod recursive_geometric_attention;
pub mod semantic;
pub mod source_free_table;
pub mod spiralcore_operator;
pub mod transformerless;
pub mod zeta_projection;
pub mod zeta_zeros;

pub const ALPHA_4: f64 = 1.0 / (2.0 * PI); // 1 / 2π
pub const ALPHA_5: f64 = 2.0 * PI; // 2π (Unity Constraint: ALPHA_4 * ALPHA_5 = 1)

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
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        }
    }

    pub fn minkowski_norm(&self) -> f64 {
        -(self.w * self.w) + (self.x * self.x) + (self.y * self.y) + (self.z * self.z)
    }

    pub fn tangent_direction_from_origin(&self) -> Self {
        let magnitude =
            (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt();
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

#[derive(Serialize, Deserialize, Clone)]
pub struct UorAttestationResult {
    pub algorithm: String,
    pub hash_algorithm: String,
    pub hash_algorithm_id: u64,
    pub address: String,
    pub kappa_label: String,
    pub fingerprint_hex: String,
    pub verify_result: String,
    pub multihash_addresses: HashMap<String, String>,
}

pub struct UorControlPlane {
    pub entropy_bias: f64,
    pub phase_transport_lambda: f64,
    pub hopf_chi_bins: usize,
    pub window_biases: HashMap<usize, f64>,
}

pub struct IdentityMeta {
    pub identity: String,
    pub identity_type: String,
    pub identity_uor_address: String,
    pub identity_uor_digest: String,
    pub identity_uor_hash_algorithm: String,
    pub identity_uor_multihash: HashMap<String, String>,
}

// ─── Free Utility Functions ───

pub fn wrap_to_pi(theta: f64) -> f64 {
    (theta + PI).rem_euclid(2.0 * PI) - PI
}

pub fn allocate_triplet_bins_budget(
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

pub fn is_prime_value(n: usize) -> bool {
    if n < 2 {
        return false;
    }
    let limit = (n as f64).sqrt() as usize;
    for i in 2..=limit {
        if n.is_multiple_of(i) {
            return false;
        }
    }
    true
}

pub fn get_primes_6k_plus_1(count: usize) -> Vec<usize> {
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

pub fn identity_key(identity: &str) -> String {
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

pub fn identity_to_qimc_prime(identity: &str) -> (usize, usize, IdentityMeta) {
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
        identity_uor_address: multihash
            .get("sha256")
            .cloned()
            .unwrap_or_else(|| format!("sha256:{}", digest)),
        identity_uor_digest: digest,
        identity_uor_hash_algorithm: "sha256".to_string(),
        identity_uor_multihash: multihash,
    };

    (prime, idx, meta)
}

pub fn derive_uor_control_plane(identity_meta: &IdentityMeta) -> UorControlPlane {
    let digest_bytes =
        hex::decode(&identity_meta.identity_uor_digest).unwrap_or_else(|_| vec![0; 32]);
    let entropy_bias = (digest_bytes[0] as f64) / 255.0;

    let phase_transport_lambda = 0.70 + (0.60 * entropy_bias);
    let hopf_chi_bins = (2 + (entropy_bias * 3.0) as usize).clamp(2, 4);

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

pub fn generate_uor_attestation(payload: &Value) -> UorAttestationResult {
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

pub fn hopf_coordinate_components_scalar(normalized_coordinate: &[f64]) -> HashMap<String, f64> {
    let a = normalized_coordinate[0];
    let b = normalized_coordinate[1];
    let c = normalized_coordinate[2];
    let d = normalized_coordinate[3];
    let rho1 = (a * a + b * b).sqrt();
    let rho2 = (c * c + d * d).sqrt();
    let denom = (rho1 * rho1 + rho2 * rho2).sqrt().max(1e-12);
    let cos_chi = rho1 / denom;
    let sin_chi = rho2 / denom;
    let chi = sin_chi.clamp(0.0, 1.0).asin();
    let chi_u = (sin_chi * sin_chi).clamp(0.0, 1.0 - 1e-12);
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

pub fn hopf_phase_transport_components_scalar(
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

pub fn assign_sector_hopf_transport_scalar(
    normalized_coordinate: &[f64],
    k: usize,
    phase_transport_lambda: f64,
    hopf_chi_bins: usize,
) -> (usize, HashMap<String, usize>, HashMap<String, f64>) {
    let components =
        hopf_phase_transport_components_scalar(normalized_coordinate, phase_transport_lambda);
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

pub fn get_word_vector(prime: usize) -> Vec<f64> {
    use crate::zeta_zeros::ZETA_ZEROS;
    let ln_p = (prime as f64).ln();
    let mut vec = vec![0.0; 512];
    let mut sum_sq = 0.0;
    for (value, zeta_zero) in vec.iter_mut().zip(ZETA_ZEROS) {
        let val = (ln_p * zeta_zero).sin();
        *value = val;
        sum_sq += val * val;
    }
    let norm = sum_sq.sqrt();
    if norm > 0.0 {
        for value in &mut vec {
            *value = (*value / norm) * 0.1;
        }
    }
    vec
}

pub fn scale_x_for_window(window_idx: usize) -> f64 {
    let x_min = 1e4_f64;
    let x_max = 1e6_f64;
    let ratio = (window_idx - 1) as f64 / 15.0;
    (x_min.ln() + ratio * (x_max.ln() - x_min.ln())).exp()
}

pub fn get_q_proj() -> Vec<[f64; 2]> {
    let mut state = 42u64;
    let mut next_random = || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (state >> 32) as f64 / 4294967296.0
    };

    let mut p_proj = vec![[0.0; 2]; 512];
    for projection in &mut p_proj {
        let u1 = next_random().max(1e-15);
        let u2 = next_random();
        let r = (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * std::f64::consts::PI * u2;
        projection[0] = r * theta.cos();
        projection[1] = r * theta.sin();
    }

    let mut q_proj = p_proj.clone();
    let mut len0_sq = 0.0;
    for projection in &q_proj {
        len0_sq += projection[0] * projection[0];
    }
    let len0 = len0_sq.sqrt();
    if len0 > 0.0 {
        for projection in &mut q_proj {
            projection[0] /= len0;
        }
    }
    let mut dot = 0.0;
    for projection in &q_proj {
        dot += projection[0] * projection[1];
    }
    for projection in &mut q_proj {
        projection[1] -= dot * projection[0];
    }
    let mut len1_sq = 0.0;
    for projection in &q_proj {
        len1_sq += projection[1] * projection[1];
    }
    let len1 = len1_sq.sqrt();
    if len1 > 0.0 {
        for projection in &mut q_proj {
            projection[1] /= len1;
        }
    }

    q_proj
}

pub fn tokenize(text: &str) -> Vec<String> {
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

pub fn split_sentences(text: &str) -> Vec<String> {
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

pub fn cosine_similarity(v1: &[f64], v2: &[f64]) -> f64 {
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

pub fn sigma_q_from_weights(p: &[f64]) -> f64 {
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

pub fn sigma_kl_from_weights(p: &[f64]) -> f64 {
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

pub fn state_metrics_from_weights(p: &[f64]) -> (f64, f64, f64, f64, f64) {
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

pub fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}

pub fn query_stopwords() -> &'static [&'static str] {
    &[
        "the", "of", "is", "a", "in", "and", "to", "for", "on", "with", "at", "by", "an", "be",
        "this", "that", "from", "are", "was", "were", "it", "as", "he", "she", "they", "what",
        "how", "why", "where", "who", "when", "tell", "me", "about", "describe", "explain", "show",
        "give", "find", "do", "does", "did", "can", "could", "would", "should",
    ]
}

pub fn uuid_placeholder(seed: i32) -> String {
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
