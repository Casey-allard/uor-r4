# spiralcore-wire

7 records.

| Date | Record | Lab | Outcome | Summary |
|---|---|---|---|---|
| 2026-10-18 | [Interposer fork-safe close  2026-10-18](../../docs/labs/spiralcore-wire-interposer-fork-safe-2026-10-18/README.md) | deepseek | tooling | Fixed the wire-transport interposer so closing a shared socket after fork no longer kills the parent connection; fork and negative-control smoke tests pass on macOS and Linux. |
| 2026-10-18 | [Wire side-exploration review fixes, round 2  2026-10-18](../../docs/labs/spiralcore-wire-review-fixes-2-2026-10-18/README.md) | deepseek | tooling | Round-two fixes to the wire transport after Codex review: fd-reuse race, stream registration order and half-close handling repaired; four smoke tests pass, no model compute. |
| 2026-10-18 | [Wire side-exploration review fixes  2026-10-18](../../docs/labs/spiralcore-wire-review-fixes-2026-10-18/README.md) | deepseek | tooling | Fixed four Codex review findings on the merged wire transport (receive lock, stream cleanup, workspace path, committed build outputs); smoke tests pass, no model compute. |
| 2026-10-18 | [Wire side-exploration review fixes, round 3  2026-10-18](../../docs/labs/spiralcore-wire-review-fixes-3-2026-10-18/README.md) | deepseek | tooling | Added a socket shutdown in close so a blocked reader wakes and reused connections do not hang; later found to break fork safety and replaced by the next record. |
| 2026-10-18 | [Wire transport side exploration  2026-10-18](../../docs/labs/spiralcore-wire-side-2026-10-18/README.md) | deepseek | tooling | Imported the standalone SpiralCore wire compression/transport work (Python, Rust, interposer) as a self-contained side exploration with loopback benchmarks; it does not touch the model. |
| 2026-10-18 | [Wire route counter aggregation  2026-10-18](../../docs/labs/wire-counter-aggregation-2026-10-18/README.md) | kimi | tooling | Fixed a one-character SpiralCore wire-layer bug where a reused route reported only its last connection's wire bytes; added a regression test that fails without the fix. |
| 2026-10-18 | [Wire matrix reproducibility and claim correction  2026-10-18](../../docs/labs/wire-matrix-repro-2026-10-18/README.md) | unknown | tooling | The SpiralCore wire test matrix is now reproducible off the owner's machine, the pass count is corrected to 18/18, and claims citing nonexistent verify tools were withdrawn. |
