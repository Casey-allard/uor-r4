# Pipelined manifest requests for fleet verification

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-verify-pipeline. Follows #2592.

## What changed

- verify_series pipelines manifest requests in bounded windows of 64: it writes the whole window, then
  reads the responses in order. A batch therefore costs about one round trip per 64 checks instead of
  one round trip per check, with the same single DONE at the end.
- The window bounds both directions so a large batch cannot deadlock by filling the request and response
  socket buffers at once; it is also the natural place to tune throughput later.
- An unknown selector still yields an error frame and keeps the session open, and later checks in the
  same window are still verified in order.

## Verification

- verify_series_continues_after_an_unknown_selector: known / unknown / known in one window yields
  Verified, Unknown, Verified in order.
- verify_series_pipelines_more_than_one_window: 100 checks of one object (two windows) all Verified.
- Existing series, stale and CLI batch tests unchanged and passing. Unit 7/7, peer 14/14, objpeer_cli
  6/6, controls 10/10, showcase 5/5; Clippy -D warnings and fmt clean.

## Why it matters

The cross-machine batch pass took 5.819 s for 20 checks because each check waited a round trip (~291 ms).
Pipelining should collapse a 20-check batch to roughly one round trip plus local verification work,
while the wire cost is unchanged (about 221 B per check). Local and cross-machine measurements follow.
