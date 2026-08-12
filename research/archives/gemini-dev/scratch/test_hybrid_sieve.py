import sys
import os
import json
import numpy as np

cache_file = "/Users/adminamn/gemini-dev/manifold_cache.json"
with open(cache_file, "r") as f:
    cache = json.load(f)

vocab = cache["vocabulary"]
word_primes = cache["word_primes"]
vocab_vectors = {w: np.array(v) for w, v in cache["vocab_vectors"].items()}
corpus_index = {}
for win_idx_str, items in cache["corpus_index"].items():
    win_idx = int(win_idx_str)
    deserialized_items = []
    for item in items:
        deserialized_items.append({
            "sentence": item["sentence"],
            "state_vector": np.array(item["state_vector"]),
            "kappa": float(item["kappa"]),
            "deficit_angle": float(item["deficit_angle"]),
            "words": item.get("words", []),
            "prime_product": int(item.get("prime_product", 1))
        })
    corpus_index[win_idx] = deserialized_items

M_MAX = 512
NUM_WINDOWS = 16
N_SAMPLES = 257
SPARSE_RADIUS = 0.3

# QUERY_STOPWORDS
QUERY_STOPWORDS = {
    "the", "of", "is", "a", "in", "and", "to", "for", "on", "with", "at", "by", "an", "be", "this", "that", "from", 
    "are", "was", "were", "it", "as", "he", "she", "they", "what", "how", "why", "where", "who", "when", 
    "tell", "me", "about", "describe", "explain", "show", "give", "find", "is", "are", "do", "does", "did", "can", "could", "would", "should"
}

# Load true zeros
def load_true_zeros(M: int) -> np.ndarray:
    zeros_file = "/Users/adminamn/gemini-dev/zeta_data/zeta_zeros_100k.txt"
    gammas = []
    with open(zeros_file, 'r') as f:
        for i, line in enumerate(f):
            if i >= M:
                break
            stripped = line.strip()
            if stripped:
                gammas.append(float(stripped.split()[-1]))
    return np.array(gammas, dtype=float)

GAMMAS = load_true_zeros(M_MAX)

