#!/usr/bin/env python3
import argparse
import collections
import json
import os
import urllib.request
import zipfile
from typing import Dict, List, Tuple

import numpy as np

WIKITEXT2_URLS = [
    "https://s3.amazonaws.com/research.metamind.io/wikitext/wikitext-2-v1.zip",
    "https://huggingface.co/datasets/Salesforce/wikitext/resolve/main/wikitext-2-v1.zip",
]

PTB_URLS = {
    "train": "https://raw.githubusercontent.com/wojzaremba/lstm/master/data/ptb.train.txt",
    "valid": "https://raw.githubusercontent.com/wojzaremba/lstm/master/data/ptb.valid.txt",
    "test": "https://raw.githubusercontent.com/wojzaremba/lstm/master/data/ptb.test.txt",
}

TINY_SHAKESPEARE_URL = (
    "https://raw.githubusercontent.com/karpathy/char-rnn/master/data/tinyshakespeare/input.txt"
)


def tokenize(text: str):
    return text.replace("\n", " <nl> ").split()


def read_tokens(path: str):
    with open(path, "r", encoding="utf-8", errors="ignore") as f:
        return tokenize(f.read())


def write_text(path: str, text: str):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)


def download(url: str, path: str) -> bool:
    try:
        urllib.request.urlretrieve(url, path)
        return True
    except Exception:
        return False


def fetch_text(url: str) -> str:
    with urllib.request.urlopen(url, timeout=30) as r:
        return r.read().decode("utf-8", errors="ignore")


def ensure_wikitext2(root: str) -> Tuple[str, str, str, bool, str]:
    extracted = os.path.join(root, "wikitext2", "wikitext-2")
    train_path = os.path.join(extracted, "wiki.train.tokens")
    valid_path = os.path.join(extracted, "wiki.valid.tokens")
    test_path = os.path.join(extracted, "wiki.test.tokens")

    if all(os.path.exists(p) for p in [train_path, valid_path, test_path]):
        return train_path, valid_path, test_path, True, "existing_files"

    os.makedirs(os.path.join(root, "wikitext2"), exist_ok=True)
    zip_path = os.path.join(root, "wikitext2", "wikitext-2-v1.zip")

    ok = False
    used = ""
    for u in WIKITEXT2_URLS:
        if download(u, zip_path):
            ok = True
            used = u
            break

    if not ok:
        return train_path, valid_path, test_path, False, "download_failed"

    try:
        with zipfile.ZipFile(zip_path, "r") as zf:
            zf.extractall(os.path.join(root, "wikitext2"))
    except Exception:
        return train_path, valid_path, test_path, False, "zip_extract_failed"

    if not all(os.path.exists(p) for p in [train_path, valid_path, test_path]):
        return train_path, valid_path, test_path, False, "missing_extracted_files"

    return train_path, valid_path, test_path, True, used


def ensure_ptb(root: str) -> Tuple[str, str, str, bool, str]:
    base = os.path.join(root, "ptb")
    train_path = os.path.join(base, "ptb.train.txt")
    valid_path = os.path.join(base, "ptb.valid.txt")
    test_path = os.path.join(base, "ptb.test.txt")

    if all(os.path.exists(p) for p in [train_path, valid_path, test_path]):
        return train_path, valid_path, test_path, True, "existing_files"

    os.makedirs(base, exist_ok=True)
    for split, url in PTB_URLS.items():
        p = os.path.join(base, f"ptb.{split}.txt")
        if not download(url, p):
            return train_path, valid_path, test_path, False, f"download_failed:{split}"

    return train_path, valid_path, test_path, True, "ptb_github_raw"


def ensure_tinyshakespeare(root: str) -> Tuple[str, str, str, bool, str]:
    base = os.path.join(root, "tinyshakespeare")
    src_path = os.path.join(base, "input.txt")

    if not os.path.exists(src_path):
        os.makedirs(base, exist_ok=True)
        try:
            text = fetch_text(TINY_SHAKESPEARE_URL)
            write_text(src_path, text)
            source = TINY_SHAKESPEARE_URL
        except Exception:
            return "", "", "", False, "download_failed"
    else:
        source = "existing_files"

    with open(src_path, "r", encoding="utf-8", errors="ignore") as f:
        lines = f.readlines()

    n = len(lines)
    n_train = max(1, int(0.8 * n))
    n_valid = max(1, int(0.1 * n))
    n_test = max(1, n - n_train - n_valid)

    train_lines = lines[:n_train]
    valid_lines = lines[n_train:n_train + n_valid]
    test_lines = lines[n_train + n_valid:n_train + n_valid + n_test]

    train_path = os.path.join(base, "tiny.train.txt")
    valid_path = os.path.join(base, "tiny.valid.txt")
    test_path = os.path.join(base, "tiny.test.txt")

    write_text(train_path, "".join(train_lines))
    write_text(valid_path, "".join(valid_lines))
    write_text(test_path, "".join(test_lines))

    return train_path, valid_path, test_path, True, source


