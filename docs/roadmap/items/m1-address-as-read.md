---
id: m1-address-as-read
title: Build the address-as-read mechanism and an object-store consumer
milestone: M1
area: mechanism
status: done
open_to: anyone
needs_gpu: true
depends_on: []
scope: [crates/uor-r4-object-store/**, docs/mechanisms/exact-memory.md, docs/labs/m1-address-as-read/**, docs/harness/prep/m1-address-as-read*]
---

## Why it matters

The model has no exact way to read stored content back by address. Building this connects the delivered object store to the read path, which M2 grounding needs. See [exact memory](../../mechanisms/exact-memory.md) and [the transformer-to-geometry map](../../start/transformer-vs-geometry.md).

## What is missing

Completed at the item's tested scope: `address_read` resolves exact and sieve queries to verified object-store content, and the registered five-arm oracle measured a true-address gain on one RTX 5090. True address produced 40/40 current, 20/20 previous and 20/20 typed abstention; no-memory recovered 0/80; deterministic sieve recovered 20/80. D11 parity held on 14,287/14,287 rows with byte-identical repeats. The meaningful next question is address formation on 20 current sieve misses, not storage. Learned address formation, default serving and M3 qualification are separate and not established. The T16_fz oracle does not need the M2 trunk checkpoint.

## Done when

A read that resolves an address to stored content from the object store, with finite tests, and a registered serving-time oracle test that measures the gain from supplying the true address.

## How to start

1. Read `docs/mechanisms/exact-memory.md`, `archive/labs/addressed-memory-2026-10-10/README.md`, `archive/labs/d19-object-store-2026-10-15/README.md`, `docs/labs/uorobj01-conformance-2026-10-18/README.md` and `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/exact-memory.json`. Skim the API in `crates/uor-r4-object-store/src/lib.rs`.
2. First deliverable, posted on the issue: a design note. It gives the geometric construction (what an address is, how it resolves to an object id, what a read returns), why it needs no softmax and no matmul, and the finite CPU controls (exact hit, exact miss, wrong address, byte-identical round trip).
3. Add a failing test first, then a small consumer module that resolves an address through `uor-r4-object-store`. Run `~/.cargo/bin/cargo test -p uor-r4-object-store` and the new tests.
4. Start from the Stage 0 receipt in `docs/loop/tasks/m1-address-as-read/kimi-40/REPORT.md` and the reviewed [object-store successor design](../../labs/m1-address-as-read/integer-read-successor.md), not the rejected 54/80 three-arm plan. Extend the existing object-store GPU runner with no-memory, no-address, deterministic-sieve, true-address and planted-random arms. Require exact 40/40 current, 20/20 previous and 20/20 abstention hard-emission gates before any hidden-fact panel; keep model generation and address formation separate from oracle assistance.
5. A repaired configuration needs its own sheet, fresh non-author FIT and D67 pre-run review before GPU evaluation. Preserve the Stage 0 parity requirement, no-memory/no-address byte identity, the deterministic sieve baseline and the random-address planted-wrong control. A trained follow-up needs 2–3 seeds and a fresh FIT; record exact numbers, including a negative.

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

