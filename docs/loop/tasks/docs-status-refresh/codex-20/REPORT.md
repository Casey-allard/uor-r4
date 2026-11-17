# REPORT for the November publication and tracker closure

Lab: codex. Item: [docs-status-refresh](../../../../roadmap/items/docs-status-refresh.md). References #3380. [ORDER](ORDER.md)

## What did not work

Before the November publication, the public catalog held only the synthetic fixture and the generated STATE still placed M1 in progress. The earlier model-only URL import lacked a bound text tokenizer. The public loader, publication and protected render supplied separate receipts for these gaps. The Mermaid version warning remains tracked under [#3972](https://github.com/UOR-Foundation/uor-r4/issues/3972).

## What changed

The top session approved marking the documentation refresh done after fresh-main verification of the protected PM-cover render. The original dates, six acceptance criteria and historical gaps remain in the item.

## Measurements

- **Project status, README inventory and figures (criteria 1–3).** The October 31 refresh recorded the six-bar M1 close and dated reference overlap, inventoried every non-archive README, reviewed the hand-maintained figures and verified 8, 5 and 4 generated figures. The November 14 refresh checked the later FBS and Q31 claims. The November 17 guide added the written-state study and its measured count baseline. [October facts](../kimi-4/FACTS.md) · [README inventory](../kimi-4/INVENTORY.md) · [Figure inventory](../kimi-4/FIGURE_INVENTORY.md) · [#3436](https://github.com/UOR-Foundation/uor-r4/pull/3436) · [#3980](https://github.com/UOR-Foundation/uor-r4/pull/3980) · [#4060](https://github.com/UOR-Foundation/uor-r4/pull/4060)
- **Live tracker and rendered views (criterion 4).** The roadmap sync from merged #4060 updated 48 issues and the M1/M2 child checklists. Its second dry run reported five zero counts. All 156 issue states, closure reasons and assignees, the manual text on #2028–#2037 and the contributor/adoption PR identities matched the pre-apply snapshot. The four expected warnings preserve the not-planned closures #2846, #3450, #3462 and #3594. Protected PM-cover PR #4071 merged at `e52f26595ae9da20a85c99e25e106dfef4ecad4d`. The merged STATE, HANDOFF, LESSONS and AGENTS lesson block matched the approved content and the compiled renderer at input `e4264acf167c8552639d6f2ead4d523c1867b038`. STATE shows M1 complete on six bars and M2 in progress. HANDOFF gives #2850 no owner block. The roadmap index passed its separate working-tree check against the merged inputs. [Cover report](../../infra-loop-harness/codex-8/REPORT.md) · [Render review](https://github.com/UOR-Foundation/uor-r4/pull/4071#issuecomment-6961310293) · [Merged render](https://github.com/UOR-Foundation/uor-r4/pull/4071)
- **Site, models and review (criteria 5–6).** The reviewed guide passed 241 xtask tests and the pinned book build found 107 pages with zero broken links. The official production run published merged source `25b67e5c5` to Pages `59179ebe7afafcc125706373b3f8f1c0339239f5`. The live home linked the new illustration, whose downloaded bytes matched main at SHA-256 `bad52596b01b8e23bc9fc6acdb0010746091c13514efc02e165b2a726883cb1a`. The live product guide exposed **Predict next tokens**. On the published Studio, Chromium observed zero off-origin requests before the explicit load, then returned IDs 223, 952, 2359, 2471, 23, 263, 964 and 1520 offline for `The capital of France is`. The one-step CLI returned `[223]`. This checks one trained seed and prefix. [Production run](https://github.com/UOR-Foundation/uor-r4/actions/runs/44175919468) · [Published catalog](https://github.com/UOR-Foundation/uor-r4/blob/59179ebe7afafcc125706373b3f8f1c0339239f5/studio/models.json) · [Guide review](https://github.com/UOR-Foundation/uor-r4/pull/4060#issuecomment-6960932081) · [Guide report](../codex-18/REPORT.md) · [November facts](../claude-382/FACTS.md)

## Checks run at the head

Roadmap validation accepted all 156 items (31 ready, 11 held, 114 done). The regenerated index passed its equality check. The full read-only issue plan proposed one body update and one closure, both for #3380, with zero label creations, new issues or checklist changes. Its only warnings named the four previously retired configurations. The branch diff passed the whitespace check.

## Skills used

Used the repository's roadmap renderer and protected publication receipts.

## Still open

After this source change merges, sync #3380 from fresh main, confirm its live state and completion reason, then verify the global dry run again.

## Plain summary

The public guide, published model example, issue tracker and generated handoff have each been checked against their own inputs.
