# #299 — C serving row re-measure on the TLA6 D3 bundle (relative_tla profile) — Phase C follow-up

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-30T06:16:43Z
- **Updated:** 2026-08-03T15:41:51Z
- **Closed:** 2026-08-03T15:41:51Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/299
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Deferred from #243 so the epic could close on measured evidence. Remaining work, all mechanical: (1) the D3 score run must declare `--quality-profile relative_tla` (the pinned fixture floor 29.7% correctly refuses natural-corpus bundles; the graph beats same-corpus TLA baseline 5.38 vs 5.17, which is the applicable gate); (2) add a C-only certify mode (R4_CERTIFY_C_ONLY) — today the 3-minute serving eval requires the full ~15-min certify to reach it, and R4_CERTIFY_ROWS_ONLY skips it entirely; (3) run the C row on .uor-models/compiled/smollm2-135m-instruct-tla6 with the part-2 (metric-consistent, witness-attested) serving path and record the exact/graph/novel + abstention split vs the TLA5-era served-1021 row. All prerequisite evidence is on #243: D3 Gate C row measured (5.38%/15.22, strict miss 13.64%, Gate C valid), fixture row 31.63%/9.12 pinned.

## Comments (4)

### comment 5147574892 — 2026-07-31T21:21:35Z — auser

Started the implementation path in draft PR #309: https://github.com/UOR-Foundation/uor-r4/pull/309

The new `R4_CERTIFY_C_ONLY=1` mode skips reference-checkpoint loading and runs only the existing C serving row. The available local bundles are not a TLA6 measurement fixture: one has incomplete corpus metadata, and the other fails the pinned R4G1 quality floor (15.96% vs 29.70%). The actual TLA6 remeasurement remains pending until that bundle is available.

### comment 5153989530 — 2026-08-01T23:29:56Z — auser

_edited 2026-08-01T23:30:09Z_

## C serving row re-measurement — done, with one material deviation on the D3 Gate C row

Measured on a worktree at **bf9a63a** (the C-only-mode commit; deliberately inside the TLA6 era — `origin/main` has since moved to era TLA7 via cff684f). macOS arm64, rustup-pinned toolchain, release binary.

### (2) `R4_CERTIFY_C_ONLY` — verified working as designed

bf9a63a is what the issue meant. `R4_CERTIFY_C_ONLY=1 r4 certify` skips reference-checkpoint loading and the full certificate and runs only the C serving row; explicit measured-row-or-recorded-skip behavior preserved. Dry run against the legacy `.uor-models/compiled/smollm2-135m-instruct` bundle correctly records `unusable (corpus: incomplete corpus …)` — the same refusal the #309 comment logged.

### Era/determinism checks (pipeline validation before trusting any new row)

- Fresh TLA6 artifact compile at bf9a63a from the checked-in fixture corpus (`R4_CORPUS_META/RECS` overrides, llama2.c teacher): **1,051,916 bytes, κ blake3:cf64fe7493d0e366…** — byte-identical to the era-3 pin.
- Fixture score sanity row: **rule12 31.63%/9.1181**, status ExactContext 25,853 / Graph 4,182 / Novel 1, strict miss **13.93%**, baseline 33.83/11.9515, graph κ blake3:0b8563fc… — exact match to the #298 pin and the pinned trend graph.

### (1) D3 score run with `--quality-profile relative_tla` — measured, but does NOT reproduce the issue's row

Reconstructed TLA6 D3 bundle: fresh SmolLM2-135M TLA6 compile (`--seconds 300 --target 150000 --sequence-length 128`; generation stream verified byte-prefix of the 7e27bd9f9532 continuity corpus) + the declared D3 natural partition (full sealed `simple-wiki-20231101`, SmolLM2 teacher-forced at seq 128, 361,232 records, canonical `blake3(id)%5` split, 72,131 held-out), default score config:

| | top1 | bits/token | strict miss | status split |
|---|---|---|---|---|
| **rule12 (relative_tla)** | **23.23%** | **11.6446** | **3.1%** | ExactContext 69,897 / Graph 2,042 / Novel 192 |
| same-corpus TLA baseline | 23.19% | 11.5513 | | |

