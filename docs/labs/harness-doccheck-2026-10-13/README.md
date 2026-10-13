# The command docs are now checked by machine, and Claude onboarded

Two things this unit: the validator that catches a class of error I had been finding by hand, and the
first evidence that the previously-unproven paths work.

**Line:** no model line — tooling. · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

## 1. `doccheck.py`: three of my four documentation errors, caught automatically

Four documentation errors shipped in three units. Every one was found by **running the command by
hand while writing a record**, which is a poor detector — it only works if the author happens to try
that exact string, and a lab following the doc gets an error without knowing whether the doc or its
own setup is at fault.

| error | caught by `doccheck`? |
|---|---|
| `heartbeat --dry-run` — no such flag (it is `--print`) | **yes** |
| a top-level `install-hooks` — it is a `git` subcommand | **yes** |
| `team list` — the subcommand is `roster` | **yes** |
| "`prep new` changes no research state" — it **writes** a sheet | **no** |

That last one is named deliberately. It is a claim about behaviour, and an argument-parser check
cannot catch it. Saying so is the point: a validator that is silently incomplete is worse than one
whose limits are written down.

It **parses** the real argparse tree rather than executing anything, so it is safe against a live
repository — `heartbeat` or `team claim` would mutate state if invoked. Registered as
`harness.documented_commands`, so it runs in `check` and therefore in CI.

```
$ python3 scripts/harness/doccheck.py
documented commands: 40 checked against the real argument parser
  every documented command exists, with the flags it claims
```

### The first version had a hole, in the exact case it exists for

`cli.py install-hooks` — error #2 — **was not detected**. `git` was in the tokeniser's skip-list, so
the walk stopped at `git` and never reached `install-hooks` to find it missing. The check would have
passed the error it was written for.

Removing `git` from the skip-list fixes it, and the tests now pin the case:

```
cli.py install-hooks        -> FAIL   (a top-level command that is really a subcommand)
cli.py git install-hooks    -> pass
```

That is worth recording plainly: a check that passes everything looks identical to a check that
works, and only a case with a **known-bad** input tells them apart. All 13 tests in
`tests/harness/test_doccheck.py` are either an error that actually shipped or a shape the docs really
use — including the shell substitution in `--repo "$(git rev-parse --show-toplevel)"`, whose inner
spaces would otherwise split into apparent command words.

## 2. Claude onboarded, and closed the two gaps

The onboarding prompt ran successfully in a lab that did not write it. Before this unit, two paths
had **zero** foreign receipts; now:

| path | foreign labs |
|---|---|
| `prep new` | **claude** |
| `team dispatch` | **claude** |
| claim, release | claude, codex, kimi |
| heartbeat | claude, codex, kimi |

`prep new` was the one that mattered: it is the D22 gate standing between a lab and prohibited
compute, and a lab that did not write it has now run it.

Claude's records are unmistakably its own work, not test data: a receipt for
`docs/harness/prep/m4-starling-read-2026-10-13.md`, a dispatch of the `evidence-auditor` role, a
claim and release of `onboard/claude`, and a heartbeat reading
`M4: softmax at runtime: yes -> target no (D23 prohibits softmax project-wide)` — real project state
with a real milestone panel.

**One thing that looked like a defect and is not.** The prep receipt names a sheet that does not
exist on disk. That is correct behaviour: step 8 of the prompt tells a lab to delete the sheet if it
is not going to use it, and Claude complied. A receipt pointing at a since-deleted sheet is not
corruption — it is a record of a registration that was made and then withdrawn.

## Evidence

```
python3 -m unittest discover -s tests/harness -t .   -> 290 passed (3.15 and 3.9)
python3 scripts/harness/cli.py check                 -> 45 rules, 0 errors
python3 scripts/harness/doccheck.py                  -> 40 commands, all exist
```

## Limitations

- **`doccheck` cannot catch a false description of a real command** — which is exactly the fourth
  error. It checks shape, not truth.
- It reads seven named documents. A command in a new doc is unchecked until that doc is added to the
  list, and nothing enforces the list staying complete.
- Claude's successful run proves the prompt is followable **once**, in one lab, on this machine. It
  is not evidence that a lab on a different runtime, or with no Rust toolchain, gets the same result.
- The `prep` receipt now points at a sheet that no longer exists. That is correct for this run, but a
  reader following the pointer later gets nothing. Whether a withdrawn registration should leave a
  receipt at all is a design question this unit did not settle.
