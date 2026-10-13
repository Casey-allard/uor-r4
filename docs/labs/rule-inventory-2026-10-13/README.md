# The port's coverage is now a checked fact, not an assumption

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the gap named in `docs/labs/port-check-harness-2026-10-13/README.md`

## The defect this closes

While porting the `harness` rules, **the registered rule count moved three times in one unit — 54, 53,
then 55** — because **other labs were registering rules into the same modules while the port ran.**

**A module is not "done" because it matched Python at one instant.** If a rule is added afterwards, the
Rust module is short a rule and **nothing reports it**, so the port would look complete while checking
less. That is this line's recurring failure — *a guard that stops firing is indistinguishable from a
guard that passes* — applied to the port itself.

## The guard

**`uor-harness check-rules [--json]`** reports what Rust implements, from
`crates/uor-r4-harness/src/ruleinventory.rs` — **the port's own coverage record**, edited in the same
commit that implements a rule.

**`parity.py` compares it against the rules Python actually registers**, and fails **in both
directions**:

- **a registered rule with no Rust implementation** — the port is behind;
- **a declared rule Python does not register** — *a declared rule that does not exist inflates
  coverage*, which is the direction that flatters the port and the one nobody would think to check.

Current output:

```
rule inventory: 8 implemented (harness 3, prep 5); 32 Python rule(s) still in Python
```

## Verified in both directions

A coverage check that has only ever passed is a check nobody has seen work:

```
normal                          -> reports 8 implemented, 32 to go, exit 0
with a rule declared that
Python does not register        -> FAILS, exit 1:
    "the Rust rule inventory declares rules Python does not register:
     harness.this_rule_does_not_exist - a declared rule that does not exist inflates coverage"
```

**The phantom probe is the one that matters**, because a missing implementation is visible as a low
count while a phantom is invisible — it makes the count *look* complete.

## Deliberately a note, not a failure, while the port is incomplete

The port **is** partial by design, so failing CI for an incomplete port would block every other lab's
work for the duration. The coverage line prints on every parity run and **becomes a failure the moment
`check` stops delegating** — which is the only claim it exists to protect. **Until then, the rule that
`check` still delegates is what keeps this honest**, and that is stated on every delegated call.

## Verification

| check | result |
|---|---|
| `uor-harness check-rules` | 8 implemented (prep 5, harness 3) |
| `parity.py` coverage line | reports both counts |
| phantom probe | **fails parity, exit 1** |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **225 passed** (4 new) |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `python3 scripts/check_no_softmax.py` | OK, no softmax added |
| `parity.py` | ok |

## Limitations, stated

- **The inventory is maintained by hand.** Nothing stops a rule being implemented without being listed,
  and then the count under-reports — the safe direction, but still wrong. **An id listed without an
  implementation is the failure that would matter, and no test catches that**: the inventory cannot
  know whether a function exists. `parity.py` catches a *phantom id*, not a *phantom implementation*.
- **`python_registered_rules` imports `checkrules` in-process.** If that import fails, the coverage line
  is **skipped silently** and parity passes — so a broken Python import disables the check without
  reporting that it did.
- **The command is a new surface in `PORTED_COMMANDS`**, so it counts as ported while doing nothing a
  user asked for. It exists for this check.
- **It does not check that a rule's *behaviour* matches**, only that an id is claimed. The per-module
  agreement measurements are separate and manual, per port.
