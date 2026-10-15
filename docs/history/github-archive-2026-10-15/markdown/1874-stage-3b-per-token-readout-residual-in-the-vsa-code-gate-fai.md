# #1874 — Stage 3b: per-token readout residual in the VSA code (gate FAILED: ceiling removed, term still inert) (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T04:10:58Z
- **Updated:** 2026-10-09T13:22:34Z
- **Closed:** 2026-10-08T04:12:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1874
- **Merged:** 2026-10-08T04:12:37Z
- **Merge commit:** ae4f3b5b155362e0e9b19abfde172a6c381016c4
- **Base / Head:** main / UOR-Foundation:deepseek/stage3b-vsa-residual
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this does

`learner/vsa_codes.rs` states the ceiling behind the Card P7 VSA verdict: codes are a function of the learned root, so a 4,096-token vocabulary collapses onto at most 120 distinct codes and two tokens in one root have `d_H = 0`. It names the remedy — codes at finer resolution from the continuous learned representation — and Stage 3(b) of `native-core-transition-plan.md` specifies that change together with its gate.

**Mode `2` (`learned`)** implements it:

```
code(token) = code_for_root(token_to_root[token]) XOR code_for_readout(readout[token])
```

`code_for_readout` locality-sensitive-hashes the token's own `discrete_s2_readout` (five Q1.14 dims: three base S2 plus two U(1) fiber) with deterministic integer hyperplanes from a seed domain disjoint from the root projection. The XOR is the VSA bind: equal roots with different readouts no longer collide at `d_H = 0`, while root differences still dominate across roots.

**No artifact format change was needed**, contrary to the plan's assumed `SECTION_VSA_CODES`. The per-token vector the residual needs is *already* carried as `discrete_s2_readout`, and mode 1's precedent rebuilds the code space on load. Mode 2 is size-compatible and declarable through `set-vsa-code-mode --mode learned`.

## The gate: FAILED

Artifact `native_geometric_prose_model.rgm` sha256 `7023c450…`, corpus `tinystories_train.u16`, discrete additive scorer, 8,127 scored positions per slice:

| slice | mode | distinct codes | baseline BPB | vsa delta | band |
|---|---|---:|---:|---:|---|
| offset 0 | fixed | 0 (on demand) | 1.8964 | −0.0010 | NEGLIGIBLE |
| offset 0 | root | 120 | 1.8969 | −0.0015 | NEGLIGIBLE |
| offset 0 | **learned** | **4096** | 1.8967 | −0.0013 | NEGLIGIBLE |
| offset 100M | fixed | 0 | 1.8783 | −0.0015 | NEGLIGIBLE |
| offset 100M | **learned** | **4096** | 1.8785 | −0.0017 | NEGLIGIBLE |

The aliasing ceiling was real and is now **removed** — 4,096 distinct codes where the root codebook can produce at most 120, asserted by tests. Held-out BPB does **not** improve, and the `vsa` ablation does **not** stop being ≈ 0: it stays within 0.002 BPB and is slightly negative in all five runs, top-1 unchanged within 0.09 pp.

**Conclusion:** at this resolution, on this artifact, codebook aliasing was not what made the VSA term inert. The plan's own Stage 4 note names the deeper limit — the serving scorer is purely additive, a log-linear model over sparse discrete features, and a better code space cannot give an additive sum a feature interaction it does not have.

## Scope, stated to prevent over-reading

- One artifact, one corpus, two 8,256-token slices, temperature 1.0, debug build. No training, no fitting, no serving default changed.
- The per-token vector used is `discrete_s2_readout`, the strongest per-token learned vector the artifact carries — a proxy for the training-time embedding, which the artifact does not store. So this refutes "finer codes alone fix this term at the resolution available", **not** "no codebook from the true continuous embedding could ever help".
- The plan's gate adds "if the heads still contribute nothing with a learned codebook, delete them rather than carry them". That deletion verdict is **not** taken here: it needs the true embedding and a decision about the routed heads, neither of which this sweep measures.
- Not a verdict on routing: September's record measured the learned-root codebook improving `shortlist-recall` recall 8.4–8.8 % → 10.7–11.2 %.

## Verification at the exact head

- `cargo check -p uor-r4-core --all-targets` — clean.
- `cargo test -p uor-r4-core vsa` — **26 passed / 0 failed**, including the new `readout_residual_breaks_the_120_code_ceiling`.
- `rustfmt` clean on every touched file. Defaults unchanged: mode 0 and 1 behave exactly as before.

Evidence document: `docs/evidence/native_geometric_vsa_stage3b_residual_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

