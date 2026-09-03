# Proposed product boundary and component transfer

Status: proposed architecture for the parent plan. This is not an approved implementation or a new scientific run.

**Current checkpoint (2026-09-03):**
[#1105](https://github.com/UOR-Foundation/uor-r4/issues/1105), a contract-only
child of #1084, delivers the
[native four-fact service ADR](../adr/0006-native-four-fact-workbench-service.md)
and [machine contract](../r4_service_contract_1105.json). They narrow the first
delivery to one dedicated, opt-in `r4-workbench` host, one private child process
of that same executable and one `answer_four_fact_raw_text/v1` research-reference
shell. Independent review accepts `SERVICE_API_CONTRACT_SPECIFIED`;
protected delivery closes #1105 only.
No host, worker, shell, build, model operation, HTTP request or browser behavior
has been implemented or exercised by this specification.

## Decision

One local Rust service owns served model identity, provenance, readiness, inference, cancellation and optional workspace operations. It serves a small static web workbench from an explicit asset root. The frontend chooses the same-origin API first and renders capabilities returned by that service. Hosted Pages remains a static client; when no native backend is connected, it may offer an explicitly named browser teacher provider, with the actual model source visible. It must not silently emulate a native geometric model.

For the first bounded flow, the #1105 contract selects a new loopback host plus
one private same-executable worker. It leaves the existing root server unchanged,
exposes no public comparison route and excludes an automatic teacher fallback.
Public model calls use only the learned-reference `qualify()` and `answer()` path.
The broader provider, workspace and hosted-client options below remain later
parent-plan work, not behavior supplied by #1105.

Use a provider interface at the frontend boundary, not model-specific conditions scattered through views:

```text
Workbench
  ├── Sessions / composer / answer and code renderers
  ├── Workspace / Monaco / diff / preview
  └── Model readiness + optional evidence/telemetry panel
         │
         ├── NativeProvider → same-origin Rust API
         │      ├── supported OpenAI text operations
         │      ├── explicit native research operations
         │      └── scoped workspace and lifecycle operations
         └── BrowserTeacherProvider (optional, explicitly selected)
                └── actual pinned Transformers/ONNX model in a worker
```

The shared contract should carry `model_id`, actual artifact/source identity, execution kind, supported operations, readiness/error state, token/context limits, request ID, finish reason and measured usage. Research metadata can be expandable, keeping ordinary UI language simple. For unsupported chat/tool/multimodal operations, disable the corresponding action or explain the supported form before sending; do not fabricate a compatible response.

Keep the existing root HTTP/OpenAI contract and its truthful errors unchanged
and separate. The new opt-in host owns its narrow `/uor/v1/workbench/` namespace;
it does not forward into `/api/sysinfo`, `/uor/v1/status` or `/v1/*`. Avoid a
general rewrite of the 31,000-line server as part of a visual port. The dedicated
host and worker remain a separately scoped implementation.

## Transfer map

All donor references below are at `5a10305126df62e838cadfec5fd509e0c9705fa7`; paths are relative to `/Users/casey.allard/Downloads/uor-r4-project`.

| Component/behavior | Decision | Concrete source and necessary adaptation |
|---|---|---|
| Dark surfaces, cyan accents, typography, spacing, cards | Extract and consolidate | `index.html:23` CSS variables/style layer; many later overrides. Turn into one design-token/style system. Keep system-font fallback; bundle/pin optional fonts. Do not copy scattered `!important` overrides and repeated injected CSS. |
| Conversation and workspace rail | Port UI pattern | `index.html:2160` rail markup, `:5584` session list. Retain clear recent chats, new chat, search and workspace switch. Give project/session stable IDs; keep messages separate from rendered DOM. |
| Composer with model selector, explicit attachments, send/stop | Port UI, rewrite execution | `index.html:2300`, `:5401`, `:5520`, `:6292`. Use provider-discovered models; explicit finalized context; actual cancellable request state. Preserve selected model through loading/errors; no silent fallback. |
| Rich answer and code actions | Port behavior; replace renderer | `index.html:2774` copy/edit/diff actions, `:5839` renderer. Retain copy code/response, edit in IDE, show proposed diff. Use sanitized Markdown and a stable message model; preserve original code bytes. |
| Monaco editor tabs and side-by-side diff | Port component, unify buffer state | `index.html:3987` initialization and `:7300` diff flow; `assets/images/monaco_editor_ide.png`, `monaco_diff_view.png`. File identity must be full relative path rather than basename; centralize original/current/proposed buffers and dirty status. |
| Apply/discard proposed change | Port explicit review interaction | `index.html:7369`. Applying should produce a file delta; saving must report the actual destination and outcome. No success toast after a failed write falls back to download. |
| Live Preview with logs → chat iteration | Rebuild from interaction concept | `index.html:4360`, `:4846`, `:4917`; screenshot `assets/images/sovereign_studio_live.png`. Current iframe/view markup is absent. Use a separate-origin or tightly sandboxed preview, a narrow message schema and request/source checks; isolate generated code from app credentials/files. |
| File tree and explicit context chips | Port UI, replace access adapter | `index.html:3305`, `:3469`, `:4925`. Local Rust owns scoped workspace file I/O; browser-only mode may use File System Access API or export. Surface permission/read/write errors and context truncation clearly. |
| GitHub repository chooser/PR overview | Port read-only views first | `index.html:3864`, `:3915`. Provider-neutral repository identity and real server/gh status. Avoid frontend PAT persistence; actual PR/issue writes remain deliberate workflow actions. |
| Commit/push and worktree management | Rewrite | Current browser code commits each file separately; native code stages all paths. New service must select named files, produce one reviewable diff/commit, honor current protected-main rules and distinguish queued/merged states. No merge or shell action inferred from a chat reply. |
| Model manager and background progress pill | Port presentation only | `index.html:5249`, `:5285`, `:5967`. Use a lifecycle state machine: unavailable → downloading(bytes/total when known) → loading/compiling(indeterminate if unmeasured) → ready → running → stopping → stopped/error. Cache availability, compiled session readiness and completed response are separate states. |
| Cache reuse/purge | Rewrite around a manifest | `index.html:5227`, `:5378`; worker registry/load code. Bind source revision, files, sizes/hash and runtime compatibility. Evict only the selected model's own namespace; don't claim complete cache from one filename. Report downloaded bytes, load time and actual backend independently. |
| Session persistence | Rewrite storage seam, retain UX | `index.html:5641` uses localStorage. Use an explicit schema/version and safe size/error handling; IndexedDB for browser data or Rust-owned persistence locally. Keep conversation history, workspace references and transient generation state distinct. |
| Search grounding | Rewrite context pipeline | `index.html:6175`, `:6364`. Preserve visible citations/source chips; ensure retrieved text actually enters the final request, with source IDs and declared context truncation. Donor's search banner does not prove grounding. |
| Brain/EEG/geometry panel | Optional design asset, not primary acceptance | `index.html:6657`, `:6914`, `:6970`. Default collapse or secondary panel; show measured evidence only. Text-derived observer data must be labeled as that, not hidden attention or biological thought. Remove unsupported speed/quality claims. |
| 512D workspace index | Separate optional retrieval experiment | `src/lib.rs:2448`. It is hashed lexical retrieval with exact-token boosts and simple declaration splitting. Do not transplant it into the research model or claim that it supplies qualified attention. Ordinary file/symbol search is sufficient for the initial product. |
| Tauri/Electron/Hermes | Do not port as the main client | User direction already selected the web workbench. Native wrapping can remain a future packaging decision after the same web/API path works; it must not introduce a second model implementation. |
| Browser Transformers worker | Optional independent provider, not native implementation | Preserve background execution concept and actual model sources if browser teacher support is wanted. Rebuild cancellation, model identity, progress and load serialization. Do not merge its teacher capability claims with native R4 qualification. |
| Branding assets/screenshots | Reference/selectively copy with attribution | `assets/images/*` offers design references and marketing images. Use current product-specific wording and honest screenshots after behavior checks; do not copy benchmark/paper claims into the new product. |

## Smallest useful delivery sequence

1. **Freeze the application contract and chosen first user flow.** #1105 delivers the independently accepted ADR and machine contract for `answer_four_fact_raw_text/v1` as `SERVICE_API_CONTRACT_SPECIFIED`; protected delivery closes only that child. No mock or specification state counts as host behavior.
2. **Implement the workbench shell and one native request path.** After #1105 delivery, leave #1084 open and separately activate an implementation child for the dedicated `r4-workbench` host, private same-executable worker and first four-fact shell. Freeze and independently accept concrete source/build/environment admission before a build, then a separate actual-host qualification release before model work. The consumed #1102 CLI/coordinator and qualification cannot authorize the host.
3. **Complete request/model lifecycle.** Serialize load/generate/cancel, keep model selection stable, preserve artifact identity, show real progress and request completion. Add one scoped cold-load and warm-load exercise only when that lifecycle is the named product decision.
4. **Add the editor and file workflow.** Explicit attachment → request context → proposed diff → review → save → reopen. Reconstruct sandboxed preview and code/log iteration separately; validate real saved file contents, not just an on-screen success label.
5. **Add Git/issue/worktree integration through the same local service or delegated harness.** Read status first; named paths; scope each issue; real PR/check/queue/merge state. Do not import the donor's multi-file Contents PUT loop or `git add .`.
6. **Package and measure the chosen release surface.** Bundle assets/dependencies, limit static-file serving, add a single launch path and deterministic version/build identity. Run only the explicitly activated product behavior checks. Existing transport queue acknowledgements are not QA.

Each delivery should change a user-visible behavior and have a short, named acceptance question. Model quality and product interaction are separate gates. A successful raw native continuation path does not establish correct coding, supplied-context reasoning, tool use or multi-turn behavior. Scientific work can continue independently without having every research issue repeat frontend QA.

## Concrete early acceptance questions

- Does the served page connect to its own API and display the actual model/artifact identity?
- Does an unavailable or unsupported operation show an honest error without silently switching to a teacher?
- Does one request finish, expose its stop state, and leave the composer usable? If cancellation is claimed, does compute actually stop?
- Does model selection survive failure, reload and a warm cache path? Is displayed progress tied to measured bytes or explicitly indeterminate work?
- Does the actual attachment/search text reach the request after all context assembly, with any truncation visible?
- Does Apply/Save/Reopen change exactly the selected file and preserve unrelated edits?
- Is preview code isolated, and do its logs reach the intended preview/session only?

These are candidate product checks for a future authorized scope. None was executed in this audit.
