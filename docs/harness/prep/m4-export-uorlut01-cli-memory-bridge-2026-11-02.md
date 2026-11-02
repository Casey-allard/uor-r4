# Prep sheet: explicit system-memory serving bridge, 2 November 2026

Lab: deepseek. Item:
[m4-export-uorlut01-cli](../../roadmap/items/m4-export-uorlut01-cli.md).
Order:
[deepseek-1](../../loop/tasks/m4-export-uorlut01-cli/deepseek-1/ORDER.md).
Source base: `e0647330314841ddf84ba3bd7cfd57e57c03680e`.

TEST FITNESS: FIT

## Question and prior evidence

Training places a `System: Memory: {value}.` turn between a recall question
and the assistant marker. Product `uor-chat` previously could not encode that
turn. The public protocol-2 baseline therefore failed both `Pickle` and tomato
memory probes, and Claude-259 independently identified the absent recall line
as a serving-path gap.

This test asks only whether the explicit memory value, encoded through the new
bridge, changes the integer model's output on two fixed factual questions and
whether a `Memory: none.` control changes an unknowable question. It does not
test automatic extraction.

## Base artifact

- Stack-chat source:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/crates/uor-r4-integer/src/stack/chat.rs`,
  SHA-256 `3820ae6fe15c5fd6486b47e99b1dba7ecc4216c99a4a65e448fa71a4622b9826`.
- CLI source:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/crates/uor-r4-integer/src/bin/uor-chat.rs`,
  SHA-256 `3d249f6818fc0c4125f0dc77add611c25a1cab53a9a7fc92e0c26cf2c2a7d7a2`.
- Protocol encoder source:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/crates/uor-r4-tokenizer/src/dialogue.rs`,
  SHA-256 `d3f70a92165ae08a1a359bf77fb83c1f0fa95fe609bbaa51d723758a01254a56`.
- Dependency lock:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/Cargo.lock`,
  SHA-256 `9ab8edb75b2a651430bda0f70edc3116d070c272b7c20ca91e85df9e386a27eb`.
- Toolchain pin:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/rust-toolchain.toml`,
  SHA-256 `0a4b55522951f5475fcdd9ebcbfbbe650bfac272e8baff599585f0e8a120d5ee`.
- Executable:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/local/deepseek/chat-memory-target/debug/uor-chat`,
  SHA-256 `52490c220071fc0bffe4f21a8d570da01a9f2fdbaa391de4f724bae95476c3bb`.
- Host architecture: `x86_64`. This test is a product-path integration check,
  not an ARM64 or WebAssembly D11 audit. The current x86 stack audit is not
  admissible as a clean numeric-kernel proof because the script's D80/D81
  scalar multiply/divide exemption recognises ARM register syntax. On the
  release binary SHA-256
  `b1be73742c75d429ef4690334393a36fc95da1bc4b4d4de21a3521f06986c487`,
  it reports three flagged symbols containing four existing scalar `mul`
  instructions; the evidence debug binary reports ten flagged symbols
  containing 54 such instructions. Both also report two missing mandatory
  symbols, and the non-TAP call-graph path cannot run because `otool` is
  unavailable. The audit self-test passes. No audit pass or portable serving
  qualification is claimed here.
- Build:

```sh
CARGO_TARGET_DIR=/home/callard/projects/codex-playground/uor-r4-chat-baseline/local/deepseek/chat-memory-target \
  /home/callard/.cargo/bin/cargo build --locked -p uor-r4-integer \
  --features serving-wasm --bin uor-chat
```

Before inference, pin the source, executable, `Cargo.lock`, toolchain and all
artifact hashes below. Re-hash the executable after the run. Stop before model
output if any pin differs.

## Artifacts

- Seed 1 artifact:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut`,
  SHA-256 `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`.
- Seed 1 tokenizer:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/tokenizer-seed-1.json`,
  SHA-256 `1ca1ad2fe36ca43b4cc7deeb46e49c275f5172720d2e2fe30fafca2c17ce1514`.
- Seed 2 artifact:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut`,
  SHA-256 `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`.
- Seed 2 tokenizer:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/tokenizer-seed-2.json`,
  SHA-256 `5c09890c37475e1fdc2ae5cb6e70c4613200a0bd6c59a1eafe7f3b4a4311f72c`.

