# ORDER: publish the neutral workbench through the typed Studio path, kimi-11

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Base: `59494646cdae3e2f8874939680324ed8cd5affd5`, tree `907f341c57da00fb03b839da54d2fb2985bcd3a7`; this is merged `origin/main` after #3890. The owner reported that the public WebUI had not changed for days. Investigation confirmed that `gh-pages` last carried a Studio build from 7 November, while merged Studio work continued on main. The existing manual publisher still stages the legacy page, so merely running it would not deliver the neutral workbench.

## Finding and decision review

- `docs-site.yml` publishes only gh-pages-preview on main pushes; live gh-pages requires a separate manual action. The older deploy workflow is intentionally paused.
- `build-and-publish.sh` and `stage-studio-site.sh` preserve and stage the legacy source by default. The reviewed typed application already has a dist/typed allowlist, catalog and browser acceptance.
- Choose an opt-in `--typed` path over changing the default or manually copying dist. It reuses the current artifact build, typed allowlist, catalog, dry-run/live guard, Pages limits and rollback behavior while leaving legacy publishing unchanged.
- No direct main push, `.github/**` edit, ad-hoc gh-pages commit, production agent-service claim, native terminal claim, or live R4 API claim. Static web limitations remain explicit.

## Files in scope

- `scripts/stage-studio-site.sh`, `scripts/studio/build-and-publish.sh`, `scripts/tests/test_typed_runtime.py`, and related existing runtime tests only if a concrete regression requires them.
- `docs/superpowers/specs/2026-11-09-typed-studio-live-publish-design.md`, `docs/superpowers/plans/2026-11-09-typed-studio-live-publish.md`, this task folder, and current item/state/product publication records where behavior changes.
- Exact-head logs, staged browser evidence, publish/rollback identities and fresh-main/public deployment evidence under local Kimi evidence directories.

No model/API/engine/worker/tokenizer implementation change, dependency change, native consent change, public terminal service, real credential/profile, or deletion of prior site/evidence/other labs' work.

## Done when

- Red-first tests demonstrate that the current publisher lacks typed staging and opt-in publisher wiring. Minimal implementation passes those tests and all existing legacy/runtime tests.
- `--typed` builds the generated artifacts before Vite, stages only dist through the typed allowlist, regenerates the catalog, and preserves default legacy behavior, dry-run, main-only live guard and rollback output.
- Focused and complete frontend/browser, runtime, desktop and repository checks pass. A dry-run staged root mounts the neutral workbench in a real browser at desktop/mobile sizes with every navigation surface, zero external/raw-TypeScript/runtime errors and explicit static-terminal unavailability. No remote ref changes during dry run.
- Fresh exact-head review, required checks and protected merge precede fresh-main publication. From exact `origin/main`, a typed dry run is browser-verified before the reviewed local gh-pages commit is pushed through the publisher's printed path. Pages completes and a hard-reloaded public URL shows the neutral workbench.
- Source/staged/published/rollback identities and evidence are preserved before this branch/worktree and reproducible outputs are cleaned. Full agent runtime, hosted native workflows and live R4 API remain open.
