#!/usr/bin/env python3
"""
tasks/ppmi_proxy.py — Build a PPMI-SVD semantic proxy data file.

Uses the PTB corpus (data/lm_proxy/raw/ptb/ptb.train.txt) to build
PPMI co-occurrence word embeddings (no external download needed), then
constructs the wikitext2-style proxy .npz that router_proxy_eval.py
accepts:

  x_{train,val,test}  shape (n, n_components)   L2-normalised mean-embed
  y_{train,val,test}  shape (n, dy)              one-hot hashed target

Mathematical basis:
  PPMI[w,c] = max(0, log(P(w,c) / (P(w) · P(c))))
  Word vectors = left singular vectors of PPMI, scaled by singular values.
  Words with similar distributional contexts cluster together, exactly the
  semantic structure that hash embeddings lack by construction.
"""

import argparse
import hashlib
import json
import os
from collections import Counter
from typing import List, Tuple

import numpy as np
from scipy import sparse
from scipy.sparse.linalg import svds


# ── vocabulary (mirrors prepare_wikitext2.py logic exactly) ──────────────────

def tokenize(text: str) -> List[str]:
    return text.replace("\n", " <nl> ").split()


def build_vocab(tokens: List[str], vocab_size: int = 10002) -> Tuple[dict, list]:
    counts = Counter(tokens)
    most_common = [w for w, _ in counts.most_common(vocab_size - 2)]
    idx_to_token = ["<pad>", "<unk>"] + most_common
    token_to_idx = {w: i for i, w in enumerate(idx_to_token)}
    return token_to_idx, idx_to_token


def encode(tokens: List[str], token_to_idx: dict) -> np.ndarray:
    unk = token_to_idx["<unk>"]
    return np.array([token_to_idx.get(t, unk) for t in tokens], dtype=np.int32)


# ── PPMI + SVD embeddings ─────────────────────────────────────────────────────

def build_cooccurrence(ids: np.ndarray, vocab_size: int, window: int = 5) -> sparse.csr_matrix:
    """Build harmonic-weighted co-occurrence matrix from token id sequence."""
    row_list, col_list, data_list = [], [], []
    n = len(ids)
    for i in range(n):
        w = int(ids[i])
        lo = max(0, i - window)
        hi = min(n, i + window + 1)
        for j in range(lo, hi):
            if j == i:
                continue
            c = int(ids[j])
            row_list.append(w)
            col_list.append(c)
            data_list.append(1.0 / abs(i - j))  # harmonic weighting
    M = sparse.coo_matrix(
        (data_list, (row_list, col_list)),
        shape=(vocab_size, vocab_size),
        dtype=np.float64,
    )
    # Sum duplicate entries (same (w,c) at different distances)
    M = M.tocsr()
    return M


def ppmi_svd(cooc: sparse.csr_matrix, n_components: int = 100) -> np.ndarray:
    """Compute PPMI then truncated SVD; returns (vocab_size, n_components)."""
    M = cooc.astype(np.float64).tocoo()
    total = M.sum()
    row_sum = np.asarray(cooc.sum(axis=1)).ravel()
    col_sum = np.asarray(cooc.sum(axis=0)).ravel()

    # PMI = log(P(w,c) / P(w)P(c)) = log(count * total / (row_sum * col_sum))
    pmi_vals = (
        np.log(M.data + 1e-12)
        + np.log(total + 1e-12)
        - np.log(row_sum[M.row] + 1e-12)
        - np.log(col_sum[M.col] + 1e-12)
    )
    ppmi_vals = np.maximum(0.0, pmi_vals)

    mask = ppmi_vals > 0.0
    ppmi = sparse.coo_matrix(
        (ppmi_vals[mask], (M.row[mask], M.col[mask])),
        shape=M.shape,
    ).tocsr()

    k = min(n_components, min(ppmi.shape) - 1)
    print(f"  PPMI nnz={ppmi.nnz}, computing SVD k={k} …")
    U, S, _Vt = svds(ppmi, k=k)

    # Sort by descending singular value
    order = np.argsort(-S)
    U = U[:, order]
    S = S[order]

    emb = (U * S[np.newaxis, :]).astype(np.float32)
    return emb


# ── proxy construction ────────────────────────────────────────────────────────

def contexts_targets(
    ids: np.ndarray, context_len: int, max_samples: int
) -> Tuple[np.ndarray, np.ndarray]:
    if len(ids) <= context_len:
        return (
            np.zeros((0, context_len), dtype=np.int32),
            np.zeros((0,), dtype=np.int32),
        )
    n = len(ids) - context_len
    if max_samples > 0:
        n = min(n, max_samples)
    X = np.zeros((n, context_len), dtype=np.int32)
    y = np.zeros((n,), dtype=np.int32)
    for i in range(n):
        X[i] = ids[i : i + context_len]
        y[i] = ids[i + context_len]
    return X, y


