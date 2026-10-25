---
id: float-training-module
title: Create the separate scanned D37 float training module
milestone: infra
area: architecture
status: done
open_to: lab
needs_gpu: false
notebook: docs/research/notebook/float-training.md
depends_on: [owner-decision-float-path-scope]
scope: [crates/uor-r4-float-train/**, Cargo.toml, Cargo.lock, crates/uor-r4-training/**, crates/uor-r4-harness/src/ratchet.rs, docs/harness/prep/float-training-module*, docs/labs/float-training-module/**, docs/research/notebook/float-training.md, docs/roadmap/items/float-training-module.md]
---

## Why it matters

It is the prerequisite for W4 and for any trunk built from lawful parts (Stage 3). D37 makes floating point legal in offline training only, with a reported export gap. This module is the one place that float code may live. See [the geometric stack](../../mechanisms/geometric-stack.md) for the parts a trunk would be built from. No trunk of lawful parts has been trained yet.

## What is missing

D37 float training is legal but unused. It must live in a separate module with its own scanned allowlist entry (D37 item 6), never `crates/uor-r4-training`, which carries the historical dense and sigmoid stack and is listed as held. No such module exists, and the 24-kernel extract is qualified at finite controls only. The chief architect's review names the likeliest mistake: reusing `crates/uor-r4-training` because D29 counts presence and D37 asks for a separate allowlist entry. Its guard is a distinct crate plus a `uor-harness` check that the float build graph excludes `uor-r4-training`.

**Scope (25 Oct 2026, claude-101).** The module's own crate, `crates/uor-r4-float-train/`, is now in `scope:`, with the root `Cargo.toml` and `Cargo.lock` for the workspace member, and the new notebook page `docs/research/notebook/float-training.md` for its row. The unit that creates that page also adds the `notebook:` line. This item file is in its own `scope:` for that, because `loop pr-check` otherwise allows only status edits to it. `loop guard` reads scope from `origin/main`, so without this the crate the done-when asks for would be refused. `crates/uor-r4-training/**` stays in scope as before, and the new crate must not depend on it. The unit must not change other crates' entries in the root `Cargo.toml` or `Cargo.lock`.

## Done when

A new module builds CPU-only on the laptop with CPU tests, passes the banned-token scan under its own scanned entry, and carries the 24-kernel extract with no dependency on `crates/uor-r4-training`. The CUDA kernels are built ahead of time by nvcc on the GPU pod (CHARTER §3.4); on the laptop they get a stub-nvcc type check only.

## How to start

1. Confirm `owner-decision-float-path-scope` is recorded; the scope of the float path is the owner's call. Read D37 in `docs/integration/DECISIONS.md` (it points to CHARTER §3.7), then `docs/labs/course-correction-2026-10-18/audits/d37-reactivation-architect.md` (the W4 section and risk 1 at its line 179) and `docs/mechanisms/geometric-stack.md`.
2. Post a design note on the issue first. It must give: the crate name and its place in the root `Cargo.toml` members list; the list of the 24 kernels in the extract and where their source comes from (start at `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/geometric-stack.json`; the extract today lives in `crates/uor-r4-training/src/geometric_stack_native.rs` and `geometric_stack_native_cuda.rs`, which the note must show entering the new crate without a dependency edge back); how the extract enters the new crate with no dependency edge on `crates/uor-r4-training`; and why nothing in it needs softmax, squashing or dense matrix multiplication.
3. Find how the banned-token scan is keyed. The ratchet in `crates/uor-r4-harness/src/ratchet.rs` scans `crates` and `docs/labs/*/instrument` and compares per-file site counts with `scripts/no-softmax-baseline.json`, `scripts/no-matmul-baseline.json` and `scripts/no-discrete-float-baseline.json`. A file absent from a baseline with any hit fails, and `--update-baseline` refuses to raise a count. Run `uor-harness ratchet d23` and `uor-harness ratchet d25 --list` (`--list` prints sites for d25 and d32 only). `scripts/check_no_softmax.py` and `scripts/check_no_matmul.py` are the Python scripts the harness ratchet mirrors. The design note must say how D37 item 6's "own scanned allowlist entry" maps onto this. Add one entry for the new crate and no other, and no new Python.
4. Write a failing test first. The guard from the architect's review is a CPU check that `~/.cargo/bin/cargo tree -p <new-crate>` lists no `uor-r4-training`. Add finite CPU controls for each kernel in the extract, so the module is qualified at the same finite level as before and no further.
5. Build and check: `~/.cargo/bin/cargo build -p <new-crate>`, `~/.cargo/bin/cargo test -p <new-crate>`, `uor-harness ratchet d23`, `uor-harness ratchet d25 --list` and `uor-harness check`. Build CUDA kernels ahead of time with nvcc on the pod only; never NVRTC. On the laptop use a stub-nvcc `--features cuda` type check, as in `docs/labs/bvgk-c2-block-apply-2026-10-18/README.md`. GPU qualification is out of scope here.
6. Open a PR that says `Closes #<this issue>`, or `References` if partial. No existing notebook covers the float path, so add one row on a new page `docs/research/notebook/<line>.md` in the format of `docs/research/notebook/README.md` and name the line in the design note.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:112`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:130`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:53`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
