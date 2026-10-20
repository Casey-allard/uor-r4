# #2315 — Make negative findings queryable: settle the mesh question as a recorded assumption (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T07:10:10Z
- **Updated:** 2026-10-13T07:12:34Z
- **Closed:** 2026-10-13T07:12:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2315
- **Merged:** 2026-10-13T07:12:15Z
- **Merge commit:** 9f397e734dfbb6d16726f126a858b7484ac9bbe4
- **Base / Head:** main / UOR-Foundation:harness/assumption-findings
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling and research records only.

## Result

Decision: **KEEP** (tooling).

The objective's clause (d) — *"research the project's mesh concept and connect the coordination layer
to it"* — had **nothing** against it across seventeen rounds. This closes it, and the honest outcome is
a **negative result**.

**Re-verified on main rather than trusted from the earlier record.** A full-text search for `mesh`
returns only geometric and unrelated hits:

| source | what it is |
|---|---|
| `docs/native_geometric_mechanism_map_973.md` | TensorFlow **Mesh**'s MoE gating, a comparator |
| SpiralCore schema, mechanism map | nearest-neighbour / **simplicial** meshes |
| research simulations | `THREE.Mesh` |
| a vendored catalog | **service** mesh entries |
| `research/**/.vendor/numpy` | `meshgrid` |

No coordination mesh, no Tailscale or WireGuard mesh, and **the one daemon-shaped thing this
repository ever had — `lab-runner coord` — is explicitly forbidden to recreate** by
`docs/labs/protocol.md`.

**So the clause resolves to:** GitHub is the bus, `main` is the truth, and the harness adds what GitHub
alone does not — who holds which branch, and what each lab is doing between merges. Anything
daemon-shaped was ruled out **on policy grounds, not effort**.

### The gap was queryability, not the research

The finding lived only in a dated record, so **every lab asking "are we on the mesh?" re-ran the
search**. New **`class: assumption`** in the lesson registry: a belief that turned out false, kept for
the *opposite* reason to a mistake — so nobody re-derives it — with the same retrieval, the same four
sections, and `guard: none`, because a negative has nothing to intercept and inventing a rule for one
would be a rule that checks nothing.

### Two entries seeded

**The mesh negative first**, with the search that settles it and a command to repeat it.

**Then portability: interpreter discovery, not the Python version.** Measured minutes apart:

```
agent shell   python3 -> 3.15.0
login shell   python3 ->  3.9.6
absolute path          ->  3.9.6
```

The same machine reports two versions, and that is what made the silent `Z`-suffix parsing bug possible
on the declared 3.9 floor. **Do not raise a floor to dodge a compatibility bug** — a floor is a promise
to the labs, and raising it drops whichever lab is least able to notice.

Both retrieve by file, by topic, and by their own question.

```
python3 -m unittest                  -> 315 passed on 3.15 AND 3.9
python3 scripts/harness/cli.py check -> 51 rules, 0 errors
python3 scripts/check_no_softmax.py  -> OK (1648, unchanged)
python3 scripts/harness/parity.py    -> all nine comparisons ok
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Record: docs/labs/multilab-harness-2026-10-12/README.md
- [x] Branch deleted after merge

### Stated rather than hidden

- **This is a negative result, so it changes no behaviour.** It makes an existing conclusion findable.
  A reviewer should judge it on whether the *retrieval path* is right, not on whether new capability
  appeared.
- **The registry's curated cap was hit** (20 → 22 entries). Raised to 24 with the rationale recorded in
  the test, plus a substantive check that every entry names evidence — a count cap without that is a
  number that says nothing. **Findings still compete with lessons for the same budget**, which is a
  design smell I have not fixed; if findings accumulate, they want their own store.
- The mesh entry says `guard: none`. That is correct, but it also means **nothing enforces the
  negative** — a future lab can still propose a mesh, and will only find out if it runs
  `mistakes find`.
- Clause (a), GitHub reconciliation, is **still Python-only**. One round left.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

