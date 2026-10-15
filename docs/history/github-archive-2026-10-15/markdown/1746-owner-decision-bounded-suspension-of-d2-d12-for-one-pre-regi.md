# #1746 — OWNER DECISION: bounded suspension of D2/D12 for one pre-registered geometry-vs-matched-control contest

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-10-05T18:01:07Z
- **Updated:** 2026-10-05T18:02:40Z
- **Closed:** 2026-10-05T18:02:40Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1746
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# OWNER DECISION REQUIRED — Move 1: a bounded suspension so one contest can lose

**This is a request for owner authority, not a proposed edit.** The council ranked it first because **without it no experiment can change a decision**, and no agent can grant it.

## The problem, stated precisely

Four documents make it impossible for a mechanism to lose:

```
DECISIONS.md:158   "a geometric mechanism is not retired on a near-zero delta"
DECISIONS.md:192   "Never retire an enabler or selector on a near-zero or NEGATIVE delta.
                    Repair the wiring or change the instrument and re-measure."
DECISIONS.md:438   "Gates promote; they never kill."
DECISIONS.md:443   FAIL, DEAD and RETIRED now read "not yet promoted at that scope"
formal_vocabulary:99  "Architectural load-bearing is ASSUMED; held-out advantage remains unproven"
```

**D2's escape hatch nullifies D1's falsification rule**, because every mechanism can be classified `enabler` or `selector` and every miss re-attributed to wiring — *including negative deltas*. **D4 already records the consequence in the project's own words:**

> *"No geometric mechanism has beaten an information- and compute-matched ordinary control on any tested task in this repository."*

**And they were retained anyway.** The project's own PI-review names the result: mechanisms *"un-retired on policy, never re-tested — **the largest single risk to this programme**."*

## The proposal — narrow, time-boxed, reversible

**Suspend exactly three clauses, for exactly one contest, until it reports:**

1. `DECISIONS.md:158` — the near-zero-delta exemption.
2. `DECISIONS.md:192` — the *negative*-delta exemption. **This is the load-bearing one.** A mechanism that measurably *hurts* must be permitted to lose.
3. `DECISIONS.md:438/443` — "gates never kill" and the FAIL → "not yet promoted" relabelling, **for this contest only**.

**Everything else stays in force:** D11/D5 serving constraints, evidence discipline, resource admission, preservation of negative results, protected delivery. **Nothing is deleted.** The clauses are suspended, not repealed, and they resume the moment the contest reports.

**Scope limit — this is not a licence to retire the architecture.** The contest decides **one** question: *does the geometric score/read mechanism beat a parameter-matched ordinary control on a task where that control measurably fails?* It cannot touch the mission, the serving contract, or any mechanism not entered.

## The contest, pre-registered before any run

**Instrument.** `mqar-bench` — with **one mandatory addition**: `enum ArmSpec` currently has a **single variant** (`Stack`) and `arch: StackArch::Geometric` is **hardcoded** (`mqar-bench.rs:449-465, :570`). The bench has therefore **never compared the geometric stack to a parameter-matched ordinary model** — only to non-learned rules. **Adding a `StackArch::Transformer` arm at matched parameters is the whole instrument change**, and the arch already exists in the codebase.

