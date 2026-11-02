# Prep sheet: public integer-chat post-smoke baseline, 2 November 2026

Lab: deepseek. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md).
Order: [deepseek-6](../../loop/tasks/one-model-end-to-end/deepseek-6/ORDER.md).
Source base: `b44402e0c` (the branch adds documentation only).
Supersedes the compute claim in
[the rejected public-chat sheet](one-model-end-to-end-public-chat-baseline-2026-11-02.md);
that sheet remains immutable as a `TEST FITNESS: NOT_FIT` record.

TEST FITNESS: FIT

## Prior observations and question

An earlier seed-1 smoke used the default protocol 1 and observed
`" Whilk from you."` on `conv-talk-01` and both turns of `conv-mem-01`.
That smoke is disclosed rather than treated as unseen. It is also an
instrument observation, not a model conclusion: TRAIN and the public
conversational panel use dialogue protocol 2, whose role-marker space is merged
into the following word. Protocol 1 sends a separate space token and has
already produced misleading empty-artifact replies. The project recorded that
correction in
[#3407 comment 6584644252](https://github.com/UOR-Foundation/uor-r4/issues/3407#issuecomment-6584644252).

This follow-up asks: **with the trained protocol selected explicitly, what does
the public integer FBS artifact emit through the product `uor-chat` path on the
same fixed eight-entry panel?**

No protocol-2 output has been observed before this sheet. The eight entries and
decision rule are fixed before any protocol-2 run. This is an ungraded
integration baseline; it does not establish language capability.

## Base artifact and executable

All paths are absolute and outside the documentation worktree.

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
- Executable:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/local/deepseek/public-chat-target/debug/uor-chat`,
  SHA-256 `4f7458c4bfe3b5129033564e97f7cd7d781d838c24f79edf62c92a26109f45a2`.
- Source:
  `crates/uor-r4-integer/src/bin/uor-chat.rs`, SHA-256
  `e98ffae93bb96a5351e5af83a130764d7df51f42284ad62d8dc5b9826034cfdc`.
- Protocol encoder source:
  `crates/uor-r4-tokenizer/src/dialogue.rs`, SHA-256
  `d3f70a92165ae08a1a359bf77fb83c1f0fa95fe609bbaa51d723758a01254a56`.
- Dependency lock: `Cargo.lock`, SHA-256
  `9ab8edb75b2a651430bda0f70edc3116d070c272b7c20ca91e85df9e386a27eb`.
- Toolchain pin: `rust-toolchain.toml`, SHA-256
  `0a4b55522951f5475fcdd9ebcbfbbe650bfac272e8baff599585f0e8a120d5ee`;
  selected Cargo and Rust are both `1.97.1`.
- Build:

```sh
CARGO_TARGET_DIR=/home/callard/projects/codex-playground/uor-r4-chat-baseline/local/deepseek/public-chat-target \
  /home/callard/.cargo/bin/cargo build --locked -p uor-r4-integer \
  --features serving-wasm --bin uor-chat
```

The binary's dependency file, mtime and embedded Rust commit must all postdate
the source pins and agree with this build. The `/usr/bin` Cargo 1.75 cannot
parse the workspace and is not the build tool.

## Data split

Panel:
`/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-panels/conversational-v2.json`,
SHA-256 `d7efc53b17a08fcf450266ae69f4a86ee82ab5fd3f3dbd1fcb58399c0c0cc362`,
160 prompts in six categories. These eight entries are fixed:

| Panel id | Category | Turns |
|---|---|---|
| `conv-clar-01` | clarify/on-topic | 1 |
| `conv-mem-01` | multi-turn memory | 2 |
| `conv-do-01` | instruction | 1 |
| `conv-do-02` | instruction | 1 |
| `conv-talk-01` | small talk | 1 |
| `conv-story-01` | story continuation | 1 |
| `conv-unk-01` | unknowable | 1 |
| `conv-mem-02` | multi-turn memory | 3 |

The six one-turn prompts run through `--say`. The two memory entries run in
separate REPL processes. Both seeds run the complete panel; seed 1 runs it a
second time into a new output directory.

## Decoder and source checks

Every invocation uses:

- `--protocol 2`;
- `--threads 16`;
- `--temperature 0`;
- `--max-tokens 16`.

Before any prompt, verify all four artifact/tokenizer hashes, both tokenizers'
`stack_sha256` fields against their matching artifact hashes, the panel hash,
the executable and source hashes, and `uor-chat --version`. The native loader
checks vocabulary and tokenizer content identity but does not itself enforce
the JSON `stack_sha256` field, so this explicit pair check is required. Re-hash
the executable after the run. Stop before model output if any pin differs.

## Steps

Define `BIN`, `ARTIFACT`, and `TOKENIZER` from the matching section above. Use
a new output root:

`/home/callard/projects/codex-playground/uor-r4-chat-baseline/local/deepseek/public-chat-post-smoke-2026-11-02/`

Do not create it if it already exists. Use these subdirectories:

```text
seed-1/
seed-2/
seed-1-repeat/
```

Use this exact pass mapping:

| Pass | Artifact | Tokenizer | Output directory |
|---|---|---|---|
| seed 1 | seed 1 path above | seed 1 path above | `seed-1/` |
| seed 2 | seed 2 path above | seed 2 path above | `seed-2/` |
| seed-1 repeat | seed 1 path above | seed 1 path above | `seed-1-repeat/` |

For each one-turn entry, run:

```sh
/usr/bin/time -v -o OUT.time \
  "$BIN" --stack "$ARTIFACT" --tokenizer "$TOKENIZER" --protocol 2 \
    --threads 16 --temperature 0 --max-tokens 16 --say "$PROMPT" \
    >OUT.json 2>ERR.log
```

Run the six fixed entries in this order:

1. `conv-clar-01`: `Will you finish it before lunch?`
2. `conv-do-01`: `Please say this in a kinder way: Give me that toy now.`
3. `conv-do-02`: `Give me three colors for painting a house.`
4. `conv-talk-01`: `Hello! How is your day going?`
5. `conv-story-01`: `Once upon a time, there was a little bunny who lost his carrot. What happened next?`
6. `conv-unk-01`: `What did I eat for breakfast yesterday?`

Use the panel id as each output basename: for example
`seed-1/conv-do-01.json`, `.err` and `.time`.

For each memory entry, start one REPL process and send every `user_turns`
entry in order, followed by `/quit`. Before every user turn and after every
assistant turn, send `/tokens`; the post-turn list makes the generated reply
ids auditable without changing serving code. Capture stdout and stderr
separately.
The exact seed 1 commands are:

```sh
printf '%s\n' \
  '/tokens' \
  'The new puppy at our house is named Pickle.' \
  '/tokens' \
  'What did we name the new puppy?' \
  '/tokens' \
  '/quit' |
  /usr/bin/time -v -o mem-01.time \
    "$BIN" --stack "$ARTIFACT" --tokenizer "$TOKENIZER" --protocol 2 \
      --threads 16 --temperature 0 --max-tokens 16 \
      >mem-01.txt 2>mem-01.err

printf '%s\n' \
  '/tokens' \
  'Grandpa grows tomatoes in a box by the window.' \
  '/tokens' \
  'He waters them every day.' \
  '/tokens' \
  'What does grandpa grow?' \
  '/tokens' \
  '/quit' |
  /usr/bin/time -v -o mem-02.time \
    "$BIN" --stack "$ARTIFACT" --tokenizer "$TOKENIZER" --protocol 2 \
      --threads 16 --temperature 0 --max-tokens 16 \
      >mem-02.txt 2>mem-02.err
```

The seed 2 and seed 1 repeat commands replace `ARTIFACT`, `TOKENIZER`, and the
output directories exactly as listed above.

The memory inputs are:

- `conv-mem-01`: `The new puppy at our house is named Pickle.` then
  `What did we name the new puppy?`
- `conv-mem-02`: `Grandpa grows tomatoes in a box by the window.` then
  `He waters them every day.` then `What does grandpa grow?`

After both seeds finish, rerun every seed-1 command from a fresh process into
`seed-1-repeat/`. Never append to or replace a prior output file.

## Readings

For every reply record or REPL turn, record:

- exit status, complete stdout, stderr and the protocol returned by the binary
  (`.protocol.schema == "uor-r4.literal-role-dialogue/2"` for one-shot JSON,
  and the same schema in the REPL banner);
- exact response text and response ids;
- stop reason, history depth, seconds and response ids/s;
- for `conv-mem-01`, whether the second reply contains `Pickle`
  case-insensitively;
- for `conv-mem-02`, whether the third reply contains `tomato` or `tomatoes`
  case-insensitively.

For a REPL turn, read the `[served] N id(s) | ...` line and take the final `N`
integers from the immediately following `/tokens` history list as the exact
reply ids. Check that the post-turn history is longer than the pre-turn history,
that `N` is between 1 and 16, and that the recorded reply text is the decode of
those ids. The initial and pre-turn `/tokens` snapshots are the controls for
this extraction. The seed-1 repeat comparison uses the extracted id lists, not
the full history lists.

Report every transcript, including failures. The seed-1 repeat must match the
first pass in reply text, reply ids and stop reason for all eleven replies.

## Bar and decision

- **Instrument pass:** every pinned check holds; all three output roots are
  fresh; every process exits 0; each seed produces 11 replies; every reply stops
  by `eos` or the 16-token cap; each REPL transcript has the expected number of
  assistant turns; returned protocol is 2; and the seed-1 repeat is exact.
- **Conversational readiness:** the public artifact is **not ready for a useful
  chat demonstration** if either memory anchor fails on either seed.
- If both memory anchors pass, classify the artifact only as **connected and
  ready for qualitative inspection**. The published artifact README says it is
  not an assistant; this ungraded panel cannot qualify useful chat.
- A pin, build, parsing, timeout or protocol failure is an instrument failure,
  not a model result. Stop and report it without interpreting the replies.

## Minimum test power

Two initialization seeds, eight fixed entries and eleven replies per seed,
plus an eleven-reply repeat of seed 1. This has enough power for product
connectivity, protocol selection, deterministic serving and two fixed memory
anchors. It has no power to estimate general answer quality or language
capability.

Panel noise is not applicable: this run computes no quality rate or seed
comparison. Its two repeated checks are exact transcript equality, and its
memory anchors are diagnostic booleans at both seeds. Neither is promoted to a
capability estimate.

## Cost and stop rule

- Local Ryzen 9 9950X only, one process at a time, 16 worker threads. There is
  no GPU, no pod and no external spend.
- Preparation and build project at 10 seconds warm-cache and at most 2 minutes
  cold. The 33 model replies project at under 2 minutes total. Review and
  delivery project at under 15 minutes.
- Every process records wall time, CPU use and peak RSS with `/usr/bin/time -v`.
  Projected peak RSS is below 256 MiB; above 1 GiB is an instrument failure.
- The existing build target is about 1.1 GiB and is retained as an ignored local
  build. New transcripts and timing records are below 1 MiB. There are no
  checkpoints or temporary datasets. Delivery is the report and this task
  record; no model state is retained.
- There is no automatic retry. If an instrument check fails, stop and create a
  fresh sibling output root only under a recorded repair; never overwrite or
  append to the failed run.
- Before execution verify at least `30 GiB + 128 MiB` free disk and a charged
  total of `$0.00` external compute.
- Stop on any pin mismatch, a missing toolchain, a command over 60 seconds, a
  nonzero exit, an absent or malformed reply record, a protocol other than 2,
  peak RSS over 1 GiB, or a non-deterministic seed-1 repeat.

## Claim boundary

This test can show that the public stack artifact loads and emits deterministic
integer tokens through the product path under its trained dialogue protocol. It
can expose an obvious multi-turn-memory failure. It cannot establish
answer-correctness, useful conversation, complete replies, fresh phrasing or
language capability.

## Non-author TEST FITNESS review

TEST FITNESS: FIT

PRE-RUN: PASS

Reviewed at exact head
`fd57d3ba8ed87717f200b4e7c06bff49ed9f4d0e` by a fresh-context, read-only
`codex` reviewer using the configured `deepseek-flash` provider in ephemeral
session `01a19dde-2d22-74b1-ad7e-f7f97f43bfb2`. It used 12 non-model checks and
did not load the artifact or invoke a prompt.

The reviewer found no blocker. It verified the clean exact head, every listed
hash, both tokenizer-to-artifact bindings, the six panel categories and exact
selected prompts, the absent output root and free-disk floor. It also checked
the binary against its source, dependency lock and Rust 1.97.1 commit, plus
protocol 2, `/tokens` suffix extraction, the deterministic repeat, the two
memory anchors, instrument-versus-model classification, the zero-cost
projection, stop rule and claim boundary.

Its two advisories were retained: the seeds are initialization seeds on a
common batch stream rather than independent replicates, and the published
artifact is not chat-capable. The run therefore reports per-seed behavior and
does not infer cross-seed noise or language capability.
