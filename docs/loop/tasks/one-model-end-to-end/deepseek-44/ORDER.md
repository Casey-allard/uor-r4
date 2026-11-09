# ORDER: one-model-end-to-end, deepseek-44 (served-panel effect repair)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `59494646c`.

## Goal

Repair the third exact-head review of PR #3895 by proving the run-level
content readout reaches decoding, correcting the remaining module-doc
contradiction, and pinning CPU sidecar refusal through `run_with_args`.

## Findings to repair

At `c763d6304`, the run-level fixture confirms that the sidecar hash is
attached to the report, but it does not check any generated token. A run that
attached the sidecar for identity and decoded from a separate un-attached model
would pass. The module header also still says teacher-forced ranking uses the
same integer read as generation, while the implementation correctly keeps it
on the CPU and refuses integer ranking.

## Design

- Configure the run fixture's always-firing content readout to target token 42.
- Assert every serialized row's first generated ID is 42.
- Exercise a run-level mutation that decodes from a separately parsed,
  un-attached model; the row assertion must fail.
- Invoke the same run path with `--engine cpu --content-readout` and assert the
  named refusal without an output report.
- Correct the module header to say teacher-forced ranking stays on the CPU and
  is unavailable for the integer engine.

## Files in scope

- `docs/labs/one-model-end-to-end/instrument/src/bin/served-reply-panel.rs`
- `docs/loop/tasks/one-model-end-to-end/deepseek-44/**`
- `docs/research/notebook/float-training.md`

## Done when

1. The run-level success test proves sidecar attachment changes every decoded
   first token.
2. The detached-model mutation fails that test.
3. CPU sidecar refusal is pinned through `run_with_args`.
4. The module documentation is internally consistent.
5. Focused tests, sealed parity, formatting, report/policy checks and gates
   pass; a fresh exact-head review approves; the same PR merges and is verified
   on `main`.

## Stop and ask

Do not change the integer library, Claude's run, any bar or scope, or reviewed
runtime semantics. Do not touch a pod or sealed result.
