# ORDER: m4-export-uorlut01-cli, deepseek-3 (default the trained protocol)

Lab: deepseek. Item: [m4-export-uorlut01-cli](../../../../roadmap/items/m4-export-uorlut01-cli.md) (#2859). Base: `7394c344d`.

## Goal

Make `uor-chat --stack` default to literal-role protocol 2, the encoding used by the pinned training corpus and by the one-model scoring and serving runs. Protocol 1 remains available explicitly for old artifacts.

Casey's program goal is a model that actually chats locally. Claude-259 found that protocol 1 inserts a separate space token after each role marker, while the corpus merges that space into the following word. Its trained artifacts therefore saw the pinned representation under protocol 2 instead of the known protocol-1 separator-space mis-tokenization. This unit repairs the default without changing either protocol, the model path, or Claude's active copy-key branch.

## Decision and whole-project fit

This is an instrument/default repair, not a model result. It advances the open M4 CLI path and the owner sequence's step 4 by making the ordinary command use the representation the model was trained on. It does not change the `UORLUT01` format, trained artifact, scoring bar or any mechanism.

The safe transition is:

- the default changes from 1 to 2;
- `--protocol 1` continues to select the old encoding;
- `--protocol 2` continues to select the trained encoding;
- help text and the in-code default state agree.

## Cancel probe

1. **Already fixed?** At `7394c344d`, `CliArgs::default()` sets `protocol: 1`, the help says default 1, and the parser has no fallback to 2. Not cancelled.
2. **Another lab in flight?** The only open PR is #3579; its file list does not include `uor-chat.rs`. Not cancelled.
3. **Compatibility constraint?** Protocol 1 remains selectable, so old artifacts are not removed. Not cancelled.

## Files in scope

- `crates/uor-r4-integer/src/bin/uor-chat.rs`
- `crates/uor-r4-integer/README.md`
- `docs/loop/tasks/m4-export-uorlut01-cli/deepseek-3/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/m4-export-uorlut01-cli.md`
- `docs/harness/claims.json`

## Done when

1. `CliArgs::default().protocol == 2`.
2. The help text names protocol 2 as the default.
3. A focused unit test pins the default, and the existing `uor-chat` tests pass with `serving-wasm`.
4. A finite CLI invocation without `--protocol` reports protocol 2, while an explicit `--protocol 1` still reports protocol 1.
5. The REPORT records the old failure classification and the exact checks run.

## Stop if

- Another open PR starts editing the same file. Merge forward and re-scope instead of overwriting it.
- Making protocol 2 the default requires removing protocol 1 or changing the tokenizer format.
- The finite default invocation cannot load a public trained FBS artifact for a reason unrelated to protocol selection.
