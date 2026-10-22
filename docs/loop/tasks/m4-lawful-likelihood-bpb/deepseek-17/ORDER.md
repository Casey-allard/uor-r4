# ORDER: m4-lawful-likelihood-bpb, deepseek-17 (implementation unit: the host-only scorer and its finite controls)

Lab: deepseek. Item: [m4-lawful-likelihood-bpb](../../../../roadmap/items/m4-lawful-likelihood-bpb.md) (#2861). Base: `3120b01c5`.

## Goal

Implement the potential-ratio lawful likelihood the merged design note specifies (`docs/labs/m4-lawful-likelihood-bpb/README.md`, PR #2940) as a host-only pure module, with the seven finite host controls. The registration unit (held-out stream name/hash, pre-registration, non-author FIT, registered GPU scoring) is out of this unit; the PR says `References #2861` and the item stays `ready`.

## History

- deepseek-8 `0a4549689` added the design note; Claude CHANGES at `e9748f1`; deepseek-9 fixed the zero-mass/bits/division/crate/geometry items; fresh-context CHANGES at `effe1e9c1`; deepseek-10 fixed conditional lawfulness, the rank-weight precedent and the emissions claim; the owner answered Card 1 ("Yes, a linear ratio (Recommended)", #2028 `2026-10-22T04:17:03Z`) and deepseek-11 recorded it; deepseek-12/13 fixed record defects and the review basis. PR #2940 merged as `cbde25bd3`.

## Cancel probe (run first)

Class **instrument** (host-only implementation; no mechanism, no training). At base `3120b01c5`: no lawful-likelihood implementation exists (`grep -rn 'lawful_likelihood\|potential.ratio' crates/uor-r4-integer/src` finds nothing), the design note is merged, and the item is `ready` and unclaimed. Two crate facts the unit must respect: the selected crate root is `src/native_geometry.rs` (D29), not `lib.rs`; and the crate's historical integration-test targets already fail to compile on `main` because the historical executors are excluded, so this unit proves itself with the lib unit tests, not those targets.

## Files in scope

`crates/uor-r4-integer/**`, `docs/labs/m4-lawful-likelihood-bpb/**`, the item's `status:` line (unchanged) and this task folder.

## Done when

- `crates/uor-r4-integer/src/lawful_likelihood.rs` implements base-1 Q48 masses summing exactly to `2^48` with u128 ratio arithmetic and largest-remainder split (ties by index), the exact MLE count baseline (zero counts stay zero and are unscorable), and integer fixed-point `log2` with the stated `2^-12 + 2^-48` bit bound.
- The module is wired into the selected crate root and the seven finite host controls pass: tiny alphabet, V=4096 worst case (minimum mass exactly 1, bits 48), uniform power-of-two, non-power-of-two ±1 remainder, token-id ties, monotonicity/argmax and permutation properties, and the analytic BPB/MLE check.
- `cargo test -p uor-r4-integer --lib` passes (42 tests, 9 of them new); `cargo fmt -p uor-r4-integer --check` is clean; clippy on the lib adds no warning for the new module; the ratchets d23/d25 are unchanged.
- The design record gains a short implementation section naming the module, its API and its limits; the item stays `ready`.
- `loop gates --full` GREEN; `report --check` and `pr-check --local` pass.

## Stop and ask if

- A banned form or a normalization table would be needed (it is not: integer add/subtract, one exact ratio, no exponential).
- A GPU or model run becomes implied (it is not in this unit), or the held-out stream must be named now (it is the registration unit).