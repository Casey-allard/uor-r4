---
id: m1-address-as-read
title: Build the address-as-read mechanism and an object-store consumer
milestone: M1
area: mechanism
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

The model has no exact way to read stored content back by address. Building this connects the delivered object store to the read path, which M2 grounding needs. See [exact memory](../../mechanisms/exact-memory.md) and [the transformer-to-geometry map](../../start/transformer-vs-geometry.md).

## What is missing

Address as the read has never been built, and nothing yet consumes the object store. The object store is delivered: 60 tests, and a conformance check that pulled 262,144 bytes back byte-identical.

## Done when

A read that resolves an address to stored content from the object store, with finite tests, and a registered serving-time oracle test that measures the gain from supplying the true address.

## How to start

1. Read `docs/mechanisms/exact-memory.md`, `archive/labs/addressed-memory-2026-10-10/README.md`, `archive/labs/d19-object-store-2026-10-15/README.md`, `docs/labs/uorobj01-conformance-2026-10-18/README.md` and `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/exact-memory.json`. Skim the API in `crates/uor-r4-object-store/src/lib.rs`.
2. First deliverable, posted on the issue: a design note. It gives the geometric construction (what an address is, how it resolves to an object id, what a read returns), why it needs no softmax and no matmul, and the finite CPU controls (exact hit, exact miss, wrong address, byte-identical round trip).
3. Add a failing test first, then a small consumer module that resolves an address through `uor-r4-object-store`. Run `~/.cargo/bin/cargo test -p uor-r4-object-store` and the new tests.
4. Write a one-page pre-registration for the oracle test: supply the true slot address and measure the gain before any training (`docs/mechanisms/exact-memory.md`, "What would reopen it", step 2). The bar for this test is not yet registered; the sheet must set it before any run. The page's brief bar for the trained form (at least 8 more rows of 40 than the 19 of 40 seen, bits per byte within 0.01, 2 to 3 seeds, a frozen panel of at least 80 rows) is a starting point, not a registered bar. Use a frozen, hashed panel and a non-author FIT review (CHARTER section 4.5).
5. Run the registered test only after FIT. Evaluation runs on the GPU (CHARTER §3.5); an outside contributor may use their own GPU and must attach its specs, and a maintainer re-runs on an RTX 5090 when the hardware differs. A trained form needs a fresh FIT and an owner release, and training stays held until Casey types that release himself. Record exact numbers, including a negative.

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

