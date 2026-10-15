# #2371 — Make the port's coverage a checked fact, not an assumption (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T20:32:15Z
- **Updated:** 2026-10-13T20:34:25Z
- **Closed:** 2026-10-13T20:34:16Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2371
- **Merged:** 2026-10-13T20:34:15Z
- **Merge commit:** e0ee0f3081ea838132aeb1f8b699da82d2628b65
- **Base / Head:** main / UOR-Foundation:deepseek/rule-inventory
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**The coverage guard.** Tooling only.

Decision: **KEEP** (tooling).

## The defect this closes

While porting the `harness` rules, **the registered total moved three times in a single unit** because **other labs were registering rules into the same modules while the port ran.**

**A module is not "done" because it matched Python at one instant.** If a rule is added afterwards, the Rust module is short a rule and **nothing reports it** — the port would look complete while checking less. That is this line's recurring failure — *a guard that stops firing is indistinguishable from a guard that passes* — applied to the port itself.

## The guard

**`uor-harness check-rules [--json]`** reports what Rust implements, from `ruleinventory.rs` — **the port's own coverage record**, edited in the same commit that implements a rule.

**`parity.py` compares it against the rules Python registers**, failing **both ways**:

- a registered rule with no Rust implementation — the port is behind;
- **a declared rule Python does not register** — *a declared rule that does not exist inflates coverage*. **That is the direction that flatters the port and the one nobody would think to check.**

```
rule inventory: 8 implemented (harness 3, prep 5); 32 Python rule(s) still in Python
```

## Verified in both directions

A coverage check that has only ever passed is one nobody has seen work:

```
normal                     -> 8 implemented, 32 to go, exit 0
with a phantom rule        -> FAILS, exit 1:
    "the Rust rule inventory declares rules Python does not register:
     harness.this_rule_does_not_exist - a declared rule that does not exist inflates coverage"
```

**The phantom probe is the one that matters**: a missing implementation is visible as a low count, while a phantom is **invisible** — it makes the count look complete.

## Deliberately a note while the port is partial

The port **is** partial by design; failing CI for that would block every other lab for the duration. The coverage line prints on every parity run and **becomes a failure the moment `check` stops delegating** — the only claim it exists to protect. Until then, **the delegation notice on every `check` call is what keeps this honest.**

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 225 passed (4 new)
python3 -m unittest discover -s tests/harness   -> 319 passed
python3 scripts/check_no_softmax.py             -> OK, no softmax added
parity.py                                       -> ok
```

Record: `docs/labs/rule-inventory-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **The inventory is maintained by hand.** A rule implemented without being listed under-reports — the safe direction, but still wrong.
- **An id listed without an implementation is the failure that would matter, and no test catches it.** `parity.py` catches a phantom *id*, not a phantom *implementation*. The inventory cannot know whether a function exists.
- **`python_registered_rules` imports `checkrules` in-process.** If that import fails, the coverage line is **skipped silently and parity passes** — a broken Python import disables the check without saying so.
- **`check-rules` is a new `PORTED_COMMANDS` surface** that does nothing a user asked for; it exists for this check.
- **It checks that an id is *claimed*, not that behaviour matches.** The per-module agreement measurements stay manual, per port.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

