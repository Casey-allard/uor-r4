# Prep sheet: does rounding the projections to 2^-16 close I-3? (one-model-end-to-end, claude-264, 3 Nov 2026)

TEST FITNESS: FIT - fresh non-author adversarial review (round 2) of this sheet, the code at `5ce220ebc`, `job-264.sh`, `linear-hunk.patch` and `compare-264.sh`. Round 1 cleared the bar, its five outcomes, the comparison and the tests' assertions. It failed the job, because its model-test path named `geometric_stack_native::` where lib.rs mounts the file as `geometric_stack`; that is fixed. Round 2 checked the integer side and found it is floor(v·2^16 + ½) of the signed gathered value in all six projections, with a left shift when the activation exponent is at or above −16. It also checked that no route read-back runs under the served bridge, and that the six weight names and widths match the forward. One pre-existing note goes to the REPORT: `q16_residual` in f32 is one step high for |x| ≥ 128.

PRE-RUN: PASS - fresh D67 review, round 2. Its fixes are in `job-264.sh` SHA-256 `b100f632bf16b1b2a48509e1201e77568317f2db8bd5c06e44fd101f1b79fb6e`, the file launched: the model-test path, five separate fetch logs, and the job refusing any sample count other than 310. `compare-264.sh` is `261f75b4913a157f6f13e7be359a0f8eeef6e353e58c8cfa37322d36582b1000` and the watcher `watch-264.sh` is `2977c4d67f54e23a6bf623b4a09e47a5730191a1d01b876dc48655d3bd1db425`.

## Question

