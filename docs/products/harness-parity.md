# UOR Harness parity matrix

Target: a UOR-branded Claude Desktop Cowork/Code-style harness using the separately owned R4 model API. This matrix is evidence, not a feature wish list. `NOT_RUN` means exactly that; a control, fixture, or old report never substitutes for the required live workflow.

Reference: [Claude Desktop](https://docs.anthropic.com/en/docs/claude-code/desktop), [Cowork surfaces](https://support.claude.com/en/articles/15520349-use-claude-cowork-on-web-desktop-and-mobile), [projects](https://support.claude.com/en/articles/14116274-organize-your-tasks-with-projects-in-claude-cowork), and [Dispatch](https://support.claude.com/en/articles/13947068-assign-tasks-from-anywhere-in-claude-cowork). The approved design is `docs/superpowers/specs/2026-11-08-claude-cowork-parity-design.md`.

## Cursor hosted-experience target (planning only, 2026-11-09)

The owner's current direction is the Cursor hosted experience, not further Cowork polish. The corrected research report is [Cursor hosted experience](../research/reports/Cursor%20hosted%20experience.md); the design is [2026-11-09-cursor-hosted-studio-upgrade-design.md](../superpowers/specs/2026-11-09-cursor-hosted-studio-upgrade-design.md); the staged executable plan is [2026-11-09-cursor-hosted-studio-upgrade.md](../superpowers/plans/2026-11-09-cursor-hosted-studio-upgrade.md).

This section is a target/plan linkage only. **No row below changes status in this planning unit.** The existing matrix remains the executed-evidence record for the earlier Cowork target and for every current row; the Cursor design's §14 feature-family coverage matrix is the authoritative mapping of Cursor families to stages, planned tests and `NOT_RUN`/`UNVERIFIED` rows. The Cursor work does not claim full parity, hosted production, a real R4 adapter or useful conversation, and it does not promote desktop Agents Window or IDE controls into web behavior.

| Document | Role | Status at this unit |
|---|---|---|
| Cursor report | Corrected public-source research, 2026-11-09 | Reference; authenticated UI unverified |
| Cursor design | Browser-owned task runtime and IndexedDB store; optional tool-only companion; two builder REVISE rounds and agent-94/95 CHANGES corrections applied | **Design only; fresh exact-head delivery review, owner approval and owner acceptance-item choice pending** |
| Cursor plan | 23 tasks in 6 executable stages, plus checkbox-free gated appendices | **Not started; implementation not authorized** |

## Baseline findings, 8 November 2026

- Original typed shell: sparse green/black generic controls; source and retained screenshots confirm the visual regression.
- Published legacy shell: closer desktop conversation layout, but not the full Cowork/Code harness.
- Existing typed state/adapters and native approval boundaries are reusable evidence, not parity proof.
- Original single-generation flow has no persistent agent loop, parallel task runtime, project memory, scheduling, embedded browser, or agent computer-use path.
- Installed Chromium is used for acceptance; no browser binary is downloaded.

| Capability | Required observable acceptance | Fixture evidence | Live evidence | Status |
|---|---|---|---|---|
| Neutral shared shell | Every view matches the reviewed neutral Claude-style hierarchy; no green placeholder theme | Playwright and direct screenshot review across agent/workspace/changes/terminal/models/guide/split | Local Vite app at 1440×900 and 390×852, both themes; native app is separately gated below | **PASS (visual shell only)** |
| Light/dark and responsive | 1440×900 and 390×852, both themes, keyboard/focus and no unintended overflow | Playwright and direct screenshot review across all seven surfaces plus mobile split open/close | Local Vite app; all 45 combined Playwright checks pass | **PASS (browser shell only)** |
| Projects | Create from scratch/existing folder; instructions, files, tasks, and instructions/memory | New-project modal and real folder callback are covered only | Full project model NOT_RUN | NOT_RUN |
| Project memory | View/edit/delete project memory; optional account-memory behavior is explicit | NOT_RUN | NOT_RUN | NOT_RUN |
| Parallel sessions/tasks | Independent project folders, transcripts, event cursors, artifacts, and approvals run concurrently | Independent create/resume is covered only | Concurrent agent tasks NOT_RUN | NOT_RUN |
| Agent plan/tool trace | Real multi-step model/tool events appear in order with usage and errors | NOT_RUN | NOT_RUN | NOT_RUN |
| Stop and redirect | Stop prevents later tools; a redirect changes the next safe action | NOT_RUN | NOT_RUN | NOT_RUN |
| Side questions and child tasks | Side questions preserve the main task; child work fans out and joins with exact identity | NOT_RUN | NOT_RUN | NOT_RUN |
| Permission modes | Environment/project/permission/model selectors change actual authority and can change mid-task | Current local environment, project picker, and ask-before-write presentation are covered only | Full permission modes NOT_RUN | NOT_RUN |
| Editor and files | Searchable tree, tabs, editing, dirty/save, attachments, and artifact preview/download | Existing adapters remain covered; full searchable tree/artifact preview NOT_RUN | NOT_RUN | NOT_RUN |
| Diff review | Real status/diff and per-file/per-line comments survive task and reload identity | Existing bounded status/diff is covered only | Diff comments NOT_RUN | NOT_RUN |
| Guarded terminal | Streaming output, cwd, timeout/output bounds, approval, and cancellation | Real PTY input/output, raw JSON contract, resize, grants, close/reaping, and bounded input/output/sessions are covered; legacy one-shot timeout remains separate | Isolated loopback: `printf`, `read` plus answer, `pwd`, `stty size`, `exit`, and post-close refusal. Native bundle at merged-main source `c66377b8fa77bbd225e52acba6539885dedc0c4a`: first output `NATIVE_FINAL`, exact fixture cwd, `ANSWER:fixture-answer`, size `24 80`, exit 0, and a distinct second-shell approval cancelled with no new shell; executable `8a545748249b867b7f55d71d49f881780ec8ab3b56db18b45506a7f2b064d83a`; `local/kimi/harness-parity/c66377b8fa77bbd225e52acba6539885dedc0c4a/native-acceptance.md`. Earlier `351172842…`, `c67eaa2a5…`, `da6cbaf4…`, `ec49e32b…`, and `975ebff` evidence remains intermediate or historical. No detached-descendant-containment claim | **PARTIAL (development PTY; hosted/full agent integration pending)** |
| Embedded browser | Navigation, screenshot/DOM inspection, preview, isolated profile, and stop | Explicit unavailable state is covered only | Real browser service NOT_RUN | NOT_RUN |
| Computer use | Per-application permission, observable action trace, and immediate stop | NOT_RUN | NOT_RUN | NOT_RUN |
| MCP/connectors/plugins | Discovery and invocation through declared transports with explicit unsupported/error states | Existing bounded remote/native MCP contracts remain covered | Broader connector/plugin parity NOT_RUN | NOT_RUN |
| Artifacts | Model/user-created outputs are inspectable, downloadable, and bound to their task | NOT_RUN | NOT_RUN | NOT_RUN |
| Schedules | Recurring tasks execute without an open browser and notify completion/approval | NOT_RUN | NOT_RUN | NOT_RUN |
| Dispatch/cross-device resume | A second isolated client assigns/resumes the same persistent task and event cursor | NOT_RUN | NOT_RUN | NOT_RUN |
| Environments | Local/hosted and configured remote execution use one contract with truthful privacy state | Local browser environment is labeled; hosted/remote environments NOT_RUN | NOT_RUN | NOT_RUN |
| GitHub PR/CI review | Explicit approved branch/commit/push/PR and CI readback; no direct main push | Existing protected host path remains in place | End-to-end PR/CI review NOT_RUN | NOT_RUN |
| Session export/resume | Exact transcript, project, task, attachments, and event cursor survive restart | Existing session migration and worker parity remain covered | Full task/event resume NOT_RUN | NOT_RUN |
| Static preview honesty | Static Pages clearly disables unsupported service/host actions and never claims full parity | Local dist explicitly marks browser/native/service-only controls unavailable; zero external requests in the existing runtime trap | PR #3903 merged as `4160a5347717d7e6786848895b758bfc5dc663f7`; publish commit `4a435ab8af9c1da6d49225c30f9dda0a4ad50b87`; Pages run `42714479496` succeeded. Fresh-main Playwright **47/47**, direct staged WASM **7/7**, 14 public requests with zero external/raw-TypeScript/runtime errors; plain `/studio/` desktop/mobile neutral workbench verified, both explicit legacy selectors retained, static Terminal explicitly unavailable. Public runtime has one shell, `legacyMcp=0`, `typedMcp=1`, artifact readiness, exact `17 + 25 = 42` and isolated/restored sessions. Rollback head `6540e2717d3a606330dbd141e3642f3f13b87ecf`; evidence `local/kimi/default-cutover/fresh-main/` | **PASS (static default-route preview published; hosted native workflows NOT_RUN)** |
| R4 model API integration | Merged Claude-owned API completes the streaming/tool-call contract with exact identities | NOT_RUN | NOT_RUN | NOT_RUN |
| Packaged desktop | The complete matrix repeats in the unsigned development Tauri app | Packaged resource contract passes; cowork identity and GUI symbols verified | `local/kimi/harness-parity/native-agent.jpg` and `native-split.jpg`: Workspace, Split, and New session pass in the unsigned development app; complete native matrix NOT_RUN | **PARTIAL (development native shell only)** |
| Hosted production | The reviewed shell and service are verified at their public/owner-provided endpoints | NOT_RUN | NOT_RUN | NOT_RUN |

## Acceptance accounting

Every completed row must name the exact source/model/API identity, viewport/host, input and expected result, actual result, failures, and evidence path. Fixture rows and live rows are reported separately. The final report lists disagreements and negative cases first, not only aggregate pass counts.
