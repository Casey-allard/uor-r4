# ORDER: repair desktop operation identity and workspace clearing

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `a60eceb65b7d38a14dba5384c0085c881e8445a7`. Prior reviewed head: `02c4ef67055a075409b4fb1eb8b449e08dd03ef2`, tree `927e26a8b41700a57575d226619933aac37d5eaf`, PR #3819 draft.

## Finished steps

- kimi-9 added six typed destinations and native-adapter Git/Terminal workflows; all 350 typed tests, 112 plain-JS tests, legacy checks, and 11 gates passed.
- Fresh review returned CHANGES: shared Enable/Run operation identity could leave a permanent busy state, and empty `rootPath` did not invalidate stale work. It also identified empty-input guidance and timeout/unmount/available-host evidence gaps.

## Goal

Give Enable and Run independent identities and prevent their overlap. Invalidate every in-flight action on any workspace path change, including `''`. Make empty-input guidance explicit and close the named evidence gaps without changing the native adapter or approval contract.

## Cancel probe

Reproduce Enable-during-Run, Run-during-Enable, and stale completion after `rootPath: ''` before changing source. Stop rather than weakening per-command approval or accepting stale output.

## Files in scope

- `web/studio/src/components/DesktopTools.tsx` and `DesktopTools.test.tsx`.
- `web/studio/src/components/StudioApp.test.tsx` and `AppShell.test.tsx` only for integration/wording.
- `web/studio/README.md`, `docs/products/web-client.md`, and `docs/research/notebook/m4-serving.md` only for corrected counts/evidence.
- This repair folder.

No other paths are authorized. No legacy adapter/HTML, Tauri host, dependency, engine, worker, tokenizer, `studio_api.js`, model artifact, publisher, deployment, `.github/**`, `Cargo.lock`, real repository/command/browser data, owner-controlled path, or generated roadmap/issue edit.

## Done when

- Enable cannot start during Run and Run cannot start during Enable. Their independent operation identities clear busy state correctly on completion/error/workspace change/unmount; no permanent waiting label remains.
- Any `uor-files-changed` rootPath, including `''`, invalidates stale work and drives the next action's path. Empty path/command has visible guidance and no run.
- Tests cover timeout, unmount completion, unavailable invocation, available-host StudioApp integration, and all original adapter/approval/inert-output behavior.
- Focused/full Studio checks, plain-JS tests, legacy checks, claim/report/local-policy checks, and full gates pass at the new exact head. Existing disclosed D76 advisories remain disclosed; no new warning is introduced.
- Fresh exact-head review approves before ready/queue. PR keeps `References #3500`; no live cutover, publish, migration, useful-chat, M7/M8, production-readiness, or accessibility-certification claim.

## Stop and report conditions

Stop before changing files outside this scope or weakening a test/verifier. Stop after this repair and protected delivery; do not start another UI unit in this session.
