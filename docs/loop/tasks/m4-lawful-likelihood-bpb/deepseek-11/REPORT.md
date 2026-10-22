# REPORT: m4-lawful-likelihood-bpb, deepseek-11 (record the owner's Card 1 answer)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `1c6c396ccaec2f599bad738da8af2581eac726c3`. References #2861; item stays `ready`.

## What did not work, first

- **The note's central word was conditional and the owner has now decided it.** At `1c6c396` the note said "nothing here is called lawful until Card 1 is answered" — correct at the time, but the owner answered at about 04:15Z on 22 October 2026 in Claude's chat, recorded verbatim by Claude on #2028 at 04:17Z: "Yes, a linear ratio (Recommended)". Leaving the note conditional would make the repository record contradict a recorded owner decision.
- **The decision had to be quoted, not paraphrased.** The note now carries the answer verbatim with its time and the #2028 source, and says why: §3.2 bans exponential and squashing transforms; dividing non-negative potentials by their sum is lawful; the item itself asks for an exact alphabet sum.
- **Every conditional sentence had to be swept, not just the status.** Status, the "why" heading, property 6, the testable-properties heading, the decisions section and the calibration card all carried the old condition. Each is updated; the three remaining choices (held-out stream, calibration, family arms) stay open and are labelled as such.
- **No new design, code or measurement is claimed.** The construction is unchanged; this is a record update. The item stays `ready`.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `docs/labs/m4-lawful-likelihood-bpb/README.md` | docs | +11/−10 (measured by `git diff --numstat 1c6c396cc..386ff64d6`) | the owner's Card 1 answer quoted verbatim with its time and source; status, headings, property 6 and the decisions section made current; Card 2 stands as read; open choices stay open |
| `docs/loop/tasks/m4-lawful-likelihood-bpb/deepseek-11/{ORDER,REPORT}.md` | docs | small | this order and report |

**Size:** the note edit is 11 insertions / 10 deletions (`git diff --numstat 1c6c396cc..386ff64d6 -- docs/labs/m4-lawful-likelihood-bpb/README.md`); no source, test, generated file or item-status change.

## Measurements

- **No numeric result is claimed.** The design's arithmetic bounds are unchanged: `MAX_ABS_SCORE < 2^39`, V=4096, `sum(phi) < 2^52`, minimum mass 1, fixed-point bits bound `<= 2^-12 + 2^-48` per token.
- Checks at the head: `loop gates --full` GREEN on five; `loop report --check` PASS; `loop pr-check --local` PASS; CI green at the prior head and re-running.

## Checks run at the head

- `UOR_LAB=deepseek uor-harness loop gates --full`: GREEN on five gates.
- `UOR_LAB=deepseek uor-harness loop report --check`: PASS.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: PASS on every rule (commit-order confirms deepseek-8 first alone and each repair ORDER added alone).
- `cargo fmt`/tests/clippy: no source changed; unchanged from the base.

## Skills used

- **holding-the-goal**: record the owner's decision exactly, with its time and source, and keep every claim inside what the note actually did.

## Still open

- **Card 2** (`-log2(mass)` as a measurement) stands as the note reads it; nobody disputed it.
- **Open choices:** the held-out stream's name/hash, calibration for the 0.90 bar, and the family choice (PRLL vs polynomial kernel vs occurrence-count ratio).
- **Implementation unit**: the pure scorer and the seven finite host controls; **registration unit**: pre-registration, non-author FIT, registered GPU scoring under an owner release.
- **The item stays `ready`** after this PR merges.

## Plain summary

- **The owner decided the one question the note could not answer itself.** Casey said a linear ratio is lawful under §3.2 (quotational record on #2028 at 04:17Z), so the design note now states the construction is lawful under that reading instead of waiting.
- **Only the records changed.** The construction, the safety properties and the finite controls are unchanged; nothing was coded, run or measured.
- **What remains is ordinary later work:** Card 2 already stands, the held-out stream and calibration are choices for the registration unit, and the implementation unit can start with the design now unblocked. The item stays `ready`.