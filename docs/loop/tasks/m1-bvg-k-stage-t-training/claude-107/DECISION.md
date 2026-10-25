# DECISION (D68): Stage T waits on `m1-address-key-full-vector` (#2834)

Lab: claude. Unit: claude-107. Item: `m1-bvg-k-stage-t-training` (#2836). The fresh decision reviews are recorded below.

## The question

The item says Stage T "is only warranted if W2 blames the learning law (H_law)". W2 returned H_mixed, and H_mixed's registered next step, the code-utilization diagnostic, is done. That diagnostic's README leaves Stage T's gate to a decision review (`docs/labs/m1-w2-code-utilization/README.md:100`). Does Stage T run now?

## The decision

No. Stage T waits on `m1-address-key-full-vector` (#2834). It stays on the M1 close (D79), and its warrant is weighed again when #2834's result is in. An item-file unit adds #2834 to Stage T's `depends_on`, so `loop next` stops offering Stage T to any lab before then.

## Why

1. **The warrant clause is not met.**
   - W2's sheet sends H_mixed to "Code-utilization/capacity diagnostic; do not change the learning law yet" (`docs/harness/prep/m1-read-code-capacity-2026-10-17.md:141`, row 3).
   - It rules H_law out whenever T ≤ M + F (:33). W2's T is 3,414 against M + F ≈ 6,435, so 3,021 below (`docs/labs/m1-w2-capacity-capture/README.md:27-29`, and :54: "not a change to the learning law").
   - W2 ran the fallback regime, which biases against H_law (README:128). The diagnostic answers that bias: its registered data-scaling projection for the code table is 3,552, still far below the H_law line, so the full-TRAIN capture was declined (`docs/labs/m1-w2-code-utilization/README.md:57`).
   - Stage T changes the learning law, from K = 1 to the BVG-K K = 16 block law.
   - The 989 rows the output map loses after admission are, in W2's words, "bounds from one artifact, not a decomposition of the mechanism" (README:61). They cannot stand in for an H_law verdict.
2. **The registered next step points at admission, not the law.** The code-utilization diagnostic returned A1a and names #2834 as "the next M1 mechanism unit" (README:94). It also found that the trained address key admits fewer target cells (A_W 1,920) than a fixed list of the 8 most frequent cells (A_U 5,935), an address-objective finding "for #2834's decision review" (README:30). Admission is the larger measured loss: at least 2,276 rows against the output map's 989 (`docs/labs/m1-w2-capacity-capture/README.md:58-59`).
3. **A8's controller finding does not change this.** A8's K = 1 arms stopped on `consecutive_rejections`, classed instrument (`docs/loop/tasks/m1-a8-full-dose-rerun/antigravity-57/REPORT.md:7-9`). That is a real coupling, and Stage T's K = 16 blocks would test it. But it is a controller-configuration question, and W2's routing says the learning law is not the next thing to change on the M1 path.
4. **This is a sequence decision, not a scope change to the M1 close.** PLAN.md's step 1 (W2) unlocks both Stage T and #2834 (`PLAN.md:58`). Running #2834 first keeps D79 as it is: Stage T is still step 4, and nothing is dropped. Dropping Stage T from the M1 close would amend D79, which is the owner's call (owner log #2836 `6393771432`). That is not proposed here.

## Rejected

- **Run Stage T now, stating the clause is unmet.** A first version of this card proposed running it. Its decision review (CHANGES, below) found that it misread H_mixed as blaming the law "in part". With that corrected, running it now would set aside two pre-registered routings (W2's row 3, then the diagnostic's A1a) for a GPU spend of about $12 and a large instrument build: the audited runner is K = 1 and seed 1 only. The routings stand.
- **Leave Stage T `ready` and simply not claim it.** `loop next` ranks it first ("unblocks 12"), so another lab would take it against the same routing. The dependency records the decision where the harness reads it.
- **Run #2834 and Stage T side by side.** Both scopes hold `crates/uor-r4-discrete/**`, so `loop claim` refuses the second while the first is open. And the routing above puts #2834 first in any case.

## Not changed

- The Stage T sheet keeps its FIT. When Stage T runs, a fresh decision review weighs #2834's result first.
- The sheet routes a partial outcome to `m1-w4-float-credit-sheet` (#2846). PLAN.md runs W4 only if W2 gives H_law and W3 fails (`PLAN.md:58`), and W2 did not give H_law. Whoever runs Stage T should drop or re-justify that routing.

## To reverse

Remove `m1-address-key-full-vector` from Stage T's `depends_on`.

## Review

- **Round 1** (fresh context, at `0ebf886e4`, on the first version, "Stage T runs now"): **CHANGES.**
  - Required: the "law in part" reading contradicts W2's row 3 and its H_law rule.
  - Required: "tests the law share and nothing else" is false, because Stage T's bar also tests admission, the controller and the bigram.
  - Advisory: the W4 routing's condition has failed; the rejection of #2834-first did not acknowledge A1a.

  This version answers each finding.
- **Round 2** (fresh context, at `c0b8a7461`): **APPROVE**, no Required findings.
  - It found every cited line to match its source.
  - It judged this a sequence call a lab may make: D79 (`DECISIONS.md:283`) lists what the M1 close contains, not its order.
  - Its advisories are taken: the line cite for the 2,276/989 figures, and the side-by-side alternative. The third, that PLAN.md:61 still lists Stage T as needing only "steps 1 and 3", is fixed in claude-109 (#3228). Codex's open #3169 edits only PLAN.md lines 21–27, so the two do not overlap.
