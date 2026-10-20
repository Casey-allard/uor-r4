# FETCH batch opt-in for high-latency pulls

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-batch.

## What changed

- `objpeer pull --batch N` (and library `pull_selected_with_batch`) sets the FETCH batch size explicitly,
  clamped to 1..=65535. The default remains the derived batch (4096 for default limits), so local pulls
  keep their current behavior.
- PROTOCOL.md records the default and the opt-in cap (wire format unchanged; the frame count field already
  allows 65535 ids).
- New test `packaged_cli_pull_batch_flag_pulls_exactly`; package 58 passed / 0 failed, Clippy/fmt clean.

## Measurement (WAN, Vast pod, same 52,535,088 B synthetic random artifact)

- default batch: 236.19 s, 143,659,164 wire bytes, 2,462,578 nodes.
- `--batch 32768`: 91.50 s, 143,651,786 wire bytes, same nodes. **2.58x faster**, same payload.

## Why

Cold pull was round-trip bound: 2,462,578 nodes / 4096 ids per FETCH = 602 sequential round trips. The
opt-in raises the window only where the caller knows the link is high-latency.

## Limit

Large batches are not free locally: local cold pull measured 37.4 s at 32768 vs 18.8 s at the default
4096 (larger frames cost more locally), which is exactly why this is opt-in and not the default.
