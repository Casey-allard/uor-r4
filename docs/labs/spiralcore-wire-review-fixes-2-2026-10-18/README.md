# Wire side-exploration review fixes, round 2  2026-10-18

Second follow-up review cycle: Codex reviewed #2614 at exact head `448a9cb4f` and reported three P1
findings plus one P2. This record covers the fixes. No model compute; transport side exploration only.

## Findings and fixes

1. `wire_interposer.c` fd reuse vs receive lock (P1). `connect` now waits on the new `io_lock` before
   resetting a closed fd's state, so a still-blocked reader cannot race the reset when the descriptor
   number is reused; the close path remains non-blocking.
2. `wire_mux.py` registration order (P1). `_ServerStream` no longer starts its reader thread in the
   constructor; `_ServerConnection.run` inserts the stream into the map first and then calls
   `start()`. A target that EOFs immediately can no longer finish before registration.
3. `wire_mux.py` half-close retention (P1). Completion now requires both directions: `input_done`
   (client FIN) and `output_done` (target EOF). The gateway only removes the stream when both are
   true; a target that half-closes its write side keeps the stream registered for client input.
4. `wire_interposer_linux_smoke.sh` default workspace (P2). Uses `printenv` with a fallback so the
   script is safe under `set -u` when `WIRE_SMOKE_WORK` is unset.

## Verification (this worktree)

    WIRE_INTERPOSER_PASS        macOS interposer smoke (rc=0)
    WIRE_MUX_PASS               Python mux smoke, 8 streams
    WIRE_MUX_ROUTING_PASS       per-stream routes and fail-closed unknown
    WIRE_INTERPOSER_LINUX_PASS  Linux smoke inside mvm guest, scoped workspace

Conflict-marker scan clean. Claim: branch `deepseek/spiralcore-wire-review-fixes-2`, lab deepseek.

## Limits

Concurrent recv on one fd remains unsupported (same as before); close defers per-fd buffer reset to the
next connect. Side exploration remains outside the dose/dispatch path.

## Next

Exact-head review, protected merge, branch/worktree cleanup and hygiene. PR: https://github.com/UOR-Foundation/uor-r4/pull/2619
