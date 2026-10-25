# REPORT: m3-nonrelation-prose-panel, claude-94

Lab: claude. Order: [ORDER.md](ORDER.md).
Closes #2856.

## What did not work, first

- **The registered baseline could never run lawfully.** The pre-registration of 22 Oct pinned the saved world=v2 compiler (`compiler-save-op-v25-rawtable`). Its relation and act heads are `SparseSoftmax` fits that decide through a normalized exponential (`relation_compiler.rs:578-584`), which CHARTER §3.2 bans. Its module, the session module and the grader were all `#[cfg(test)]`, so no lawful binary could run them. An earlier draft of this item (#3187) recorded that as a cancel. This unit fixes it instead: it builds a lawful compiler and measures it.
- **The decision review required six changes before any building** (owner log `6394013509`). They were a recall floor, no panel leakage, scoping the claim, two superseding notes, and naming the host steps. All six are in the ORDER.
- **The Mac could not type-check the launcher under `cuda`.** That feature pulls Candle's own CUDA kernels, which a stub nvcc cannot build. The sieve kernel needs only cudarc, so a lean `sieve-cuda` feature (which `cuda` implies) builds just its PTX. The crate's unsafe-code policy (forbid without a GPU feature, deny with one, and an `allow` at each launch site) extends to it.
- **`loop guard` reads item scope from `origin/main`,** so this unit could not add its own notebook to the item's scope in place. claude-96 (#3191) landed that, and the discriminator gate's note, on `main` first.
- **The first FIT was NOT FIT, with two faults, both fixed.** The second-build check named no artifact to compare (now `decisions.json`), and `--expect-ptx` was optional (now required).

- **The result's own negatives.**
  - **R2 is 0/220.** The compiler compiles no unseen phrasing, though every miss is `Unresolved`, never a wrong write.
  - **The panel's agreement gate was never exercised.** No panel turn aligned with any frame (0 survivors in total), so the panel is clean because nothing aligned.
  - Both follow from exact frames. They are the starting point for `m3-memory-turn-discriminator-gate`, not a pass of it.
- **One copy off the pod failed and was redone.** The first tar stream carried `sha256sum` text ahead of the archive, so it did not open. The redo verified sums and streamed the tar as two separate commands, and all 12 sums matched locally.
- **`uor-pod down` could not reach the pod over SSH to verify its archive.** I confirmed the HF upload with `uor-store ls` (17 files), then ran `down --confirm-archived`. The pod is deleted, and 0 pods are running.

## What changed

- **`turn_action.rs`** holds the compiled-turn types (`CompiledAction`, `MemoryEffect`, `TurnCompiler`, `GroundedSessionError` and the rest). They moved unchanged out of the test-only `stack_grounded_session`, which re-exports them. The grader `nonrelation_prose_panel` and the world=v2 data modules (`milestone_world`, `milestone_world_v2`) are in the non-test build. They contain no banned code, and un-gating them adds no warning.
- **`sieve_frame_compiler.rs`**, the lawful compiler's host side:
  - the word rule and the prime lexicon;
  - the frame table from training phrasings only, with its digest `379c2ecc…604f` pinned by a test;
  - the flat device layout;
  - decoding of stored decisions;
  - `DecidedCompiler`, the registered control draw, and 10 finite host tests.
- **`sieve_frame.cu`**: the decision (sieve, align, agree) in one ahead-of-time kernel, one thread per turn. It uses integer AND, compare and bounded loops only.
- **`prose-panel-sieve`**: the binary. `controls` runs the 12 finite CUDA controls. `run` runs the pinned panel, R1 and R2 in one launch, twice, and writes `record.json` and `decisions.json`.
- **Build wiring.** `build.rs` builds each PTX module under its own feature, and the D29 interlock covers `sieve_frame.ptx`.
- **Docs.** The prep sheet, a superseding note on the 22 Oct sheet, and a section in the lab README.

## Measurements

One RTX 5090, about 6 minutes, run ref `3e99a0d63912e4dfd7b8a7043c1274d13137e250`. The full table and pins are in the [lab record](../../../../labs/m3-nonrelation-prose-panel/README.md#sieve-baseline-claude-94-25-oct-2026). Outputs are in the HF store under `results/claude/prose-panel-sieve-2026-10-25/`.

- **896,898 tokens/s** (lean build; 732 turns, 4,280 words, 4,772 µs from upload to download). The `cuda` build gives 811,682.
- **Panel:** false_assert **0/50**, false_overwrite **0/50**, clean **50/50**.
- **R1** (training phrasings with development values, plus each query phrasing once): **462/462** exact, so the 95% floor is met. **R2** (development phrasings): **0/220**.
- **Registered bar met**, together with the floor.
- **Instrument:** finite CUDA controls 12/12 on both builds. The second build's `decisions.json` is byte-identical (`d3f05ecf…f645`). D29 passed on both features, 5 tests each: under `cuda` in the bootstrap (`d29.log` inside the HF store's `bin/3e99a0d63912e4dfd7b8a7043c1274d13137e250-sm120.tar`, sha256 `0cf7dba4…db5f`), and on the lean build (`d29-lean.log`). The pins held: table `379c2ecc…`, PTX `b80f96c4…`, panel `7c29c665…`.
- **Notebook:** one row in `docs/research/notebook/m3-durable-memory.md`, verdict positive at this scope.

## Checks run at the head

- `cargo test -p uor-r4-training --no-fail-fast`: lib 72, chat-grade 26 and doc-tests 0 passed, none failed.
- `sieve-cuda` stub-nvcc type-check of the lib, the binary and the tests: clean. The real build, and D29 on both features, ran on the pod.
- `uor-harness loop gates --full`: 11 of 11 GREEN. `loop gates --quick`: GREEN.
- `cargo xtask roadmap check`: 92 items OK. `roadmap render`: re-rendered for `status: done`.
- After review, `device_layout_matches_the_frames` checks `prefix`, `suffix`, `slot` and each signature one by one. **Negative gate:** with the two count pushes in `device_frames` swapped, the rebuilt test fails (it panics at its `prefix` assert). Restored, the lib passes 72 after a rebuild. This change is to the test only.
- The fresh in-unit review is in the PR's review record: three slices (the wiring and kernel, the host compiler and binary, and a claims check of this write-up against the run record).

## Skills used

- holding-the-goal: the item's done-when is the baseline rates, so a banned compiler is a reason to build a lawful one, not to stop.
- scope-guard: the notebook scope went through claude-96 on `main` rather than around the guard.

## Still open

- **Review note, minor, not changed after the run.** The launcher uploads `frames.tokens` without the empty-array guard it gives the turn tokens (`sieve_frame_compiler.rs`, `device::compile`). A frame table whose every frame has no words would fail at allocation. The world=v2 table cannot do that: its test asserts every frame has a word. Wrap it the same way when the compiler first gets another table.
- **Review notes, minor, not changed after the run.**
  - `decode` trusts the hole the kernel reports. It does not re-check `hole_start == prefix length`, the suffix fit or the value cap, so a wrong but in-range hole would show as a recall miss, not exit 1. The 12 CUDA controls pin the exact holes, and R1's 462/462 exact spans show the kernel's holes are right.
  - The binary ignores unknown flags. A typo falls back to the registered default, and the record stores the seed and values per phrasing it used.

- `m3-memory-turn-discriminator-gate` (#2855) stays open. The sieve compiler's abstention is not that gate. Its work is a compiler that also compiles unseen phrasings and keeps the panel clean.

## Plain summary

The prose panel finally has its baseline, measured on a compiler the rules allow. Ordinary prose never turned into a memory write (0 of 50), and true statements in the trained wordings still wrote correctly (462 of 462). The same compiler understood none of the 220 unseen wordings. That is the gap the next M3 item, the discriminator gate, has to close without letting prose through.