def ensure_synthetic(root: str) -> Tuple[str, str, str, bool, str]:
    base = os.path.join(root, "synthetic")
    os.makedirs(base, exist_ok=True)
    tiny = (
        "The brain routes signals through sparse pathways and local neighborhoods. "
        "Geometry can compress routing choices while preserving useful structure. "
        "This fallback corpus exists to keep the benchmark pipeline runnable offline. "
    )
    train_path = os.path.join(base, "synthetic.train.txt")
    valid_path = os.path.join(base, "synthetic.valid.txt")
    test_path = os.path.join(base, "synthetic.test.txt")
    write_text(train_path, (tiny + "\n") * 800)
    write_text(valid_path, (tiny + "\n") * 120)
    write_text(test_path, (tiny + "\n") * 120)
    return train_path, valid_path, test_path, True, "synthetic_fallback"


def build_vocab(tokens, vocab_size: int):
    cnt = collections.Counter(tokens)
    most = [w for (w, _c) in cnt.most_common(max(0, vocab_size - 2))]
    idx_to_token = ["<pad>", "<unk>"] + most
    token_to_idx = {t: i for i, t in enumerate(idx_to_token)}
    return token_to_idx, idx_to_token


def encode(tokens, token_to_idx):
    unk = token_to_idx.get("<unk>", 1)
    return np.array([token_to_idx.get(t, unk) for t in tokens], dtype=np.int32)


def pick_source(dataset: str, root: str) -> Tuple[str, str, str, Dict[str, str]]:
    attempts: List[Dict[str, str]] = []

    def record(kind: str, status: bool, detail: str):
        attempts.append({"kind": kind, "status": "ok" if status else "fail", "detail": detail})

    if dataset in ("auto", "wikitext2"):
        tr, va, te, ok, detail = ensure_wikitext2(root)
        record("wikitext2", ok, detail)
        if ok:
            return tr, va, te, {"dataset": "wikitext2", "source_detail": detail, "attempts": attempts}
        if dataset == "wikitext2":
            tr, va, te, ok2, detail2 = ensure_ptb(root)
            record("ptb", ok2, detail2)
            if ok2:
                return tr, va, te, {"dataset": "ptb", "source_detail": detail2, "attempts": attempts}

    if dataset in ("auto", "ptb"):
        tr, va, te, ok, detail = ensure_ptb(root)
        record("ptb", ok, detail)
        if ok:
            return tr, va, te, {"dataset": "ptb", "source_detail": detail, "attempts": attempts}
        if dataset == "ptb":
            tr, va, te, ok2, detail2 = ensure_tinyshakespeare(root)
            record("tinyshakespeare", ok2, detail2)
            if ok2:
                return tr, va, te, {"dataset": "tinyshakespeare", "source_detail": detail2, "attempts": attempts}

    if dataset in ("auto", "tinyshakespeare"):
        tr, va, te, ok, detail = ensure_tinyshakespeare(root)
        record("tinyshakespeare", ok, detail)
        if ok:
            return tr, va, te, {"dataset": "tinyshakespeare", "source_detail": detail, "attempts": attempts}

    tr, va, te, _ok, detail = ensure_synthetic(root)
    record("synthetic", True, detail)
    return tr, va, te, {"dataset": "synthetic", "source_detail": detail, "attempts": attempts}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--data_dir", type=str, default="data/lm_proxy/raw")
    ap.add_argument("--output", type=str, default="data/wikitext2_proxy/wikitext2_tokens.npz")
    ap.add_argument("--meta_out", type=str, default="data/wikitext2_proxy/wikitext2_meta.json")
    ap.add_argument("--dataset", type=str, default="auto", choices=["auto", "wikitext2", "ptb", "tinyshakespeare"])
    ap.add_argument("--vocab_size", type=int, default=30000)
    args = ap.parse_args()

    train_path, valid_path, test_path, source_meta = pick_source(args.dataset, args.data_dir)

    train_tokens = read_tokens(train_path)
    valid_tokens = read_tokens(valid_path)
    test_tokens = read_tokens(test_path)

    token_to_idx, idx_to_token = build_vocab(train_tokens, args.vocab_size)

    train_ids = encode(train_tokens, token_to_idx)
    valid_ids = encode(valid_tokens, token_to_idx)
    test_ids = encode(test_tokens, token_to_idx)

    os.makedirs(os.path.dirname(args.output) or ".", exist_ok=True)
    np.savez_compressed(
        args.output,
        train_ids=train_ids,
        valid_ids=valid_ids,
        test_ids=test_ids,
    )

    meta = {
        "dataset_used": source_meta["dataset"],
        "source_detail": source_meta["source_detail"],
        "attempts": source_meta["attempts"],
        "vocab_size": len(idx_to_token),
        "train_tokens": int(len(train_ids)),
        "valid_tokens": int(len(valid_ids)),
        "test_tokens": int(len(test_ids)),
        "source": {
            "train": train_path,
            "valid": valid_path,
            "test": test_path,
        },
    }
    with open(args.meta_out, "w", encoding="utf-8") as f:
        json.dump(meta, f, indent=2, sort_keys=True)

    print(f"Prepared LM token cache: {args.output}")
    print(json.dumps({"dataset_used": meta["dataset_used"], "source_detail": meta["source_detail"]}))


if __name__ == "__main__":
    main()
