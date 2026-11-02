# uor-r4-cpu-train

Isolated CPU development trainers authorized by D84. GPU remains the qualification authority, and this crate is deliberately outside the workspace's default-member set.

The first unit provides `CpuWriterCore`, an independent implementation of exact cell admission followed by full-vector token ranking. `WriterReading` exposes one row's top-8 and target outcomes; `WriterSummary` aggregates top-1, top-8 and target admission. These are component oracles, not a training result.

`CpuFbsForward` executes the SpiralCore/FBS trunk and reproduces the serving query at Q12 with both whole-table nearest and native-cell writer heads. It supports snapped and raw Hamilton gates/transitions. `from_artifact` loads Hamming copy/type reads, `/7` opening/reply memory and the `/8` point-key fallback; `mark_reply_start`, `clear_reply_start` and `reset` control the memory path. `from_model` remains the no-read, no-memory entry point.

`CpuRankMarginTrainer` is the first finite learning unit on that path. It applies a deterministic frozen-query Q12 subgradient coordinate step to the target and hardest admitted rival rows of the single tied token table, reparses the patched artifact immediately to refresh Hamming codes, reports exact admission/top-1/top-8/rank readings, and round-trips a versioned checkpoint. Routes, gates and normalization remain unchanged; this is not yet a complete end-to-end trainer.

`CpuCounterfactualTrainer` extends the same exact full-forward acceptance loop to normalization gains, convolution/gate biases, decay offsets and all six FBS route tables through bounded BVG-K coordinate blocks. `CpuPanelEvaluator` grades a finite panel and records target admission, top-1/top-8, rank margin and component-wise CPU-versus-serving query/token/kept-cell gaps. `panel_admission_gate` refuses checkpoint admission loss or any serving mismatch; these are component controls, not a language result.

`CpuQuantizationShadow` pairs a no-read FBS snapshot with its exact Q12 artifact, applies accepted Q12 table deltas to both, and records the float-shadow, quantized-reference and exact CPU/serving decisions on one finite panel. Its checkpoint binds both states and refuses admission loss, serving drift or `EXPORT_LIMITED` float-only gains. This is component evidence only.

`cpu-public-discrete-parity` binds the public M1 seed-1 discrete artifact and public DEV split by SHA-256, extracts the trained codebook, and compares `CpuWriterCore` with `IntegerDiscreteModel` across all 14,287 eligible rows. It reproduces the published GPU aggregate `9,012` admitted, `7,002` top-1 and `7,485` top-8 with zero row mismatches.

`cpu-public-fbs-parity` binds both public raw FBS seeds, compares every complete Q12 query/token/kept-cell result over 192 positions per seed, and exercises one 16-sample rank-margin/panel control per seed. It is trained-artifact component evidence, not useful-learning or complete-reply evidence.

`CpuMaskedCorpus` parses the canonical `UORT` token container and matching response mask, builds causal samples only from each target's document prefix, and assigns whole documents to deterministic training or evaluation sides. `cpu-corpus-driver` runs an explicit bounded step budget, writes interval and final checkpoints, and reports exact admission/top-1/top-8/export readings alongside token and wall-clock throughput. The driver does not run automatically and does not treat open-ended panel text as a correctness result.

## Checks

```sh
cargo test -p uor-r4-cpu-train
cargo clippy -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings
cargo run --release -p uor-r4-cpu-train --bin cpu-writer-bench
cargo run --release -p uor-r4-cpu-train --bin cpu-fbs-forward-bench
cargo run --release -p uor-r4-cpu-train --bin cpu-rank-margin-bench
cargo run --release -p uor-r4-cpu-train --bin cpu-counterfactual-bench
cargo run --release -p uor-r4-cpu-train --bin cpu-panel-export-bench
cargo run --release -p uor-r4-cpu-train --bin cpu-public-discrete-parity -- <public-fixture-dir>
cargo run --release -p uor-r4-cpu-train --bin cpu-public-fbs-parity -- <public-fbs-fixture-dir>
cargo run --release -p uor-r4-cpu-train --bin cpu-corpus-driver -- <artifact> <tokens.u16> <mask.u8> <manifest.json> <seed> <steps> <evaluation-stride> <max-evaluation> <checkpoint-every> <checkpoint-dir>
```

The forward benchmark prints read-free, copy-only, copy-plus-type and full Hamming-plus-memory cases with hashes and process peak RSS. Its readings are component throughput, not training or language evidence.

The rank-margin benchmark times one repeated synthetic sample through the Hamming-read forward, exact ranking, accepted table edit and artifact rebuild. The counterfactual and panel benchmarks cover coordinate evaluation and exact CPU/serving panel parity. Their rates are component ceilings, not corpus learning or language evidence.
