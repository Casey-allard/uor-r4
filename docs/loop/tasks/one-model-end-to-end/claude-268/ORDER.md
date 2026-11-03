# ORDER: one-model-end-to-end, claude-268 (a mixed pair of copy heads: one on copy key 2, one on copy key 1, built through training, export and the integer serve)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `6392cdcce7682eb57e7e3d2b30b9c606551fdea9` (main with claude-267 merged). Hand-written under the umbrella exception (cancel probe 2).

## Goal

claude-267's count found that copy key 2, backed off to copy key 1, was right on 4,388 (DEV) and 13,990 (p-w) copyable positions. Copy key 1, the trained base, was right on 3,859 and 12,371. This unit builds the trained-read form of that rule: each copy head gets its own key. The GPU run is the next unit, with its own sheet and FIT.

**Plan step (the claim check):** order step 4, one model trained as served end to end. **What it changes in the model:** which key each copy head of the Hamming read matches on, in training and in serving alike.

## The design (mine, reversible)

- `--hamming-copy-keys K0,K1,…` gives one key per copy head, each 1 or 2. Its length must equal the number of copy heads (2 in the recipe). `--hamming-copy-key K` stays, and means every head uses K.
- **Records.** A uniform key is written exactly as today: the scalar `copy_key`, omitted at 1. A mixed list is written as a new field, `copy_keys`, in `hamming_read.safetensors`, `config.json`, the snapshot and the artifact header. That keeps copy keys 1 and 2 byte-identical. A uniform list is never written as `copy_keys`. Old engines refuse a mixed artifact through `deny_unknown_fields`.
- **The read.** Head h pairs its halves (`pair_halves`) only if its key is 2. A key-1 head keeps the whole code of `x_{s−1}`.
- **The serve.** `CopyHeads` holds one optional pair mask per head. `fbs-toks --copy-codes` refuses a mixed artifact, as it refuses key 2.
- **`uor-r4-cpu-train`** (DeepSeek's crate, borrowed under Casey's direction of 3 November) gets the same per-head change in its private `CopyHeads` and audit. Today any key other than 2 reads as 1 there, so a mixed artifact would be misread silently. DeepSeek gets a `[claude]` note naming the change.

## Tests

- (a) A mixed read is served exactly as trained, as claude-261's copy-key-2 parity tests do, in the integer stack and in cpu-train's served-forward audit.
- (b) Records:
  - `copy_keys` is written only when the keys are mixed;
  - wrong lengths and values are refused;
  - existing key-1 and key-2 bytes are unchanged.
- (c) The mixed read tells apart two facts after the same token, through its key-2 head.
- (d) Every existing copy-key test passes, cpu-train's included.
- CUDA-only float tests run once on a pod, as finite kernel tests (no sheet).

## Cancel probe

1. **Already done?** `git grep -i -e copy_keys -e 'hamming-copy-keys' -e 'per-head copy key'` finds nothing in `crates/` or `docs/roadmap/`. Not cancelled.
2. **The lock.** `loop claim one-model-end-to-end --lab-topic mixed-heads` refused: the item file overlaps the `docs/roadmap/items/*.md` glob of #3622 (deepseek-104, draft). Its files are its own task folders, `claims.json`, `docs/roadmap/{README.md,issues.json}` and `items/geometric-image-input.md`. None is in this scope.
3. **Inputs.** None (a build unit).

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-268/**`, `docs/research/notebook/float-training.md` (added rows), `docs/roadmap/items/one-model-end-to-end.md`
- `crates/uor-r4-float-train/src/{flock_read_hamming.rs,in_space_long.rs,fbs_export.rs,bin/train-in-space.rs}`
- `crates/uor-r4-integer/src/{fbs_snapshot.rs,stack/format.rs,stack/hamming.rs,stack/synthetic.rs,stack/tests.rs,bin/fbs-toks.rs}`
- `crates/uor-r4-cpu-train/src/{hamming.rs,served_forward.rs,fbs.rs}` (the borrowed change only)

## Done when

1. Tests (a)–(d) pass, and the three crates' suites pass with `--no-fail-fast` (CUDA-only tests on one pod).
2. `uor-harness loop gates --full`, `loop report --check` and `loop pr-check --local` pass.
3. A fresh-context review approves at the exact head.

## Amendment (3 November, mid-build): the guard's scope

The loop guard reads an ORDER's scope from the item file on `main`, as claude-260 found. The item lists neither `bin/fbs-toks.rs` nor the three cpu-train files, so the guard refused the first edit to `fbs-toks.rs`.

**My decision (top session, reversible).** This unit lands the float and integer sides, which are in scope. It also widens the item's scope by those four files. The `fbs-toks --copy-codes` refusal and cpu-train's per-head change become claude-269, the next unit, before any GPU run.

**The gap fails closed (corrected after the first review).** The first draft said the gap was safe because no mixed artifact exists yet. The fresh review refuted that. `StackHamming` now declares `copy_keys`, so a header carrying it no longer fails `deny_unknown_fields`, and cpu-train, which builds one pair mask from `copy_key`, would read a mixed record as copy key 1. So the two public integer readers, `read_hamming_uorlut01` and `served_copy_codes_uorlut01`, refuse a record with `copy_keys` ("this reader takes one copy key for every head"). They are in this unit's scope (`stack/hamming.rs`), and they are the only ways cpu-train and `fbs-toks` reach the record. The session, `IntegerStackModel`, still serves mixed keys through the crate's own path. The refusal reaches every `fbs-toks` mode that reads the record, not only `--copy-codes`: `--hamming-trace`, `--forced --decision-trace` and `--context-sweep` refuse a mixed artifact too. claude-269 lifts the refusal when it gives those callers a key per head.

**Tests.** Test (a)'s cpu-train half and test (d)'s cpu-train clause move to claude-269. This unit still runs the cpu-train suite, to show the integer change leaves it green.

## Stop and decide (never stop and wait)

- If a struct literal outside scope breaks, add the field so that it keeps compiling (for example `..Default::default()` or a constructor). Otherwise record it and decide.
- If a per-head key cannot be served exactly as trained, stop the build and record why. Do not ship a key that only trains.
