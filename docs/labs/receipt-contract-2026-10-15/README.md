# `receipt` refuses what it cannot honour: the frozen kind list, the short milestone form, and flags it does not read

Date: 2026-10-15. Lab: deepseek. Line: no model line - tooling. Task source: Kimi PM follow-up
[#2028 6195505734](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6195505734), found by the
Stage-A sweep-record agent. No model compute, no GPU, no pod, no headline change.

## What was wrong

**1. An unknown `--kind` was written to the shard.** `receipt::Receipt::validate` checked that a kind was
present and that the timestamp parsed, but not that the kind was one the project recognises. So
`uor-harness receipt --kind nonsense_kind` recorded the line, and the next `uor-harness check` hard-errored on
it at parse (`receipts.parse`, `unknown kind`). **The harness accepted a record its own reader rejects**, and
the only repair was to find the line in the shard and delete it by hand - which is what the sweep-record agent
did before re-recording with `--kind pr`.

**2. `-m` was parsed, forwarded and dropped.** `-m` is the documented short form of `--milestone` (heartbeat
takes both), and the global parser forwards it, but `cmd_receipt` read only the long form. A receipt recorded as
`receipt --kind heartbeat -m m1` therefore carried **no milestone at all**, silently.

**3. Any other flag the parser knows was dropped the same way.** `receipt --count 1/3` or
`receipt --headline ...` was accepted by the global parser and ignored by the command, on the one command whose
whole purpose is to record what happened.

## What changed

| file | change |
|---|---|
| `crates/uor-r4-harness/src/receipt.rs` | `Receipt::validate` checks the kind against `receiptsrules::KINDS` and names the frozen list in the refusal. This is the one place every writer passes through (`receipt`, `skill use`, `prep fit`, `team dispatch`), so the contract is enforced at WRITE time and cannot be bypassed by a new caller |
| `crates/uor-r4-harness/src/main.rs` | `-m` is read as the milestone short form; `RECEIPT_FLAGS` declares the flags the command reads, and `receipt_unread_flag` refuses anything else with exit 2 and the list, instead of dropping it |
| `crates/uor-r4-harness/src/main.rs` (usage) | the `receipt` help line states that the kind is validated and that unread flags are refused |

## Verification

- `cargo test -p uor-r4-harness --offline`: **534 unit + 3 CLI contract tests pass** (the new file is
  `tests/receipt_command_contract.rs`), fmt and `clippy --all-targets` clean.
- **Every fix is pinned by a test that fails when the fix is removed**, proved one at a time:

  | fix removed | test that fails |
  |---|---|
  | the frozen-kind check | `an_unknown_kind_is_refused_before_it_reaches_the_record` (unit) and `an_unknown_kind_is_refused_and_nothing_is_written` (CLI) |
  | the `-m` lookup in `cmd_receipt` | `the_short_milestone_form_is_recorded_like_the_long_one` |
  | the unread-flag guard | `a_flag_the_command_does_not_read_is_refused_loudly` |

- Live, with the built binary: `receipt --kind nonsense_kind` exits 2 naming the list and creates no shard;
  `receipt --kind heartbeat -m m1 --dry-run` and `--milestone m1` produce the same line, milestone included;
  `receipt --kind heartbeat --count 1/3` exits 2 naming `--count`; `--summary` still exits 2 at the parser.

## What this does not do

- It does not change the frozen list. `check`, `claim`, `context`, `heartbeat`, `pr`, `prep`, `receipt`,
  `release`, `skill`, `team` are the ten kinds, and adding one is still a contract change.
- It does not make the parser refuse unknown flags for every ported command - only `receipt` declares its reads
  today. The same `RECEIPT_FLAGS` pattern is the template for the others; a lab that hits a dropped flag on
  another command should file it the same way.
- Python's `receipts.record` keeps its own behaviour; the Rust binary is the deployed path.

## Cost

Laptop CPU only: source edits, one build, four test runs and four guard-removal probes. No pod, no GPU, no new
dependency, no schema change beyond the enforcement of the existing one.