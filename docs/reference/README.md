# Reference documents

This page indexes the project's normative and reference documents: the ones that define terms, contracts and formats, rather than explain or report. Most sit in this folder. A few stay at their original paths because code compiles them in or an evidence table pins them by path and hash; moving them would break those records. The table says which.

Many of these documents date from earlier stages of the project, and some describe surfaces that are no longer built (the R4G1 serving path in particular). The [charter](../../CHARTER.md) is the rule book; where a reference document disagrees with it, the charter wins.

## Definitions and contracts

| Document | What it defines | Where it lives, and why |
|---|---|---|
| [Formal vocabulary, notation and claim classes](../formal_vocabulary.md) | The claim language: what counts as a proof, a measurement or a hypothesis | `docs/`: cited from about twenty source files and more than fifty pages |
| [Configuration reference](../CONFIGURATION.md) | Runtime and build configuration | `docs/`: pinned by path in the integrated-attention evidence table |
| [Inference contract](../inference_contract.md) | The multiplication-free, zero-allocation serving contract | `docs/`: pinned by hash in the integrated-attention evidence tables, and read by the release workflow |
| [Inference operation contract](../transformerless/INFERENCE_OPERATION_CONTRACT.md) | The operation-level serving contract | `docs/transformerless/`: compiled in by `uor-r4-graph-format` and by the `uor-r4-wasm-router` tests |
| [Release pipeline](../RELEASE_PIPELINE.md) | How releases are built and published | `docs/`: pinned by path in the integrated-attention evidence tables |
| [Minimal client](../minimal_client.md) | The historical terminal client and local vendor API | `docs/`: pinned by path and hash |
| [Reproducibility](../reproducibility.md) | Byte-identical artifacts under compiler parallelism | `docs/`: linked from many archived records |
| [Corpus-scale law](../scaling_law.md) | Sizing an observe run for a teacher | `docs/`: linked from many archived records |
| [Hologram / R⁴ formal monograph](../hologram_r4_formal_monograph.md) | The formal specification of the R⁴ geometry | `docs/`: compiled into the `uor-r4-graph-compiler` tests |

## Decision records and design specifications

| Folder | What is in it |
|---|---|
| [`adr/`](adr/0000-template.md) | Architecture decision records ADR-0001 to ADR-0006, and a template. Some describe paths that were later superseded; each keeps its original text. |
| [`specs/`](specs/2026-10-19-rust-bpe-trainer-design.md) | Design specifications: the Rust BPE trainer and the admission anchor audit (both 19 Oct 2026) |

The ADRs and specifications moved here from `docs/adr/` and `docs/specs/` on 20 October 2026.

## Storage

| Document | What it covers |
|---|---|
| [Storage indexes](storage.md) | Where every stored file is: the HF store index, the map of folders removed from the laptop on 20 Oct 2026, the old research tree, pod results and the iCloud archive |
