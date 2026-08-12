#!/usr/bin/env python3
import argparse
import hashlib
import json
import os
from typing import Tuple

import numpy as np


def contexts_targets(ids: np.ndarray, context_len: int, max_samples: int) -> Tuple[np.ndarray, np.ndarray]:
    if len(ids) <= context_len:
        return np.zeros((0, context_len), dtype=np.int32), np.zeros((0,), dtype=np.int32)
    n = len(ids) - context_len
    if max_samples > 0:
        n = min(n, max_samples)
    X = np.zeros((n, context_len), dtype=np.int32)
    y = np.zeros((n,), dtype=np.int32)
    for i in range(n):
        X[i] = ids[i:i + context_len]
        y[i] = ids[i + context_len]
    return X, y


def hashed_features(contexts: np.ndarray, d: int) -> np.ndarray:
    X = np.zeros((contexts.shape[0], d), dtype=np.float32)
    for i in range(contexts.shape[0]):
        row = contexts[i]
        for tid in row:
            idx = int((int(tid) * 2654435761) % d)
            sign = 1.0 if (int(tid) & 1) == 0 else -1.0
            X[i, idx] += sign
    n = np.linalg.norm(X, axis=1, keepdims=True)
    n = np.maximum(n, 1e-8)
    X /= n
    return X


def hashed_targets(targets: np.ndarray, dy: int) -> np.ndarray:
    Y = np.zeros((len(targets), dy), dtype=np.float32)
    for i, tid in enumerate(targets):
        idx = int((int(tid) * 11400714819323198485) % dy)
        Y[i, idx] = 1.0
    return Y


def array_sha256(arr: np.ndarray) -> str:
    h = hashlib.sha256()
    h.update(arr.tobytes())
    return h.hexdigest()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--input", type=str, default="data/wikitext2_proxy/wikitext2_tokens.npz")
    ap.add_argument("--output", type=str, default="data/wikitext2_proxy/wikitext2_proxy.npz")
    ap.add_argument("--meta_out", type=str, default="data/wikitext2_proxy/wikitext2_proxy_meta.json")
    ap.add_argument("--context_len", type=int, default=32)
    ap.add_argument("--d", type=int, default=128)
    ap.add_argument("--dy", type=int, default=256)
    ap.add_argument("--max_samples_per_split", type=int, default=50000)
    args = ap.parse_args()

    z = np.load(args.input)
    train_ids = z["train_ids"]
    valid_ids = z["valid_ids"]
    test_ids = z["test_ids"]

    xtr_tok, ytr_tok = contexts_targets(train_ids, args.context_len, args.max_samples_per_split)
    xva_tok, yva_tok = contexts_targets(valid_ids, args.context_len, args.max_samples_per_split)
    xte_tok, yte_tok = contexts_targets(test_ids, args.context_len, args.max_samples_per_split)

    xtr = hashed_features(xtr_tok, args.d)
    xva = hashed_features(xva_tok, args.d)
    xte = hashed_features(xte_tok, args.d)

    ytr = hashed_targets(ytr_tok, args.dy)
    yva = hashed_targets(yva_tok, args.dy)
    yte = hashed_targets(yte_tok, args.dy)

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
        "context_len": args.context_len,
        "d": args.d,
        "dy": args.dy,
        "n_train": int(xtr.shape[0]),
        "n_val": int(xva.shape[0]),
        "n_test": int(xte.shape[0]),
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

    print(f"Saved routing-compatible proxy tensors: {args.output}")


if __name__ == "__main__":
    main()
