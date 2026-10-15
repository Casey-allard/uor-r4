# #267 — Pinned D3 corpus is not re-materializable (manifest-only, no fetch path, upstream has drifted)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T15:53:21Z
- **Updated:** 2026-07-29T23:03:44Z
- **Closed:** 2026-07-29T23:03:44Z
- **Labels:** bug
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/267
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
The pinned D3 corpus (`.uor-models/corpora/simple-wiki-20231101/`) is **not re-materializable**: the repo/machine carries only `manifest.json` (source recipe + `corpus_cid`), no `articles.jsonl` and no fetch tool. Discovered when the #242 measurement required a fresh observation on a machine that had the manifest but not the 12MB corpus.

Worse: refetching per the manifest's own recipe (wikimedia/wikipedia, `20231101.simple`, rows 0..2999 via datasets-server) yields **12,038,579 text bytes vs the pinned 11,972,624** (~0.5% drift) — the upstream dataset has been reprocessed since the 2026-07-22 fetch, so the pinned content cannot be reconstructed from the recipe at all. A pinned corpus whose bytes exist nowhere retrievable breaks the reproducibility story for every D3-anchored number.

### Fetch script used (working, with integrity check)
```python
import json, urllib.request, sys, time

OUT = ".uor-models/corpora/simple-wiki-20231101/articles.jsonl"
BASE = "https://datasets-server.huggingface.co/rows?dataset=wikimedia%2Fwikipedia&config=20231101.simple&split=train"
rows = []
for offset in range(0, 3000, 100):
    url = f"{BASE}&offset={offset}&length=100"
    for attempt in range(5):
        try:
            with urllib.request.urlopen(url, timeout=60) as r:
                data = json.load(r)
            rows.extend(data["rows"]); break
        except Exception as e:
            if attempt == 4: sys.exit(f"FETCH FAILED at offset {offset}: {e}")
            time.sleep(3 * (attempt + 1))
assert len(rows) == 3000
text_bytes = 0
with open(OUT, "w", encoding="utf-8") as f:
    for item in rows:
        row = item["row"]
        obj = {"id": row["id"], "url": row["url"], "title": row["title"], "text": row["text"]}
        text_bytes += len(row["text"].encode("utf-8"))
        f.write(json.dumps(obj, ensure_ascii=False) + "\n")
print(f"{len(rows)} articles, text_bytes={text_bytes} (manifest pins 3000 / 11972624)")
```

### Proposal
1. Commit the corpus bytes somewhere the project controls (repo LFS, release asset, or the ModelStore CAS — it is 12MB and CC-BY-SA-4.0 with attribution already recorded in the manifest), keyed by the manifest's `corpus_cid`, so "pinned" means retrievable.
2. Land the fetch script as `scripts/fetch_d3_corpus.py` with the CID check — as the fallback re-materialization path, documenting that upstream drift is expected.
3. Re-pin: either bless the 2026-07-28 refetch as the new pinned corpus (new `corpus_cid`, era note, D3-anchored baselines re-measured), or locate the original bytes and store them per (1). Decision for the team.
4. Manifest gains a `retrieval` field stating where the pinned bytes live.

### DoD
`scripts/fetch_d3_corpus.py` (or CAS retrieval) reproduces the pinned corpus byte-for-byte on a fresh machine, verified against `corpus_cid` in CI or a documented manual step; the #242 measurement chain is re-runnable from a clean clone.


## Comments (1)

### comment 5124270217 — 2026-07-29T23:03:43Z — Casey-allard

## Closing — delivered in batch-2 (PR #289, merge c187ef0)

The central premise dissolved on measurement: **the corpus never drifted.** blake3 of the 2026-07-28 refetched articles.jsonl equals the pinned corpus_cid (blake3:194db0ee…) bit-for-bit; the reported 0.5% delta was an accounting mismatch — the manifest's `text_bytes` was a *character* count (11,972,624 chars, exact match) while the refetch compared utf-8 bytes (12,038,579).

Delivered against the proposal: (1) corpus bytes are project-controlled — release asset at tag `d3-corpus-simple-wiki-20231101`, keyed by corpus_cid, verify-after-download documented; (2) `scripts/fetch_d3_corpus.py` landed in #271 and now verifies character totals AND the corpus_cid itself when the blake3 module is present; (4) manifest gains the `retrieval` field (release asset primary, fetch script fallback) plus `text_chars`/`text_utf8_bytes`/accounting note. (3) Re-pin decision is **moot** — the original bytes were never lost. D3-anchored baselines stand unchanged; #268's corpus-drift candidate was eliminated by this finding.


---
_Comment/review ids in this file: 5124270217_

