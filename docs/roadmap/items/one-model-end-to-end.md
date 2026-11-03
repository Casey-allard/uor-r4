---
id: one-model-end-to-end
title: Score one model end to end on prompts it has not seen, with tokens per second
milestone: research
area: evaluation
status: ready
open_to: lab
needs_gpu: true
depends_on: [turn-memory-reply]
notebook: docs/research/notebook/float-training.md
scope: [docs/labs/one-model-end-to-end/**, docs/harness/prep/one-model-end-to-end*, docs/loop/tasks/one-model-end-to-end/**, docs/research/notebook/float-training.md, docs/roadmap/items/one-model-end-to-end.md, crates/uor-r4-float-train/src/hamilton_sheet.rs, crates/uor-r4-float-train/src/geometric_stack_native.rs, crates/uor-r4-float-train/src/in_space_long.rs, crates/uor-r4-float-train/src/bin/train-in-space.rs, crates/uor-r4-float-train/src/flock_read_memory.rs, crates/uor-r4-float-train/src/flock_read_hamming.rs, crates/uor-r4-training/examples/m-world.rs, crates/uor-r4-float-train/src/fbs_export.rs, crates/uor-r4-integer/src/fbs_snapshot.rs, crates/uor-r4-integer/src/stack/format.rs, crates/uor-r4-integer/src/stack/hamming.rs, crates/uor-r4-integer/src/stack/synthetic.rs, crates/uor-r4-integer/src/stack/tests.rs, crates/uor-r4-training/src/bin/m-world-recall-corpus.rs, crates/uor-r4-training/src/bin/mix-chat-corpus.rs, crates/uor-r4-training/Cargo.toml]
---

## Why it matters

This item is the last part of the open owner direction's order: "`turn-memory-reply` first, then the integer serving of the parts, R7, and one model end to end" (`PLAN.md:86`). Casey added on 29 Oct (about 20:05Z) that it be done "in a smart way that considers the whole project and outside research we can draw from to see what exactly we are probably dealing with instead of making quick (and wrong) decisions".

[The assessment](../../loop/tasks/infra-loop-harness/claude-201/ASSESSMENT.md) is that reading. A fresh decision review passed it in its third round (owner log on #2028, `6507593846`). Its findings on 29 Oct, before the held-out-phrasing panels and generation runs:
- **The replies are templated; DEV and Fresh V2 do not hold out prompt templates.**
  - A reply-only control, keyed on the reply's own last tokens, already gets 8,089 of DEV's later rows. The chosen key gets 8,822 / 9,655.
  - The sealed Fresh V2 panel comes from the same generator. The bigram table T2 scores 6,582 / 14,479 there, against 6,300 / 14,287 on DEV.
- **The memory gives a real gain inside the templates.** Openings are 598.7 / 598.3 with it, against 266.3 / 281.0 without, under a ceiling of 607.
- **The trunk now leans on it.** Without its tables, turn-memory-reply's (r) falls to 4,101 / 3,807, below the empty-table control (r0)'s 10,079 / 11,171.
- **Generated text had not yet been scored at that assessment.** By 30 Oct, claude-216 and claude-217 had scored free generation on new-template panels. The trained cut returned "reduces, content lost" and did not replace `r-drop`; the cycle step ends every registered loop but every content decision is underpowered, so greedy remains the decode.

So before the model the project ships is trained, the scoring must be able to tell recall of a template from reading the prompt.

## Current public-artifact baseline

On 2 November, the two published dual-c131 integer artifacts were run through
the product `uor-chat` path under their trained protocol 2 on a fixed
eight-entry public conversational panel, at seeds 1 and 2 with an exact seed-1
repeat. The integration path passed: all 24 processes exited 0, all 33 replies
stopped by EOS or the 16-token cap, every reply text decoded from its recorded
ids, and the repeat matched all text, ids, stops and history depths. Both fixed
multi-turn memory anchors failed on both seeds. Peak RSS was 11,592 KiB and
observed reply rates were 144.47–350.35 ids/s on the local workstation.

This is an ungraded product baseline, not a language or mechanism result. It
establishes that the public artifact is connected and deterministic and is not
ready for a useful-chat demonstration; it does not change this item's status or
replace the served-training comparison above. Full transcripts and pins:
[public-chat post-smoke baseline](../../labs/one-model-end-to-end/public-chat-post-smoke-2026-11-02/README.md).

The CPU-only `m2-evidence` lane now executes the public trained 19-table M1
artifact through the independent `CpuDiscreteTrunkReader`. On its existing
616-target public panel it checked all 19,712 Q12 query coordinates and all
complete admitted-cell, top-8 and token decisions with zero mismatches. This
closes the earlier full-CPU-trunk query gap in the evidence instrument; it is
not a training, complete-reply, fresh-phrasing or GPU-qualification result.
See [deepseek-11](../../loop/tasks/one-model-end-to-end/deepseek-11/REPORT.md).

The same instrument now supports bounded greedy complete-reply generation.
On the public M1 panel, 51 of 58 prompts reached EOS within 16 new tokens;
three reached the token limit and four had no remaining context. None matched
its reference reply, only 11 distinct non-empty texts appeared across 58
prompts, and four context-limit rows were empty. Two replies repeated a 4-gram.
Generated 4-/8-gram novelty against the generated control was 57/411 and
195/195. This is a useful complete-reply instrument and a public-model
readiness negative, not a fresh-phrasing or chat verdict. See
[deepseek-12](../../loop/tasks/one-model-end-to-end/deepseek-12/REPORT.md).

The complete-reply grader was then rerun on an unused-seed 218-prompt
held-out-phrasing panel. None of its prompt templates occur in the same-seed
generated control at K = 256 or 1,024, and both the panel and decode reproduce
byte for byte. The public M1 model still produced only 22 distinct non-empty
texts across 199 non-empty replies and matched no reference reply. Every
generated 8-gram was new, while only 201 of 1,517 generated 4-grams were new.
Teacher-forced and CPU trunk parity remained exact. This confirms the
public-model readiness negative under a genuinely new-template draw; it is not
semantic-correctness, fresh-reply-wording or GPU evidence. See
[deepseek-13](../../loop/tasks/one-model-end-to-end/deepseek-13/REPORT.md).

The same instrument now has a count-table arm. It builds the corpus-wide most
common successor and each previous token's most common successor from adjacent
tokens inside the supplied reference documents, then runs the identical
218-prompt, 16-token, context-limited and EOS-aware reply pass. On the public
seed-1 M1 artifact and the seed-2201 generated control, the model completed 184
replies with 22 distinct non-empty texts, seven repeated-4gram replies and five
replacement-character replies; the count table completed 196 with four distinct
texts and no repeated 4-gram or replacement-character reply. Neither matched an
exact reference, and the two paths differed on 199 of 218 first tokens. The
count table is therefore a matched naive control, while the public model's
extra text forms are not evidence of correctness. Memory-off and no-read remain
unavailable because this artifact carries neither mechanism. See
[deepseek-14](../../loop/tasks/one-model-end-to-end/deepseek-14/REPORT.md).

## What is missing

The D11 diagnostic/training scopes are now panic-safe on the host: a drop
guard restores gate, snap and training-bridge state after a caught panic, so a
failed diagnostic cannot contaminate the next forward on that thread. See
[deepseek-15](../../loop/tasks/one-model-end-to-end/deepseek-15/REPORT.md).

- **Served-surrogate training is repaired; it now passes the operational
  gate, near the ceiling.** Q16 residuals and dynamic i16 recurrence/MLP
  activations run in the real geometric hidden path under
  `--train-as-served 1`, with D11 snaps, a Q12 scored query and D11 checkpoint
  evaluation. [deepseek-5](../../loop/tasks/one-model-end-to-end/deepseek-5/REPORT.md)'s
  four-run comparison took 6.83–6.96× plain. [claude-262](../../loop/tasks/one-model-end-to-end/claude-262/REPORT.md)
  found two faults: the activation requantizer had dropped the step's
  exponent `PRODUCT_EXP = −32`, and the D11 snap's 4-axis score `sum` launched
  one GPU block per score. It repaired both, each pinned by a CUDA test, and
  reran the same sheet. Served training then took 1.79–1.84× plain, and both
  checkpoints survived export rounding unchanged. The quality smoke stayed
  inconclusive, with served above plain on the bar's evaluator top-1 (53 / 67
  against 40 / 43, band 24) and on DEV nearest (305 / 357 against 239 / 248). The step's 1.85× margin to the 2× ceiling and served-run
  openings are still open before a longer run.
- **A model trained as served does not yet serve as trained.** [claude-263](../../loop/tasks/one-model-end-to-end/claude-263/REPORT.md)
  exported claude-262's served step-256 checkpoints snapped and served them in
  integers. Their first tokens differ from the model's own float first tokens
  on 112 and 137 of DEV's 310 prompts, against I-3's bar of 8. claude-261's
  2,048-step checkpoints, a different configuration, missed 295–305 before
  the repair. The float side's turn memory
  is not needed to explain the gap: float first tokens with and without
  memory differ on 3 and 0 prompts. The integer memory path is unchecked.
  [claude-264](../../loop/tasks/one-model-end-to-end/claude-264/REPORT.md)
  made the float served forward round every FBS projection to 2^-16, as the
  integer session does. The misses fell only to 100 and 131. Some matches broke
  as well as others being made. That fits other unmatched roundings,
  an inexact match of this one, or run-to-run variation in the scorer, and the
  unit does not separate them.
  [claude-265](../../loop/tasks/one-model-end-to-end/claude-265/REPORT.md)
  trained and served without the icosian snap: the gates pass through and
  the transition is unit-normalized, in training and in the raw export alike.
  Retrained at two seeds, the misses fell to 32 and 14 of 310, and the float
  scorer gave identical output twice on one host. So the snap was the main
  part of the gap, and the rest is a real, smaller one; whole replies still
  differ on 287 and 240. The remaining sites (the Hamming reads, the turn
  memory, the writer's Q12 query, the convolution, the embedding) come next,
  read per layer. The full-length served run's sheet is FIT and its pre-run
  review passed; it needs re-pinning to the unsnapped contract.
- **A census of the data:**
  - the TRAIN size correction: [claude-202's census](../../loop/tasks/one-model-end-to-end/claude-202/REPORT.md) measured 7,512,797 tokens and 2,425,925 eligible targets in the pinned TRAIN file. The older sheet's 26,214,400 / 10,975,281 figures for the same `tokens.u16` SHA match neither its header nor its payload and are not the corpus count;
  - a TRAIN reply index;
  - which panel, if any, holds templates TRAIN never shows.
- **The check that could overturn the diagnosis:** whether (r) falls below (r0) where no key matches.
- **Held-out-phrasing evidence now exists, with a sharp limit.** M1's p-f panel passed bar (f): 7,850 / 7,872 of 15,953 integer top-1 rows against T2's 7,375 (paired band 81), at two seeds. Its prompt-template census is 0.000 at K = 1,024; 0.625 of its reference replies are verbatim TRAIN replies. That reference overlap is not a generated-copy rate or useful-chat qualification. On p-s, claude-217's teacher-forced reply openings are only 17.4%, 13.7% and 18.6% at three seeds, against 80.8%, 80.2% and 79.9% on later tokens; exact turn memory's exact-token key finds little on new phrasing. Claude-218's offline teacher-forced key probe selected the cheapest passing sign key, while recording the point key's larger gain and 1 KB-against-32-byte size tradeoff. [Claude-219](../../loop/tasks/one-model-end-to-end/claude-219/REPORT.md) then tested the keys in free generation on fresh p-v: the point key improves templates by +13/+8/+12 and exact replies by +9/+8/+12 at three seeds, while the sign key is inconclusive at this panel size. The point key reaches only 12–18% of the forced-opening lift; its openings are right on 27–31% of cases. [Claude-220](../../loop/tasks/one-model-end-to-end/claude-220/REPORT.md) then served the point key inside one float model on fresh p-w: templates improve by +41/+40/+42 at three seeds with exact claude-219 parity, while recalling TRAIN replies rather than generalizing. [Claude-222](../../loop/tasks/point-key-integer/claude-222/REPORT.md) has built the integer point table, search and `/8` record, with 380 library tests and a full serving audit. [Claude-224](../../loop/tasks/point-key-integer/claude-224/REPORT.md) has built the panel decode and grade. [Claude-227](../../loop/tasks/integer-retention-rates/claude-227/REPORT.md) passed retention GPU parity within the 3% bar; [Claude-228](../../loop/tasks/point-key-integer/claude-228/REPORT.md) built the run's reference/read-back/BOS checks; [Claude-229](../../loop/tasks/point-key-integer/claude-229/REPORT.md) found an instrument fault: the integer session uses the native writer instead of the trained whole-table nearest readout, so no bar result or throughput is reported. [Claude-231](../../loop/tasks/integer-nearest-readout/claude-231/REPORT.md) built the nearest readout and export on native and browser paths; [claude-232](../../loop/tasks/integer-nearest-readout/claude-232/REPORT.md) then missed greedy parity in every cell, so the mismatch must be localized before the p-x rerun, with the client-sized integer table and search still to prove. Useful chat remains unproven. DEV and Fresh V2 remain template-sharing controls, not untouched new-template generation panels.
  - M-world v2 already splits phrasings: `dev_phrasing × dev_value` is its development cell (`crates/uor-r4-training/src/milestone_world_v2.rs`, "Cross cells").
    - **The split covers user-turn phrasings and fillers only.** The assistant's reply templates are shared across splits. They include `MQAR_ACKS`, `MQAR_REPLIES`, `UPDATE_ACKS` and v1's closers and absent replies.
    - So the panel tests held-out *prompts* with new values, not held-out reply wording. Its census's masked reply-template share is expected to stay high.
    - **Two risks the panel's census must count:** a TRAIN paraphrase near-equal to a dev template (the generator refuses exact equality only), and chat-v0 turns that coincide with dev phrasings. It also records the `paraphrases=` setting TRAIN was built with.
  - But `m-world corpus` writes TRAIN conversations only. The writer is a `cell=` option on `corpus world=v2` in `crates/uor-r4-training/examples/m-world.rs`, added to this item's scope by claude-203.
  - The panel it writes is counted by `train-in-space --census` before it is used.
- **A scoring protocol, fixed before any training run, with:**
  - strata: keyed, unkeyed, new-template;
  - free-generation measures;
  - decode rules: greedy against a Flock rank rule;
  - integer-artifact measures.

## Done when

1. **The census (laptop, counts only, CHARTER §4.5).** A standalone Rust instrument under `docs/labs/one-model-end-to-end/instrument/` must:
   - resolve the TRAIN size conflict, and build the TRAIN reply index (exact replies and reply 8- and 13-grams);
   - report for DEV, Fresh V2 and `m2-fresh256`: the share of reply n-grams in TRAIN replies, the share of prompt templates seen in TRAIN with values masked, and the 8-token key share;
   - read only the manifest of the K16 fresh slice, and leave the slice unopened.

   It also says which panel, if any, holds new templates.
2. **The overturn check (GPU, under a prep sheet with a fresh FIT and a pre-run review).** Score turn-memory-reply claude-194's (r), (r0), and (r) with its memory off.
   - **Pinned in the sheet:** the checkpoints by store path, `SHA256SUMS` and step; the binary's sha; a lease and cost inside the caps.
   - **The pod** also seals a float reference for the read's parity: claude-194's (r), with its memory off, at both seeds. That is every DEV row's writer and nearest pick, and its 64 greedy samples. The parity itself, kept sets and top-1, is `integer-hamming-read`'s step 5, on the checkpoint that item's prep sheet names; the first greedy divergence is done-when 3's integer-artifact measure (claude-205's decision review, on #3407).
   - **Rows:** DEV's 3,406 later rows the reply table does not name, and its 32 unkeyed openings, judged pooled (3,438).
   - **Band, fixed in advance:** `max(2·√(b+c), the two-seed spread)`, where b and c are the paired discordant rows.
   - **The cells:**
     - (r) ahead at both seeds, beyond the band: the memory recipe stays the training default;
     - (r) behind at both seeds, beyond the band: a memory-dropout arm joins the training run;
     - mixed (the seeds disagree, or either seed is inside the band): both arms train.
   - **Descriptive only:** Fresh V2, and 64 greedy samples per arm scored against the census index.
3. **The protocol.** A prep sheet, with a fresh FIT, written before the training run of the model to ship. It holds:
   - **Strata:**
     - keyed rows;
     - unkeyed rows;
     - new-template rows, from a panel no decision has touched. Fresh V2 is never one.
   - **Arms:** (r), memory-off, (r0), tables only, and a longest-suffix table over TRAIN.
   - **Scores:**
     - teacher-forced top-1, per stratum;
     - free generation: the share of replies that are verbatim TRAIN replies, new n-grams for n = 4–8, distinct-1 and distinct-2, the repeated-n-gram rate, and the table hit rate under generation against teacher forcing;
     - on the integer artifact: kept-set agreement, first divergence, and tok/s against context.
   - **Decode rules:** greedy, and a Flock rank rule with the harmonic weights of `stack/flock.rs`.
   - **The verdict rule, written in advance:**
     - "memorized": the gains appear only on keyed rows, and generated replies are mostly TRAIN replies;
     - "generalizes": the model beats (r0) and the longest-suffix table on unkeyed and new-template rows, with novelty near DEV's own replies.

   The training run itself (ASSESSMENT §6, step 5) adds:
   - the transport controls: frozen and identity SpiralCore routes, under the fixed backward;
   - the read-by-meaning arms: the Flock read, and the type spin.
4. **Lead with tokens per second** (decode against context). One notebook row per result; each REPORT negatives first.

## How to start

Read [the assessment](../../loop/tasks/infra-loop-harness/claude-201/ASSESSMENT.md) sections 1, 5 and 6. Then read:
- `docs/research/notebook/float-training.md` rows 14–15;
- `docs/loop/tasks/turn-memory-reply/claude-194/REPORT.md`;
- the fresh-transfer instrument in `docs/labs/m1-d11-artifact-scorer/instrument/` (`fresh_rows_from_corpus`, `build_train_count_tables`), to reuse rather than rebuild.

## Rules that apply

- **Deterministic scores only:** count tables are lawful controls, and there are no neural or transformer baselines (CHARTER §3.10, §4.4).
- **No bits per byte without a lawful likelihood** (§4.8). A rank code is a diagnostic only.
- **Model scoring on the GPU; counting the data on the laptop** (§3.5).
- **The M1 bar question is answered:** the owner adopted D82's held-out-phrasing bar (f), now in [CHARTER §4.8](../../../CHARTER.md#4-evidence). The saved model passed (a)–(f), and [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) closed on 30 Oct. This does not complete this item's protocol or integer-serving requirements.

## Sources

- [The assessment](../../loop/tasks/infra-loop-harness/claude-201/ASSESSMENT.md), with its outside literature: kNN-LM in generation arXiv:2305.14625; RETRO arXiv:2112.04426; Engram arXiv:2601.07372; RAVEN arXiv:2111.09509; degeneration arXiv:1904.09751.
- [float-training notebook](../../research/notebook/float-training.md), rows 14–15; [turn-memory-reply claude-194](../../loop/tasks/turn-memory-reply/claude-194/REPORT.md); [the Fresh V2 sheet](../../harness/prep/m1-fresh-panel-transfer-2026-10-26.md).
