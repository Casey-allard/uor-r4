#!/usr/bin/env python3
"""Download precalculated zeta-zero datasets from public sources.

Sources:
- LMFDB download endpoint (JSON payload on second line)
- Andrew Odlyzko zero tables (plain text / gzipped)
"""

from __future__ import annotations

import argparse
import gzip
import io
import json
import os
import re
import urllib.request
from datetime import datetime, timezone
from typing import List, Tuple

URLS = {
    "lmfdb": "https://www.lmfdb.org/L/download_zeros/1-1-1.1-r0-0-0",
    "odlyzko_100k": "https://www-users.cse.umn.edu/~odlyzko/zeta_tables/zeros1",
    "odlyzko_2m_gz": "https://www-users.cse.umn.edu/~odlyzko/zeta_tables/zeros6.gz",
}


def fetch_bytes(url: str, timeout: int = 60) -> bytes:
    req = urllib.request.Request(url, headers={"User-Agent": "codex-prime-research/1.0"})
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return resp.read()


def parse_lmfdb_payload(raw: bytes) -> List[float]:
    text = raw.decode("utf-8", errors="replace")
    lines = [ln.strip() for ln in text.splitlines() if ln.strip()]
    for ln in lines:
        if ln.startswith("{"):
            payload = json.loads(ln)
            if "positive_zeros" in payload and isinstance(payload["positive_zeros"], list):
                return [float(x) for x in payload["positive_zeros"]]
            if "zeros" in payload and isinstance(payload["zeros"], list):
                return [float(x) for x in payload["zeros"]]
    raise ValueError("could not parse LMFDB payload")


def parse_float_list_from_text(text: str) -> List[float]:
    vals = []
    for tok in re.findall(r"[+-]?(?:\d+\.\d+|\d+)", text):
        v = float(tok)
        if v > 0.0:
            vals.append(v)
    vals.sort()

    # Odlyzko files contain an index column; keep distinct strictly increasing values.
    out = []
    prev = -1.0
    for v in vals:
        if v > prev and (v > 10.0):
            out.append(v)
            prev = v
    return out


def parse_odlyzko_payload(raw: bytes, gzipped: bool) -> List[float]:
    if gzipped:
        raw = gzip.decompress(raw)
    text = raw.decode("utf-8", errors="replace")
    return parse_float_list_from_text(text)


def fetch_dataset(source: str) -> Tuple[str, List[float]]:
    if source not in URLS:
        raise ValueError(f"unsupported source: {source}")

    url = URLS[source]
    raw = fetch_bytes(url)
    if source == "lmfdb":
        zeros = parse_lmfdb_payload(raw)
    elif source == "odlyzko_100k":
        zeros = parse_odlyzko_payload(raw, gzipped=False)
    elif source == "odlyzko_2m_gz":
        zeros = parse_odlyzko_payload(raw, gzipped=True)
    else:
        raise ValueError(f"unsupported source: {source}")
    return url, zeros


def main() -> None:
    parser = argparse.ArgumentParser(description="Fetch precalculated zeta-zero datasets")
    parser.add_argument("--source", type=str, default="lmfdb", choices=sorted(URLS.keys()))
    parser.add_argument("--limit", type=int, default=20000)
    parser.add_argument("--output", type=str, default="")
    args = parser.parse_args()

    url, zeros = fetch_dataset(args.source)
    if args.limit > 0:
        zeros = zeros[: args.limit]

    payload = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "source": args.source,
        "source_url": url,
        "count": len(zeros),
        "zeros": zeros,
    }

    out_path = args.output or f"research/data/zeta_zeros_{args.source}.json"
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(payload, f, indent=2)

    print(f"wrote: {out_path}")
    print(f"count: {len(zeros)}")
    if zeros:
        print(f"first: {zeros[0]:.6f}")
        print(f"last: {zeros[-1]:.6f}")


if __name__ == "__main__":
    main()
