# Prep sheet: automatic serving-side memory sieve, 2 November 2026

Lab: deepseek. Item:
[m3-qualify-memory](../../roadmap/items/m3-qualify-memory.md).
Order:
[deepseek-1](../../loop/tasks/m3-qualify-memory/deepseek-1/ORDER.md).
Reviewed code base: `fb24b48dce5e175fbc8e84eca9a8735a6e7a2c22`.
The source pins below bind that commit; the prep-sheet wording and report may
advance without changing the executable or model inputs.

TEST FITNESS: FIT

PRE-RUN: PASS

## Question and prior evidence

The explicit `System: Memory: {value}.` bridge is merged and deterministic.
Its 14-process probe showed that the public artifacts consume the line but do
not answer either factual question correctly.

This unit tests the layer before that limitation: can a deterministic
serving-side sieve derive `Pickle` and `tomatoes` from prior user turns and pass
the exact value into the bridge automatically? It does not test model quality.

## Base artifact

- Sieve source:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/crates/uor-r4-integer/src/stack/dialogue_memory.rs`,
  SHA-256 `d700ddaa2272d37f4d22f69df568627f9b53eefab0cf4fdf6d7f037c018e643a`.
- Stack-chat source:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/crates/uor-r4-integer/src/stack/chat.rs`,
  SHA-256 `3820ae6fe15c5fd6486b47e99b1dba7ecc4216c99a4a65e448fa71a4622b9826`.
- CLI source:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/crates/uor-r4-integer/src/bin/uor-chat.rs`,
  SHA-256 `da6375cfbac284c639f46533259197655129875bc14628d535fbf933fceb5a04`.
- Protocol encoder:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/crates/uor-r4-tokenizer/src/dialogue.rs`,
  SHA-256 `d3f70a92165ae08a1a359bf77fb83c1f0fa95fe609bbaa51d723758a01254a56`.
- Dependency lock:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/Cargo.lock`,
  SHA-256 `9ab8edb75b2a651430bda0f70edc3116d070c272b7c20ca91e85df9e386a27eb`.
- Toolchain pin:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/rust-toolchain.toml`,
  SHA-256 `0a4b55522951f5475fcdd9ebcbfbbe650bfac272e8baff599585f0e8a120d5ee`.
- Executable:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/local/deepseek/chat-auto-memory-target/debug/uor-chat`,
  SHA-256 `ed1e082fb485862368062b68d4b996242d1e200b0eb31e610ee63c05abbb38a9`.
- Build command:

```sh
CARGO_TARGET_DIR=/home/callard/projects/codex-playground/uor-r4-chat-baseline/local/deepseek/chat-auto-memory-target \
  /home/callard/.cargo/bin/cargo build --locked -p uor-r4-integer \
  --features serving-wasm --bin uor-chat
