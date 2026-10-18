# Wire side-exploration review fixes  2026-10-18

Follow-up to merged side exploration #2613 (merge commit `94d4d34a`). Codex reviewed the exact head
`3af1acf` after merge and reported four P1 findings; this record covers their fixes. No model compute,
no training-hold/D11/dose-path change.

## Findings and fixes

1. `wire_interposer.c` held the per-FD lock during blocking `recv`, so a `close`/`shutdown` from another
   thread could block behind a receive. Fixed with a separate `io_lock`: recv serializes on it, while
   `close`/`shutdown` take only the metadata lock and interrupt the socket directly. `close` no longer
   frees receive buffers inline (a concurrent recv may still own them); buffers are reset on the next
   connect for that fd. Concurrent `recv` on the same fd remains unsupported, as before.
2. `wire_mux.py` gateway did not release completed streams (`_ServerStream` completion left entries in
   `_ServerConnection.streams`). Added `finish_stream()` and calls from target-EOF/FIN-completion and
   `close()`.
3. `wire_interposer_linux_smoke.sh` used host `/tmp` for its workspace. Default is now
   `local/wire-interposer-linux` inside the checkout; the mvm wrapper sets `WIRE_SMOKE_WORK` explicitly
   for the guest temp filesystem.
4. Transient `spiralcore-base/wire/local/` build outputs were committed in #2613 (logs, probe binary,
   dylib). Removed from the tree and `spiralcore-base/wire/.gitignore` now ignores `local/`.

## Verification (this worktree)

    WIRE_INTERPOSER_PASS            macOS LD_PRELOAD/DYLD smoke after the lock change
    WIRE_MUX_PASS                   Python mux smoke (8 streams) after stream cleanup
    WIRE_MUX_ROUTING_PASS           per-stream routes and fail-closed unknown after cleanup
    WIRE_INTERPOSER_LINUX_PASS      Linux smoke inside mvm guest with scoped WIRE_SMOKE_WORK

Conflict-marker scan on the edited tools: clean. Claim: branch `deepseek/spiralcore-wire-review-fixes`,
lab deepseek, registered before mutation.

## Limits

- The interposer close path defers per-fd buffer reclamation until the next connect (bounded by
  `MAX_FD`), trading a small retained buffer for no close-vs-recv deadlock.
- Side exploration remains transport-only and outside the dose/dispatch path.

## Next

Exact-head review, protected merge, branch/worktree cleanup and hygiene. PR: https://github.com/UOR-Foundation/uor-r4/pull/2614

## Claims cleanup (2026-10-18)

After #2613 and #2614 merged, their branch entries were removed from docs/harness/claims.json
(including this cleanup branch entry after its own merge). No code, model or evidence change.
