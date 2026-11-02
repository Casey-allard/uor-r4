# ORDER: exact raw FBS parity on the public trained pair, deepseek-4

Lab: deepseek. Item: [cpu-shared-vector-train](../../../../roadmap/items/cpu-shared-vector-train.md) (#3505). Base: `52ddbb0b0`.

## Goal

Make the independent CPU FBS forward execute the integer serving path's unsnapped Hamilton gates and recurrence transition exactly, then prove the path on both public dual-c131 trained artifacts and exercise the existing rank-margin and panel controls on genuine trained state.

## Scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-shared-vector-train/deepseek-4/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-shared-vector-train.md`

No serving edit, corpus-scale run, GPU run or language claim.

## Done when

- CPU quaternion normalization reproduces the serving `stack_rotation` order: unsigned square root, truncating division, wrapping product and shift, with transition epsilon `4_295` and Hamilton-gate epsilon `0`.
- Snapped and raw gate/transition modes compose in all four combinations while unsupported recurrence, pointer and read patterns remain refused.
- Synthetic parity compares every complete Q12 query, top-1 token and kept-cell vector against `IntegerStackModel::session().step_native` in all four combinations.
- Negative controls prove each raw flag changes at least one decision and neither flag is silently ignored.
- Raw mode composes with merged Hamming and exact-memory state without changing their parity.
- A public fixture runner pins both dual-c131 artifact SHA-256 values and compares long prefixes on both seeds.
- At least one finite rank-margin and panel-control step executes on each public seed; this is component evidence, not useful learning.
- The FBS session checkpoint binds raw/snapped gate and transition identity.
- Full release tests, clippy, formatting, D23/D25/D32 ratchets, local PR check and all loop gates pass.
- A fresh exact-head adversarial review approves the change before merge-queue delivery.
