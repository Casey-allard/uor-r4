#!/usr/bin/env python3
"""Re-materialize the D3 natural-partition corpus (issue #267).

Fetches the 3000-article Simple English Wikipedia sample per
`.uor-models/corpora/simple-wiki-20231101/manifest.json` and verifies the
result against the manifest's pinned totals. Upstream drift is EXPECTED
(the manifest's own recipe no longer reproduces the pinned bytes — see
issue #267); on mismatch this script says so loudly and exits nonzero
unless --allow-drift is passed. The authoritative path for the pinned
bytes is project-controlled storage keyed by corpus_cid (issue #267,
proposal 1); this script is the documented fallback.
"""
import argparse
import json
import sys
import time
import urllib.request
from pathlib import Path

BASE = (
    "https://datasets-server.huggingface.co/rows"
    "?dataset=wikimedia%2Fwikipedia&config=20231101.simple&split=train"
)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--manifest",
        default=".uor-models/corpora/simple-wiki-20231101/manifest.json",
        help="corpus manifest to verify against",
    )
    parser.add_argument(
        "--out",
        default=".uor-models/corpora/simple-wiki-20231101/articles.jsonl",
        help="output path for the fetched corpus",
    )
    parser.add_argument(
        "--allow-drift",
        action="store_true",
        help="exit 0 even when totals do not match the manifest (drift is recorded either way)",
    )
    args = parser.parse_args()

    manifest = json.loads(Path(args.manifest).read_text())
    want_count = manifest["article_count"]
    want_bytes = manifest["text_bytes"]

    rows = []
    for offset in range(0, want_count, 100):
        url = f"{BASE}&offset={offset}&length=100"
        for attempt in range(5):
            try:
                with urllib.request.urlopen(url, timeout=60) as response:
                    payload = json.load(response)
                rows.extend(payload["rows"])
                break
            except Exception as error:  # noqa: BLE001 — retry then fail loudly
                if attempt == 4:
                    print(f"FETCH FAILED at offset {offset}: {error}", file=sys.stderr)
                    return 2
                time.sleep(3 * (attempt + 1))
        print(f"fetched {len(rows)}/{want_count}", flush=True)

    if len(rows) != want_count:
        print(f"expected {want_count} rows, got {len(rows)}", file=sys.stderr)
        return 2

    text_bytes = 0
    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with out_path.open("w", encoding="utf-8") as handle:
        for item in rows:
            row = item["row"]
            record = {
                "id": row["id"],
                "url": row["url"],
                "title": row["title"],
                "text": row["text"],
            }
            text_bytes += len(row["text"].encode("utf-8"))
            handle.write(json.dumps(record, ensure_ascii=False) + "\n")

    print(
        f"wrote {len(rows)} articles, text_bytes={text_bytes} "
        f"(manifest pins {want_count} / {want_bytes})"
    )
    if text_bytes == want_bytes:
        print("MATCH: totals reproduce the manifest")
        return 0
    print(
        "DRIFT: upstream content differs from the pinned corpus "
        "(expected per issue #267) — D3-anchored numbers from this fetch "
        "are not directly comparable to pinned baselines"
    )
    return 0 if args.allow_drift else 1


if __name__ == "__main__":
    sys.exit(main())
