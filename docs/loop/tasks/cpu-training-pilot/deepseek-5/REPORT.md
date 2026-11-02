# REPORT: cpu-training-pilot, deepseek-5

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `0fa4c5fce`. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The public artifacts do not exercise reads or memory.** Both two-megabyte
  public raw-FBS controls have no Hamming records and no `/7` or `/8` memory.
  Their audit proves complete parity for the public trunk and native writer,
  while Hamming, copy-only, type-head, memory and point-key accounting are
  exercised only by synthetic controls in the unit suite.
- **This is not a model or training result.** It does not train through the
  exact integer graph, qualify a GPU path, score a capability panel, or show
  useful chat. The surrogate inventory records that full exact-graph
  backpropagation and dynamic activation quantization remain unresolved.
- **The timing is a diagnostic, not a stable benchmark.** Each reading is one
  fresh 64-position pass after a separate parity pass, without warm-up or
  repetition. The two seeds differ substantially, so the values must not be
  compared as a model-quality signal or used as a release throughput claim.
- **The existing Cargo feature layout exposes an artifact collision.**
  `uor-r4-integer` is built once for the normal serving graph with
  `serving-wasm` and once as a host build dependency without it; both write the
  same Cargo output filename. Incremental mixed-target builds can therefore
  see the wrong library. Verification used a fresh isolated `CARGO_TARGET_DIR`
  after reproducing the collision. This unit did not change the shared Cargo
  configuration.
- **The fresh review found a live served Hamming range bug.** Both Hamming
  implementations call `nearest` with `tokens.len() - 1`, whose scan is
  `1..t`, and so omit the immediately preceding position. The audit counter now
  models that observed range exactly. The shared serving kernel is outside this
  ORDER and was not changed; the discrepancy from its documented ideal
  neighbourhood must be repaired in a separate Claude-owned unit.
- **The contract's first rounding wording was too broad.** RMS mean division,
  reciprocal and both Hamming mean quotients truncate; only the specified
  later shifts round half-up. The contract now states the exact split and a
  focused assertion guards it.

## What changed

- [`crates/uor-r4-cpu-train/src/served_forward.rs`](../../../../../crates/uor-r4-cpu-train/src/served_forward.rs)
  defines the exact FBS contract: representation and exponents, operation
  order, rounded shifts, saturation, every active tie rule, Hamming reads,
  exact opening/reply and point keys, and both output heads. It emits the
  serialized and decoded-resident table payloads separately from semantic
  stage footprints.
- The same module binds an artifact, head, token sequence and reply cursor;
  compares every Q12 coordinate, token and kept cell with
  `IntegerStackSession`; hashes both decision streams; and returns row-zero
  failures as instrument faults. Modelled candidates and measured wall time
  are separate structures.
- The report inventories float-training operations absent from the exact
  forward and assigns `exact`, `straight_through`, `secant` or `unresolved`.
  Export rounding, icosian snaps, Hamming candidates, exact memory selection
  and point fallback are straight-through. Native writer selection is secant.
  Dynamic activation quantization, integer residual backpropagation and the
  training-only memory dropout/reply row cut are unresolved.
- [`crates/uor-r4-cpu-train/src/bin/cpu-served-forward-audit.rs`](../../../../../crates/uor-r4-cpu-train/src/bin/cpu-served-forward-audit.rs)
  adds the command. `--no-timing` emits the deterministic contract, manifest,
  parity and cost sections; timing is opt-out because wall-clock values cannot
  be byte-deterministic.
- [`crates/uor-r4-cpu-train/README.md`](../../../../../crates/uor-r4-cpu-train/README.md),
  this report, the CPU research notebook/program and the roadmap item document
  the command, evidence and limits.

## Measurements

Command shape:

```sh
CARGO_TARGET_DIR=<fresh-directory> cargo build -p uor-r4-cpu-train --release --bin cpu-served-forward-audit
<fresh-directory>/release/cpu-served-forward-audit \
  --artifact <public-dual-c131-raw-artifact> \
  --tokens <64 deterministic token ids> \
  --output <receipt>
```

The machine was the AMD Ryzen 9 9950X, 16 cores / 32 threads, Linux
7.0.0-42-generic, Rust/Cargo 1.97.1. Both artifacts were 2,144,960 bytes and
used the 4,096-token, width-256, MLP-384, six-layer, context-192 raw trunk with
the native cell writer.

| Seed | Artifact SHA-256 | Q12 coordinates / tokens / cells | Mismatches | CPU oracle tok/s | Serving tok/s | Slowest mean stage |
|---|---|---:|---:|---:|---:|---|
| 1 | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` | 16,384 / 64 / 512 | 0 / 0 / 0 | 4,331.057 | 3,556.453 | layer 2 recurrence core, 26,062 ns |
| 2 | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` | 16,384 / 64 / 512 | 0 / 0 / 0 | 5,648.712 | 4,681.020 | layer 0 recurrence core, 19,718 ns |

The complete CPU and serving decision hashes were identical within each seed:
seed 1 `029991dd2c1a24c940a142a0abb5f15da4f7200b08738f8123e591ec130d641e`;
seed 2 `015f4117e4a10690bdf68d5e3a2d550a3894ce3b2db6e936f782b0ee1e780bba`.
Each artifact reports 2,137,792 bytes of serialized table payload and
2,147,296 bytes of decoded resident table payload. These are structural table
measurements, not hardware PMU counters.

Raw receipts are [seed 1](evidence/public-seed-1.json) and
[seed 2](evidence/public-seed-2.json). The release binary SHA-256 was
`ce204b72e377792a946a357a7e290ad64aebe792d2dda607916e51588d631656`.

## Checks run at the head

- Eight focused release tests passed: deterministic report, complete parity,
  named stage timing, malformed artifact and token-sequence refusal,
  unsupported retention-mode refusal, exact copy-only/readout accounting,
  reply-less-memory accounting, rounding-boundary wording and a planted parity
  fault.
- Both public raw-FBS seeds completed their 64-position audit with zero query,
  token or kept-cell mismatches.
- Full release CPU-crate suite: 122 library tests passed (4 release benchmarks
  ignored), three corpus-driver tests passed and four discrete-driver tests
  passed.
- `cargo fmt -p uor-r4-cpu-train -- --check`, strict Clippy for the crate's all
  targets with no dependencies, `git diff --check`, `uor-harness check`
  (zero errors; 43 pre-existing advisories), `uor-harness loop report --check`
  and `cargo run -q -p xtask -- roadmap check` (134 items) passed.

## Skills used

No repository skill was invoked. The unit followed D84 CPU research, the
existing integer serving contract and the loop's fresh-context review rule.

## Still open

- A retained public or private artifact containing Hamming, turn memory and the
  point key, so those public controls need not be synthetic.
- A warmed, repeated timing protocol with idle-machine controls if the audit is
  to support performance claims.
- Gradient propagation through the exact integer recurrence, dynamic
  quantization and discrete selection graph.
- A trained float shadow and credentialed complete-reply/fresh-phrasing panels.
- The larger project gaps: useful local chat, fresh-template capability and the
  remaining roadmap milestones.

## Plain summary

The CPU trainer now has a precise written description of the exact model it is
serving, a command that lists every active geometric mechanism, and an
independent decision-for-decision check against the real serving session. On
both public trained seeds, every tested Q12 query coordinate, chosen token and
kept writer cell matched. The audit also makes the remaining training gap
explicit: some float-only operations have established surrogate gradients,
while exact integer backpropagation, dynamic activation quantization and
training-only memory behavior do not yet. This is a trustworthy development
instrument for the chat-model work; it is not evidence that the chat model is
finished.
