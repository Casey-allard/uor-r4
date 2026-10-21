---
id: r-lawful-pointer-forced-copy-oracle
title: Build a lawful pointer, then run the forced-copy oracle
milestone: research
area: mechanism
status: ready
open_to: lab
needs_gpu: true
depends_on: []
scope: [crates/uor-r4-training/**, docs/harness/prep/r-lawful-pointer-forced-copy-oracle*, docs/labs/r-lawful-pointer-forced-copy-oracle/**]
---

## Why it matters

The pointer is the planned copy head: it copies a stored answer by the slot's exact name, not by the surrounding wording. Every pointer built so far is a softmax pointer inside a dense float model, and each one missed its bar or cannot be served. The forced-copy oracle supplies the true slot and bounds how much a perfect address could help, so it decides whether pointer-style copying is worth learning lawfully. See [the pointer-key page](../../mechanisms/pointer-key.md).

## What is missing

- **A lawful pointer.** Today both the copy mixture and the pointer attention use softmax, the score is a dot product in a dense float model, and export refuses a pointer with an identity term. No pointer exists on the discrete path.
- **The oracle run.** The forced-copy oracle is "named the cheapest decisive test, never run". The instrument was built on 12 October (`StackModel::set_copy_span_oracle` and `chat-grade reply copy_oracle=span`, 919 library tests passed at the time), but it exists today only as preserved historical source that the build excludes. `crates/uor-r4-training/src/lib.rs:110-111` compiles `geometric_stack_native.rs`, which has no span oracle, instead of `geometric_stack.rs`; `copy_identity.rs` and `stack_prime_route.rs` are `#[cfg(any())]` (lib.rs:36-37 and 213-214). `chat-grade reply` is retired under D24/D29 and only returns an error (`crates/uor-r4-training/src/bin/chat-grade.rs:2113-2121`), and `copy_oracle=span` survives only in its doc comment (line 13). So the oracle has to be rebuilt on the active native path. The run on the bound model was not done. Running it on the old softmax-pointer artifact would execute a banned model (D23, D30).
- **A registered bar for the new run.** The old pre-registration read three conditions: oracle off must reproduce the recorded 23 of 40 exactly, then O1 (span, learned gate) and O2 (span plus gate floor 0.9). That reading was written for the old artifact. The bar for a lawful pointer is not yet registered.

The sources disagree on one point. The audit synthesis says the oracle is interventional, so stored outputs cannot answer it. The pointer-key page's reopen list says "a softmax-free, matmul-free pointer or stored outputs only". This plan follows the synthesis. Settle it in the design note.

## Done when

A lawful pointer with finite controls, then the forced-copy oracle run once under a registered sheet that has a non-author TEST FITNESS: FIT review. The result is one row in a research notebook. The training step that follows (a slot-address key, at least 3 seeds, at least 120 memory rows, a gain of at least 5 rows of 40 over the control) is a separate item, not part of this one.

## How to start

1. Read the "How it works", "What has never been tested" and "What would reopen it" sections of `docs/mechanisms/pointer-key.md`. Then read `archive/labs/copy-span-oracle-2026-10-12/README.md` (the instrument and its pre-registered reading), `archive/labs/ptr-addr-2026-10-10/README.md` (the prime-route pointer and its bar) and `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/pointer-key.json`. Historical code to read (preserved source, not compiled today): `PointerConfig` (line 2254), `PrimeRoute` (line 2325) and `set_copy_span_oracle` (line 6647) in `crates/uor-r4-training/src/geometric_stack.rs`, plus `crates/uor-r4-training/src/stack_prime_route.rs`, `crates/uor-r4-training/src/copy_identity.rs` and the retired `reply` command in `crates/uor-r4-training/src/bin/chat-grade.rs`. The active module is `crates/uor-r4-training/src/geometric_stack_native.rs`, which has its own `PointerConfig` (line 410) and no copy-span oracle. Do not expect `cargo test --lib <filter>` to reach the historical files.
2. First deliverable, posted on the issue: a design note. It must give the geometric construction of the lawful pointer (the page's options are a rank table, a hard select, or exact admission by gcd or n-let with a learned ranking). It must show how the copy distribution loses both softmax terms in `p(v) = (1-g) softmax(z)_v + g sum a_j`, and how the score avoids a dot product and any matmul. It must list the finite host controls, for example that admission picks the right source on a fixed table of entity and attribute primes, and that the copy emits the supplied span's token. It must also say whether the pointer needs the slot-address key (entity prime times attribute prime) or only the ability to copy a supplied span.
3. Choose the sheet type and say why: a discrete sheet (D32) or a new offline float sheet (D37) in a separately scanned module. Note the roadmap item `float-training-module` if D37 is chosen. The new run needs a bound model that answers memory rows.
4. Build the pointer and its finite controls on the host. Run `~/.cargo/bin/cargo test -p uor-r4-training --lib <filter>`, then `~/.cargo/bin/cargo check -p uor-r4-training --bins` and `~/.cargo/bin/cargo fmt --check -p uor-r4-training`. Check that the lawful pointer's oracle-off path reproduces the bound model's own recorded memory score exactly.
5. Freeze the panel and write a one-page pre-registration sheet. The panel is a memory-row panel such as the one the historical `crates/uor-r4-training/src/bin/heldout-memory-panel.rs` built. That file is not a registered binary (the crate sets `autobins = false` in `crates/uor-r4-training/Cargo.toml`), so it needs a `[[bin]]` entry or a native-path rewrite before it can run. Its size is not registered; the page's later training step asks for at least 120 memory rows. Bind the panel, checks file and bound model by sha256, as the copy-span record does. State the conditions (oracle off, span with learned gate, span with gate floor 0.9), the bar, and the stop rules. Post it and ask for the non-author FIT review.
6. After FIT and the owner's release, run once on a GPU through `scripts/pod/uor-pod` (`uor-pod status` first). Labs stage the exact commands, and Casey types any owner release personally.
7. Open a PR that says `Closes #<this issue>` if the oracle ran, or `References #<this issue>` if only the pointer was built. Update `docs/mechanisms/pointer-key.md` and the board row in `docs/mechanisms/README.md`. Record the result as one row in the research notebook. No pointer page exists in `docs/research/notebook/`; the nearest is `m1-discrete-attention.md`, so ask a maintainer on the issue which page to use.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:54`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:114`](../../../docs/mechanisms/README.md)
- [`docs/research/open-questions.md:60`](../../../docs/research/open-questions.md)
- [`docs/start/transformer-vs-geometry.md:32`](../../../docs/start/transformer-vs-geometry.md)
- [`docs/mechanisms/pointer-key.md`](../../../docs/mechanisms/pointer-key.md)
- [`archive/labs/copy-span-oracle-2026-10-12/README.md`](../../../archive/labs/copy-span-oracle-2026-10-12/README.md)
