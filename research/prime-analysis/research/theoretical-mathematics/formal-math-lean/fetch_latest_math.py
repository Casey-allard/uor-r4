#!/usr/bin/env python3
"""Fetch recent arXiv entries for Riemann/prime-related queries."""

from __future__ import annotations

import argparse
import json
import os
import time
import urllib.parse
import urllib.error
import urllib.request
import xml.etree.ElementTree as ET
from datetime import datetime, timezone
from typing import Dict, List

ARXIV_API = "https://export.arxiv.org/api/query"


def fetch_arxiv(
    query: str,
    max_results: int,
    max_retries: int = 5,
    retry_backoff_sec: float = 3.0,
) -> List[Dict[str, str]]:
    params = {
        "search_query": query,
        "sortBy": "submittedDate",
        "sortOrder": "descending",
        "max_results": str(max_results),
    }
    url = ARXIV_API + "?" + urllib.parse.urlencode(params)
    req = urllib.request.Request(url, headers={"User-Agent": "codex-prime-research/1.0"})
    xml_data = b""
    for attempt in range(1, max_retries + 1):
        try:
            with urllib.request.urlopen(req, timeout=60) as resp:
                xml_data = resp.read()
            break
        except urllib.error.HTTPError as exc:
            if exc.code != 429 or attempt == max_retries:
                raise
            sleep_s = retry_backoff_sec * attempt
            time.sleep(sleep_s)

    ns = {
        "atom": "http://www.w3.org/2005/Atom",
        "arxiv": "http://arxiv.org/schemas/atom",
    }

    root = ET.fromstring(xml_data)
    entries: List[Dict[str, str]] = []
    for e in root.findall("atom:entry", ns):
        title = (e.findtext("atom:title", default="", namespaces=ns) or "").strip().replace("\n", " ")
        summary = (e.findtext("atom:summary", default="", namespaces=ns) or "").strip().replace("\n", " ")
        published = (e.findtext("atom:published", default="", namespaces=ns) or "").strip()
        updated = (e.findtext("atom:updated", default="", namespaces=ns) or "").strip()
        link = ""
        for l in e.findall("atom:link", ns):
            if l.attrib.get("rel") == "alternate":
                link = l.attrib.get("href", "")
                break
        entries.append(
            {
                "title": title,
                "summary": summary,
                "published": published,
                "updated": updated,
                "url": link,
            }
        )
    return entries


def main() -> None:
    parser = argparse.ArgumentParser(description="Fetch latest arXiv papers for prime/zeta topics")
    parser.add_argument(
        "--queries",
        type=str,
        default='cat:math.NT AND all:"riemann zeta",cat:math.NT AND all:"zeta zero",cat:math.NT AND all:"prime gaps",cat:math.NT AND all:"L-function zeros"',
        help="comma-separated raw arXiv search queries",
    )
    parser.add_argument("--max-results", type=int, default=8)
    parser.add_argument("--max-retries", type=int, default=5)
    parser.add_argument("--retry-backoff-sec", type=float, default=3.0)
    parser.add_argument("--output", type=str, default="")
    args = parser.parse_args()

    queries = [q.strip() for q in args.queries.split(",") if q.strip()]
    result: Dict[str, object] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "source": "arXiv API",
        "queries": {},
        "errors": {},
    }

    for q in queries:
        try:
            result["queries"][q] = fetch_arxiv(
                q,
                args.max_results,
                max_retries=args.max_retries,
                retry_backoff_sec=args.retry_backoff_sec,
            )
        except Exception as exc:  # pragma: no cover - network failures are environment-dependent
            result["queries"][q] = []
            result["errors"][q] = str(exc)

    out_path = args.output or "research/data/latest_math_arxiv.json"
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(result, f, indent=2)

    md_path = out_path.replace(".json", ".md")
    with open(md_path, "w", encoding="utf-8") as f:
        f.write("# Latest Math Feed (arXiv)\n\n")
        f.write(f"Generated: {result['timestamp_utc']}\n\n")
        for q in queries:
            f.write(f"## Query: {q}\n\n")
            for item in result["queries"][q]:
                f.write(f"- {item['published'][:10]} | [{item['title']}]({item['url']})\n")
            f.write("\n")

    print(f"wrote: {out_path}")
    print(f"wrote: {md_path}")
    for q in queries:
        print(f"query='{q}' count={len(result['queries'][q])}")
        err = result["errors"].get(q)  # type: ignore[index]
        if err:
            print(f"query='{q}' error={err}")


if __name__ == "__main__":
    main()
