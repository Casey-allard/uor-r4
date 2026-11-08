# ORDER: one-model-end-to-end, deepseek-25 (wait for the detached SFT job)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `43ab4b62495eb5dfd98183095f859a5a6bece0e1`.

## Goal

Repair the seed-1 reply-SFT controller before spending compute. `uor-pod run`
submits a detached remote job and exits after printing its log path. The
controller must poll that exact log through its terminal `# exit=N` footer,
keep renewing the lease, then accept only a zero remote exit and the sealed
store readback.

## Cancel probe

- `uor-pod run` is detached and writes the remote log path in its submission
  message.
- The job seal writes `READBACK=verified` only after upload and hash readback.
- `m2-reply512` remains unavailable; this unit does not substitute for it.

## Files in scope

- `docs/labs/one-model-end-to-end/reply-sft-2026-11-07/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-25/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-22/REPORT.md`
- `docs/harness/prep/one-model-end-to-end-reply-sft-2026-11-07.md`
- `docs/harness/claims.json`
- `crates/uor-r4-float-train/src/in_space_long.rs`

## Done when

1. The launcher treats only remote submission success as a successful submit.
2. It parses the actual one-line `uor-pod` success message and polls that log.
3. It fails on a missing log, a nonzero remote exit or missing readback.
4. Static gates and mutation tests prove submission-only waiting and ignored
   remote exits are rejected.
5. Shell syntax, ShellCheck, diff checks and CI pass before merge.

## Stop and ask if

Never launch a pod or weaken a stop, hash, readback or teardown rule in this
repair. Never touch Claude's pods or branch.