claude-263 (#3592) found that claude-262's served step-256 checkpoints, exported and served in integers, give a different first token from the float model on 112 and 137 of DEV's 310 prompts, against a bar of 8.

A read of the code at `0073cd07b` found one rounding site that the integer session has and the float served surrogate lacks:
- **Integer side.** Every FBS projection output (`rec.in`, `rec.gate`, `mlp.gate`, `mlp.up`, `mlp.down`, `rec.out`) is a gathered, signed input value converted to the residual's 2^-16 grid, rounding half up and clamping to i32 (`stack_fbs_gather`, `to_exp_i32`, `kernels.rs:91`).
- **Float side.** `with_served_surrogate_training` rounds only the residual stream (`q16_residual`). `linear` returns the projection unrounded, and `rec.gate` reaches the decay score and the icosian snap as raw f32.

claude-135 measured near-tie snap margins of 3e-5 to 1.2e-4, which is 2 to 8 units of 2^-16.

This unit adds that rounding to the surrogate and re-scores the same two checkpoints in float. Its question: does the float served forward then give the integer session's first tokens?

## Base artifact

- claude-262's served runs `r/served-s1` and `r/served-s2` at `models/step-000256/`, from the seal `results/claude/one-model-end-to-end-served-repair-2026-11-03/claude-262.tar` (SHA-256 `d04505ac64af7c9f37c60c1e26b00943cad16b3bd36aebde37541da8cabb00e3`).
- claude-263's integer replies `served-s1-toks.json` and `served-s2-toks.json`, which this unit does not change. They come from the seal `results/claude/one-model-end-to-end-i3-repaired-2026-11-03/claude-263-a-serve.tar` (SHA-256 `7216cd08cd118c02b5bd4eee9abe60a9aac1c44a93b3a5baf2a3f3eb0d8eb96e`), members `serve-out/parity/served-sN-toks.json` (`be36ef56…cc94`, `dc5079eb…ca84`).
- claude-263's float `samples-memory.json` files, from the seal `claude-263-a.tar` beside it (SHA-256 `3a364efdc5c0bd633142d3ef21c3ef6a6d2404e00308e4f46e0f3d08f9cbaaf5`), members `i3/out/served-sN-scored/samples-memory.json` (`2194f598…bcd`, `fb149441…d82`). The laptop copies were checked against both tars on 3 Nov.
- **The scoring commit `C`** is `5ce220ebc1e52638c1dadcc60aca42d2caec600a` on this unit's branch: main `45fad9191` plus `served_projection_q16`. The helper is called in `linear`'s FBS path only while the served bridge is on, so plain, Export and D11 forwards are unchanged. Every later commit on the PR touches documents only; the REPORT shows `git diff --stat C H -- crates Cargo.lock` empty at the PR head `H`.

## Readings

For each seed, against claude-263's integer replies:
- order mismatches and prompt mismatches;
- prompts whose first reply token differs between the new float `samples-memory.json` and the integer `served-sN-toks.json`;
- prompts whose whole reply differs;
- how many differing replies first diverge at position 0.

Beside them:
- claude-263's own counts from the same integer files against its float scores. These must give 112 and 137, or the seed is not read (the instrument check).
- How many first tokens the rounding moved in float: claude-263's float scores against the new ones.

## Bar

- **Pass, at both seeds:** order and prompt mismatches 0, and first-token misses ≤ 8 of 310 (2.9%, claude-261's bar). The part is then found. The served trainer takes the same rounding automatically, because training runs the same surrogate, and the full-length served run (claude-263's part B sheet, re-pinned) runs next.
- **Partial:** misses fall below 112 and 137 by more than 10, but stay above 8. The site is one of the parts. The next unit reads the remaining sites: the Hamming reads, the turn memory, the writer's Q12 query, the convolution, and the embedding.
- **Not the part:** misses within ±10 of 112 and 137. The rounding is kept, because it is a real mismatch, and the next unit reads the remaining sites.
- **Worse:** misses more than 10 above 112 and 137. The rounding is wrong in the surrogate; the next unit reads `stack_fbs_gather` against `linear` again before anything else.
- **Not read:** any order or prompt mismatch, a sample count other than 310, or an instrument check that does not give 112 and 137.

A fail is classed instrument (§4.2), as in claude-263.

## Instrument checks

- **I-0a (pod), the negative gate.** The `linear` hunk alone (`linear-hunk.patch`, SHA-256 `b5be1874020f759a30bde32f459ce047cddcc18267a98cdaca4f2d77dd39f026`, from `git diff 45fad9191`) is set aside. The model-level test must then compile, run and fail as exactly one failing test. The hunk is restored, and the tree must be clean at `C` again.
- **I-0 (pod).** `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` passes at `C`, rebuilt after the restore. Both new tests are named in it:
  - `served_projection_rounding_is_the_integer_gather_on_cuda`. The helper equals `(held + 2^7) >> 8` at 2^-16 on exact halves of both signs, and is bit-identical outside the surrogate, under the Q16 control alone and in D11 mode.
  - `served_fbs_projections_round_to_the_residual_grid_on_cuda`. For all six projections of a fixed model, under the surrogate the output is on the grid, equals the unrounded output rounded half up, and carries the unrounded gradient. Outside it, the output is bit-identical.
- **I-1 (laptop, done before the pod).** `git diff --stat 0073cd07b C -- crates Cargo.lock` lists nine files:
  - this unit's two;
  - seven in `crates/uor-r4-cpu-train` (DeepSeek's CPU pilot), which neither `uor-r4-float-train` nor `uor-r4-integer` depends on.

  `Cargo.lock` is unchanged. Apart from this unit's change, the served forward claude-263 scored is unchanged.
- **I-2.** The seal SHA-256 values match `SHA256SUMS`, and the unpacked members match `MEMBERS.sha256`.
- **I-4 (sample count).** Each `samples-memory.json` holds 310 samples, or I-3 is not read for that seed.
- **The comparison itself (laptop, done before the pod).** `compare-264.sh`, run with claude-263's float files standing in as the new ones, gives 112 and 137 first-token misses, 0 moved, and "not the part" at both seeds.

## Data split

DEV, TRAIN and the tokenizer by claude-261's pins, as in claude-263's sheet:
- `ft/dev/tokens.u16` `0919e90f…555b`;
- `ft/dev/response_mask.u8` `a838bb1a…078`;
- `tokenizer.json` `d36d3e87…0f89`.

No panel is read.

## Steps

1. Bring up one RTX 5090 pod at `--ref C`. Push `job-264.sh` as `/workspace/job.sh` and `linear-hunk.patch` as `/workspace/linear.patch`, and check both hashes on the pod. Run `uor-pod run POD --gpu 0 -- bash /workspace/job.sh C`. The job does steps 2 to 4, sealing every exit with `SHA256SUMS.files`, `DONE` and `EXIT`.
2. Source pin, I-0a, I-0, then `cargo build --release --locked --features cuda --bin train-in-space`, recording the binary's SHA-256.
3. Fetch the DEV and TRAIN files and check every pin; fetch claude-262's seal and check I-2.
4. Score the seeds one after the other, never two processes on the card. The command is claude-263's: `train-in-space --data-root DATA --score-rows CKPT --out served-sN-scored --samples 1536 --reply-tokens 64 --retain-rates 1 --seed N`.
5. Pull `/workspace/i3p`, check `SHA256SUMS.files`, and put a tar of it with a `SHA256SUMS` under `results/claude/one-model-end-to-end-i3-projection-2026-11-03`. Read it back, then take the pod down.
6. On the laptop, run `zsh compare-264.sh NEW OLD PAR OUT` against claude-263's float outputs and integer replies. The script's text and output go into the REPORT. No export and no serving are redone.

## Minimum test power

There are 310 paired prompts per seed. The miss counts before the change are 112 and 137. A pass (≤ 8) and "not the part" (≥ 102 and ≥ 127) are more than an order of magnitude apart.

## Cost and stop rule

- **Estimate.** In claude-263, the CUDA suite took 50 s and the build 10 s on a bootstrapped pod, and each seed scored in 532–568 s. Here the negative gate adds one more test build. With bootstrap, about 35 min.
- **Lease.** 1.5 h on one RTX 5090 at under $0.80/h: at most $1.20. A watcher is armed on the job.
- **Stop rule.**
  - A build, I-0a, I-0 or scorer failure stops that step and is an instrument result.
  - A resource fault is rerun once, on a new pod.
  - A healthy run that outlasts the lease is extended, never cancelled.
  - The pod goes down once step 5's read-back matches.
