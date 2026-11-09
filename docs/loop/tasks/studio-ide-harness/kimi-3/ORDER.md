# ORDER: make normal launch open the recovered workbench, kimi-3

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Base: `4447a28321f78f313805cdde624f3079d3e26577`, with predecessor PR #3879 still separate.

## Observed defect and decision review

The owner reported that every launch opens the old green app. The running process proved this: an earlier Kimi command opened `local/kimi/studio-shared-shell-recovered/UOR Harness.app` with identity `org.uor.r4.desktop`, not the recovered `org.uor.r4.desktop.cowork`. A direct launch of the recovered executable then displayed the actual neutral workbench. The defect is the launch path, not permission to return to the old design. Provide one canonical launcher rather than relying on an agent to choose a historical bundle path.

## Files in scope

- `scripts/studio/open-cowork.sh`: prepare the current typed frontend, build with mandatory `gui`, verify the cowork bundle identity, and open that exact app.
- `scripts/tests/test_typed_runtime.py`: launcher regression contract.
- `web/studio/README.md` and `docs/products/desktop-client.md`: the canonical local launch command and its development-only limits.
- `docs/loop/tasks/studio-ide-harness/kimi-3/**` and ignored launch evidence under `local/kimi/harness-parity/`.

No R4 API/model/engine/worker/tokenizer change, old-state deletion, installation into `/Applications`, production publish, signed release, or claim of full agent parity. Do not touch the predecessor branch's files here.

## Done when

- A test fails because the canonical launcher is absent, then verifies mandatory GUI build, current frontend preparation, exact cowork identity, and launch of that bundle without `open -g` hiding the window.
- The launcher runs successfully and the actual native window is inspected for neutral workbench landmarks. The default target is `target/cowork` unless `CARGO_TARGET_DIR` is explicitly supplied. `UOR_COWORK_NO_OPEN=1` supports build-only verification.
- Existing typed/runtime checks pass, evidence and REPORT are exact about the live window and unsigned development build, and this repair is included in the protected workbench delivery after the predecessor merges.

## Stop and report

Stop before launching the old bundle, hiding the new window, overwriting the rejected app's saved state, installing outside the repository, or claiming the whole application is functionally complete. Preserve the old app and all evidence; stop the exact test process only when cleanup is required.
