# REPORT: one-model-end-to-end, deepseek-270 (reply-control bake-off)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Laptop only; no training, pod or spend.

**In one line:** no tested termination controller beats current greedy/cycle
on exact complete replies, so serving keeps its current rule.

## What did not work, first

- **V1 rules omitted the observed cycle mark from the cycle fixture.** The
  bake-off checker therefore could not test the declared input.
- **V2 rules used an active EOS rank in generic fixtures.** A fixed-rank
  candidate triggered during the cycle and cap tests, so those tests stopped
  isolating their named property.
- **V3 corrected both.** Generic rows use EOS rank 8, and the cycle row carries
  `short_cycle_period = 1`. V3 is the first scored bake-off.
- **The fixed and permeability candidates hit their committed kill lines.**
  They did not beat current exact-complete replies and truncated more
  references.

## What worked

The V3 socket, rules, control, six sibling candidates, scoreboard and pick are
committed in bake-off order. The checker verifies ancestry, frozen files, kill
lines, first-run times, rule coverage and the pick.

Every candidate decision uses trace features only. The reference is read by
the scorer after the controller returns. The control and candidates emit only
served tokens or EOS.

## What changed

- `bake-off/reply-control-v3.json` is the checked record.
- `bake-off/reply-control-v3/` holds the V3 socket, control, scoreboard and
  pick. Earlier V1/V2 documents remain as the rule-development record.
- The standalone instrument crate gains shared trace types, the frozen flow
  rule, the bake-off scorer and a controller module whose candidate versions
  live only on sibling branches.
- Serving code was not changed after the reply-control trace. The base current
  rule remains the pick.

## Measurements

Real arm-r seed-2 artifact:
`a1d705586b35d7da282913b1e21b242a74c0661b32d73e2ec89f56310a8140e8`.
Panel sample SHA-256:
`53d966dcfd63ff71e4b2d30f9af0fdbdc0607ca6c6d96c7d09aa4fb37e99a2e2`.
All arms score the same 1,141 cases and the same 48-token trace.

| Arm | Exact complete | Truncated | EOS position error |
|---|---:|---:|---:|
| copy-all control | 607 | 294 | 490 |
| current greedy/cycle | **607** | **290** | **490** |
| fixed EOS rank 2 | 577 | 342 | 521 |
| stability, zero margin | **607** | **290** | **490** |
| permeability-aware | 558 | 363 | 539 |
| shuffled feature | 271 | 712 | 846 |
| seeded random | 606 | 291 | 491 |

The stability predicate never fired because all trace margins were nonzero.
Fixed and permeability are kill-line hits. Shuffled and random are worse, so
the measured differences are not explained by arbitrary stopping.

## Checks run at the head

- Bake-off V3 check: `GREEN`.
- Bake-off V3 scoreboard: `GREEN`.
- Every candidate branch passes all 8 V3 rule tests.
- The dumb control passes 4 of 8; each passing rule is marked `TOO WEAK`.
- Instrument Clippy with `-D warnings`: pass.
- `git diff --check`: pass.
- Report check and local PR check run after the final committed head.

## Skills used

`bake-off` supplied the socket/rules/control/candidate/scoreboard/pick order
and the record checker. Its first two rejected rulesets directly caused the V3
fixture correction. `solution-integrity` kept the reference outside the
controller decision and required exact-complete replies as the primary metric.

## Still open

- Current retention is a measured negative for these three candidates on one
  seed and one panel, not a general retirement of reply control.
- A later controller must beat 607 exact complete replies and 290 truncations
  on a fresh held-out phrasing panel before touching serving.
- No controller was merged to a serving path.

## Plain summary

We compared six ways to decide when a reply should stop against the existing
greedy/cycle rule. None improved complete correct replies. The two rules that
changed stopping most often made results worse, and the random controls did not
reveal a hidden gain. The model keeps the current rule; only the diagnostic
bake-off tooling is added.
