# REPORT: lawful M-world oracle-corpus support, deepseek-8

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](ORDER.md). Base:
`5e50f3e193746996c6157aa7b82cb6b6bc27584f`.

## What did not work, first

- **Claude-259's stated corpus route is not buildable on current `main`.**
  `crates/uor-r4-training/examples/m-world.rs` has no `[[example]]` target:
  D29 set `autoexamples = false`, and that retained source imports historical
  compiler types. Running `cargo test -p uor-r4-training --example m-world`
  fails with “no example target named `m-world`”. Adding coverage only to that
  source would not execute in the lawful build.
- **The first real-tokenizer attempt used the wrong public artifact.** Public
  upstream SmolLM2 does not carry the project's atomic `<|bos|>`, `<|eos|>` and
  `<|unk|>` tokens, so the writer correctly refused it. The exact pinned
  `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`
  tokenizer was then found in the existing local lab cache. The test verifies
  that SHA before generating, and no substitute vocabulary is used.
- **The byte-tokenizer control is a finite corpus smoke, not model capability.**
  It uses the repository test's deterministic byte tokenizer and a relation-only
  M-world mix because byte-level character counting cannot hit MQAR's short
  distance bands. It proves encoding, mask length, memory-line counts, matched
  controls, determinism and report sealing. It does not prove training benefit
  and is not substituted for the pinned-tokenizer test.
- **Probe decontamination is not duplicated in the new writer.** The historical
  probe module is excluded from the native graph. Callers can pass an
  `exclude=PANEL.json` reply panel, and the output remains composable with
  `mix-chat-corpus`; automatic sealed-probe screening needs its own lawful
  implementation.

## What changed

- Added `m-world-recall-corpus`, an explicitly compiled Rust writer. It imports
  only `milestone_world`, `milestone_world_v2`, the tokenizer's literal-role
  protocol, the UORT corpus writer and report sealing.
- Modes:
  - `recall=off` is the default matched control and emits no memory turns.
  - `recall=oracle` inserts one `System: Memory: ...` turn for every MQAR or
    relation query, at `recall_at=reply` or `recall_at=query`.
- The manifest distinguishes value-bearing lines from `Memory: none.` lines:
  `value_lines` and `none_lines` sum to `lines`.
- The writer records the M-world version, revision, digest, mix, seed, protocol,
  context, per-kind and per-category counts, exclusion-panel identity, token and
  mask SHA-256 values, and the sealed wrapper root.
- Tokenizer and exclusion-panel bytes are read once; the same bytes are parsed
  and hashed, so the manifest cannot name a different file version from the one
  the generator used.
- Per-category response counts use the selected protocol's meter and sum to the
  token store's response count.
- Enabled the existing lawful `mix-chat-corpus` source as a Cargo binary, so
  the generated off/oracle stores can be composed without reintroducing a
  historical target.
- Added the binary targets and widened the item's file scope. Historical
  `examples/m-world.rs` is byte-unchanged.

## Measurements

With the exact pinned tokenizer, protocol 2 and the default M-world mix, the
bounded 128-conversation pair generated:

| Reading | `recall=off` | `recall=oracle` |
|---|---:|---:|
| Conversations | 128 | 128 |
| Turns | 446 | 446 |
| Tokens | 15,134 | 16,214 |
| Response tokens | 4,903 | 4,903 |
| Recall lines | 0 | 72 |
| Value-bearing recall lines | 0 | 64 |
| `Memory: none.` lines | 0 | 8 |
| Longest episode | 241 | 256 |

| Corpus | `tokens.u16` SHA-256 | `response_mask.u8` SHA-256 |
|---|---|---|
| off | `418de0957841455deb6c0f519b96cdb07650c1806222d0619ff95e1f833af443` | `78d06b5a29caa378e13313add83748ee320b9be5b81f971a59d5b24ab262aa9d` |
| oracle | `d8c84bc5194ca79543a2a2eeff0f26297cee947dde98cb5b8d078ab1ee0fd45e` | `8b72a0ff12d706ff7fba10ab8f4d17af02402bd4e6bcf9b4e4a56ba46a8e322e` |

