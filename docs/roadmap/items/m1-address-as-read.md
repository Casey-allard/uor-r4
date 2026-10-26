---
id: m1-address-as-read
title: Build the address-as-read mechanism and an object-store consumer
milestone: M1
area: mechanism
status: ready
open_to: anyone
needs_gpu: true
depends_on: [m1-trunk-assembly-and-training]
scope: [crates/uor-r4-object-store/**, docs/mechanisms/exact-memory.md, docs/labs/m1-address-as-read/**, docs/harness/prep/m1-address-as-read*]
---

## Why it matters

The model has no exact way to read stored content back by address. Building this connects the delivered object store to the read path, which M2 grounding needs. See [exact memory](../../mechanisms/exact-memory.md) and [the transformer-to-geometry map](../../start/transformer-vs-geometry.md).

## What is missing

The address consumer is delivered: `address_read` resolves exact and sieve queries to verified object-store content, with finite controls (`86eb68ebb`, #2950). Stage 0 then measured the pinned T16_fz D11 artifact on one RTX 5090: parity passed on all 14,287 DEV rows, but free-running exact recovery was 0/40 in the exact-hint, no-hint and random-hint arms, so `COPY_GATE_FAIL` blocked panel-v3, BPB and training. This is an artifact/copy-layout limitation, not a mechanism negative or an oracle result. Still missing is a valid serving-time oracle measurement on an artifact and read instrument that can first copy an explicitly supplied value. No oracle gain or language capability is established.

## Done when

A read that resolves an address to stored content from the object store, with finite tests, and a registered serving-time oracle test that measures the gain from supplying the true address.

## How to start

1. Read `docs/mechanisms/exact-memory.md`, `archive/labs/addressed-memory-2026-10-10/README.md`, `archive/labs/d19-object-store-2026-10-15/README.md`, `docs/labs/uorobj01-conformance-2026-10-18/README.md` and `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/exact-memory.json`. Skim the API in `crates/uor-r4-object-store/src/lib.rs`.
2. First deliverable, posted on the issue: a design note. It gives the geometric construction (what an address is, how it resolves to an object id, what a read returns), why it needs no softmax and no matmul, and the finite CPU controls (exact hit, exact miss, wrong address, byte-identical round trip).
3. Add a failing test first, then a small consumer module that resolves an address through `uor-r4-object-store`. Run `~/.cargo/bin/cargo test -p uor-r4-object-store` and the new tests.
4. Start from the Stage 0 receipt in `docs/loop/tasks/m1-address-as-read/kimi-40/REPORT.md`, not the rejected 54/80 three-arm plan. Reassess the artifact and read instrument; do not build panel-v3 or spend on a hidden-fact oracle until a repaired configuration passes the registered copy non-vacuity gate.
5. A repaired configuration needs its own sheet, fresh non-author FIT and D67 pre-run review before GPU evaluation. Preserve the Stage 0 parity requirement and matched no-hint/random-hint controls. A trained follow-up needs 2–3 seeds and a fresh FIT; record exact numbers, including a negative.

6. Open a PR that says `Closes #<this issue>` (or `References` if partial), and add one row to `docs/research/notebook/m3-durable-memory.md`.


## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:22`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:113`](../../../docs/mechanisms/README.md)
- [`docs/model/architecture.md:68`](../../../docs/model/architecture.md)
- [`docs/research/open-questions.md:86`](../../../docs/research/open-questions.md)
- [`docs/start/transformer-vs-geometry.md:33`](../../../docs/start/transformer-vs-geometry.md)
- [`docs/start/where-we-are.md:34`](../../../docs/start/where-we-are.md)

