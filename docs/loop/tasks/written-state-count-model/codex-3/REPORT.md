# REPORT: written-state-count-model, codex-3

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

The fitted geometric selector chose position 4 on every one of the 16 fixed role prompts, on both seeds. The first wrong token stayed at answer offset zero. With copy disabled, token 1155 (leading-space G) won every opening. With learned copying enabled, the first token became the name of whoever appeared first in the facts, Ann (2241) or Ben (546), eight each. All three arms scored 0/16 replies per seed against the registered immediate-EOS rule. The bounded selector also missed its separate held-out matched-pair criterion. [Per-case old/new outputs and first errors](writer/first-wrong.tsv)

## What changed

An opt-in Rust serving operator and an ahead-of-time CUDA writer consume four native u32 state words: active flag, absolute source start, next cursor and exclusive original prompt fence. A copied step emits the original prompt's exact ID, increments the cursor once and returns that ID for feedback. It preserves S0 identity through geometric collisions. A stop token, the original fence or the eight-token limit closes the cursor; the same step emits the ordinary generator's prediction. A closed cursor stays closed. The actual emitted word, copied flag, exact ID and release reason accompany the next four state words in the receipt. The existing D11 artifacts and their default generator retain their original behavior. No new action gate was trained; this diagnostic starts the copy action once at the answer boundary.

The selector fits two offsets of an integer E8-root comparison by maximizing exact TRAIN occurrence hits on Wick. CUDA makes a source-position prediction from prefix-only input. The rollout reads the frozen state and independently recomputes that position in serving Rust. The writer then receives the device's position. Oracle supplies only a start occurrence. Neither arm uses a gold length, pointer sequence or teacher-forced output. During rollout, the ordinary generated ID is recorded beside the ID actually emitted. The prior Q31 read checkpoints used a different input representation, so the two D11 seeds each received a new bounded selector fit. [Fit design and input hashes](SELECTOR-SHEET.md) · [Writer design](SHEET.md)

## Measurements

Numbers are per released artifact unless noted; the two seeds agree on every first divergence in these comparisons. Answer offsets start at zero. Whole value excludes EOS; strict reply requires immediate EOS.

| Arm on 16 role prompts | Correct selected start | Correct opening | Whole value | First wrong offset | Strict reply |
|---|---:|---:|---:|---|---:|
| Copy disabled | — | 0 | 0 | 0 on all 16 | 0 |
| Learned source + exact writer | 0 | 0 | 0 | 0 on all 16 | 0 |
| Oracle source + exact writer | 16 | 16 | 16 | 1 on eight one-token values; 2 on eight two-token values | 0 |

**One full trace.** For “Ann chose red fox. Ben chose red kite. What did Ben choose? Ignore Ann.”, the target is [677,1999,1] (red, kite, EOS). Oracle sets source/cursor 13 with original fence 33. It emits ID 677 from 13 and ID 1999 from 14, even though the ordinary generator at the second step proposed 16. At 15 the source period releases the cursor. The native generator emits 3 (exclamation), then 1 (EOS). The first error is therefore at offset 2, expected 1 and emitted 3. In #4084 the geometric writer receiving 1999 as its own projected input had emitted its equal-vector alias 878. Directly carrying the S0 ID resolves that collision on all four kite requests in each seed. Across the oracle rows the first free token is period 12 times and exclamation four times per seed. Every oracle case stopped on its following step. [Seed 1 complete step trace](writer/seed1.json) · [Seed 2 complete step trace](writer/seed2.json)

**Why the learned source fails.** Both seeds uniquely selected d=2, q=13 (48/128 training hits). On every fixed role prompt the learned query offset lands on token 3241 (leading-space choose). The nearest eligible historical match after deterministic tie-breaking is token 2728 (ser within the User label), at position 2. Adding d chooses position 4, the first name. On “Ann chose red. Ben chose blue. What did Ann choose? Ignore Ben.” this yields [2241,641,1935,677,16,1], or “ Ann chose red.” plus EOS. Reversing the question to request Ben leaves that first source and ID unchanged. The copy cursor accurately reproduces the wrong addressed span. The selector sees the same query token despite the requested entity changing.

**Independent selector measurements.** TRAIN used 128 balanced constructions. The independent held-out set used 64 other prompts (32 matched question pairs), with different name/value inventories and two different wordings. All 208 TRAIN, held-out and old role-panel inputs were checked distinct. Grading uses occurrence equality. At both seeds:

