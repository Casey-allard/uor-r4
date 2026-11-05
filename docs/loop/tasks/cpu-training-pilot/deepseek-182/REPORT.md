# REPORT: cpu-training-pilot, deepseek-182

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `5bd24915e`.

## Status

Complete protected release for merged PR #3732. The anchored writer-port audit
and its expected rejection of both unanchored public artifacts remain
unchanged.

## What did not work

Nothing in this bookkeeping unit. The original unit found no published
anchored training artifact, so its positive evidence remains synthetic; that
limitation is unchanged.

## What changed

- Recorded reviewed head
  `1052f92555ce0ce3402d0cc96a9b4c22eca7f1c0`, PR #3732 and protected merge
  `5bd24915e92f434d902c55b0db46063f5ab40851` in the deepseek-181 report.
- Recorded exact-head CI and fresh-main verification.
- Removed the completed `deepseek/writer-anchor-audit` and temporary release
  claims.
- Changed no model, serving, CUDA, float-inference, exporter or Claude-owned
  source.

## Measurements

- reviewed delivery head:
  `1052f92555ce0ce3402d0cc96a9b4c22eca7f1c0`;
- protected merge: `5bd24915e92f434d902c55b0db46063f5ab40851`;
- fresh-main CPU library: **340 passed**, 4 expected ignores;
- fresh-main CPU binaries: **101 passed**;
- fresh-main strict Clippy: passed;
- all required checks at the approved head: passed.

## Checks run at the head

- Independent exact-head review: **APPROVE**.
- All required PR checks at the approved head.
- Fresh-main CPU library, binary, strict Clippy and report checks.
- Claims JSON parsing and absence of both completed claims.
- Scope, evidence and `git diff --check`.
- Full loop gates and local protected-queue policy.

## Size

This release changes four documentation paths against base `5bd24915e`.

## Skills used

No system-provided Codex skill applies. Repository delivery procedures were
used.

## Still open

Run the audit against Claude's first exported anchored training artifact and
bind its hash/result to that run's evidence. A zero-mismatch audit establishes
export/read-back preservation only; it does not establish improved writer
admission or language quality.

## Plain summary

The anchored writer-port audit is merged and verified on main. This record
releases the completed implementation and release claims.
