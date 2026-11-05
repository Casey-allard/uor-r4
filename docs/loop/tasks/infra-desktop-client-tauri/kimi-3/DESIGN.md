# Design and verification: harness review repairs

## Record boundary

The `kimi-1` design record is prior-unit history and is not used for repair evidence. This file carries the current repair decisions and measured results only. The macOS asset origin is `tauri://localhost`; HTTP(S) aliases are confined to platforms that use them. The page never receives the sidecar token.

## Lifecycle and provenance decisions

- Runtime verification binds every local model, tokenizer and served-forward audit path and SHA-256 to `models.json`. Allowed suffixes alone are not provenance; undeclared files and modified declared files fail. The staging fixture is trained-manifest-format data using synthetic bytes, not a trained model.
- Git local/worktree filter scopes are queried with includes, and one process-wide guard serializes repository writes with Git status/diff and terminal Git execution. Static file reads require canonical containment.
- Native failed MCP listing releases its session and child. MCP replacement uses one in-flight token, disables both controls, retains the old session on failure/cancel and disconnects stale replacements.
- Process-group termination first checks `try_wait`; an already-exited/reaped child returns `false` without another signal. Sidecar Drop/shutdown and MCP Drop/timeout can therefore repeat safely without signaling a reused PID.
- Sidecar cancellation uses one `entry` operation. A deterministic two-phase probe proves cancellation after registration cannot be overwritten by request setup.
- Recent files retain `{path, rootPath}`, concurrent opens re-check after awaiting, and initial deep links cover Changes/Terminal without prepending Chat.
- Product records distinguish automatic loopback sidecar startup from user-approved external MCP. Current matrices are default 5/12/6/10/8 and GUI 7/16/7/10/9; `IntegerStackModel::parse` is the actual model-load API. Trained-manifest-format fixtures use synthetic bytes, not trained artifacts.

## Verification

Failing-first tests covered extra allowed-suffix files, tampered declared artifacts, repeated termination and the deterministic cancellation overwrite. Complete results: 314 Vitest plus TypeScript, 66 Node, 7 runtime; default desktop 5 library, 12 controls, 6 MCP, 10 review, 8 sidecar; GUI 7, 16, 7, 10, 9. Package verification is 2/2.

The formatted macOS app is 20.86 MiB with executable SHA-256 `7ab5303f984ea307fe1c7f0a7c322124be98a93b484dcc2186cc5e641fb109e7`. A fresh synthetic `Hi` returned 16 tokens with usage `19/16/35`, `18549.8 tok/s`, TTFT `0.458 ms` and total `0.863 ms`. Normal Quit left no app or sidecar process. Earlier real acceptance also covered A/B file isolation, both MCP transports and cancellation preserving the active stdio session; all remain protocol evidence, not useful conversation.

Both product diagrams were inspected in a real browser and accurately show the local sidecar and approved outbound paths. The previous design record is restored; the retired current-state file and old out-of-scope SVGs were not edited. Regenerable build outputs are removed after verification, leaving about 28 GiB free rather than the 30 GiB target.
