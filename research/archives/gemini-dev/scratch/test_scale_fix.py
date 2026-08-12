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
            "words": item.get("words", [])
        })
    corpus_index[win_idx] = deserialized_items

# Fix the scale of vocab vectors
# If the norm of a vector in the first 50 dimensions is small, it means it's a fallback vector.
# Let's scale all vectors to have the same norm in the first 50 dimensions (e.g. 4.0),
# or scale up only the project-specific words.
TARGET_NORM = 4.0
for w in vocab_vectors:
    vec = vocab_vectors[w]
    nrm50 = np.linalg.norm(vec[:50])
    if nrm50 > 0:
        vec[:50] = (vec[:50] / nrm50) * TARGET_NORM
    else:
        # If it was all zeros, generate a random vector of norm TARGET_NORM
        rng = np.random.default_rng(hash(w) % 10000)
        vec[:50] = rng.standard_normal(50)
        vec[:50] = (vec[:50] / np.linalg.norm(vec[:50])) * TARGET_NORM

M_MAX = 512
NUM_WINDOWS = 16
N_SAMPLES = 257
SPARSE_RADIUS = 0.3

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

def retrieve_geometric_resonance_with_blend(prompt_text, routing_data, blend_glove=True, top_n=5):
    words = [w.lower().strip(".,?!()\"';:-") for w in prompt_text.split() if w.strip()]
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
        if blend_glove:
            state_vec[0:50] = S[0:50]
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
            v_sent = np.copy(item["state_vector"])
            if blend_glove:
                sent_glove = np.zeros(50)
                for w in item["words"]:
                    if w in vocab_vectors:
                        sent_glove += vocab_vectors[w][:50]
                v_sent[0:50] = sent_glove
                
            sim = cosine_similarity(q_vec, v_sent)
            relevance = sim * slice_norm
            scored.append((item["sentence"], relevance, win_idx, sim, slice_norm))
            
    scored.sort(key=lambda x: x[1], reverse=True)
    return scored[:top_n]

query = "explain how the r4 router works in this project"
print("==================================================")
print("WITH GLOVE BLENDING + SCALE FIX IN RETRIEVAL:")
routing_data = route_query_to_manifold(query)
res_blend = retrieve_geometric_resonance_with_blend(query, routing_data, blend_glove=True)
for s, rel, win, sim, snorm in res_blend:
    print(f"  Score: {rel:.6f} | Sim: {sim:.4f} | Sentence: '{s}'")