# Precomputed windows
PRECOMPUTED_WINDOWS = []
x_grid = np.exp(np.linspace(np.log(1e4), np.log(1e6), NUM_WINDOWS))
for idx, x in enumerate(x_grid):
    angular_phase = np.log(x) / np.log(1e6)
    center_idx = int(angular_phase * M_MAX)
    window_radius = max(4, int(M_MAX * SPARSE_RADIUS // 2))
    s_idx = max(0, center_idx - window_radius)
    e_idx = min(M_MAX, center_idx + window_radius)
    
    H = 4.0 * np.sqrt(x)
    t_grid = np.linspace(-H, H, N_SAMPLES)
    active_gammas = GAMMAS[s_idx:e_idx]
    Phi = np.exp(1j * np.outer(np.log(x + t_grid), active_gammas))
    Q, _ = np.linalg.qr(Phi, mode="reduced")
    
    PRECOMPUTED_WINDOWS.append({
        "x": x,
        "s_idx": s_idx,
        "e_idx": e_idx,
        "Q": Q
    })

def cosine_similarity(v1, v2):
    dot = np.dot(v1, v2)
    norm1 = np.linalg.norm(v1)
    norm2 = np.linalg.norm(v2)
    if norm1 == 0 or norm2 == 0:
        return 0.0
    return dot / (norm1 * norm2)

def route_query_to_manifold(text: str):
    words = [w.lower().strip(".,?!()\"';:-") for w in text.split() if w.strip()]
    S = np.zeros(M_MAX)
    word_count = 0
    for w in words:
        if w in vocab_vectors:
            S += vocab_vectors[w]
            word_count += 1
            
    candidates = []
    fallback_chars = text if text else "prime"
    
    for idx, win in enumerate(PRECOMPUTED_WINDOWS):
        s_idx = win["s_idx"]
        e_idx = win["e_idx"]
        Q = win["Q"]
        
        if word_count > 0:
            y_raw = np.real(Q @ S[s_idx:e_idx])
            y_raw = y_raw - np.mean(y_raw)
            nrm = np.linalg.norm(y_raw)
            y = y_raw / nrm if nrm > 0 else y_raw
        else:
            t_grid = np.linspace(-4.0 * np.sqrt(win["x"]), 4.0 * np.sqrt(win["x"]), N_SAMPLES)
            y_raw = np.zeros(N_SAMPLES)
            for i, char in enumerate(fallback_chars):
                val = ord(char)
                amp = (val % 8 + 1) / 8.0
                freq = ((val % 13) + 1) * 0.2
                phase = i * (np.pi / 6.0)
                y_raw += amp * np.sin(freq * t_grid + phase)
            y_raw = y_raw - np.mean(y_raw)
            nrm = np.linalg.norm(y_raw)
            y = y_raw / nrm if nrm > 0 else y_raw
            
        a_sparse = np.real(Q.conj().T @ y)
        norm = np.linalg.norm(a_sparse)
        candidates.append((idx, win, a_sparse, norm, y))
        
    best_candidate_idx = 0
    best_norm = -1.0
    for idx, (win_idx, win, a_sparse, norm, y) in enumerate(candidates):
        if norm > best_norm:
            best_norm = norm
            best_candidate_idx = idx
            
    best_idx, best_win, best_state_slice, _, best_y = candidates[best_candidate_idx]
    
    best_state = np.zeros(M_MAX)
    best_state[best_win["s_idx"]:best_win["e_idx"]] = best_state_slice
    
    all_routes = []
    for idx, win, state_slice, norm, y in candidates:
        if idx == best_idx:
            all_routes.append({
                "window_index": idx + 1,
                "active_range": [win["s_idx"], win["e_idx"]],
                "state_vector": state_slice.tolist()
            })
        else:
            all_routes.append({
                "window_index": idx + 1,
                "active_range": [win["s_idx"], win["e_idx"]],
                "state_vector": np.zeros(win["e_idx"] - win["s_idx"]).tolist()
            })
            
    return {
        "routed": {
            "window_index": best_idx + 1,
            "state_vector": best_state
        },
        "all_routes": all_routes
    }

def retrieve_geometric_resonance_hybrid(prompt_text, routing_data, top_n=5):
    words = [w.lower().strip(".,?!()\"';:-") for w in prompt_text.split() if w.strip()]
    query_primes = [word_primes[w] for w in words if w in word_primes and w not in QUERY_STOPWORDS]
    
    S = np.zeros(M_MAX)
    for w in words:
        if w in vocab_vectors:
            S += vocab_vectors[w]
            
    query_projections = {}
    for r in routing_data["all_routes"]:
        win_idx = r["window_index"]
        s_idx, e_idx = r["active_range"]
        state_vec = np.zeros(M_MAX)
        state_vec[s_idx:e_idx] = np.array(r["state_vector"])
        query_projections[win_idx] = state_vec
        
    scored = []
    for win_idx, items in corpus_index.items():
        win = PRECOMPUTED_WINDOWS[win_idx - 1]
        s_idx = win["s_idx"]
        e_idx = win["e_idx"]
        slice_norm = float(np.linalg.norm(S[s_idx:e_idx]))
        
        q_vec = query_projections.get(win_idx)
        if q_vec is None:
            continue
            
        for item in items:
            shared_count = 0
            s_prod = item["prime_product"]
            for p in query_primes:
                if s_prod % p == 0:
                    shared_count += 1
                    
            sim = cosine_similarity(q_vec, item["state_vector"])
            # Hybrid score: prime factor matches take precedence (multiplied by 100),
            # then sub-ranked by geometric cosine resonance
            relevance = shared_count * 100.0 + (sim * slice_norm)
            scored.append((item["sentence"], relevance, win_idx, sim, shared_count))
            
    scored.sort(key=lambda x: x[1], reverse=True)
    return scored[:top_n]

query = "explain how the r4 router works in this project"
print("==================================================")
print("HYBRID SIEVE RESONANCE RETRIEVAL:")
routing_data = route_query_to_manifold(query)
res_hybrid = retrieve_geometric_resonance_hybrid(query, routing_data)
for s, rel, win, sim, scount in res_hybrid:
    print(f"  Score: {rel:.6f} | Sim: {sim:.4f} | SharedPrimes: {scount} | Win: {win} | Sentence: '{s}'")
