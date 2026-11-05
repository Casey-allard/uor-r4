# ORDER: close remaining guard and record gaps

Lab: kimi. Item: [infra-desktop-client-tauri](../../../../roadmap/items/infra-desktop-client-tauri.md) (#2826). Base: `f6503bf38e1b9c9d4900d882955143cc34c081bc`.

## Prior units

Implementation and evidence are complete through `f6503bf38`. Fresh reviewers confirmed the major repairs but found attached Git override forms, output-cap mislabeled as timeout, remaining runtime symlinks, and a dropped prior design record. No protected merge has occurred.

## Goal

Close the last execution/provenance bypasses without changing the product scope, preserve accurate timeout versus truncation results, and restore all prior-unit records while moving current evidence to kimi-5.

## Files in scope

- `src-tauri/**` and focused tests.
- `scripts/studio/**`, `scripts/tests/**`, and `scripts/stage-studio-site.sh` for runtime symlink/provenance checks.
- `docs/products/**`, `docs/labs/infra-desktop-client-tauri/**`, `docs/research/notebook/m4-serving.md`, `docs/roadmap/items/infra-desktop-client-tauri.md`, and this task folder.
- `docs/loop/tasks/infra-desktop-client-tauri/kimi-3/DESIGN.md` only to restore the prior record byte-identically; `docs/loop/tasks/infra-desktop-client-tauri/kimi-4/DESIGN.md` for current repair design/evidence movement to `kimi-5`.

No other paths are authorized. Retired current-state and old out-of-scope SVGs stay unchanged.

## Required repairs

1. Refuse attached Git `-c<name>=<value>` and `-C<path>` forms as well as separated/long overrides; add execution negatives for both. Refuse every runtime symlink, including `pkg/`, with direct pkg-root and extra-link negatives.
2. Track the termination reason: output-cap truncation must not set the wall-clock timeout flag or print the timeout message. Preserve direct-exit bounded abandonment and add a cap-versus-deadline regression.
3. Restore `kimi-3/DESIGN.md` byte-identically from its prior head. Move current evidence to kimi-5, correct the notebook unit, and bind implementation and final review heads explicitly.
4. State repository-root context in the lab command record.

## Done when

All new regressions fail first for their intended reason; focused and full browser/Rust/runtime checks pass at the final head; rebuilt app stream and normal exit pass; both fresh reviewers approve the exact head; required GitHub checks, protected merge, fresh-main verification and hosted publication are observed. Regenerable outputs are cleaned after verification.

## Stop and report conditions

No useful-conversation, visual-understanding, production-readiness, M3/M7/M8, exact hidden-state resume, signed-distribution, forced-kill-cleanup or GitHub-auth claims. No new dependencies, arbitrary execution/file access, public tunnels, secrets, committed binaries or paid compute. Do not drop prior records or weaken any existing guard. Stop and re-scope before touching another path.
