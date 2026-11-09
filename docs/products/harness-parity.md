# UOR Harness parity matrix

Target: a UOR-branded Claude Desktop Cowork/Code-style harness using the separately owned R4 model API. This matrix is evidence, not a feature wish list. `NOT_RUN` means exactly that; a control, fixture, or old report never substitutes for the required live workflow.

Reference: [Claude Desktop](https://docs.anthropic.com/en/docs/claude-code/desktop), [Cowork surfaces](https://support.claude.com/en/articles/15520349-use-claude-cowork-on-web-desktop-and-mobile), [projects](https://support.claude.com/en/articles/14116274-organize-your-tasks-with-projects-in-claude-cowork), and [Dispatch](https://support.claude.com/en/articles/13947068-assign-tasks-from-anywhere-in-claude-cowork). The approved design is `docs/superpowers/specs/2026-11-08-claude-cowork-parity-design.md`.

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
| Static preview honesty | Static Pages clearly disables unsupported service/host actions and never claims full parity | Local dist explicitly marks browser/native/service-only controls unavailable; zero external requests in the existing runtime trap | PR #3897 merged as `a8e51c0f4d0ed6e999b6d9e13f8289215efbd684`; publish commit `6540e2717d3a606330dbd141e3642f3f13b87ecf`; Pages run `42668187497` succeeded. Fresh-main explicit staged Playwright **45/45**, direct staged WASM **7/7**, 14 requests with zero external/raw-TypeScript/runtime errors; at explicit `?typed=1`, live desktop/restored mobile primary and split-close verified; static Terminal explicitly unavailable. Plain `/studio/` remains the legacy UI at this publish, with cutover pending #3903 and post-deployment records assigned to `kimi-17`. Previous rollback head `ab752d595`; evidence `local/kimi/typed-publish-main/a8e51c0f4d0ed6e999b6d9e13f8289215efbd684/` | **PARTIAL (typed preview published; plain-root cutover and hosted native workflows pending)** |
| R4 model API integration | Merged Claude-owned API completes the streaming/tool-call contract with exact identities | NOT_RUN | NOT_RUN | NOT_RUN |
| Packaged desktop | The complete matrix repeats in the unsigned development Tauri app | Packaged resource contract passes; cowork identity and GUI symbols verified | `local/kimi/harness-parity/native-agent.jpg` and `native-split.jpg`: Workspace, Split, and New session pass in the unsigned development app; complete native matrix NOT_RUN | **PARTIAL (development native shell only)** |
| Hosted production | The reviewed shell and service are verified at their public/owner-provided endpoints | NOT_RUN | NOT_RUN | NOT_RUN |

## Acceptance accounting

Every completed row must name the exact source/model/API identity, viewport/host, input and expected result, actual result, failures, and evidence path. Fixture rows and live rows are reported separately. The final report lists disagreements and negative cases first, not only aggregate pass counts.
