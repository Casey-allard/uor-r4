# #1828 — pod tooling: fix the three things that cost an hour running the Phase 2b gate

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T04:39:28Z
- **Updated:** 2026-10-09T13:20:22Z
- **Closed:** 2026-10-07T04:40:42Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1828
- **Merged:** 2026-10-07T04:40:42Z
- **Merge commit:** fcc5c9a97fa8e6abba5f2169d8dd86886ae0efa3
- **Base / Head:** main / UOR-Foundation:deepseek/gate-tooling
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Running the frozen Phase 2b gate cost an hour in avoidable failures. Each of these is now fixed in the tool, not in a session's shell history.

## `scripts/pod/bf16-phase2b-gate.sh`

**1. `REPO` defaulted to `/root/ds-uor-r4`, which does not exist on a bootstrapped pod** — the checkout is `/root/build/src-<sha>/`. Result: `line 68: cd: /root/ds-uor-r4: No such file or directory`. Now resolved: `/root/ds-uor-r4` if present, else the single `/root/build/src-*`, else fail naming both.

**2. `cargo` is not on PATH for a non-interactive ssh shell** — `line 68: cargo: command not found`. Now exports `RUSTUP_HOME`/`CARGO_HOME` and prepends `/root/.cargo/bin` and `$CUDA_DIR/bin`.

**3. The shared corpora are not staged into `$D` by bootstrap.** They live as tars on the canonical volume, so `/root/data` is empty and the gate cannot find its inputs. Now extracts `step5-inputs.tar` and `tokenizer.json.tar` from `${VOL_DATA:-/workspace/uor-r4/data}` on first use, then **checks all five inputs** it reads and fails with the expected archive names if any is missing.

## `scripts/pod/uor-pod` — `up`

**A healthy draw is SSH-ready in ~20–40 s; a stalled one never becomes ready.** The stalled pod sits at `runtimeStatus: initializing` / `awaiting_container` — the host was rented but the container never started — and no amount of waiting fixes it; deleting and redrawing does. Against the 900 s SSH wait, **four such draws cost an hour on 2026-10-07**, and the ledger shows the same failure for Codex and Claude.

Now detects the stall at `UOR_POD_STALL_SECONDS` (default **180**) — `runtimeStatus=initializing` or unknown, with no SSH address — and fails fast so the redraw is cheap. A draw that comes up normally is unaffected; the existing 900 s deadline and `up_failed` cleanup are unchanged.

## Checks

- `bash -n` on both scripts — clean.
- The three gate-script fixes were each hit and then worked around by hand in a real run: with all three applied the gate completed four 70,609-step runs and produced the measurement now recorded in `docs/compute/bf16-phase2b-gate-2.md` (PR #1827).
- The `uor-pod` change only adds an early-exit branch; it does not alter the success path.

References #820.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

