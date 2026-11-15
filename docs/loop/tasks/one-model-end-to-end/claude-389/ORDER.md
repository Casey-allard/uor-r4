# ORDER: one-model-end-to-end, claude-389 (the native Q31 first-pick read operator)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `origin/main` at `e5a32f45`, which holds claude-388 (#3997).

## Why this unit

The first step of the 15 Nov architects' path is done. claude-388 showed that the vector read's hard copy, Flock's first pick alone at one fixed gain, reaches `R_ask` 0.965 to 0.984 at both seeds. That was measured on float reference states.

D85 says floating point is reference code only. So the next step is to write the same read as a native Q31 operator. It goes in `uor-r4-native-vector`, the crate that already holds codex's Q31 Hamilton gate, beside its CUDA twin.

The architects' step 2 names three parts:
- one typed operator;
- called by both the native training step and `IntegerStackSession`;
- with a bit-identical replay.

No native token table, training step for reads, or session read exists yet (codex-1 to codex-11 built only the gate). This unit therefore builds the operator and its device twin, and checks them against claude-388's saved states. Attaching the read to `IntegerStackSession` and training its fields natively are the next units, named under "Still open" in the REPORT.

Casey answered "Start the native Q31 read operator unit here?" with "Yes, start it" (15 Nov 2026).

## What is built

**1. `q31_first_pick_read`, in `crates/uor-r4-native-vector/src/read.rs`.** It is integer only, with no float, transcendental or normalization fallback. For one window of `time` rows, each `width` lanes (a multiple of 16), all words Q31:

- **Address.** Each row's stream is cut into `width / 4` quaternion blocks. Each block is rotated by its own query gate and its own key gate, using `q31_hamilton_rotate`, the crate's existing operator. The float read's `q ⊗ u` is the same left product. The rotated lanes are folded to 16 address words: coordinate `c` is the exact i64 sum of lanes `g·16 + c` over the `width / 16` groups. This matches `vector_reads.rs:669-678`.
- **Score.** The score is the L1 distance `Σ_c |q_t[c] − k_s[c]|`, accumulated in u64, over strictly earlier rows `s < t`. The float score's `/16` is monotone, so it is dropped.
- **Choice.** The least distance wins, and a tie goes to the latest `s`, as in the float read. Row 0 reads nothing. The choice is returned as `s + 1`, with 0 meaning none.
- **Payload.** The payload is the token table's row for the token after the pick, `table[ids[s + 1]]`. Each lane is weighted by a Q31 weight and a gain shift `g` in `0..=31`. For each lane: `p = sat((w·e + r) >> (31 − g))`, then `out = sat(x + p)`. The rounding term is `r = 2^(30−g)` for `g ≤ 30` and `r = 0` at `g = 31`, where the shift is 0. With `g = 0` this is the crate's own rounded Q31 product.
- **Refusals.** The operator refuses words outside the closed domain `-2^31..=2^31`, a width that is not a multiple of 16, `ids` and rows of different lengths, a token id outside the table, and a gain shift above 31.

The operator is a pure function. It returns the picks and the out words, and it changes no state.

**2. Its CUDA twin, in `crates/uor-r4-discrete/src/native_vector.cu`.** The new entry is `native_q31_first_pick_read`, built ahead with nvcc (no NVRTC), admitted in `NATIVE_VECTOR_ENTRIES`, and launched by a `NativeVectorEngine` method next to a `*_reference` twin. The kernel uses integer arithmetic only.

**3. A native replay in the probe** (`recall-payload-probe --native-replay`). At each forward of the float read, it captures exactly the operator's inputs:
- the stream rows `x` at the read;
- the window's token ids;
- the raw query and key gates, before normalization;
- the weight;
- the shared token table.

