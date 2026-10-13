#!/usr/bin/env python3
import argparse
import datetime
import json
import os
import time

import numpy as np


def fit_ridge(X: np.ndarray, Y: np.ndarray, l2: float) -> np.ndarray:
    d = X.shape[1]
    xtx = X.T @ X
    reg = l2 * np.eye(d, dtype=np.float64)
    xty = X.T @ Y
    W = np.linalg.solve(xtx + reg, xty)
    return W


def mse(y_hat: np.ndarray, y: np.ndarray) -> float:
    return float(np.mean((y_hat - y) ** 2))


def top1_acc(y_hat: np.ndarray, y: np.ndarray) -> float:
    if y_hat.shape[0] == 0:
        return 0.0
    p = np.argmax(y_hat, axis=1)
    t = np.argmax(y, axis=1)
    return float(np.mean(p == t))


def bytes_of(*arrs: np.ndarray) -> int:
    return int(sum(a.nbytes for a in arrs))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--input", type=str, default="data/wikitext2_proxy/wikitext2_proxy.npz")
    ap.add_argument("--l2", type=float, default=1e-2)
    ap.add_argument("--out_json", type=str, default="")
    args = ap.parse_args()

    z = np.load(args.input)
    x_train = z["x_train"].astype(np.float64)
    y_train = z["y_train"].astype(np.float64)
    x_val = z["x_val"].astype(np.float64)
    y_val = z["y_val"].astype(np.float64)
    x_test = z["x_test"].astype(np.float64)
    y_test = z["y_test"].astype(np.float64)

    t0 = time.perf_counter()
    W = fit_ridge(x_train, y_train, args.l2)
    t_train = time.perf_counter() - t0

    t1 = time.perf_counter()
    yv = x_val @ W
    yt = x_test @ W
    t_eval = time.perf_counter() - t1

    result = {
        "schema_version": "1.0",
        "task": "wikitext2_proxy_dense_baseline",
        "metrics": {
            "val_mse": mse(yv, y_val),
            "test_mse": mse(yt, y_test),
            "val_top1": top1_acc(yv, y_val),
            "test_top1": top1_acc(yt, y_test),
        },
        "timings_sec": {
            "train": float(t_train),
            "eval": float(t_eval),
            "total": float(t_train + t_eval),
        },
        "memory_bytes": {
            "weights": int(W.nbytes),
            "train_arrays": bytes_of(x_train, y_train),
            "eval_arrays": bytes_of(x_val, y_val, x_test, y_test),
        },
    }

    if args.out_json:
        out_json = args.out_json
    else:
        ts = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
        out_json = f"results/raw/dense_baseline_{ts}.json"

    os.makedirs(os.path.dirname(out_json) or ".", exist_ok=True)
    with open(out_json, "w", encoding="utf-8") as f:
        json.dump(result, f, indent=2, sort_keys=True)

    print(json.dumps(result, indent=2, sort_keys=True))
    print(f"Wrote baseline metrics to {out_json}")


if __name__ == "__main__":
    main()