| Frozen source policy | Asked source / 64 | Other person's source / 64 | Both members correct / 32 |
|---|---:|---:|---:|
| Trained on asked occurrence | 8 | 8 | 0 |
| Unfitted (d=1, q=0) | 0 | 0 | 0 |
| Fitted to the other occurrence | 0 | 64 | 0 |

The trained rule improves individual hits over both controls but misses the preregistered paired gain. It does not transfer reliable role selection on this grammar. The swapped-label rule reliably found the other role on these wordings. The observed joint failure is specific to this small fixed-offset policy. Only the source selector was evaluated on the 64 additional cases; full replies were evaluated on the historical panel. These estimates describe the selected constructions, with both seeds evaluated on the same prompts.

## Checks run at the head

* The finite CUDA writer fixture matched Rust on all eight action/state words and generator prediction over 26 steps, seven trajectories. It included a 2,000-way same-cell/same-vector tie, inactive state, all four stop IDs, cap, fence and non-restart. An initial synthetic fixture had invalid block count and stopped before evaluation. Its record is preserved. Three Rust tests and the C++ shared-kernel host test passed. The finite selector compared 576 CUDA choices against the native integer implementation, with tie, displacement, empty-domain and swapped-label controls. [Writer checks](finite-writer/pass/stdout.txt) · [Non-author FIT](FIT.md) · [Pre-run decisions](PRE-RUN.md)
* The two panel jobs compared **339 and 326** device output transitions with integer serving, step by step: cursor/action, emitted and source IDs, ordinary prediction, read vector, admitted cells and top eight. Every disabled output sequence equaled its pinned #4084 native sequence. Independent fresh processes reproduced each full JSON byte for byte at both seeds. Wick RTX 4090 sm_89, primary wall 0.94/0.95 s, peak host RSS 1,033,244/1,016,780 KiB. These include file loading and parity checks. End-to-end model throughput was outside the measurement.
* The canonical primary receipts hash to a25335836b19cb8106838262a3f9affcf11431671ac61af3e3ea2d150a2f97b2 and ff5152a96562b74aa6253d33f58c31ff14e333467c48595c836f873b44aed7ad. The writer binary was e9e0a53a1b9234fa50d35615cd8c40fbb66c9e4b2402ef82d124d113f5dc5252. The committed JSON preserves every field with whitespace compacted.
* Build the standalone scorer on Wick with `UOR_NVCC_ARCH=compute_89 cargo build --locked --release --target x86_64-unknown-linux-gnu --features cuda --manifest-path docs/labs/m1-d11-artifact-scorer/instrument/Cargo.toml --bin d11-scorer`. With the original seed-specific D11, blinded fitted state and predictions, run `d11-scorer copy-rollout SEED.d11 ../codex-1/panel.json trained.json role-trained.json ../codex-1/role/seedN.json NEW.json`. See the selector sheet for fit data and controls.

The [selector results and per-prefix controls](SELECTOR-RESULT.md) include the final-source replay. The [private Wick evidence bundle](https://huggingface.co/datasets/caseyallard/uor-r4-store/blob/ebf2528b446952029e00ea67de3e5c7216950d5c/results/codex/geometric-copy-20261118/geometric-copy-wick.tar.gz) contains both fresh-process panel runs per seed, both selector builds’ fit/prediction/grade outputs, frozen inputs, logs and finite tests. Its SHA-256 is 070e0d1649dc5a3b379bc55692217fad621f445256cb5e49185c68703d7e944e. The independently downloaded bundle matched byte for byte.

## Skills used

Native-space-cycle supplied the state schema, integer replay and shared serving/device requirements.

## Still open

The forced start gate and punctuation delimiters were fixed diagnostic choices. Learning when to enter, release or suppress copying remains open. The next test suggested by the failure is to bind the query anchor to the requested entity from content, score contrastive same-facts/opposite-question sources on held-out phrasings and retain the exact writer. Keep immediate-EOS calibration separate so boundary punctuation cannot obscure a source failure. P3, the complete trained and served model, and M2 retain their registered bars.

## Plain summary

When the model is given the right place to start, it now copies every word exactly. Even the two tokens that share coordinates stay distinct. Its first mistake moves to punctuation after the complete answer. When the model has to choose the place itself, it copies the first name in the prompt, and its very first word is still wrong. The learned rule looked at “choose” in both versions of the question, so switching the requested person did not change its choice.
