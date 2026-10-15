# #1389 — Train an integrated recurrent geometric memory learner with matched control

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-24T23:30:00Z
- **Updated:** 2026-10-09T13:17:36Z
- **Closed:** 2026-09-25T06:14:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1389
- **Merged:** 2026-09-25T06:14:23Z
- **Merge commit:** 74a3530e7569b98d3139d2f194e516d42da1d458
- **Base / Head:** main / UOR-Foundation:codex/joint-recurrent-20260924
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Result and programme decision

References #973 and #820. Complete D8 rung 1 at its frozen engineering scope, and make quantization-aware continuation of this same learner the next rung.

Both quaternion and matched Householder-pair recurrent models completed **29,999,104 target visits / 7,324 updates**, including **21,381,120 visits at the corrected 256-token training/evaluation horizon**. The matched 64-token warmup, all optimizer state and every stopped/alternative candidate are retained. Both select final step 7,324 under the recorded tune-prefix rule, frozen before the comparison-tail evaluation.

| Comparison-tail NLL, nats/token | Read | Whole-prefix NoRead |
|---|---:|---:|
| Quaternion learner | 2.110368 | 2.592991 |
| Matched ordinary recurrent learner | 2.085241 | 2.561712 |
| Count/cache baseline | 2.391786 | — |
| Historical offline transformer reference | 1.574024 | — |

The exact Rust comparison joins all 249,856 targets and reproduces all 21 original partition means. Both learners improve retained training loss and produce varied loaded text. Actual stories still show semantic drift, role confusion, malformed words and repetition. The limited noncollapse judgement comes from principal and independent inspection of actual responses, not an invented numerical quality detector.

The fixed source-edit panel yields quaternion 28/32 versus ordinary 23/32 strict completions, but first-noun correctness is 28/32 versus 31/32. Both NoRead arms score zero. The prompts are only 56–62 tokens. NoRead removes value feedback and pointer copying together. These results do not establish distant exact recall, a geometric advantage, useful general chat, integer serving or energy savings. This is one paired seed on exposed development.

## Implementation

- Add one differentiable Rust recurrent graph connecting transport, state, causal contextual Q/K/V reads/writes and normalized vocabulary/pointer output. No transformer backbone or runtime teacher is introduced into this learner.
- Add named AdamW state, counter-based sample continuation, sealed checkpoints, actual loaded generation, whole-prefix controls and exact same-population comparisons.
- Correct the training horizon through a hash-bound explicit transition that retains weights, moments and global clocks. Resume rejects undeclared changes.
- Add synchronous whole-sequence CPU gradient partitioning: ordered weighted mean gradients before one global clipping/AdamW update. Two workers per arm with both arms concurrent beat the tested four-worker-per-arm and Metal alternatives on the M1. Sustained final fitting: approximately 1,362 / 1,365 targets/s.
- Pin the offline Candle 0.9.2 backend and correct four Accelerate operand slice lengths. The large vendor diff is the licensed upstream source import; `UPSTREAM.json` and `UOR-PATCH.md` bind every upstream file and the four executable line changes. Portable serving crates are not changed to depend on this backend.

## Executed validation

Executed release source: `ad4e639fedecf9d490035f9986be736117bd3e29`; binary SHA-256: `f68d654294607df7891e300d572d08bffa772606674c80cdaad2b95c7d122238`. Subsequent commits change documentation/configuration/evidence only.

- Release build with `metal,cpu-accelerate,reference-accelerate`.
- Eight focused `joint_` release checks: causal distributions, incremental/core agreement, full history gradients, transport arithmetic, language gradients, counter-window continuation, AdamW restart and every named gradient under CPU batch partitioning.
- Complete fitting, checkpoint save/reload, four actual population/generation evaluations, exact joined comparison and Rust report-file-set verification.
- Independent scientific and execution audits reconciled actual responses, selected hashes, all 15 modern resume records, all 23 learning curves, optimizer counters, data/tokenizer/evaluator/binary bindings and control scope.
- `cargo fmt --check`, `python3 scripts/check_claim_wording.py`, and `git diff --check` pass.

The five historical CI status names are compatibility acknowledgements that execute no formatting, Clippy or tests. They are not counted as the local checks above.

## Resources and preserved limitations

Final-phase peak sampled combined RSS: 6.01 GiB. Two early disk stops preserved checkpoints. An overly restrictive 26.1-GB stop was corrected prospectively to 20 GiB reserve plus 128 MiB stop margin and 64 MiB closeout headroom. Final-phase minimum available disk remained 25.03 GB. Validated inactive compiler cleanup reclaimed 3.65 GB physical space; all source, research, models, executed binaries, reports and worktrees remain.

Exact gross new compiler allocation is **UNRESOLVED**: current retained size and disk free space cannot reconstruct all overwritten/deleted compiler outputs. The result makes no gross-storage ceiling-compliance claim. Complete-cycle wall time is charged once to the shared ledger; summed concurrent process time is reported separately. A measured delivery tail follows the committed cutoff. No paid/external compute.

## Reviewable record and next step

- `docs/integration/joint-recurrent-result-2026-09-25.md`: interpretation, history, actual failure examples and next dependency.
- `docs/evidence/joint-recurrent-result-2026-09-25.json`: compact exact comparisons, hashes, all actual generated text and source-edit outcomes.
- Selection, hardware, budget and closeout receipts preserve the prospective decisions and interruptions.
- README, current state, canonical plan, source map, evidence index and training documentation agree on scope and the rung 2 dependency.

Next: specify and train the quantized forward path of these same selected learners, preserve language/read/output behaviour against continuous and ordinary controls, and inspect actual hard-path generation early. Bounded admission, final integer export, terminal parameter sparsity and physical energy remain later unqualified gates. Issues #973 and #820 remain open.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

