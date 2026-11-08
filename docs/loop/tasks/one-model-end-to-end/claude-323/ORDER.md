# ORDER: one-model-end-to-end, claude-323

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `ec12fb855`.

## Goal

**Train (b64L) on claude-322's × 6 capped mix with a mixed pair of copy keys, `--hamming-copy-keys 2,1`, and see whether the replies start carrying the asked-for value.** This is claude-322's decision review (row 5, #3407 6723513106). The replies now stop (109 and 86 of 256) without collapse, but none holds the asked-for value. The most common stopped reply, " Your friend's name is basalt lo.", copies the start of a two-token value and drops the rest.

Why this lever:
- `--hamming-read 4` has 2 copy heads. Both read with a one-token key, the token before the copy position (claude-322's recipe, the default). A one-token key reads "my friend is" and "my number is" alike (claude-259). After " lo" it has no key that says which value it is in the middle of.
- A two-token key (the token and the one before it) can continue a multi-token value. But on its own it was right less often than the one-token key: claude-267's oracle on DEV / p-w gave k1 3,859 / 12,371 and k2 3,506 / 10,971. claude-266's (k2) arm never beat (b)'s copies.
- The best rule in claude-267's oracle was k2 backed off to k1: **4,388 / 13,990**, about 14% over k1 alone. A mixed pair is the read's form of that rule: one copy head on key 2, one on key 1.
- claude-268 and claude-269 built the mixed pair through training, export and the integer serve, with parity tests at `[2,1]` and `[1,2]`. It has never been trained.

So this run changes only the copy keys. Everything else is claude-322's: recipe, mix, pin, seeds, steps, panel and bars.

1. **The prep sheet** `docs/harness/prep/one-model-end-to-end-b64l-d3mwcap6-k21-2026-11-08.md`. It follows claude-322's sheet in everything but the one flag. It pins:
   - claude-322's mix (tokens `0273173e…`, mask `da506e85…`, manifest `442ead2c…`, 417,421 windows), read from the store;
   - the job, chain, seal, watch, seed and read scripts' shas;
   - a gate that the run's `config.json`, checkpoint and served artifact header all record `copy_keys` `[2, 1]`;
   - a fresh, non-author FIT and the session's pre-run review.
2. **The run and the reading:** `m2-fresh256` served replies, scored as in claude-322, with the no-memory panel pair against (b64L-D3) and against claude-322's (b64L-d3mwcap6), the same recipe with one key. **The first-line count** (asked-for, other, both) is the lever's first reading.

**The bars are unchanged from claude-322's sheet:**
- stopping: at least 128 of 256 EOS stops at both seeds;
- collapse: under 64 at both seeds;
- the panel: 104 complete and 52 pairs;
- the held-out pair is not `behind`.

Claim check:
- **(a)** This is Track A of "Both in parallel" (#2028, 6715318106), under "pipeline first" (#2028, 6709947526). The run is judged by the served-reply score.
- **(b)** It changes one trainer flag that is already built and tested. No bar moves.

## Cancel probe (run first)

1. **No open PR trains mixed copy keys.** The one open PR is #3860 (deepseek-40, a fresh-panel opening probe on DeepSeek's own trunk).
2. **The inputs are present:**
   - the pinned trainer `ff5156bcc` carries `--hamming-copy-keys`: 11 references in `in_space_long.rs`;
   - its validation refuses the flag only with `--hamming-read 0`, with `--hamming-copy-key` also given, or with one key for every head;
   - `--hamming-read 4` has 2 copy heads, so the list is `2,1`;
   - claude-322's mix is in the store under `results/claude/one-model-end-to-end-b64l-d3mwcap6-2026-11-08/train/`;
   - claude-322's scripts and seals are on the laptop.
3. **Compute:** no pod is running. Two pods stay inside 4 pods and $8/h. Expected spend is about $3, as in claude-322 (about $3.0).
4. **Disk:** 16 GiB free on the laptop.

**Result, 8 Nov 15:55Z:** 1 to 4 hold as written.

## Files in scope

- `docs/labs/one-model-end-to-end/**`
- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. The sheet is committed with the mix's shas, the scripts' shas, its FIT and its pre-run review, before any pod comes up.
2. Both seeds run, are sealed to the store, and are scored. Each run's `config.json` and served header record `copy_keys` `[2, 1]`.
3. The REPORT gives the outcome row, negatives first, with one notebook row.

## Stop and ask if

Nothing waits on Casey (§5.1).
- **If the pre-run review fails:** the run does not start, and the REPORT says why.
- **If a seed is NOT_RUN for an instrument fault:** one repair and one rerun inside this unit.
- **If the served path refuses a mixed record:** it is an instrument fault (row 0c). The seals stay; the export or scorer is repaired and re-read with no pod.

Bound: one unit, two pods, $8 of spend.
