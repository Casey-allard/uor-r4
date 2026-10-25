# REPORT: m1-bvg-k-stage-t-training, claude-107

Lab: claude. Order: [ORDER.md](ORDER.md). Decision: [DECISION.md](DECISION.md).
References #2836.

## What did not work, first

- **The unit is cancelled at its cancel probe.** Stage T does not run now. The D68 decision is that it waits on `m1-address-key-full-vector` (#2834). No code was written, no GPU was used, and the item stays open.
- **My first decision card got the evidence wrong, and its fresh decision review caught it.** The card read W2's H_mixed as blaming the learning law "in part", from the 989 rows the output map loses after admission. W2's own registered table says the opposite. Row 3, H_mixed, reads "do not change the learning law yet" (`docs/harness/prep/m1-read-code-capacity-2026-10-17.md:141`), and W2's README calls the 989 a bound, "not a decomposition of the mechanism". The card is rewritten on the registered routing.
- **`loop next` offered a unit its own item says is not warranted.** The item's warrant clause ("only warranted if W2 blames the learning law") is prose, and the harness ranks only on `depends_on` and status. So Stage T ranked first ("unblocks 12") after W2 had ruled H_law out. The fix is in the item file: the companion `infra-loop-harness` unit adds #2834 to `depends_on`.
- **The claim held the discrete crate for about an hour.** Stage T's scope holds `crates/uor-r4-discrete/**`, so this open order blocked every discrete item, #2834 among them, until it closes.
- **What the unit learned for whoever runs Stage T later.** The D40-audited runner (`attention_audited.rs`) runs K = 1 and seed 1 only (`validate_config`). Its `AuditMachine` assigns one family per read batch by K = 1 indexing, while a K = 16 read block can straddle the Combine/Relation gate boundary. So Stage T needs a block evidence mode in the controller, seed-2 A8 configs, the sheet's five Step 0 controls, and the checkpoint-restricted W2 tables before any arm runs. The sheet's "$12" covers GPU time only.

## What changed

Docs only, in this task folder:
- **ORDER.md** has the cancel probe. Four checks hold, and the warrant check fails.
- **DECISION.md** is the D68 card, with its two review rounds.
- **REPORT.md** is this file.

## Measurements

None. There was no run, no GPU and no model.

## Checks run at the head

- `loop report --check` and `loop pr-check --local`: PASS (the review-record and required-checks rules report N/A in a local run).
- **Fresh decision reviews of the card:**
  - round 1, CHANGES (on the "runs now" version);
  - round 2, APPROVE, no Required findings. Its advisories are taken; the stale PLAN.md:61 row is fixed in claude-109 (#3228).

## Skills used

- holding-the-goal: the goal is the M1 path, not this unit. The registered routing puts admission (#2834) ahead of a learning-law change, so the unit stops rather than build toward a run its own evidence does not warrant.

## Still open

- **The companion `infra-loop-harness` unit** adds `m1-address-key-full-vector` to Stage T's `depends_on`.
- **#2834** is the next M1 mechanism unit, once this order and its crate lock close.
- **When Stage T runs,** its decision review weighs #2834's result first, and drops or re-justifies the sheet's routing of a partial result to W4 (#2846), whose H_law condition has failed. The instrument gaps listed above are owed before any arm runs.

## Plain summary

The next planned training step changes how the model learns. The measurements so far say the bigger problem is which words the model even considers, and the plan's own rules say to fix that first. So this step waits until that fix is tried.
