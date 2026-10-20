---
id: m1-faster-writer
title: Replace the full-vocabulary writer scan
milestone: M1
area: mechanism
status: ready
open_to: lab
needs_gpu: true
depends_on: []
---

## Why it matters

The writer is the step that picks one token from the read's integer vector: it admits the K nearest of 256 address cells, then takes the nearest token by L1 distance. Today it scans the whole vocabulary, which breaks the per-token sparsity rule (D5-b, CHARTER §3.9) once the vocabulary grows past 4,096 pieces. A cheaper writer is a precondition for any larger vocabulary and for the end-state serving claim. See [admission and the writer](../../mechanisms/admission.md).

## What is missing

The writer scans the whole vocabulary: about 5×10⁶ vocabulary-cell comparisons per row (the project's own figure). The chief architect's review (§3.5) gives the cost as O(K·256) cell scans plus O(K·V) membership tests plus distances. It calls the O(V) scan the real serving-cost defect, and notes that no D37 mechanism touches it. This is a proof by count of comparisons per row, not a timing. A bounded K limits the shortlist, not the work to build it.

No faster writer design exists. It needs new kernel work in `crates/uor-r4-discrete/src/discrete.cu` (the writer is at lines 307-345) and a fresh CUDA qualification.

## Done when

A writer whose measured per-row comparison count is below the full-vocabulary scan (about 5×10⁶ today), with finite controls that match the full-scan writer, and a CUDA qualification. How far below the scan counts as enough is not yet registered; register it in step 4.

## How to start

1. Read the writer as it stands. Open `docs/labs/course-correction-2026-10-18/audits/d37-reactivation-architect.md` (§3.2 and §3.5), `docs/mechanisms/admission.md`, `docs/mechanisms/tokenizer.md`, and `crates/uor-r4-discrete/src/discrete.cu` (`address_distance` line 274, `token_cell` 286, `cell_precedes` 301, `writer` 307-345). Count the comparisons per row in the code and check that you reproduce the 5×10⁶ figure.
2. Post a design note on the issue as your first deliverable. It must give: the geometric construction that finds the nearest token without visiting every token (for example an index over the 256 address cells that visits only cells near the read; propose it, the sources do not name one); whether the output must equal the full scan exactly or may differ, and how a difference is measured; the comparison count as a formula in K and V; and how it uses only integer comparisons and L1 distances, so it needs no softmax, no squashing and no matmul.
3. Write finite CPU controls first, in Rust. Add a host reference of the current full-scan writer and tests in `crates/uor-r4-discrete` that compare the new writer with it on small vocabularies, including every tie (see `cell_precedes`) and a counter test for the comparison count. Run `~/.cargo/bin/cargo test -p uor-r4-discrete`. Use `crates/uor-r4-discrete/src/rows_qualification.rs` and `bvgk_qualification.rs` as the pattern for a qualification module.
4. Draft the one-page pre-registration sheet in `docs/harness/prep/`, using `docs/harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md` as the format model. State the bar (comparisons per row on the pinned 4,096-piece vocabulary and on a larger vocabulary), the controls, the cost and what each outcome decides. A non-author must give TEST FITNESS: FIT before any GPU run (CHARTER §4.5).
5. After FIT, build the kernel ahead of time with nvcc through `crates/uor-r4-discrete/build.rs`, in its own module (NVRTC is banned; CUDA is prebuilt by nvcc, CHARTER §3.4 and `docs/model/gpu-and-cuda.md`). Run the finite CUDA qualification on a GPU, through `scripts/pod/uor-pod` or your own GPU with the hardware specs attached to the PR (`uor-pod status` first).
6. Open a PR that says `Closes #<this issue>` when the qualified writer is in, or `References #<this issue>` for a partial piece such as the note, the controls or the sheet. Add one row to `docs/research/notebook/m1-discrete-attention.md`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:170`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:19`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/model/architecture.md:68`](../../../docs/model/architecture.md)
- [`docs/research/open-questions.md:88`](../../../docs/research/open-questions.md)

