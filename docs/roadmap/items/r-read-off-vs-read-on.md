---
id: r-read-off-vs-read-on
title: Compare read-off and read-on for Route and Flock on checkpoints
milestone: research
area: evaluation
status: ready
open_to: lab
needs_gpu: true
depends_on: []
scope: [crates/uor-r4-discrete/**, docs/harness/prep/r-read-off-vs-read-on*, docs/labs/r-read-off-vs-read-on/**]
---

## Why it matters

Route is the base read of every trained artifact, and Flock runs inside every Relation arm, yet nobody has switched either one off to see what it adds. This comparison decides whether the two reads are worth carrying into the trunk, or whether the work should go to the address (admission, codebook) and the writer instead. It is a no-training test on checkpoints that already exist. See [the Route read](../../mechanisms/route-read.md) for how the read works.

## What is missing

Whether the Route and Flock reads add anything is untested. Route got only 20 proposals in 16,404 updates (0.12%; 16 of the 20 were accepted), so its own effect is unattributed ([attention state](../../labs/attention-state-2026-10-18/README.md)). The mechanism board lists the next step as a "Read-off versus read-on test with enough proposals". Flock passes its 6 finite controls, but its own value was never isolated; the two October control runs lost admission to learning moves before the read could be judged.

This is a no-training comparison on existing checkpoints. Two gaps stand in the way:

- **No read-off switch for Route.** `ReadSpec` in `crates/uor-r4-discrete/src/lib.rs` has only `Route`, `Flock { kept }` and `Relation { kept, sweeps }`, and `read_vector` in `crates/uor-r4-discrete/src/discrete.cu` dispatches only between them. There is no "no read" mode. A Route bypass is new kernel code and needs its own finite CUDA controls.
- **Flock has a zero-gate arm.** In `discrete.cu`, a gate of 0 leaves the output unchanged, and a Relation model with all relation dials at zero is bit-identical to Flock ([Flock read](../../mechanisms/flock-read.md)). So the Flock read-off arm may need no new kernel code, but confirm this on a finite control.

The checkpoints must carry zero accepted updates, because `eval-corpus` in `crates/uor-r4-discrete/src/main.rs` refuses any checkpoint with accepted updates (`Error::TrainingHeld`). That limits which retained checkpoints can be used, and it limits what "same retained checkpoints" can mean. Settle this in the design note.

## Done when

Paired read-on versus read-off results on the same retained checkpoints are recorded as notebook rows, with the classification instrument versus mechanism stated. The bar for a "read adds something" verdict is not yet registered; registering it, with the score it is read on, is part of How to start. For scale only: on the 14,287 development rows a context-free successor table scores 6,300 top-1, and the trained read scored 149.

## How to start

1. **Pick the checkpoints and write down what is retained.** Read `docs/labs/attention-state-2026-10-18/README.md` and the K16 and A8 records it links, and list which checkpoints are on disk or in the HF store, their read mode, and whether each has `accepted_updates == 0`. Open `crates/uor-r4-discrete/src/main.rs` (the `eval-corpus` branch, near line 102) to confirm the refusal rule. If every retained checkpoint has accepted updates, say so on the issue and ask the PM whether a zero-update checkpoint or a new driver path is the intended input. Do not guess.
2. **Post a design note on the issue.** It must give: (a) the read-off arm for each read, for Route a bypass that returns no routed vector, and for Flock the zero-gate arm; (b) why neither arm uses softmax or a matrix product (both reads use gather, sign flip, compare and integer add, as in `docs/mechanisms/route-read.md` and `docs/mechanisms/flock-read.md`); (c) finite CUDA controls for the new switch, in the style of `crates/uor-r4-discrete/src/qualification.rs` (11 controls) and `crates/uor-r4-discrete/src/flock_qualification.rs` (6 controls): read-on output unchanged by the switch, read-off output exactly the bypass, and a refusal if the switch is set in the wrong read mode; (d) the instrument-versus-mechanism question: what in the result would be a fault of the harness (same driver hash, same rows, same admission) and what would be about the read.
3. **Draft the pre-registration sheet.** Use `docs/harness/prep/m1-attention-address-structured-cells-DRAFT-2026-10-19.md` as the template. Fix the arms (read-on, read-off, per read), the checkpoints with their hashes, the dev rows (14,287; the 3,847 `P_succ` subset), the metrics (top-1, top-8, admission) and the bar. Add a read-on control with a random or shuffled read if it can be built from the project's geometry. Request a non-author TEST FITNESS: FIT review before anything runs (CHARTER §4.5).
4. **Build the switch and the battery script.** Add the Route bypass in `discrete.cu` and the finite controls, and extend `ReadSpec` or the init spec only as the design note says. Use `docs/labs/address-cells-preflight-2026-10-19/evidence/asc-battery.sh` as the model for a Rust-driven battery: pinned input SHAs, an exclusive result directory, and hashes of every output in a `FILES.sha256`. Per the charter, do not add new Python. CPU builds and static checks run on the laptop; run `~/.cargo/bin/cargo check -p uor-r4-discrete` there, and build CUDA kernels ahead of time with nvcc on the pod (no NVRTC).
5. **Run only after FIT, on a leased GPU.** Use `scripts/pod/uor-pod status` first, lease the GPU, and run `eval-corpus` once per arm and checkpoint: `~/.cargo/bin/cargo run --release -p uor-r4-discrete --features cuda --bin discrete-stack -- eval-corpus CONFIG.json CHECKPOINT.json NEW_REPORT_DIR`. Save results off the pod before `down`. If you use your own GPU, attach the hardware specs; a non-5090 result is re-run by a maintainer under the same FIT.
6. **Record the result.** Add one row per paired comparison to `docs/research/notebook/m1-discrete-attention.md`, with the verdict (positive, negative, inconclusive or diagnostic-stop) and the instrument-versus-mechanism class. A negative closes a configuration, never the mechanism. Keep a dated evidence folder only if the run seals artifacts.
7. **Open a PR** that says `Closes #<this issue>` when both reads are compared on the same checkpoints, or `References #<this issue>` if only one read, or only the design note and sheet, is done.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:38`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:89`](../../../docs/mechanisms/README.md)
- [`docs/model/architecture.md:43`](../../../docs/model/architecture.md)
- [`docs/research/open-questions.md:44`](../../../docs/research/open-questions.md)
- [`docs/mechanisms/route-read.md`](../../../docs/mechanisms/route-read.md)
- [`docs/mechanisms/flock-read.md`](../../../docs/mechanisms/flock-read.md)
- [`docs/labs/attention-state-2026-10-18/README.md`](../../../docs/labs/attention-state-2026-10-18/README.md)
- [`crates/uor-r4-discrete/src/discrete.cu`](../../../crates/uor-r4-discrete/src/discrete.cu) (read dispatch, lines 268-273)
- [`crates/uor-r4-discrete/src/main.rs`](../../../crates/uor-r4-discrete/src/main.rs) (`eval-corpus`)
