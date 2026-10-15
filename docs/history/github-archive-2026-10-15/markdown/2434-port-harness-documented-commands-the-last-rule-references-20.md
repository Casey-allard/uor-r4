# #2434 — Port harness.documented_commands: the last rule (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T04:04:34Z
- **Updated:** 2026-10-14T04:07:04Z
- **Closed:** 2026-10-14T04:06:45Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2434
- **Merged:** 2026-10-14T04:06:45Z
- **Merge commit:** 8d160d4a28fe3ec2e9abfce79d0716f4a1e6272b
- **Base / Head:** main / UOR-Foundation:deepseek/documented-commands
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`harness.documented_commands` — the last rule, and four bugs the comparison found.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

**`harness.documented_commands`** — every documented invocation must exist — **ported and registered.** **The last of `check`'s own rules.**

## The decisive evidence

**The port resolves every documented command in the repository; Python's live `argparse` resolves the same ones:**

| comparison | result |
|---|---|
| rows | **143 both sides** |
| only in Rust | **0** |
| only in Python | **0** |
| `ok` disagreements | **0** |

## I said I would not approximate it, and this is why

Two units ago I declined this rule: **Python resolves through the live `argparse` parser — the command, its subcommand AND its flags** — and a version without that introspection **could only check the command word.** *"A weaker check wearing the name of the real one."*

**What changed is that the table now comes from the runtime rather than from me**: the flag set, the subcommand groups and the command set were **taken from the parser and compared against it** — the same strength from a different source, **derived from the binary rather than from `cli.py`**, which is what lets the rule outlive the Python implementation.

## Four divergences, all real

**The comparison was worth more than the port:**

1. **A flag's value was dropped from the rendered command** — 101 rows in one list and not the other.
2. **Then every non-flag token became a command word** — 110 rows apart, **worse.**
3. **Resolving against `PORTED_COMMANDS` instead of the accepted set** — **44 false failures** on commands that work (`check`, `mistakes capture`, `arena board`, `git clean-merged --apply`). **`PORTED_COMMANDS` answers "which commands no longer delegate"; the rule asks a different question, and I used the wrong one.**
4. **The order of two checks.** Python short-circuits `RUST_ONLY_COMMANDS` **before** the parser walk, so for those commands the flag list is **never consulted** — which is why `uor-harness onboard - can this machine…` passes despite `-` not being a flag.

**The deeper lesson cost three attempts**: I inferred `_tokenise` and `_is_value` **from their output** instead of reading them. **Reading four lines of the function was cheaper than another round of inferring it** — and my own earlier claim in this port, *"every rule called unportable turned out to be unread"*, **applied to the helpers inside this rule too.**

## A stale comment inside a registration

An earlier unit's comment said this rule **"deliberately is NOT"** ported. True when written, **false the moment this landed — and it sat between the new entry and the reader.** **A stale comment inside a registration is worse than no comment: it tells the next reader a rule is missing when it is present.**

```
cargo test -p uor-r4-harness -- --ignored the_table_agrees  -> 143 rows, identical
cargo test -p uor-r4-harness                               -> 388 passed, 3 ignored
cargo fmt --check / clippy -D warnings                     -> clean / 0 findings
python3 -m unittest discover -s tests/harness              -> 332 passed
python3 scripts/check_no_softmax.py                        -> OK — 1641, baseline 1641
parity.py                                                  -> exit 0
```

Record: `docs/labs/documented-commands-port-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **`parity.py` still reports 52, not 53**, because it reads the binary in the **owner checkout**, which is on `main` and does not contain this branch. **The inventory in this branch has seven `harness.*` ids including this rule**, verified by reading it — **and the count moves when this merges and the binary is rebuilt.** I am saying which number I trust and why rather than reporting the one that looks finished.
- **`check` still delegates**, because **`prlint.parse` cannot exist in Rust** — the parser is total, so the gate that fires on an unparseable body has no Rust equivalent. **Whether `check` may stop delegating with that one gate unreachable is a policy decision, not a port.**
- **`ACCEPTED` is still a hand-kept list**, and Python's set is 25 only because two sources happen to agree. **A command added to one and not the other will diverge, and only the comparison catches it** — so it must be run when the command surface changes.
- **The comparison test is `#[ignore]`d, so CI does not run it.** **All four bugs above were found by it and none by CI.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

