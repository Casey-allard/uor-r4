# ORDER: one-model-end-to-end, claude-325

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `46aeeb486`.

## Goal

**Train (b64L) at claude-322's recipe on claude-324's mix, which is claude-322's ×6 capped mix plus the current-state corpus. Then see whether the bare reply and `complete` move.**
- claude-322's served replies answered the job and home questions with friend/dog templates. The mix taught neither the task nor the bare `{value}.` reply.
- claude-324 built that corpus. Its mix is sealed at `results/claude/one-model-end-to-end-currentstate-2026-11-08/train/`, with shas: tokens `b34e72542273f3ac1b040f7ec5c940b07b132898d6862e6df5155b2b9962c191`, mask `a8aaa73bbd6c8ee6759cf6ea2a2e46a8eb620885274c1ba75f962d0535b1f50c`, manifest `5efff97971db6a05148f01153c626b4f7ef8177d084bee0da096a93542e6ec13`.

1. **The prep sheet** is `docs/harness/prep/one-model-end-to-end-b64l-d3mwcs-2026-11-08.md`. It follows claude-322's sheet (`…-b64l-d3mwcap6-2026-11-08.md`) in everything but TRAIN:
   - the pin `ff5156bc…`, seeds 1 and 2, 16,384 steps, and one RTX 5090 Vast pod per seed;
   - the export, serve and score path, unchanged.

   It pins the mix's shas and the scripts' shas. Its I-3 checks the trainer's own count of usable windows, expected near 457,421. Its FIT is fresh and by a non-author, and it carries the pre-run review.
2. **The reading:**
   - `m2-fresh256` served replies, scored as in claude-322;
   - the first-line count;
   - the share of job and home replies that are the bare value;
   - the no-memory panel pair against (b64L-D3);
   - the pair against claude-322's pinned (b64L-d3mwcap6) grades, the same recipe without the corpus.

**Bars, unchanged from claude-322's sheet:**
- stops: at least 128 of 256 at both seeds;
- the most common stopped reply: under 64 copies;
- the panel: 104 complete and 52 pairs;
- the held-out pair: not `behind`.

Claim check:
- **(a)** This is Track A of "Both in parallel" (#2028, 6715318106), under "pipeline first" (#2028, 6709947526).
- **(b)** Only TRAIN changes. No bar moves.

## Cancel probe (run first)

1. **No open PR trains on this mix.** The open PRs are #3863 (claude-323, copy keys on claude-322's mix) and #3860 (deepseek-40, its own trunk).
2. **The inputs are present:**
   - the mix in the store (145.7 MB tokens, 72.8 MB mask, manifest);
   - claude-322's job, chain, seal, watch, seed and read scripts, and its grades at `local/claude/claude-322/score/`.
3. **Compute:** claude-323's two pods run ($1.12/h). Two more stay inside 4 pods and $8/h. Expected spend is about $3.
4. **Disk:** 15 GiB free.

**Result, 8 Nov 17:20Z:** 1 to 4 hold as written.

## Files in scope

- `docs/labs/one-model-end-to-end/**`
- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. The sheet is committed with its pins, its FIT and the pre-run review before any pod comes up.
2. Both seeds run, are sealed to the store and are scored.
3. The REPORT gives the outcome row, negatives first, with one notebook row.

## Stop and ask if

Nothing waits on Casey (§5.1).
- **The pre-run review fails:** no run starts, and the REPORT says why.
- **A seed is NOT_RUN for an instrument fault:** one repair and one rerun inside this unit.

Bound: one unit, two pods, $8 of spend.