Each tokenizer's `stack_sha256` must equal its matching artifact SHA-256.

## Data split

Every invocation uses `--protocol 2 --threads 16 --temperature 0
--max-tokens 16`.

| Id | User turn | Explicit value |
|---|---|---|
| `recall-name` | `What did we name the new puppy?` | `Pickle` |
| `recall-grow` | `What does grandpa grow?` | `tomatoes` |
| `absent-breakfast` | `What did I eat for breakfast yesterday?` | `none` |

Each prompt runs under two arms:

- `off`: omit `--memory`;
- `on`: pass `--memory <value>` with the value in the table.

Both artifacts run both arms. Seed 1 repeats both factual `on` arms from fresh
processes. The CLI's bridge wraps a value as `System: Memory: VALUE.` through
the same protocol-2 encoder as training.

## Steps

Use a new output root:

`/home/callard/projects/codex-playground/uor-r4-chat-baseline/local/deepseek/m4-memory-bridge-2026-11-02-rerun/`

The original output root is preserved but is not authoritative because review
added the public `/memory` command-state test and changed the executable hash.
If the rerun root exists, stop. Use fresh `seed-1/`, `seed-2/` and
`seed-1-repeat/` subdirectories. Run this exact block from the task worktree:

```sh
set -eu
ROOT=/home/callard/projects/codex-playground/uor-r4-chat-baseline/local/deepseek/m4-memory-bridge-2026-11-02-rerun
BIN=/home/callard/projects/codex-playground/uor-r4-chat-baseline/local/deepseek/chat-memory-target/debug/uor-chat
ART1=/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
TOK1=/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/tokenizer-seed-1.json
ART2=/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut
TOK2=/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/tokenizer-seed-2.json
test ! -e "$ROOT"
mkdir -p "$ROOT/seed-1" "$ROOT/seed-2" "$ROOT/seed-1-repeat"
printf 'pass\tarm\tprompt\tstatus\n' >"$ROOT/status.tsv"
run_off() {
  out=$1 pass=$2 artifact=$3 tokenizer=$4 id=$5 prompt=$6
  if /usr/bin/time -v -o "$out/off-$id.time" "$BIN" --stack "$artifact" \
    --tokenizer "$tokenizer" --protocol 2 --threads 16 --temperature 0 \
    --max-tokens 16 --say "$prompt" >"$out/off-$id.json" 2>"$out/off-$id.err"; then
    status=0
  else
    status=$?
  fi
  printf '%s\toff\t%s\t%s\n' "$pass" "$id" "$status" >>"$ROOT/status.tsv"
  test "$status" -eq 0
}
run_on() {
  out=$1 pass=$2 artifact=$3 tokenizer=$4 id=$5 prompt=$6 value=$7
  if /usr/bin/time -v -o "$out/on-$id.time" "$BIN" --stack "$artifact" \
    --tokenizer "$tokenizer" --protocol 2 --threads 16 --temperature 0 \
    --max-tokens 16 --memory "$value" --say "$prompt" \
    >"$out/on-$id.json" 2>"$out/on-$id.err"; then
    status=0
  else
    status=$?
  fi
  printf '%s\ton\t%s\t%s\n' "$pass" "$id" "$status" >>"$ROOT/status.tsv"
  test "$status" -eq 0
}
run_pass() {
  out=$1 pass=$2 artifact=$3 tokenizer=$4
  run_off "$out" "$pass" "$artifact" "$tokenizer" recall-name 'What did we name the new puppy?'
  run_on "$out" "$pass" "$artifact" "$tokenizer" recall-name 'What did we name the new puppy?' Pickle
  run_off "$out" "$pass" "$artifact" "$tokenizer" recall-grow 'What does grandpa grow?'
  run_on "$out" "$pass" "$artifact" "$tokenizer" recall-grow 'What does grandpa grow?' tomatoes
  run_off "$out" "$pass" "$artifact" "$tokenizer" absent-breakfast 'What did I eat for breakfast yesterday?'
  run_on "$out" "$pass" "$artifact" "$tokenizer" absent-breakfast 'What did I eat for breakfast yesterday?' none
}
run_pass "$ROOT/seed-1" seed-1 "$ART1" "$TOK1"
run_pass "$ROOT/seed-2" seed-2 "$ART2" "$TOK2"
run_on "$ROOT/seed-1-repeat" seed-1-repeat "$ART1" "$TOK1" recall-name 'What did we name the new puppy?' Pickle
run_on "$ROOT/seed-1-repeat" seed-1-repeat "$ART1" "$TOK1" recall-grow 'What does grandpa grow?' tomatoes
sha256sum "$BIN" >"$ROOT/postflight-binary.sha256"
```

