# REPORT: one-model-end-to-end, claude-269 (cpu-train and fbs-toks read a copy key per head; the integer readers' refusal is lifted)

Lab: claude. [ORDER](./ORDER.md). PR #3627, item #3407.

**In one paragraph.** claude-268 gave each copy head its own key in the float read, the export and the integer session, and left its two other consumers failing closed. This unit gives both a key per head. cpu-train's `CopyHeads`, the trainer's own copy of the integer serve, now builds one pair mask per copy head from `head_copy_key`, as the integer `CopyHeads` does. Its served-forward audit and its forward at both hooks match the integer session exactly on mixed artifacts at `[2,1]` and `[1,2]`, and the audit's manifest records `copy_keys` when the heads differ. `fbs-toks --copy-codes`, whose per-token reconstruction assumes copy key 1, refuses a header with `copy_keys`. Its other modes drive the session, or read only the record's head count and the writer table, so they now take a mixed artifact. The integer crate's public readers no longer refuse one. A negative gate shows the mixed parity tests fail when cpu-train goes back to one key for every head. No model was trained.

## Tokens per second first

No model ran, so there is no model tok/s. The change adds no work to the audit: each copy head looks up its own stored mask instead of a shared one.

## What did not work

1. **cpu-train builds race on this laptop, and three of mine failed to compile** with errors the source does not have: "can't find crate for `uor_r4_core`" and "unresolved import `uor_r4_integer::stack`". The cause, read from `cargo -v`:
   - `uor-r4-discrete`'s build script depends on `uor-r4-core`, which depends on `uor-r4-integer`. Under resolver 2 that host copy of the integer crate has no features, while cpu-train asks for `serving-wasm` and `fbs-synthetic`. So one build compiles the integer crate twice (`-C metadata` `ebbe49189a096fa7`, no features; `5eb848be23ef948f`, both).
   - The integer crate is a `cdylib` as well as an `rlib`, so cargo gives its files no hash suffix. Without `--target`, host and target builds share `target/debug/deps`, so both units write the same `libuor_r4_integer.rlib`, and the last to finish wins. When the featureless one wins, cpu-train finds no `stack` module, and core no longer matches the integer crate it was built against.
   - cpu-train has depended on `uor-r4-discrete` since #3511 (1 November), so every cpu-train build without `--target` can race; claude-268's runs happened to compile.

   A build that compiles is still sound: the featureless unit has no `stack` module, so cpu-train cannot compile against it. Classification: instrument (the build graph), not the change. The negative gate's first try was one of these failed builds, so it proved nothing and was run again. The verdict below is from a build that compiled. Not fixed: the `Cargo.toml` files of `uor-r4-discrete`, `uor-r4-core` and `uor-r4-integer` are outside this ORDER's scope. `--target aarch64-apple-darwin` separates the host build and avoids the race. DeepSeek, who owns #3511, gets a note.
2. **Eight `uor-r4-integer` integration targets fail on this laptop.** Seven fail at any head, as in claude-268: `adversarial_challenger_m1_token_boundary`, `adversarial_challenger_m2_projection_latency`, `challenger_m2_stress_test`, `challenger_m3_hardware_invariants`, `diagnostic_slot_scaling`, `real_bundle_alignment_m1` and `zero_matmul_4k_projection_m2`. Each reads a fixture bundle under `/Users/casey.allard/uor-r4-investigations/`, which no longer exists after the 20 October workspace reset. The eighth is new: `openai_loopback`'s `http_busy_preflight_disconnect_and_header_bounds` (Kimi's #3606, merged today), whose second stream got 200 where the test expects 429. The first stream asks a synthetic model for 40 tokens, which can finish before the second request arrives; run alone, the test passed three times out of three. Classification: instrument (the environment and a timing test), not this change, which does not reach the loopback server; recorded, not fixed.

## What changed

- **cpu-train `hamming.rs`:** `CopyHeads.pair` is one optional mask per copy head, from `record.head_copy_key(h)`, and `choose` reads the head's own mask. The two-facts test gains the two mixed masks.
- **cpu-train `served_forward.rs`:** the manifest's Hamming entry gains `copy_keys`, left out when none. The test helper `artifact_with_keys` builds an artifact with a key list.
- **cpu-train `fbs.rs`:** the Hamming test fixture takes a key list. A uniform list sets `copy_key` and a mixed one `copy_keys`, as the exporter records them.
- **`fbs-toks`:** `artifact_hamming` reads the header's Hamming record, which `artifact_copy_key` now uses. `require_one_token_copy_codes` also refuses a header with `copy_keys` ("--copy-codes audits the one-token copy key only; the artifact records copy_keys [2,1]"), since the list is written only when the keys differ, so one head is two-token.
- **integer `stack/hamming.rs`:** `one_copy_key` and its two calls are gone. `read_hamming_uorlut01` and `served_copy_codes_uorlut01` return a mixed record as it is.

