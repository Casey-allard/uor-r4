# #1114 — #973: Fit contextual retained value writes

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-04T13:46:49Z
- **Updated:** 2026-09-04T13:49:52Z
- **Closed:** 2026-09-04T13:48:33Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1114
- **Merged:** 2026-09-04T13:48:33Z
- **Merge commit:** a72f082eff8bce359cfbdad1472793cad3c93b52
- **Base / Head:** main / UOR-Foundation:codex/973-contextual-fit
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The contextual retained-write path was executable, but its weights still came from the earlier token-only write law. This adds one bounded warm-start fit that trains the existing 252,160 parameters through `Wv(RMSNorm(x_t + strict_prior_retained_residual))`, writes a separate artifact, and lets direct generation select and verify that fitted artifact.

The fit is deliberately capped at 128 updates, 2,048 open training windows, four CPU threads, and an 840-second hard wall. It does not open validation, sealed, teacher, or source-model files. The historical V1 artifact remains unchanged.

Measured direct result:

- completed 128/128 updates over 245,760 causal targets in 97.098266 seconds
- all 24 parameter tensors had finite, nonzero gradients on the first update
- the optimizer included all 252,160 parameter values
- fitted artifact CID: `blake3:909ac6647a1f923da5523509cf7a6abf03d827da755d0d852ede12172476a76d`
- first/last batch loss: 3.973213 / 4.034052; these are different training batches and are not an evaluation

The same two direct generation checks produced:

- `A purple turtle found a clock in the garden` → `.\n"Thank you you, "Yes, I is be proud of you`
- `Albert Einstein was born in` → ` the dog. He said, "Hi, I are so she put it?"`

The bounded fit and source-free generation mechanics work. The outputs remain incoherent and factually wrong, so this does not establish convergence, quality improvement, coherence, factuality, reasoning, or general chat capability. No retry, tuning pass, validation run, or architecture expansion was performed.

Checks:

- `python3 -m py_compile` on the new fit path, generator, and CLI
- `git diff --check`
- one bounded contextual fit
- the two named direct generation commands using the fitted artifact

Broad suites were not run because the active build-first policy calls for the smallest direct behavior check for this decision; the protected merge queue remains the binding repository gate.

References #973


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

