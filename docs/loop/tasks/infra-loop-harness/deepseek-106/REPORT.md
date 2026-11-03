# REPORT: image-item record fidelity, deepseek-106

> **Superseded by [deepseek-107](../deepseek-107/REPORT.md).** A final review
> found that this report described the following commit as receipts-only even
> though that commit also updated this report's size line.

Lab: deepseek. Item:
[infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Order: [ORDER.md](ORDER.md). Base:
`9e0403b132da34752499c657def95dfce3235538`.

## What did not work, first

- The active report's prose said 8 files and 321 insertions while the exact
  cumulative diff was 9 files and 332 insertions.
- Its "checks at the head" wording did not distinguish implementation/record
  head `cf0cf69ad` from receipt-only `0f9fa0d20`.
- The item generalized two explicit issue requirements into weaker language:
  real-browser exact usage and truthful 501/413 errors.
- Copied PR-body Markdown retained relative links that resolve only inside the
  repository. This report also cited the unavailable `cargo xtask` alias.

## What changed

The item again requires exact usage in the real-browser flow, truthful 501
responses for unsupported endpoint families and truthful 413 resource/context
refusals. It retains the repaired integer API manifest scope and corrected
source link.

The superseded deepseek-105 report now records the exact cumulative size and
the runnable `cargo run -q -p xtask -- links` command. This active report uses
repository-absolute links where the PR body needs them.

The exact PR diff is 12 files: 446 insertions and no deletions.

## Measurements

No model or serving measurement applies. The registration remains one `ready`
item mapped to issue #3619.

## Checks run at the head

- The implementation/record commit immediately before the final receipts passed
  `cargo run -q -p xtask -- roadmap check`, focused link checks,
  `git diff --check`, `loop report --check`, `loop pr-check --local` and
  `loop gates --full`.
- The final commit after those checks adds only
  `docs/loop/tasks/infra-loop-harness/deepseek-106/receipts.jsonl`.
- `geometric-image-input` in `docs/roadmap/issues.json` maps to #3619.

## Skills used

No available Codex skill applied.

## Still open

- Implement the deterministic integer RGBA-to-R4 core.
- Add bounded native `uor_r4_image` ingestion and Studio Canvas input.
- Run real-browser, native/WASM and trained visual-understanding qualification.

## Plain summary

The image item's scope and links are now correct, but its recent records still
had stale size, ambiguous gate timing and weakened acceptance wording. This
repair restores the explicit exact-usage, 501 and 413 requirements and records
which commit carried the gates versus the follow-up receipts.