The graph beats the same-corpus baseline on top-1 (23.23 ≥ 23.19) so the deployed gate passes with `relative_tla` (the 29.7% pinned floor is correctly bypassed for this natural-corpus bundle); per the #234 declaration, at 3.1% strict miss Gate C **cannot** measure generalization on this distribution — the row restates exact-context recall.

**Deviation:** the issue's D3 row (5.38%/15.22, strict miss 13.64%, baseline 5.17) is **not reproducible from any corpus on this machine**. Full candidate matrix (all bf9a63a, `relative_tla`, default config, fresh TLA6 artifacts):

| pairing (artifacts + corpus) | rule12 | bits | strict miss | baseline | gate |
|---|---|---|---|---|---|
| llama2.c (κ cf64fe74) + llama2.c-observed wiki full (133,417 ho) | 30.72 | 19.98 | 1.5% | 30.72 | pass (tie) |
| llama2.c + llama2.c-observed wiki slice400 (17,083 ho) | 21.27 | 11.61 | 1.9% | 21.38 | FAIL |
| SmolLM2 + SmolLM2-observed wiki full (72,131 ho) | **23.23** | **11.64** | **3.1%** | **23.19** | **pass** |
| SmolLM2 + continuity corpus (40,342 ho) | 15.57 | 14.41 | 22.9% | 17.27 | FAIL |
| SmolLM2 + fresh 150k own corpus (31,696 ho) | 14.37 | 14.21 | 25.7% | 16.39 | FAIL (verified: engine load refuses — "graph runtime top-1 14.37% is below TLA baseline 16.39%") |

The corpus behind the pre-TLA6 `graph-*` variants (κ blake3:37e4d217…, 40,965 records / 8,193 held-out) is lost on this machine and is **not** a story-boundary prefix of the deterministic generation stream at seq 128 (record 40,965 sits mid-story); a seq-256 regeneration lands at exactly 40,000 records. The 5.38/15.22 row therefore appears to depend on the bundle as compiled on Casey's Mac — if that bundle's corpus/artifact κs can be posted here I can verify byte-for-byte.

### (3) C serving row — measured (part-2 metric-consistent, witness-attested serving path)

`R4Engine` + `score.r4g1` + D4 policy on the reconstructed bundle, deterministic stride subsample, default budgets. Rerun byte-identical (determinism confirmed).

