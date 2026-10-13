#!/usr/bin/env python3
"""
generate_ppmi_proxy.py — Build the PPMI-SVD data file required by hopf_routing_demo.py
========================================================================================

Usage:
  python generate_ppmi_proxy.py

Requirements: numpy, scipy. Runtime: ~1–3 minutes on first run (downloads ~5MB).

What this does:
  1. Downloads WikiText-2 from the public source (Merity et al. 2016).
  2. Builds a PPMI co-occurrence word embedding (window=5, harmonic weighting).
  3. Truncated SVD to 100 dimensions.
  4. L2-normalizes each row vector.
  5. Saves x_train to data/ppmi_proxy.npz.

After running this script, execute:
  python hopf_routing_demo.py

References:
  WikiText-2: Merity et al. 2016 (https://arxiv.org/abs/1609.07843)
  Public source: https://s3.amazonaws.com/research.metamind.io/wikitext/wikitext-2-v1.zip
                 https://huggingface.co/datasets/Salesforce/wikitext/resolve/main/wikitext-2-v1.zip
"""

import io
import os
import sys
import urllib.request
import zipfile
from collections import Counter

import numpy as np
from scipy import sparse
from scipy.sparse.linalg import svds

# ---------------------------------------------------------------------------
# Configuration — must match hopf_routing_demo.py
# ---------------------------------------------------------------------------

VOCAB_SIZE = 10000
N_COMPONENTS = 100
WINDOW = 5
CONTEXT_LEN = 8
MAX_SAMPLES = 50000
OUT_PATH = os.path.join(os.path.dirname(__file__), "data", "ppmi_proxy.npz")

WIKITEXT2_URLS = [
    "https://s3.amazonaws.com/research.metamind.io/wikitext/wikitext-2-v1.zip",
    "https://huggingface.co/datasets/Salesforce/wikitext/resolve/main/wikitext-2-v1.zip",
]


# ---------------------------------------------------------------------------
# Download
# ---------------------------------------------------------------------------

def download_wikitext2() -> dict:
    """Download WT2 zip and return {split: text} dict."""
    for url in WIKITEXT2_URLS:
        print(f"Downloading WikiText-2 from {url} ...")
        try:
            with urllib.request.urlopen(url, timeout=60) as resp:
                data = resp.read()
            break
        except Exception as e:
            print(f"  Failed ({e}), trying next mirror ...")
    else:
        print("ERROR: could not download WikiText-2 from any mirror.")
        print("Please download manually from one of:")
        for u in WIKITEXT2_URLS:
            print(f"  {u}")
        sys.exit(1)

    splits = {}
    with zipfile.ZipFile(io.BytesIO(data)) as zf:
        for name in zf.namelist():
            if name.endswith(".tokens") or name.endswith(".txt"):
                key = None
                if "train" in name:
                    key = "train"
                elif "valid" in name or "val" in name:
                    key = "valid"
                elif "test" in name:
                    key = "test"
                if key:
                    splits[key] = zf.read(name).decode("utf-8", errors="ignore")
    print(f"  Downloaded. Splits: {list(splits.keys())}")
    return splits


# ---------------------------------------------------------------------------
# Tokenize and build vocab
# ---------------------------------------------------------------------------

def tokenize(text: str):
    return text.replace("\n", " <nl> ").split()


def build_vocab(tokens, vocab_size):
    counts = Counter(tokens)
    most_common = [w for w, _ in counts.most_common(vocab_size - 2)]
    idx_to_token = ["<pad>", "<unk>"] + most_common
    token_to_idx = {w: i for i, w in enumerate(idx_to_token)}
    return token_to_idx, idx_to_token


def encode(tokens, token_to_idx):
    unk = token_to_idx["<unk>"]
    return np.array([token_to_idx.get(t, unk) for t in tokens], dtype=np.int32)


# ---------------------------------------------------------------------------
# PPMI + SVD
# ---------------------------------------------------------------------------

