# ORDER: m4-lawful-likelihood-bpb, deepseek-12 (repair: three residual record defects)

Lab: deepseek. Item: [m4-lawful-likelihood-bpb](../../../../roadmap/items/m4-lawful-likelihood-bpb.md) (#2861). Base: `386ff64d6936535b96b45bc9521d5b812818d8d3`.

## Goal

Answer the fresh-context CHANGES record on PR #2940 at `386ff64d6`. **Correction (deepseek-13):** advisory, not counted — the `receipts.jsonl` files are class `other`, so the PR is not doc-only to the gate and a same-lab record is refused (D46); an other-vendor APPROVE at the head is required: (1) `README.md:61` still cites a "discrete crate can host the scorer" sentence that deepseek-10 removed — delete/rephrase; (2) `deepseek-11/REPORT.md` and the PR body claim the note edit is "14 insertions / 13 deletions (measured)" while the actual `git diff --numstat 1c6c396cc..386ff64d6` for the note is 11/10 — correct it to the measured figure; (3) `deepseek-11/REPORT.md` says "the two remaining choices (held-out stream, calibration, family arms)" while listing three — say three. Design content and the owner-decision record are confirmed good; these are record/wording fixes only.

## History

- deepseek-8 `0a4549689`: the design note; Claude CHANGES at `e9748f1`; deepseek-9 fixed the zero-mass/bits/division/crate/geometry items; fresh-context CHANGES at `effe1e9c1`; deepseek-10 fixed conditional lawfulness, the rank-weight precedent and the emissions claim; the owner answered Card 1 at ~04:15Z on 22 Oct, and deepseek-11 recorded it verbatim.

## Cancel probe (run first)

Class **instrument** (a design record). The CHANGES record is counted at the current head `386ff64d6`, so the repair is needed. All three findings are verifiable against the files: `README.md` no longer contains the sentence it references; `git diff --numstat` gives 11/10, not 14/13; the list has three items but the text says two.

## Files in scope

The item's seven globs and this task folder; this repair edits `docs/labs/m4-lawful-likelihood-bpb/README.md`, the task folder's REPORT-11 lines, and adds REPORT-12.

## Done when

- The stale crate-choice sentence in `README.md:61` is replaced by one that matches the note.
- REPORT-11's measured size and "two/three choices" wording are corrected to the true figures.
- REPORT-12 records the three defects and their fixes; the design note's content and the owner-decision record are unchanged.
- `loop gates --full` GREEN; `report --check` and `pr-check --local` pass; CI green.

## Stop and ask if

- A design-content change would be needed (none is; the reviewer approved the substance).
- A bar, status, constraint or spend change is needed; GPU work becomes implied.