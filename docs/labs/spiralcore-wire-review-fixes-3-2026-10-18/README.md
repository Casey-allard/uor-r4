# Wire side-exploration review fixes, round 3  2026-10-18

Third follow-up to the wire side-exploration review: Codex reviewed #2619 at exact head `5d6ecdd62` and
reported one further P1.

## Finding and fix

`wire_interposer.c` (P1: avoid blocking reused connects on the old receive). When one thread is blocked
in `recv` and another closes the socket, Linux can leave that receive asleep on the old underlying file,
so a later `connect` on the reused descriptor would wait forever on `io_lock`. `close` now issues
`shutdown(fd, SHUT_RDWR)` after marking the fd inactive and before `sys_close`, waking the blocked
receive; the reused-fd connect wait is then bounded. Buffers are still reset on the next connect.

## Verification

    WIRE_INTERPOSER_PASS        macOS interposer smoke (rc=0)
    WIRE_INTERPOSER_LINUX_PASS  Linux smoke in the mvm guest, scoped workspace

No model compute; side exploration only. Claim: `deepseek/spiralcore-wire-review-fixes-3`, lab deepseek.

## Limits

Concurrent recv on one fd remains unsupported. close defers per-fd buffer reset to the next connect.

## Next

Exact-head review, protected merge, branch/worktree cleanup and hygiene. PR: https://github.com/UOR-Foundation/uor-r4/pull/2621

## Claims cleanup (2026-10-18)

All merged wire side-exploration branch claims (side-exploration, review-fixes, review-fixes-2, review-fixes-3, and this cleanup branch) were removed from docs/harness/claims.json. No code, model or evidence change.
