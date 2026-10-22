# REPORT: m4-lawful-likelihood-bpb, deepseek-12 (repair of three residual record defects)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `386ff64d6936535b96b45bc9521d5b812818d8d3`. References #2861; item stays `ready`.

## What did not work, first

Answering the fresh-context CHANGES record at `386ff64d6`. **Correction (deepseek-13):** advisory, not counted — the `receipts.jsonl` files are class `other`, so the PR is not doc-only to the gate and a same-lab record is refused under D46; an other-vendor APPROVE at the head is required. Three record/wording defects, not design defects:

- **`README.md:61` cited a sentence that no longer exists.** It referred to "the sentence in the construction about 'the discrete crate can host the scorer'", which deepseek-10 had removed. Fixed: the crate paragraph now only says the pure function's home is the integer crate and the discrete crate integrates it into `eval-corpus` and its report — no backward reference.
- **The measured size was wrong.** The deepseek-11 REPORT and the PR body said "14 insertions / 13 deletions (measured)" for the note edit; `git diff --numstat 1c6c396cc..386ff64d6 -- docs/labs/m4-lawful-likelihood-bpb/README.md` is **11/10**. Fixed in the REPORT-11 file: the table row and the Size line both cite the exact command and the true figures. (The prior review had already flagged a size low; repeating it is the defect.)
- **A count was wrong.** The deepseek-11 REPORT said "the two remaining choices (held-out stream, calibration, family arms)" while listing three. Fixed to "three".
- **The owner-decision content and the design note are unchanged and confirmed good by both fresh reviewers.** No new design, code or measurement is claimed; the item stays `ready`.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `docs/labs/m4-lawful-likelihood-bpb/README.md` | docs | +1/−1 (measured by `git diff --numstat HEAD`) | remove the stale crate-sentence reference; state the crate split directly |
| `docs/loop/tasks/m4-lawful-likelihood-bpb/deepseek-11/REPORT.md` | docs | +3/−3 (measured) | correct the size to 11/10 with the exact command, and "two" to "three" choices |
| `docs/loop/tasks/m4-lawful-likelihood-bpb/deepseek-12/{ORDER,REPORT}.md` | docs | small | this repair order and report |

**Size:** the repair diff is 1 insertion / 1 deletion in the note and 3/3 in the REPORT-11 file (measured with `git diff --numstat HEAD` before committing); no source, test, generated file or item-status change.

## Measurements

- **No numeric result is claimed.** The design's arithmetic bounds are unchanged: `MAX_ABS_SCORE < 2^39`, V=4096, `sum(phi) < 2^52`, minimum mass 1, fixed-point bits bound `<= 2^-12 + 2^-48` per token.
- The corrected size figures are exact commands in the table above; nothing is estimated.

## Checks run at the head

- `UOR_LAB=deepseek uor-harness loop gates --full`: GREEN on five gates.
- `UOR_LAB=deepseek uor-harness loop report --check`: PASS.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: PASS on every rule (commit-order confirms deepseek-8 first alone and each repair ORDER added alone).
- `cargo fmt`/tests/clippy: no source changed; unchanged from the base.

## Skills used

- **solution-integrity**: a record's numbers are claims; measure them with the command and quote the command.

## Still open

- **Card 2** (`-log2(mass)` as a measurement) stands as read.
- **Open choices:** held-out stream name/hash, calibration for the 0.90 bar, family choice (PRLL vs polynomial kernel vs occurrence-count ratio).
- **Implementation unit** (pure scorer + seven finite host controls) and **registration unit** (pre-registration, non-author FIT, GPU scoring under owner release).
- **The item stays `ready`** after this PR merges.

## Plain summary

- **Three small records defects, all fixed.** A stale reference to a deleted sentence, a wrong measured size (11/10, not 14/13), and "two" where the list has three.
- **The design note itself is unchanged.** Both fresh reviewers said its content and the owner's Card 1 record are good; this repair makes the paperwork true.
- **Nothing was coded, run or measured**, and the item stays `ready` for its implementation and registration units.