```

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

Each tokenizer's `stack_sha256` must equal its artifact hash. Re-hash the
executable after execution. The host is x86_64; no ARM64 or WebAssembly audit
pass is claimed.

## Data split

Two fixed conversations from the public `conversational-v2` panel:

| Id | User turns | Expected selected value |
|---|---|---|
| `conv-mem-01` | `The new puppy at our house is named Pickle.`; `What did we name the new puppy?` | `Pickle` |
| `conv-mem-02` | `Grandpa grows tomatoes in a box by the window.`; `He waters them every day.`; `What does grandpa grow?` | `tomatoes` |

The sieve must print `[memory sieve] Memory: <value>.` immediately before the
assistant response to each final question. The factual statements themselves
must not request or emit a memory line. Assistant-produced text must not enter
the fact log.

## Steps

Use a new output root:

`/home/callard/projects/codex-playground/uor-r4-chat-baseline/local/deepseek/m3-auto-memory-2026-11-02/`

If it exists, stop. Use fresh `seed-1/`, `seed-2/` and `seed-1-repeat/`
directories. Run this exact block:

```sh
set -eu
ROOT=/home/callard/projects/codex-playground/uor-r4-chat-baseline/local/deepseek/m3-auto-memory-2026-11-02
BIN=/home/callard/projects/codex-playground/uor-r4-chat-baseline/local/deepseek/chat-auto-memory-target/debug/uor-chat
ART1=/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
TOK1=/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/tokenizer-seed-1.json
ART2=/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut
TOK2=/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/tokenizer-seed-2.json
test ! -e "$ROOT"
mkdir -p "$ROOT/seed-1" "$ROOT/seed-2" "$ROOT/seed-1-repeat"
printf 'pass\tconversation\tstatus\n' >"$ROOT/status.tsv"
run_mem1() {
  out=$1 pass=$2 artifact=$3 tokenizer=$4
  if printf '%s\n' \
    '/tokens' \
    'The new puppy at our house is named Pickle.' \
    '/tokens' \
    'What did we name the new puppy?' \
    '/tokens' \
    '/quit' |
    /usr/bin/time -v -o "$out/mem-01.time" \
      "$BIN" --stack "$artifact" --tokenizer "$tokenizer" --protocol 2 \
      --threads 16 --temperature 0 --max-tokens 16 --memory-sieve \
      >"$out/mem-01.txt" 2>"$out/mem-01.err"; then
    status=0
  else
    status=$?
  fi
  printf '%s\tmem-01\t%s\n' "$pass" "$status" >>"$ROOT/status.tsv"
  test "$status" -eq 0
}
run_mem2() {
  out=$1 pass=$2 artifact=$3 tokenizer=$4
  if printf '%s\n' \
    '/tokens' \
    'Grandpa grows tomatoes in a box by the window.' \
    '/tokens' \
    'He waters them every day.' \
    '/tokens' \
    'What does grandpa grow?' \
    '/tokens' \
    '/quit' |
    /usr/bin/time -v -o "$out/mem-02.time" \
      "$BIN" --stack "$artifact" --tokenizer "$tokenizer" --protocol 2 \
      --threads 16 --temperature 0 --max-tokens 16 --memory-sieve \
      >"$out/mem-02.txt" 2>"$out/mem-02.err"; then
    status=0
  else
    status=$?
  fi
  printf '%s\tmem-02\t%s\n' "$pass" "$status" >>"$ROOT/status.tsv"
  test "$status" -eq 0
}
run_mem1 "$ROOT/seed-1" seed-1 "$ART1" "$TOK1"
run_mem2 "$ROOT/seed-1" seed-1 "$ART1" "$TOK1"
run_mem1 "$ROOT/seed-2" seed-2 "$ART2" "$TOK2"
run_mem2 "$ROOT/seed-2" seed-2 "$ART2" "$TOK2"
run_mem1 "$ROOT/seed-1-repeat" seed-1-repeat "$ART1" "$TOK1"
run_mem2 "$ROOT/seed-1-repeat" seed-1-repeat "$ART1" "$TOK1"
sha256sum "$BIN" >"$ROOT/postflight-binary.sha256"
```

## Readings

For every conversation record:

- exit status, stderr, complete stdout and `/usr/bin/time -v` output;
- the exact `[memory sieve]` line before each final question;
- every assistant reply text, reply ids, stop reason and history position;
- whether the final factual reply contains the expected value;
- whether each memory line equals the value independently extracted by the
  pinned sieve executable.

The seed-1 repeat must match the first pass in every assistant reply's text,
ids, stop and history and in every emitted memory line; timing fields may
differ.

## Bar and decision

- **Sieve pass:** the unit tests return `Pickle` and `tomatoes`, return no value
  for unrelated and pronoun-only questions, select the latest correction, and
  never record generated assistant text as a fact.
- **Instrument pass:** all six processes exit 0; stderr is empty; every stream
  uses protocol 2 and valid stops; both factual questions emit the expected
  automatic memory line on both seeds; and the seed-1 repeat is exact.
- Record separately whether the model's factual answer contains the supplied
  value.
- If either factual memory line is absent or wrong, report an instrument or
  extractor failure and do not interpret model output.
- If both lines are correct but neither reply contains the value, report
  `AUTO_MEMORY_OBSERVED` with `MODEL_RECALL_NOT_OBSERVED`.

No model-quality bar is set. The result can establish serving-side extraction
and context injection only; it cannot qualify M3.

## Minimum test power

Two initialization seeds and two fixed multi-turn conversations, with all
turn outputs retained, plus a complete seed-1 repeat. This can detect whether
the extractor feeds both required values into the product path and whether the
result is deterministic. It cannot estimate extraction accuracy or language
quality.

## Cost

Local Ryzen 9 9950X, 16 threads, one process at a time, no GPU and `$0.00`
external spend. At FIT time the host had `589,721,354,240` bytes available,
above the required `30 GiB + 128 MiB`; the isolated build target is
1,785,321,921 bytes and new evidence is under 1 MiB. The shared training ledger
path is absent; this serving-only run creates no model and charges no
model-time. No automatic retry; use a fresh sibling root for any recorded
repair.

## Stop rule

Stop on any pin mismatch, malformed or missing transcript, nonzero exit,
non-protocol-2 record, unexpected stop, absent/wrong memory line, peak RSS over
1 GiB, or non-exact seed-1 repeat.

## Claim boundary

This test can show that a deterministic lexical sieve selects and injects a
value from prior user turns. It cannot show general information extraction,
that the model uses the value correctly, useful conversation, fresh-phrasing
transfer, M3 qualification or M4 readiness.

## Non-author TEST FITNESS review

TEST FITNESS: FIT

PRE-RUN: PASS

A fresh-context deepseek reviewer read the code at
`fb24b48dce5e175fbc8e84eca9a8735a6e7a2c22`, the full branch diff and all source
pins above at branch head `57b212b0e`. It found no blockers. It independently
passed the 5 dialogue-memory tests, 5 `uor-chat` tests, 12 stack-chat tests,
the source/executable/model/lockfile/toolchain pins, the tokenizer-to-artifact
bindings, `git diff --check` and `prep-check` block completeness.

The reviewer recorded three nonblocking limits for later hardening: the
in-memory fact log is unbounded while automatic mode is off, any recognized
question without a lexical match requests `Memory: none.`, and an automatic
fact log survives the CLI's internal context-overflow reset. These do not
change this fixed two-conversation smoke's registered question, but the report
must not generalize beyond it.
