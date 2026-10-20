# #2462 — Port panel score; the last site loses its first subcommand (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T09:27:05Z
- **Updated:** 2026-10-14T09:29:43Z
- **Closed:** 2026-10-14T09:29:34Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2462
- **Merged:** 2026-10-14T09:29:34Z
- **Merge commit:** e284cd2e1b2c096c103b03918fe0fc58b2c150da
- **Base / Head:** main / UOR-Foundation:deepseek/panel-score
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21)

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

Decision: **KEEP** (tooling).

## What this delivers

**`uor-harness panel score` is Rust** — the D22 acceptance-panel check: one measured value against one
frozen bar, answering `PASS` / `FAIL` / `AT NOISE`.

**The unported-command count went 7 → 6; the delegation-site count went 1 → 2.** Both are true and only
one is progress, so both are stated: the general fallback carried all seven subcommands and now carries
five (`arena` x3, `budget` x2), while `panel show` is delegated by its own named line — its `--json`
prints each bar as the panel file's own object in the file's key order, `serde_json` is a `BTreeMap`, and
**there is no `note` field on `registries::Bar` at all**. A `show` that answered with those keys
re-ordered would look right and diff wrong.

## Three things the comparison caught

1. **I had read only half of `classify`.** Python appends " - noise is unstated for this bar, so this
   cannot be called a real difference (D22 section 3)" whenever the band is unstated — which is most
   panels — and the first port printed the shorter sentence.
2. **Python quotes the headline bar id and not the alternatives** (`'complete-correct-512'` vs
   `fresh-transfer`).
3. **The `--bar` note was printed before `--value` was validated**, so a command that then failed still
   printed a note about the headline bar. argparse refuses first; now so does this.

## Verification: both implementations, 18 cases

Against the repo's real panels, text and `--json`: at target (PASS), at current, midway, a decimal,
zero, an explicit `--bar`, `--label`, an unknown panel, an unknown bar, a prefix panel match, and a
panel with no bars.

| check | result |
|---|---|
| stdout / stderr / exit on 18 cases | all MATCH except two argparse usage blocks (same exit 2) |
| `cargo test -p uor-r4-harness --offline` | 461 passed, 0 failed, 6 ignored (was 450) |
| fmt, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | exit 0 |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |

## Checklist

- [x] own worktree, port with tests, record at `docs/labs/panel-score-port-2026-10-14/README.md` plus a newest-first current-state entry, full verification, protected PR
- [x] both implementations exercised and compared byte for byte, not read back
- [x] the three defects are each named in the record and covered by tests
- [x] limitations stated below

### Stated rather than hidden

* **A missing or non-numeric `--value` prints this binary's message, not argparse's usage block.** Exit
  code 2 in both — the divergence already recorded for `check` and `git switch`.
* **`panel show` still delegates**, for the key-order reason above; the ordered parse is the next unit.
* **`--value nan` / `inf`**: `json.dumps` writes bare `NaN`/`Infinity` and so does this renderer; `%g`
  writes `nan`/`inf` in the text form as Python does.

References #2028


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6165211727 — 2026-10-14T09:27:13Z — Casey-allard

## Verification at the exact head (local, this worktree)

```
CARGO_TARGET_DIR=$PWD/local/harness-target cargo fmt -- --check   -> clean
cargo clippy --all-targets -- -D warnings                         -> clean, 0 findings
cargo test -p uor-r4-harness --offline                            -> 461 passed, 0 failed, 6 ignored
python3 -m unittest discover -s tests/harness -t .                -> exit 0, 0 failure lines
python3 scripts/harness/parity.py                                 -> exit 0
python3 scripts/check_no_softmax.py                               -> OK 1641, baseline 1641
```

**Both implementations on 18 cases against the repo's real panels (`m2-reply512`), text and `--json`:**

```
at target (PASS)         exit 0/0  out ok  err ok
at current               exit 0/0  out ok  err ok
midway                   exit 0/0  out ok  err ok
decimal value            exit 0/0  out ok  err ok
zero                     exit 0/0  out ok  err ok
explicit --bar           exit 0/0  out ok  err ok
json explicit --bar      exit 0/0  out ok  err ok
--label                  exit 0/0  out ok  err ok
json at target / midway  exit 0/0  out ok  err ok
json with label          exit 0/0  out ok  err ok
prefix panel match       exit 0/0  out ok  err ok
unknown panel            exit 2/2  out ok  err ok
unknown bar              exit 2/2  out ok  err ok
no value / bad value     exit 2/2  out ok  err DIFF (argparse usage block)
```

**What the comparison changed in the first version of this port:**

* `classify` continues past the PASS/FAIL branches — with an unstated band Python appends *" - noise is
  unstated for this bar, so this cannot be called a real difference (D22 section 3)"*, and the first port
  printed the shorter sentence on every unstated-noise panel.
* `no --bar given: scoring the headline bar 'complete-correct-512' (also available: fresh-transfer)` — the
  headline id is `!r`, the alternatives are not.
* The `--bar` note was emitted before `--value` was validated, so a failing command still printed it.
  argparse refuses first; now so does this.

**The delegation count, stated both ways:** unported commands **7 → 6**; `delegate_to_python` call sites
**1 → 2** (the general fallback now carries `arena` x3 and `budget` x2; `panel show` has its own named
line). A site count is not a progress measure here.


---
_Comment/review ids in this file: 6165211727_

