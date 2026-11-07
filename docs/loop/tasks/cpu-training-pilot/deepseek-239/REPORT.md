# REPORT: cpu-training-pilot, deepseek-239

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506),
supporting
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](ORDER.md). Base: `195c21778`.

## What did not work, first

- The accepted checkpoints could not use the existing snapshot path on this
  host because it opens CUDA. The new path reads the saved tensors directly and
  never initializes a graphics device.
- The first full suite exposed two stale expectations after retention became
  supported. One memory test had been pointed at the writer head instead of its
  original nearest-head oracle, and the served-forward audit still expected a
  retention refusal. Both were repaired by restoring the established oracle and
  asserting exact retention parity.
- A repository-wide formatting invocation touched unrelated tracked files,
  including float-trainer sources. The complete diff confirmed those changes
  were formatting-only; every path was restored to `HEAD` before delivery and
  final formatting checks were scoped to changed Rust files.
- Fresh exact-head review found three fail-open config cases. The first version
  checked only the native writer's schema and dimensions, not its source,
  geometry, query-port, scoring and identity binding; it did not enforce the
  float Hamming readout's six-layer contract; and its boolean reader treated
  malformed values as false. The exporter now rejects all three classes.
- CPU/serving parity did not imply float/integer equivalence. Across 16 fixed
  oasst2 prompts and 32-token replies, seed 1 had 0/16 first-token differences
  but 9/16 complete-reply differences, first diverging at offsets 2-9. Seed 2
  also had 0/16 first-token differences and 4/16 complete-reply differences,
  first diverging at offsets 3, 4, 6 and 20.
- The local chat boundary is connected but its output is not useful
  conversation. Seed 1 answered `Hello! How is your day going?` with
  `"Hello 1999"` and stopped by short cycle. Seed 2 returned a repeated
  assistant-role fragment. The second seed-1 prompt
  `Tell me why geometry can make local language models cheaper.` settled into
  `"Tell membannlicannlicannlic"`.

## What changed

- Added `cpu-fbs-export`, which reads `model.safetensors`, the model config,
  the run config and `hamming_read.safetensors` without CUDA.
- The exporter reconstructs the SpiralCore/FBS route plans, checks all five
  projection fingerprints against the trained model config, validates the
  served recipe, exact tensor names and shapes, finite F32 values, the complete
  native-writer binding, Hamming inventory, head count and six-layer readout
  depth, and refuses existing output paths before reading.
- It decodes learned route queries, applies the run's served contract, writes
  the existing snapshot and `UORLUT01` formats, and reparses the artifact before
  success.
- Extended `CpuFbsForward` to execute `geometric-retention` artifacts with the
  same Q31 rate/keep tables as serving, and bound the mode into session
  identity with backward-compatible deserialization.
- Updated `cpu-served-forward-audit` to name the retention-rate recurrence in
  its manifest and prove it through parity rather than refusing it.
- Added focused tests for malformed, missing and extra model-config fields; a
  foreign served contract and native-writer bindings; projection-binding
  drift; missing and extra trunk and Hamming tensors; mismatched Hamming head
  and layer counts; non-boolean run flags; non-finite values; and existing
  output refusal.

The implementation stayed inside `crates/uor-r4-cpu-train/**` and the CPU
pilot's documentation scope. No CUDA source, pod, trainer or Claude-owned
checkpoint was changed.

## Measurements

Inputs from Claude-301's sealed b64L pair:

| Input | Seed 1 | Seed 2 |
|---|---|---|
| archive | `f7852f2540e58c17b2311d39fb47f8e53471e2cda8d6136f81b034cdf6bf5bba` | `1f24f03a4b638f7ad6fafd184f980bca324de758085a62e5b5d0dfb3a4f6ab32` |
| run config | `6b2b84640e37d1f6322a5b4ab6db0d783469d7adf4db1c7e67b532d4310056f2` | `f21567d0b43ba5fcc444bda840ad77bf8a5e9b46315ed8e515d223619f15e593` |
| model config | `1045acb61d92d390f0f61617ffa4c1847437473c9694c09787b48bf46e761ed2` | `778145167a18ff3b1006d4f609b7f608cc528b0dd753279b3ff38c8c63a88b95` |
| model tensors | `6fd2d764171cc416d5ff272e19e3ebe0644183ab9c43ca0662706756ac75f5de` | `13f3ae6230831ad9797224c0d20ecb6d15463fc28a3fcbea0ddfd9f0046c3306` |
| Hamming tensors | `672de97fcc329b10c3cde591f66047352851f9fe1d02df85a168190f5ba12ddd` | `3de5f52daa5d48504c3bba757e6ede95dd6a31bcba42653c5d12a6c1c5c35c7d` |

The model was width 256, MLP 384, vocabulary 4,096, context 192, six
recurrence layers, four Hamming heads, output-pattern `rrrrrr`, objective
`nearest`, `retain_rates=true` and `train_as_served=true`. The pinned tokenizer
`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` was found
locally; no object-store retrieval was needed.

