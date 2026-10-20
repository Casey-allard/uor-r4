# #626 — router: tropical (max-plus) multi-hop route composition — P-4-legal, witness-bearing; pre-registered experiment

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-12T00:08:51Z
- **Updated:** 2026-08-12T00:54:11Z
- **Closed:** 2026-08-12T00:54:11Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/626
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Spawned from #622 work item 1 (parent campaign #589). The P-4 legality memo on #622 established: uor-matmul's tropical (max-plus) path is add+compare-only arithmetic — inside the deployed kernel's legal vocabulary — and its `⊕` is a *selection*, making the argmax witness-bearing by construction (winning path identity falls out of the accumulation, D-6 smallest-index tie rule).

## Mechanism

Multi-hop route composition as tropical linear algebra: with per-edge additive scores (log-domain or integer route costs), the max-plus product over a path IS the path score, and tropical "matmul" over the transition structure composes k-hop routes with no multiply, no float, no growth in accumulator width (k-independent, e.g. 10 bits for i8 scores). Candidate mechanism for extending the router's single-hop scoring to witness-bearing multi-hop routing **inside** the zero-matmul runtime.

## Implementation plan (from the #622 memo — route (a))

- Tropical kernel implemented in r4 runtime source (widening int add + compare + `-inf` absorbing guard), so the P-4 source scan keeps full authority; allocation census must stay green.
- uor-matmul as **dev-dependency only** (git rev pin): differential parity test — r4's kernel must byte-agree with `uor_matmul_core::dot_tropical_ref` over the conformance corpus, incl. `-inf`/padding semantics (their CK-17: pad = mask = `-inf`, A-6).
- Quantization of existing scores into the tropical alphabet (i8/i16 route costs) is part of the experiment design, not assumed free — saturation policy must be declared.

## Measurement discipline (binding)

