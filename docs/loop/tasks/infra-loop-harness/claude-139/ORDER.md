# ORDER: infra-loop-harness, claude-139 (done-when audit)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance: the item's status and prose). Base: `168712ac3`.

## Goal

The item's done-when reads:

> Every PR in the build plan on [the loop page](../../loop/README.md#build-plan) is merged, each with its planted fault turning its gate red. The four labs (Kimi, Claude, DeepSeek, Antigravity) are onboarded, each with a committed onboarding receipt. Each lab has completed one loop task end to end: claim, probe, work, gates, report, review by another vendor, queue, and verify on fresh `main`.

**Check each clause with evidence, and decide the item's status.** Every build-plan item, `infra-loop-pr02` to `infra-loop-pr12`, is `done` on `origin/main`. `docs/harness/onboarding/` holds a receipt for each registered lab. Each of the four labs has merged loop units. So the bar may be met.

1. **Build plan.** For each row (A, 1 to 12), record:
   - the merged PR;
   - the planted fault named on the loop page;
   - the test or gate that holds it on `origin/main`.

   Run each of those tests at this unit's base.
2. **Onboarding.** Run `loop onboard --check-receipts` from a build of this base. It must report four receipts verified.
3. **One end-to-end unit per lab.** For each lab, record one unit that went claim → ORDER and probe → work → gates → REPORT → review → queue → verified on `main`, with its PR, review record and verify note.
   - **The review clause.** "Review by another vendor" was the rule when the bar was written. D66 (24 Oct) replaced it with the lab's own fresh-context exact-head review, and CHARTER §5.5 now says that. For each lab, record which kind its unit had. The charter wins over the bar's older wording.
4. **The item's own prose.** The title ("Build the five-lab loop harness"), "Why it matters" ("Five labs from five vendors …") and "What is missing" ("None of the loop tooling exists …") are stale. They are rewritten to match `main`, keeping the facts that are still true. This is lab-editable item text, not an owner path.
5. **Status, a D68 decision.**
   - **If every clause holds:** the item becomes `done`, the views are re-rendered, and the REPORT says `Closes #2904`.
   - **The decision review must weigh one cost.** Many units have used this umbrella as the vehicle for item-file maintenance: claude-106, -108 and -121, and antigravity-80 and -109. A `done` item cannot be claimed. The review says where that maintenance goes next, or whether the item should stay open for it.
   - **If any clause fails:** the gap is recorded, the status stays `ready`, and the gap becomes the next unit.

## Cancel probe (run first)

Cancel if the item is already `done`, or another open PR changes the item file.

To check:
- the item on `origin/main`;
- the open PRs' files.

Result (2026-10-27T01:55Z, `origin/main` at `168712ac3`):
- The item is `status: ready`.
- The open PRs are #3306 (kimi), #3312 (antigravity) and this unit's #3313. Neither of the others touches the item file.

**Not cancelled (instrument: none needed).**

## Files in scope

- `docs/roadmap/items/infra-loop-harness.md` (title, prose, status)
- `docs/roadmap/README.md` (the render, a shared file)
- `docs/roadmap/issues.json`, only if the render or the roadmap check needs it
- this task folder

## Done when

1. Each clause has an evidence row in the REPORT, and each build-plan row's planted-fault test has been run at the base.
2. The item's prose matches `main`. `xtask roadmap check` passes, and the render is current.
3. A fresh decision review, reading the whole project, approves the status call, with its record on #2904 as an owner log.
4. A fresh exact-head review approves the PR. `loop report --check`, `loop pr-check --local` and the required checks pass.

## Stop and ask if

Stop and decide; never wait.
- **A build-plan row has no merged PR, or no planted-fault test on `main`.** Record it, keep `ready`, and make it the next unit.
- **The decision review finds the umbrella is still needed** as a maintenance vehicle, and no other item can take that role. Keep `ready`, and record why.
