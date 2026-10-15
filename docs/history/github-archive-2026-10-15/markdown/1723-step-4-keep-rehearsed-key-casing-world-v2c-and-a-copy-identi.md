# #1723 — Step 4: keep rehearsed-key casing (world=v2c) and a copy-identity table (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T06:25:27Z
- **Updated:** 2026-10-09T13:08:47Z
- **Closed:** 2026-10-05T06:48:10Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1723
- **Merged:** 2026-10-05T06:48:09Z
- **Merge commit:** 19dc379467601f18a9865b2d1595bca7c0d0b0d4
- **Base / Head:** main / UOR-Foundation:codex/step4-key-casing
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Step 4: keep the rehearsed key's casing (world=v2c) and add a copy-identity key fold (References #820)

Step 0b (#1709) found that 41 of the 109 D19 MQAR replies rehearse the key. All 41 capitalize it (`milestone_world_v2.rs` `capitalize()`, so `' l','u','k'` becomes `' L','u','k'`). The rehearsed key's token tuple therefore never occurs in context. This PR adds two fixes. Both are opt-in, and legacy outputs are unchanged.

### What changed
**(a) `world=v2c`** (`milestone_world_v2::Variant::V2c`)
- **Reply frame.** A rehearsing MQAR reply now reads **"So {k} is {v}."** instead of "{K} is {v}.".
- **Why this frame rather than skipping capitalization.** With the frame, the key is never the first word of the reply. It keeps the asserted casing and follows a space mid-sentence, exactly as in the assertion and the question. That holds under protocol 1 and protocol 2. Skipping capitalization only when the key is reply-initial would leave a lowercase sentence start. It would also make reply casing depend on the key's position, which is a new surface irregularity the model would have to learn.
- **Bare replies.** "It's {v}." and "That's {v}." do not state the key and are unchanged.
- **Same choices as v2.** The reply is chosen on v2's table by v2's template lengths, then written from the v2c table at the same index. A seed therefore makes the same choices in both worlds. The draws diverge only where the one-piece-longer reply changes whether an episode fits the context. After that point the two are independent draws of the same mix.
- **v2 is byte-identical.** `pick` is `items[rng.below(len)]`, and the rewrite consumes the RNG identically. The test `the_v2_seed_9101_draw_is_unchanged` pins a SHA-256 over the D19 session draw (development split, seed 9101, 300 conversations). `MWorld2::digest()` is unchanged. v2c has its own `digest_for(V2c)`.
- **m-world wiring.** `corpus`, `evaluate`, `session`, `cells`, `baselines`, `route`, `compiler`, `compiler-save` and `compile-corpus` all accept `world=v2c` and record `m-world-v2c` with its digest. `rejudge` reads the variant from the source report.

**(b) Copy-identity table** (`crates/uor-r4-training/src/copy_identity.rs`)
- **Fold rule.** This is a fixed table with one entry per vocabulary id, mapping each token id to a canonical id. A token whose bytes are ASCII letters, optionally after one leading space, folds onto the least id of its lowercased class. Every other token is its own class: digits, punctuation, spaces, non-ASCII and special tokens. The rule is `ascii-letters-case-and-one-leading-space/least-id`.
- **Build and save.** The table is built from the tokenizer vocabulary (`CopyIdentity::from_tokenizer`). It is saved as `uor-r4.copy-identity/1` JSON with its schema, rule, digest and tokenizer address. Loading checks the digest and the least-id/idempotence structure.
- **Scope.** `StackModel::set_pointer_key_fold(Some(table))` makes a **routed** pointer (`PrimeRoute`) admit sources by comparing canonical ids. The default is `None`, and the setting is not saved. The table is used only for key matching:
  - The pointer still copies the raw input token at the admitted source.
  - The generated branch never reads the table.
  - An unrouted (learned) pointer scores sources by query and key, not by identity, so it is unaffected.
  - The loss, `score_targets` and `next_scores` all read the same fold.
- **D11.** Serving it costs one table read per token (4096 u32 on #1017) plus integer equality on the values read. That means no multiplier, no float and no learned parameter. Neither integer engine serves a routed pointer yet (`stack_export` refuses one), so the table is not yet part of an exported artifact. `to_json` is the format a port would carry.
- **`rehearsal-probe` controls.** The probe gains `pointer_route=` and `key_fold=off|on` (both default off). The route is saved into the probe's checkpoint. The fold applies to the forced arms only, because the session's free run serves the saved checkpoint, which does not carry the table.
- **Coverage.** On #1017 the fold merges 497 of the 4096 ids. It cannot repair a key that BPE segments differently after capitalization (Step 0b: 15 of the 41); v2c is the fix for those keys.

**`scripts/pod/step4-corpus.sh`**
- It builds the v2c corpus and the v2 control with Step 1's exact M-world arguments: protocol 2, 40,000 conversations, seed 12, recall off, mqar .6 / copy .1 / relation .2 / other .1.
- It then builds Step 1's mix of 4 copies of the M-world train split plus `chat-v0-p2/train`, via `mix-chat-corpus`.
- The rebuilt v2 control's `tokens.u16` is compared byte for byte with Step 1's `$D/ft/mw-hi`, and the script prints MATCH or DIFFER.

### Tests (exact head, laptop, `CARGO_TARGET_DIR=~/.cache/uor-claude-target -j 2`)
- **Formatting.** `rustfmt --check` passes on every touched file. A package-wide `cargo fmt --check` fails only on `relation_compiler.rs`, which is already unformatted on `main` and is not touched here.
- **Warnings.** `cargo check -p uor-r4-training --tests --examples` adds no new warnings in touched code. The m-world `unreachable pattern` and the `geometric_stack` dead-code warnings are already on `main`.
- **Lib tests:** `cargo test -p uor-r4-training --lib -- copy_identity milestone_world_v2::tests::the_v2_seed_9101 milestone_world_v2::tests::v2c milestone_world_v2::tests::variants a_key_fold pointer` gives **26 passed, 0 failed**. These include:
  - `the_v2_seed_9101_draw_is_unchanged`
  - `v2c_rehearses_the_key_in_its_asserted_casing`, which checks that the v2c key equals the asserted and queried form, that paired episodes keep v2's choice, and that bare replies are identical
  - `variants_parse_and_name_their_worlds`
  - three `copy_identity` tests: the fold rule, a save round-trip with tamper refusal, and a byte-level BPE where " Luk" folds onto " luk"
  - `a_key_fold_admits_a_recased_key_and_is_inert_unless_routed`, which checks that an unrouted fold is bit-identical, that a routed fold puts copy mass on the value after the recased key and none on the key ids, that loss and scored rows match, and that clearing the fold restores raw bits. It also checks that short tables and pointerless models are refused.
  - every existing pointer, export and session test
- **Example tests:** `cargo test -p uor-r4-training --example rehearsal-probe` gives **6 passed**, including `the_step4_controls_default_off_and_refuse_bad_values`.

### Smoke (laptop, release, 3 threads, about 10 min in all)
**Corpus** (400 conversations, seed 12, Step 1's mix, about 1 s each)

| world | world_digest | tokens | response | Mqar/Relation/Copy/Other |
|---|---|---|---|---|
| v2 | `04ad3bb0…` | 56,316 | 17,440 | 228/95/40/37 |
| v2c | `e99a2093…` | 56,999 | 17,483 | 242/73/40/45 |

`mix-chat-corpus` accepted the v2c train split as input.

**`rehearsal-probe`** on 29M `chat-29m-B-lr5e-4` with Step 0c's arguments, but with `conversations=30` (13 MQAR rows):

| run | wall | session errors | rows vs reference |
|---|---|---|---|
| legacy (no new args) | 113 s | 0 | **13/13 rows identical** to Step 0c's sealed `29m-v2` rows |
| `key_fold=on` | 156 s | 0 | rows **identical** to legacy (the fold is inert on an unrouted pointer) |
| `pointer_route=ngram-ranked:3` | 109 s | 0 | `is` gold-in-top-5 rises from 3 to 6 of 13 |
| route + `key_fold=on` | 251 s | 0 | forced scores differ on 2 rows (mw2-0002, mw2-0007); free replies are unchanged |

The table had 4096 ids, merged 497, and digest `3fb352a3…`. Serving does not break under any setting. These are 13 rows, so the smoke shows only that the path works. It is not evidence of a model-quality effect.

### Pod run script: `scripts/pod/step4-corpus.sh`
```bash
cd /root/uor-r4 && git fetch origin codex/step4-key-casing && git checkout --detach cc100dbd8898ff2b145c227119d184de4a0023c5
REPO=/root/uor-r4 D=/root/data TOKENIZER=/root/data/tokenizer.json \
CHAT=/root/data/chat-v0-p2/train OUT_ROOT=/root/data/ft/step4 \
CARGO_TARGET_DIR=/root/target-step4 RAYON_NUM_THREADS=8 WORLDS="v2c v2" \
  bash scripts/pod/step4-corpus.sh > /root/runs/step4-corpus.log 2>&1
```
The script's M-world calls, per world, are:
```
m-world corpus world=$W out=$OUT_ROOT/mw-hi-$W tokenizer=$TOKENIZER protocol=2 conversations=40000 seed=12 recall=off mqar_share=0.6 copy_share=0.1 relation_share=0.2 other_share=0.1
mix-chat-corpus out=$OUT_ROOT/hi-$W tokenizer=$TOKENIZER inputs=T,T,T,T,$CHAT labels=mw-hi-$W-1,mw-hi-$W-2,mw-hi-$W-3,mw-hi-$W-4,chat-v0-p2
```
**Expected wall time:** about 15 min on CPU. That is about 8 min to build m-world and mix-chat-corpus, about 2 min per 40,000-conversation M-world corpus, and about 1 min per mix. The fine-tune A/B itself would be Step 1's `ft` with `train_*=$OUT_ROOT/hi-v2c/…` against `hi-v2`; it is not part of this PR.

### Caveats
- **v2 and v2c streams diverge.** They decouple at the first changed context fit, so per-kind episode counts differ by sampling noise: at 400 conversations, MQAR is 242 vs 228. At 40,000 the shares converge to the mix, and `corpus.json` records both.
- **Step 0c already found the learned pointer does not use the matching tuple.** Its `is_lc` arm, which forces the lowercase key, was no better than the capitalized form. So v2c and the fold remove a token-identity obstacle, but on that evidence they are not expected to fix recall by themselves.
- **The fold is not served.** It is not carried by checkpoints or by the integer export. The free run in `rehearsal-probe` does not use it.
- **No GPU run or fine-tune has been done.**

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5989523511 — 2026-10-05T06:46:52Z — Casey-allard

Adversarial review at cc100dbd8898ff2b145c227119d184de4a0023c5:

**Verdict: APPROVE.** Nothing blocks the merge. Nits are listed below.

**What I ran.** I ran everything at the exact head, in a `git archive` scratch copy, with `CARGO_TARGET_DIR=~/.cache/uor-claude-target -j 2 --offline`.
- **`cargo fmt --check`:** the only diffs are in `relation_compiler.rs`. That file is untouched by this PR, and I confirmed that `rustfmt --check` already fails on it at `origin/main`.
- **`cargo test -p uor-r4-training --lib -- copy_identity milestone_world_v2::tests::the_v2_seed_9101 milestone_world_v2::tests::v2c milestone_world_v2::tests::variants a_key_fold pointer`:** 26 passed, 0 failed, plus 1 in the second filter group. All of the following pass:
  - `the_v2_seed_9101_draw_is_unchanged`
  - `v2c_rehearses_the_key_in_its_asserted_casing`
  - `a_key_fold_admits_a_recased_key_and_is_inert_unless_routed`
  - the three `copy_identity` tests
- **`cargo test --example rehearsal-probe`:** 6 passed, including `the_step4_controls_default_off_and_refuse_bad_values`.
  - The first attempt failed to compile with "unresolved import copy_identity". The cause was a stale fingerprint in the shared cache: `git archive` sets file mtimes to the commit time. After `touch`, it passes. This is a reviewer-environment pitfall, not a PR defect.
- **`cargo check --example m-world --bin mix-chat-corpus`:** clean.
- **`bash -n scripts/pod/step4-corpus.sh`:** OK.

**Correctness checks**
- **v2 byte identity.** `pick` is `&items[rng.below(len)]`. The new index-based `min_by_key` consumes the RNG identically and keeps the first minimum, as before; the tie between the duplicate `{k} is {v}.` entries gives identical text. The pinned digest test passes. `digest()` is unchanged.
- **Fold causality.** `keys` reaches `pointer_attention` only in the routed branch, through the loss, `score_targets` and `next_scores`. That covers every non-test caller. Copy mass still reads `self.ids`, so the pointer emits the raw token. An unrouted pointer ignores `ids`, so the fold is inert there by construction, and the test checks that bit for bit. `pointer_mixture_loss` and the existing tests pass `keys: None`.
- **Errors.** No new `unwrap`, `expect` or `panic` on library paths; `expect` appears only in tests. The table is validated on load (least id, idempotence, digest), and short tables and pointerless models are refused.
- **D11.** The claim is scoped correctly: one table read plus integer equality per token. The PR does not claim the fold is served, and it states that `stack_export` refuses a routed pointer.
- **Script.** It matches Step 1's `run-step1.sh` exactly: `m-world corpus world=v2 … protocol=2 conversations=40000 seed=12 recall=off mqar .6/copy .1/relation .2/other .1`, then the mix of 4 copies with `chat-v0-p2/train` and the same labels.
  - The fields the Python summary reads exist in `corpus.json`: `composition.m_world.{tokens,response_tokens,episodes_per_kind[*].episodes}`.
  - Every output root is claimed fresh.

**Nits (non-blocking)**
1. **`fit_route` provenance.** `fit_route` (`evaluate_v2 recall=route`) still builds `MWorld2::new(count, mix)` with no variant, while the report records `m-world-v2c` and its digest. The route is fit on user turns, so the effect is probably nil, but the report should say "route fit on v2 draws" or pass the variant.
2. **The A/B is not paired.** The streams decouple early: at 400 conversations, Relation is 95 vs 73 and MQAR 228 vs 242. The later v2 vs v2c A/B should therefore:
   - freeze its decision rule before it runs: D19 MQAR with `log_recall=off`, at least the two `data_seed`s as in Step 1, and a minimum effect against seed spread;
   - report per-kind exposure from `corpus.json`.

   The PR states this correctly; it just needs to go on the work card before compute.
3. **DIFFER is only printed.** When the v2 control DIFFERS from Step 1's `mw-hi`, the script prints and continues. That is fine, because the A/B uses the rebuilt `hi-v2`. Comparisons with the Step 1 arms would then be invalid, though, so make DIFFER loud or set an exit status. Note also that Step 1's binary was at `dec3c096`, and the pinned digest was taken at `e4922ec5`.
4. **No terminal marker on failure.** With `set -e`, a failure leaves no marker, and only success prints `STEP4 CORPUS DONE`. Add `trap 'echo "STEP4 CORPUS FAILED $?"' ERR` so a completion watcher sees every terminal state.
5. **Untested comment in `digest_for`.** The comment that "So" cannot collide with a generated word because it is one syllable is asserted, not tested. Either add a reserved-word assertion or drop the claim.
6. **Fold scope.** The fold is reachable only from `rehearsal-probe`'s forced arms. It is absent from `dialogue-train`, the session and checkpoints. This is disclosed. Any later claim that the fold helps recall needs it wired into a served or trained path first.


### comment 5989528427 — 2026-10-05T06:47:17Z — Casey-allard

Lab-lead exact-head check at cc100dbd8898ff2b145c227119d184de4a0023c5: the adversarial review approved with nits (https://github.com/UOR-Foundation/uor-r4/pull/1723#issuecomment-5989523511). **Verified at this head:**
- lib tests 26/26 plus 1, and rehearsal-probe tests 6/6;
- `world=v2` is byte-identical, pinned by a digest test;
- the m-world and mix-chat-corpus check is clean.

The script nits (failure marker and exit status, the fit_route variant, the v2/v2c pairing rule) go into a follow-up before any v2c run. Queuing.


---
_Comment/review ids in this file: 5989523511, 5989528427_

