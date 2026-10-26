# REPORT: d11-exporter-float-trunk, claude-136

Lab: claude. Order: [ORDER.md](ORDER.md). This is the decision-relevance bound that the decision review after [claude-135](../claude-135/REPORT.md) required (#2820 owner log, comment 6426588305, change 4).

References #2820.

**Speed and cost.**
- No GPU and no model compute.
- It grades claude-132's stored, verified GPU receipts on the laptop, and it trains, evaluates and exports nothing.
- No tokens/s is measured; the artifacts' are in claude-129 to claude-131.

## What did not work, first

- **The exporter is not cleared, and this unit cannot clear it.**
  - The bound shows that export fidelity cannot change any of the six verdicts.
  - It does not show that the exporter is faithful.
  - The D-against-A leftover, at most 8 rows of net top-1 on these models, stays part of the export gap.
- **The three FAILs stand,** as configuration results, never a mechanism result (§4.1).
  - Even a perfectly faithful exporter (A = D) would leave every one outside its band.
  - What fails is the D11 forward's own gap from the float trunk, F − D, not the export.
- **The continuous-localisation GPU run that claude-135's decision review named is not run.**
  - Its question cannot change the item's verdict on these models.
  - This is that review's own stopping condition (change 4), its "neither" branch.
  - A fresh decision review approved ending the investigation here.

## What changed

Docs only: the ORDER, this REPORT and the notebook row. No code.

## Measurements

**The bound, per model,** from claude-132's six receipts in the store at `results/claude/d11-fbs-residual-2026-10-26/resid/`. All six check against the folder's `SHA256SUMS`.
- F is the plain float panel, D the D11 forward and A the artifact, each scored on the 14,287-row `ft/dev` panel.
- Each pairing is the receipt's own: `.pairs.float_vs_artifact`, `.pairs.float_vs_d11` and `.pairs.d11_vs_artifact`, with its paired F_I.

| Model | Top-1 F / D / A | F vs A net (F_I) | F vs D net (F_I) | F vs D, a faithful exporter | D vs A net |
|---|---|---|---|---|---|
| plain-c126 s1 | 1,988 / 1,911 / 1,907 | 81 (60) | 77 (60) | outside, by 17 | +4 |
| plain-c126 s2 | 2,643 / 2,290 / 2,294 | 349 (70) | 353 (69) | outside, by 284 | −4 |
| d11-c130 s1 | 3,941 / 3,666 / 3,663 | 278 (69) | 275 (68) | outside, by 207 | +3 |
| d11-c130 s2 | 917 / 1,257 / 1,249 | −332 (59) | −340 (58) | outside, by 282 | +8 |
| dual-c131 s1 | 3,845 / 3,486 / 3,492 | 353 (73) | 359 (72) | outside, by 287 | −6 |
| dual-c131 s2 | 3,128 / 2,903 / 2,897 | 231 (70) | 225 (70) | outside, by 155 | +6 |

- **Why the bound holds.** For each model, net(F,A) = net(F,D) + net(D,A), and every row agrees with the receipt (for example 81 = 77 + 4, and −332 = −340 + 8).
- **What it shows.** F against D is what the export gap would read with an exporter that computed the D11 forward exactly. It is outside its own F_I on all six, by 17 to 287 rows. So no exporter that computes the D11 forward exactly, or with no more net error than now, turns any FAIL into a pass. The reason: |net(F,A′)| ≥ |net(F,D)| − |net(D,A′)|, and that is at least 77 − 8 = 69 > 60 even on the closest model.
- **The leftover's size.** It moves each FAIL's net by at most 8 rows (D vs A, +8 to −6). That is against F_I of 59 to 73 for F vs A (58 to 72 for F vs D), and FAIL nets of |81| to |353| rows (F vs A).
- **After claude-135's forcing,** 5 to 139 rows still differ. Most are rows where unforced D and A already differed; 1 to 26 per model are new to forcing.

## Checks run at the head

See the exact-head review record on the PR.

## Skills used

- **holding-the-goal.** No outcome is reclassified and no bar moves. The bound is read for what it shows: export fidelity cannot change the verdict. It does not show the exporter is faithful.

## Still open

- **The next unit is claude-131's pivot note's decisive run,** under the fresh decision review recorded on #2820 (owner log, comment 6427050406). It is also M1's Path B (antigravity-105's DECISION), so it is on the M1 path. Its sheet must:
  - **Make one change only.** The candidate is longer dual-forward training at 1:1, since both DEV losses were still falling at step 1,024. Its step count and LR scaling are fixed before the run, and everything else stays byte-identical.
  - **State the risk.** Dual training raised F's top-1 without shrinking F − D.
  - **Compute the new models' own D panel and net(D,A).** A seed outside F_I by no more than |net(D,A)| sends the run to the continuous localisation before any "stop".
  - **Keep the guard and the seed-spread rule.** A PASS needs the artifacts' top-1 above 1,907 and 2,294. The loss band is read against 0.0230, and the top-1 spread against 655 rows.
  - **Pre-register every outcome.**
    - PASS on both seeds closes #2820.
    - A mixed-seed result is a FAIL.
    - A FAIL means "stop": the item parks by lab decision card, and the mechanism stays open.
    - An instrument failure is re-run.
- **The owner's reading of §3.7** (F_I against the two-seed spread) is on #2028. Nothing waits on it.

## Plain summary

- **The question:** whether a perfect version of the integer exporter could turn the three failed results into passes.
- **The answer: it could not.** Even if the integer file computed the reference exactly, all six models would still miss the bar. The gap comes almost entirely from the integer-precision forward itself; the export adds at most 8 words of net.
- **So the long check of the exporter ends here,** with the exporter not cleared but no longer able to change the verdict.
- **The next step** is the one decisive training run the plan set aside, with a single change.
