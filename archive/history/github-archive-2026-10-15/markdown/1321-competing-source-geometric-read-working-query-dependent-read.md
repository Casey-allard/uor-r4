# #1321 — Competing-source geometric read: working query-dependent read, severe text harm (References #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-21T01:33:00Z
- **Updated:** 2026-10-09T13:17:09Z
- **Closed:** 2026-09-21T01:43:25Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1321
- **Merged:** 2026-09-21T01:43:25Z
- **Merge commit:** a1fadd1f494733144c96f8419106a95ac77d8f61
- **Base / Head:** main / UOR-Foundation:codex/competitive-reader
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

> **Principal correction: [PR #1322](https://github.com/UOR-Foundation/uor-r4/pull/1322).** The preserved constructed gain is -0.179540 bits/candidate position; -2.4956 is bits/sequence. The categorical map did not learn, and fixture/prefix-state defects narrow attribution. Natural-text harm and `positive=false` remain. Bounded source repairs and the contextual-utility successor are documented in the [review](https://github.com/UOR-Foundation/uor-r4/blob/8034a74cd4eaf440e8267fc3c59c8afea3fd280a/docs/integration/competitive-reader-review-2026-09-20.md). Original report follows unchanged.

Executes [deepseek-competitive-reader-step-2026-09-20.md](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/deepseek-competitive-reader-step-2026-09-20.md) from refreshed `main` at PR #1320 (`37bf2bd1`), incorporating the corrections named by the [principal review of PR #1319](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/relational-reader-review-2026-09-20.md).

## One shared causal inference path

`read_step` + `predict_next` operate on the actual observed prefix — **no target, no sentinel, no precomputed supervised record**. Teacher-forced evaluation, autoregressive generation, interventions and timing all call them; the shared path is verified identical to a direct call at 220 positions. The old synthetic `cands_of` index references are gone: the selected occurrence carries its real `(seq, abs)` reference and the payload's real absolute position. All six retained prompts now emit **48 tokens** in both arms, fixing the previous empty-output defect.

## One relation encoding everywhere

`relation_index` is the single function used by training, hard-forward inference, export and reload. The `RLR2` artifact carries the arm's mode and code map, and an **independent** loader is exercised for **every** arm with **0 parity failures**. This repairs the defect that invalidated the previous categorical comparison (training used `(7·code[q]+code[k]) % 120` while evaluation read H4 addresses). Initialization is named explicitly as `palette.elements[a_codes[t]]`. Descriptor refinement now deduplicates affected positions, keeps the incumbent, preserves ties, accepts strict improvements with a declared tolerance and returns the exact full-objective change (−4,527.418 → −5,057.605, 52 roots moved).

## Competing-source construction

Several blocks share the query key and **all** use roles from the same paired-role class, so a source-class shortcut cannot separate them; the correct block uses the query role's partner. Placement is shuffled so recency cannot solve it either. Fresh uses a disjoint payload bank: **1,946 positions, 761 covered, 1,453 with competing candidates**.

Fresh hard-action CE (bits, lower better), paired by sequence, 2,000 draws:

| arm — same pool, actions, dose, objective | hard CE | covered reads | precision | emitted acc |
| --- | ---: | ---: | ---: | ---: |
| local (NoRead) | 12.6347 | 0/761 | 0.000 | 0.000 |
| exact-recurrence | 10.1377 | 592/761 | 0.778 | 0.251 |
| categorical (**learned** code map) | 10.1513 | 591/761 | 0.777 | 0.250 |
| **relational (H4)** | **9.9582** | **623/761** | **0.819** | **0.261** |
| relational, query blind | 10.2716 | 571/761 | 0.750 | 0.243 |

**relational − exact = −2.4956 bits [−3.7642, −1.3010]** — the whole interval below zero (the upper bound is negative, the sign check the previous run got wrong).

**The controls do real work this time.** The **query-blind** arm is *worse* than the exact reader (10.2716 vs 10.1377), so the gain is query-dependent and not a recency or source-class shortcut. The **categorical** arm — same table size, same pool, same dose, the same chance to learn its code map, independently reloaded — is indistinguishable from exact and clearly worse than H4.

## The failure: natural text

| natural-text development regression | value |
| --- | ---: |
| documents / positions | 8 / 488 |
| local → reader bits/token | 6.8002 → **8.0910** |
| delta | **+1.2908** (declared tolerance +0.05) |

The reader **harms** the language predictor. This is the PR #1314 failure mode and it is **not repaired**. `positive = false`; retained as a **partial** result with the harm explicit. Two named causes, neither addressed here: the strength is still **global** (`score = v(c) + sb[a]` factorises, so `sb` again selects 8 nats in every arm) and the descriptor is a **static single-token** code with no natural-text fit.

## Controls

Shared-path identity **pass**; independent S-query-only check **pass** (0/1946) using the retained scorer's declared absence behaviour with the identity row removed; stale reference rejected after reset **pass**; artifact reload parity **pass** for every arm; altered source payload 535 changed / 273 selected the new payload / **246 emitted** it; future-token intervention 1,053 positions compared, all positions with `i < cut` unchanged.

**One control is mis-specified and diagnosed, not hidden:** the intervention control also compared `i == cut`, where the mutated token *is* the current input and a change is expected. Those 28 changes make `instrument_checks_pass` false. Causality itself is not violated.

## Cost

| label | predictions | per prediction | ring records scanned |
| --- | ---: | ---: | ---: |
| local, **no reader work at all** | 30 | 1,279.3 µs | **0** |
| full reader path | 30 | 1,265.2 µs | 464 |

Serialized: parent E 454,788 + local artifact 53,555 + relational artifact **8,904** bytes. Peak RSS 38,584,320 B; wall 237.4 s; **energy UNAVAILABLE**.

## Tests and resources

**11** `relational` module tests pass, including three new ones (single relation function ≡ trainer index with the arms differing on identical inputs; `RLR2` round-trip rejecting identity mismatches, trailing bytes and a non-categorical arm carrying a code map; deduplicated strict descriptor search that never increases the objective, including a repeated-role example). `cargo fmt --check`, the touched-package check and `scripts/check_claim_wording.py` are clean.

Sealed, verified root `.uor-models/realtext-prior-2026-09-20/competitive-reader-1`. The PR #1319 root is preserved untouched, including its unlisted `summarize.py`, recorded honestly. **No limit extension was required**; 1,400,000 ms charged once; cumulative **183,538,565 / 191,300,000 ms**.

## Next handoff

1. **One contextual strength interaction** replacing the factorising global `sb`, over causal candidate/query/local-score observations only — never coverage or correctness labels.
2. **A source-separated natural-text fit** so a construction gain cannot silently harm the predictor.

Then re-measure the same construction and text panels. Do not widen the ring or candidate bound, sweep widths, or reopen the frozen S attribution. `#973`/`#820`/`#963`/`#964` remain open.

**References #973, #820, #963, #964 — does not close any of them.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