def build_cooccurrence(ids, vocab_size, window=5):
    """Harmonic-weighted co-occurrence matrix."""
    rows, cols, vals = [], [], []
    n = len(ids)
    for i in range(n):
        w = int(ids[i])
        for j in range(max(0, i - window), min(n, i + window + 1)):
            if j == i:
                continue
            rows.append(w)
            cols.append(int(ids[j]))
            vals.append(1.0 / abs(i - j))
    M = sparse.coo_matrix((vals, (rows, cols)), shape=(vocab_size, vocab_size)).tocsr()
    return M


def ppmi_svd(cooc, n_components=100):
    """PPMI then truncated SVD; returns (vocab_size, n_components) float32."""
    M = cooc.astype(np.float64).tocoo()
    total = M.sum()
    row_sum = np.asarray(cooc.sum(axis=1)).ravel()
    col_sum = np.asarray(cooc.sum(axis=0)).ravel()

    pmi_vals = (
        np.log(M.data + 1e-12) + np.log(total + 1e-12)
        - np.log(row_sum[M.row] + 1e-12)
        - np.log(col_sum[M.col] + 1e-12)
    )
    ppmi_vals = np.maximum(0.0, pmi_vals)
    mask = ppmi_vals > 0.0
    ppmi = sparse.coo_matrix(
        (ppmi_vals[mask], (M.row[mask], M.col[mask])), shape=M.shape
    ).tocsr()

    k = min(n_components, min(ppmi.shape) - 1)
    print(f"  PPMI nnz={ppmi.nnz:,}. Computing SVD (k={k}) ...")
    U, S, _ = svds(ppmi, k=k)
    order = np.argsort(-S)
    emb = (U[:, order] * S[order]).astype(np.float32)
    return emb


# ---------------------------------------------------------------------------
# Build proxy data
# ---------------------------------------------------------------------------

def embed_and_sample(ids, emb, context_len, max_samples):
    """Mean-pool context windows → L2-normalize → return (n, d) float32."""
    n = min(len(ids) - context_len, max_samples)
    if n <= 0:
        return np.zeros((0, emb.shape[1]), dtype=np.float32)
    contexts = np.stack([ids[i:i + context_len] for i in range(n)])
    X = emb[contexts].mean(axis=1)
    norms = np.linalg.norm(X, axis=1, keepdims=True)
    norms = np.maximum(norms, 1e-8)
    return (X / norms).astype(np.float32)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    os.makedirs(os.path.dirname(OUT_PATH), exist_ok=True)

    # 1. Download
    splits = download_wikitext2()

    # 2. Vocab from train
    print("Building vocabulary ...")
    train_tokens = tokenize(splits.get("train", ""))
    token_to_idx, _ = build_vocab(train_tokens, VOCAB_SIZE)
    train_ids = encode(train_tokens, token_to_idx)
    print(f"  Train tokens: {len(train_tokens):,}  Vocab: {len(token_to_idx):,}")

    # 3. PPMI-SVD on train
    print("Building PPMI co-occurrence (window=5, harmonic weighting) ...")
    cooc = build_cooccurrence(train_ids, len(token_to_idx), window=WINDOW)
    emb = ppmi_svd(cooc, n_components=N_COMPONENTS)
    print(f"  Embedding shape: {emb.shape}")

    # 4. Build x_train proxy embeddings
    print("Building proxy embedding sequences ...")
    x_train = embed_and_sample(train_ids, emb, CONTEXT_LEN, MAX_SAMPLES)
    print(f"  x_train: {x_train.shape}")

    # 5. Save
    print(f"Saving to {OUT_PATH} ...")
    np.savez_compressed(OUT_PATH, x_train=x_train)
    size_mb = os.path.getsize(OUT_PATH) / 1e6
    print(f"  Saved ({size_mb:.1f} MB).")
    print()
    print("Done. Now run:  python hopf_routing_demo.py")


if __name__ == "__main__":
    main()
