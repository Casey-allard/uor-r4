# #1747 — D20: geometry stays first; suspend the no-loss rule only after exhaustion

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T18:02:30Z
- **Updated:** 2026-10-09T14:16:48Z
- **Closed:** 2026-10-05T18:18:47Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1747
- **Merged:** 2026-10-05T18:18:47Z
- **Merge commit:** 86bec270c960554070921a0274c9bf21dae80956
- **Base / Head:** main / UOR-Foundation:geometry-priority-suspension
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## D20 — geometry stays first; the no-loss rule is suspended only after exhaustion

Owner decision, 2026-10-05. **Documentation only: no model compute, no new numerical result, nothing deleted.**

### The owner's direction, quoted in the decision

> *"geometric mechanisms will need to be prioritized and novel invention of geometric mechanisms will likely be needed, but if we have exhausted everything, we can suspend the rule until we have something working so we can measure and then we build the fix"*

### Why

Four clauses made it impossible for a mechanism to lose:

```
DECISIONS.md:158   "a geometric mechanism is not retired on a near-zero delta"
DECISIONS.md:192   "Never retire an enabler or selector on a near-zero or NEGATIVE delta"
DECISIONS.md:438   "Gates promote; they never kill."
DECISIONS.md:443   FAIL, DEAD, RETIRED now read "not yet promoted at that scope"
```

**D2's escape hatch nullified D1's falsification rule** — every mechanism can be called an `enabler` or `selector`, and every miss re-attributed to wiring, *including negative deltas*. **D4 already records that "no geometric mechanism has beaten an information- and compute-matched ordinary control on any tested task in this repository"** and they were retained anyway. The project's own catalogue names the result: items un-retired on policy and never re-tested is *"the largest single risk to this programme."*

### What D20 does

**1. Geometry first — and the expectation is raised.** Novel invention is **project work, not a diagnostic excuse**. An `enabler` now carries an obligation to **name the missing component and attempt to build it.** That obligation is what closes the un-retired-never-retested gap.

**2. "Exhausted everything" is defined objectively**, as five recorded conditions: matched ordinary controls for **every** ledger mechanism (not component results, not non-learned-rule comparisons, not unreached code paths); **≥3 seeds**; **wiring verified REACHED in a real run** by a firing counter or equivalent; a root-cause case **naming what would change the answer**; and — explicitly per the owner — **novel geometric candidates invented and tested to the same standard.**

> Condition 3 is not a formality. Two recorded examples: a precedence guard written into an **unreached** policy arm, whose "no effect" measurement meant nothing; and a closed-label fallback that was reachable, gated, gate-exact, and **fired 0 times in 60 conversations**.

**3. Exactly three clauses are suspended, and only after (2) is recorded** — the near-zero exemption, the negative-delta exemption, and "gates never kill" with its FAIL relabelling. Purpose is the owner's: *"so we can measure and then we build the fix."* **Nothing else is suspended**: D11/D5 serving, evidence discipline, ≥3 seeds, resource admission, preservation of negative results, protected delivery.

**4. One contest, pre-registered, with rules that are total and disjoint.** The completeness critique found the old rules **defective** — Step 1's promote (≥10) and kill (both ≤15) **could both fire**, and the 16–29 band had **no action**:

```
d = geometric - matched ordinary control, held-out nats

d <= -0.01                      -> RETIRED from the critical path
-0.01 < d < +0.01   (a tie)     -> RETIRED: no advantage at this scope
d >= +0.01, p<0.05, >=3 seeds   -> PROMOTED; the effect size becomes the claim
```

**A tie is a loss.** That is the narrowest change that can produce an answer.

**5. It lapses automatically on report** — a promotion resumes the no-loss rule with the winner carried forward; a tie or loss resumes it with the **measured defect named**, and the fix is built for *that* instead of another mechanism stacked on top.

**Retirement means removal from the critical path and from claim language — never deletion.** Code, artifacts and negative results are preserved.

### The instrument change the contest requires, named in the decision

`enum ArmSpec` has a **single variant** and `arch: StackArch::Geometric` is **hardcoded** (`mqar-bench.rs:449-465, :570`). **The bench has never compared the geometric stack to a parameter-matched ordinary model** — only to non-learned rules. That, plus a `StackArch::Transformer` arm at matched parameters, is the whole change; **the arch already exists**, and the `read=lorentz|dot|l2` and `RotationGroup::Quaternion|U1` controls are already parameter-matched and same-kernel.

### Also in this PR

- **D2 and D12 cross-reference D20** at the exact clauses bounded.
- **`formal_vocabulary.md:99`** — the `E8 = H4 × H4` row's *"architectural load-bearing is assumed"* — now **schedules that assumption for test under D20 §4** rather than standing as a permanent exemption. **The vocabulary's distinction between structural priority and measured advantage stays normative.**

### Checks

```
python3 scripts/check_claim_wording.py   PASS
git diff --stat                          2 files, +139 -2
```

**Reverting D20 would restore the prior rules and would not invalidate new evidence, delete artifacts or restore obsolete serving exceptions.**

Refs #820, #1552, #1746.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6000369652 — 2026-10-05T18:14:04Z — Casey-allard

## Correction at `798cf6da` — a comparison may NEVER rule out geometry

**The owner corrected this PR's first draft, and the correction is the most important line in the decision:**

> *"comparison ruling out is AWFUL for real research so do not let the test claiming a transformer did better rule out the geometric options... we need to make the geometry work first, then we can tune it to beat the transformers"*

**What I got wrong.** The first draft proposed *"a tie is a loss → RETIRED from the critical path."* That would have converted the project's own diagnostic instrument into an **elimination machine**, at exactly the stage where a matched-control comparison measures the **missing component** rather than the mechanism's ceiling. The owner rejected it, and it is now replaced.

