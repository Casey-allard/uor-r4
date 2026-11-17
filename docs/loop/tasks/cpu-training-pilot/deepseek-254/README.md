# Grouped carry-hold checkpoint audit

`cpu-carry-hold-audit` reads Claude-334 checkpoints without training, exporting
or serving them. It now reproduces the run's diagnostic buckets:

- hold gain over the 16 question tokens, four header tokens, 512 value tokens
  and all remaining tokens;
- each ask depth's mean absolute type-weight row (`|w_y|`);
- the existing 16-relation ask-selection reading.

## One checkpoint

```sh
target/deepseek-carry-hold-release/release/cpu-carry-hold-audit \
  --checkpoint CHECKPOINT \
  --expected-model-sha256 MODEL_SHA256 \
  --expected-hamming-sha256 HAMMING_SHA256 \
  --expected-hold-rows 8 \
  --out NEW-REPORT.json
```

## Several arms and seeds

The manifest and checkpoint paths are local. A checkpoint path must be relative
to the manifest's directory and must not contain `..`.

```json
{
  "schema": "uor-r4.cpu-carry-hold-manifest/1",
  "arms": [
    {
      "id": "b2-aqh-s1",
      "label": "B2-aqH",
      "seed": 1,
      "checkpoint": "s1/B2-aqH/models/step-016384",
      "model_sha256": "MODEL_SHA256",
      "hamming_sha256": "HAMMING_SHA256",
      "expected_hold_rows": 8
    },
    {
      "id": "b2-ah-s1",
      "label": "B2-aH",
      "seed": 1,
      "checkpoint": "s1/B2-aH/models/step-016384",
      "model_sha256": "MODEL_SHA256",
      "hamming_sha256": "HAMMING_SHA256",
      "expected_hold_rows": 8
    }
  ]
}
```

```sh
target/deepseek-carry-hold-release/release/cpu-carry-hold-audit \
  --manifest MANIFEST.json \
  --workers 8 \
  --out NEW-BATCH-REPORT.json
```

The manifest runner rejects duplicate arm ids, label/seed pairs, checkpoint
paths and checkpoint-hash pairs. It preserves manifest order while auditing
arms in parallel and writes only to a new path.

Deepseek-259 used this mode on Claude-334's eight sealed held checkpoints:
both arms, both seeds and both saved steps. All grouped gains and per-depth
type-weight means matched the training log exactly. Deepseek-260 retains the
expected-versus-observed values and recall-file hashes in
`evidence/real-eight-gain-parity.json`.

No result from this tool is B2, carry, capability, export or serving evidence.
