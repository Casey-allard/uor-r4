# Interposer fork-safe close  2026-10-18

Closes the outstanding Codex P1 from the wire side-exploration review: close() called
shutdown(SHUT_RDWR) to wake a blocked reader, which disturbs a socket shared after fork
(a child closing an inherited routed descriptor kills the parent connection).

## Change

- wire_interposer.c no longer issues a socket-wide shutdown from close. It stores
  active=0 and calls sys_close; a reader in read_exact_active observes the flag between
  non-blocking chunks and exits.
- Receive is a loop of non-blocking reads (MSG_DONTWAIT via SYS_recvfrom) with a 50 ms
  poll() readiness wait that re-checks an atomic active flag. Poll success is re-validated;
  POLLNVAL is an error, not a readable event. A partial frame header on EOF is treated as EOF.
- recv, send and shutdown capture a per-fd connection generation and fail with
  ECONNRESET/EPIPE if the descriptor was closed and reconnected while the call was pending,
  so a stale call cannot write onto a new stream.
- active and eof are C11 atomics. A pthread_atfork child handler re-initializes the per-fd
  mutexes and sets a child flag; reset_state then drops pointers instead of freeing them in the
  child (bounded leak) to avoid stale-free windows across a fork taken mid-mutation.
- wire_interposer_fork_test.c (stock binary) covers three hazards: (1) child closes an
  inherited routed descriptor and the parent continues; (2) close wakes a reader blocked in
  recv, and a new connection completes on the reused descriptor while that reader unwinds
  (asserted descriptor reuse and a non-positive recv result); (3) a round trip on the reused fd.
- Both platform smokes build an unsafe variant with WIRE_UNSAFE_CLOSE_SHUTDOWN as a negative
  control: it must exit 7 with post-fork round trip failed, proving the test is sensitive to the
  old shutdown-in-close behaviour.

## Verification (this head)

    WIRE_INTERPOSER_PASS ... fork_ok=true negative_control=true          macOS, rc=0
    WIRE_INTERPOSER_LINUX_PASS ... fork_ok=true negative_control=true    Linux mvm guest, rc=0

## Trade-offs and measured cost

- A routed reader blocked with no data polls every 50 ms (20 wakeups/s per blocked fd).
- Reads are chunked; a poll happens only when a non-blocking read returns EAGAIN.
- This run: 2 MiB text probe interposed wall 16.7 ms (two earlier single runs were 19.0 ms before
  hardening and 28.2 ms after the first poll-only attempt). These are uncontrolled single runs on
  a loaded host; treat them as order-of-magnitude, not a benchmark.
- After close, a reused-fd connect can additionally wait for the current frame decompression
  (bounded CPU), not for further socket I/O.

## Residual limits

- Concurrent recv on one fd remains unsupported.
- Fork safety means: a child may close an inherited routed descriptor and exit without disturbing
  the parent. Parent and child must not both keep using the same inherited routed descriptor (the
  per-process compression state makes that stream unsound).
- There is no atfork prepare handler; the child skip-free flag removes the known double-free windows
  but fork during interposer I/O is supported for the close-and-exit pattern, not arbitrary use.
- The send/shutdown generation guard is a best-effort snapshot at entry; a call preempted before the
  snapshot can still attach to a new connection. Concurrent close and I/O on the same fd is
  unsupported.
- close can still block behind a blocking send that holds the per-fd state lock (pre-existing).

## Next

Protected PR, exact-head review, merge, cleanup and hygiene.
PR: https://github.com/UOR-Foundation/uor-r4/pull/2625

## Claims cleanup (2026-10-18)

After #2625 merged, its branch claim and this cleanup branch claim were removed from docs/harness/claims.json. No code, model or evidence change.
