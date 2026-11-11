# ORDER: register the LexLean MVM workbench item, kimi-20

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569, registration only). New item: `studio-lexlean-mvm-workbench`. Base: `93e089ff256654e75a19641b7e2fbd2bc8e24524`. References #3569; never closes it or changes acceptance.

## Owner goal

The owner requested LexLean's UI, real MVM, GitHub PR workflow and desktop parity: **"Please use your best judgement when making decisions."** Register only; the top session decides scope/acceptance.

## Cancel probe (ran first)

Roadmap check passed with 144 items. No MVM crate exists at base, no open PR overlaps Studio/Tauri/MVM paths, and #3569 is OPEN. Use this parent because a new item cannot anchor its registration ORDER at its own merge base. Missing implementation is work, not cancellation.

## Scope decision and files

Top-session D76 review accepts only the new-item/design/plan advisory. Touch only those documents and this folder. Implementation uses the merged item's ORDER and official roadmap output. No model/API/service, `.github`, rendered-view, credential or deployment change.

## Binding implementation rulings

1. `AgentStore`/`AgentTaskStore` remain the only task authority. MVM owns only bounded per-tab files, virtual Git, memory-only history and CoW; it is not a model/task runtime.
2. Pin LexLean `01956d5c35f268fc7d1b646d97d3dff21b9e945f` and hashes. Extract only `lexlean_generated.rs:45,751-46,087`, the self-contained vsock tail module; exclude every other generated line and AST Cargo evaluator. Initramfs is hash-pinned Rust source, never a committed binary.
3. Tasks 1–2 must pass exact-output native/browser MVM preflight before UI. `cc` runs two genuine C programs; Cargo/rustc without a real guest toolchain reports unavailable.
4. `WorkspaceEditor` gets an MVM file source and per-file revisions. Preserve serialized path/saveId saves and refuse stale editor/terminal writes.
5. Enforce numeric caps, transactional digest read-back and corrupt/quota/version refusal. CoW goes directly worker→IndexedDB; JSON carries only snapshot ID/digest. Web Locks prevents duplicated-tab shared writes. Persist no raw history/tokens; refuse secret-like input and page-callable secret/proxy/raw-CoW APIs.
6. Pin `RepoTarget`; use base ref/blob SHA CAS and single-flight separate branch/edit/commit/PR approvals. Refuse cross-repo/default-branch/unapproved writes. Tokens are memory-only, never registered with MVM; secret tests are synthetic.
7. Desktop reads/writes use `AgentHost`-approved fixed `gh api` under unchanged CSP or unavailable. Bind writes to an existing selected task/run without synthetic records. Direct static-token writes remain UNVERIFIED unless exercised.
8. Default tests never write GitHub. Smoke needs `UOR_GITHUB_SMOKE=1` and a non-production allowlist; record owner/repo/provisioning in committed task evidence. Drive static web through loopback `gh` and test desktop approval separately; leave the PR open.
9. Preserve selectors, skip link, 1023/1024, keyboard/focus, CJK/RTL/IME and 390/1024/1440 behavior. Generate MVM in ignored `node_modules/.cache`; Vite hashes the `new URL` input into local `assets/*.wasm`. Desktop must execute status/cc, not only mount.
10. Never hand-edit generated views. Use red-first tests, serial writers and independent design/implementation/exact-head reviews.

## Done when

- Design, plan, item and records pass roadmap/diff checks; independent design findings are resolved.
- ORDER/REPORT, policy/full gates and required GitHub checks pass; fresh exact-head review and protected merge complete.
- After merge, official `roadmap sync --only studio-lexlean-mvm-workbench --apply` creates the issue and official render is delivered separately. Verify item/issue identity before creating the implementation ORDER/worktree.

## Stop and re-scope if

A check requires changing #3569 acceptance, undeclared source, second runtime, persisted credentials, committed binary, default-branch write, broader CSP, hand-edited generated output or unsupported claims.