As comparison-only fields it also captures the f32 first pick (`latest_best`'s rank 0) and the f32 out row. **No float pick, rank or score is passed to the native operator.** The comparison fields are read only after the operator returns.

The replay normalizes the raw gates **once, in f64**. Both sides use these values: the native operator gets them rounded to Q31, and the f64 reference gets them as they are. So the only gate difference between the two is the Q31 rounding, `2^−32`, which is what B assumes.

The replay also computes the **f64 reference**. This is the float read's own definition, written once more in f64 on the same captured f32 inputs:
- the f64-normalized gates above;
- the left Hamilton product;
- the 16-group fold;
- `mean|q − k|` over earlier rows, with the least one winning and a tie going to the latest row;
- `x + w·e`.

The bar judges the native operator against this f64 reference. f64 rounding, about 2^−52 relative, is negligible next to the Q31 LSB. The f32 picks, which are what claude-388 read, are compared on report lines.

Conversion (one rule, recorded in the JSON):
- The gates use the fixed Q31 unit scale `2^31` on both sides, rounded to nearest.
- The stream uses one power-of-two scale `2^e_x`, shared by the query and key addresses. It is the largest scale at which all three of the following fit within `2^31`. That leaves one bit of headroom, for two things:
- a rotated lane, which can reach the block norm, up to `2·max|x|`;
- an out word's rounding, at most `2^g + 2` LSB, which is far below `2^30`.

The three:
  - `2 · max|x|`;
  - `2 · max|x + w·e|`;
  - `2 · max|x + 16·w·e|`.
- The table uses its own scale `2^e_t`, the largest with `max|e| · 2^e_t ≤ 2^31`.
- The weight scale is `2^e_w`, with `e_w = 31 + e_x − e_t − g`. The gain shift `g` is the least one in `0..=31` with `max|w| · 2^e_w ≤ 2^31`. That way the operator's `>> (31 − g)` lands the payload on the stream's scale. A state where no `g` fits is refused and reported, and the outcome reads D. The same applies to a state whose width is not 256: B below is derived for `G = 16`. The JSON records the width.
- The f64 normalization keeps the float gate's `+1e-12` inside the square root (`kernel_controls.rs:93`).

The JSON counts every saturated word: inputs, rotated lanes and out words. Rotated-lane clips are counted in the replay, by recomputing each lane's Hamilton product exactly in i128 from the same Q31 words. A lane counts as clipped when its rounded value has `|lane| > 2^31` (strictly, because the domain is closed). The operator stays a pure function. A clip in a barred computation sends the outcome to D.

This conversion is a diagnostic in reference code (D85). It is not a model export, and nothing it makes is served.

## Native-space five items (SKILL)

- **State schema.** The operator's learned fields are the query gates and key gates (`width / 4` Q31 quaternions each), the per-lane Q31 weights (`width` words) and one gain shift. Its inputs are the Q31 stream words, the token ids and the Q31 token table.
- **Shared operator path.** There is one Rust source definition (`read.rs`) and one CUDA twin, and a bit-identical differential test covers both. A WASM build of the crate is a plain `--target wasm32` check, because the crate is dependency-free.
- **Replay oracle.** Raw words are compared, never rounded values: the picks (`u32`) and the out words (`i64`), CPU against CUDA, on fixed and seeded fixtures.
- **Forward, gradient, update.** Forward only. This unit has no gradient or update for the read's fields; the native update rule is a later unit. So by the SKILL this is an **operator unit, not a training run**.
- **Serialization.** The fields are raw little-endian words with a version byte, like `Q31HamiltonModel::to_bytes`, and a round trip is tested. Attaching them to the lut-stack artifact is the session unit's.

## Sheet (§4.5: one GPU evaluation, the native replay on claude-386's saved states)

- **States.** These are claude-386's saved states on wick, under `/workspace/in/c387/`, with their `model.safetensors` / `vector_read.safetensors` sha256:
  - **Barred**, the 16,384-step states:
    - `try-5/ext-flockg-b2-1/run/run/models/step-016384` (`e39cb058…`, `944c759d…`);
    - `try-6/ext-flockg-b2-2/run/run/models/step-016384` (`13cfba37…`, `bccbdb23…`).
  - **Run and reported, with no bar**, the 4,096-step states:
    - `try-1/arm-flockg-b2-1/run/run/models/step-004096` (`bffad79f…`, `98a1a7c3…`);
    - `try-3/arm-flockg-b2-2/run/run/models/step-004096` (`b00f5adc…`, `1fa08f29…`).
  - Full sha256, in the same order:
    - `e39cb058447f5deefa22ac277f82261338d4923cec75bffda46a871ff8004fc5` / `944c759df1c82f5600eed8176bdb82ff6d6162d51148c71991c181964b2362e8`
    - `13cfba378fdbfc2eb34849c6eeeaf40ad423d8aa408d1f2b67917e4480032061` / `bccbdb23543e811e426174db6d13e5279af0b0126f73fd8654fc3713fd659314`
    - `bffad79fcb8a628ad0b8306d3c2113117c1db28334cca49c55f843fac81b9e4a` / `98a1a7c315b692aba16d0f524d1b856f4c0dd5d4aace0f6a4c21d4032628e26d`
    - `b00f5adc01e484511cbd17d0ead32cdf37da8baf3997d3bd9e643fafd633fd10` / `1fa08f29036df646e0c2ab78bdea75ebb587dd4aff880cccbf4a678e66bb4bc2`
- **Error bound B**, of the native score against the f64 reference. It is derived for width 256, so 16 fold groups `G`, with `LSB = 2^−e_x` in float units and every stream word at most `2^30` (the headroom rule):
  - Each rotated lane errs by at most 2.5 LSB: 1 from the stream's rounding through a unit gate (the sum of `|g_i|` is at most 2), 1 from the gate's rounding (4 terms, each `2^−32 · 2^30` relative), and 0.5 from the product's rounding. **4 LSB** is used.
  - So an address coordinate errs by at most `4G = 64` LSB, and `|q − k|` by at most 128 LSB.
  - The L1 sum of 16 coordinates divided by 16, the float read's mean, errs by at most **`B = 8G = 128` LSB**.
  - Two native scores can reverse the f64 order only where the f64 scores are within **`2B`**.
- **f32 bound `B_f`**, of the float read that ran against the f64 reference. Unit roundoff is `u = 2^−24`, and `M = max|x|`:
  - Each rotated lane errs by at most `8u · 2M`. That covers f32 normalization, four products and three sums.
  - A fold coordinate sums 16 lanes, adding `16u` of summation error per lane. So it errs by at most `(16 · 8u + 16 · 16u) · 2M < 2^10 · u · M`.
  - So `|q − k|` errs by at most `2^11 · u · M`. The f32 abs-diff and the sequential 16-term mean add up to about `2^10 · u · M` more. The bound used is **`B_f = 2^12 · u · M = 2^−12 · M`**, and the f32 out word by at most **`2^−22 · (|x| + |w·e|)`** per lane.
  - `B_f` is a gross bound, far above `B`. It exists only to catch a capture error, such as swapped gates, a wrong stream tensor or a wrong window. Such an error moves scores by amounts of the order of `M`.
- **Out-word bound** against the f64 `x + w·e`, per lane: **`(2^g + 2)` LSB**.
  - The weight's and the table's rounding each give `0.5 · 2^g` LSB at the stream's scale.
  - The product's shift and the stream's rounding give 0.5 LSB each.
  - The last LSB is slack for the f64 reference.
- **Bar.** On the 1,024 held-out query rows of each barred state, all three conditions must hold:
  1. **Picks.** The native pick equals the f64 reference pick on at least 99% of rows (at least 1,014).
  2. **Out words.** On the rows where the picks agree, every native out word, divided by `2^e_x`, is within the out-word bound of the f64 reference, at the trained scale (×1, gain `g`). The REPORT names `g`.
  3. **No clip.** No saturated word in either barred computation.
- **Noise.** None. Picks are deterministic, so a miss is rounding at a near-tie or a bug. The JSON counts, before any classification:
  - the rows whose f64 top-1 and top-2 scores are within `2B`, the most mismatches rounding can explain;
  - for each mismatching row, `|ref score(ref pick) − ref score(native pick)|` in LSB.
- **Controls:**
  1. **The replay follows the definition, and can fail, on the real states.** Every key gate is replaced by the fixed non-identity gate `[2^30, 2^30, 2^30, 2^30]`, in both the native operator and the f64 reference. This is a unit quaternion, and left multiplication by it is an isoclinic 60° rotation of four-space. Two conditions must hold at both seeds:
     - (a) under the substitution, no native-vs-reference pick mismatch lies above `2B`. Its near-tie count, the rows within `2B`, is reported.
     - (b) the native picks under the substitution agree with the unsubstituted native picks on less than 99% of rows.

     If (a) or (b) fails, the outcome reads D.
  4. **The capture matches the read that ran.** This ties the replay to the float forward, because the native operator and the reference share the captured inputs. Two conditions, on the 1,024 held-out query rows of every state:
     - (a) every row where the f32 first pick and the f64 reference pick disagree has `|ref score(ref pick) − ref score(f32 pick)| ≤ 2·B_f`;
     - (b) on rows where those picks agree, every f32 out word is within the f32 out-word bound of the f64 out word.

     If (a) or (b) fails, the outcome reads D.
  2. **CPU against CUDA.** On the same converted inputs from one barred state, the operator and its CUDA twin agree bit for bit on picks and out words.
  3. **A 1-LSB perturbation.** In the fixture built near a tie, changing one key-gate word by 1 LSB changes at least one pick or out word. This shows the bit-for-bit differential can fail.
- **Structural check (not a control).** Every window's row 0 returns pick 0, with out words equal to its stream words. The JSON counts these rows.
- **Report lines (no bar):**
  - native picks against the f32 first picks claude-388 read, with each f32-vs-f64 disagreement counted;
  - native first-right against the f32 counts, which were 1,000 and 990;
  - the 4,096-step states' agreement and first-right;
  - the out-word check at ×16 (gain `g + 4`, bound `2^(g+4) + 2` LSB).
- **Seeds.** 2 (claude-386's B2 seeds 1 and 2).
- **Cost.** Minutes on wick, $0. No pod and no training.
- **Reading map** (fixed before any number is read). Each outcome takes exactly one letter, tested in this order:
  - **D:** a control fails (1, 2, 3 or 4), the structural check fails, or a barred computation clips. The outcome is reported as read, with no fix named.
  - **N2 (definition):** otherwise, if any of a barred state's 1,024 held-out query rows has a pick mismatch above `2B` or an out-word miss above 4× its bound. Rounding cannot explain such a row, so the bar's 99% does not excuse it. The native definition (rotation, fold, tie rule, gain) differs from the float one, and the next step is to find the difference. Every such row is listed.
  - **P:** otherwise, if all three bar conditions hold at both seeds. The Q31 operator reproduces the float read's definition, both its choice and its write, and the next unit attaches it to `IntegerStackSession`.
  - **N1 (precision):** otherwise. A bar condition fails, but every pick mismatch is within `2B` and every out-word miss is within 4× its bound. The fix is a finer scale or a wider accumulator. It is not a mechanism negative.
- **FIT:** fresh-context non-author reviews.
  - The first review, on 15 Nov, returned UNFIT with six findings, folded in at `2e59c2fac`.
  - The second, also UNFIT, found five, all folded in here:
    - the f64 reference in place of f32, since f32's own error exceeded B;
    - one bit of headroom, with clips counted and sent to D;
    - the out-word bound `(2^g + 2)` LSB;
    - an ordered reading map, with controls 2 and 3 routed to D;
    - control 1 as a fixed non-identity rotation in both the native operator and the reference;
    - row 0 relabelled as a structural check, and full sha256 values.
  - The third, also UNFIT, found four, folded in here:
    - N2 is tested before P, and any mismatch above `2B` reads N2;
    - control 4 ties the capture to the f32 read that ran, within `B_f`;
    - the gates are normalized once in f64 and shared by both sides;
    - control 1(a) is judged by no mismatch above `2B`.

    It also raised three minor points, all folded in: the rounding term at `g = 31`, the label of the control-1 gate, and the strict clip test.

  - The fourth, at `9d9b960ce`, returned **TEST FITNESS: FIT**. It raised six minor points, all folded in here, and each only tightens the sheet:
    - `B_f` doubled to cover the mean's own rounding;
    - one bit of headroom for out words;
    - "no `g` fits" and a width other than 256 both read D;
    - the rows for N2 and control 4 named;
    - the `+1e-12` epsilon stated.

## Cancel probe

Before the work, I checked whether any native read operator exists on `main`. At base there is none: `uor-r4-native-vector` has only the gate, its rotation and its training step (codex-1 to codex-11).

## Files in scope

- `crates/uor-r4-native-vector/src/{lib.rs,read.rs}` and its tests
- `crates/uor-r4-discrete/src/native_vector.cu`, `src/native_vector_device.rs`, `build.rs` (the entry list only), `tests/native_vector_cuda.rs`
- `crates/uor-r4-float-train/` (the probe's `--native-replay` and its capture of the read's inputs; `Cargo.toml` gains the dependency-free `uor-r4-native-vector`)
- this folder, and one row in `docs/research/notebook/float-training.md`

## Stop list

- No training, and no change to the float read's default forward or to training.
- No change to `IntegerStackSession` or to the artifact format.
- No pod. No owner path.

## Done when

1. `q31_first_pick_read` and its tests pass on the laptop: fixtures for the tie rule, the fold, saturation, the gain and each refusal.
2. The CUDA twin matches the oracle bit for bit on wick (`--features cuda --test native_vector_cuda`), and the 1-LSB control fails as it should.
3. The native replay, with control 1, has run on the four named states: the two barred states and the two reported 4,096-step states. Its JSON is fetched with sha256.
4. The REPORT names the reading map's letter with the numbers, and the notebook row is written.
5. The PR goes through a fresh exact-head review, the queue and `close --verify`.