def embed_contexts(contexts: np.ndarray, emb: np.ndarray) -> np.ndarray:
    """Vectorised mean-pool over context tokens then L2-normalise."""
    # contexts: (n, ctx_len) int32; emb: (vocab_size, d) float32
    vecs = emb[contexts]       # (n, ctx_len, d)
    X = vecs.mean(axis=1)      # (n, d)
    norms = np.linalg.norm(X, axis=1, keepdims=True)
    norms = np.maximum(norms, 1e-8)
    return (X / norms).astype(np.float32)


def hashed_targets(targets: np.ndarray, dy: int) -> np.ndarray:
    """Same target encoding as wikitext2_proxy.py for fair comparison."""
    Y = np.zeros((len(targets), dy), dtype=np.float32)
    for i, tid in enumerate(targets):
        idx = int((int(tid) * 11400714819323198485) % dy)
        Y[i, idx] = 1.0
    return Y


def array_sha256(arr: np.ndarray) -> str:
    h = hashlib.sha256()
    h.update(arr.tobytes())
    return h.hexdigest()


# ── pre-screen dim search ─────────────────────────────────────────────────────

def find_best_phase4_dims(x: np.ndarray, top_k: int = 8) -> Tuple[int, int, int, int]:
    """
    Greedy search for the 4-tuple (i,j,k,l) of distinct dims whose absolute
    pairwise sample correlation is highest overall.

    Strategy:
      1. Compute full d×d correlation matrix on the first min(n,5000) samples.
      2. Find pair (i1, j1) = argmax |corr| with i1 ≠ j1.
      3. Remove i1, j1; find pair (i2, j2) = argmax |corr| among remaining dims.
      4. Return (i1, j1, i2, j2).

    Returns (i, j, k, l) as phase4_dims.
    """
    n_screen = min(x.shape[0], 5000)
    x_screen = x[:n_screen]
    # Column-centre
    x_screen = x_screen - x_screen.mean(axis=0)
    # Correlation matrix (clip std to avoid 0-div)
    std = x_screen.std(axis=0)
    std = np.where(std < 1e-8, 1e-8, std)
    x_norm = x_screen / std
    corr = (x_norm.T @ x_norm) / n_screen  # (d, d)
    d = corr.shape[0]

    # Zero out diagonal
    np.fill_diagonal(corr, 0.0)
    abs_corr = np.abs(corr)

    # First pair
    flat_idx = np.argmax(abs_corr)
    i1, j1 = divmod(int(flat_idx), d)

    # Second pair: exclude i1, j1
    abs_corr2 = abs_corr.copy()
    abs_corr2[i1, :] = 0.0
    abs_corr2[:, i1] = 0.0
    abs_corr2[j1, :] = 0.0
    abs_corr2[:, j1] = 0.0

    flat_idx2 = np.argmax(abs_corr2)
    i2, j2 = divmod(int(flat_idx2), d)

    print(f"  pre-screen best dims: ({i1},{j1},{i2},{j2})")
    print(f"    pair1 |corr|={abs_corr[i1,j1]:.4f}  pair2 |corr|={abs_corr[i2,j2]:.4f}")
    return i1, j1, i2, j2


# ── main ──────────────────────────────────────────────────────────────────────

