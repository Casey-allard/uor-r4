# #1373 — Causal continuation: emitted-token repair, exact training resume and compiled sparse readout

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-24T01:05:14Z
- **Updated:** 2026-10-09T13:17:08Z
- **Closed:** 2026-09-24T01:08:01Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1373
- **Merged:** 2026-09-24T01:08:01Z
- **Merge commit:** 48106e7c0dd112c94f44bd535141002286ca92a1
- **Base / Head:** main / UOR-Foundation:codex/causal-geometric-continuation-20260924
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Causal geometric continuation

References #820, #973. The terminal objective is unchanged: fully transformerless geometric language modeling, exact ownership, and learned geometry replacing floating-point matrix multiplication at serving.

### Implemented and exercised
- Training-only crossed Copy-feedback supervision, unchanged default runtime ownership, and a candidate reaching16/16 crossed cases including four gradient-withheld combinations; matched ordinary training remains8/16. Historical temporal32/train36/class4/held-out3 remain retained.
- Versioned exact latent/Adam/configuration/RNG/data/schedule checkpoints. A separate process resumes step128 to256 and reproduces final checkpoint/model/selection/generation bytes and records.
- Broader native training on new prose/Rust sources plus finite dialogue/code supervision; a frozen three-source evaluation distinguishes domain adaptation from causal supervision and retains failed generations.
- Learned finite relative Q8 actions and a strengthened same-information ordinary signed-permutation control. Both reach1,024/1,024; the initial weak ordinary result is not claimed as geometric superiority.
- Compiled sparse readout into caller-owned buffers. All4,196,352 tested scores and complete generations match per run; idle same-model prompt/generation loops measure about10.4x faster. The successful readout allocation census is zero; recurrence still allocates.

### Main boundaries
Causal-model repository loss6.34326 regresses from parent6.27371; final-source loss10.61422 improves from11.47179 but trails ordinary broader training10.59886. All four generated Rust programs fail compilation, all four dialogue cases fail exact output, and no new application default or general-language qualification is promoted. Withheld crossed combinations participate in an exposed selection gate; no untouched causal final-set claim.

The inspected readout symbol has no enumerated multiply/divide/floating arithmetic instructions; this is not a whole-binary or energy certificate. Sparse plan metadata adds memory rather than shrinking the stored model. The Q8 synthetic learner is separate from the lexical model's numerical path.

### Verification, provenance and preservation
-61 focused tests rerun with binary identities;11 completed report roots plus the retained candidate root passed the native manifest verifier.
- Real-data step128-to256 resume reproduces latent optimizer bytes, served/selected model bytes, validation history and generated text exactly.
- New final sources and selection gates were declared before scoring; control weaknesses, text-loss tradeoffs and generation/compiler failures are retained.
- Two clean redundant worktrees were removed only after checking zero modified/untracked/ignored files and preserving commit-bearing refs. Physical free space remains approximately54.7GB at the resource checkpoint.
- Owner tracked edits are unchanged.54 of55 snapshotted hashes match; a live .omo continuation record changed independently and is disclosed, not restored.
- The historical CI names are compatibility acknowledgements, not execution of their named suites. Actual local executed verification supplies evidence.

The new handoff names the retained candidate, complete checkpoints, experiment binary, dataset locations and exact replay command. A fresh Git clone still requires the ignored local model/data store. No programme issue is closed by this component delivery.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

