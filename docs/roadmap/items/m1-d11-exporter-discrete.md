---
id: m1-d11-exporter-discrete
title: Write the exporter from a trained discrete model to a D11 file
milestone: M1
area: serving
status: done
open_to: anyone
needs_gpu: false
depends_on: []
scope: [crates/uor-r4-discrete/**, crates/uor-r4-integer/**, crates/uor-r4-lut/**, docs/mechanisms/lut-serving.md, docs/model/serving-and-export.md]
---

## Why it matters

Needed for M1 bar (d), Stage 4 and M4; today the serving engine runs only a synthetic stand-in. The discrete model's table is already whole numbers, so there is no float to round, but nothing turns a saved snapshot into the file the integer engine reads. See [the lookup-serving page](../../mechanisms/lut-serving.md).

## What is missing

No exporter turns a trained discrete model into a D11 (UORLUT01) file. Older float stacks were exported and served in October; nothing trained under today's rules has been. The architecture page says the existing schemas /1 and /2 serve softmax through exponent tables, and all three schemas stream dense 4-bit maps with SiLU and GELU tables, so a new table-native schema is likely needed (a hypothesis in the sources). A lawful exporter for any trained trunk is also missing (the float-trunk exporter is the separate item `d11-exporter-float-trunk`), so the export gap is unmeasured. The `crates/uor-r4-lut` crate is a metadata-only facade under D29: it keeps the artifact read-write API and compiles no engine.

## Done when

Exporter on main; a discrete snapshot round-trips through `crates/uor-r4-integer` with matching integer outputs on finite tests; the serving audits stay green (0 float, 0 mul/div). The audits that exist today are 44 native symbol ranges and 7 of 7 WebAssembly serving functions.

## How to start

1. Read `docs/mechanisms/lut-serving.md`, `docs/model/architecture.md` (the Export and Serving items near line 53), `docs/integration/project-track.md` (M4, native audit), `docs/labs/wasm-serving-runtime-2026-10-18/README.md` and the retired record `archive/labs/d11-serving-2026-10-10/README.md`. Then open the code: `crates/uor-r4-discrete/src/snapshot.rs` (the `Snapshot` type), `crates/uor-r4-lut/src/format.rs` (container read-write), `crates/uor-r4-integer/src/format.rs` and `model.rs` (what the runtime reads).
2. First deliverable, posted on the issue: a design note. It maps each `Snapshot` field to a section of the D11 file and names the schema to use. It shows why no exponent table (softmax) and no dense multiply are needed on the read path, and that the file holds whole numbers only. It names the crate where the exporter lives (the discrete crate has CUDA parts, so check whether `~/.cargo/bin/cargo check -p uor-r4-discrete` builds without nvcc; keep the exporter's host code where it builds on a laptop). It defines the finite controls: a tiny hand-built snapshot with known outputs, the same input run through the snapshot path and through the exported file, and integer outputs compared for exact equality.
3. Write the failing round-trip test first, then the exporter. Run `~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm` (without that feature the serving modules do not compile) and the tests of the crate you chose. Add a negative control: an edited table must change the integer output, so the test cannot pass by comparing a file with itself.
4. Run the audits on the built runtime: the native audit `python3 scripts/audit_zero_matmul_serving.py <path_to_binary_or_rlib> --stack` (project-track.md, M4, records it green with 44 symbol ranges; read its header for usage; build the binary with `~/.cargo/bin/cargo build --release -p uor-r4-integer --features serving-wasm --bin uor-r4-stack`) and `python3 scripts/audit_zero_matmul_wasm.py <wasm> --serving --call-graph --tap` (run both from the repository root; `<wasm>` is the `uor_r4_integer_bg.wasm` that the `wasm-pack build` command in the 18 October record writes, in `crates/uor-r4-integer/pkg/`). Both are existing scripts; add no new Python. Both must report 0 float, 0 mul/div.
5. No pre-registration or GPU is needed for builds and finite host tests. Any GPU run that scores a trained model is the item `m1-export-gap-and-toks`.
6. Open a PR that says `Closes #<this issue>` (or `References` if partial). Add one row to `docs/research/notebook/m4-serving.md`. Read its format page first: a row needs a bar fixed beforehand, and here the bar is the finite round-trip test above.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:23`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:94`](../../../docs/mechanisms/README.md)
- [`docs/model/architecture.md:53`](../../../docs/model/architecture.md)
- [`docs/research/open-questions.md:87`](../../../docs/research/open-questions.md)
- [`docs/start/transformer-vs-geometry.md:61`](../../../docs/start/transformer-vs-geometry.md)
- [`docs/start/where-we-are.md:58`](../../../docs/start/where-we-are.md)
- [`docs/start/where-were-going.md:50`](../../../docs/start/where-were-going.md)
- [`crates/uor-r4-lut/README.md`](../../../crates/uor-r4-lut/README.md)
