# Correction: a test asserted an exact document count, and it was mine

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the `harness.entry_point` port from two units ago

## What broke, and why CI caught it and I did not

`doccheck::tests::discovery_agrees_with_python_on_the_real_repo` asserted:

```rust
assert_eq!(docs.len(), 927, "python discovers 927");
```

**CI failed with `left: 928`.**

**The count is not a property of the code.** It changes **whenever any lab adds a Markdown file**,
and the repo gains them continuously — **including the record I was writing in that same unit.**
My local checkout was at 927 and CI checked out the merged head, so **the assertion passed here and
failed there**, which is the worst way for a test to be wrong: **it agreed with the tree I was looking
at rather than with the tree being merged.**

## The fix is not "update 927 to 928"

**A count that must be edited whenever a doc is added is not a test of discovery — it is a test of
whether someone remembered to update it.** So the number is gone and **what the rule actually
promises is asserted instead**:

- `AGENTS.md` sorts **first**, and the next two are `CONFORMANCE.md` then `CONTRIBUTING.md`;
- **the exclusions hold on real data**, not only in a fixture: no `docs/history/`, no
  `current-state.md`, **no dated `docs/labs/<topic>-<date>/` record**, and nothing outside the repo
  root and `docs/`;
- **every reported invocation is in a doc discovery returned** — the rule's own contract.

## This is the third time in this port that a fixture encoded a transient

The previous two were the `prep.rs` "python citation list" that was stale against the registry and the
`prlint` fixture captured **while the tree was dirty**, which encoded a finding that no longer
existed. **All three share one shape: an assertion that was true of the tree at the moment it was
written and is not a property of the code.**

**The first two I found by reasoning about them. This one CI found for me**, and the difference is
that **I asserted an exact number about a quantity other people control.**

## Verification

| check | result |
|---|---|
| the properties, on the real repo | asserted |
| every reported invocation is in a discovered doc | asserted |
| `cargo test -p uor-r4-harness` | **379 passed** |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | 332 passed |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0 |

## Limitations, stated

- **The assertion still names the second and third documents.** A new root-level Markdown file sorting
  before `CONFORMANCE.md` would fail it — **that is a real fragility, and I have moved it from a
  number that changes weekly to a name that changes rarely, not to something that cannot change.**
- **The properties are weaker than the count was.** The count would have caught a discovery that
  merged or split a doc; **the properties do not**, and nothing else does either.
- **`check` still delegates.** This unit fixes my regression and moves no coverage.
