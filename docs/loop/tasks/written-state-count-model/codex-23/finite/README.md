# Finite S2 ordinary-client replay

The inputs are the public M1 seed 1 D11 (`04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`) and its original byte BPE tokenizer (`a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`). The fixture places explicit retained-count rows on that unchanged trunk payload using the historical count-replay layout. Its source header carries the public trunk digest, zero for the absent TRAIN digest, and the SHA-256 of the planted row bytes in the synthetic source slot.

From the repository root, set `base` and `tokenizer` to those files, `build` to an absolute Cargo target directory, and `run` to a new absolute scratch path whose parent exists.

```sh
fixture=docs/loop/tasks/written-state-count-model/codex-23/finite
CARGO_TARGET_DIR="$build" cargo build --locked -p uor-r4-integer --bin uor-chat --bin activate-s2-expert
CARGO_TARGET_DIR="$build" cargo build --locked --manifest-path "$fixture/Cargo.toml"

"$build/debug/active-s2-finite" make "$base" "$tokenizer" "$run"
"$build/debug/active-s2-finite" validate "$run" "$build/debug/uor-chat" \
  "$build/debug/activate-s2-expert" "$base" "$tokenizer"
```

`make` writes the passive fixture, raw 20-byte rows, whitespace-delimited query IDs, explicit route plan and exhaustive Q32 mass oracle. `validate` invokes the production activator and ordinary client, checks matching payload bytes and equal total file size, and records the arguments, exit status, stdout and stderr of each subprocess. It also checks runtime route keys against `PrimeRouteKey` for orders zero through ten, all legal and excluded ID masses on each finite query, and exact sparse top-eight ordering. It verifies loading failures by passing a stale input digest, a byte-altered valid tokenizer JSON file and a serialized rank greater than 31. Fresh processes test deterministic reloads.

The original tokenizer frames both text prompts with the same last nine IDs. The tenth ID is `677` for red and `924` for blue. The selected row copies that ID. The appended ID makes a bigram reachable, and that row emits EOS. The raw prompts also exercise a missing order ten with a retained order nine, uniform fallback, a four-way count tie, and boundaries with higher-order decoy rows. These are planted finite serving controls.

To reproduce the ordinary text calls directly after validation:

```sh
"$build/debug/uor-chat" --discrete "$run/fixture-passive.d11" \
  --tokenizer "$tokenizer" --say 'red saw the oak' --max-tokens 2
"$build/debug/uor-chat" --discrete "$run/validation/fixture-active.d11" \
  --tokenizer "$tokenizer" --say 'red saw the oak' --max-tokens 2
"$build/debug/uor-chat" --discrete "$run/validation/fixture-active.d11" \
  --tokenizer "$tokenizer" --say 'blue saw the oak' --max-tokens 2
"$build/debug/uor-chat" --discrete "$run/validation/fixture-active.d11" \
  --prompt-ids 0,55,2728,28,677,384,265,4027,201,35,560,652,714,28 --max-tokens 2
```
