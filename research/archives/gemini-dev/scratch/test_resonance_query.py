import sys
import os
import numpy as np

sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import server

server.load_manifold_cache("manifold_cache.json")

query = "explain how the r4 router works in this project"
routing_data = server.route_query_to_manifold(query, include_eigenvalues=True)
resonances = server.retrieve_geometric_resonance(query, routing_data, top_n=5)

print(f"Query: '{query}'")
print(f"Routed to Window {routing_data['routed']['window_index']}")
print("\nTop 5 Resonant Sentences:")
for idx, (sent, score, win_idx, kappa, deficit_angle) in enumerate(resonances):
    print(f"[{idx+1}] Score: {score:.4f} | Window: {win_idx} | κ: {kappa:.4f} | θd: {deficit_angle:.4f}")
    print(f"    Sentence: '{sent}'")
