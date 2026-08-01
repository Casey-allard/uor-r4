# SmolLM2 teacher baseline — issue #320 P1

This records the P1 rehearsal for the teacher-upgrade issue. It does not
replace the pinned stories15M fixtures or make a baseline-migration decision.

## Pinned source

| field | value |
|---|---|
| repository | `HuggingFaceTB/SmolLM2-135M-Instruct` |
| revision | `7e27bd9f95328f0f3b08261d1252705110c806f8` |
| source κ | `blake3:12d2cd8a877ef2cdcf785b3d4d1f373e0419074cc884aeaff06fc059686a5ba5` |
| source κ scope | `model.safetensors` |
| source bytes | `269060552` |
| license | Apache-2.0 |

The source κ is emitted by the HF teacher loader after reading the pinned
weights. The revision and κ are both recorded in
`models/smollm2-135m-instruct.json`.

## Rehearsal

The source was downloaded with the existing pinned HF path and compiled in an
isolated worktree:

```bash
hf download HuggingFaceTB/SmolLM2-135M-Instruct \
  --revision 7e27bd9f95328f0f3b08261d1252705110c806f8 \
  --local-dir .uor-models/sources/smollm2-135m-instruct \
  --include '*.safetensors' --include '*.json' --include '*.model' \
  --include 'merges.txt' --include 'LICENSE*' --include 'README.md'

cargo run --release --offline --bin r4 -- transformerless compile \
  --source .uor-models/sources/smollm2-135m-instruct \
  --output .uor-models/compiled/smollm2-135m-instruct \
  --seconds 60 --target 1000 --sequence-length 128
```

Observed rehearsal result:

- source directory: 260 MB;
- corpus: 1,000 teacher-labeled tokens across 11 stories;
- teacher generation: 1,000 tokens in 8.77 s (114.1 tokens/s);
- table-native compilation completed, including artifacts, store, tokenizer,
  calibration, hierarchical codes, and manifest.

This is a pipeline smoke/rehearsal, not a quotable quality baseline: the
1,000-token corpus is not the D3 held-out distribution and no comparison with
the stories15M teacher floor was made. P2 should run the declared row matrix
on a complete corpus before any migration decision.
