# CPU checkpoint grading, 2026-11-07

This lab ran the `cpu-checkpoint-grade` path on the two accepted b64L integer
artifacts and on the registered `dd-2788` development panel.

The command has two input modes:

```sh
# Grade an existing artifact.
cpu-checkpoint-grade \
  --artifact b64L-s1.uorlut \
  --tokens dd-2788/tokens.u16 \
  --mask dd-2788/response_mask.u8 \
  --manifest dd-2788/manifest.json \
  --max-evaluation 4900 --max-replies 16 --reply-tokens 16 \
  --out s1.json

# Export a saved train-as-served checkpoint and grade it in one process.
cpu-checkpoint-grade \
  --checkpoint step-016384 \
  --run-config config.json \
  --snapshot-out b64L-s1.snapshot.json \
  --artifact-out b64L-s1.uorlut \
  --raw-gates --raw-transition \
  --tokens dd-2788/tokens.u16 \
  --mask dd-2788/response_mask.u8 \
  --manifest dd-2788/manifest.json \
  --max-evaluation 16 --max-replies 2 --reply-tokens 4 \
  --out report.json
```

`--baseline` grades a second artifact over the identical deterministic sample.
For the nearest head it reports candidate-only and baseline-only top-1 rows; for
the native writer it reports top-8 and admission differences.

The result is development evidence on the D84 host. It is not a GPU
qualification result, a float/integer equivalence result or useful-chat
evidence.
