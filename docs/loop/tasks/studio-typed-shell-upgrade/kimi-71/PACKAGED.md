# Packaged Studio acceptance, kimi-71

Lab: kimi. Acceptance code head: `42bc46f81a5f7c3ccc7508866162bdbd7a42d89c`. The unsigned development bundle was built after that merge, before documentation-only acceptance records. Its executable SHA-256 is `c8ddeb4039c1721139c990a06c3d188620596cdbb3cdd654308fa3074f53c8c6`.

## Measured workflow

- The packaged window loaded `tauri://localhost/index.html?typed=1` and mounted the typed Preact application. Only the task-owned `local/kimi/studio-final-acceptance/packaged-workspace` folder was granted.
- A fresh `19+23` session returned exact `42` with the visible R4 geometric trace and `Artifact: ready`. This is a geometric tool response, not model generation or useful-chat evidence.
- New file `untitled-6.md` transitioned from dirty to `File: saved`. Disk readback was exactly `Packaged final native edit/save acceptance.\n` (44 UTF-8 bytes), SHA-256 `24fb7f9e465b859e8f5f4f922d7b14ee2b510f2d5f58f27a7b454cc99391dfb7`.
- Attach current file retained `untitled-6.md` with a Remove action. Remove was not activated. The attachment remained after relaunch.
- Changes returned `HEAD (unborn) · 2 changed files` and the complete intent-to-add `note.txt` diff. Neither `HEAD (timeout)` nor `diff timed out` appeared. `untitled-6.md` was untracked.
- Terminal required a native session-enablement dialog and then a separate native one-run approval showing the exact task-owned cwd and lowercase `pwd`. The command returned exactly `/Users/casey.allard/uor-r4/local/kimi/studio-final-acceptance/packaged-workspace`.
- Quitting and relaunching the same executable restored the `19+23` transcript, attachment, and exact saved editor content. Reopen showed `No source selected`, `Artifact: unselected`, no folder grant, and terminal disabled. Explicitly reselecting only the task folder preserved the restored content; terminal still required new enablement. No post-reopen model readiness is claimed.

`local/kimi/studio-final-acceptance/packaged-evidence/manifest.json` links these observations to raw accessibility captures, screenshots, and their hashes. The package resource contract passed with the executable hash above. All app processes owned by this acceptance were quit.

## Limits

This is unsigned development evidence only: no signed release, production publication, accessibility certification, M7/M8, useful-chat, or Phase B claim. Several older untitled fixture tabs remain; this final sequence saved only `untitled-6.md`. Native-runner and static-WASM host-action limitations are separate and remain explicit. Complete Rust/product/browser/repository checks and protected delivery are recorded separately in `REPORT.md`; they are not inferred from this GUI workflow.