### §4 — elimination rules → DIAGNOSTIC bands

```
d >= 0          -> the mechanism WORKS. Promote it, then begin competitive tuning.
d < 0 (a gap)   -> the mechanism is INCOMPLETE. The gap is a LOCALISER, not a verdict.
                   Required output: the missing component, NAMED, with the evidence that
                   identifies it, and the next attempt to build it.
                   RETIREMENT IS NOT AN AVAILABLE OUTCOME.
```

and an unqualified guard, on its own line:

> **No result from this or any other comparison retires, parks, demotes or deprioritizes a geometric option.** A geometric mechanism that loses a matched-control comparison is **incomplete**, and the correct response is to identify and build the missing component and re-measure — never to eliminate it.

**This is what the prior art says too**, and it was already quoted in the D5 memo: comparisons that pit a mechanism *lacking* a component against one that *has* it **measure the missing component, not the mechanism.**

### §5 — the exit is a WORKING mechanism, not a competitive win

In the owner's words, *"until we have something working so we can measure and then we build the fix."* **"Something working" now means a working geometric mechanism.** A repair that cannot be completed inside the window lapses the suspension with **the missing component named**, and the mechanism stays a **live candidate** — *a pause, not a retirement.*

### §3 — restated, because with elimination off the table the suspension needed its real purpose said plainly

`D2:192` (*"repair the wiring or change the instrument and re-measure"*) plus `AGENTS.md` (*"a failed experiment does not demote the whole architecture"*) together mean **a gap produces no obligation.** Suspending them makes a gap **BINDING**: it must yield a **named missing component and a repair attempt**, recorded, instead of quietly deferring.

**So the suspension's function is to convert a null into a REPAIR PLAN** — the owner's *"so we can measure and then we build the fix."* **It does not, and cannot, retire anything.**

### §6 — corrected

D12's *"never kill"* is **unchanged in substance and now reinforced**. A comparison cannot kill a geometric mechanism.

### The research order, now binding

**1. Make the geometry work. 2. Then tune it to beat the transformers.** Asked before the mechanism functions, the competitive question measures the wrong thing — which is precisely the three-month error this decision exists to stop repeating.

### Checks at `798cf6da`

```
python3 scripts/check_claim_wording.py   PASS
2 files changed, 167 insertions(+), 2 deletions(-)
```

Refs #820, #1552, #1746.


### comment 6000428710 — 2026-10-05T18:17:45Z — Casey-allard

## Exact-head self-review — `798cf6da084c9612c55f8602ddeed6344aaccb43`

Self-review is permitted under the owner's current exact-head rule. Checks run locally at this head; `local == remote` verified.

### Checks

```
1. python3 scripts/check_claim_wording.py                     PASS
2. git diff --name-only origin/main..HEAD                     docs/integration/DECISIONS.md
                                                              docs/formal_vocabulary.md   (2 files, no others)
3. anchors referenced: 3, broken: 0                           d9--, d11--, d20--
4. D20 guard lines present                                    4/4
5. elimination rules surviving the owner's correction          0
6. D2 + D12 both cross-reference D20                          2
7. markdown: code fences balanced (2), table rows well-formed  PASS
8. local head == remote head                                  798cf6da…  OK
```

### One defect found and fixed during review

**My first anchor checker reported all three internal links BROKEN — including `d11--native-multiplier-free-…`, which has worked on main for days.** That was the checker, not the document: GitHub replaces **each** space with a hyphen rather than collapsing runs, so `## D20 — Geometry` yields `d20--geometry` (two hyphens) and not `d20-geometry`. Corrected and re-run: **3 referenced, 0 broken.** Recording it because a checker that fails a known-good link is a broken instrument, and it would have produced a false "links broken" review.

### What this review verified by reading the diff, not the description

- **The owner's correction is load-bearing in the text, not just the PR body.** `§4` states the order — *"1. Make the geometry work… 2. Then tune it to beat the transformers"* — and carries the unqualified guard: *"No result from this or any other comparison retires, parks, demotes or deprioritizes a geometric option."* `§4` also records *"RETIREMENT IS NOT AN AVAILABLE OUTCOME"* and marks the superseded draft rule explicitly as **wrong**.
- **No elimination rule survives.** Grepping for the rejected `"a tie is a loss"` / `"RETIRED from the critical path"` returns only the sentence that supersedes them.
- **`§3`'s purpose is consistent with the correction.** With elimination off the table the suspension needed a real function, and it has one: `D2:192` plus `AGENTS.md` together mean *a gap produces no obligation*; suspending them makes a gap **binding** — a named missing component and a repair attempt. That is the owner's *"so we can measure and then we build the fix."*
- **`§5`'s exit is a working mechanism, not a win**, and a failed repair inside the window is *"a pause, not a retirement."*
- **`§6` reinforces D12 rather than weakening it**: *"A comparison cannot kill a geometric mechanism."*
- **`formal_vocabulary.md`** changes one clause only — the `E8 = H4 × H4` assumption is now **scheduled for test** under D20 §4. The vocabulary's structural-priority vs measured-advantage distinction is untouched.

### Scope

**Documentation only.** No code path, no model compute, no artifact, no existing decision deleted. D2's `enabler`/`selector` classes retain their meaning; D11/D5, evidence discipline, ≥3 seeds, resource admission and preservation of negative results are untouched. **Reverting would restore the prior rules without invalidating evidence or deleting artifacts.**

**Reviewing at `798cf6da`. Approving and merging.**



---
_Comment/review ids in this file: 6000369652, 6000428710_

