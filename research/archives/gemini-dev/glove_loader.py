"""
glove_loader.py — GloVe-50D semantic word vectors for the R4 Prime Router.

Downloads glove.6B.50d.txt (~83MB) from Hugging Face on first run,
then saves a fast-load numpy cache. Each word's 50D vector is stored
in the first 50 dims of a 512D array (zero-padded) so it slots
directly into VOCAB_VECTORS without changing downstream code.
"""

import os
import sys
import numpy as np
import urllib.request
import zipfile
import json

GLOVE_CACHE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "glove_cache.npz")
# Use the HuggingFace-hosted mirror (much faster than Stanford NLP direct)
GLOVE_URL   = "https://huggingface.co/stanfordnlp/glove/resolve/main/glove.6B.zip"
GLOVE_ZIP   = os.path.join(os.path.dirname(os.path.abspath(__file__)), "glove.6B.zip")
GLOVE_50D   = "glove.6B.50d.txt"
M_MAX       = 512


def _progress(count, block_size, total_size):
    pct = count * block_size * 100 // total_size
    sys.stdout.write(f"\r    Downloading GloVe: {pct}%   ")
    sys.stdout.flush()


def load_glove(vocab_words: list[str]) -> dict[str, np.ndarray]:
    """
    Returns a dict mapping word -> 512D numpy array (50D GloVe + 462D zeros).
    Only words in vocab_words are loaded to keep memory usage minimal.
    Falls back to random vectors for words not found in GloVe.
    """
    vocab_set = set(w.lower() for w in vocab_words)

    # --- Try fast numpy cache first ---
    if os.path.exists(GLOVE_CACHE):
        print("[*] Loading GloVe vectors from local cache...")
        data = np.load(GLOVE_CACHE, allow_pickle=True)
        words_arr  = data["words"]
        vecs_arr   = data["vectors"]
        glove_dict = {w: v for w, v in zip(words_arr, vecs_arr)}
        result = {}
        rng = np.random.default_rng(42)
        for w in vocab_words:
            wl = w.lower()
            if wl in glove_dict:
                result[w] = glove_dict[wl]
            else:
                v = np.zeros(M_MAX)
                v[:50] = rng.standard_normal(50) * 0.1
                result[w] = v
        found = sum(1 for w in vocab_words if w.lower() in glove_dict)
        print(f"[+] GloVe: {found}/{len(vocab_words)} vocab words matched ({found*100//max(len(vocab_words),1)}%)")
        return result

    # --- Download if needed ---
    glove_txt = os.path.join(os.path.dirname(os.path.abspath(__file__)), GLOVE_50D)
    if not os.path.exists(glove_txt):
        if not os.path.exists(GLOVE_ZIP):
            print(f"[*] Downloading GloVe-6B embeddings (~830MB zip)...")
            try:
                urllib.request.urlretrieve(GLOVE_URL, GLOVE_ZIP, reporthook=_progress)
                print()
            except Exception as e:
                print(f"\n[-] GloVe download failed: {e}")
                print("[!] Falling back to random vocab vectors.")
                return {}
        print(f"[*] Extracting {GLOVE_50D}...")
        try:
            with zipfile.ZipFile(GLOVE_ZIP, "r") as zf:
                zf.extract(GLOVE_50D, os.path.dirname(os.path.abspath(__file__)))
            print(f"[+] Extracted {GLOVE_50D}")
        except Exception as e:
            print(f"[-] Extraction failed: {e}")
            return {}

    # --- Parse the txt file ---
    print(f"[*] Parsing {GLOVE_50D}...")
    all_words  = []
    all_vecs   = []
    glove_dict = {}
    with open(glove_txt, "r", encoding="utf-8") as f:
        for line in f:
            parts = line.rstrip().split(" ")
            if len(parts) < 51:
                continue
            word = parts[0].lower()
            vec50 = np.array([float(x) for x in parts[1:51]], dtype=np.float32)
            # Pad to 512D
            full = np.zeros(M_MAX, dtype=np.float32)
            full[:50] = vec50
            glove_dict[word] = full
            all_words.append(word)
            all_vecs.append(full)

    # --- Save numpy cache for next time ---
    print(f"[*] Saving GloVe cache ({len(all_words)} words)...")
    np.savez_compressed(GLOVE_CACHE,
                        words=np.array(all_words),
                        vectors=np.array(all_vecs, dtype=np.float32))
    print(f"[+] GloVe cache saved to {GLOVE_CACHE}")

    # --- Clean up large files ---
    try:
        os.remove(GLOVE_ZIP)
        os.remove(glove_txt)
    except Exception:
        pass

    # --- Build result for this vocab ---
    result = {}
    rng = np.random.default_rng(42)
    for w in vocab_words:
        wl = w.lower()
        if wl in glove_dict:
            result[w] = glove_dict[wl]
        else:
            v = np.zeros(M_MAX)
            v[:50] = rng.standard_normal(50) * 0.1
            result[w] = v

    found = sum(1 for w in vocab_words if w.lower() in glove_dict)
    print(f"[+] GloVe: {found}/{len(vocab_words)} vocab words matched ({found*100//max(len(vocab_words),1)}%)")
    return result


if __name__ == "__main__":
    # Quick test
    test_words = ["water", "ocean", "river", "physics", "geometry", "prime",
                  "quantum", "routing", "mathematics", "language"]
    vecs = load_glove(test_words)
    if vecs:
        # Cosine similarity: water vs ocean should be high
        def cos(a, b):
            return float(np.dot(a, b) / (np.linalg.norm(a) * np.linalg.norm(b) + 1e-9))
        print(f"\nSemantic similarity test:")
        print(f"  water ↔ ocean:    {cos(vecs['water'], vecs['ocean']):.4f}  (expect ~0.85)")
        print(f"  water ↔ prime:    {cos(vecs['water'], vecs['prime']):.4f}  (expect ~0.10)")
        print(f"  physics ↔ quantum:{cos(vecs['physics'], vecs['quantum']):.4f}  (expect ~0.70)")
