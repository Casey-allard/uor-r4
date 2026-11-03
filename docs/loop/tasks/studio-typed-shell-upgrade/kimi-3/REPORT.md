# REPORT: deliver the live Studio IDE experience

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Design: [design](../../../../superpowers/specs/2026-11-02-studio-ide-experience-design.md). Plan: [plan](../../../../superpowers/plans/2026-11-02-studio-ide-experience.md). Base: `aa2fa4da148439e2954ff004abe5d8c0301fd676`. Exact delivery-head commit/tree evidence is posted on the PR.

## What did not work

- The initial desktop was serviceable but had weak hierarchy and a marketing-like Agent empty state. The new welcome panel presents useful source status and actions without implying a serving model.
- No-source and discrete-M1 Send appeared available. One shared readiness predicate now disables the button and shared send path, while supported non-discrete synthetic continuation and valid HTTP(S) API configuration remain available.
- Demo workspace existed only behind a hash/public API, could replace dirty work, restored as an unlabeled partial state, and fabricated a dirty edit. The shared demo path now refuses dirty work, restores/labels demo state, and never fabricates unsaved edits.
- Initial phone rendering put quick actions behind the composer and squeezed attachments into a vertical strip. Mobile quick actions now fit in two rows and the composer stacks full-width controls and attachments.
- The first full suite exposed an older page-hook fixture missing the new `chatReadiness` function. Including it restored both actual-page tests; no production behavior was weakened to accommodate the fixture.
- Policy initially reported only the required REPORT missing. Other checks were advisories; the report now exists.
- **The first fresh review returned CHANGES.** API readiness accepted malformed/non-HTTP URLs and implied connectivity, demo-created tabs lost demo mode/tree state, mobile scrim/inert state missed breakpoint changes, quick actions left stale section state, and source edits were described as deployed. Seven focused regressions failed first. The page now reuses HTTP(S) validation, says API configured without a reachability claim, restores demo tabs/tree, synchronizes breakpoints and quick-action sections, and labels these as source changes.
- A later full-suite fixture also needed the existing URL validator injected; that test-only binding was corrected without weakening readiness. An empty restored demo tab has explicit tree/content/mode coverage, while a real-browser reload preserved a non-empty demo buffer and seven-file tree.
- Rebase onto merged #3541 initially exposed overlapping duplicated history. Reducing integration to the six visual-only commits left two live-page merges and two test-fixture merges. The production merge preserves both chat readiness and request-correlated rollback. Four combined-suite failures were fixture integration gaps: readiness helpers, current send state, API message extraction, and a chat-capable rollback artifact were added or corrected without changing assertions into approval of a regression.
- Browser DOM snapshots missed the visible Settings modal during cleanup. A direct visual capture confirmed the real modal and its three demo tabs/one unsaved draft; Clear draft visibly restored “No open tabs.” This was an automation-reading limitation, not a product repair.
- **The second fresh review returned CHANGES.** The rebase had dropped the design/plan and left broken relative links, drawer Escape preempted modals/palette, non-synthetic FBS stacks looked generation-ready, the SDD-record wording was unsupported, and destination dismissal left stale expanded state on reload. Three focused regressions failed first. Restored records and corrected links now match scope; local continuation requires `caps.synthetic === true`; visible modals/palette close before the drawer; destination dismissal persists collapsed; and unsupported ledger wording is removed. Repeated test-module imports also installed duplicate keydown listeners, which the navigation harness now removes between tests.
- No real model, real API response, real user file, real disk access, production publish, or typed-library cutover is claimed. Real File System Access and full typed-library parity/migration remain later units.

## What changed

- Added shared source readiness for no source, research-only artifacts, discrete M1, positively synthetic non-discrete continuation, and valid HTTP(S) API configuration. Blocked sends return false and direct users to Models; API configuration does not prove reachability.
- Replaced the generic Agent empty state with distinct unavailable, API-ready, and local-ready welcome panels and quick actions: Open Models, Open Workspace, Explore demo workspace, and New session.
- Exposed the existing demo tree through the Agent action. Dirty tabs block replacement; demo tabs restore as demo with topbar/status/inspector labels, and no automatic dirty timer remains.
- Added one labeled mobile scrim, synchronized `aria-expanded` and main-content inertness, persisted destination dismissal, and Escape handling after any open modal or palette.
- Applied a restrained IDE visual system to the existing shell: clearer panel surfaces, readable type, stronger active states, useful welcome cards, editor tabs, model cards, dialogs, status fields, and responsive layouts in dark/light themes.

