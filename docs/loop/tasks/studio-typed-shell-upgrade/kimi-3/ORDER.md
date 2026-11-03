# ORDER: deliver the live Studio IDE experience

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500), continuing the owner-approved goal of a polished IDE-harness-style AI Studio with working non-model workflows. Base: `aa2fa4da148439e2954ff004abe5d8c0301fd676`.

## Goal

Refine the existing live Studio into a coherent, readable IDE experience. Make source readiness honest, expose a safe demo workspace, and make mobile navigation usable. Preserve the engine and all non-model client workflows. Do not claim or simulate a serving conversational model.

## Design and plan

- [Design](../../../../superpowers/specs/2026-11-02-studio-ide-experience-design.md)
- [Implementation plan](../../../../superpowers/plans/2026-11-02-studio-ide-experience.md)

The owner directed the goal and told the lead to proceed. The chosen approach is to refine the live shell rather than cut over the typed library or add a framework.

## Files in scope

- `web/studio/index.html`, `web/studio/index.css`, and `web/studio/README.md`.
- Narrow UI behavior in `web/studio/assets/js/studio_ui.js` and `web/studio/assets/js/studio_files.js`.
- Focused actual-page/module tests under `web/studio/src/adapters/**` and component tests under `web/studio/src/components/**`.
- The design, plan, this task folder, `docs/products/web-client.md`, and the owning roadmap item/generated issue data if the normal generator permits it.

Approximate implementation budget: 800 lines across production and focused tests, excluding the design/plan/ORDER/REPORT prose. Split at the plan's task boundaries if this cannot remain coherent.

## Done when

- No-source, research-only, and discrete-M1 chat are disabled consistently in the button and shared send path; positively synthetic non-discrete continuation and valid HTTP(S) API configuration remain available with honest labels.
- Unavailable, API-ready, and local-ready empty states are distinct and useful, with quick actions including Explore demo workspace.
- Demo loading never discards dirty work, restores/labels demo state coherently, and does not fabricate dirty edits.
- The existing shell has a polished IDE visual hierarchy in dark/light themes and desktop/mobile layouts. Mobile navigation closes via scrim, section selection, and Escape.
- Focused failing-first tests and complete Studio checks pass. Real built-in-browser checks cover all four views, both themes, navigation, and the demo workflow without real user files, network, or a model request.
- Repository gates, claim wording, local policy, fresh exact-head review, green GitHub checks, and protected queue delivery pass. PR uses `References #3500`; #3500 remains open for parity/migration/cutover.

## Stop and report conditions

Stop and report before changing any file outside the listed scope, or if the work would alter the engine, worker, tokenizer, `studio_api.js`, model artifacts, acceptance bars, real browser data, publisher, deployment, `.github/**`, Claude's active work, or owner-controlled paths. Do not weaken a verifier, add a dependency, use real credentials/files, or claim a real model/API/disk trajectory. Stop after the declared visual/non-model deliverable is verified; do not start typed-library cutover.