**Arms.** geometry ON (Lorentz read, rotation on) vs geometry OFF (Dot/L2 read, rotation off), matched params, matched tokens, matched data, **≥3 seeds** (the assessment's own rule, which its own critique found violated at 2).

**Pre-registered decision rules — total and disjoint.** The completeness critique found the previous rules **defective**: Step 1's promote (≥10 gap) and kill (both ≤15) **could both fire**, and the 16–29 band had **no action**. These rules have no gap and no overlap:

```
Δ = geometric − matched ordinary control, in nats, on the held-out split

Δ ≤ −0.01 nats (geo WORSE)      -> geometric score mechanism RETIRED from the critical path
−0.01 < Δ < +0.01 (tie)         -> RETIRED: no advantage demonstrated at this scope
Δ ≥ +0.01 nats, p<0.05, ≥3 seeds -> PROMOTED, and the effect size is recorded as the claim
```

**Every band has an action and no band is empty.** A tie is a **loss**, not a retention — that is the specific change, and it is the one that makes the contest worth running.

**What retirement means, concretely.** The retired mechanism is removed from the **critical path** and from the **claim language**. It is not deleted: the code, the artifacts and the negative result are preserved (per existing policy). This is D12's "not yet promoted" with **one** difference: it stops being counted as a reason to keep building on it.

## Why this is safe to grant

- **It is bounded.** One contest, one instrument, defined rules, and the suspension lapses on report.
- **It costs almost nothing.** 1.36M-param bench models train in ~20 minutes on CPU; the arch already exists; the controls (`read=lorentz|dot|l2`, `RotationGroup::Quaternion|U1`) are already parameter-matched and same-kernel.
- **Both outcomes are useful.** If geometry wins, the project gains its first measured differentiator and a defensible basis for the whole architecture. If it ties or loses, the project stops spending on the class the synthesis names as **the three-month barrier** and redirects to the assets the council agreed are real.
- **It is reversible by construction** — the clauses are suspended, not removed, and the owner can end the suspension at any time.

## What I am NOT asking for

- **Not** a transformer in the product. The control is a **measurement arm in a bench**; D11/R4 serving is untouched. The owner's 3 Oct ban on transformer *baselines* is exactly the rule that makes this contest impossible, and **that is the one line I am asking to revisit — for one bench, one run, and only as a control.**
- **Not** a change to the mission, the goal, or the serving contract.
- **Not** an un-retirement or re-litigation of any historical result. Prior results keep their scope.
- **Not** a decision I can make. **This needs the owner.**

## If the answer is no

Then the honest statement is that **the geometry's advantage is unmeasurable by policy**, and the council's Move 2 (D5) should proceed alone — because D5 is the one claim that can still be tested without a control, by asking whether sparse access beats dense access **at equal quality**. That question does not require a transformer arm, which is why I am asking for Move 1 as a separate, optional grant rather than bundling it.

Refs #820, #1552, D2, D4, D12.


## Comments (1)

### comment 6000184057 — 2026-10-05T18:02:39Z — Casey-allard

## Decided and delivered — see PR #1747

The owner approved the change with a **clear framing** that is now written into the decision itself:

> *"geometric mechanisms will need to be prioritized and novel invention of geometric mechanisms will likely be needed, but if we have exhausted everything, we can suspend the rule until we have something working so we can measure and then we build the fix"*

**That framing is stricter than what I proposed, and in the right direction.** My draft asked for a bounded suspension; the owner's version makes the suspension **conditional on exhaustion**, which turns the five conditions from a schedule into a **precondition**, and adds an obligation I had not written: **novel geometric candidates must have been invented and tested** before exhaustion can be claimed.

### What changed from my proposal

| my draft | the owner's decision |
|---|---|
| suspend for one contest | **suspend only after exhaustion is recorded** |
| geometry priority restated | geometry priority **plus an invention obligation**: an `enabler` must name its missing component **and attempt to build it** |
| 5 conditions as a checklist | the same 5, now a **gate**: any one missing and the rule stays in force |

Condition 5 is the owner's addition and it is the one that closes the *un-retired, never re-tested* gap named in the project's own catalogue: **"we exhausted everything" cannot mean "every existing mechanism failed." It means the missing components were built and the new mechanisms were invented and measured.**

### Delivered

[**PR #1747**](https://github.com/UOR-Foundation/uor-r4/pull/1747) — `dad8ad8d`, documentation only, no model compute:

- **D20** appended to `DECISIONS.md`, quoting the owner's direction verbatim.
- **D2** and **D12** cross-referenced at the exact clauses bounded.
- **`formal_vocabulary.md:99`** — the `E8 = H4 × H4` *"load-bearing is assumed"* row now **schedules that assumption for test** under D20 §4 instead of standing as a permanent exemption.
- `check_claim_wording.py` **PASS**; 2 files, +139 −2.

**Closing this issue** — the decision is made and the delivery is in review.

Refs #820, #1746, #1747.



---
_Comment/review ids in this file: 6000184057_

