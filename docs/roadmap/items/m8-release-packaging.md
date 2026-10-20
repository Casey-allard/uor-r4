---
id: m8-release-packaging
title: Prepare the checksummed alpha release package and install path
milestone: M8
area: tooling
status: ready
open_to: anyone
needs_gpu: false
depends_on: [m4-export-uorlut01-cli, m5-laptop-full-path-measurement, m6-qualify-reasoning-coding, m7-parity-library-cli-browser]
---

## Why it matters

This is the deliverable of M8: the single file and install path a person outside the project would actually download. It sits at the end of the path from trained model to clients, after the export (M4), the laptop measurement (M5), reasoning and coding (M6) and the library, command-line and browser parity (M7). Nothing here can be finished until those bars have real numbers to bind. The packaging work can still be designed and tested now, against a stand-in file, so the path is ready when a model exists. See [the releases and artifacts page](../../products/releases-and-artifacts.md).

## What is missing

No versioned, checksum-pinned release artifact, reproducible install path or release notes exist. The two GitHub releases on file (`v0.1`, `v0.1.0-alpha`) belong to an earlier engine and are historical. [`.github/workflows/release.yml`](../../../.github/workflows/release.yml) has all three jobs paused (`if: ${{ false }}`) and packages the old R4G1 bundle, not `UORLUT01`. Release notes must bind the exact measured scores and known limitations, and the owner declares alpha. No lawful model has been exported yet, so there is no file to pin.

## Done when

M8 bar ([`ROADMAP.md`](../../../ROADMAP.md) line 21): "One versioned, checksum-pinned model file, declared alpha by the owner". The issue adds a reproducible installation and release notes binding the exact measured scores and known limitations. The [releases page](../../products/releases-and-artifacts.md) reads the full M8 bar in [the project track](../../integration/project-track.md) as also asking for independent verification on consumer hardware. Check the wording of issue #2036 before registering the checklist. The bar is not met by a draft package: it needs the M1 to M7 bars met first, and the owner's declaration.

## How to start

1. Read [`docs/products/releases-and-artifacts.md`](../../products/releases-and-artifacts.md), the M8 section of [`docs/integration/project-track.md`](../../integration/project-track.md), issue #2036 and CHARTER sections 2, 3.6 and 3.12 (every artifact binds its data, seed, configuration, parameters and format). Also read `docs/RELEASE_PIPELINE.md`, which is a preserved reference for the older R4G1 bundle, not the product pipeline.
2. Post a design note on the issue first. It must say: the release checklist (model file with SHA-256, tokenizer identity, release notes, `uor-chat` binaries per platform with `.sha256`, WebAssembly bundle, Studio at `/studio`, independent verification); which items can be built before a model exists; and which two inference contracts apply. `docs/inference_contract.md` covers the older R4G1 runtime and D11 rules R1 to R5 cover the product. Say which one the release binds.
3. Build the checksum tooling against a stand-in. Open `crates/uor-r4-integer/src/stack/format.rs`, where the `UORLUT01` reader already computes the SHA-256 of the whole file. Write a small Rust release-manifest tool, with tests, that records the file hash, tokenizer identity (see [`number-aware-tokenizer`](number-aware-tokenizer.md) for how identity is pinned), code SHA and measured scores, and refuses a file whose hash does not match. Run `~/.cargo/bin/cargo test -p uor-r4-integer` before and after. No new Python.
4. Draft the release-notes template in the PR. It must have fixed fields for the code SHA, file hash, tokenizer, training data and seed, every measured score with its panel, the export gap and known limitations. Leave score fields as "NOT_RUN" until M1 to M7 report. Do not write a score the notebooks do not hold.
5. Plan the workflow change as a separate step: [`.github/workflows/release.yml`](../../../.github/workflows/release.yml) must be repointed from R4G1 to `UORLUT01` and re-enabled. Do not enable it from this issue without the owner. Re-run the zero-multiply audits (native 44 symbol ranges, WebAssembly 7 of 7 at last count) on the shipped binaries.
6. Open a PR that says `References #<this issue>`. It closes the issue only when M1 to M7 are met, the file is pinned and the owner has declared alpha. Record the result as one row in a research notebook. There is no M8 page under `docs/research/notebook/` yet, so ask on the issue whether to add one or use `docs/research/notebook/m4-serving.md`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`ROADMAP.md:21`](../../../ROADMAP.md)
- issue#2036
- [`docs/products/releases-and-artifacts.md`](../../products/releases-and-artifacts.md)
- [`docs/integration/project-track.md`](../../integration/project-track.md)
