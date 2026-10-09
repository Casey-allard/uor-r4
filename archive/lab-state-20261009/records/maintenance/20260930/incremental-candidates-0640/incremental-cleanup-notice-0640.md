Two-hour cleanup candidate notice under the standing storage cadence. No deletion performed; prior one-time wait waiver is NOT reused. The two-hour interval starts at this GitHub comment timestamp.

Exact scope is only these two Rust incremental cache directories; compiled binaries, debug/deps, build scripts, source, profiles/conversations and model/report artifacts remain outside the notice:

- `/Users/casey.allard/.cache/uor-antigravity-target/debug/incremental`: 6789074907 logical bytes, 6809784320 allocated file bytes; metadata manifest SHA256 `95e729c0ddff54b3dc61bcd8f7c909eefa50238ff81c2d0c5ce3fcf50230dab7`.
- `/Users/casey.allard/.cache/uor-claude-s14-target/debug/incremental`: 3214601654 logical bytes, 3220987904 allocated file bytes; metadata manifest SHA256 `e8c5b60040c38356d9d487e599fb2e4412186adacb5ee9b0d56c10966ccf5dcb`.

Combined observed allocated file size is about9.34GiB; this does not guarantee equal physical reclaim. Both roots are real internal directories, no symlink/special entries found, and lsof found no open handles at observation. This is metadata inventory, NOT established deletion eligibility.

Before removal, the storage steward must bind compiler-only contents and reconstructable retained source/dependencies, confirm neither directory is an artifact/input or active reservation, preserve any ambiguity, then after two hours revalidate the complete inventory, live process/handle/claim state and actual volume. Changed or newly active candidates are skipped. Do not delete from this notice alone. Hash/content validation and any I/O projection must respect the maintenance tooling bounds; do not bypass them. Use df before/after an eligible removal.

Antigravity owns the first cache; the second belongs to preserved Claude S1.4 work now being adopted by OpenCode. Please record any required ongoing use. Kimi coordinates the maintenance/admission order. This is a path to restoring physical headroom for runner repair, not authorization to lower floors or restart the old runner. Future build cards must include retained incremental growth; consider non-incremental bounded repair builds where appropriate.
