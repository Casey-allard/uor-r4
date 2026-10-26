<!-- rendered by uor-harness loop render from 66b8a75059e6c014d5f4d81ab567b2490f24faa4; do not edit -->
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
| done | 58 |
| held | 15 |
| ready | 19 |
| **total** | 93 |

## Newest notebook row per research line

Source: `docs/research/notebook/*.md`; the row with the latest date, the last such row on a tie.

| Line | Date | Number | Bar | Verdict |
|---|---|---|---|---|
| data-tokenizer | no rows yet | | | |
| float-training | 2026-10-25 | 19 of 19 CPU tests pass (`5` lib contract tests, `2` dependency/D29 entry-point guards, `12` finite control tests covering all 24 kernels); 0 `uor-r4-training` or `candle-nn` edges in `cargo tree`; 0 D23 sites and 0 D25 sites in `crates/uor-r4-float-train/`; stub-`nvcc` `--features cuda --all-targets` exits 0 | Standalone crate builds CPU-only with CPU tests, passes the banned-token scan under its own scanned entry (`D37_SCANNED_FLOAT_CRATES`), carries the 24-kernel extract with 0 dependency edges on `crates/uor-r4-training`, and passes the stub-`nvcc` `--features cuda` type check (`#2823`) | positive (finite CPU controls and static module boundary only; GPU qualification and W4 training are separate downstream items) |
| harness-governance | no rows yet | | | |
| legal-construction | 2026-10-24 | Speed-up only: Unit 11's rational outputs reproduced bit-for-bit, `solve_ms 69,140`, `disjunctive_ms 146,897` (Unit 10: `264,832` / `1,116,550`). With the sort fix: tree lower bound `≈ -21.02881252`, gap `≈ 0.00685092` (`0.03259%`), `6,406/28,800` codes eliminated, 115 fixed, 186 binary-or-fixed; `solve_ms 57,057`, `disjunctive_ms 148,896` (taken under machine load) | none registered (instrument run on the saved guard-constrained linear problem, not a panel score) | **certified discrete gap narrowed to `0.006851` (`0.03259%`); `disjunctive_ms` cut 86.66% from Unit 10 with exact outputs** — item stays ready until the gap is closed |
| m1-discrete-attention | 2026-10-25 | `10 / 10` `position_mechanism` unit controls pass, `10 / 10` `order_slice` frozen-slice / `C_shuf` / paired-McNemar scorer controls pass, and `#3205` RTX 5090 finite CUDA qualification passes (`72 / 72` leaf traces matched across modes `1..=3` in `Flock` and `Relation`, `status 10` CUDA refusal on step `16`, `-2,896` B stack compaction to `22,080` B with `0` spills, `305,795` / `295,120 tok/s`). Pre-registration sheet receives fresh-context non-author `TEST FITNESS: FIT` and D68 decision review across 3 arms (`H_pos`, `C_match`, `C_shuf`) and 2 seeds | All finite CHARTER §3 controls pass and the 3-arm A8 proxy trunk order sheet receives non-author `TEST FITNESS: FIT` (`H_pos(ordered)` beats `C_match(ordered)` by $b - c \ge \max(24, \lceil 2\sqrt{b+c}\rceil)$ with $\Delta\text{admitted} \ge 0$ and `H_pos(ordered)` vs `C_shuf` drops by $\ge F_I$ on both seeds across full DEV `14,287`, `P_succ` `3,847`, and frozen order slice `1,620`, plus fresh-V2 replication $b - c \ge \lceil 2\sqrt{b+c}\rceil$) | positive (mechanism, finite CPU/GPU controls, paired order-slice scorer, and pre-registration complete; closes `#2843`; the registered two-seed GPU order training run `H_pos` vs `C_match` and `C_shuf` remains `NOT_RUN`) |
| m1-prime-zeta | 2026-10-14 | 4,096 of 4,096 tokens get distinct vectors (0 collisions, 256 of 256 writer cells occupied); 17 of 17 base and 6 of 6 prime-zeta CUDA controls pass; 0 updates applied, 0 corpus rows | Bit-exact table across seeds 0, 1 and 2; no floating-point instruction in the GPU code; 0 collisions over all 4,096 tokens; 17 of 17 base controls; the codebook must separate structurally from both nulls | positive (construction only; the learned comparison against the nulls is deferred to a training sheet) |
| m2-grounded-reply | no rows yet | | | |
| m3-durable-memory | 2026-10-25 | **896,898 tokens/s** (732 turns, 4,280 words, 4.8 ms upload to download). Panel: false_assert **0/50**, false_overwrite **0/50**, clean **50/50**. R1 **462/462** exact. R2 **0/220** (every miss `Unresolved`, no wrong write). Finite CUDA controls 12/12. A second build's decisions are byte-identical (`d3f05ecf…`) | Owner bar (#2028 comment `6356034555`): 0/50, 0/50, 50/50, counted only together with R1 ≥ 95% (decision review on #2856). R2 has no bar | positive, at this scope only: the sieve compiler on world=v2. It is not a pass of `m3-memory-turn-discriminator-gate` |
| m4-serving | 2026-10-26 | **Control:** the exported container's integer session matches the quantized float reference on all 256 single tokens and an 8-token sequence: same kept cells, same token, Q12 query within 2 LSBs, with 0 near-tie exceptions in 264 positions. **Float reference:** its plain and quantized forms agree on 224 of 256 tokens. **Audit:** zero-multiply `FULL PASS` on the rlib (40 call-graph functions) and on `uor-r4-stack` (97). | the integer forward reproduces the float reference on every input of a small vocabulary (the item's step-2 finite control); the audits stay green | positive (finite controls only: not an export gap of a trained model) |
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