def main() -> None:
    ap = argparse.ArgumentParser(
        description="Build PPMI-SVD semantic proxy .npz for router_proxy_eval.py"
    )
    ap.add_argument(
        "--ptb_train",
        type=str,
        default="data/lm_proxy/raw/ptb/ptb.train.txt",
    )
    ap.add_argument(
        "--tokens_input",
        type=str,
        default="data/wikitext2_proxy/wikitext2_tokens.npz",
    )
    ap.add_argument(
        "--output",
        type=str,
        default="data/wikitext2_proxy/ppmi_proxy.npz",
    )
    ap.add_argument(
        "--meta_out",
        type=str,
        default="data/wikitext2_proxy/ppmi_proxy_meta.json",
    )
    ap.add_argument(
        "--emb_out",
        type=str,
        default="data/wikitext2_proxy/ppmi_emb.npz",
    )
    ap.add_argument("--vocab_size", type=int, default=10002)
    ap.add_argument("--window", type=int, default=5)
    ap.add_argument("--n_components", type=int, default=100)
    ap.add_argument("--context_len", type=int, default=32)
    ap.add_argument("--dy", type=int, default=256)
    ap.add_argument("--max_samples_per_split", type=int, default=50000)
    args = ap.parse_args()

    # ── 1. Rebuild PTB vocabulary (mirrors prepare_wikitext2.py) ─────────────
    print("Reading PTB training text …")
    with open(args.ptb_train, encoding="utf-8") as f:
        text = f.read()
    train_tokens = tokenize(text)
    token_to_idx, idx_to_token = build_vocab(train_tokens, args.vocab_size)
    print(f"  vocab_size={len(idx_to_token)}, tokens={len(train_tokens)}")

    # ── 2. Encode PTB training text ───────────────────────────────────────────
    train_ids_ptb = encode(train_tokens, token_to_idx)

    # ── 3. Build co-occurrence matrix ─────────────────────────────────────────
    print(f"Building co-occurrence (window={args.window}, tokens={len(train_ids_ptb)}) …")
    cooc = build_cooccurrence(train_ids_ptb, args.vocab_size, window=args.window)
    print(f"  co-occurrence nnz={cooc.nnz}")

    # ── 4. PPMI + SVD embeddings ──────────────────────────────────────────────
    print(f"Computing PPMI + SVD (n_components={args.n_components}) …")
    emb = ppmi_svd(cooc, n_components=args.n_components)
    print(f"  embedding shape={emb.shape}")

    os.makedirs(os.path.dirname(args.emb_out) or ".", exist_ok=True)
    np.savez_compressed(
        args.emb_out,
        emb=emb,
        idx_to_token=np.array(idx_to_token, dtype=object),
    )
    print(f"  saved embedding matrix: {args.emb_out}")

    # ── 5. Context windows from wikitext2_tokens.npz ──────────────────────────
    print("Building context windows …")
    z = np.load(args.tokens_input)
    train_ids = z["train_ids"]
    valid_ids = z["valid_ids"]
    test_ids = z["test_ids"]

    xtr_tok, ytr_tok = contexts_targets(train_ids, args.context_len, args.max_samples_per_split)
    xva_tok, yva_tok = contexts_targets(valid_ids, args.context_len, args.max_samples_per_split)
    xte_tok, yte_tok = contexts_targets(test_ids, args.context_len, args.max_samples_per_split)
    print(f"  train={xtr_tok.shape[0]}, val={xva_tok.shape[0]}, test={xte_tok.shape[0]}")

    # ── 6. Embed contexts ─────────────────────────────────────────────────────
    print("Embedding contexts (mean-pool + L2-normalise) …")
    xtr = embed_contexts(xtr_tok, emb)
    xva = embed_contexts(xva_tok, emb)
    xte = embed_contexts(xte_tok, emb)
    ytr = hashed_targets(ytr_tok, args.dy)
    yva = hashed_targets(yva_tok, args.dy)
    yte = hashed_targets(yte_tok, args.dy)

    # ── 7. Pre-screen: find best phase4_dims ─────────────────────────────────
    print("Running pre-screen dim search on x_train …")
    i1, j1, i2, j2 = find_best_phase4_dims(xtr)
    phase4_dims_str = f"{i1},{j1},{i2},{j2}"

    # ── 8. Save proxy .npz ───────────────────────────────────────────────────
    os.makedirs(os.path.dirname(args.output) or ".", exist_ok=True)
    np.savez_compressed(
        args.output,
        x_train=xtr,
        y_train=ytr,
        x_val=xva,
        y_val=yva,
        x_test=xte,
        y_test=yte,
    )

    meta = {
        "embedding": "ppmi_svd",
        "window": args.window,
        "n_components": args.n_components,
        "context_len": args.context_len,
        "d": int(args.n_components),
        "dy": int(args.dy),
        "n_train": int(xtr.shape[0]),
        "n_val": int(xva.shape[0]),
        "n_test": int(xte.shape[0]),
        "pre_screen_phase4_dims": phase4_dims_str,
        "sha256": {
            "x_train": array_sha256(xtr),
            "x_val": array_sha256(xva),
            "x_test": array_sha256(xte),
            "y_train": array_sha256(ytr),
            "y_val": array_sha256(yva),
            "y_test": array_sha256(yte),
        },
    }
    with open(args.meta_out, "w", encoding="utf-8") as f:
        json.dump(meta, f, indent=2, sort_keys=True)

    print(f"\nDone.")
    print(f"  proxy:          {args.output}")
    print(f"  meta:           {args.meta_out}")
    print(f"  embedding:      {args.emb_out}")
    print(f"  x_train shape:  {xtr.shape}")
    print(f"  y_train shape:  {ytr.shape}")
    print(f"  phase4_dims:    {phase4_dims_str}")
    print(f"\n__PHASE4_DIMS__ {phase4_dims_str}")


if __name__ == "__main__":
    main()