The seed-1 repeat runs only the two factual `on` arms from fresh processes.

## Readings

For every run record:

- exit status, exact wall time and peak RSS;
- returned protocol schema, artifact hash, user text, memory line and reply;
- reply text, exact reply ids, stop reason, history depth and elapsed seconds;
- whether the reply contains the factual value case-insensitively;
- whether each `on` reply-id list differs from its matching `off` control;
- whether the two factual `on` outputs change from off on each seed.

The recorded `memory` field must equal `Memory: <value>.` on every `on` run and
be null on every `off` run.

## Bar and decision

- **Instrument pass:** every pin and status matches; all 14 processes exit 0;
  all schemas are protocol 2; all stops are `eos`, `short_cycle` or
  `max_new_tokens`; text decodes from ids; the seed-1 repeat is exact; and all
  output files are retained.
- If both factual memory-on replies equal their off controls on both seeds,
  report `NO_OBSERVED_EFFECT` for this explicit-injection configuration.
- If any factual memory-on reply id list differs from its off control, report
  `MEMORY_LINE_CONSUMED` as an integration observation. Separately report
  whether the expected value is present.
- The absence control is descriptive. It cannot establish whether absent-store
  reads should consult a log sieve.

No model-quality bar is set. This test distinguishes an unconnected serving
format from complete model failure; it does not qualify chat.

## Minimum test power

Two initialization seeds, two factual questions, one absence control, two arms,
and two repeated seed-1 factual arms produce 14 replies. This is enough to
detect whether the explicit line reaches the model path and whether it visibly
changes any fixed answer. It is not enough to estimate language or memory
quality.

## Cost

Local Ryzen 9 9950X only, one process at a time, 16 threads, no GPU and `$0.00`
external spend. Preparation, the warm build and checks are projected below five
minutes; the 14-process panel below two minutes; report, review and delivery
below 20 minutes. Peak RSS is projected below 256 MiB and is measured per
process. There is no automatic retry; a fresh sibling output root is created
only for a recorded repair.

At preflight on 2 November 2026:

- free disk: `593,570,574,336` bytes, above the required
  `30 GiB + 128 MiB`;
- retained build: `729,942,000` bytes in the ignored target directory;
- new transcripts and reports: projected below 1 MiB;
- external charge: `$0.00`;
- the shared live model ledger
  `/home/callard/projects/codex-playground/uor-r4/.uor-models/native-joint-learning-2026-09-04/model-time.json`
  is absent, and no active storage receipt exists. This serving-only run trains
  no model and produces no checkpoint, so no model-time charge or prior training
  balance is claimed; the observed filesystem space and retained build size
  above are the complete local admission record.

## Stop rule

Stop on any pin mismatch, malformed record, command over 60 seconds, protocol
other than 2, peak RSS over 1 GiB or non-exact seed-1 repeat.

## Claim boundary

This test can show that an explicit `System: Memory:` turn is encoded and can
change deterministic integer outputs. It cannot show that facts are
automatically extracted, that a reply is correct, that the model generalizes,
that the artifact is ready for useful chat, or that the path passes an ARM64 or
WebAssembly zero-multiplier audit. It changes no numeric kernel.

## Non-author TEST FITNESS review

TEST FITNESS: FIT

PRE-RUN: PASS

Reviewed at exact head
`f2ca27e984310fea67b3ece9c8a32ab6d94ba9fe` by a fresh-context, read-only
`codex` reviewer using `deepseek-flash` in session
`01a19e0b-7e34-78d0-8341-940e919a9b8b`. The reviewer confirmed that the REPL
uses the tested command-state function, set/show/clear/one-turn/invalid
transitions are covered, the corrected release-binary audit accounting matches,
all pins match, and the rerun root was absent. It ran no model inference.
A final evidence check corrected the debug-binary count to ten flagged symbols
containing 54 scalar multiplies; this does not change FIT because the sheet
never claims that audit passes.
