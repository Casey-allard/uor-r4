# ORDER: one-model-end-to-end, claude-281 (P2b's Negative classified: trained phrasing against held-out phrasing)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `ae8fce0b7` (`ae8fce0b715211bf204a83e6130811c895a21bfa`, main after claude-280 and deepseek-140).

## Claim check

- **The step it advances:** THE ORDER of 4 Nov (#2028 6634218223), step 4, "P2b classification: a sheet, a FIT and a finite scoring pass". It also carries the four decode-cache hardening SHOULDs from claude-279's review. [P2b](../claude-276/REPORT.md) read Negative, and §4.2 asks for an instrument-or-mechanism classification before that Negative picks the next configuration.
- **What it changes in the model that trains or serves:** no weights, and no scored output on P2b's panels. It decides what the next training unit changes:
  - the data (held-out phrasings in TRAIN, or the real-text corpus, THE ORDER step 6) if the read recalls in trained phrasings;
  - the read's configuration (copy key 2, or the latest match) if it does not.

  It also lands the four refusals in the decode cache that every scoring pass now uses, and makes the census name the queries it sets aside.

## How this unit was claimed

By hand, as [claude-276](../claude-276/ORDER.md) to [claude-279](../claude-279/ORDER.md) were. The app's edit guard lets this session write only in its own worktree, `.worktrees/claude-i3-projection`, so the branch `claude/p2b-class` is cut there, off `origin/main` at the base. Every other check `loop claim` makes holds by hand:
- `one-model-end-to-end` is on `origin/main` with a `scope:`;
- Claude has no other open PR (#3677 merged at `4da8de353`);
- the only open PR, Kimi's #3675, changes no file in this unit's scope.

## Goal

Score P2b's sealed (L) at seeds 1 and 2, at 2,048 with the decode cache, on three new panels beside p-long: `train,train` (the positive control), `train,dev` and `dev,train`. Then read the sheet's Bar: Phrasing, Mechanism, or Inconclusive.

## Cancel probe (run first)

- **Is P2b's Negative still the open question?** Check that no merged unit after claude-276 scored a positive control at 2,048.
- **Do the checkpoints exist?** `uor-store ls` P2b's `s{1,2}/L/models/step-008192/` and their `SHA256SUMS.files`.
- **Does `heldout-panel` draw the three cells?** `cell_of` parses either order (instrument `main.rs:72-82`, the test at `:492`).
- **Classed:** instrument.

## Files in scope

- `crates/uor-r4-float-train/src/flock_read_hamming.rs`, `crates/uor-r4-float-train/src/geometric_stack_native.rs`, `crates/uor-r4-float-train/src/in_space_long.rs`: the four refusal fixes, the census's `found_queries`, and their tests
- `docs/harness/prep/one-model-end-to-end-p2b-classification-2026-11-04.md`
- `docs/loop/tasks/one-model-end-to-end/claude-281/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. **The four refusals and the census field** land, each with a CPU or CUDA test that fails with the fix set aside. The CUDA tests pass in I-0 on the pod.
2. **The sheet** is in `docs/harness/prep/` with a fresh non-author FIT and a pre-run review, each posted on #3407.
3. **The run** is scored and sealed under `results/claude/one-model-end-to-end-p2b-classification-2026-11-04/`, never inside P2b's sealed root (L-002), and read back from the store. The pod is down.
4. **The REPORT** reads the Bar per the sheet, with R2 to R5 and the distance bins, negatives first. One notebook row, and the item line.
5. **Review:** a fresh exact-head review passes, with a claims check of every number. `loop report --check` and `loop pr-check --local` pass.

## Stop and decide (never wait) if

- an instrument check fails: repair it and re-run once inside the $3 bound, and never count it as a result;
- the seeds disagree: read Inconclusive, and name or close nothing;
- the cost projects past $3: extend the bound once, note it on #3407, and never cancel a healthy run.

## Cancel probe result (4 Nov, about 20:50Z)

- **Still open.** Nothing merged after claude-276 scored a positive control: claude-277 and claude-278 are P3, and claude-279 is the decode cache.
- **The checkpoints exist.** `s1` and `s2` each hold `L/models/step-008192/` (`config.json`, `hamming_read`, `model`, `turn_memory`, `turn_memory_reply`) and a `SHA256SUMS.files`.
- **The panels are drawn,** on the laptop, with the pinned tokenizer `d36d3e87…`. Each draws `heldout-panel … conversations=160 protocol=2 chain=2048` and gives 0 metered mismatches, and `check-queries` reports 0 failed:
  - q-tt `cell=train,train seed=2761`: 617 queries;
  - q-td `cell=train,dev seed=2762`: 610;
  - q-dt `cell=dev,train seed=2763`: 590.

  They are stored, read back against their `SHA256SUMS`, under `results/claude/one-model-end-to-end-p2b-classification-2026-11-04/inputs/`.
- **The census against P2b's TRAIN** reproduces p-long's 6 number answers (10, 11 and 16) and finds:

  | panel | words in TRAIN | numbers in TRAIN |
  |---|---|---|
  | q-tt | 3 of 298 | 318 of 319 |
  | q-dt | 2 of 286 | 303 of 304 |
  | q-td | 0 of 301 | 10 of 309 |

  A trained value that is a number comes from TRAIN's own pool, so a reply can state it without reading its note. The sheet's deciding reading is therefore words against words, and the census must name the queries it sets aside: it lists at most 16 found values today. Classed instrument; the fix is the census's `found_queries`, in this unit.
- **One build fault, in my own target directory:** the instrument crate is a separate workspace whose lock pins `serde_core` 1.0.229, against the workspace's 1.0.228. Building it into the shared target directory overwrote `uor-r4-integer`'s and `uor-r4-core`'s artifacts, and `train-in-space` then failed with E0277. A `cargo clean -p` of the two crates repaired it. From now on the instrument builds into its own target directory.
