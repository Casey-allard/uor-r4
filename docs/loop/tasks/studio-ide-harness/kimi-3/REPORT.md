# REPORT: canonical launch for the recovered cowork workbench, kimi-3

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Order: [ORDER.md](ORDER.md). Base: `4447a28321f78f313805cdde624f3079d3e26577`, with predecessor PR #3879 delivered separately.

## What did not work

- The owner reported that every launch opened the old green application. The actual process was `org.uor.r4.desktop`, started by an earlier Kimi command from `local/kimi/studio-shared-shell-recovered/UOR Harness.app`. It was not the recovered `org.uor.r4.desktop.cowork` application.
- `open -n -g` started the cowork identity but left its window off-screen. Directly executing the recovered binary then displayed the actual neutral workbench: project/session navigation, compact composer, environment/project/permission/model controls, and pane controls. That test-owned process was closed intentionally before rebuilding; it did not crash.
- The new launcher regression initially failed because `scripts/studio/open-cowork.sh` did not exist. The launcher and its contract were then added; the test now passes **6/6**.
- Full Cowork functionality remains open: persistent agent runtime, parallel tasks, project memory, scheduling/Dispatch, browser automation, and R4 API integration. This launch repair does not establish those capabilities, a production publish, a signed release, or useful model behavior.
- The canonical launcher was verified end to end: it built and directly opened the actual neutral workbench, with executable SHA-256 `1d7f7900a0546c9e30ad20f5724af81918bf82c05a9f01cc3288180f35e40297`. The application remains an unsigned development build, with no `/Applications` installation and no deletion of the old app or its saved state.

## What changed

- Added `scripts/studio/open-cowork.sh`: it prepares the current typed frontend, builds with `--features gui`, verifies `org.uor.r4.desktop.cowork`, and directly executes that bundle's `uor-desktop-client` so the window is visible. Launch Services (`open -n` and `open -n -g`) left the new window off-screen in this environment and is not used.
- The default target is `target/cowork`; `CARGO_TARGET_DIR` supplies an explicit alternative and `UOR_COWORK_NO_OPEN=1` selects build-only verification. `--check` prints the intended target, app, identity, and frontend without building or opening.
- Added a regression covering mandatory GUI compilation, identity, default target, visible launch, and rejection of the historical recovered-shell path. Updated both desktop launch documents.

## Measurements

- Direct native observation before the rebuild confirmed the neutral workbench at `org.uor.r4.desktop.cowork`, including Agent/Workspace/Changes/Terminal/Models/Guide, session controls, Chat/Editor/Diff/Terminal/Browser, and Split.
- `python3 scripts/tests/test_typed_runtime.py`: **6/6**, after the launcher-missing failure was corrected by implementing the launcher.
- `bash scripts/studio/open-cowork.sh --check`: prints `target/cowork`, the corresponding `UOR Harness.app`, `org.uor.r4.desktop.cowork`, and `web/studio/dist` without side effects.
- The canonical script built and directly opened `org.uor.r4.desktop.cowork` with executable SHA-256 `1d7f7900a0546c9e30ad20f5724af81918bf82c05a9f01cc3288180f35e40297`. The actual window is visible with the neutral theme, project/session navigation, compact composer, and pane controls. Evidence: `local/kimi/harness-parity/canonical-launch.jpg` (SHA-256 `25e570b3ca2b44c886135527cf98e95cc389502e1e731789ccbcdc929d4ef009`).

## Checks run at the head

- `python3 scripts/tests/test_typed_runtime.py`: **6/6**.
- `bash scripts/studio/open-cowork.sh --check`: correct target, app, identity, and frontend; no side effects.
- `bash -n scripts/studio/open-cowork.sh`: PASS.
- `CARGO_TARGET_DIR=<generated-target> bash scripts/studio/open-cowork.sh`: frontend preparation and GUI bundle PASS; actual direct native window inspected at `tauri://localhost/index.html?typed=1` with the neutral workbench landmarks. No code-signature or production-release claim follows.
- Final workbench typed/native checks, exact-head review and protected delivery remain separate.

## Skills used

Systematic launch-path diagnosis, actual native-window inspection, red-first launcher regression testing, and verification before completion. No model/API implementation or visual redesign was used for this repair.

## Still open

Finish the predecessor's separate review delivery; merge the resulting `main`; run the workbench's exact-head review and protected delivery. Full agent/runtime parity and hosted production are subsequent units.

## Plain summary

The old launches selected the wrong application bundle. A single launcher now builds and directly opens the recovered neutral workbench and its distinct saved state, with the actual window verified.
