# ORDER: one-model-end-to-end, deepseek-42 (served-panel parity repair)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `59494646c`.

## Goal

Repair the fresh review of PR #3895 by pinning the content-readout report
identity and the sidecar's artifact and vocabulary binding.

## Findings to repair

The exact-head review at `c04c5139b` found:

1. The code records `inputs.content_readout_sha256` and
   `settings.content_readout`, but no test reaches that report assembly. A
   deletion of those assignments would pass the added tests.
2. The content-readout test exercises only a matching sidecar. No integration
   test pins refusal of a sidecar bound to another artifact or vocabulary.

## Design

- Extract the two report assignments into a small helper used by `run`.
- Test that the helper adds the exact sidecar SHA and attachment marker, and
  adds neither when no sidecar is supplied. A mutation that omits either field
  must fail.
- Extend the content-readout sidecar fixture with negative cases:
  - the sidecar names a different artifact SHA;
  - the sidecar names a target outside the model vocabulary.
- Keep generation on `serve_step`, teacher-forced ranking on the CPU port, and
  turn-memory priming unchanged.

## Files in scope

- `docs/labs/one-model-end-to-end/instrument/src/bin/served-reply-panel.rs`
- `docs/loop/tasks/one-model-end-to-end/deepseek-42/**`
- `docs/research/notebook/float-training.md`

## Done when

1. The report-identity helper tests pass, and each report mutation fails.
2. Wrong-artifact and wrong-vocabulary sidecars are refused.
3. The focused suite, sealed CPU/integer parity, formatting, report/policy
   checks and gates pass at the repair head.
4. A fresh exact-head review approves the repair; the same PR merges and is
   verified on `main`.

## Stop and ask

Do not change the integer library, Claude's run, any bar or scope, or the
already-reviewed generation behavior. Do not touch a pod or sealed result.
