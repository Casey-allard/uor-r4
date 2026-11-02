# FIT review: public CPU corpus restart equivalence

Lab: deepseek. Reviewer: fresh-context non-author Codex subagent. Reviewed at
`2026-11-02T05:56:59-05:00`.

- Source head: `fc488513b`.
- Reviewed prep sheet SHA-256: `6fd2630cb2f874078ff897be3063bb412c14bda8e97fa5cf9e37f93547793ca8`.
- Review mode: read-only, independent context, no training or tests executed.

## Verdict

**APPROVE.** The control is fit as a bounded host determinism and
state-continuity check. Its two public artifact seeds, six process-separated
runs, exact artifact/checkpoint/panel/export equality bar and sidecar tamper
tests support only the stated step-64-to-256 checkpoint-resume claim.

The reviewer required the existing six-run comparison to remain diagnostic and
all six runs to be repeated after FIT against the reviewed source and binary,
with fresh identities and receipts. Until that repeat passes, no downstream
record may describe restart as complete or deterministic.

The claim remains separate from crash consistency, capability evidence, GPU
qualification and #3506 closure.
