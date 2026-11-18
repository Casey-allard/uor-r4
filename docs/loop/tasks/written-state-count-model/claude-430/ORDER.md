# ORDER — written-state-count-model (claude-430)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Unit: claude-430, the radix sort, as Casey asked on 18 Nov ("Order the sort unit").
It follows claude-429 (PR #4088).

## Why

After claude-429, the N = 10 count takes 408.5 s, and the sort is 169.7 s of it (41.5%), the
largest phase. The sort is a stable LSD radix sort of positions, 6 bits per pass. Each pass has
two kernels, a histogram and a scatter, and **both** read the digit from the token array at
`tokens[pos[i] - back]`. After the first slot, the positions are in gram order, not stream
order, so each of those reads is a random read into the 2 GB token array.

So each position pays four random token reads per slot (two passes, two kernels each), and
twelve for the left digit (three passes, two kernels, two tokens each). claude-429's data fit
this: its 27 left passes added 20.8% more passes but 53.7% more sort time. That run's GPU was
also shared, so the split between the two causes was not measured.

## What it does

1. **Gather once per slot** (`ngram_count.cu`, `ngram_count_device.rs`).
   - A new kernel, `ngram_radix_gather`, writes each position's whole key word for one slot into
     a `u16` array beside the positions: `tokens[pos[i] - back]`, or the 13-bit left digit L for
     `back & LEFT_DIGIT`. It uses the same `left_digit` and the same bounds as today.
   - `ngram_radix_hist` and `ngram_radix_scatter` read the digit from that key array, at
     `(keys[i] >> shift) & 63`, instead of from the tokens. The scatter moves each key with its
     position, so the key array stays aligned for the slot's next pass.
   - `sort` runs one gather per slot, then that slot's passes (two for a token, three for L).
     The slots, the shifts and their order are unchanged, so the sort is the same stable sort
     and its output is the same permutation.
   - Random token reads per position fall from 4(k + 1) + 12 to (k + 1) + 2 for order k with
     L, and from 4(k + 1) to k + 1 without. The passes read and write `u16` keys sequentially.
2. **`build.rs`** lists the new kernel; the entry list grows from 20 to 21.
3. **Timing.** The gather counts under `sort`. No phase is added or renamed.
4. **`kn-reference`** writes `unit: claude-430` and is otherwise unchanged.

The key buffers are two `u16` arrays of the partition's size, at most 64 MiB at budget 2^24.

## Gates

- **G1, laptop:**
  - stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib --tests` and the
    instrument's `cargo check --features cuda` are clean;
  - the host `kn_` and `ngram_count` tests pass, and so do the instrument's tests;
  - the D32 grep of the added lines is clean, and each touched file's rustfmt hunk count
    equals main's.
- **G2, wick (CUDA tests at the head):** every `ngram_count_cuda`, `kn_score_cuda` and
  `kn_filter` test passes. They include the raw device count against `oracle_count` and the
  adjusted device sweep against `oracle_adjusted` and the host-runs sweep, at N in {1, 3, 10}
  and three budgets. No test is changed.
- **G3, negatives,** each set aside against the base, run on wick, expected to fail, then
  restored:
  - a scatter that writes the positions but not the keys fails G2's oracle tests;
  - a gather that reads `tokens[pos[i] - back - 1]` for a token slot fails G2's oracle tests.
- **G4, wick, the same GPU for both:** main's binary (`34eff518f`) and the head's binary each run
  one N = 10 `--count-only --profile-phases` count on all of TRAIN against DEV-tune's keys,
  budget 2^24, one after the other. `nvidia-smi` is read before each.
  - The head's `count-10.json` without `unit` equals claude-429's `run1/c429/count-10.json`
    without `unit`, and so does main's.
  - Phases, peak RSS and the whole-device GPU peak are recorded for both.
- **G5, the expected outcome:** the head's sort phase is at most 70% of main's sort phase in
  the same G4 pair, and its count wall is lower than main's.

  This gate is a bar, not a stop. A miss is recorded first in the REPORT, with its cause, and
  the unit still lands if G1–G4 pass.

## Decisions recorded here

- **The digit width stays 6 bits.** Wider digits would cut the passes and the scans, but they
  grow the per-tile histogram 64-fold at 12 bits. This unit takes the random reads out first,
  so a later digit-width unit can measure the passes alone. [claude], reversible by Casey.
- **G4 times main's binary beside the head's** on the same GPU, because claude-429's sort was
  timed on a GPU shared at 98%. [claude], reversible by Casey.
- **No pre-registration sheet,** as in claude-425–429: G4 counts and scores nothing. DEV-test is
  not read.

## Out of scope

- Wider radix digits, fewer passes, or a different sort.
- `download`, `records`, validation, the digest and the census.
