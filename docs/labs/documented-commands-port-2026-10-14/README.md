# `harness.documented_commands` — the last RULE, and four bugs the comparison found

> **Correction added after verification:** registering the last rule did **not** complete the objective.
> `uor-harness check` runs the **format contract**, not the rule set. See the section at the end — that
> correction is the most important thing in this record.

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the rule I twice declined to approximate

## What landed

**`harness.documented_commands`** — every documented invocation must exist — **ported and registered.**

**The decisive evidence: the port resolves every documented command in the repository, and Python's
live `argparse` resolves the same ones — 143 rows, identical on both sides, zero commands in one list
and not the other, zero validity disagreements.**

| comparison | result |
|---|---|
| rows | **143 both sides** |
| only in Rust | **0** |
| only in Python | **0** |
| `ok` disagreements | **0** |

## I said I would not approximate it, and this is why

Two units ago I declined to port this rule, because **Python resolves a documented command through
the live `argparse` parser — the command, its subcommand AND its flags** — and a version without that
introspection **could only check the command word.** *"A weaker check wearing the name of the real
one."*

**What changed is that the table now comes from the runtime rather than from me**: the flag set, the
subcommand groups and the command set were **taken from the parser and compared against it**. That is
the same strength from a different source, **and it is derived from the binary rather than from
`cli.py`, which is what lets the rule outlive the Python implementation.**

## Four divergences, and what each one was

**The comparison was worth more than the port.** Each round of it found a real defect:

1. **A flag's value was dropped from the rendered command** — 101 rows in one list and not the other.
2. **Then every non-flag token became a command word** — 110 rows apart, worse.
3. **Resolving against `PORTED_COMMANDS` instead of the accepted set** — **44 false failures** on
   commands that work (`check`, `mistakes capture`, `arena board`, `git clean-merged --apply`).
   `PORTED_COMMANDS` answers *"which commands no longer delegate"*; **the rule asks a different
   question, and I used the wrong one.**
4. **The order of two checks.** Python short-circuits `RUST_ONLY_COMMANDS` **before** the parser walk,
   so for those commands the flag list is **never consulted** — which is why
   `uor-harness onboard - can this machine…` passes despite `-` not being a flag.

**The deeper lesson is about method, and it cost three attempts**: I inferred `_tokenise` and
`_is_value` **from their output** instead of reading them. **Reading four lines of the function was
cheaper than another round of inferring it**, and my earlier claim in this port — *"every rule called
unportable turned out to be unread"* — **applied to the helpers inside this rule too.**

## A stale comment inside a registration

An earlier unit's comment in the inventory said `harness.documented_commands` **"deliberately is NOT"**
ported. That was true when written and **false the moment this landed** — **and it sat between the new
entry and the reader.** **A stale comment inside a registration is worse than no comment: it tells the
next reader a rule is missing when it is present.**

## Verification

| check | result |
|---|---|
| the table vs Python's live parser over every documented command | **143 rows, identical** |
| the live `gh` chain (previous unit's `#[ignore]`d test) | still passes |
| `cargo test -p uor-r4-harness` | **388 passed, 3 ignored** |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | 332 passed |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0 |

**`parity.py` still reports 52, not 53**, because it reads the binary in the **owner checkout**, which
is advanced to `main` and does not contain this branch. **The inventory in this branch has seven
`harness.*` ids including this rule**, verified by reading it — **and the count in the parity report
will move when this merges and the binary is rebuilt.** I am saying which number I trust and why
rather than reporting the one that looks finished.

## Limitations, stated

- **`harness.documented_commands` is registered but `check` still delegates**, because
  **`prlint.parse` cannot exist in Rust** — the parser is total, so the gate that fires on an
  unparseable body has no Rust equivalent. **Whether `check` may stop delegating with that one gate
  unreachable is a policy decision, not a port**, and it is the objective's own remaining condition.
- **`ACCEPTED` is still a hand-kept list** of 25 commands, **and Python's set is 25 by coincidence of
  two sources** (14 argparse choices plus 11 Rust-only). **A command added to one and not the other
  will diverge, and only the comparison catches it** — so the comparison must be run when the command
  surface changes, not only when this rule changes.
- **The comparison test is `#[ignore]`d**, so CI does not run it. **All four bugs above were found by
  it and none by CI.**
- **`FLAGS` and `GROUPS` are kept but not consulted by `resolve`** — that is Python's behaviour, and
  they are annotated `dead_code` as the record of what the table came from rather than removed.

---

## 2026-10-14 — `check` runs the format contract, NOT the rule set — the objective is not met

**Found while verifying rather than while porting, at the moment the rule count reached 100%.** Attaching it to
`docs/labs/documented-commands-port-2026-10-14/README.md` rather than a new record, because it corrects the
claim that unit makes about what registering the last rule achieved.

### What the numbers say, and what they do not

`parity.py` now reports **`0 of check's own 39 rule(s) still in Python`** and **53 implemented**. That is a
statement about **the rule inventory** — every rule in `check`'s set has a Rust implementation and is registered.

**It is not a statement that `uor-harness check` runs them.**

`cmd_check` (`crates/uor-r4-harness/src/main.rs:1990`) calls **`Contract::load(repo)` and `contract.check(repo)`**,
which reports the state-format contract: formats, files, rows, violations. **The 53 registered rules are not
invoked by it.** They exist, they are tested, and `cmd_check` does not call them.

**So `uor-harness check` is WEAKER than `python3 scripts/harness/cli.py check`** — it checks the frozen formats
and omits arena, briefing, budget, harness, mistakes, multilab, prep, prlint, receipts and status rules entirely.

### This is the failure mode I have been guarding against, one level up

Every unit in this port has refused to ship a weaker check wearing a real rule's name. **`cmd_check` is that
failure at the command level**: the rule count says complete, the registered ids say complete, and the command
a lab actually runs checks a fraction of what Python's does. **The parity metric I have been reporting measures
registration, not dispatch — and I read it as dispatch.**

### It also means the delegation question is not the blocker

I had been treating `prlint.parse` as the last obstacle, since it cannot exist in Rust (the parser is total). It
is not the obstacle. **The obstacle is that `cmd_check` must call the rule set before `check` can stop
delegating**, and then the dispatch table is what needs the parity evidence.

### What is actually done

- **53 of 54 rules implemented and registered.** Every rule in `check`'s own set.
- **`prlint.parse` is the 54th** and is unreachable in Rust by construction — a policy question, not a port.
- **Five real `delegate_to_python` sites remain** (`main.rs` 362, 548, 1034, 1167, 1305): the unported tail of
  `mistakes`, `git`, `multilab` and `team`.

### The next unit, precisely

**Wire `cmd_check` to the 53 rules** — a dispatcher plus a parity comparison of `check`'s *output* against
Python's, not its inventory. **Until that lands, `check` must stay delegated**, because a `check` that returned
the format contract's verdict while a lab believed it had run every rule **is exactly the silent downgrade this
port exists to prevent.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).
