# REPORT: one-model-end-to-end, claude-268 (a mixed pair of copy heads built through training, export and the integer serve; fbs-toks and cpu-train move to claude-269)

Lab: claude. [ORDER](./ORDER.md). PR #3624, item #3407.

**In one paragraph.** Each copy head of the Hamming read can now take its own key: `--hamming-copy-keys 2,1` keys copy head 0 on the two tokens before a position and copy head 1 on one. The float read pairs a head's halves only when its key is 2. The checkpoint, `config.json`, the snapshot and the artifact header record the list only when the keys differ, so every existing key-1 and key-2 file is byte for byte as before. The integer `CopyHeads` serves each head by its own key and matches its f64 reference on all four key lists. A mixed read tells apart two facts after the same token through its key-2 head, in the integer serve and in the float read on the GPU, and each gives the expected blend, (3Q + P) / 4, on its own control (no test yet compares the two sides on the same inputs). The guard reads an ORDER's scope from the item on `main`, which lists neither `fbs-toks` nor the cpu-train files. So their two changes, the `--copy-codes` refusal and cpu-train's per-head `CopyHeads`, are claude-269, the next unit. This unit widens the item's scope for them. Until then the integer crate's public readers refuse a mixed record, so those two fail closed instead of reading it as copy key 1. No model was trained.

## Tokens per second first

No model ran, so there is no model tok/s. The per-head key adds no work to the serve: a head's code is joined or not by one stored mask, as before.

## What did not work

