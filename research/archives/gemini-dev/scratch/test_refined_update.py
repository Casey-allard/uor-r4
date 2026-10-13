import sys
import os
import numpy as np
import time

sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import server

# Load the base cache first to get sentences
server.load_manifold_cache("manifold_cache.json")

sentences = []
for win_idx, items in server.CORPUS_INDEX.items():
    for item in items:
        sentences.append(item["sentence"])

print(f"Total sentences: {len(sentences)}")

# Re-build initial vocabulary vectors to start clean
server.build_vocabulary_vectors(server.DEFAULT_CORPUS) # Or we can just use the sentences to build the vocab
# Let's build vocabulary from the full corpus sentences
all_text = "\n".join(sentences)
server.build_vocabulary_vectors(all_text)

# We will index sentences using the raw initial prime-log vectors first
server.CORPUS_INDEX = {}
sentence_states = []

for idx, s in enumerate(sentences):
    routing_data = server.route_query_to_manifold(s)
    best = routing_data["routed"]
    idx_win = best["window_index"]
    
    s_idx, e_idx = best["active_range"]
    full_state = np.zeros(server.M_MAX)
    full_state[s_idx:e_idx] = np.array(best["state_vector"])
    
    if idx_win not in server.CORPUS_INDEX:
        server.CORPUS_INDEX[idx_win] = []
        
    server.CORPUS_INDEX[idx_win].append({
        "sentence": s,
        "state_vector": full_state,
        "kappa": best["metrics"]["kappa"],
        "deficit_angle": best["metrics"]["deficit_angle"]
    })
    
    sent_words = [w.lower().strip(".,?!()\"';:-") for w in s.split() if w.strip()]
    sentence_states.append({
        "words": sent_words,
        "state_vector": full_state
    })

# Refine word vectors keeping inactive dimensions strictly at 0.0
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
for word, count in word_counts.items():
    if count > 0:
        active_mask = (word_sums[word] != 0.0)
        vec = np.zeros(server.M_MAX)
        vec[active_mask] = (word_sums[word][active_mask] / count) - global_mean[active_mask]
        server.VOCAB_VECTORS[word] = vec

print("[+] Refined vocabulary vectors updated.")

# Now, let's run a test query: "aquifers"
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
print("\nTop 10 matches after refined update:")
for idx, (s, sim, w) in enumerate(scored[:10]):
    print(f"  [{idx+1}] Sim: {sim:.4f} | Window {w} | {s}")
