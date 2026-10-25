<!-- rendered by uor-harness loop render from 9ac274a8766789501e12f082ee6cfd0bb4f6cc44; do not edit -->
# STATE

Rendered from committed inputs at the commit in the stamp above.

## Milestone bars and holds

Source: `docs/harness/milestones.json`.

| Milestone | Status | Headline bar | Training hold |
|---|---|---|---|
| M1 Language base: a geometric stack language model good enough to carry chat | in-progress | reply acceptable: 37 of target 116 (out of 232 acceptable replies) (suspended) | released |
| M2 Grounded reply from exact memory: the native learner completes correct replies | blocked-on-owner | complete correct replies (dev): 437 of target 256 (out of 512 replies) (direction: open) | HELD |
| M3 Durable conversation memory: provenance, conflict, abstention, reset and restart | paused | typed outcomes qualified on the saved model: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |
| M4 One served model: the chosen model through D11 integer serving and a CLI | not-started | softmax at runtime: yes; target no (D23 prohibits softmax project-wide) | released |
| M5 Laptop cost (D5): full-path M1 cost and selected parameter access | not-started | full-path cost and selected parameter access qualified: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |
| M6 Reasoning and coding on the same native model | not-started | unseen compositions and changed constraints on one model: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |
| M7 Distribution: capability API, WASM runtime and the Pages Studio | not-started | native model served through the API, WASM and Studio: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |
| M8 Alpha release: the same model for conversation, memory and coding | not-started | one labelled release on the same native path: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |

## Roadmap items by status

Source: `docs/roadmap/items/*.md` frontmatter.

| Status | Items |
|---|---|
| decision | 1 |
| done | 34 |
| held | 1 |
| ready | 55 |
| **total** | 91 |

## Newest notebook row per research line

Source: `docs/research/notebook/*.md`; the row with the latest date, the last such row on a tie.

| Line | Date | Number | Bar | Verdict |
|---|---|---|---|---|
| data-tokenizer | no rows yet | | | |
| harness-governance | no rows yet | | | |
| legal-construction | 2026-10-24 | Speed-up only: Unit 11's rational outputs reproduced bit-for-bit, `solve_ms 69,140`, `disjunctive_ms 146,897` (Unit 10: `264,832` / `1,116,550`). With the sort fix: tree lower bound `≈ -21.02881252`, gap `≈ 0.00685092` (`0.03259%`), `6,406/28,800` codes eliminated, 115 fixed, 186 binary-or-fixed; `solve_ms 57,057`, `disjunctive_ms 148,896` (taken under machine load) | none registered (instrument run on the saved guard-constrained linear problem, not a panel score) | **certified discrete gap narrowed to `0.006851` (`0.03259%`); `disjunctive_ms` cut 86.66% from Unit 10 with exact outputs** — item stays ready until the gap is closed |
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
- ACTIVE: Discrete geometric learning
- ACTIVE: Flock read
- ACTIVE: Prime-zeta codebook
- ACTIVE: Rank-margin hinge
- ACTIVE: Relation read
- ACTIVE: Routed read
- ACTIVE: Tokenizer
- READY: D11 integer serving
- READY: Hamilton gate (G2)
- READY: Hopf observation: kept-fibre map
- READY: Object store (D19)
- READY: Recurrence gates
- READY: Spin-native weights
- READY: SpiralCore/FBS projection
- READY: Ternary/XOR
- PARKED: Carrier credit
- PARKED: E8 orbit memory
- PARKED: Exact memory
- PARKED: Geometric stack
- PARKED: Hopf observation: sector routing
- PARKED: Legal construction
- PARKED: Native geometric engine
- PARKED: Pointer key
- PARKED: R4G1 graph
- PARKED: Read binding
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