The real pinned-tokenizer test generated the oracle corpus twice and produced
byte-identical token stores, masks and train manifests. Decoding yielded exactly
72 `Memory:` lines, of which 8 were `Memory: none.`. The off control had the
same 128 conversations and 446 turns, no memory turns and no recall block.
Per-kind token sums equal the store total and per-category response sums equal
the store's 4,903 response tokens.

The synthetic byte-tokenizer control remains useful as an independent fixture:
24 conversations, 66 turns, 4,793 tokens, 10 recall lines and the same matched
off/oracle behavior.

The default `conversations=5600`, seed 73, protocol 2 generation also completed
for both modes in about 13 seconds each:

| Reading | `recall=off` | `recall=oracle` |
|---|---:|---:|
| Turns | 19,895 | 19,895 |
| Tokens | 680,952 | 734,929 |
| Response tokens | 227,031 | 227,031 |
| Recall lines | 0 | 3,563 |
| Value-bearing recall lines | 0 | 3,136 |
| `Memory: none.` lines | 0 | 427 |
| Longest episode | 245 | 263 |

| Corpus | `tokens.u16` SHA-256 | `response_mask.u8` SHA-256 |
|---|---|---|
| off | `562d5a3de8053c65a2434750ad2322ebe234ddbd72b1b37dfbdff7ed16ba2ff5` | `c1270548bdca5a5eafaf58f4cb7115f0cdb716347fe54bab589bd90e7b184785` |
| oracle | `299d3a46aa01ca1d8e6af0d19aeb5b80dce9513a32748153b3138a75d0bdb685` | `64cf637890f6a310459058f59ac041335dd8a2888f262b2a96742b89869289b3` |

The existing `mix-chat-corpus` consumer accepted the two generated stores and
emitted a 31,348-token, 9,806-response-token store with one source span per
input. This is a structural integration check, not training.

No training, model evaluation, GPU run or capability claim was made.

## Checks run at the head

- `cargo test --locked -p uor-r4-training --bin m-world-recall-corpus`:
  **5 passed, 1 ignored**; the ignored test is the pinned-tokenizer path.
- `UOR_R4_PINNED_TOKENIZER=<pinned path> cargo test --locked
  -p uor-r4-training --bin m-world-recall-corpus -- --include-ignored`:
  **6 passed**, including the exact pinned tokenizer and default mix.
- `cargo test --locked -p uor-r4-training --lib`: **80 passed**.
- `cargo check --locked -p uor-r4-training --bin m-world-recall-corpus`: passed.
- `cargo build --locked -p uor-r4-training --bin mix-chat-corpus`: passed.
- `mix-chat-corpus` on the two generated 128-conversation stores: passed,
  31,348 tokens and 9,806 response tokens across two exact source spans.
- `cargo fmt --check -p uor-r4-training`: passed.
- `uor-harness ratchet d23`: passed, 4,451 unchanged.
- `uor-harness ratchet d25`: passed, 93 unchanged.
- `git diff --check`: passed.

## Skills used

No available Codex skill applied. The repository charter, D29 target boundary
and admitted M-world/tokenizer APIs were read directly.

## Still open

- Upload the generated TRAIN-scale off/oracle stores and their SHA-256 values
  to the private store. Uploading requires private-store write authority.
- Decide whether the next training unit composes the M-world source with the
  pinned chat store through `mix-chat-corpus` or consumes an integrated writer.
- Bring the sealed English probe's exclusion rule into the lawful writer before
  calling a generated TRAIN fully decontaminated.
- Claude-260 deliberately keeps TRAIN fixed and tests the two-token key alone,
  so this corpus is ready for a later isolated key-plus-memory comparison.

## Plain summary

The old M-world source still documents an oracle-memory corpus option, but it is
not compiled after the D29 cleanup. This unit adds a small lawful writer that
can make the missing off/oracle training pair without importing the old
compiler. Its controls prove the generated oracle corpus contains real memory
turns and repeats byte for byte with both a synthetic fixture and the exact
pinned tokenizer. The full TRAIN-scale corpus, store upload and training run
remain open.
