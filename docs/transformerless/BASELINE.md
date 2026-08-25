# Baseline and Working Assumptions — R⁴ Graph Compiler

Phase 0 deliverable of `docs/r4_graph_compiler_implementation_plan.md` (§5 Phase 0).
Date: 2026-07-21. Status: living document; numbers are marked **fresh** (measured this phase),
**cited** (from prior certificates), or **pending** (harness in flight / tool missing).

## 1. Working assumptions for decisions D1–D8

Adopted as working assumptions on maintainer go-ahead (2026-07-21); reversible until the plan is
formally committed. Full text: plan §2.

- **D1** — Commit scope is the M0–M5 tranche. Go/no-go at the M.V.G. checkpoint (§4 below).
- **D2** — Reproducibility semantics: canonical deterministic compiler mode (normative scalar FP,
  pinned seeds, no platform BLAS) for certificate-bearing artifacts; platform-accelerated modes
  allowed for local iteration, validated by behavioral equivalence (PDF §15). Recorded in
  `docs/transformerless/R4G1.md` §7.
- **D3** — Evaluation distribution: declared in §2 below.
- **D4** — Runtime fallback: consult EXCT exact-residual evidence, then abstain with explicit
  status; manifest-declared per status. Chat UX decided when Phase 5 lands the manifest.
- **D5** — Phases 6/7 are trigger-gated on measured region counts / bytes-read counters, not
  scheduled.
- **D6** — Deployment target: **revised by maintainer 2026-07-21** — the primary deployment target
  is another platform (to be named before Phase 1 freezes R4G1 widths and runtime features); wasm
  is demoted to optional. Caller-owned-bytes-first remains as a target-neutral design rule.
- **D7** — Process: backlog filed as GitHub issues #11–#34 under Phase milestones on
  `UOR-Foundation/uor-r4`; work continues on `feature/proof-carrying-semantic-routing` with the
  graph path cfg-gated; benchmark/reproducibility gates run on the pinned dev machine until a
  pinned CI runner exists.
- **D8** — This plan serves the research thesis. Product-usefulness gate (chat quality eval)
  required before funding Phases 8–10.

## 2. Evaluation distribution declaration (D3)

**Decided 2026-07-22 (maintainer):**

- **Continuity partition**: the existing teacher-generated story corpus (`Corpus`,
  `compiler.rs:51`) with its 80/20 construction/held-out split (`train_cut`), so new numbers are
  comparable to the cited P2 certificate.
- **Natural partition**: a pinned **Simple English Wikipedia** sample, license **CC BY-SA 4.0**
  (recorded in PROV as SPDX `CC-BY-SA-4.0`). Sizing and split rules are fixed at first use
  (target: a few thousand articles, construction/held-out by content hash), pinned by CID; the
  corpus manifest lives under `.uor-models/` and its CID is referenced in artifact HEAD/PROV.
  (Alternative considered: TinyStories validation split — cleaner licensing-wise but keeps the
  evaluation synthetic; rejected in favor of a genuinely natural distribution.)
- All fidelity claims carry: distribution id, n, confidence interval, slices, seeds, stopping
  rule (Gate K). No claim generalizes beyond the declared distribution.

## 3. Baseline measurements

### 3.1 Fidelity

