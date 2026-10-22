# ORDER: m1-address-as-read, deepseek-27 (repair: the FIT review link and the verdict gaps)

Lab: deepseek. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md) (#2833). Base: `2d24cf1bb`. Item PR (not a loop setup PR).

## Goal

Repair the two CHANGES records on PR #2979 (Claude at `e7a6f06d`, Kimi at `2d24cf1bb`): the deepseek-26 ORDER sheet link is one `../` short and fails the docs gate, and the `NOT_FIT` verdict should name the measurement/stop-rule gaps Claude listed.

## Cancel probe (run first)

Class **instrument** (review record; no GPU). At `2d24cf1bb`: `docs/loop/tasks/m1-address-as-read/deepseek-26/ORDER.md:7` resolves to `docs/loop/harness/prep/...` (wrong by one level) and `xtask links` reports it as the one new broken link; the sheet Verdict section is bulleted rather than the plain shape `prep::render_fit` emits, and it omits the BPB stream/hash, the `0.01` tolerance source, the `>= 54/80` derivation and the bar-miss stop rule. Both reproduce. The task stands.

## Files in scope

`docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md`, `docs/loop/tasks/m1-address-as-read/deepseek-26/{ORDER,REPORT}.md`, `docs/loop/tasks/m1-address-as-read/deepseek-27/**`.

## Done when

- `ORDER.md:7` links `../../../../harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md` and resolves; `xtask links` shows no new broken link against the base.
- The sheet `## Verdict` names all five prerequisite groups in priority order, including the unpinned measurement definitions (held-out BPB stream unnamed/unhashed, unsourced `0.01` tolerance, no `>= 54/80` derivation) and the missing bar-miss stop/negative-closes rule.
- The recorded block matches the plain shape `prep::render_fit` emits (verdict line, `sheet:`, `reviewer:`, `note:`, `date:`, D22 closing line) and says it clears no compute.
- `prep-check` still reads `NOT_FIT` / `may_compute: false`; `report --check`, `pr-check --local`, `loop gates --full` and `--break all` pass; receipts at the final head; the `deepseek-26` REPORT line count matches the actual sheet diff.

## Stop and ask if

- Any GPU execution becomes necessary (it must not: the run is a separate unit behind the owner bar and release).
- A file outside the scope above would need to change.