## Measurements

These are tests, not results:

| Test | Where | Result |
|---|---|---|
| (a) `mixed_copy_keys_audit_matches_serving_exactly`, at `[2,1]` and `[1,2]`: zero query, token and kept-cell mismatches against the integer session, the same decision sha256, no instrument fault; the manifest has copy key 1 and the list | cpu-train | pass |
| (a) `mixed_copy_key_heads_match_serving_at_both_hooks`, at `[2,1]` and `[1,2]`: the complete query, the token and the kept cells equal the session's at every row, and the copy reads move the query | cpu-train | pass |
| `copy_key_two_audit_matches_serving_exactly` (now also: no `copy_keys` in a key-2 manifest) and `copy_key_two_heads_match_serving_at_both_hooks` | cpu-train | pass |
| (b) `copy_key_two_tells_two_facts_after_the_same_token_apart`, mixed masks `[Some, None]` and `[None, Some]`: each head chooses as its own key's uniform heads do | cpu-train | pass |
| (c) `copy_codes_refuses_every_non_one_token_key_before_comparison`: `copy_keys` `[2,1]` and `[1,2]` are refused, while `artifact_copy_key` still reads 1 | `fbs-toks` | pass |
| (d) `mixed_copy_keys_are_recorded_only_when_they_differ`: both public readers return the parsed record with its `copy_keys`, and the served codes equal copy key 1's, since a key only pairs them | integer | pass |

## Next

- **The training-time unit** from claude-268's speed survey: batched decode by default with parity, streaming decode, cached arm replay, fewer host syncs.
- **Then the mixed pair's GPU run:** a sheet with FIT, three seeds against (b). Its proxy is claude-267's `k2>k1` count (4,388 and 13,990 against copy key 1's 3,859 and 12,371). The trained form differs: the mixed pair averages both heads' means where both find a candidate, while `k2>k1` takes copy key 2's match alone.

## Checks run at the head

All at code head `6b8add630` (the commits after it change only docs):

- **On the laptop**, with `--no-fail-fast`:
  - `cargo test -p uor-r4-cpu-train`: the library 161 passed, 0 failed, 4 ignored, and the bins 12 passed. The first full run failed to compile (What did not work, item 1); this is the second, with nothing changed.
  - `cargo test -p uor-r4-integer --features serving-wasm`: the library 452 passed, 0 failed, 1 ignored, and the `fbs-toks` bin 7 passed. The eight targets in What did not work, item 2, fail; every other target passes.
  - `cargo fmt -p uor-r4-cpu-train -p uor-r4-integer --check`: clean.
- **Negative gate** for (a): with cpu-train's per-head mask replaced by the shared `record.copy_key == 2`, `mixed_copy_key_heads_match_serving_at_both_hooks` and `mixed_copy_keys_audit_matches_serving_exactly` both fail, while the two key-2 parity tests and the two-facts test pass. Restored byte for byte (`cmp`), rebuilt, and the five tests pass.
- **Review:** a fresh-context review at `5180ce531` approved, with three docs-only findings in this REPORT (a wrong cross-reference, the loopback flake missing from Still open, and an overstated audit claim), fixed in the next commit; a second fresh-context review read that commit, recorded on PR #3627 by `loop review`.

## Skills used

None applied.

## Still open

1. **The mixed pair's trained result** is unknown. It averages the two heads where both find a candidate, so it is not `k2>k1`; only the run can show which is better.
2. **No test compares the float read with the integer serve on the same trained weights.** cpu-train's audit compares its own integer copy of the serve with the session, which now holds on a mixed artifact; a float-against-integer check of a mixed pair comes with the training run's export.
3. **The seven integration targets** that read `/Users/casey.allard/uor-r4-investigations/` fail on this laptop, outside this ORDER's scope.
4. **`openai_loopback`'s busy test** (#3606) is a timing flake: it failed once in the full suite and passed three times alone (What did not work, item 2). It is Kimi's to make deterministic; Kimi gets a note.
5. **The cpu-train build race** (What did not work, item 1) is not fixed: it needs a `Cargo.toml` change outside this scope. DeepSeek gets a note.

## Plain summary

The model copies words from earlier in the conversation through two "copy heads", and since the last unit each head can match on its own key: one word back, or two words back in order. Two checking tools still assumed both heads used the same key, so they refused a file with mixed keys rather than misread it. This unit teaches both about the per-head key. The trainer's check of the laptop model now reads each head by its own key and agrees with the laptop model exactly on mixed files. The one tool mode that only works for one-word keys says so and stops. Nothing was trained. Next comes a unit to make training faster, then training the mixed pair on the GPU.

References #3407.
