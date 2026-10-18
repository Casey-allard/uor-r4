# Wire route counter aggregation  2026-10-18

Fixes the original #2613 P2 review finding in `spiralcore-base/wire/tools/wire_socket.py`.

## Defect

`WireGateway` buckets are persistent per route: `connections`, `plain_in`, `plain_out` and `wire`
accumulate across completed connections. The connection-completion path accumulated plain bytes but
assigned the wire total:

```python
bucket["wire"] = wire.stats()["wire"]
```

A reused route therefore reported only the most recently completed connection's wire bytes while its
plain bytes covered every connection, progressively understating the documented per-route ratio and
misreporting live traffic. The finding was raised as P2 on PR #2613 (line 346).

## Change

- `spiralcore-base/wire/tools/wire_socket.py`: the completion path now adds the finished connection's
  wire count to the route bucket (`bucket["wire"] += wire.stats()["wire"]`). No other line or behavior
  changes; `wire.stats()` and the snapshot ratio formula are untouched.

## Focused regression

- `spiralcore-base/wire/tools/wire_socket_stats_smoke.py` (new, standard-library-only, existing
  `tools/*_smoke.py` style): one `WireGateway` route to a loopback echo service, two sequential
  `WireSocket` clients with distinct payload sizes. Each client's own `wire.stats()["wire"]` is
  captured; the test then asserts the route bucket's `connections == 2`, `plain_in + plain_out`
  equals both directions of both payloads, `wire` equals the sum of both connections (not the last
  one), and `ratio` equals `round(sum_wire / plain, 4)` while differing from the last-connection-only
  ratio. A `wait_for` on the bucket value is the handler-completion signal, not a fixed sleep.
- Negative control: the same test run against the unmodified base source fails with
  `route wire is not the sum of both connections` and reports only the last connection's `wire`
  bytes (about 2.4 KB) against the accumulated plain count (815232), so the test detects the original
  defect rather than only exercising the path.

## Verification (host-only; no model compute, pods, training or evaluation)

Base `42ab73dd9a6f656c99dfe865c57f53e4bddc8341`, forward-merged with fresh `origin/main`
`58757e219b0e14e17d56fdd01c8d02ac52b4e324` in a merge commit (no rebase), worktree
`.worktrees/kimi-wire-counter-fix`, branch `kimi/wire-counter-aggregation-2026-10-18`, Python 3.9.6.
Commands below run from `spiralcore-base/wire/`:

    python3 tools/wire_socket_routing_smoke.py
      -> WIRE_SOCKET_ROUTING_PASS model_ratio=0.0041 store_ratio=0.0041
         unknown_failed=true model_wire=5276 store_wire=5278
    python3 tools/wire_socket_stats_smoke.py     (three consecutive runs)
      -> WIRE_SOCKET_STATS_PASS connections=2 wire_a=897 wire_b=2460 wire_total=3357
         plain=815232 ratio=0.0041 last_only_ratio=0.0030
      -> WIRE_SOCKET_STATS_PASS ... wire_b=2458 wire_total=3355 ratio=0.0041 ...
      -> WIRE_SOCKET_STATS_PASS ... wire_b=2459 wire_total=3356 ratio=0.0041 ...
    python3 -m py_compile tools/wire_socket.py tools/wire_socket_stats_smoke.py -> PASS
    ~/.cargo/bin/cargo run -q --locked -p uor-r4-claim-wording -- . -> PASS
    git diff --check -> PASS

Wire counts vary by a byte or two between runs (compression scheduling), but the per-run sum and ratio
assertions compare values from the same run; the aggregate ratio (0.0041) is stable and distinct from
the last-connection-only ratio (0.0030).

## Scope and limits

- No protocol, framing, route-table, TLS, mux, adapter, Rust binding, model or dose path is changed.
- The regression is loopback-only and host-load independent in its assertions; it is a focused
  counter test, not a transport-performance measurement or a capability claim.
- `docs/` claims about the wire layer's dormancy and PM ownership are unchanged; this is a bugfix,
  not promotion of the side exploration.

## Delivery state

- Claim: `kimi/wire-counter-aggregation-2026-10-18`, lab `kimi`
  (`docs/harness/claims.json`, registered before mutation).
- Delivery files: the source fix, the focused regression, the branch claim, this record, and the
  newest-first `docs/integration/current-state.md` entry.
- The branch was forward-merged with fresh `origin/main` (`58757e219`) in a merge commit, no rebase;
  the merge preserves the source and test identities (`wire_socket.py` SHA-256
  `0debe0c8c8398b087fe14cec3007aa9df72414f93a66578d776499c638c0ccae`; test SHA-256
  `7a98279fd30ce2c1142b32a32106a7c7b5f0479cd0cb192708b706be9230dcff`).
- A protected PR is open for exact-head review: https://github.com/UOR-Foundation/uor-r4/pull/2651
  (not queued, merged or deleted from this worktree). The PR carries `References #2613` and
  `References #2029`, the actual checks above, and the `$0`/no-compute scope.
