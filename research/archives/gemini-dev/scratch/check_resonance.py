import sys
import os
import numpy as np

sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import server

server.load_manifold_cache("manifold_cache.json")

query = "aquifers"
routing_data = server.route_query_to_manifold(query)

query_projections = {}
for r in routing_data["all_routes"]:
    win_idx = r["window_index"]
    s_idx, e_idx = r["active_range"]
    state_vec = np.zeros(server.M_MAX)
    state_vec[s_idx:e_idx] = np.array(r["state_vector"])
    query_projections[win_idx] = state_vec

# Let's find the sentence containing "aquifers"
target_sentence = "Amongst these problems is the depletion of underground aquifers through overdrafting."

found_item = None
found_win = None
for win_idx, items in server.CORPUS_INDEX.items():
    for item in items:
        if "aquifers" in item["sentence"].lower():
            print(f"Found match in Window {win_idx}: '{item['sentence']}'")
            # Calculate cosine similarity with query projection in this window
            q_vec = query_projections[win_idx]
            dot = np.dot(q_vec, item["state_vector"])
            norm1 = np.linalg.norm(q_vec)
            norm2 = np.linalg.norm(item["state_vector"])
            sim = dot / (norm1 * norm2) if (norm1 > 0 and norm2 > 0) else 0.0
            print(f"  - Cosine Similarity: {sim:.6f}")
            print(f"  - Query norm: {norm1:.6f}, Item norm: {norm2:.6f}")
            print(f"  - Dot product: {dot:.6f}")
            # Also calculate similarity in all other windows
            for other_win in range(1, 17):
                q_vec_other = query_projections[other_win]
                dot_other = np.dot(q_vec_other, item["state_vector"])
                norm1_other = np.linalg.norm(q_vec_other)
                sim_other = dot_other / (norm1_other * norm2) if (norm1_other > 0 and norm2 > 0) else 0.0
                if sim_other > 0.1:
                    print(f"    - Sim in Window {other_win}: {sim_other:.6f}")

# Let's print the top 10 sentences by similarity in the entire CORPUS_INDEX
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
print("\nTop 10 overall matches:")
for idx, (s, sim, w) in enumerate(scored[:10]):
    print(f"  [{idx+1}] Sim: {sim:.4f} | Window {w} | {s[:80]}")
