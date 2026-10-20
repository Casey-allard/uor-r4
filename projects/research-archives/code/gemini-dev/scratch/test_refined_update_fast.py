import sys
import os
import numpy as np

sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import server

# Load the manifold cache
server.load_manifold_cache("manifold_cache.json")

# Gather sentences and their corresponding state vectors from CORPUS_INDEX
sentence_states = []
for win_idx, items in server.CORPUS_INDEX.items():
    for item in items:
        sent_words = [w.lower().strip(".,?!()\"';:-") for w in item["sentence"].split() if w.strip()]
        sentence_states.append({
            "words": sent_words,
            "state_vector": item["state_vector"]
        })

print(f"Loaded {len(sentence_states)} sentences from cache.")

# Compute sums
word_sums = {}
word_counts = {}
vocab_set = set(server.VOCABULARY)
for s_data in sentence_states:
    state = s_data["state_vector"]
    for word in set(s_data["words"]):
        if word in vocab_set:
            if word not in word_sums:
                word_sums[word] = np.zeros(server.M_MAX)
                word_counts[word] = 0
            word_sums[word] += state
            word_counts[word] += 1

global_sum = np.zeros(server.M_MAX)
for s_data in sentence_states:
    global_sum += s_data["state_vector"]
global_mean = global_sum / max(1, len(sentence_states))

# Refined update: keep inactive dimensions at 0.0
refined_vocab_vectors = {}
for word, count in word_counts.items():
    if count > 0:
        active_mask = (word_sums[word] != 0.0)
        vec = np.zeros(server.M_MAX)
        vec[active_mask] = (word_sums[word][active_mask] / count) - global_mean[active_mask]
        refined_vocab_vectors[word] = vec
    else:
        # Fallback to the original/cached vector
        refined_vocab_vectors[word] = server.VOCAB_VECTORS[word]

# Temporarily assign to server.VOCAB_VECTORS
original_vocab = server.VOCAB_VECTORS
server.VOCAB_VECTORS = refined_vocab_vectors

print("[+] Refined vocabulary vectors updated (fast path).")

# Now, test query: "aquifers"
query = "aquifers"
routing_data = server.route_query_to_manifold(query)

query_projections = {}
for r in routing_data["all_routes"]:
    win_idx = r["window_index"]
    s_idx, e_idx = r["active_range"]
    state_vec = np.zeros(server.M_MAX)
    state_vec[s_idx:e_idx] = np.array(r["state_vector"])
    query_projections[win_idx] = state_vec

# Find cosine similarity of the query with all indexed sentences
scored = []
for win_idx, items in server.CORPUS_INDEX.items():
    q_vec = query_projections.get(win_idx)
    if q_vec is None:
        continue
    for item in items:
        dot = np.dot(q_vec, item["state_vector"])
        norm1 = np.linalg.norm(q_vec)
        norm2 = np.linalg.norm(item["state_vector"])
        sim = dot / (norm1 * norm2) if (norm1 > 0 and norm2 > 0) else 0.0
        scored.append((item["sentence"], sim, win_idx))

scored.sort(key=lambda x: x[1], reverse=True)
print("\nTop 10 matches after refined update (fast path):")
for idx, (s, sim, w) in enumerate(scored[:10]):
    print(f"  [{idx+1}] Sim: {sim:.4f} | Window {w} | {s}")
