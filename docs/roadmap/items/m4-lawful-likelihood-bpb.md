---
id: m4-lawful-likelihood-bpb
title: Build a lawful likelihood so bits per byte can be scored
milestone: M4
area: evaluation
status: ready
open_to: lab
needs_gpu: false
depends_on: []
notebook: docs/research/notebook/m4-serving.md
scope: [crates/uor-r4-discrete/**, crates/uor-r4-integer/**, docs/mechanisms/lut-serving.md, docs/model/serving-and-export.md, docs/research/findings.md, docs/research/open-questions.md, docs/labs/m4-lawful-likelihood-bpb/**, docs/harness/prep/m4-lawful-likelihood-bpb*]
---

## Why it matters

M4 ([#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032)) asks for one served integer model with held-out quality at or below 0.90 bits per byte, and no likelihood exists to score that bar. Any later language-modelling comparison needs the same likelihood. It sits at the end of the serving path described in [serving and export](../../model/serving-and-export.md), so the likelihood has to be a form the integer artifact can produce under the D11 contract. See also [lookup-table serving](../../mechanisms/lut-serving.md).

## What is missing

The current model's loss gives no probabilities, so the 0.90 BPB bar that M4 still carries cannot be scored on it. The M1 bar was changed on 19 October 2026 to integer top-1 because of this (CHARTER §4.8), and BPB "returns at a later milestone once the model gives probabilities". A lawful likelihood is still unbuilt, and the sources name no design.

## Done when

A likelihood with no softmax or squashing (D23, D25) feeding a registered instrument that scores the chat held-out stream in bits per byte, so the M4 bar "held-out quality at or below 0.90 bits per byte" can be tested. The sources do not say which stream file or byte count is "the chat held-out stream"; naming and hashing it is part of the pre-registration (step 4).

## How to start

1. Read `CHARTER.md` §3.2 (cross-entropy, log-softmax and "renamed or table forms" are all banned), §3.7 (float only offline, with an export gap), §4.8 (BPB returns once a lawful likelihood exists) and the "What counts as success for the first milestone" section of `docs/research/open-questions.md`. Read `docs/mechanisms/lut-serving.md` and `docs/model/serving-and-export.md` for what the integer artifact can emit.
2. Post a one-line intent note on this issue, then write a design note and post it on the issue before any code. It must state: (a) the geometric construction that turns the model's integer scores into a probability over the next byte or token; (b) why it contains no softmax, cross-entropy, normalized exponential, sigmoid or table form of them, and no dense matmul; (c) what "lawful" means here as a testable property (for example, probabilities are non-negative and sum to exactly 1 over the full alphabet), and how bits are computed from them without a banned function; (d) the finite host controls (unit tests, not research runs): a tiny alphabet where every probability can be enumerated and checked, and a count-table baseline scored by the same code.
3. Check `docs/mechanisms/README.md` and `docs/research/findings.md` for an existing mechanism that already gives a normalized read, so the note builds on geometry the project already has. Ask the PM through a decision card if the note needs a choice the charter reserves (a change to a bar, a status or a constraint).
4. Write a one-page pre-registration sheet and get a non-author FIT review before any GPU run (CHARTER §4.5). It names the held-out chat stream, its byte count and its sha256, freezes it, and fixes the scoring formula and the 0.90 BPB threshold. The design, the code and the finite host controls need no GPU. The registered panel-scoring run is research compute, so it runs on the GPU (§3.5) through `scripts/pod/uor-pod`; run `uor-pod status` first.
5. Implement the likelihood and the scorer in Rust, in the crate that fits. No crate is named for scoring: the integer export and serving engine is `crates/uor-r4-integer` (see the Code list in [lut-serving](../../mechanisms/lut-serving.md)), and held-out evaluation of the discrete model runs through `crates/uor-r4-discrete` (`eval-corpus`). Say in the design note which crate you pick and why. Add the finite tests from step 2 and run them on the host with `~/.cargo/bin/cargo test -p <crate>`, without `--features cuda`. Run `uor-harness check` before pushing.
6. Open a PR that says `Closes #<this issue>` once the instrument is registered, or `References #<this issue>` if only the design or the likelihood lands. Record the result as one row in `docs/research/notebook/m4-serving.md`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`ROADMAP.md:17`](../../../ROADMAP.md) (the M4 row with the 0.90 bar)
- [`ROADMAP.md:23-24`](../../../ROADMAP.md)
- [`docs/research/open-questions.md:102`](../../../docs/research/open-questions.md)
- issue#2032
- [`CHARTER.md`](../../../CHARTER.md) §3.2, §4.8
