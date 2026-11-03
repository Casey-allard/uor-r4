# UOR Studio IDE experience design

## Intent and owner direction

Make the live UOR webclient feel like a polished IDE harness while ensuring its non-model workflows work. There is no serving conversational model yet; the interface must make that limitation useful and explicit rather than pretending chat is available.

## Current state

The live `web/studio/index.html` already has a three-region IDE shell: primary navigation and sessions, Agent/Workspace/Models/Guide views, a context inspector, command palette, editor, and status bar. The typed workflow library is a separate foundation and is not yet live. Desktop appearance is a useful base but overly flat and low-contrast in places. Mobile navigation can cover most of the workspace without a backdrop. The empty Agent view is a marketing-like introduction rather than a useful IDE start page. Send appears available without a usable chat source. A demo workspace exists only behind a hash/public API and is not discoverable.

## Goals

- A coherent, polished IDE-harness experience on the existing live page.
- Clear visual hierarchy, readable typography, compact panels, useful empty states, and consistent dark/light themes.
- Honest non-model behavior: unavailable chat is disabled and directs the user to Models; next-token prediction is not presented as conversation.
- Discoverable demo workspace for evaluating file tree, editor tabs, attachments, and prompt preparation without real user files or a model.
- Mobile navigation that opens deliberately and closes through a backdrop, section selection, or Escape without hiding the main workflow.
- Preserve existing files, sessions, settings, provider configuration, model loading, prediction, and public browser APIs.

## Non-goals and anti-goals

- No serving-model or useful-conversation claim and no engine, worker, tokenizer, `studio_api.js`, or model-artifact change.
- No typed-library cutover, real browser-data migration, publisher, deployment, new framework, runtime dependency, external asset, or `.github/**` change.
- No broad rewrite of storage or model-source behavior and no cosmetic change that breaks existing element IDs or verification contracts.
- No fake terminal output, fabricated agent activity, or hardcoded chat responses.

## Approaches

1. **Refine the existing live shell (chosen).** Keep its DOM, public APIs, and workflows; adjust presentation, empty-state composition, discoverability, and narrow UI-state behavior. Lowest risk and directly improves the page users see.
2. **Cut over the typed component library now.** Rejected for this unit: full real-browser parity and migration are not complete, so this would expand scope and risk rather than deliver a visual upgrade.
3. **Introduce a new frontend framework or separate app.** Rejected: unnecessary dependency and migration work with no benefit over the existing shell.

The owner directed the overall goal and told the lead to proceed; the phased approach is the lead's call.

## Design

### Visual system

Retain warm graphite and ember, with clearer separation among background, panel, elevated panel, and hover/active surfaces. Use readable 12–14px body text, smaller monospace metadata only where useful, consistent 6px radii, restrained borders, and a visible accent reserved for active actions and current selection. Keep light theme contrast equally strong. Use the existing offline font assets only.

The desktop shell keeps top toolbar, left navigation/sessions, central view, right inspector, and bottom status bar. The center gets the strongest hierarchy; the inspector remains secondary. Interactive targets remain at least 28px desktop and 36–44px on touch layouts. Reduced-motion behavior remains intact.

### Agent start page

Replace the marketing-like empty state with an IDE welcome panel:

- Title: build with files, connect an agent when ready.
- A clear status block explaining that conversational serving is not configured; M1 is next-token prediction only.
- Quick actions: Open Models, Open Workspace, Explore demo workspace, and New session.
- A short three-step workflow: choose files, attach context, configure an agent; no fabricated chat content.

When no chat-capable local artifact or valid API configuration exists, disable Send and use its title to explain how to enable it. Do not disable manual editing, attachments, sessions, or next-token prediction.

### Demo workspace

Expose the existing demo workspace through an **Explore demo workspace** action. It loads the existing synthetic tree and switches to Workspace without reading real user files. Users can open tabs, edit buffers, attach files, copy text to Agent, and use existing save/download behavior. Demo state is labeled QA/demo, never disk-backed user work.

### Mobile and responsive behavior

At narrow widths, navigation starts closed unless the user explicitly opens it. Opening shows a labeled backdrop/scrim over the main column, not over the drawer. Clicking the scrim, choosing a destination, or pressing Escape closes the drawer and persists the collapsed state. An open palette or modal consumes Escape before the drawer. ARIA expanded state and main-content inertness remain synchronized. Main content, editor, composer, and status fields must fit without horizontal page overflow.

### Workspace, Models, Guide, and Inspector

Use the same panel/type/spacing language across all views. Workspace emphasizes file tree and tabs; Models separates local prediction from remote chat providers; Guide remains scannable; Inspector presents attached context, current source, and workspace state without inventing activity. Existing IDs and public methods remain available.

## Implementation boundaries

Expected paths: `web/studio/index.html`, `web/studio/index.css`, `web/studio/assets/js/studio_ui.js`, narrow tests under `web/studio/src/**`, `web/studio/README.md`, and this unit's ORDER/REPORT/design/plan records. No other production module is intended.

Prefer editing the existing final design-token/rule sections over adding a competing stylesheet or another app. Keep changes small and behavior-specific; no wholesale formatter run.

## Verification

- Failing-first tests for Send disabled/enabled states, demo action, and navigation dismissal/state.
- Existing Studio Vitest/Node suites, TypeScript, Vite build, JavaScript syntax, and strict legacy page checks.
- Real built-in-browser checks at desktop 1440×900 and mobile phone dimensions: all four views, dark/light themes, no horizontal page overflow, drawer open/scrim/Escape close, and no-source Send disabled.
- Demo workflow in the real browser: Explore demo → open a file/tab → edit buffer → attach it → confirm prompt/context behavior without any model request.
- Repository gates, claim wording, local policy, fresh exact-head review, protected merge queue, and fresh-main verification.

Do not claim real model/API trajectories, real disk access, production deployment, or full typed-library parity from this visual/non-model unit.

## Settled builder decisions

- **API readiness:** a selected provider with a valid HTTP(S) base URL is enough to enable the API chat path. Reachability is not persisted and a passing Test is not required; API keys remain optional for local/no-key endpoints. Existing API-client validation remains authoritative.
- **Local readiness:** a loaded stack with `caps.synthetic === true`, an enabled engine, generation not explicitly disabled, and a non-discrete schema may use the existing local generation path and is labeled synthetic continuation, never useful conversation. Non-synthetic research stacks do not become chat-capable from schema or `engine_supported` alone. Discrete M1 uses next-token prediction and disables chat.
- **Demo safeguards:** the shared demo entry point refuses to replace any workspace with dirty tabs, preserving unsaved work and existing attachments. Clean real folders may be left for the demo. Demo tabs restore as demo mode, the topbar/status identifies demo state, and the old automatic dirty-timer is removed so the demo never fabricates unsaved edits.
- **Conditional empty states:** unavailable, API-ready, and local-generation-ready sessions get different headings/status text and actions. API-ready copy identifies external baseline mode; local-ready copy identifies synthetic continuation. Browser checks cover unavailable and ready states without making a network request.
- **One send gate:** both `updateComposerState` and `sendFromComposer` use the same readiness rule, so the button, Enter, retry, and public `sendPrompt` cannot bypass it. Blocked sends return false and direct the user to Models. The button title explains the blocked reason.
- **Verification fixtures:** ready-state tests use synthetic capabilities/configuration only. No real provider, real model, real user file, or network response is needed.
