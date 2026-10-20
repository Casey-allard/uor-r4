# Result — competing-source geometric read with one shared causal inference path

> **Superseding principal correction:** [Read the PR #1321 review](competitive-reader-review-2026-09-20.md) before using this preserved report. The categorical map did not learn; -2.4956 is bits/sequence (correct per-position gain -0.179540); the intended partner/absence fixture and generation state differed from their claims. Natural-text harm remains valid. Current root inventory includes unlisted `sum.py`, and the artifact is 4,340 bytes. Source repairs and focused checks do not retroactively validate this old experiment. Original `positive=false` remains.

Date: 2026-09-20. Base source `37bf2bd1` (merge of PR #1320), executing
[the competitive-reader prompt](deepseek-competitive-reader-step-2026-09-20.md). Isolated worktree
`codex/competitive-reader`. Retained root
`.uor-models/realtext-prior-2026-09-20/competitive-reader-1` (sealed, verified, 0 unlisted).

**Decision: a working query-dependent read on competing sources, and a severe natural-text
regression. `positive = false`.** The primary hard-loss criterion passes decisively and two matched
controls show the query and the geometry both matter — but the declared natural-text tradeoff fails
badly (+1.29 vs a +0.05 tolerance), so the mechanism is retained as a **partial** result with the
language harm explicit, not promoted.

## What changed structurally

**One shared causal inference path.** `read_step` + `predict_next` operate on the actual observed
prefix — no target, no sentinel, no precomputed supervised record. Teacher-forced evaluation,
autoregressive generation, interventions and timing all call them; the artifact records
`one_shared_inference_path: true` with 220 positions compared against a direct call. The old runner's
`cands_of` synthetic index references are gone: the selected occurrence carries its real `(seq, abs)`
reference and the payload's real absolute position.

**One relation encoding everywhere.** `relation_index` is the single function used by training,
hard-forward inference, export and reload. `RLR2` carries the arm's mode and code map, and an
independent loader (not a clone-and-patch) is exercised for **every** arm with 0 parity failures.
This fixes the defect that made the previous categorical comparison invalid: previously training used
`(7·code[q]+code[k]) % 120` while evaluation read H4 addresses.

**Named initialization.** `init_roots = palette.elements[a_codes[t]]` — the actual S write element,
stated explicitly rather than copying slot labels as group IDs.

**Refinement repaired.** Affected positions are deduplicated (a query-role and source-role occurrence
of the same token otherwise counts twice), the incumbent is the current assignment, ties are
preserved, and only strictly improving moves beyond a declared tolerance are accepted; the returned
value is the exact full-objective change. Fit objective −4,527.418 → −5,057.605 over the search; 52
roots moved; covered by a repeated-role test.

## Competing-source construction

Several blocks share the query key and **all** use roles from the same paired-role class, so a
source-class shortcut cannot separate them; the correct block uses the query role's partner while
competitors use partners from other families. Placement is shuffled, so recency cannot solve it
either. Fresh uses a disjoint payload bank and fresh draws: 1,946 positions, 761 covered, 1,453 with
competing candidates. Absent-answer sequences were generated but none reached the fresh panel as
candidate-bearing positions (reported, not hidden).

## Results — fresh construction, hard-action CE (bits, lower is better)

| arm (same pool, actions, dose, objective) | hard-action CE | vs local | covered reads | covered precision | emitted accuracy |
| --- | ---: | ---: | ---: | ---: | ---: |
| local (NoRead) | 12.6347 | 0.0000 | 0/761 | 0.000 | 0.000 |
| exact-recurrence | 10.1377 | −2.4970 | 592/761 | 0.778 | 0.251 |
| categorical, **learned** code map | 10.1513 | −2.4834 | 591/761 | 0.777 | 0.250 |
| **relational (H4)** | **9.9582** | **−2.6766** | **623/761** | **0.819** | **0.261** |
| relational, query blind | 10.2716 | −2.3631 | 571/761 | 0.750 | 0.243 |

Paired by sequence, 2,000 draws: **relational − exact = −2.4956 bits [−3.7642, −1.3010]** — the whole
interval is below zero (the upper bound is negative, the sign check the previous run got wrong).

**The controls do real work this time.** The **query-blind** arm scores *worse than the exact reader*
(10.2716 vs 10.1377): removing the query from the relation destroys the gain, so the improvement is
query-dependent, not a recency or source-class shortcut. The **categorical arm** — same table size,
same pool, same dose, the same opportunity to learn its code map, and independently reloaded — is
**indistinguishable from the exact reader** (10.1513 vs 10.1377) and clearly worse than H4. So on this
panel the group structure, not merely a larger learned table, carries the improvement.

## The failure: natural text

| natural-text development regression | value |
| --- | ---: |
| documents / positions | 8 / 488 |
| local bits/token | 6.8002 |
| reader bits/token | 8.0910 |
| **delta** | **+1.2908** (declared tolerance +0.05) |
| reader reads | 178 |

The reader **harms** the language predictor badly. This is the same failure mode seen at PR #1314 and
it is **not repaired**: the selector reads on 36% of natural-text positions and a wrong bounded read
is still expensive. The declared tradeoff therefore fails, and `positive` is `false` even though the
primary construction criterion passes.

Two named causes, both already identified by the audit and not yet addressed: the strength is still
**global** (`score = v(c) + sb[a]` factorises, so `argmax_a` is the same for every candidate — `sb`
again selects 8 nats in all arms), and the descriptor is a **static single-token** code with no
natural-text fitting. A contextual strength interaction and a source-separated natural-text fit are
the identified successors; neither was attempted here.

## Controls

| control | result |
| --- | --- |
| One shared inference path (generation ≡ evaluation) | **pass**, 220 positions |
| Independent S-query-only check, absence behaviour + identity row removed | **pass**, 0/1946 mismatches |
| Stale reference rejected after reset | **pass** |
| Artifact reload parity, independent loader, every arm | **pass**, 0 failures |
| Altered source payload | 535 changed, 273 selected the new payload, **246 emitted** it |
| Future-token intervention | 1,053 positions compared; all positions with `i < cut` unchanged |

**One control is mis-specified, diagnosed rather than hidden.** `future_token_intervention` reports
`unchanged_action_or_payload: false` with 28 changes. Those changes are exactly at `i == cut`, where
the mutated token *is* the current input token and a change is the expected consequence of the
intervention. Positions strictly before `cut` are unchanged. The check should compare `i < cut` plus
the predecessor position `cut−1` and exclude `i == cut`; as written it makes `instrument_checks_pass`
false. Causality itself is not violated.

Generation: all six retained prompts produced 48 tokens in both arms — the previous empty-output
defect is fixed by the shared path — and the outputs remain repetitive.

## Cost

| label | predictions | per prediction | ring records scanned |
| --- | ---: | ---: | ---: |
| local, no reader work at all | 30 | 1,279.3 µs | **0** |
| full reader path | 30 | 1,265.2 µs | 464 |

The local arm now builds no ring and performs no admission or feature work, and the denominator is
the actual prediction count. Serialized: parent E 454,788 + local artifact 53,555 + relational
artifact **8,904** bytes. Resident: shared row table 1,966,080 B, parent scratch 16,384 B. Peak RSS
38,584,320 B; wall 237.4 s. **Physical energy UNAVAILABLE.**

## Boundaries

The construction is a synthetic binding panel with authored role families; answers are defined by
document position, not by any model root, and no task family, coverage or answer label enters the
serving policy. Fresh is held-out payloads and combinations of known roles, not unseen role-family
inference and not natural language. The natural-text panel is pinned repository documentation, already
inspected, and is a development regression, not a fresh external benchmark. No alpha, conversation,
coding, frontier or energy claim follows. Dependent reads, scheduling and derived lexical computation
remain unrun.
