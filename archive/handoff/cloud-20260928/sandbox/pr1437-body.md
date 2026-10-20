**Carries the cycle-4 main comparison's result; the cycle-5 experiment is implemented and held (NOT_RUN).** References #820 and #973. Follows #1414.

## Cycle-4 main comparison (result)

*Measured.* One seed per arm and 7,324 updates (29,999,104 target visits). The final score is on the 512 evenly spaced development windows (131,072 targets).

| Arm | Parameters | NLL (nats) | Bits per byte | Train tokens/s |
|---|---:|---:|---:|---:|
| Geometric stack (`rrarra`, Lorentz reads, rotation) | 7,153,860 | **1.998113** | 0.803432 | 789.3 |
| #1017-shape transformer control | 7,155,360 | 2.011149 | 0.808673 | 788.1 |

- **Reading by the card's frozen rule: viable.**
  - The stack is 0.0130 nats below its control, at equal speed.
  - With one seed per arm, this is parity with an equal-size transformer, not an advantage.
  - The director's D0 decision (#820) adopted the stack as the main-line core. Its next step is export to the native, multiplier-free bundle (I2, R1–R5).
- **Records.** They are in [cycle 4](docs/integration/geometric-stack-cycle4-2026-09-27.md) §0, §6, §9 and §10, and in the evidence packet's `main/` directory (schema /5, all hashes verified).
- **Weights.** They stay outside `main`, pinned by SHA-256, on the temporary orphan branch `transfer/cycle4-main-20260928` (to be deleted after the owner's copy). It holds both final models, the step-7,300 checkpoints and the evaluation inputs.
- **Provenance.** Only the last of three attempts counts. A container rollback lost the first. The second trapped when the sandbox moved to a different CPU.
  - The counted run is `b87acd63` rebuilt with `-C target-cpu=x86-64-v3`, resumed from its 250-update checkpoint.
  - On that build, a stopped-and-resumed run gives the same model SHA-256 as an uninterrupted one.
- **Continuations.** They are still not useful code, in either arm.

## Cycle 5: product-key memory layers (implemented, NOT_RUN, held)

**Why.** Every model so far reads its whole weight store per token. D5 makes per-token sparsity the end-state serving contract, and names the contest that has never run: geometric addressing (H4, E8) against learned indexes at a matched access budget.

**What is added.**
- **`crates/uor-r4-training/src/stack_memory.rs`: a product-key memory** (Lample et al. 2019). It replaces the MLP of chosen layers.
  - Per head, the query's two halves score two sets of sub-keys.
  - The top `k` of each side form `k²` candidates. The best `k` of one shared value table of `S²` rows are mixed by their softmax weights.
  - One fused CPU op with an exact backward holds the selection fixed; ties break toward the lower index.
  - The index geometry is the variable: learned sub-keys with the Dot or the Lorentz score, or fixed codebooks.
  - The fixed codebooks are the 120 unit icosians (600-cell, H4) for quaternion halves, or the 240 E8 roots for 8-dimensional halves. Their keys never learn.
- **`geometric_stack.rs`**: `StackConfig.memory`, the memory's parameters, forward pass and active-parameter count. Configurations without a memory are unchanged.
- **`StackAdamW`**: now one parallel pass per variable, performing the same f32 operations in the same order. A test checks the step bit for bit against the old composition.
- **The example**: `memory_*` options, and a report of the parameters read per token.
- **Guard**: integer export refuses memories.
- **`stack_mlp=`** (`4eef03e6`): pins a geometric stack's MLP width instead of matching the control's parameter count. This lets two stacks differ in one component at equal width. The control keeps `mlp=` and refuses `stack_mlp=`.
  - The transport attribution D1 (#1453 §4, work card on #973) uses it. D1 is running in the lab sandbox now, and its reading follows on this PR or its successor.
- **Plan**: [memory-layers-cycle5](docs/integration/memory-layers-cycle5-2026-09-27.md) records five pre-registered arms with 0.02-nat decision rules.
  - Serving (§3) follows R1–R5.
  - The experiment is held by the director plan: no new learner starts until the base decision.
  - The D0 decision supersedes its reads-only base. If it runs, its base is the `rrarra` stack as settled by D0 and the transport attribution D1, and §4 is amended first. Arms on the reads-only base count only as comparator evidence (ruling 8).

## Validation

- **Unit tests.** The stack, memory, export, dialogue and tracking tests pass on the merged head. They include:
  - product-key selection equals a brute-force top-`k`;
  - gradients match finite differences on every coordinate;
  - both codebooks have their defining inner-product sets;
  - codebook keys stay fixed while the rest trains;
  - memory stacks are causal and round-trip through save and load;
  - the fused optimizer step is bit-identical.
- **Formatting and claims.** `cargo fmt --check` is clean, and the claim-wording gate passes.
- **`stack_mlp=`.** A release build's two-update runs save `mlp_hidden` 749, with 7,153,860 parameters (rotation) and 6,820,932 (identity). `arch=transformer` refuses the option.
- **Merge with `main`** (`30cf13d3`). The merge brings in B1's `stack_tracking`, D11 (#1443), #1433 and Lab 3's serving work.
  - The one textual conflict was module declarations in `lib.rs`.
  - The tracking code's `StackConfig` literals gain `memory: None`, which leaves their behaviour unchanged.
  - After the merge, 35 focused tests pass, and `cargo check --all-targets` and `cargo fmt --check` are clean.

## Note on provenance

A container restart on 09-27 rolled the lab sandbox back to its 08:24 UTC state and lost this branch, which existed only locally.
- This branch replays the session's recorded edits in order onto the same base.
- The rebuilt diff matches the recorded one line for line.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_018jzJYgXbX8FmctqekthjLJ