| | served | exact / graph / novel (served) | abstained (exact/graph/novel/contradictory) | top1 on served | agreement on served | probe |
|---|---|---|---|---|---|---|
| **TLA6 D3 bundle (this run, n=1003)** | **987 (98.4%)** | **834 / 153 / 0** (0 widened) | **16 (0/0/16/0)** | **2.9%** | **4.9%** | 61/64 served, 4 hits |
| TLA5-era (#280, n=1021) | 1021 (100%) | 1021 / 0 / 0 | 0 (0/0/0/0) | 5.6% | 12.6% | 14/64 hits |

Reading vs the TLA5 row: the D4 abstention lane now engages (16 Novel abstentions vs 0 — the first non-zero abstention split on record for this row) and 15.2% of served positions route through the graph path (153 vs 0), so the serving split does read differently under TLA6; served-surface accuracy dropped (top1 5.6→2.9%, agreement 12.6→4.9%) on this natural-corpus bundle. Caveat: the bundle is my reconstruction (see deviation above), not Casey's original — the split percentages, not the absolute accuracy, are the era-comparable signal here.

### Commands (run from the worktree; `$OBS` = `.uor-models/observations/simple-wiki-20231101-hf` in the main checkout)

```bash
git worktree add .worktrees/pr-299 bf9a63a --detach && cd .worktrees/pr-299
CARGO_TARGET_DIR="$PWD/target-wt" rustup run stable cargo build --release --offline --bin r4

# TLA6 artifacts (resumable; ran 5× to reach 150,000 tokens, done=1)
target-wt/release/r4 compile --source <main>/.uor-models/sources/smollm2-135m-instruct \
  --output .uor-models/compiled/smollm2-135m-instruct-tla6 --seconds 300 --target 150000 --sequence-length 128

# D3 score, relative_tla (bundle: corpus.meta←$OBS/state.bin, corpus.records←$OBS/merged.bin)
target-wt/release/r4 transformerless score \
  --corpus-meta $OBS/state.bin --corpus-recs $OBS/merged.bin --stories $OBS/stories.jsonl \
  --artifacts .uor-models/compiled/smollm2-135m-instruct-tla6/tless_artifacts.bin \
  --quality-profile relative_tla --out .uor-models/compiled/smollm2-135m-instruct-tla6/graph

# C serving row
R4_CERTIFY_C_ONLY=1 R4_CERTIFY_SERVING_BUNDLE=$PWD/.uor-models/compiled/smollm2-135m-instruct-tla6 \
  target-wt/release/r4 certify
```

Bundle left in place at `.worktrees/pr-299/.uor-models/compiled/smollm2-135m-instruct-tla6/` (artifacts κ blake3:ff7f2218…, graph κ blake3:9cec4773…, 15,170,635 bytes). Leaving the issue open per handoff — the D3-row deviation above wants a maintainer decision (accept the reconstruction's row, or supply the original bundle's κs).


### comment 5166910918 — 2026-08-03T13:25:53Z — Casey-allard

## Maintainer decision: original bundle located — κs posted for byte-for-byte verification

The TLA6 D3 bundle behind the 5.38/15.22 row is intact on my machine at `.uor-models/compiled/smollm2-135m-instruct-tla6/` (main checkout). Recorded input κs from its `score_report.json`:

- artifact κ: `blake3:4503e1d25ecca561329080ce947be834bf47d8873de84c41474d28e73583a5d4`
- corpus κ: `blake3:d36361f1b5f456bee5ee9184302bd015159adf4056d3aa0c2e960a0fccff470d`
- graph κ: `blake3:4377c9f3e9002975e748a85026733abd541d2b08cfa2060e4b3e80ee231acf91`

Report row, verbatim from that file: `rule12_precedence` 72,258 positions, top1 **0.05380719** (5.38%), bits **15.2231**; `tla3_baseline` 5.17% / 16.904; status split **62,404 EXCT / 8,705 graph / 1,149 novel**.

These κs differ from the reconstruction's (`ff7f2218…` artifacts / `9cec4773…` graph), which confirms the reconstruction was a different bundle, as suspected. Decision: **the original row stands as the evidence of record.** @auser — if you want byte-level verification I can post per-file blake3 sums or ship the bundle files (~15 MB). Leaving the issue open until you've verified; then it closes on the original row, with your re-measured serving-split observations (first non-zero abstention lane, 15.2% graph-path share) noted as the era-comparison companion.


### comment 5168553143 — 2026-08-03T15:41:51Z — Casey-allard

## Maintainer decision: accepting the reconstruction — closing on delivered evidence

Ari redid the work end-to-end and the remaining gap is bookkeeping, not measurement — resolving it rather than holding the issue open on a byte-verification round-trip:

- **The reconstruction's C serving row is accepted as the evidence of record** for the TLA6-era re-measure, with Ari's own caveat quoted as part of the record: the era-comparable signal is the serving split (16 Novel abstentions — first non-zero abstention lane on record — and 153/987 graph-path served), not the absolute accuracy of a reconstructed bundle.
- The original bundle's κs posted above stand as the historical anchor: they confirm the reconstruction was a different bundle, so the 5.38/15.22 D3 row and the reconstruction's 2.9% served row are both retained in this thread as measurements of *different* bundles, not a contradiction. The original bundle remains archived on my machine if byte-level verification is ever wanted.

All three scope items are delivered: (1) relative_tla declaration honored, (2) `R4_CERTIFY_C_ONLY` verified working as designed, (3) C row measured on the part-2 witness-attested serving path. Era note: these are TLA6-era records; main has since moved to TLA7 (#329/#338), and the serving-measurement story continues on #375 (the current-era fixture bundle is refused by the engine quality gate) and the #374 campaign tracker.

Closing. Thanks @auser for the thorough reconstruction and deviation record.


---
_Comment/review ids in this file: 5147574892, 5153989530, 5166910918, 5168553143_

