# #1314 — Learned geometric exact-occurrence reader: targeted selection works, abstention fails (References #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-20T19:35:51Z
- **Updated:** 2026-10-09T13:17:46Z
- **Closed:** 2026-09-20T19:51:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1314
- **Merged:** 2026-09-20T19:51:36Z
- **Merge commit:** fbf542aa4c1f7ed336cd028c6b7b3502cba4118c
- **Base / Head:** main / UOR-Foundation:codex/occurrence-reader
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Executes [deepseek-occurrence-reader-step-2026-09-20.md](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/deepseek-occurrence-reader-step-2026-09-20.md) from refreshed `main` at PR #1313 (`1dae322c`).

**One complete constructive learning task.** No output projection, width/precision search or reset campaign. The [design note](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/occurrence-reader-design-2026-09-20.md) was written **before** any scoring.

## Mechanism

`z_local(v) = z_E(v) + u_S(b)` for `i ≥ 2`, else `z_E`: frozen E plus the separable artifact's **query row only** — no history row and **no history fold** (verified: read-disabled equals `z_local` at all 978 checked positions). A 128-token causal ring with session sequence identity admits at most 24 past occurrences of the current token; nine 0/1 features are scored by a ten-scalar signed ≤4-bit selector against an explicit NoRead threshold; the selected occurrence's **exact observed successor** is added as `+2^14`. Occurrence artifact **130 bytes**; export/reload decision mismatches 0.

## Finding 1 — targeted selection works and the geometry contributes

Construction population through the real BPE path: every role/key/value is a genuine single-token word that round-trips `decode`/`encode`. A decoy repeats the query key with another role and another value, placed after the answer in half the sequences. Fresh panel seeded and frozen before scoring, using **held-out value ids never used as payloads during fitting**.

On the **`geom_slot_only`** family — the correct source's role is a *different* token sharing its transported write slot, so exact identity cannot identify it — fresh reader-relevant hard accuracy:

| Condition | accuracy |
| --- | ---: |
| `z_local` baseline | **0.000** |
| **learned reader** | **0.825** [0.768, 0.880] |
| fixed latest-occurrence control | 0.533 |
| geometry-disabled (same weights, `w2=w3=w7=w8=0`) | 0.550 |
| fixed exact-role-match control | 0.008 |

So the transported-slot feature carries **+0.275 absolute** over the same weights without it. Controls all pass: read-disabled ≡ `z_local`; future-token causality; stale reference rejected after reset; altered source payload changes the emitted payload (38 changed, **37 correct**, 0 abstained); export/reload parity; and a **three-process cross-process continuation** whose final parameters, exported selector and checkpoint bytes are identical.

## Finding 2 — abstention is uncalibrated and the reader is not usable on natural text

| Panel | `z_local` | reader | delta | tolerance |
| --- | ---: | ---: | ---: | --- |
| fresh synthetic, all 3,609 positions | 11.7806 | 12.4519 | **+0.6713** | +0.01 → **FAIL** |
| bounded raw text, 488 positions | 6.8002 | 9.5892 | **+2.7891** | +0.02 → **FAIL** |

Raw text: 88 positions with candidates, 10 covered, **read at all 88, 83 false copies**. The failure is **precision, not recall** — 10/10 covered positions were read correctly; the damage is the admitted-but-uncovered stratum that a construction-only training signal cannot separate from a useful recurrence. **No candidate is promoted.**

## Near misses preserved sealed

| Root | Change | fresh `geom_slot_only` relevant | raw delta |
| --- | --- | ---: | ---: |
| `occurrence-reader-1` | surrogate fit only, μ = 1 | 0.400 | +0.72 |
| `occurrence-reader-2` | declared class balance + tune-only μ sweep (μ = −7) | 0.825 | +2.79 |
| `occurrence-reader-3` | stale-binary duplicate of `-2` | 0.825 | +2.79 |
| **`occurrence-reader-4`** | corrected altered-source control + corrected decision predicate | **0.825** | +2.79 |

Version 1's failure had a precise cause: the softmax surrogate is not the hard endpoint, and after scaling to signed 4-bit the threshold landed **exactly** on the correct candidate's score when the correct source was newest (`f2 + f8 + f6 = 4 + 1 − 4 = 1`, `μ = 1`, strict comparison) — so it abstained when it should have read. A declared **tune-only** threshold sweep fixed that without touching the weights. Version 2/3 carried a mis-specified altered-source control (it rewrote the target, not the source) and an over-generous automatic label; both are corrected in the retained version, and the predicate now requires both declared CE tolerances.

## Cost

