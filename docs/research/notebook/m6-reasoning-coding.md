# Notebook: reasoning and coding (M6)

This notebook records reasoning and executable-coding results on the same native model. A generated panel or deterministic oracle is an instrument, not evidence of learned reasoning or coding.

## Results

Sources: [`archive/catalog.jsonl`](../../../archive/catalog.jsonl). Verdict words are defined on the [format page](./README.md#the-five-verdict-words).

| Date | What ran | Number | Bar | Verdict | Artifact |
|---|---|---|---|---|---|
| 2026-11-07 | [deepseek] Adopted outside contributor PR #3754: compiled and ran every frozen CodeTasks row's canonical `SandboxProgram::to_rust_source` output with `rustc --edition 2021 --test` under bounded compile/run deadlines. Host: AMD Ryzen 9 9950X, Linux x86_64; `rustc 1.97.1`. | **80 / 80 rows passed**, covering all **320 / 320** held-out vector assertions; 7 verifier integration tests passed in 10.06 s. | none registered (instrument qualification) | **positive (instrument QA):** the frozen bar's literal `rustc --test` cross-check now runs in CI; this is not model-emitted code, a capability result or useful-chat evidence. | [REPORT](../../loop/tasks/m6-code-task-rustc-verifier/deepseek-1/REPORT.md) |

## What a row here will need

A row needs the frozen panel identity, deterministic oracle results with denominators, a pre-registered bar, the exact saved model, and a fresh transfer result. Code rows need executed compilation/tests and complete generated programs, not only source text. A panel seal or fixture pass is not a capability result.

## Go deeper

- [Format and verdict words](./README.md).
- [`CHARTER.md`](../../../CHARTER.md), sections 3.10, 4.3 and 5.6.
- [`m6_rustc_verifier`](../../../crates/uor-r4-harness/src/panel_m6_rustc_verifier.rs): `rustc --test` verifier that compiles and runs all 80 frozen CodeTasks rows (instrument QA, not a capability result).
