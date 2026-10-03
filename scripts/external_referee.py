#!/usr/bin/env python3
"""Measure SmolLM2 teacher cross-entropy outside the r4 pipeline.

This is the external referee for issue #300. It tokenizes the raw D3 article
text with Hugging Face Transformers, keeps the first ``sequence_length``
tokens of each held-out article, and scores the next-token labels directly
with the pinned checkpoint. No r4 corpus records, observation vectors, or
compiled artifacts are read.

Run with, for example:

    uv run --with torch --with transformers --with safetensors --with blake3 \
      python scripts/external_referee.py \
      --model .uor-models/sources/smollm2-135m-instruct \
      --corpus .uor-models/corpora/simple-wiki-20231101/articles.jsonl \
      --out /tmp/i300-external-referee.json
"""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path
from typing import Any

import torch
from transformers import AutoModelForCausalLM, AutoTokenizer


def d3_split(article_id: str) -> bool:
    try:
        import blake3  # type: ignore
    except ImportError as error:
        raise SystemExit(
            "The blake3 package is required to reproduce the D3 split; "
            "run through uv with --with blake3"
        ) from error
    return blake3.blake3(article_id.encode("utf-8")).digest()[0] % 5 == 0


def read_held_out(path: Path, tokenizer: Any, sequence_length: int) -> list[list[int]]:
    articles: list[list[int]] = []
    with path.open(encoding="utf-8") as handle:
        for line in handle:
            row = json.loads(line)
            if not d3_split(str(row["id"])):
                continue
            ids = tokenizer(
                row["text"], add_special_tokens=False, truncation=True,
                max_length=sequence_length, return_attention_mask=False,
            )["input_ids"]
            if len(ids) >= 2:
                articles.append(ids)
    return articles


def score_batches(
    model: Any, articles: list[list[int]], batch_size: int, pad_token_id: int
) -> tuple[float, int]:
    total_nats = 0.0
    total_tokens = 0
    model.eval()
    with torch.inference_mode():
        for start in range(0, len(articles), batch_size):
            batch = articles[start : start + batch_size]
            width = max(len(ids) for ids in batch)
            input_ids = torch.full(
                (len(batch), width), pad_token_id, dtype=torch.long
            )
            attention = torch.zeros((len(batch), width), dtype=torch.long)
            for row, ids in enumerate(batch):
                input_ids[row, : len(ids)] = torch.tensor(ids, dtype=torch.long)
                attention[row, : len(ids)] = 1
            logits = model(input_ids=input_ids, attention_mask=attention).logits
            # Position t predicts token t+1. The final padded position has no
            # target, and the attention mask excludes padded targets.
            log_probs = torch.log_softmax(logits[:, :-1, :].float(), dim=-1)
            targets = input_ids[:, 1:]
            valid = attention[:, 1:].bool()
            selected = log_probs.gather(-1, targets.unsqueeze(-1)).squeeze(-1)
            total_nats += float((-selected[valid]).sum())
            total_tokens += int(valid.sum())
            print(
                f"scored {min(start + batch_size, len(articles))}/{len(articles)} "
                f"articles, {total_tokens} target tokens",
                flush=True,
            )
    return total_nats, total_tokens


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--model", type=Path, required=True)
    parser.add_argument("--corpus", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--sequence-length", type=int, default=128)
    parser.add_argument("--batch-size", type=int, default=8)
    args = parser.parse_args()
    if args.sequence_length < 2 or args.batch_size < 1:
        raise SystemExit("sequence length must be >= 2 and batch size must be >= 1")

    tokenizer = AutoTokenizer.from_pretrained(args.model, local_files_only=True)
    if tokenizer.pad_token_id is None:
        tokenizer.pad_token = tokenizer.eos_token
    articles = read_held_out(args.corpus, tokenizer, args.sequence_length)
    if not articles:
        raise SystemExit("D3 held-out split contained no scoreable articles")

    model = AutoModelForCausalLM.from_pretrained(
        args.model, local_files_only=True, torch_dtype=torch.float32
    )
    model.to("cpu")
    total_nats, total_tokens = score_batches(
        model, articles, args.batch_size, tokenizer.pad_token_id
    )
    result = {
        "suite": "external_referee_issue_300",
        "model": str(args.model),
        "corpus": str(args.corpus),
        "split": "blake3(article id as utf-8)[0] % 5 == 0",
        "sequence_length": args.sequence_length,
        "held_out_articles": len(articles),
        "target_tokens": total_tokens,
        "bits_per_token": total_nats / math.log(2.0) / total_tokens,
        "method": "raw article text, no BOS or chat template, Hugging Face Transformers",
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
