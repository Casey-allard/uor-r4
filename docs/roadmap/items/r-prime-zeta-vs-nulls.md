---
id: r-prime-zeta-vs-nulls
title: Test the prime-zeta code against injective nulls at zero updates
milestone: research
area: evaluation
status: ready
open_to: lab
needs_gpu: true
depends_on: []
notebook: docs/research/notebook/m1-prime-zeta.md
scope: [docs/harness/prep/r-prime-zeta-vs-nulls*, docs/labs/r-prime-zeta-vs-nulls/**, docs/loop/tasks/r-prime-zeta-vs-nulls/**]
---

## Why it matters

The prime-zeta codebook is the starting table that new training lineages load in place of a transformer's random embedding. Today it is the baseline arm P8 and the fallback cell inside the address-structured table. This test tells us whether its prime and zeta-phase structure matters, or whether any code that gives each word a distinct address would do the same job. See [the prime-zeta codebook page](../../mechanisms/prime-zeta-codebook.md). No capability is established yet, and this item trains nothing.

## What is missing

PrimeZeta gives all 4,096 tokens distinct addresses (4,096 of 4,096 unique vectors, 256 of 256 cells) ([construction record](../../../archive/labs/prime-zeta-construction-2026-10-14/README.md)). It is unknown whether its structure matters or only its uniqueness.

**`crates/uor-r4-discrete/**` was removed from `scope:` on 25 Oct (antigravity-83).** The `RandomInjective` and `QuotaOnlyInjective` modes, `prime-zeta-codebook` CLI, `7/7` host tests, and `pz-nulls-battery.sh` are already merged on `main` (`antigravity-27`, `antigravity-28`, `antigravity-36`). Only the pre-registration sheet registration + `TEST FITNESS: FIT` review and the zero-update GPU evaluation battery + notebook row remain.

The comparison against a random-injective null and a quota-only-injective null has not been run at zero updates. Those two nulls do not exist in code: `CodebookMode` in `crates/uor-r4-discrete/src/prime_zeta.rs` has only `PrimeZeta`, `TangentNull` and `NormalNull`. The two existing nulls run the same quota and uniqueness passes as PrimeZeta, so they are injective too, but they still use zeta phases.

The sources also leave two things undefined. They do not say which "reach" and "capacity" measures to report. They register no bar for this comparison. The nearest recorded measures are the census's distinct vectors and reachable rows (alias ceiling), and the 19 October pre-flight's admitted rows (127 of 3,847 for P8, against a gate of 500) ([pre-flight](../../labs/injective-preflight-and-c2-qualify-2026-10-19/README.md)). The trained comparison is `NOT_RUN` and held; it is out of scope here.

## Done when

Zero-update reach and capacity numbers for prime-zeta, random-injective and quota-only-injective codes on the same panel, recorded as a notebook row. The bar is not yet registered; registering it, and defining "reach" and "capacity", is part of the pre-registration sheet.

## How to start

1. **Read the construction.** Open `archive/labs/prime-zeta-construction-2026-10-14/README.md` and `docs/mechanisms/prime-zeta-codebook.md`. Then read `crates/uor-r4-discrete/src/prime_zeta.rs`: the `CodebookMode` enum (line 135), `generate_codebook` (line 275) and its quota and uniqueness passes (`cell_quota`, line 361). Note the page's caveat that both passes already run on the existing nulls. Also read the census in `crates/uor-r4-discrete/src/coverage_census.rs` and `docs/labs/coverage-census-2026-10-18/README.md`, and check whether the alias ceiling can differ between two injective codes. If it cannot, it cannot separate them.
2. **Post a design note on the issue (first deliverable).** It must contain these parts.
   - A definition of each null, proposed for FIT review. For example, random-injective: 4,096 distinct vectors drawn from a seeded integer generator with no cell quota. Quota-only-injective: the same 16 tokens per cell and unique interior roots as PrimeZeta, but with the token-to-cell assignment from a seeded shuffle instead of zeta phases.
   - The geometric construction, which is the E8 root-orbit table from `geometry::ROOTS`, and how each null is built without any new floating-point step, softmax or matmul (CHARTER §3.1–§3.3).
   - Host unit tests (finite, no pre-registration needed): 4,096 of 4,096 distinct vectors, exactly 16 tokens in each of the 256 cells for the quota null, and a table that is bit-exact across seeds and across macOS and Linux, as in the construction record.
   - Which measures count as "reach" and "capacity", with their definitions.
3. **Build the nulls and host-check them.** Add the two modes in `crates/uor-r4-discrete/src/prime_zeta.rs`, or a separate module. Export them with `crates/uor-r4-discrete/src/bin/prime-zeta-codebook.rs` (usage: `prime-zeta-codebook SPEC_JSON SEED OUTPUT [prime_zeta|tangent_null|normal_null:SEED]`). Add unit tests beside `v4096_codebook_is_bit_exact_across_named_seeds_and_collision_free`. Run `~/.cargo/bin/cargo test -p uor-r4-discrete prime_zeta`. Before editing, check whether `training::driver_source_sha256()` in `crates/uor-r4-discrete/src/training.rs` hashes any file you touch. Host builds and finite tests need no pre-registration.
4. **Draft the pre-registration sheet.** Use `uor-harness prep new` and `uor-harness prep fit`, with `docs/harness/prep/m1-attention-address-structured-cells-DRAFT-2026-10-19.md` as the model. The arms are PrimeZeta (P8, the pinned table) plus the two new nulls, with fixed seeds. The panel is the 14,287-row DEV panel used by the 19 October pre-flights, frozen and hashed (its data pins are listed in `docs/labs/address-cells-preflight-2026-10-19/README.md`). The sheet registers the bar and the stop rules and states the classification, instrument versus mechanism (§4.1, §4.2). It needs a non-author `TEST FITNESS: FIT` before any GPU run (§4.5).
5. **Run at zero updates on the GPU.** Copy the shape of `docs/labs/address-cells-preflight-2026-10-19/evidence/asc-battery.sh`: rebuild the tables on the pod and check their hashes, then run `init-corpus` and `eval-corpus` from `crates/uor-r4-discrete/src/main.rs`. `eval-corpus` refuses any checkpoint with accepted updates, so nothing trains. Use `scripts/pod/uor-pod` and run `uor-pod status` first. Save results off the pod before `down`. Ask on the issue before spending pod time: the PM or owner decides whether a zero-update battery needs a release.
6. **Open a PR and record the row.** The PR says `Closes #<this issue>`, or `References` if only the sheet or the tables landed. Add one row to `docs/research/notebook/m1-prime-zeta.md`, with the verdict worded as a result for a configuration, not for the mechanism.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:39`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:87`](../../../docs/mechanisms/README.md)
- [`docs/research/open-questions.md:45`](../../../docs/research/open-questions.md)
- [`docs/start/where-we-are.md:31`](../../../docs/start/where-we-are.md)
- [`docs/mechanisms/prime-zeta-codebook.md`](../../mechanisms/prime-zeta-codebook.md)
- [`archive/labs/prime-zeta-construction-2026-10-14/README.md`](../../../archive/labs/prime-zeta-construction-2026-10-14/README.md)
