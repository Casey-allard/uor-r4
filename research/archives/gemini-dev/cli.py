import os
import sys
import json
import math
import numpy as np
import time
import random

# Suppress debug output from server
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
import server

def main():
    print("======================================================")
    print("           UOR PRIME ROUTER - WORLD MODEL             ")
    print("        Pure Geometric Terminal Edition               ")
    print("======================================================")
    print("[*] Booting mathematical manifold...")

    server.load_manifold_cache("manifold_cache.json")

    print("[+] Core geometry initialized and ready.")
    print("Type 'exit' or 'quit' to terminate.")
    print("======================================================")

    while True:
        try:
            user_input = input("\n> You: ").strip()
            if user_input.lower() in ["exit", "quit"]:
                break
            if not user_input:
                continue

            start_time = time.time()

            # Filter user input first to keep only content words for routing
            query_words = [w.lower().strip(".,?!()\"';:-") for w in user_input.split() if w.strip()]
            content_words = [w for w in query_words if w not in server.QUERY_STOPWORDS]
            filtered_input = " ".join(content_words) if content_words else user_input

            # Calculate S for steering
            S = np.zeros(server.M_MAX)
            for w in query_words:
                if w in server.VOCAB_VECTORS:
                    S += server.VOCAB_VECTORS[w]

            # 1. Route query to R4 Manifold
            routing_data = server.route_query_to_manifold(filtered_input)
            best_route = routing_data["routed"]
            win_idx = best_route["window_index"]
            metrics = best_route["metrics"]

            # 2. Generate using GCD-steered geometry
            description, trajectory, _ = server.generate_geometric_response_with_trajectory(
                filtered_input, S,
                max_len=30,
                mac="00:1a:2b:3c:4d:5e"
            )

            elapsed = time.time() - start_time

            # Format outputs
            print(f"\n> World Model Voice ({elapsed:.3f}s):")
            print(f'  "{description}"')
            print()

            # Print step-by-step trajectory
            for step in trajectory:
                win  = step["window_index"]
                curv = step["deficit_angle"]
                energy = step["kappa"]
                entropy = math.pi - abs(curv)
                q = step.get("quantum", {})
                stratum = q.get("stratum", 0)
                cascade = q.get("cascade_length", 0)
                catastrophe = q.get("catastrophe", False)
                winding = q.get("winding_number", 0.0)
                commutator = q.get("commutator_curvature", 0.0)
                mono = q.get("monodromy", {})
                mono_label = mono.get("label", "r^0") if mono else "r^0"
                cat_flag = " [!CATASTROPHE]" if catastrophe else ""
                print(f"  [State => Window: {win} | Curvature (θd): {curv:.4f} | Energy (κ): {energy:.4f} | Entropy (λ): {entropy:.4f}]")
                print(f"    Stratum: {stratum} | Cascade: {cascade} | Winding: {winding:.3f} | Monodromy: {mono_label} | Commutator: {commutator:.4f}{cat_flag}")

            # Summary state line
            print(f"\n  [Final Window: {win_idx} | Curvature: {metrics['deficit_angle']:.4f} | Energy κ: {metrics['kappa']:.4f}]")

        except KeyboardInterrupt:
            break
        except Exception as e:
            print(f"\n[!] Geometry Error: {e}")
            import traceback
            traceback.print_exc()

if __name__ == "__main__":
    main()
