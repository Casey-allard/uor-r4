# FIT review: corrected public CPU corpus restart equivalence

Lab: deepseek. Reviewer: fresh-context non-author Codex subagent.

- Source head: `5646c8eaf2b956662413ef1232df74e61ab1ae9c`.
- Reviewed prep sheet SHA-256: `275450b83d1b85904bee5043b3e29555f39e84efd6ff348cfa802e94d5cc83a9`.
- Review mode: read-only, independent context, no training or tests executed.

## Verdict

**APPROVE with unavoidable conditions.** After this FIT, all six arms must run
against one reviewed source and binary identity. Checkpoint binaries remain in
the ignored local run root while sidecars, receipts and hash inventories are
committed. Any binding, artifact, checkpoint, panel, admission or export
mismatch must fail closed.

A pass is limited to the step-64-to-256 host determinism boundary on these two
public development artifacts. It excludes crash consistency, capability, GPU
qualification and #3506 closure.

Earlier `6fd2630c…` and pre-FIT receipts are retained as diagnostic context only.
