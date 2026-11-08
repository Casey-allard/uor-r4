# ORDER: one-model-end-to-end, claude-324

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `ec12fb855`.

## Goal

**Build a current-state TRAIN corpus whose replies are in the form the `m2-fresh256` panel accepts, with values kept out of every panel, and mix it into claude-322's × 6 capped mix.** This unit is a laptop build only. The GPU run of the new mix is the next unit, under its own prep sheet, FIT and pre-run review.

Why, from claude-322's served replies (`local/claude/claude-322/score/s{1,2}.json`):
- **The panel asks** "What is my current job?" or "Where do I currently live?", after the user states or updates the value. It accepts only the bare value and a period, for example `basalt loggia.` (`docs/labs/m2-fresh-heldout-panel/instrument/src/lib.rs:1134`).
- **The trained trunk answers in another relation's sentence.** At seed 1, 71 of its 109 EOS-stopped replies to job and home questions begin "Your friend's name is …", and 28 begin "Got it, your friend is …". At seed 2, "Got it, your friend is …" leads, with 36.
- **So two gaps stack before recall matters:** the mix never teaches this task (job and home with an open value stated in the chat), and it never teaches the bare reply form. claude-323's `unwrap.jq` finds 14 and 2 stopped replies that would be complete with the wrapper stripped.
- **No generator in the repository writes this.**
  - `milestone_world.rs:1299` has job and home, but with small closed value pools ("doctor", "Paris") and sentence answers ("Your job is {v}.").
  - `synthetic-memory-corpus` also answers in sentences ("Your dog is a {v}.").
  - The generator of `mw-balp/train` is not recorded.

Work:
1. **A generator:** a standalone lab crate under `docs/labs/one-model-end-to-end/`, detached from the workspace like the panel's instrument. It writes a `uor-r4-chat-corpus/v1` store (`tokens.u16` with its header, `response_mask.u8`, `manifest.json`) in the document layout of `milestone_world_v2.rs:1291`.
   - **Episodes:** a user states a job or a home, may update it one or more times with other turns between, then asks for the current one. The assistant's last reply is `{v}.` and EOS, with the CURRENT value.
   - **The phrasings:** the panel's `supported_wire` templates and queries (lib.rs:585; the panel says these are supported, so its freshness lies in its values), plus `milestone_world.rs`'s train phrasings for job and home.
   - **Values:** open, multi-word values of 1 to 8 words (like the panel's 2, 4 and 8) from a word list in the crate.
2. **The checks, as tests in the crate:**
   - no value word is one of the panel's `FRESH256_*` content words or `HISTORICAL_EXPOSED_LITERALS`, or any word of the sealed dev512 and opened-fresh128 inputs;
   - no generated prompt is byte-equal to any `m2-fresh256` prompt;
   - every reply decodes to `{v}.` for the episode's last stated value;
   - the store round-trips through `mix-chat-corpus` with claude-322's mix.
3. **A census:** the episode, token and window counts, the share of the mix, and the mix's shas. Seal the corpus and the mix to the store under `results/claude/one-model-end-to-end-currentstate-2026-11-08/`.

**What a run of this mix could and could not show:** a lift in `complete` would show the trunk binds a fresh value, stated in the chat, into the bare reply for this task family. It would not show the same for other relations, or for a reply form that was never trained. The REPORT says so.

Claim check:
- **(a)** This is Track A of "Both in parallel" (#2028, 6715318106), under "pipeline first" (#2028, 6709947526). It unblocks the reply-form lever that claude-323's sheet names (rows 2 and 3), and the panel bar (104 complete, 52 pairs) needs that lever: at 0 bare-form replies, no amount of recall moves `complete`.
- **(b)** It changes TRAIN, the data the served model learns from. No bar moves, and the panel stays held out on its values.

## Cancel probe (run first)

1. **No open PR builds a bare-reply corpus.** The open PRs are #3863 (claude-323, mixed copy keys) and #3860 (deepseek-40, a point-key opening probe on DeepSeek's trunk; panel labels are evaluation-only there).
2. **The inputs are present:** the panel's lab crate and its literal lists, the pinned tokenizer, `mix-chat-corpus`, and claude-322's mix in the store under `results/claude/one-model-end-to-end-b64l-d3mwcap6-2026-11-08/train/`.
3. **Compute:** no GPU in this unit. claude-323's two pods run apart from it.
4. **Disk:** 14 GiB free on the laptop.

**Result, 8 Nov 16:45Z:** 1 to 4 hold as written.

## Files in scope

- `docs/labs/one-model-end-to-end/**`
- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. The generator's crate is committed with its tests green, including the disjointness and form checks.
2. The corpus and the new mix are sealed to the store, with their shas and the census in the REPORT.
3. The REPORT gives the result, negatives first, with one notebook row, and names the next unit's sheet.