**No run before a pre-registered criterion is posted as the FIRST comment on this issue**, per repo measurement discipline: the harness (score pipeline / Gate C surface), the baseline row (current deployed scoring path on the same inputs), the metric and threshold that would make this mechanism adopted vs recorded-negative, and the null. Cheap check first: a small fixed corpus slice before any corpus-scale run; serialize with other heavy cloud runs. Negative results close this issue with the record, not silence (pattern: #457 Arm B).

## Safeguards

P-4 scan and allocation census untouched and green; no change to deployed artifact wire semantics; any store/score κ that moves gets a re-pin record. This issue is an experiment — adoption into the serving surface is a separate decision with its own gate.


## Comments (2)

### comment 5260531450 — 2026-08-12T00:25:41Z — Casey-allard

## PRE-REGISTERED CRITERION (binding; posted before any run, per this issue's terms)

### Mechanism under test

Max-plus (tropical) composition over the compiled forward-transition graph (`compile_transitions_with_quantization` — region→region, Q16.16 log-domain `ScoreQ`, top-8 out-degree), made **token-dependent by composing through destination-region emission residuals** — because the deployed record (score_runtime.rs:801-807) proves a token-independent transition offset is argmax-neutral: E_f edges ship in every artifact and contribute exactly zero to serving today. Route-then-emit model, per held-out position with active regions A (same `binary_memberships` path as the scorer):

- 1-hop: `trop1(t) = max over u∈A, edge u→w of [ wq(u→w) + ΔE_w(t) ]` (ΔE absent ⇒ 0)
- 2-hop: `trop2(t) = max over u∈A, paths u→v→w of [ wq(u→v) + wq(v→w) + ΔE_w(t) ]`
- Arm score: `S(t) = root(t) + trop(t)`; positions with no outgoing route fall back to root. `⊗` = i64 add (saturation unreachable at Q16.16 magnitudes), `⊕` = max, zero = -inf sentinel; witness (u,v,w) retained by the selection — the tropical argmax is witness-bearing by construction.

Experiment code runs test-side (P-4 scans runtime source only); runtime adoption is a separate decision gated on this result.

### Arms (identical inputs; in-repo 500k fixtures; same `load_inputs`/split as the #457 harness so rows are cross-comparable)

- **NULL** — root prior alone. Cross-check: must reproduce the #457 table's floor (0.0620 top-1 / 8.5635 bits ± small quantization drift) or the instrument is broken.
- **1H** — root + trop1. The control: does the forward-transition seat carry ANY token signal at one hop?
- **2H** — root + trop2. The mechanism under test.
- **N** — instrument null: 2H recomputed with edge destinations deterministically deranged (sorted-dst rotation by ⌊len/2⌋; no RNG). Must NOT beat 1H or 2H; if it does, the instrument is invalid and no claim issues.

### Metrics, thresholds (predeclared)

Held-out N=5000 (`R4_TROP_HELD`), top-1 teacher-argmax agreement + bits/token over the support (root-top-2000 ∪ route-constrained tokens ∪ true next; miss ceiling 24 bits — the #457 conventions verbatim).

- **ADOPTION GATE (POSITIVE):** top-1(2H) ≥ top-1(1H) + 0.02 absolute AND bits(2H) ≤ bits(1H) AND top-1(2H) > top-1(NULL). (0.02 = the pinned Gate-C alarm threshold, the repo's own effect-size floor.)
- **Secondary (reported either way):** 1H vs NULL — whether the E_f seat carries any signal at all; if 1H ≤ NULL, that is a decisive negative for the whole seat, independently corroborating why f_emissions is off.
- **NEGATIVE ⇒** close #626 with the measured table; register the lane as a dormant `open` claim in model/ledger.toml (cd-space-dormant pattern). No corpus-scale run follows from this issue in either direction; scale-up would be its own pre-registered issue.
- Anti-vacuity (Rule 5): assert >0 positions composed ≥1 two-hop path, and mean L1(2H vs NULL distributions) > 0 — the instrument must be able to fail.

### Semantics parity (binding before the run counts)

The harness's max-plus fold must agree exactly with `uor_matmul_core::dot_tropical_ref` (uor-matmul @ b13c984, **dev-dependency only**) on a deterministic vector corpus including -inf absorption, empty folds, and padding (their CK-17: pad = mask = -inf). This lands the #622 item-2 dependency addition as scoped there (dev-dep at first real use; deny.toml gains the uor-matmul git source deliberately).

### Cost

Region graph ≤ ~256 nodes × out-degree 8 ⇒ ≤ ~16k 2-paths; the expensive step is the store build any fixture harness pays. Single run, nothing else heavy in flight.


### comment 5260798815 — 2026-08-12T00:54:10Z — Casey-allard

## RESULT — NEGATIVE under the pre-registered exit rule (run 2026-08-12, single run as registered)

Harness: PR #627 (parity test green — the fold is pinned to `uor_matmul_core::dot_tropical_ref` before the run counted). Fixtures: in-repo 500k, 5000 held-out, same split as #457. Full command + log in the harness module docs.

```text
  routes present: 1-hop at 5000 positions, 2-hop at 5000
  2H argmax won via a residual-constrained route at 5000 positions
  witness examples: region 35 -> 35 -> 35 (token 9038), region 8 -> 15 -> 16 (token 2501)

  arm            bits/token   top-1
  NULL (root)      8.5701     0.0620    <- reproduces the #457 floor (8.5635/0.0620): instrument valid
  1H tropical     14.5113     0.0044
  2H tropical     14.5171     0.0044
  N  deranged     16.9093     0.0044    <- worse than both legit arms: ordering valid
```

**Verdict (pre-registered):** NEGATIVE. 2H−1H top-1 = +0.0000 against a +0.02 gate; bits 14.5171 vs 14.5113; both tropical arms fall ~5.9 bits BELOW the unigram floor.

**The secondary finding is the decisive one:** 1H−NULL = −0.0576 top-1 / +5.94 bits — **the E_f seat carries no route-composable token signal at ANY hop count on this cover.** The transition weights compose fine (the mechanism worked; witnesses were retained at every position; the self-loop witness `35→35→35` shows the compiled transition graph is dominated by region self-transitions — the graph itself is the degenerate part), but composing routes through emission residuals actively hurts, because the residuals are near-noise (the #456/#459/#499 line) and the route term drags argmax onto them. This corroborates the deployed argmax-neutral configuration (`f_emissions` off) from a second, independent angle: it is not just that the token-independent offset was neutral — there is nothing token-shaped in this seat to recover by ANY composition, tropical included.

**What survives the negative:** the max-plus kernel itself — P-4-legal, witness-bearing, parity-pinned to the ecosystem reference — is mechanism, not claim, and stays available. Registered as `tropical-route-dormant` in model/ledger.toml (cd-space-dormant pattern) with its activation gate: a transition/emission structure on which a pre-registered rerun clears the #626 thresholds. The plausible future trigger is a compiled graph whose forward transitions are NOT self-loop-dominated (e.g. post-#597/#603 broad-corpus artifacts or the #604/#605 attention-operator line), not a retune on this cover.

Closing per the registered rule: no corpus-scale run follows; scale-up would be its own pre-registered issue. Ledger PR follows.


---
_Comment/review ids in this file: 5260531450, 5260798815_

