# REPORT: exact raw FBS parity on the public trained pair, deepseek-4

Lab: deepseek. Item: [cpu-shared-vector-train](../../../../roadmap/items/cpu-shared-vector-train.md) (#3505). Base: `52ddbb0b0`.

## What did not work, first

- **The CPU forward refused the only public trained FBS pair.** Both dual-c131 artifacts carry raw Hamilton gates and a raw recurrence transition, so the earlier snapped-only implementation could not run the trainer, counterfactual scorer or panel evaluator on either public seed.
- **No trained float source is published with the integer pair.** This unit proves exact integer CPU/serving parity and executes finite update controls on genuine trained state; it does not reconstruct or compare the original continuous training shadow.
- **A random-target panel admitted no rows.** Choosing arbitrary targets produced zero admitted samples. The final control chooses an admitted writer rival for each input before training, so its 16/16 admission reading is a component fixture, not a language score.
- **The release binary initially measured setup and training with prefix throughput.** The final runner reports prefix time separately.

## What changed

- Added an independent integer `raw_rotation` implementation matching serving's unsigned square root, truncating division, wrapping multiplication and shift.
- Raw transitions use epsilon `4_295`; raw Hamilton gates use epsilon `0`. Both recurrence gates and feed-forward gates honor `raw_gates`.
- All four snapped/raw gate and transition combinations match `IntegerStackModel::session().step_native` over complete Q12 query, token and kept-cell vectors.
- Raw mode composes with Hamming reads and exact turn-memory/point fallback.
- The FBS session checkpoint schema is now `/2` and binds both raw flags in its canonical shape.
- Added `cpu-public-fbs-parity`, which pins both public dual-c131 SHA-256 values, checks 192 positions per seed, and executes 16 admitted rank-margin/panel control samples per seed.

## Measurements

Public dual-c131 on the Ryzen 9 9950X:

| Seed | Artifact SHA-256 | Prefixes | Prefix time | Rank-margin controls | Panel |
|---|---|---:|---:|---:|---:|
| 1 | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` | 192 | 77.977 ms | 16/16 updated | baseline and updated: 16/16 admitted, 16/16 top-8, exact export |
| 2 | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` | 192 | 74.940 ms | 16/16 updated | baseline and updated: 16/16 admitted, 16/16 top-8, exact export |

The prefix timer includes both independent CPU stepping and the serving reference used for comparison. It is not a pure decode benchmark and does not establish corpus training throughput.

## Checks run at the head

- `cargo test --release --locked -p uor-r4-cpu-train`: PASS after the known duplicate-artifact retry, `98 passed`, `4 ignored`.
- Focused FBS suite: `29 passed`.
- `cpu-public-fbs-parity`: PASS on both public seeds with the readings above.
- Strict clippy, formatting and D23/D25/D32 ratchets: PASS.
- Local PR check and all 11 loop gates: PASS.

## Skills used

No repository skill was invoked. The exact serving rotation and public fixture identities were audited directly before implementation.

## Still open

- A trained float shadow paired with these integer artifacts.
- A masked corpus driver and complete-reply/fresh-phrasing evaluation.
- The pre-registered bounded pilot and final CPU training disposition.

## Plain summary

The CPU trainer now runs both published trained FBS models exactly, including their unsnapped arithmetic. It can perform finite admitted rank-margin updates on those real artifacts without an export mismatch, but this is still not a useful-learning or complete-reply claim.
