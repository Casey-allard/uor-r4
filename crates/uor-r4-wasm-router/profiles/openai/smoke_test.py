#!/usr/bin/env python3
"""#654 phase F — official OpenAI **Python** SDK smoke test against a live R4 server.

This drives the pinned wire surfaces (`POST /v1/chat/completions` non-streaming
and streaming, and `POST /v1/responses`) with the *official* `openai` package and
asserts the SDK round-trips: it builds a request our server accepts and parses the
response our server returns. It is intentionally NOT wired into CI — it needs a
running server with a compiled model loaded (a declined-by-all cascade has no text
to serve). It is the developer-run companion to the deterministic DTO/response
fixtures in `src/server.rs` (which prove the exact SDK-emitted request bytes
deserialize without a live server).

Usage:
    pip install openai
    python3 profiles/openai/smoke_test.py --base-url http://127.0.0.1:8080/v1 --model <compiled-model-id>

`--base-url`/`--model` default to the `UOR_OPENAI_BASE_URL` / `UOR_OPENAI_MODEL`
environment variables. The API key is a placeholder (the R4 server does not
authenticate the pinned profile). Exit code 0 = every surface round-tripped.
"""
from __future__ import annotations

import argparse
import os
import sys

try:
    from openai import OpenAI
except ImportError:  # pragma: no cover - operator guidance, not a test path
    sys.exit("openai package not installed; run `pip install openai` first")


def _check(label: str, ok: bool, detail: str) -> bool:
    print(f"[{'PASS' if ok else 'FAIL'}] {label}: {detail}")
    return ok


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--base-url",
        default=os.environ.get("UOR_OPENAI_BASE_URL", "http://127.0.0.1:8080/v1"),
    )
    parser.add_argument("--model", default=os.environ.get("UOR_OPENAI_MODEL", "uor-r4"))
    args = parser.parse_args()

    client = OpenAI(base_url=args.base_url, api_key="sk-uor-r4-smoke", max_retries=0)
    results: list[bool] = []

    # 1. Chat Completions, non-streaming.
    chat = client.chat.completions.create(
        model=args.model,
        messages=[{"role": "user", "content": "Say hi in one word."}],
    )
    choice = chat.choices[0]
    results.append(
        _check(
            "chat.completions (non-stream)",
            bool(choice.message.content) and choice.finish_reason in {"stop", "length"},
            f"content={choice.message.content!r} finish_reason={choice.finish_reason} "
            f"usage.total_tokens={chat.usage.total_tokens}",
        )
    )

    # 2. Chat Completions, streaming (phase D SSE).
    reconstructed = ""
    saw_role = False
    stream_finish = None
    stream = client.chat.completions.create(
        model=args.model,
        messages=[{"role": "user", "content": "Say hi in one word."}],
        stream=True,
    )
    for event in stream:
        delta = event.choices[0].delta
        if delta.role:
            saw_role = True
        if delta.content:
            reconstructed += delta.content
        if event.choices[0].finish_reason:
            stream_finish = event.choices[0].finish_reason
    results.append(
        _check(
            "chat.completions (stream)",
            saw_role and stream_finish in {"stop", "length"},
            f"reconstructed={reconstructed!r} finish_reason={stream_finish}",
        )
    )

    # 3. Responses.
    resp = client.responses.create(model=args.model, input="Say hi in one word.")
    results.append(
        _check(
            "responses",
            bool(resp.output_text) and resp.status in {"completed", "incomplete"},
            f"output_text={resp.output_text!r} status={resp.status} "
            f"usage.total_tokens={resp.usage.total_tokens}",
        )
    )

    ok = all(results)
    print(f"\n{'ALL SURFACES ROUND-TRIPPED' if ok else 'ONE OR MORE SURFACES FAILED'} "
          f"({sum(results)}/{len(results)})")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