1. **The guard refused `crates/uor-r4-integer/src/bin/fbs-toks.rs`.** The ORDER named it and three cpu-train files in scope, but the guard reads the scope from the item file on `main`, as claude-260 found. Classification: instrument (the unit's own scope), repaired by widening the item's scope in this PR (ORDER, Amendment). The two changes those files carry move to claude-269. Until it merges, a mixed artifact is served by the integer session, and cpu-train's served forward and every `fbs-toks` mode that reads the record (`--copy-codes`, `--hamming-trace`, `--forced --decision-trace`, `--context-sweep`) refuse it (item 2).
2. **My first draft left that gap open, and the fresh review caught it.** The draft's Amendment called the gap safe because no mixed artifact exists yet. But `StackHamming` now declares `copy_keys`, so a header carrying it no longer fails `deny_unknown_fields`, and cpu-train, which builds one pair mask from `copy_key`, would have read a mixed record as copy key 1, with no error. Classification: instrument (my own claim, not the mechanism). Repaired in this PR: the integer crate's two public readers, `read_hamming_uorlut01` and `served_copy_codes_uorlut01`, refuse a record with `copy_keys`. They are the only ways cpu-train and `fbs-toks` reach the record. The session reads it through the crate's own path and still serves it. A negative gate shows the test fails with the refusal set aside (Checks).
3. **Seven `uor-r4-integer` integration targets fail on this laptop**, at any head: `adversarial_challenger_m1_token_boundary`, `adversarial_challenger_m2_projection_latency`, `challenger_m2_stress_test`, `challenger_m3_hardware_invariants`, `diagnostic_slot_scaling`, `real_bundle_alignment_m1` and `zero_matmul_4k_projection_m2`. Each reads a fixture bundle under `/Users/casey.allard/uor-r4-investigations/`, which no longer exists after the 20 October workspace reset. Not this change's (instrument, the environment); recorded, not fixed.
4. **The stub `nvcc` cannot check the CUDA build any more.** The vendored `candle-kernels` build calls `nvcc --ptx`, a mode the stub does not know. So the CUDA-only tests were compiled and run on a pod instead (Checks).

## What changed

- **The float read** (`flock_read_hamming.rs`):
  - `HammingRead` holds one key per copy head, and `pair_halves` takes a per-head mask, so a key-1 head keeps `x_{s−1}`'s whole code;
  - `with_copy_keys`, `copy_keys` and `mixed_copy_keys` are new, with `check_copy_keys`. `copy_key()` is the shared key, or 1 when the keys differ;
  - `hamming_read.safetensors` records `copy_keys` only when mixed. Loading refuses a file that records both, or a list whose keys are all the same.
- **The trainer:** `LongConfig.hamming_copy_keys` (left out of `config.json` when none) and `--hamming-copy-keys K0,K1`. It refuses `--hamming-copy-key` and `--hamming-copy-keys` together, and a list that gives every head the same key (that is `--hamming-copy-key`), before it builds the read. So `config.json` and the checkpoint name a key the same way.
- **The export** (`fbs_export.rs`) writes the snapshot's `copy_keys` when mixed.
- **The integer record and serve:**
  - `StackHamming.copy_keys` and `FbsHammingSnapshot.copy_keys` are optional and left out when uniform;
  - `head_copy_key(h)` gives each head's key;
  - `check()` refuses a list that is the wrong length, names anything but 1 or 2, is all equal, or comes with `copy_key` 2. A key-2 head still needs whole-lane halves;
  - `CopyHeads.pair` is one optional mask per head;
  - the public readers `read_hamming_uorlut01` and `served_copy_codes_uorlut01` refuse a record with `copy_keys` until claude-269 ("this reader takes one copy key for every head").
- **The item's scope** gains `bin/fbs-toks.rs` and cpu-train's `hamming.rs`, `served_forward.rs` and `fbs.rs`, for claude-269.

## Measurements

These are tests, not results:

| Test | Where | Result |
|---|---|---|
| (a) `copy_heads_match_the_f64_reference`, key lists `[1,1]` `[2,2]` `[2,1]` `[1,2]` | integer | pass |
| (b) `mixed_copy_keys_are_recorded_only_when_they_differ`: header and artifact round trip, refusals at export, load and session; both public readers refuse a mixed record | integer | pass |
| (b) `the_trainer_refuses_copy_keys_it_would_record_two_ways`: `[2,2]` `[1,1]` `[2]`, with `--hamming-copy-key 2`, and without the read | float | pass |
| (b) `hamming_record_checks_name_their_reason`: seven bad lists, and a key-2 head on 12-coordinate heads | integer | pass |
| (b) `the_copy_key_is_recorded_only_when_two` (unchanged) | integer | pass |
| (b) `a_copy_key_is_one_or_two_tokens_on_halves_of_whole_lanes`, `the_copy_key_is_saved_only_when_two` (now also an all-equal saved list, refused), `a_config_writes_the_copy_key_only_when_two` | float | pass |
| (c) `copy_key_two_tells_two_facts_after_the_same_token_apart`: each mixed head reads as its own key's uniform record, and the mean is exactly `(3Q + P) << 29` | integer | pass |
| (c) the same, in the float read: both mixed lists give (3Q + P) / 4 at every column | float, CUDA | pass (RTX 5090, at `9fc262d79`) |
| `pair_halves_puts_the_first_token_in_each_copy_heads_low_half`, masks `[t,t]` `[t,f]` `[f,t]` | float, CUDA | pass (RTX 5090, at `9fc262d79`) |

## Next

- **claude-269:** `fbs-toks --copy-codes` refuses a mixed artifact, and cpu-train's private `CopyHeads` and served-forward audit take a per-head key, with a mixed parity test. DeepSeek gets a `[claude]` note naming the change.
- **Then the mixed pair's GPU run:** a sheet with FIT, three seeds against (b). Its proxy is claude-267's `k2>k1` count (4,388 and 13,990 against copy key 1's 3,859 and 12,371). The trained form differs: the mixed pair averages both heads' means where both find a candidate, while `k2>k1` takes copy key 2's match alone.

## Checks run at the head

All at code head `9fc262d79` (the commits after it change only docs):

- **On the laptop**, with `--no-fail-fast`:
  - `cargo test -p uor-r4-integer --features serving-wasm`: the library 452 passed, 0 failed, 1 ignored, and the `fbs-toks` bin 7 passed. The seven fixture-bundle targets fail as in What did not work (item 3); every other target passes.
  - `cargo test -p uor-r4-float-train`: the library 225 passed, 0 failed.
  - `cargo test -p uor-r4-cpu-train`: the library 159 passed, 0 failed, 4 ignored.
- **On the GPU**, one RTX 5090 pod (Vast, about 15 minutes at $0.975/h, then deleted): `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` gave 311 passed, 0 failed, 7 ignored. The claude-268 tests passed by name: `pair_halves_puts_the_first_token_in_each_copy_heads_low_half`, `copy_key_two_tells_two_facts_after_the_same_token_apart`, `a_copy_key_is_one_or_two_tokens_on_halves_of_whole_lanes`, `the_copy_key_is_saved_only_when_two`, `a_config_writes_the_copy_key_only_when_two` and `the_trainer_refuses_copy_keys_it_would_record_two_ways`. `train-in-space` and `fbs-snapshot` then built with CUDA. The sealed folder is `local/claude/claude-268/i0-268`, with `SHA256SUMS.files` sha256 `2da3831fab60261ae5aefd19b35d1df93f53752649f5ec71b873b0441aab00ea`, verified after the copy.
- **Negative gates**, each run on the laptop, then restored byte for byte and rebuilt green:
  - with `CopyHeads`' per-head key replaced by the shared `record.copy_key == 2`, `copy_key_two_tells_two_facts_after_the_same_token_apart` and `copy_heads_match_the_f64_reference` both fail;
  - with both `one_copy_key` calls removed, `mixed_copy_keys_are_recorded_only_when_they_differ` fails at "read_hamming refuses [2, 1]".
- **Review:** two fresh-context rounds. Round 1 asked for changes: the fail-open gap (What did not work, item 2) and three minor items, all fixed. Round 2 approved, and its three wording notes are fixed in the ORDER and this REPORT.

## Skills used

None applied.

## Still open

1. **claude-269** (above): the `fbs-toks` refusal and cpu-train's per-head key, which also lifts the public readers' refusal. Until then a mixed artifact is served, and cpu-train and every `fbs-toks` mode that reads the record refuse it.
2. **The mixed pair's trained result** is unknown. It averages the two heads where both find a candidate, so it is not `k2>k1`; only the run can show which is better.
3. **The seven integration targets** that read `/Users/casey.allard/uor-r4-investigations/` fail on this laptop, outside this ORDER's scope.

## Plain summary

The model copies words from earlier in the conversation through two "copy heads", each matching a short key at every position. Until now both heads had to use the same key: either one word back, or two words back in order. A count on the data in the last unit suggested that one of each would be right more often. This unit makes that possible. Each head can now have its own key, in training, in the saved files and in the laptop's integer model. The files for the old settings are unchanged byte for byte. On a small hand-built case, the laptop's integer model and the trained read each give the expected answer for a mixed pair, each in its own test. A check of the two sides against each other, on real trained weights, comes with the training run. Two small checking tools still need the same change, and they are the next unit; until then they refuse a file with mixed keys rather than misread it. After that, the combination gets trained and compared on the GPU.

References #3407.
