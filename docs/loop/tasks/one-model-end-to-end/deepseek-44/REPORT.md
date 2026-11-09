# REPORT: one-model-end-to-end, deepseek-44

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `59494646c`.

**In one line:** the run-level sidecar test now proves the attached readout
changes all 256 decoded first tokens, the detached-model mutation fails, CPU
sidecar refusal goes through the same run path, and the module documentation
matches the CPU-only ranking behavior.

## What did not work, first

- **The third exact-head review returned CHANGES.** The run fixture reported
  the sidecar identity but did not assert that the attached model produced the
  readout target.
- **The detached-model mutation fails as required.** Decoding from a separately
  parsed model left the report identity fields present but changed the first
  tokens away from 42; the run test failed at the row assertion.
- **No real asking artifact was scored.** This remains a synthetic host
  integration test and sealed parity rerun.

## What worked

- **The mounted sidecar reaches decoding.** The always-firing readout targets
  token 42. The successful run writes 256 rows, and every row's first generated
  ID is 42.
- **The full run path remains covered.** The fixture passes argument parsing,
  manifest checks, artifact and tokenizer loading, sidecar validation and
  attachment, decoding, report construction and JSON serialization.
- **Binding failures remain run-level.** Wrong-artifact and
  outside-vocabulary sidecars are refused before an output report is created.
- **CPU refusal is run-level.** `--engine cpu --content-readout` is refused
  with the instruction to use `--engine integer`, and no output is created.
- **The documentation is consistent.** The module header now says
  teacher-forced ranking stays on the CPU; it no longer implies that integer
  generation and ranking use the same read.

## What changed

One code file,
`docs/labs/one-model-end-to-end/instrument/src/bin/served-reply-panel.rs`:

- corrects the module header's teacher-forced description;
- asserts every run-level row starts with the sidecar target;
- invokes CPU sidecar refusal through `run_with_args`.

This ORDER and REPORT and one notebook-row update are the remaining changes.
No runtime semantics, bar, scope or integer-library code changed.

## Measurements

- `served-reply-panel` tests with `cpu-parity`: **27 passed, 0 failed**.
- Successful run: 256 rows; every `rows[*].reply_ids[0]` is `42`.
- Detached-model mutation: row assertion failed.
- Wrong artifact through run: refused; no report file.
- Outside vocabulary through run: refused; no report file.
- CPU engine with `--content-readout`: refused; no report file.
- Sealed CPU/integer parity: stripped reports equal at `0081f5f3…`; observed
  1,348 / 828 decode tokens/s.
- Release binary:
  `b4f5986b40b2476d2b96ca265bfd4f2cdc0993f4d9535378351f8fffc5529571`.
- Effect reports: `local/deepseek/served-product-parity/final-parity-effect/`
  (`cpu.json` `b201ac58…`, `integer.json` `7c82f8da…`).

## Checks run at the head

- `served-reply-panel` tests with `cpu-parity`: 27 passed, 0 failed.
- `rustfmt --check --edition 2021` on the changed file: pass.
- `git diff --check`: pass.
- Scoped Clippy, release build, sealed parity, report/policy checks and full
  gates run after the repair commit.

## Skills used

None beyond the loop itself: the effect repair ORDER, run-level assertion and
mutation.

## Still open

- A real asking artifact remains unserved; claude-339 is Claude's run.
- Integer teacher-forced ranking remains unavailable by design.
- `uor-r4-cpu-train`'s independent Hamming port still omits ask/carry.

## Plain summary

The run test previously proved the sidecar was named in the report. It now
proves the sidecar actually changed the answer the run produced. If the code
attaches a sidecar but decodes with a different model, the test fails. The
review's remaining documentation inconsistency and CPU refusal coverage are
also repaired.
