# ORDER: d11-exporter-float-trunk, claude-128

Lab: claude. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md). Base: `ddeb8ce52`.

## Goal

The item's done-when reads:

> A float-trunk snapshot exports to D11 with the audits green and the export gap measured inside the band. The trunk is a SpiralCore/FBS recurrence trunk with the Hamilton gate and the native vector writer head, as Stage 3 trains it. The synthesis (Stage 4) adds tokens per second, reported for decode and for context separately.

**This unit builds and qualifies a GPU evaluator of the `UORLUT01` FBS stack container.** It is decision D3 of claude-127 (#2820 comments 6413121841 and 6413557594).
- The D11 GPU emulation cannot vouch for the artifact at the gap bar's resolution: about 1% of tokens differ at 6 layers.
- So the artifact itself is to be scored on the GPU.
- CHARTER §3.6 keeps panel scoring of an exported artifact on the GPU, and the item names "a GPU evaluator of the container" as one of its two routes.

**The evaluator:**
- **Its core** is an integer-only port of `IntegerStackSession::step_native`'s FBS step (`stack_step_fbs`, its kernels, and the native vector writer). It is written once, in a header compiled two ways:
  - by nvcc into an admitted PTX module, the CUDA evaluator;
  - by the host C++ compiler, a host twin that exists only to qualify the port.

  This follows the `m1-d11-artifact-scorer` instrument (claude-116/117).
- **Its input** is the parsed artifact, through a new read-only accessor in `uor-r4-integer`. There is no second parser.
- **Its output,** per position: the token, the kept cells and the Q12 query. These are what the gap bar's top-1 half and its loss half (the MarginHinge loss from the Q12 query) need.

Scoring the panel and reading `G-D11-ExportGap` is the next unit, under an amended prep sheet. It is not this one.

## Cancel probe (run first)

Cancel if a GPU evaluator of stack containers exists or is in progress, or if the stack runtime changed after `ddeb8ce52`. To check:
- `git log ddeb8ce52..origin/main -- crates/uor-r4-integer/src/stack docs/labs/d11-exporter-float-trunk docs/loop/tasks/d11-exporter-float-trunk`;
- the open PRs and their files;
- a search for `.cu` files and `step_native` callers.

Result:
- No commit after `ddeb8ce52` touches those paths.
- The open PRs are #3274 (this one), #3273 (kimi, m1-address-as-read), #3272 (antigravity, m1-faster-writer) and #3169 (codex, D63). None touches the stack runtime or this item's folders.
- No `.cu` file serves `uor_r4_integer::stack`, and `step_native` is called only in `stack/`, `fbs-compare` and `fbs-toks`.

**Not cancelled (instrument: none needed).**

## Files in scope

- `crates/uor-r4-training/**`
- `crates/uor-r4-integer/**`
- `crates/uor-r4-float-train/**`
- `docs/harness/prep/d11-exporter-float-trunk*`
- `docs/labs/d11-exporter-float-trunk/**`
- `docs/loop/tasks/d11-exporter-float-trunk/**`
- `docs/research/notebook/m4-serving.md`

## Done when

1. **The accessor.** `uor-r4-integer` exposes a read-only, integer-only image of a parsed FBS model: every table, map and shape flag `stack_step_fbs` reads.
   - It adds no arithmetic to the serving path.
   - A crate test checks the image against the parsed model's shape.
2. **The core and its host twin.** An integer-only port of the FBS step sits in `docs/labs/d11-exporter-float-trunk/instrument/`, a standalone crate outside the workspace.
   - Its host twin equals `step_native` exactly (token, kept cells and every Q12 query value) at every position of:
     - the three synthetic FBS controls of `stack/tests.rs` (claude-122's small trunk, claude-127's non-square trunk and its tiny-gain trunk), over their full vocabularies;
     - multi-token sequences that carry state across at least 16 positions.
   - Two planted faults each fail it: one rounding changed in the core, and one lane of the recurrence state dropped.
3. **The CUDA evaluator.**
   - The same core, built by nvcc ahead of time (no NVRTC), into a PTX module admitted with no floating-point, `mma`, `wmma` or tensor tokens.
   - One CUDA thread per sequence, 64 per block. Before any GPU run, this replaced "one block per sequence", to match the evaluator (FIT round 1). The launch shape does not change the integers.
   - A CLI that evaluates an artifact over a file of token sequences, and writes per-position tokens, kept cells and Q12 queries with the artifact's sha256.
   - Type-checked on the Mac with a stub nvcc.
4. **GPU qualification on the pod,** after a prep sheet with a non-author FIT and a D67 pre-run review.
   - On one RTX 5090, the CUDA evaluator equals the host twin and `step_native` bit for bit, at every position of:
     - the finite controls of item 2;
     - both stored Stage 4b artifacts (`results/claude/d11-fbs-export-gap-2026-10-26/d11gap/save/seed-{1,2}/artifact.uorlut`) over every eighth DEV window.
   - It throughput-projects the full 14,287-row panel.
5. **A REPORT and a notebook row.** The REPORT says `References #2820`.

## Stop and ask if

Stop and decide; never wait.
- **The port cannot be bit-exact** without changing the artifact format or the serving runtime's arithmetic. A runtime change is a separate unit with its own failing-first test. A format change needs a schema decision.
- **The device lacks a primitive the port needs** (128-bit integers in device code, for example). Emulate it with 64-bit pairs and test the emulation on the host twin first.
- **A trained artifact disagrees at a position the synthetic controls pass.** That is an instrument fault until localized. Do not record it as a negative.
- **The evaluator would need a host loop in the model path.** That is a defect (CHARTER §3.5). The host twin qualifies the port and never scores.