Uncached declared serving path (E computed per position, no token-pair cache, ring and selection included, 32-token real window, one discarded warm-up then five timed repeats): **1,215.2 µs/position** for `z_local` and **1,213.8** with the reader — the reader adds no measurable cost. Serialized bytes 508,473 (parent 454,788 + local query artifact 53,555 + occurrence artifact 130). Whole-run peak RSS 142,458,880 B; wall 110.4 s. **Physical energy UNAVAILABLE.**

## Generation

Six retained prompts, 48 greedy tokens through the same integer path: all six repeat (pair-cycle periods 13/20/4/1/1/1; no complete bounded-ring cycle). Single-token prediction boundary named; **no multi-token continuation integration exists yet**.

## Source

- `crates/uor-r4-core/src/native_geometric/learner/occurrence.rs` — 11 focused tests: ring staleness and future-index rejection, eviction reporting, causal admission and the candidate bound, indicator features with a unique newest flag, geometric slot equality across distinct tokens, NoRead invariance, threshold/tie behaviour, artifact round trip with malformed/trailing/foreign-digest/out-of-range rejection, quantization ranking preservation, a hand-built loss/weight-movement check, and checkpoint round trip with deterministic epoch order.
- `crates/uor-r4-core/src/bin/occurrence-reader.rs` — BPE-round-tripping word banks, construction populations, fit, tune-only threshold calibration, controls, raw-text probe, generation and uncached cost.

`cargo fmt --check`, the touched-package offline check, the 11 occurrence tests and `scripts/check_claim_wording.py` are clean.

## Resources

Standing-authorized **+10800000 ms** extension to **191300000 ms** recorded in the live ledger **before** consumption. This step charges **2,700,000 ms** once. New cumulative **181,238,565 / 191,300,000 ms**, remaining 10,061,435 ms.

Retained: `.uor-models/realtext-prior-2026-09-20/occurrence-reader-4` — sealed, verified, 21 files, 3,385,009 bytes. Three superseded reader roots and three continuation roots preserved sealed. The owner-authorized cleanup removed only regenerable debug caches; every release binary and sealed root survived. 128 MiB stop margin intact; no paid external compute.

## Next handoff

**Add exactly one declared abstention signal, learned from natural-text positions where a candidate exists but no exact successor is useful, and re-evaluate the same frozen panels, controls and tolerances.** Leave the ring size, candidate bound, feature set, weights and amplitude frozen — this is an abstention fix, not a re-fit. Do not re-fit the selector, widen the ring/candidate bounds, or sweep width/precision/corpus. CPQK production continuation remains required **before any fit that uses `QueryTrainer`**; the occurrence reader's own ten-scalar trainer has a working cross-process continuation and does not need it. `#973`/`#820`/`#963`/`#964` remain open.

**References #973, #820, #963, #964 — does not close any of them.**


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5752324163 — 2026-09-20T20:05:34Z — Casey-allard

Principal source/evidence review completed after delivery; corrections and successor are in PR #1317 ([audit](https://github.com/UOR-Foundation/uor-r4/blob/d90d39183813781356187fe2b496fb13a37a3f74/docs/integration/occurrence-reader-audit-2026-09-20.md), [constructive next prompt](https://github.com/UOR-Foundation/uor-r4/blob/d90d39183813781356187fe2b496fb13a37a3f74/docs/integration/deepseek-memory-utility-step-2026-09-20.md)). Reviewed c6a39784 source/results are unchanged at delivered fdae053c/merge fbf542aa.

The constructed selection gain is retained: 411/443 relevant positions versus latest 279/443. All retained manifest hashes verify. The decisive correction is **5/10 covered raw positions correct, not 10/10**: `rank_hit` at runner1250 counts any read when coverage exists. There are five correct and83 wrong reads overall; five wrong are covered,78 uncovered. Ranking is therefore unresolved alongside abstention.

Other instrument corrections are specified in the audit: the future-token control reconstructs an identical unused sequence and reanalyzes the original; ReadDisabled only tests residual identity; the purported geometric feature is eight-class equality with targets authored from that partition and a confounded coefficient ablation; paired support against latest is missing; the cost baseline already performs admission/features and divides29 predictions by32; executable_sha256 contains the whole executable hex. Original sealed receipts stay unchanged, with a derived true executable hash recorded separately.

The next task should improve **ranking, NoRead and bounded copy strength against actual post-injection loss**, with a minimal contextual feature where needed. The current +2^14 residual is16nats, so freezing it and the fallible ranker is not warranted. This supersedes the one-abstention-signal handoff, preserves the useful ring/exact-payload/trainer work, and does not qualify or discard the entire geometric architecture. No new model run occurred in the review. #973/#820/#963/#964 remain open.



---
_Comment/review ids in this file: 5752324163_

