import sys
import os
import time
import numpy as np

# Add workspace to path
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import server

print("[*] Loading existing cache...")
server.load_manifold_cache("manifold_cache.json")

sentences = []
for win_idx, items in server.CORPUS_INDEX.items():
    for item in items:
        sentences.append(item["sentence"])

print(f"[*] Total sentences collected: {len(sentences)}")

# Clear CORPUS_INDEX
server.CORPUS_INDEX = {}

start_time = time.time()
indexed_count = 0

for idx, s in enumerate(sentences):
    if idx > 0 and idx % 2000 == 0:
        elapsed = time.time() - start_time
        print(f"    - Progress: {idx}/{len(sentences)} sentences... Elapsed: {elapsed:.2f}s")
    try:
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
        indexed_count += 1
    except Exception as e:
        print(f"Error indexing sentence '{s[:30]}': {e}")
        continue

print(f"[+] Re-indexed {indexed_count} sentences successfully in {time.time() - start_time:.2f}s.")

print("[*] Saving new manifold cache...")
server.save_manifold_cache("manifold_cache.json")
print("[+] Finished!")