No engine, worker, tokenizer, `studio_api.js`, model artifact, publisher, deployment, real browser-data migration, external asset, runtime dependency, or `.github/**` change was made. The typed library remains unmounted; this is a live legacy-page visual/non-model unit, not a typed cutover. The branch is rebased onto #3541 and preserves its generation, provider, and transactional-send safeguards.

## Measurements

- **213 Vitest tests pass across 25 files**, including 9 live Agent readiness/empty-state tests, 5 actual demo workspace tests, and 3 actual mobile navigation tests. **12 Node model-source tests pass. 225 tests total, 0 failures.**
- `npm run typecheck`: PASS, no TypeScript errors.
- `npm run build`: PASS; `dist/studio-foundation.js` 123.07 kB (gzip 33.77 kB) and `dist/studio-foundation.css` 3.57 kB (gzip 1.14 kB), uncommitted.
- `node verify_studio_page.mjs --strict-router`: PASS, both legacy strict pages clean. Modified `studio_ui.js` and `studio_files.js` syntax checks pass; `model_sources.js` also passes the integration syntax check. `node verify_studio_page_load.mjs --strict-router`: PASS, including the planted external-request self-test; no external requests from either page.
- Repository gates: 11/11 GREEN at the pre-integration visual head and rerun at the exact delivery head. Claim wording and local policy PASS with disclosed D76 scope/ORDER/FILEMAP advisories.

## Real browser evidence

Built-in browser checks exercised the running source page after rebase onto #3541, not only component fixtures or the deployed site:

- Desktop 1440×900: Agent welcome, Workspace, Models, Guide, demo tree, both editor tabs, and no-source Send disabled. The six-file demo and saved state appeared without fabricated dirty edits.
- Mobile 393×852: all four quick actions fit above the composer and controls remain full width. The drawer opens with a dimmed backdrop and closes via scrim, Escape, and destination selection. After the review fixes, a live Settings/drawer check closed Settings on the first Escape while leaving the drawer open, then closed the drawer on the second; selecting a destination and reloading left the menu collapsed in the new document generation.
- Dark and light themes preserve readable contrast across the checked surfaces.
- Demo reload: created `untitled.md`, entered a 10-character buffer, reloaded, and reopened Workspace. The unsaved tab/content, demo labels, and seven-file tree including `untitled.md` were preserved. The reload tool timed out while navigation settled; the new document generation and restored content were verified directly rather than inferred from the tool result.
- The direct `#settings` route visibly opened Settings with three demo tabs and one unsaved draft. Clear draft visibly restored “No open tabs.” Browser DOM/element snapshots missed the visible modal; visual capture and the actual Clear draft result are the evidence. Earlier visual-head checks also covered demo attachment and a dummy API-configured state without contacting an endpoint.

These are source-page visual/non-model checks, not model, API, real-disk, production-publish, accessibility-certification, or typed-library parity claims.

## Checks run at the head

Focused tests cover readiness, demo guard/restoration, and navigation. The second exact-head review's readiness, modal/palette Escape, and persisted-navigation regressions were run failing-first before their repairs. Full Studio checks, repository gates, claim wording, local policy, exact-head review, and green GitHub checks precede queueing. PR uses `References #3500`; #3500 remains open for later parity/migration/cutover.

## Skills used

Brainstorming, design and implementation planning, isolated worktrees, TDD, systematic debugging, modern web guidance, verification before completion, and independent exact-head delivery review. No model compute, paid resource, real credential, or real user data was used.

## Still open

This delivery does not provide real model/API/disk trajectories or mount the typed component library. Production publication and full typed-library parity/migration remain later units; #3500 stays open.

## Plain summary

The Studio source page now looks and behaves like a coherent IDE harness: navigation and sessions at left, useful work in the center, context at right, and command/status chrome around it. Users can explore and edit a demo workspace before a serving model exists. Chat availability is honest, the demo is safe, and desktop/mobile presentation is verified. No serving-model capability is claimed.
