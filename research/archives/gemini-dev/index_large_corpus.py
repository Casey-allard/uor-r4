#!/usr/bin/env python3
import os
import sys
import numpy as np
import time

# Set single-thread environment variables BEFORE importing numpy to avoid deadlocks on macOS
os.environ["VECLIB_MAXIMUM_THREADS"] = "1"
os.environ["OMP_NUM_THREADS"] = "1"
os.environ["MKL_NUM_THREADS"] = "1"
os.environ["OPENBLAS_NUM_THREADS"] = "1"

# Import server module
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
import server

def main():
    if len(sys.argv) < 2:
        print("Usage: python index_large_corpus.py <path_to_text_file>")
        sys.exit(1)
        
    filepath = sys.argv[1]
    if not os.path.exists(filepath):
        print(f"[-] File not found: {filepath}")
        sys.exit(1)
        
    print(f"[*] Reading raw text file from {filepath}...")
    try:
        with open(filepath, "r", encoding="utf-8") as f:
            content = f.read()
    except Exception as e:
        print(f"[-] Failed to read file: {e}")
        sys.exit(1)
        
    # Split content into sentences
    import re
    raw_lines = content.split("\n")
    sentences = []
    for line in raw_lines:
        line = line.strip()
        if not line:
            continue
        # Split on sentence terminals followed by whitespace
        sents = re.split(r'(?<=[.!?])\s+', line)
        for s in sents:
            s_clean = s.strip()
            # Clean sentence filters: 30 to 400 chars, at least 5 words
            if len(s_clean) > 30 and len(s_clean) < 400 and s_clean.count(" ") > 4:
                sentences.append(s_clean)
                
    print(f"[+] Extracted {len(sentences)} sentence candidates from text.")
    
    # Load current cache
    cache_file = server.CACHE_FILE
    print(f"[*] Loading existing cache: {cache_file}...")
    if os.path.exists(cache_file):
        server.load_manifold_cache(cache_file)
    else:
        print("[!] No existing cache found. Starting a fresh manifold model.")
        
    # Gather existing sentences to prevent indexing duplicate entries
    existing_sentences = set()
    for items in server.CORPUS_INDEX.values():
        for item in items:
            existing_sentences.add(item["sentence"].strip().lower())
            
    # Filter new sentences
    new_sentences = [s for s in sentences if s.strip().lower() not in existing_sentences]
    print(f"[+] Found {len(new_sentences)} new unique sentences to index onto the manifold.")
    
    if not new_sentences:
        print("[*] All sentences in the text file are already indexed. Cache is up-to-date.")
        return
        
    # 1. Update vocabulary with new terms
    print("[*] Expanding vocabulary and prime numbers coordinate mappings...")
    for s in new_sentences:
        for w in s.split():
            clean = w.strip(".,?!()\"';:-")
            server.add_word_to_vocabulary(clean)
            
    # 2. Index new sentences onto R4 manifold scale windows
    print(f"[*] Starting geometric manifold projection of {len(new_sentences)} sentences...")
    t0 = time.time()
    indexed_count = 0
    
    for idx, s in enumerate(new_sentences):
        if idx > 0 and idx % 2000 == 0:
            print(f"    - Indexing progress: {idx}/{len(new_sentences)} (elapsed: {time.time()-t0:.1f}s)...")
        try:
            routing_data = server.route_query_to_manifold(s)
            best = routing_data["routed"]
            idx_win = best["window_index"]
            s_idx, e_idx = best["active_range"]
            
            full_state = np.zeros(server.M_MAX)
            full_state[s_idx:e_idx] = np.array(best["state_vector"])
            
            if idx_win not in server.CORPUS_INDEX:
                server.CORPUS_INDEX[idx_win] = []
                
            sent_words = [w.lower().strip(".,?!()\"';:-") for w in s.split() if w.strip()]
            prime_prod = server.get_sentence_prime_product(sent_words)
            u, v = server.get_sentence_projection(full_state, idx_win)
            v_4d = server.get_state_4d_projection(full_state)
            
            server.CORPUS_INDEX[idx_win].append({
                "sentence": s,
                "state_vector": full_state,
                "kappa": best["metrics"]["kappa"],
                "deficit_angle": best["metrics"]["deficit_angle"],
                "prime_product": prime_prod,
                "words": sent_words,
                "u": u,
                "v": v,
                "v_4d": v_4d
            })
            indexed_count += 1
        except Exception as e:
            continue
            
    print(f"[+] Successfully indexed {indexed_count} sentences in {time.time() - t0:.1f} seconds.")
    
    # 3. Rebuild transitions
    print("[*] Rebuilding scale-invariant transition matrices...")
    server.rebuild_transitions_from_corpus()
    server.build_2nd_order_transitions()
    
    # 4. Save cache back to disk
    print(f"[*] Writing updated cache to {cache_file}...")
    server.save_manifold_cache(cache_file)
    print("[+] Done! Start the server now to load the updated manifold model instantly.")

if __name__ == "__main__":
    main()
