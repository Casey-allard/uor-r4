# ORDER: prove and correct the served Hamming candidate range, deepseek-6

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `a2d318159`.

## Goal

Resolve the Hamming "off-by-one" reported by the exact-head review of PR #3560. Prove the candidate range with independent integer and CPU tests, correct the served-forward contract wording and evidence, and leave the shared serving implementation unchanged if, as the source indicates, it is already causal and correct.

The owner's follow-up direction, 2 November 2026, authorizes this cross-track verification. It does not authorize changing the shared kernels unless a failing independent control demonstrates they are wrong.

## Cancel probe

Read the float `is_candidate`, integer and CPU `nearest`, their callers, and the existing tie tests. Reconstruct the payload position supplied to each candidate. Stop the kernel-repair branch if every path includes the immediate preceding payload and excludes the current row; correct the audit instead.

## Scope

- `crates/uor-r4-cpu-train/**`
- `crates/uor-r4-integer/src/stack/hamming.rs`
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-6/**`
- `docs/research/cpu-training-program.md`
- `docs/research/notebook/cpu-training.md`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/cpu-training-pilot.md`

No float training or serving kernel logic is expected to change. The float
file receives one predicate regression test only.

## Done when

- The row normalization is stated explicitly: at zero-based current row `r = tokens.len() - 1`, payload candidates are `1 <= s < r`; candidate `r - 1` is the immediate preceding payload, candidate 0 has no predecessor, and current row `r` is excluded.
- Independent tests in the integer and CPU Hamming modules pin the candidate set at a concrete row and distinguish the immediate predecessor from the current row.
- The exact-FBS contract no longer describes the Hamming range as defective.
- Modelled Hamming candidate counts still match the proven range and tests cover copy-only and typed heads.
- Corrected evidence is added without rewriting the already-merged evidence files; the new REPORT explains the superseded narrative.
- Focused tests pass, changed lines follow their files' existing style, the CPU crate's formatter check passes, and CPU-crate Clippy passes. If integer/float crate-wide formatting or Clippy still reports pre-existing debt outside the changed Hamming tests, record it rather than rewriting unrelated code. If a shared kernel defect is found instead, stop before changing it and request an integer-serving decision.
