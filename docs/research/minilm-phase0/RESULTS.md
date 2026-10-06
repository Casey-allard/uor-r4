# Phase 0 results: MiniLM-teacher clause retrieval vs a lexical baseline

References #820. Rules: [PREREG.md](PREREG.md), committed and pushed at
`c840d259` before any arm was scored. Sealed report:
[`run3/`](run3/) (`report.json`, `rows.jsonl`, `manifest.json`). MiniLM is an
offline teacher/comparator only; nothing here is served.

## Frozen decision: **DROP**

On generator-dev, L = 55.3%, E = 51.6%, H = 49.7%. Both candidates are below
75% and below L, so each one is DROP and the overall result is **DROP**. Under
the frozen rule the static-E8 clause index is not built.

## Results (top-1 clause hit)

| set | n | reachable | same-clause | L | M | S | S_raw | **E** | **H** | H_F | H_E8root |
|---|---|---|---|---|---|---|---|---|---|---|---|
| generator-dev | 376 | 293 (77.9%) | 77 (20.5%) | **55.3%** | 54.8% | 50.8% | 51.3% | **51.6%** | **49.7%** | 50.8% | 40.7% |
| v3 memory (descriptive) | 30 | 30 | 0 | 80.0% | 76.7% | 80.0% | 80.0% | 80.0% | 73.3% | 76.7% | 80.0% |

- *reachable*: some history clause holds an expected spelling and no forbidden
  one. This is the ceiling for every arm under the frozen segmentation.
- *same-clause*: some clause holds both an expected and a forbidden value (the
  segmentation limit).
- M is the contextual MiniLM sentence embedding (upper-bound comparator). S,
  E and H use static tables. S_raw, H_F and H_E8root are descriptive.

### E8 quantization (centered static table, 30,522 × 384)

| quantity | value |
|---|---|
| scale | 2 / rms = 5.9086 (centered rms 0.33849) |
| global relative error ‖v − q‖/‖v‖ | 0.1339 |
| mean per-token relative error | 0.1378 |
| mean per-token cosine(v, q) | 0.9905 |
| max \|doubled coordinate\| | 27 (fits `i8`) |
| distinct 8-block codes | 1,279,903 (pair table 1.64 × 10¹² entries) |

## What the result shows and does not show

**Measured (pre-registered):**

- The decision is DROP. On generator-dev, no static arm reaches the lexical
  baseline. The contextual teacher M does not reach it either (54.8% vs 55.3%).
- The ceiling is low because of segmentation. Only 77.9% of dev questions
  have a clean expected clause, and 20.5% have the expected value and a
  distractor in the same clause. Even a perfect selector could not reach the
  90% BUILD threshold under this segmentation. The probe therefore rejects
  "clause = segmentation unit + semantic score" for these families. That is a
  different claim from "semantic selection does not help".
- E8 quantization is not the cause. E (51.6%) is within 1 point of float S
  (50.8%), and the per-token cosine to the float vector is 0.99.
- The proposed table realization does not hold at this scale. The realized
  block codes are not a small codebook: 1.28 M distinct codes give a pair
  table of about 1.6 × 10¹² entries. At this scale the integer dot product can
  only be served as a direct integer dot product, which uses multiplications,
  not as a pair-table lookup. A servable version needs bounded codebooks, for
  example the 240 roots or a shell-limited E8P codebook. Phase 0 did not test
  that version. H_E8root is the nearest arm, and its score is lower (40.7%).
- The Hopf base point loses information. On dev, H (fiber discarded) scores
  49.7% against 50.8% for S. Keeping the fiber (H_F, 50.8%) recovers that
  difference exactly. The descriptive evidence matches the AGENTS warning that
  Hopf observation loses fiber information unless the fiber is retained.

**Descriptive, post-hoc (not in the decision; one draw):** results by
generator family, as hits out of n.

| family | n | reachable | L | M | S | E | H |
|---|---|---|---|---|---|---|---|
| binding | 77 | 76 | 62 | 68 | 68 | **69** | 64 |
| count | 43 | 43 | 35 | 38 | 37 | 37 | 36 |
| self_fact | 31 | 31 | 17 | 26 | 20 | 20 | 19 |
| attribute | 32 | 29 | 25 | 21 | 20 | 21 | 21 |
| reverse | 12 | 12 | 11 | 11 | 11 | 11 | 10 |
| cross_relation | 15 | 15 | 15 | 15 | 15 | 15 | 15 |
| implicit_update | 33 | 32 | 14 | 13 | 10 | 10 | 11 |
| assistant_stated | 20 | 20 | 17 | 2 | 0 | 0 | 2 |
| update | 30 | 8 | 7 | 7 | 6 | 7 | 5 |
| order | 52 | 19 | 2 | 3 | 2 | 2 | 2 |
| rule | 31 | 8 | 3 | 2 | 2 | 2 | 2 |

On binding questions, the "turtle vs goldfish" pattern that motivated the
probe, the static and E8 arms beat lexical: E 89.6% vs L 80.5%. On
assistant_stated questions they fail: E 0/20 vs L 17/20. Update, order and
rule questions turn on recency, version or negation, not similarity. This
split is a hypothesis for a later, separately pre-registered probe, for
example semantic scoring restricted to binding plus a version/recency rule. It
is not a result of this one.

## Execution record

- Commits after the pre-registration changed only the implementation, not any
  rule:
  1. `linear` now runs one flattened 2-D matmul instead of a broadcast batched
     matmul, for speed. Parity was re-checked with the same maximum deviation,
     4.5e-5.
  2. The separator scan lowercases ASCII only, so byte offsets stay on char
     boundaries. Run 2 panicked on an "é" in a dev message. A unit test now
     covers this case.
- Attempts:
  - Run 1 was stopped during static-table construction, before any arm was
    scored. Its root (`attempt.json` only) was removed.
  - Run 2 crashed partway through scoring. Its unsealed, partial root is
    preserved and was not read.
  - Run 3 is the only complete, sealed run.
  - Run 2, run 3 and the generator output are stored as cloud-store
    `minilm-phase0-runs-20261006`. The teacher files are stored as
    `minilm-l6-v2-weights` (md5 `a04ecd91…`).
- Teacher parity: the Rust forward reproduces the published sentence-transformers
  cosines with maximum deviation 4.5e-5 and the expected token ids. The test
  `clause_probe::tests::minilm_parity` is `#[ignore]` and needs
  `UOR_MINILM_DIR`. At this head it passed together with the 6 unit tests.
- Cost:
  - Laptop CPU only.
  - Run 3 took 134 s, of which 39 s built the static table.
  - Builds took about 20 min in total across rebuilds, in
    `~/.cache/uor-claude-target`.
  - New laptop data: 91 MB of model files, since moved to cloud-store and
    removed locally, plus under 1.5 MB of generator and report output.
- Inputs:
  - `model.safetensors`: `53aa5117…d9db`
  - `tokenizer.json`: `be50c362…2037`
  - v3 panel: `578e2564…a163`
  - v3 checks: `be827811…30c3`
  - dev: `f501281c…c5c1`
  - static table: `e600b99b…c589`
  - E8 codes: `a77d1058…f9f1`
- `data/panels/conversational-v4*` was neither read nor evaluated.
