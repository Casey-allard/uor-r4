# ORDER: one-model-end-to-end, claude-278 (P3, cache unit 1: the trunk's exact step cache)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `14a47d998`. Draft PR #3672.

## Goal

Give the GPU float model a one-position step with carried state, exact to the bit against today's full forward. Today every decode step re-runs the whole window: at a 2,048 window a query decodes at 15.2 tok/s, and decode is 99% of a scoring pass ([claude-277](../claude-277/REPORT.md)). This unit builds the trunk's step and proves it on the GPU. The next unit adds the Hamming read's step and switches the scorer's decode loops to it.

- **Plan step:** Casey's 4 November plan, P3 (faster scoring), cache unit 1 of 2. The cache replaces P3(b) and carries P3(c)'s aim, inside P3's 3-unit bound, by a lab decision posted for Casey on #2028 (6634218223). Casey asked "Can we add cache?" at about 14:45Z on 4 November.
- **What it changes in the model:** nothing the model computes. It adds a second way to run the same forward, one position at a time:
  - **the cache:** each layer's scan state, kept at f32, and the convolution's last three inputs;
  - **the scan's step:** the existing scan kernel, unchanged, run over two positions from a carried state (revised; see below).

  Training's arithmetic, kernels and outputs do not change. Nothing calls the step until unit 2.
- **Why it can be exact:** the trunk is causal. Every op but two works one row at a time, with fixed thread blocks, so its result does not depend on how many rows a launch holds: the norms, the projections, the gates, the quantizers, the residuals and the MLP. The two that look back are the width-4 causal convolution and the scan. The scan walks time one position at a time in f32 registers, and under f32 storage it stores exactly that state.
- **What it mirrors:** the integer session already carries, per layer, the scan state and the convolution's last three inputs (`IntegerStackSession::step`, `crates/uor-r4-integer/src/stack/session.rs`; `LayerState`). The float cache keeps the same two parts, so unit 2 and P4 can share one state format.

## Cancel probe (run first)

1. Does another open PR or order hold the float trainer's files?
2. Does a step cache already exist?
3. Do the caps hold one short pod for the CUDA gates (at most 4 pods and $8/h across all labs)?
4. Is the base `origin/main`?

**Result, at `14a47d998`: not cancelled.**
1. No. No open PR changes a path under `crates/uor-r4-float-train` (`gh pr list` over every open PR's files).
2. No. `rg "DecodeCache|step_cached|prefill_cached|quaternion_scan_step" crates/` finds nothing.
3. Yes. `uor-pod status`: 0 pods, $0/h of $8/h; 5 verified 5090 offers, the cheapest at $1/h.
4. Yes. The base is claude-277's merge commit, `14a47d9988c1bd8beb886a51b7c9c9e8b9ed7ae4`.

## Files in scope

- `docs/labs/one-model-end-to-end/**`
- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/hamilton_sheet.rs`

Both code paths are in the item's `scope:` list (revised; see below).

## Done when

1. **The cache and the step.** `DecodeCache` and two methods on the native stack model:
   - `prefill_cached`: the unchanged full forward over a prompt, which fills the cache from that forward's own tensors;
   - `step_cached`: one position per row, through the same per-position code as the full forward.
2. **The scan's step** through the existing scan kernel over two positions, with a CPU-twin test of the identity it rests on (revised; see below).
3. **Typed refusals,** never a silent second path:
   - bf16 storage;
   - a step that would slide the window;
   - any read hook other than turn memory (until unit 2), and a step row named as a turn-memory opening row;
   - the rate instruments;
   - gate telemetry;
   - a prompt shorter than the convolution's width;
   - added in the work: a D11 snap capture or replay, an FBS projection arm, an id outside the vocabulary, and a prompt longer than the window.
4. **The gates, on the laptop:** the crate's CPU tests pass, and the stub-nvcc CUDA check compiles every target.
5. **The gates, on one GPU pod.** These are finite kernel tests, so they need no prep sheet (§4.5).
   - **Step equals full, bit for bit:** a random small model with the served surrogates on, at f32; prompts of 4, 7 and 33 tokens; 12 steps each; batch 1 and batch 3.
   - **The window edge:** stepping up to exactly the context works, and the next step is refused.
   - **The negatives:** one flipped bit in a scan state, or one zeroed convolution row, makes the compare fail.
   - **The two-position scan** through the CUDA kernel equals the whole scan's next position, bit for bit, on random inputs (revised).
   - **The existing golden CUDA tests** still pass.
6. **The REPORT,** negatives first, with one notebook row and a line on the item.

## Revised during the work (before any pod, at the second code commit)

1. **No new kernel.** The step runs the existing `quaternion_scan_fwd` over two positions:
   - position 0: a zero transition, with the carried state as the drive, gives the state back from the kernel's zero start;
   - position 1: this position's transition and drive.

   **Why:**
   - The step runs the very instructions the whole-window scan runs at that position. A new kernel would be compiled on its own, and nvcc's multiply-add fusing would then have to match the old kernel's for the bits to agree.
   - The 26-kernel extract stays as it is. That keeps D29's compiled-module boundary, `KERNEL_ENTRY_POINTS` and its pin in `tests/no_training_crate_dep.rs`, and every page that counts 26 kernels unchanged.

   **The one difference:** a carried component that is exactly −0 enters as +0. It can change a bit only where every term of a component of the Hamilton product is zero and the drive component added to it is exactly −0. The CPU test pins it.

   **So done-when 2 and gate 5's kernel line change.** The step's identity is tested on the CPU twin, and on the CUDA kernel itself.
2. **Files.** The cache's types live in `geometric_stack_native.rs`. One accessor, `is_instrumenting_snaps`, goes in `hamilton_sheet.rs`. Both are in the item's `scope:` list on `main`.
   - The loop guard refused a new `decode_cache.rs`: it reads the item's scope from `main`, where the widening had not landed.
   - So the widening commit is undone in this commit, and the four kernel-side paths were never needed.
3. **The turn memory in a step:** the reply table's rows only, named with their keys.
   - The opening row reads the window's turn anchor, so a step refuses it. In a decode, the opening is the prompt's last row, which the prefill computes (`in_space_long.rs` names it as `window.len() - 1` on the reply's first step).

## Stop and ask if

- **Step and full differ in a bit,** and the cause is an op whose result depends on the launch's shape. Then the full re-run stays. The REPORT names the op, and unit 2 is re-planned.
- **The change would alter training's arithmetic** or the existing scan kernel's output.
- **The pod's CUDA gates need more than about $1.**