| Metric | Value | Status | Source |
|---|---|---|---|
| top-1 accuracy | 28.9% | cited (150k era) | PROOF.md P2 (legacy stories15M teacher, ~10⁵ store keys) |
| teacher-argmax agreement | 31.7% | cited (150k era) | PROOF.md P2 |
| bits/token (WB) | 6.54 (teacher floor 1.5960, ceiling 70.4%) | cited (150k era) | PROOF.md P2 |
| store keys | 89,200 | cited (150k era) | PROOF.md P2 |
| **top-1 accuracy (500k era)** | **34.7%** | fresh, 2026-08-01 | #327 re-pin full certify; PROOF.md P2 era note (500,000 tokens, 2,507 stories, 100,306 held-out, TLA7 artifact κ `blake3:ef6a20f3…`) |
| **teacher-argmax agreement (500k era)** | **39.0%** | fresh, 2026-08-01 | same |
| **bits/token WB (500k era)** | **8.0249** (teacher floor 1.4260, ceiling 70.5%) | fresh, 2026-08-01 | same |
| **store keys (500k era)** | **179,068** | fresh, 2026-08-01 | same |
| **TLA7 cpy8 residual row (500k era)** | **35.3% / 39.6% / 8.5247 WB / 195,650 keys** | fresh, 2026-08-01 | #335 Phase C rows; certifier log |
| **TLA7 mantissa-fold candidate (500k era)** | **34.6% / 38.9% / 8.3097 WB / 162,119 keys** | fresh, not adopted | #335 Phase C rows; no top-1 gain |
| HF-path evaluation tooling | exists | landed | PR #41 (`evaluate-report`); issue #34 closed |
| **Gate C harness (Phase 4)** | TLA3 store baseline 31.7% / 11.88 bits-token | fresh, 2026-07-22 | `r4 transformerless score`, fixture corpus, 30,036 held-out positions — reproduces the P2 agreement anchor; bits/token is the canonical cross-entropy definition (GLOSSARY.md), scorer+ds named |
| **Gate C: graph formula v1 (Σ-over-cloud)** | **0.3% / 70.47 bits-token** | fresh, unfavorable | correlated sibling-subtree residual stacking (issue #64, redesign in flight) |
| **Gate C: Rule 1+2 (chain+precedence)** | **31.7% / 9.86 bits-token** | fresh, 2026-07-22 | argmax-identical to baseline on all 30,036 positions, better bits; redesign landed (#64 closed) |
| **Gate C: broad-corpus 360M (PINNED, #516)** | **24.30% / 11.94 bits** (best live arm 31.48% / 10.43) | fresh, 2026-08-09 | **broad** D3 (Simple-Wiki), SmolLM2-360M-Instruct teacher, 360,924 records / 2,994 stories, 72,864 held-out, EXCT-miss 25.7%, teacher floor 3.6015 bits, 64/0 witness replays; `docs/smollm2_teacher_baseline_320.md` #516. Distinct distribution from the stories15M home rows above — this is the canonical baseline for **broad-text** claims; stories15M is retained for the home distribution. |
| **Gate C re-run at `aea30bae` (baseline audit)** | rule12 **36.55% / 8.3222**; TLA3 baseline 39.17% / 8.4985; EXCT-miss-slice generalization **1.81% / 16.89** (n=14,943) | fresh, 2026-08-18 | committed 500k/TLA7 fixtures, full census n=100,306, `positions_sampled: 0`, witness replay 64/64, EXCT resolves 85.1%; trend alarm PASS vs the 2026-07-30 pin (31.63%/9.1181). Same-machine live probes the same day: all three loadable local bundles (pre-#755 bytes) produce deterministic, prompt-invariant degenerate output through the R4G1 ask path — `docs/project_baseline_audit_2026_08_18.md` §13. |

**Current scope correction (#933/#934, 2026-08-24; historical rows above are
unchanged).** The fixed top-1 tolerance of **29.7%** (`31.7% - 2pp`) belongs to
the pinned/legacy quality profile; it is not a universal 30% requirement.
Broad-corpus `relative_tla` reports compare the deployed scorer with TLA on the
same positions. The attested `smollm2-360m-broad-clean` reference report is a
full census of 72,130 held-out positions: Rule 1+2 **24.393%** versus TLA
**28.121%**. Consequently strict production admission is not established by
that report or by the bypassed local-load canary.

Gate C Rule 1+2 and #908's **29.702%** skip-mix row are compiler-side
`GraphScorer` / `R4Engine` reference measurements, not evidence that the sole
normative deployed selector, `R4G1Runtime`, served those tokens. #908 remains
valid teacher-free reference/off-serving evidence; its gate was a paired
skip-mix delta whose 95% lower bound had to clear **+20‰**, not an absolute
top-1 floor. See [the #934 audit](../canonical_quality_baseline_934.md), which
owns remediation ordering. RF-31 remains **NOT ESTABLISHED** at deployed-serving
scope until the content-bound normative-runtime report in
[`normative_r4g1_quality_933.md`](../normative_r4g1_quality_933.md) receives an
evidence-backed verdict; #932's BDD parity harness remains downstream.

**Current normative outcome (#933, 2026-08-25; appended correction).** The
content-bound schema-2 production bundle is **RATIFIED at its exact scope**:
`R4G1Runtime` greedy decode records **21,293 / 72,130 (29.5203%)**, compared
with same-position TLA **20,284 / 72,130 (28.1214%)**, for a paired **+13.988
permille, 95% CI [11.057, 16.919]**. Its same-generation sections-absent
control records **18,806 / 72,130 (26.0723%)**, giving the RF-31 lane **+34.479
permille [31.681, 37.277]**. These paired gates, not an absolute 30% threshold,
are the decision rule.

The binding graph/report CIDs are
`ff82dfd5f04eac7e944443b1ea4cc9fe93a007b3b8f07286876d52709a98bc49`
and `88ee8210e1f4c48dc26999f5685350b2d2343676cdbd6f9b1aee7c7f1c66146f`.
The hardened release manifest's raw BLAKE3 is
`c2025e9e507e8367993d78bd83ef099ce5851c838d3cc5cf01eda5560986ad33`
(SHA-256
`7572e07a1e3722f3ffc0ea749a67b4ac162221de79b5b4b8a315f4e4e6570fde`)
and it binds comparator-store CID
`c1749e62077758c4a098e2a02150b5455e1ca3c02c60b87e6d45fcbb9e2b4404`.
Strict admission passed from an empty model store after verifier hardening at
`f901cd97577da3117fd52c9b1c6dcf075cc4d3a2` (graph/evaluator revision
`74ced4d12a84a176d73665106f88d0aab9407453`).

This does not relabel #908's 29.702% `R4Engine` reference/off-serving row and
does not establish instruction following, reasoning, factuality, semantic
abstention, free-running coherence, live-teacher parity, or a cross-model
floor. The BDD suite was 124 / 124, but live-teacher parity fixtures were absent
and those scenarios vacuously skipped; #932 remains downstream.

**Canonical bits/token (issue #76, resolved 2026-07-22):** one definition — mean cross-entropy of
the true next token under a scorer's predicted distribution, `(1/N) Σ −log2 P_scorer(v_i|c_i)`
with floor mass included (GLOSSARY.md). Values are comparable only within the same scorer AND
distribution; the historical "families" are scorer/distribution differences, not metric
differences: 6.54 = P2 certificate (Witten-Bell store, legacy corpus), 11.88 = same helper on the
fixture corpus (Gate C baseline row), 9.86 = Rule 1+2 graph scorer on the fixture corpus.
Every report must name scorer + distribution alongside the value.

Important: the cited certificate belongs to the legacy llama2.c stories15M teacher, **not** to
the current default SmolLM2-135M-Instruct compile. The Gate C harness reproduces its 31.7%
agreement anchor on the fixture corpus; HF-path certificates for the SmolLM2 compile are
producible via the PR #41 tooling on the D3 distribution (§2).

**D3 first pass (declared n), issue #75 — fresh, 2026-07-23.** First Gate C evaluation on the
declared D3 distribution (§2), both partitions scored with the same SmolLM2 TLA5 artifacts
(`.uor-models/compiled/smollm2-135m-instruct/tless_artifacts.bin`, κ
blake3:d4623b3a7db8888200a9210decd9b363b42b7fb6f32823ec9810b5223708aa3f) and the default
cover/score configuration (add-one smoothing). Reports:
`.uor-models/observed/simple-wiki-slice400/score_report.json` and
`.uor-models/observed/smollm2-continuity/score_report.json`.

Corpora:

- **Natural partition** — 400-article prefix slice of the sealed Simple English Wikipedia corpus
  (sealed `articles.jsonl` CID blake3:194db0eebf2d49823ece01ee935447a0cc9edeaf018454ceea480ce7590132cf;
  slice CID blake3:33d6bded0dd477b891b0c80bd27da13818d5aacc38457b6d3174cba043fa4c17; CC BY-SA 4.0).
  n = 400 articles → 320 construction / 80 held-out (the §2 blake3(id)%5 rule lands exactly
  80/20 on this slice); 90,019 teacher-forced observation records (72,562 construction /
  17,457 held-out; 285/400 articles truncated at the 256 teacher sequence length), 8 shards,
  observation merged κ blake3:43d961d73798579567692f680c55da78020c32054ce72ae892327e85afee12d6.
  The cover/score consumption layout (story-major, construction ordinals first; only record
  order and the story-ordinal field derived from the merged shards) is κ
  blake3:ff7ee846c598e5e2ee31ae74a22cfe2fe600e9efbf55b4832c46759af057c893.
- **Continuity partition** — the existing SmolLM2 teacher-generated corpus
  (`.uor-models/compiled/SmolLM2-135M-Instruct-7e27bd9f9532/corpus.{meta,records}`, complete),
  n = 2,002 stories → `train_cut` 1,601; 200,000 records (159,658 construction /
  40,342 held-out); corpus κ blake3:74491d1d80f426f675a35f22a98e6ca0a7de83bbfd1f87bcb9ad763ebe96f12a.

Natural partition — 17,457 held-out positions:

| scorer | top-1 agree | bits/token |
|---|---|---|
| graph Σ-cloud (old) | 0.03% | 60.41 |
| graph chain (Rule 1) | 0.19% | 56.69 |
| graph chain+EXCT (1+2) | **15.04%** | **13.30** |
| TLA3 store baseline | 15.04% | 18.76 |

Status: ExactContext 17,457 / Graph 0 / Novel 0. Win/loss 1+2 vs baseline: 2,626 both, +0/−0,
14,831 neither — argmax-identical on every position. Candidate recall (Rule 1 / 1+2):
64.9%/85.0% and 44.5%/62.6% top-1/top-3. Witness replay 64/64. Graph: 41 nodes, 458 edges,
10.53 MB scored artifact.

Continuity partition — 40,342 held-out positions:

| scorer | top-1 agree | bits/token |
|---|---|---|
| graph Σ-cloud (old) | 0.005% | 50.32 |
| graph chain (Rule 1) | 3.23% | 46.97 |
| graph chain+EXCT (1+2) | **14.76%** | **14.62** |
| TLA3 store baseline | 14.76% | 20.63 |

Status: ExactContext 40,342 / Graph 0 / Novel 0. Win/loss 1+2 vs baseline: 5,955 both, +0/−0,
34,387 neither — argmax-identical. Candidate recall (Rule 1 / 1+2): 60.5%/79.1% and
42.7%/59.6%. Witness replay 64/64. Graph: 45 nodes, 532 edges, 21.56 MB scored artifact.

§4 rows 1–2 verdicts (baseline = each run's own HF-path TLA3 store row):

| row | natural | continuity |
|---|---|---|
| 1: agreement ≥ baseline + 5pp (stretch) / ≥ baseline (floor) | 15.04% vs 20.04% — **stretch FAIL; floor PASS** (equal) | 14.76% vs 19.76% — **stretch FAIL; floor PASS** (equal) |
| 2: bits/token ≤ baseline − 0.3 | 13.30 ≤ 18.46 — **PASS** | 14.62 ≤ 20.33 — **PASS** |

Deployed quality gate (`src/r4g1.rs::validate_quality_report`: Rule 1+2 agreement must not be
worse than baseline): **PASS on both partitions** (agreement equal, never worse).

Read of the gap vs the fixture row above (Rule 1+2 = 31.71%/9.86, TLA3 31.7%/11.88, n=30,036):
the fixture is the legacy stories15M-teacher distribution, so its 31.7% anchor is not comparable
to these first SmolLM2-teacher measurements beyond harness shape. Within the D3 distribution
(same teacher, same artifacts, same harness), natural text is mildly easier for the store than
the teacher's own generations (+0.28pp agreement; baseline bits 18.76 vs 20.63, −1.88), and the
pure graph lane (Rule 1, no EXCT) is far weaker on natural text (0.19% vs 3.23%): natural
8-token contexts repeat across articles far less than teacher-generated ones, so region-level
residuals transfer poorly while exact-context store coverage stays total (ExactContext 100% on
both partitions — the Rule 1+2 rows are entirely EXCT-driven, hence argmax-identical to the
baseline).

**Full-corpus confirmation (issue #75, 2026-07-23).** The first pass above scored a 400-article
prefix slice of the natural partition; the declared natural partition is the full sealed corpus
(3000 articles, #72). Re-run at n=3000 (361,232 records, 72,131 held-out by the same §2 rule;
observation merged κ `blake3:a646421d…`, teacher-forced at sequence length 128 with 2545/3000
articles truncated): Rule 1+2 **28.1% / 11.3565** vs TLA3 baseline **28.1% / 11.2481**
(argmax-identical, ExactContext 100%; scored artifact κ `blake3:3c99b707…`). The larger corpus
shifts the target-2 natural verdict: the baseline's bits/token improves from 18.76 (n=400) to
11.25 (n=3000) as exact-context coverage grows, while the graph's bits stay ~flat — so on the
full declared corpus the natural row reads **target 1: stretch FAIL / floor PASS (equal)**,
**target 2: FAIL** (11.3565 > 11.2481 − 0.3 threshold would need ≤ 10.95). The slice verdicts
above are retained as the n=400 record; the n=3000 rows are the declared-corpus measurement.
Continuity corroboration: an independently regenerated 150,000-record SmolLM2 continuity corpus
(same recipe as the declared 200k one) scored Rule 1+2 4.7% / 15.37 vs baseline 4.7% / 18.11 —
same shape as the declared-corpus row (floor PASS, target-2 PASS), lower absolute agreement,
i.e. generation-stream sensitive. §4.1 carries the consolidated target table.

### 3.2 Artifact sizes (fresh, 2026-07-21, `.uor-models/compiled/smollm2-135m-instruct/`)

| File | Bytes | Note |
|---|---|---|
| `tless_artifacts.bin` (TLA4) | 1,710,348 | codebooks, thresholds, class sigs (incl. certifier-only f32 `ctx_cb`) |
| `tless_store.bin` (TLS1) | 494,286 | graded evidence store — **stale: u16-era entries (6 B); current u32 parser rejects it. Regenerate via recompile; incident validates R4G1's versioning rules** |
| `tokenizer.bin` | 528,975 | byte-level BPE export |
| total deployed | 2,733,609 | vs. ~271 MB BF16 source (~99× smaller, nominal) |

Cited for the legacy artifact: 87.2× compression at 0.9692 mean cosine (full depth), 28.1×
end-to-end vs. 60.8 MB source (PROOF.md P5).

### 3.3 Runtime contract

All fresh numbers verified 2026-07-21 by `tests/allocation_census.rs` against the real SmolLM2
artifacts (deterministic across runs; debug profile).

| Metric | Value | Status |
|---|---|---|
| multiplies in runtime kernel | 0 (machine-checked source scan, witness P-4) | cited/enforced |
| integer ops per token | ~1.8×10⁵ | cited (PROOF.md P1, legacy path) |
| op census, SmolLM2 path (32 greedy tokens) | **144,496 avg ops/token**: adds 48,530 · xors 36,864 · shifts 11,666 · compares 1,324 · table-reads 46,112 | fresh |
| allocations per generated token, steady state | **0** (asserted over 32 tokens across `assign_window`, `predict`, `predict_witness`, `generate_greedy_into`; `Runtime::new` also 0) | fresh, Gate B pattern holds |
| allocations, warm-up (finding) | **5 allocs / 496 B during the first ~34 predictions** — `Runtime.recent` (repetition guard) is a `Vec` that grows to steady-state capacity. The "allocation-free hot path" is amortized, not unconditional. Graph-runtime fixed-capacity `RuntimeState` (Phase 5) removes this by construction | fresh |
| allocations at parse/load | artifacts: 18 allocs / 1.71 MB; store (real, legacy TLS1-u16 parse): **57,498 allocs / 5.40 MB for a 494 KB container** (~10.9× — per-key `Vec<u8>` + `BTreeMap` nodes) | fresh; Phase 1 R4G1 packed layout is the fix |
| allocations on write path (`add_evidence` ×64) | 563 allocs / 51.1 KB | fresh, known; formalized as patch epochs in Phase 9 |
| bytes read / cache misses per token | deferred | needs pinned runner (D7) |
| per-token latency | pending | needs bench harness (criterion) |

### 3.4 Reproducibility

- κ-reproduction (byte-identical recompile) holds for the canonical deterministic path, fixtures in
  `crates/uor-r4-core/tests/fixtures/baseline_kappa.json` (`--release --test kappa_reproduction
  -- --ignored` with `TLESS_CANONICAL_DETERMINISTIC=1`). Legacy accelerated teacher builds remain
  platform-sensitive and are not used for the cross-platform claim.
- **Baseline anchor moved 2026-07-21** (maintainer decision): the pin was stale from
  `b142c93`-era after two deliberate compiler redesigns — `5baa7c0` (phase-10: u32 token IDs, new
  corpus record layout with top-3 tokens/weights, oracle separation) and `bbdd596` (hash-index
  RVQ projection, relational prefixes). Investigation before re-pinning: (1) compiler determinism
  verified — two independent compiles produced identical 27-κ sets; (2) stage-0 drift traced to
  those redesigns, not to nondeterminism or platform wobble. The same u32 migration is what
  invalidated the on-disk TLS1 store (§3.2). Re-pinning helper: `dump_baseline_kappa` in
  `tests/kappa_reproduction.rs`. Lesson recorded for Gate E: an unversioned baseline plus a
  redesigned compiler = a broken reproduction gate; R4G1's HEAD records compiler identity for
  exactly this reason.
- **Baseline anchor moved 2026-08-01** (maintainer decision, issue #327): the teacher corpus was
  scaled 150,000 → 500,000 tokens (2,507 stories, 100,306 held-out) and the #318 Phase B residual
  wiring (TLA7 container) landed in the same pin. Investigation before re-pinning: compile ran
  twice (`transformerless compile` then the full `certify` recompile) with byte-identical
  containers — κ `blake3:ef6a20f3…`, 1,346,836 bytes — so the pin drift is the intended corpus +
  artifact-era change, not nondeterminism. Token-side pins unchanged by construction; threshold,
  context-codebook, class-signature and container κs moved with the corpus. Fixture
  `c_recs.bin` grew 1.8 MB → 24 MB (12-byte legacy records → 48-byte records with anchors/top-8).
  Full-certificate record: PROOF.md P2 era note; pins: `baseline_kappa.json`.

- **D2 canonical re-pin verified 2026-08-02** (issue #265): the 500k/TLA7
  fixture was compiled with `TLESS_CANONICAL_DETERMINISTIC=1`; all token-side,
  bundle-derived, and container κs matched the existing pin exactly (container
  1,346,836 bytes, κ `blake3:ef6a20f3…`). The fixture bytes therefore required
  no replacement; the mode is recorded as the certificate reproducibility
  policy and CI Gate E now runs it on Linux and macOS.

- **Baseline anchor moved 2026-08-04, fifth re-pin** (maintainer decision, issue #407,
  PR #413): `CTX_SAMPLE` raised 6,000 → 50,000 after the #411 attribution sweep (era
  self-consistency +1.4pp, sample size +0.5–0.7pp; `CTX_ITERS` unchanged at 6). Compiled
  under canonical deterministic mode (`TLESS_CANONICAL_DETERMINISTIC=1`, the #265 D2 policy,
  now required for re-pins); container κ moved `blake3:ef6a20f3…` → `blake3:8fbf3f68…`
  (1,346,836 bytes, size unchanged). Token-side pins and the threshold vector are
  byte-identical to the prior era by construction; only the context codebooks, class
  signatures, and container moved. Pins: `baseline_kappa.json`.

- **Phase C adoption evidence recorded 2026-08-01** (issue #335): the
  single-key/query-beam shape remains selected over write-time fan-out
  (34.7% / 39.0% / 8.0249 WB / 179,068 keys versus 20.3% / 22.7% / 8.1473 /
  817,683). i8 and i16 residual-copy rows are tied at reported precision;
  i8 is retained for the smaller artifact. The 1.5× mantissa-bit norm-fold
  candidate improves WB and key count but does not improve top-1, so it is
  recorded rather than enabled. The certifier also replays the persisted
  TLA7 container witness on 512/512 sampled positions.

## 4. M.V.G. checkpoint targets (D1) — CONFIRMED

**Confirmed by the maintainer 2026-07-22 ("all defaults", unamended).** Absolute values are set
relative to the first HF-path certificate (PR #41 tooling); reference points are the cited
legacy numbers above. These are the go/no-go contract for the Phase-5 checkpoint review:
missing 1–2 or 4 ⇒ stop or redesign; missing 3/5/6 ⇒ redesign discussion.

Pass conditions for the Phase-5 minimum viable graph, all on the declared distribution (§2):

1. Teacher-argmax agreement ≥ (HF-path TLA3 baseline + 5 percentage points), and in no case
   worse than that baseline (Gate C floor).
2. Bits/token ≤ HF-path baseline − 0.3.
3. Deployed artifact bytes ≤ 2× the current TLA4+TLS1 total (~5.5 MB).
4. Zero allocations per token step (hard requirement, Gate B).
5. Per-token latency ≤ 2× the current runtime on the pinned machine.
6. Novel/Contradictory fallback rate measured and reported; on-distribution rate < 20%.

Missing target 1–2 or 4 ⇒ stop or redesign. Missing 3/5/6 ⇒ redesign discussion.

### 4.1 Measured values (2026-07-23, issue #75)

Cover + score + Gate C for the SmolLM2-135M compile on both D3 partitions; per-partition
4-way tables and run details in §3.1 (declared corpora; natural at the full sealed n=3000)
and issue #75.

| target | continuity (declared 200k corpus) | natural (full sealed corpus, n=3000) | verdict |
|---|---|---|---|
| 1. agreement ≥ baseline + 5pts | 14.76% vs 14.76% (+0.0) | 28.1% vs 28.1% (+0.0) | stretch **FAIL** both; floor ("not worse") **PASS** both (argmax-identical prediction sets) |
| 2. bits/token ≤ baseline − 0.3 | 14.62 ≤ 20.33 (−6.01) | 11.3565 > 10.9481 (+0.11 over baseline) | **PASS** continuity, **FAIL** natural |

Structural caveat, material to the verdict: on both partitions every held-out prediction
resolved via exact context (ExactContext 100%, Graph 0, Novel 0), so rows 1–2 currently
compare EXCT-table scoring against the store baseline, not graph generalization. The
natural target-2 verdict is corpus-size sensitive (PASS at the n=400 slice, FAIL at the
declared n=3000) because exact-context coverage improves the baseline faster than the
graph with more data. Until an EXCT-free evaluation exists, the stop-or-redesign signal
from missing targets 1–2 should be read as "the current judge measures exact-context
memory" — the Phase-5 review decision input is here and in issue #75.

## 5. Threat-model note (backlog #22)

Full adversarial model: `docs/transformerless/THREAT_MODEL.md`. Headline threats: crafted region
activation, overlap poisoning, frontier/candidate exhaustion, fallback denial-of-service, integer
saturation, collision with privileged concepts. Defenses: strict fan-out/frontier limits,
validated routing bytecode, checked integer semantics, bounded patch layers, adversarial collision
suites, separation of semantic routes from cryptographic identity.

## 6. Phase 0 exit status

- [x] Glossary frozen (`docs/transformerless/GLOSSARY.md`)
- [x] R4G1 RFC drafted (`docs/transformerless/R4G1.md`)
- [x] D1–D8 working assumptions recorded (§1)
- [x] Backlog filed as issues #11–#34 with phase milestones (D7)
- [~] Baseline measurements: artifact sizes, allocation census, and op census fresh
  (`tests/allocation_census.rs`); fidelity pending issue #34; bytes-read/latency pending bench setup
- [x] Threat model written (`docs/transformerless/THREAT_MODEL.md`)
- [x] M.V.G. targets confirmed by maintainer (§4; confirmed 2026-07-22 — checkbox reconciled 2026-08-18, baseline audit)
