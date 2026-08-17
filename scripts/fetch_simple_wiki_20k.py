#!/usr/bin/env python3
"""Page the Simple English Wikipedia rows endpoint to a larger corpus (#531).

Extends the 3000-article D3 sample to ~TARGET articles so a teacher observe can
pass the ~1-2M-record coverage knee. Same jsonl schema (id, url, title, text)
and dataset; deterministic row order. RESUMES: re-running continues from the
existing line count. Backs off hard on HTTP 429 (datasets-server rate limit).

Usage: python3 scripts/fetch_simple_wiki_20k.py [TARGET] [OUT]
"""
import json
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

BASE = (
    "https://datasets-server.huggingface.co/rows"
    "?dataset=wikimedia%2Fwikipedia&config=20231101.simple&split=train"
)
TARGET = int(sys.argv[1]) if len(sys.argv) > 1 else 18000
OUT = Path(sys.argv[2]) if len(sys.argv) > 2 else Path(
    ".uor-models/corpora/simple-wiki-20k/articles.jsonl"
)
OUT.parent.mkdir(parents=True, exist_ok=True)


def fetch(offset, length):
    url = f"{BASE}&offset={offset}&length={length}"
    for attempt in range(40):
        try:
            req = urllib.request.Request(url, headers={"User-Agent": "uor-r4-corpus-fetch"})
            with urllib.request.urlopen(req, timeout=90) as resp:
                return json.loads(resp.read())
        except urllib.error.HTTPError as err:
            wait = 60 if err.code == 429 else 5 * (attempt + 1)
            print(f"  HTTP {err.code} at offset {offset}, wait {wait}s (attempt {attempt})", flush=True)
            time.sleep(wait)
        except (urllib.error.URLError, TimeoutError, ValueError) as err:
            print(f"  {err} at offset {offset}, wait {5 * (attempt + 1)}s", flush=True)
            time.sleep(5 * (attempt + 1))
    raise SystemExit(f"failed at offset {offset} after 40 attempts")


def main():
    written = sum(1 for _ in open(OUT, encoding="utf-8")) if OUT.exists() else 0
    print(f"resuming from {written} existing articles", flush=True)
    offset = written
    with open(OUT, "a", encoding="utf-8") as handle:
        while written < TARGET:
            length = min(100, TARGET - written)
            data = fetch(offset, length)
            rows = data.get("rows", [])
            if not rows:
                print(f"no more rows at offset {offset}; stopping at {written}", flush=True)
                break
            for entry in rows:
                row = entry.get("row", {})
                record = {
                    "id": str(row.get("id", "")),
                    "url": row.get("url", ""),
                    "title": row.get("title", ""),
                    "text": row.get("text", ""),
                }
                handle.write(json.dumps(record, ensure_ascii=False) + "\n")
                written += 1
            handle.flush()
            offset += len(rows)
            print(f"  {written} articles (offset {offset})", flush=True)
            time.sleep(1.5)
    print(f"FETCH DONE: {written} articles -> {OUT}", flush=True)


if __name__ == "__main__":
    main()