Fresh CPU export reproduced the prior bytes exactly:

| Seed | `UORLUT01` SHA-256 | snapshot SHA-256 | artifact bytes | snapshot bytes |
|---:|---|---|---:|---:|
| 1 | `9a370566cd9da163191c0196763c68ddb395927fcf61fdecc1736b04941a4bd9` | `0c2f4c1cefa293c5076293535e3dda4b206e409167610733d75ea3455bdf722c` | 2,156,416 | 12,839,628 |
| 2 | `5039cadefd60633bcff5ac8511aff362e1174ef4cda20844a8febad625d2e304` | `a80c406da9ace065d90d78aeaf9115b4c937b182f5b72b11fe900368b31f8f6d` | 2,156,416 | 12,584,108 |

The independent 16-reply path audit compared each seed's 1,344 positions,
344,064 Q12 query coordinates, 1,344 token decisions and 10,752 kept cells
with zero mismatches and no instrument fault. The fresh 128-position audit
then checked 32,768 Q12 coordinates, 128 tokens and 1,024 kept cells per seed,
again with zero mismatches. Both manifests reported
`recurrence_only_geometric_retention_rate` and
`geometric_hamilton_output_with_retention_rate`.

`fbs-toks` completed the standard 128-token prefill and 64-token decode
protocol on both artifacts. Seed 1 measured 1,520 prefill tokens/s and 1,492
decode tokens/s; seed 2 measured 1,520 and 1,489. Seed 1's 64-token synthetic
reply settled on token 27; seed 2's settled on token 592. These are runtime
readings, not language evidence.

The product boundary used protocol 2, 16 worker threads, temperature zero and
the pinned tokenizer. Seed 1's repeated `--say` records were exactly equal in
reply ids, text and stop:
`[318,2301,223,19,27,27,27]`, `"Hello 1999"`, `short_cycle`.

Ignored raw evidence is under
`local/deepseek/b64l-cpu-export/`, including input-run exports, fresh
re-exports, parity rows, 128-position audits, `fbs-toks` records, chat records
and `SHA256SUMS`. The compact bound result is
[result.json](evidence/result.json). The 82-file archive has SHA-256
`cba67ceb79121eea8c26bb5eca59ba232419d59f55567edefad454c30676bdb9` and is
stored at
`caseyallard/uor-r4-store:results/deepseek/b64l-cpu-export-2026-11-06/b64l-cpu-export-evidence-2026-11-06.tar.gz`.
A fresh download matched that hash; `uor-store ls` lists the 17.7 MB object.

## Checks run at the head

- `cargo test --release --locked -p uor-r4-cpu-train --lib`: 450 passed,
  4 ignored, 0 failed.
- focused exporter tests: 13 passed.
- focused retained-rate CPU/serving, audit and checkpoint tests: 3 passed.
- scoped Clippy passed after allowing only the crate's four pre-existing lint
  categories (`type_complexity`, `needless_borrow`,
  `cloned_ref_to_slice_refs`, `unnecessary_min_or_max`).
- Rustfmt on changed files and `git diff --check` passed.
- `cargo build --release --locked -p uor-r4-cpu-train --bin cpu-fbs-export`
  passed.
- `cargo build --release --locked -p uor-r4-integer --features serving-wasm
  --bin uor-chat` and the same build for `fbs-toks` passed.
- determinism: both fresh exports matched the recorded artifact and snapshot
  SHA-256 values.
- path qualification: `fbs-toks`, `cpu-served-forward-audit`, `uor-chat` and
  the seeded repeat passed on both artifacts.
- D23, D25 and D32 ratchets, claim-wording, repository harness checks and loop
  delivery gates are recorded with the final commit.

## Skills used

No system-provided Codex skill was needed. The repository's D84 CPU research,
loop delivery and evidence procedures were followed directly.

## Still open

- complete float-to-integer reply equivalence: only first-token agreement is
  established on the fixed 16-prompt check;
- a model-quality run or conversational demonstration: the current b64L
  artifacts return repetitive, low-quality text through `uor-chat`;
- GPU qualification of this CPU export path, which this host cannot perform;
- the wider one-model roadmap work remains with Claude's active trainer track.

## Plain summary

Claude's best current width-256 recipe was saved as float tensors, and this
host has no usable GPU. The new converter reads those files on the CPU,
rebuilds the geometric route plan, checks that it matches the recipe exactly,
turns the learned routes and weights into the integer serving artifact, and
writes the same formats the normal exporter uses. Both seeds produce the same
bytes every time. The integer files run in the command-line token harness and
in the real chat CLI, and an independent CPU-versus-serving audit agrees on
every checked coordinate, token and kept cell. That proves the local export
and integer-serving boundary, not that the float model and integer model give
the same complete conversation, and not that the text is useful.

References #3506, #3407.
