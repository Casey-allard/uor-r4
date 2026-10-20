# #72 — D3 natural-corpus observation run (Simple English Wikipedia)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-23T00:05:27Z
- **Updated:** 2026-07-23T07:30:39Z
- **Closed:** 2026-07-23T05:36:23Z
- **Labels:** graph-compiler
- **Milestone:** Phase 2: Overlapping multiresolution regions
- **Assignees:** auser
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/72
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Depends on: nothing (can start anytime; consumes #59 pipeline). The confirmed D3 distribution includes a Simple English Wikipedia natural partition (CC BY-SA 4.0, SPDX in PROV) that does not exist as data yet. Produce it: fetch a pinned Simple Wiki snapshot, shard-generate teacher observations via the observe CLI into .uor-models, write the corpus manifest with CID + license, and record the held-out split rule. Acceptance: manifest committed under .uor-models with CID; the first full Gate C run on the natural partition prints in the issue; PROV section of produced artifacts references the corpus CID.


---

## Definition of Done
- A pinned Simple English Wikipedia corpus exists as teacher observations under `.uor-models/`, with a manifest carrying its CID and SPDX `CC-BY-SA-4.0` license.
- The held-out split rule is fixed in the manifest and the first full Gate C run on the natural partition is posted here.

## Acceptance criteria
- [ ] Manifest committed with corpus CID, license identifier, article count, and split rule.
- [ ] Observations are generated via the observe pipeline (content-addressed sample IDs, resumable shards).
- [ ] PROV sections of artifacts built on it reference the corpus CID.
- [ ] The natural-partition Gate C 4-way table is in this thread.

## Comments (2)

### comment 5054858383 — 2026-07-23T05:36:23Z — auser

Complete — merged via the merge queue as PR #90 (mergedAt 2026-07-23T04:57:00Z).

DoD evidence: (1) Manifest committed with corpus CID blake3:194db0ee..., SPDX CC-BY-SA-4.0 + attribution, 3,000 articles, and the deterministic split rule (blake3(id)[0] % 5 == 0, ~20% held-out). (2) From-text observation driver (observe_text.rs) produces format-identical v3 records teacher-forced via the shared encoder; content-addressed sample IDs and resumable shards per the pipeline's T-invariance rules; per-article atomic checkpoints converge to the exact single-pass merged κ on resume (crash-tested). (3) Smoke run on 10 sealed-corpus articles: 2,331 records, 16 shards, 1819 construction / 512 held-out records (8/2 articles ≈ 20% — matching the rule, independently recounted); merged κ blake3:7ba60de6...; rerun wrote 0 bytes. (4) Verified in a clean detached worktree: 47 suites, clippy -D warnings, fmt clean; PR checks all green; merged through the queue. The natural partition of D3 is now real data; #75 (HF-path full Gate C on D3) and #73 (larger-teacher ceiling) are unblocked.

### comment 5055665554 — 2026-07-23T07:30:39Z — auser

## D3 natural partition: observation run + first Gate C

**Observation run (llama2.c teacher, teacher-forced, complete):**
- Input: `.uor-models/corpora/simple-wiki-20231101/articles.jsonl` (3000 articles, sealed)
- 667,942 v3 records, 16/16 shards κ-pinned; merged κ `blake3:212bfdd49503fa3765ced1a9ce4d73605a32eddd1f0d197c05c18b465d3dff30`
- D3 partition: 534,525 construction / 133,417 held-out records = 2404 / 596 articles (19.87% held-out, rule `blake3(article id)[0] % 5 == 0`)
- 2090/3000 articles truncated at teacher sequence length 128; 736 characters unencodable in the teacher vocab replaced with spaces (deterministic, recorded in the observation report)
- PROV chain verified byte-identical: observation `manifest.json input_cid` == `committed.bin` input κ == corpus manifest `corpus_cid` = `blake3:194db0eebf2d49823ece01ee935447a0cc9edeaf018454ceea480ce7590132cf`

**Gate C, held-out D3 natural partition (133,417 positions; 16 regions induced; artifacts = llama2.c TLA container):**

| scorer | top-1 agree | bits/token |
|---|---|---|
| graph Σ-cloud (old) | 0.0% | 25.0836 |
| graph chain (Rule 1) | 17.7% | 20.1020 |
| graph chain+EXCT (Rule 1+2) | **47.9%** | **18.2465** |
| Rule 1 no-F (ablation #66) | 17.7% | 23.9442 |
| Rule 1+2 no-F (ablation #66) | 47.9% | 18.2465 |
| TLA3 store baseline | 47.9% | 35.5548 |

- Rule 1+2 status: ExactContext 133,417, Graph 0, Novel 0 — **every held-out prediction resolved via exact context**; win/loss vs baseline: both 63,862 / neither 69,555 (identical top-1 sets), but bits/token 18.25 vs 35.55.
- Candidate recall — Rule 1 top-1/top-3: 74.7%/87.2%; Rule 1+2: 87.1%/91.7%. Witness replay 64/64 ok.
- Scored artifact κ `blake3:8498a60a5fa9e85052e52e36d0bd28c03c9aee03503c47d8e164f34101a550e7` (score.r4g1, 66 MB).

**Honest caveats for the chain:** on natural text the held-out set is decided entirely by EXCT (repeated phrasing dominates at this corpus scale), so the graph rules never fire — the D3-natural judge currently measures exact-context memory, not graph generalization. Also 70% of articles are truncated at sequence length 128, so the partition covers article openings only. Both feed #75 (HF-path on D3) and the Chain-1 overlap work.


---
_Comment/review ids in this file: 5054858383, 5055665554_

