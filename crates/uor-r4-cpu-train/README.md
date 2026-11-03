# uor-r4-cpu-train

Isolated CPU development trainers authorized by D84. GPU remains the qualification authority, and this crate is deliberately outside the workspace's default-member set.

The first unit provides `CpuWriterCore`, an independent implementation of exact cell admission followed by full-vector token ranking. `WriterReading` exposes one row's top-8 and target outcomes; `WriterSummary` aggregates top-1, top-8 and target admission. These are component oracles, not a training result.

`CpuFbsForward` executes the SpiralCore/FBS trunk and reproduces the serving query at Q12 with both whole-table nearest and native-cell writer heads. It supports snapped and raw Hamilton gates/transitions. `from_artifact` loads Hamming copy/type reads, `/7` opening/reply memory and the `/8` point-key fallback; `mark_reply_start`, `clear_reply_start` and `reset` control the memory path. `from_model` remains the no-read, no-memory entry point.

`CpuRankMarginTrainer` is the first finite learning unit on that path. It applies a deterministic frozen-query Q12 subgradient coordinate step to the target and hardest admitted rival rows of the single tied token table, reparses the patched artifact immediately to refresh Hamming codes, reports exact admission/top-1/top-8/rank readings, and round-trips a versioned checkpoint. Routes, gates and normalization remain unchanged; this is not yet a complete end-to-end trainer.

`CpuCounterfactualTrainer` extends the same exact full-forward acceptance loop
to normalization gains, convolution/gate biases, decay offsets and all six FBS
route tables through bounded BVG-K coordinate blocks. Its independent
candidate scores run in parallel and return to coordinate order before the one
serial joint decision. `CpuPanelEvaluator` grades panel rows in parallel and
combines their integer readings in source order, recording target admission,
top-1/top-8, rank margin and component-wise CPU-versus-serving
query/token/kept-cell gaps. `panel_admission_gate` refuses checkpoint admission
loss or any serving mismatch; these are component controls, not a language
result.

`cpu-discrete-corpus-driver` connects the exact discrete proposal/apply/checkpoint path to a strict masked corpus. It starts from a four-table `uor-r4.lut-discrete/1` artifact at any recorded epoch, executes bounded K=1 vector/read coordinates with the GPU short-final-batch semantics, reports matched panel and CPU-versus-serving readings, writes hash-bound interval/final checkpoints, and reconstructs a reloadable four-table artifact from the trained state. Its target-rank sum is over tokens in admitted cells, not all vocabulary entries. It refuses the 19-table M1 trunk because the CPU scorer does not implement the extra trunk tables. This is a development screener, not GPU training qualification.

`CpuQuantizationShadow` pairs a no-read FBS snapshot with its exact Q12 artifact, applies accepted Q12 table deltas to both, and records the float-shadow, quantized-reference and exact CPU/serving decisions on one finite panel. Its checkpoint binds both states and refuses admission loss, serving drift or `EXPORT_LIMITED` float-only gains. This is component evidence only.

`cpu-public-discrete-parity` binds the public M1 seed-1 discrete artifact and public DEV split by SHA-256, extracts the trained codebook, and compares `CpuWriterCore` with `IntegerDiscreteModel` across all 14,287 eligible rows. It reproduces the published GPU aggregate `9,012` admitted, `7,002` top-1 and `7,485` top-8 with zero row mismatches.

`cpu-public-fbs-parity` binds both public raw FBS seeds, compares every complete Q12 query/token/kept-cell result over 192 positions per seed, and exercises one 16-sample rank-margin/panel control per seed. It is trained-artifact component evidence, not useful-learning or complete-reply evidence.

`cpu-served-forward-audit` defines the supported exact FBS contract in
`served_forward::exact_fbs_contract`, emits the artifact's active mechanism
manifest, compares every query coordinate, token and kept cell with
`IntegerStackSession`, and reports modelled stage work. Its timing section uses
separate fresh CPU-oracle and CPU-serving passes after parity; table bytes are
semantic stage footprints, not hardware cache counters. `--no-timing` removes
the measured section and makes the JSON byte-deterministic for a fixed artifact,
token sequence and reply cursor. The report also inventories float-only training
operations and their exact, straight-through, secant or unresolved treatment.
It is an instrument, not a qualification result.

For release candidates, bind a real corpus slice instead of a hand-written list:

```sh
python3 scripts/audit_served_forward_gate.py \
  --binary target/release/cpu-served-forward-audit \
  --artifact ARTIFACT \
  --uort TOKENS.u16 --token-offset 0 --token-count 192 \
  --expected-artifact-sha256 SHA256 \
  --report REPORT.json --promote FINAL.uorlut
```

The gate validates the complete deterministic report and refuses to overwrite
the report or final artifact. Build the audit in an isolated Cargo target when
the workspace's mixed-feature integer outputs may collide.

`CpuMaskedCorpus` parses the canonical `UORT` token container and matching response mask, builds causal samples only from each target's document prefix, and assigns whole documents to deterministic training or evaluation sides. `cpu-corpus-driver` runs an explicit bounded step budget, writes interval checkpoints plus reloadable final checkpoint and `UORLUT01` artifacts, and reports exact admission/top-1/top-8/export readings alongside token and wall-clock throughput. Each checkpoint carries a sidecar binding for the source artifact, corpus inputs, trainer seed, split controls and checkpoint bytes. An optional eleventh argument resumes from a bound interval checkpoint; the step argument remains the total target, so only the remaining steps execute. The driver does not run automatically and does not treat open-ended panel text as a correctness result.

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
cargo run --release -p uor-r4-cpu-train --bin cpu-served-forward-audit -- --artifact <fbs-artifact> --tokens 0,7,11 --no-timing
cargo run --release -p uor-r4-cpu-train --bin cpu-corpus-driver -- <artifact> <tokens.u16> <mask.u8> <manifest.json> <seed> <steps> <evaluation-stride> <max-evaluation> <checkpoint-every> <checkpoint-dir> [resume-checkpoint]
cargo run --release -p uor-r4-cpu-train --bin cpu-discrete-corpus-driver -- <four-table-artifact> <tokens.u16> <mask.u8> <manifest.json> <seed> <steps> <evaluation-stride> <max-evaluation> <checkpoint-every> <checkpoint-dir> [resume-checkpoint]
```

The forward benchmark prints read-free, copy-only, copy-plus-type and full Hamming-plus-memory cases with hashes and process peak RSS. Its readings are component throughput, not training or language evidence.

The rank-margin benchmark times one repeated synthetic sample through the
Hamming-read forward, exact ranking, accepted table edit and artifact rebuild.
The counterfactual and panel benchmarks compare 1, 8, 16 and 32 workers with
stable checksums and exact CPU/serving panel parity. Their rates are component
ceilings, not corpus learning or language evidence.
