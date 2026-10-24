<!-- rendered by uor-harness loop render from 875a2f309465be7528a9e3701206a745e3120308; do not edit -->
# STATE

Rendered from committed inputs at the commit in the stamp above.

## Milestone bars and holds

Source: `docs/harness/milestones.json`.

| Milestone | Status | Headline bar | Training hold |
|---|---|---|---|
| M1 Language base: a geometric stack language model good enough to carry chat | in-progress | reply acceptable: 37 of target 116 (out of 232 acceptable replies) (suspended) | HELD |
| M2 Grounded reply from exact memory: the native learner completes correct replies | blocked-on-owner | complete correct replies (dev): 437 of target 256 (out of 512 replies) (direction: open) | HELD |
| M3 Durable conversation memory: provenance, conflict, abstention, reset and restart | paused | typed outcomes qualified on the saved model: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |
| M4 One served model: the chosen model through D11 integer serving and a CLI | blocked-on-owner | softmax at runtime: yes; target no (D23 prohibits softmax project-wide) | HELD |
| M5 Laptop cost (D5): full-path M1 cost and selected parameter access | not-started | full-path cost and selected parameter access qualified: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |
| M6 Reasoning and coding on the same native model | not-started | unseen compositions and changed constraints on one model: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |
| M7 Distribution: capability API, WASM runtime and the Pages Studio | not-started | native model served through the API, WASM and Studio: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |
| M8 Alpha release: the same model for conversation, memory and coding | not-started | one labelled release on the same native path: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |

## Roadmap items by status

Source: `docs/roadmap/items/*.md` frontmatter.

| Status | Items |
|---|---|
| decision | 4 |
| done | 28 |
| held | 26 |
| ready | 33 |
| **total** | 91 |

## Newest notebook row per research line

Source: `docs/research/notebook/*.md`; the row with the latest date, the last such row on a tie.

| Line | Date | Number | Bar | Verdict |
|---|---|---|---|---|
| data-tokenizer | no rows yet | | | |
| harness-governance | no rows yet | | | |
| legal-construction | 2026-10-24 | Discrete solve: 3,679 total pivots across 12 schedules, 1,461/1,920 coefficients moved, 0/380 active guards (`min J·δ = 18161425/274877906944 ≈ +0.00006607 > 0`), exact objective `-94671994445923515/4503599627370496` (≈ `-21.021405604209846`), full guard vector matches, second run identical; exact gap decomposition `gap = 0.00786207` (`0.03739%`, down 14.22% from Unit 7) = `reduced_cost_loss (0.00229004, 29.13%)` + `complementary_slackness_loss (0.00557203, 70.87%)` with `1,890/1,920` coefficients (`98.44%`) at zero reduced cost; RCBT eliminates 5,821/28,800 discrete codes (`20.21%`), fixes 102 coefficients to a single optimal code, and reduces 152 coefficients to `<= 2` codes | none registered (instrument run on the saved guard-constrained linear problem, not a panel score) | **discrete gap reduced 14.22% to `0.007862` (`0.03739%`); 5,821/28,800 codes eliminated (102 coefficients fixed); 1,890/1,920 coefficients at zero reduced cost (`70.87%` of remaining gap is guard slack)** — item stays ready until the remaining `0.007862` discrete gap is closed |
| m1-discrete-attention | 2026-10-22 | Relation against the frozen-zero control: tokens whose cell moved 80 against 91; P_succ rows whose target moved 1,461 against 1,369; row-weighted target-to-predecessor distance 8.564 to 10.123 against 9.799; all 256 cells stay occupied | None (diagnosis; the item registers no bar) | inconclusive (placement remains a suspect on the row-weighted measures and the read is not cleared; separating them needs a device admission run under its own FIT sheet) |
| m1-prime-zeta | 2026-10-14 | 4,096 of 4,096 tokens get distinct vectors (0 collisions, 256 of 256 writer cells occupied); 17 of 17 base and 6 of 6 prime-zeta CUDA controls pass; 0 updates applied, 0 corpus rows | Bit-exact table across seeds 0, 1 and 2; no floating-point instruction in the GPU code; 0 collisions over all 4,096 tokens; 17 of 17 base controls; the codebook must separate structurally from both nulls | positive (construction only; the learned comparison against the nulls is deferred to a training sheet) |
| m2-grounded-reply | no rows yet | | | |
| m3-durable-memory | 2026-10-15 | 20 object-store integration tests and 13 existing tokenizer tests pass; a repeated fixture of 7,776 raw bytes is held in 380 record bytes | `rebuild(fold(bytes)).bytes()` equals the input and a fresh whole-object hash matches. The numeric limits (16 MiB per object, 1,000,000 records) are resource ceilings, not acceptance bars | positive (kept as a source and data-structure prototype; not promoted into the model; no language result) |
| m4-serving | no rows yet | | | |
| m6-reasoning-coding | no rows yet | | | |

## Mechanisms

Source: the ACTIVE / READY / PARKED tables in `docs/mechanisms/README.md`.

- ACTIVE: Address cells (A8)
- ACTIVE: Admission
- ACTIVE: D11 integer serving
- ACTIVE: Discrete geometric learning
- ACTIVE: Flock read
- ACTIVE: Prime-zeta codebook
- ACTIVE: Rank-margin hinge
- ACTIVE: Relation read
- ACTIVE: Routed read
- ACTIVE: Tokenizer
- READY: Hamilton gate (G2)
- READY: Object store (D19)
- READY: Recurrence gates
- READY: SpiralCore/FBS projection
- PARKED: Carrier credit
- PARKED: E8 orbit memory
- PARKED: Exact memory
- PARKED: Geometric stack
- PARKED: Hopf observation
- PARKED: Legal construction
- PARKED: Native geometric engine
- PARKED: Pointer key
- PARKED: R4G1 graph
- PARKED: Read binding
- PARKED: Spin-native weights
- PARKED: Ternary/XOR
- PARKED: VSA

## Lab registry

Source: `docs/harness/labs.json`.

| Lab | Vendor | Client | Open PR limit |
|---|---|---|---|
| antigravity | Google | Antigravity | 3 |
| claude | Anthropic | Claude Code | 3 |
| codex | OpenAI | Codex | 3 |
| deepseek | DeepSeek | DeepSeek harness | 3 |
| kimi | Moonshot AI | Kimi Code | 3 |
