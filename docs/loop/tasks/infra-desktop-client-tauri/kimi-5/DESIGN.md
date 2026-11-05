# Design and verification: harness review repairs

## Record boundary

The `kimi-1` design record is prior-unit history and is not used for repair evidence. This file carries the current repair decisions and measured results only. The macOS asset origin is `tauri://localhost`; HTTP(S) aliases are confined to platforms that use them. The page never receives the sidecar token.

## Lifecycle and provenance decisions

- Runtime verification binds every local model, tokenizer and served-forward audit path and SHA-256 to `models.json`. Allowed suffixes alone are not provenance; undeclared files and modified declared files fail. The staging fixture is trained-manifest-format data using synthetic bytes, not a trained model.
- Git local/worktree filter scopes are queried with includes, and one process-wide guard serializes repository writes with Git status/diff and terminal Git execution. Static file reads require canonical containment.
- Native failed MCP listing releases its session and child. MCP replacement uses one in-flight token, disables both controls, retains the old session on failure/cancel and disconnects stale replacements.
- Process-group termination first checks `try_wait`; an already-exited/reaped child returns `false` without another signal. Sidecar Drop/shutdown and MCP Drop/timeout can therefore repeat safely without signaling a reused PID. Bounded terminal processes and MCP requests abandon stuck reader joins after direct-child exit and return a timeout instead of hanging on descendant-held pipes.
- Sidecar cancellation uses one `entry` operation. A deterministic two-phase probe proves cancellation after registration cannot be overwritten by request setup.
- Terminal `rg --pre` and every Git `-c`/`-C` or long config/exec-path override are refused before execution. Output-cap truncation is reported separately from wall-clock timeout. Catalog verification covers `models/` and bundled `pkg/` paths and hashes, and every runtime symlink is refused.
- Recent files retain `{path, rootPath}`, concurrent opens re-check after awaiting, and initial deep links cover Changes/Terminal without prepending Chat.
- Product records distinguish automatic loopback sidecar startup from user-approved external MCP. Current matrices are default 7/12/7/13/8 and GUI 9/16/8/13/9; `IntegerStackModel::parse` is the actual model-load API. Trained-manifest-format fixtures use synthetic bytes, not trained artifacts.

## Verification

Failing-first tests covered extra allowed-suffix files, tampered declared artifacts, repeated termination and the deterministic cancellation overwrite. Complete results: 314 Vitest plus TypeScript, 66 Node, 7 runtime; default desktop 7 library, 12 controls, 7 MCP, 13 review, 8 sidecar; GUI 9, 16, 8, 13, 9. Package verification is 2/2.

The final macOS app is 20.86 MiB with executable SHA-256 `20dce8ddd3d16b4a11ce49bbd0635becb1af5f7aa52fc966685f5b541911db2c`. A fresh synthetic `Hi` returned 16 tokens with usage `19/16/35`, `8141.1 tok/s`, TTFT `0.810 ms` and total `1.965 ms`. Normal Quit left app PID 1881 and sidecar PID 1884 absent. Earlier real acceptance also covered A/B file isolation, both MCP transports and cancellation preserving the active stdio session; all remain protocol evidence, not useful conversation.

Both product diagrams were inspected in a real browser and accurately show the local sidecar and approved outbound paths. The previous design record is restored; the retired current-state file and old out-of-scope SVGs were not edited. Regenerable build outputs are removed after verification, leaving 25 GiB free rather than the 30 GiB target.
