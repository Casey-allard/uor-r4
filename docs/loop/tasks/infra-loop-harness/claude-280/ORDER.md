# ORDER: infra-loop-harness, claude-280 (register the float-kernel-fusion item)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md), the umbrella that edits roadmap items. Base: `ca9944c5a` (`ca9944c5aecefbc7b91b1b5ff384c6a98ae30cd5`, main after claude-279).

Setup row: 2 (item-file maintenance: a new gap item).

## Claim check

- **The step it advances:** THE ORDER of 4 Nov (#2028 6634218223), step 5, the kernel unit. Its first part is "a new roadmap item float-kernel-fusion first, through an infra-loop-harness unit". This is that part.
- **What it changes in the model:** nothing yet. It registers the item whose units fuse the served quantizers' op chains, `activation_quantize16` and `q16_residual`. Those chains run in every training step under `--train-as-served 1` and in every scoring forward. The fused kernels must be equal bit for bit, so every trained and served value stays the same, and only the launch count and seconds change. It unblocks the kernel unit, which cannot start without a scoped item.

## Goal

Add `docs/roadmap/items/float-kernel-fusion.md`, render the roadmap, and open its issue with `cargo xtask roadmap sync --apply`, run from this worktree.

**What it does not take on.** DeepSeek's [served-rounding-cost](../../../../roadmap/items/served-rounding-cost.md) (#3594) is the item for training as served's slowdown. Its host probe points at the export rounding: `grid_code` on the host and nine matrices copied each step. The new item names that item and leaves the export rounding to it. Its fusion waits on a GPU measurement of the two quantizer chains' share of a step.

**Why.** Casey's words, 4 Nov 2026, about 18:00Z, in Claude's chat, recorded on #2028 (6638209391): "Yes, please build this too to do it the smarter way going forward." That answered the note that a fused kernel moves the float crate's pinned count of 26 kernel entry points. So fused kernels are an approved scope change, and the count moves from one generated kernel list. Casey's other two calls the same hour: the RTX 4090 is for development and kernel work first, and qualification stays on 5090 pods until the bring-up's step 4 shows the cards agree; and "Linux first".

[claude-279](../../one-model-end-to-end/claude-279/REPORT.md) adds a second reason. With scoring's decode on the exact cache, a cached step is estimated at 8–9 ms for (L) at both 192 and 2,048 tokens. That is a fixed cost per forward, and the forward runs the same quantizer chains.

## How this unit was claimed

By hand, as [claude-250](../claude-250/ORDER.md) was, for two reasons:
- **The overlap.** Kimi's open draft #3675 holds `docs/roadmap/items/infra-desktop-client-tauri.md`, so the umbrella's `docs/roadmap/items/*.md` overlaps it and `loop claim infra-loop-harness` refuses. No file this unit edits is in #3675's file list, which has no `docs/roadmap/README.md` or `issues.json`. Casey, in Claude's chat, 1 Nov, about 23:00Z, verbatim: "I told all of the labs never to hold you up because you are doing work on the main track, if kimi is holding on to something or any other lab is, you may supercede them to get whatever done".
- **The edit guard.** The app's edit guard lets this session write only in its own worktree, `.worktrees/claude-i3-projection`. So the branch `claude/kernel-item` is cut there, off `origin/main` at the base, as claude-276 to claude-279 were.

Every other check `loop claim` makes holds by hand:
- `infra-loop-harness` is on `origin/main` with a `scope:`;
- Claude has no other open PR (#3673 merged at the base);
- no open order names this unit's files.

Nothing of Kimi's or DeepSeek's is edited, closed or pushed to.

## Cancel probe (run first)

- **Does an item already cover this?** Search `docs/roadmap/items/` on `origin/main` for the kernel list, `activation_quantize16`, `q16_residual` and fusion.
- **Does an open order hold any file in the new item's scope?** Check every open PR's file list.
- **Are the scope's paths real?** Check each named file with `git cat-file -e origin/main:<path>`; a glob may name files a later unit creates.
- **Classed:** instrument. Nothing cancels the unit unless an item already covers it.

## Files in scope

- `docs/roadmap/items/float-kernel-fusion.md`
- `docs/roadmap/issues.json`
- `docs/roadmap/README.md`
- `docs/loop/tasks/infra-loop-harness/claude-280/**`

## Done when

1. **The item** is on the branch with the six required sections, `status: ready`, `needs_gpu: true` and a `scope:` naming only the files its units edit.
2. **`cargo xtask roadmap check`** passes, and `roadmap render --check` reports the render current.
3. **Its issue exists**, opened by `roadmap sync --apply` from this worktree, and `issues.json` maps the id to it.
4. **A fresh exact-head review passes.** It checks every line number and count the item cites against `origin/main`, and its scope against the work its done-when needs.
5. **`loop report --check` and `loop pr-check --local`** pass.

## Stop and decide (never wait) if

- the review finds a cited count or line wrong. Then fix the item before the issue is opened, or edit the issue body with the next sync;
- another open order holds a file in the scope. Then narrow the scope, or order the units.

## Cancel probe result (4 Nov, about 20:40Z)

- **No item covers it.** On `origin/main`, only `served-rounding-cost` (#3594) names `activation_quantize16`, and as a source: its done-when repairs the served graph's measured cost, the export rounding first. No item names `KERNEL_ENTRY_POINTS`, `EXPECTED_KERNELS` or a fused kernel. The new item names #3594 and leaves the export rounding to it.
- **No open order holds a file in the scope.** The only other open PR, Kimi's #3675, changes none of them.
- **The scope's paths are real.** Each named file exists on `origin/main`. `fused_quantize*.rs`, `tests/fused_*.rs`, the prep sheet and the task folder are globs for files the item's units create.
- **A fresh fact-check** of the draft item against `origin/main` read REQUEST CHANGES with seven fixes. Every one is applied before the first commit of the item:
  - the dispatch check also scans `spiralcore_fbs_arms.cu`, whose two kernels launch through `launch_prebuilt`;
  - `build.rs`'s line ranges;
  - claude-157 shows a host-bound step, not a launch-bound one;
  - the flag's home: a thread-local in `hamilton_sheet.rs`, set from `train-in-space.rs`, added to the scope;
  - `act_t` lives at `cuda_stack.cu:61-85`;
  - the PTX load is a check too;
  - the literal 26 in doc comments and messages.

  A correction of my own while applying them, "once a layer, not twice", was wrong. The exact-head review restored "twice a layer", at `:2109` and `:1757` (REPORT, What did not work).
- **The sync's plan:** it creates this issue, and it re-renders five generated bodies from their merged item files: #3506, #3440, #3407, #3608 and #3392. Each carries the "Edit that file in a PR, not this issue" footer, so applying changes no hand-written text.
- **Classed:** instrument. Nothing cancels the unit.
