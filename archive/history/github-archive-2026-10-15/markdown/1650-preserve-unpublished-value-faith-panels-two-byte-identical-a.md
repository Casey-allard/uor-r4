# #1650 — Preserve unpublished value-faith panels (two byte-identical aliases) and the dose-continuation launcher

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T16:28:27Z
- **Updated:** 2026-10-09T13:17:58Z
- **Closed:** 2026-10-03T19:42:11Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1650
- **Merged:** 2026-10-03T19:42:11Z
- **Merge commit:** 63ec83885cf4da3b8d81cc36dece9f4ea10c366b
- **Base / Head:** main / UOR-Foundation:codex/value-faith-panels
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Why

Branch/evidence reconciliation against live `main`. My corpus PRs — **#1641, #1642, #1645** — are **fully delivered**: verified by comparing **actual changed-file contents** against `main`, not by commit-ID history, since squash-merging leaves non-ancestor branches behind. `codex/copy-fidelity` diffs empty; `codex/wide-relation-corpus` is an older base whose generator `main` supersets (main carries #1641 + #1642 + #1645 combined).

Two pieces of genuinely unpublished material were left over. Both are preservation only.

## 1. Eleven unpublished value-faith panels (+ two byte-identical aliases)

`main` held only `value-faith-oracle-20261002.json` and its `.md`. The other eleven 10-row probe panels were never published, though their **run results already exist** (five `value-faith-*` objects in the iCloud result store) — so the missing half was the **source the results were computed from**.

Hashing them during delivery revealed something recorded nowhere:

| sha256 (first 12) | files |
|---|---|
| `364ea161032a` | `value-faith-A-original`, **`value-faith-parent`** |
| `80985a62f96e` | `value-faith-B-no-distractor`, **`value-faith-F-adjacent`** |

**Two pairs are byte-identical.** Preserved as found rather than silently deduplicated, because collapsing them would erase the fact that two condition names pointed at the same rows. Consequences, documented in the README:

- a table reporting **both members of a pair as independent conditions double-counts**;
- **"F-adjacent" cannot be contrasted against "B-no-distractor"** — same 10 rows.

Every result derived from these panels is a **phrase/value-presence** measurement, **not complete-answer membership**. The README records the distinction.

## 2. The dose-continuation launcher and its LR-restart finding

`scripts/launch-continue.sh` existed **only in two unpushed commits in a local worktree**. It produced this lane's best chat result (**pooled 2.059190 / first-four 1.389810**) and carries a measured finding published nowhere else:

> `init=` loads **weights only**. Adam moments restart at zero and the LR jumps to its warmup target, so a converged model is pushed by a fresh high LR before dose can help. Measured at `lr=0.002`, the model got **worse** over 1,000 further steps: dev NLL **2.304152 → 2.437755 (+0.1336)**. The continuation therefore runs at **`lr=0.0002`**, an order of magnitude lower, so dose dominates the restart perturbation.

That is a real trap: without checking step 0 against the winning arm's known final LM, the **+0.1336 restart artefact would have been reported as evidence *against* more dose**. It is also why this is a **continuation, not an exact resume** — the script says so explicitly.

Preserved on a fresh branch off `main` rather than force-pushed onto the old branch (which had diverged and would have needed `--force-with-lease`). Every `$HOME` path it references was verified to resolve before committing.

## Scope

Preservation only — **no scorer, model, kernel, or training change. No new fit.** 13 files plus 2 README/doc-bearing additions.

References #1512


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5972914583 — 2026-10-03T19:55:11Z — Casey-allard

## Local exact-head checks (per Claude's note: GitHub runner pool is saturated)

Head SHA: **`7f271ce68b3938fba47f06735b3c79c1b18df870`**
Checked out detached into a clean worktree at that SHA — no Rust is touched by this PR, so no cargo run applies.

### Check 1 — shell syntax
```
$ bash -n scripts/launch-continue.sh
exit=0
```
`scripts/launch-continue.sh` is the only executable file in this PR.

### Check 2 — the 11 panels this PR adds parse and are well-formed
```
$ python3 -c "<parse each added panel; require list of 10 rows with id + user_turns>"
ok  value-faith-A-original-20261002.json                      rows=10 sha=364ea161032a
ok  value-faith-B-no-distractor-20261002.json                 rows=10 sha=80985a62f96e
ok  value-faith-C-distractor-first-20261002.json              rows=10 sha=81fb120f1fe0
ok  value-faith-D-distractor-not-an-assignment-20261002.json  rows=10 sha=4c00a621b914
ok  value-faith-E-distractor-unrelated-relation-20261002.json rows=10 sha=cc592593c84f
ok  value-faith-F-adjacent-20261002.json                      rows=10 sha=80985a62f96e
ok  value-faith-G-reasserted-20261002.json                    rows=10 sha=f62441265702
ok  value-faith-H-two-relations-20261002.json                 rows=10 sha=649b0c29198d
ok  value-faith-control-20261002.json                         rows=10 sha=132b1c485829
ok  value-faith-heldout-20261002.json                         rows=10 sha=0074794cb949
ok  value-faith-parent-20261002.json                          rows=10 sha=364ea161032a
11 panels, 0 failures
exit=0
```

**Scoping note:** `value-faith-oracle-20261002.json` is **pre-existing on `main`** (with its `.md`) and is deliberately **not** in this PR. My first pass used a glob that swept it in and reported a spurious failure against the 10-row expectation; the corrected check tests only the files this PR adds. Recorded rather than quietly fixed, because a check that reports failures on files it does not own is a bad check.

### Check 3 — the alias claim re-verified at head
```
364ea161032a == 364ea161032a  MATCH  A-original / parent
80985a62f96e == 80985a62f96e  MATCH  B-no-distractor / F-adjacent
```
Both byte-identical pairs documented in the README are confirmed at this exact SHA.

### Check 4 — every `$HOME` path in the launcher resolves
```
resolved=5  missing=1
MISSING  $HOME/.local/share/uor-r4/locks/model-slot.json
```
The one miss is **correct by construction**: `launch-continue.sh` `rm -f`s the model slot on completion, so it is absent whenever no job holds it.

### What is not claimed

No model, kernel, scorer or training behaviour is exercised by this PR — it adds preserved data and one launcher script. The checks above verify well-formedness and path resolution, **not** any model result.

References #1512



---
_Comment/review ids in this file: 5972914583_

