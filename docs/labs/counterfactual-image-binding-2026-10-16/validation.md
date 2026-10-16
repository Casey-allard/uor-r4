# Image-binding validation

Source base: `575645a26e7d207a822e20520d69c447d1168aac` (#2555).
Independent design review: [6224975357](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6224975357).

At this source commit: standalone host suite **38 PASS,0 failed,1 existing
ignored**; the two new crate image-metadata tests **2 PASS**, with46 other
crate tests filtered out. The refusing version2 [template](controls-pin-template.json)
and host plan preserve NOT_RUN, no-context/no-model and apply0 labels.
Independent provisional source review found no blocking source defect;
final exact-head confirmation follows the committed revision on the PR.
CUDA check/strict-Clippy, actual cubin inspection and no-context CUDA identity
are PENDING at source commit. Their exact-head construction transcript and
independent verdict will be attached to the protected PR without changing
the reviewed source. Finite GPU controls: NOT_RUN. Stage2: NOT_RELEASED.

Host checks exercise pin/image identity and JSON refusal, source-only
checkpoint envelope mapping and preserved fixture receipts. They implement
no host model, scoring or reference path. The initial sparse checkout missed
root `src`, the explicitly declared `tests/bdd` target and an included
historical pilot-plan JSON. Cargo refused manifest/source inclusion before
the test executable ran. Adding those tracked directories repaired checkout
completeness; project source and test expectations were not modified.

Construction must use the protected-delivery head and the existing admitted
standalone manifest `docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml`.
Compile check and strict Clippy use `--features cuda --all-targets` where
applicable. The no-context identity command is
`decompose-controls-identity`; no control/diagnostic command runs in this gate.
Preserve the PTX-input assembly command and exact input/output/report hashes,
and inspect the actual cubins with `cuobjdump` before accepting the transcript.

New paid construction cost: $0 so far. No new model/reference execution,
CUDA context, pod or finite control result is reported by this source record.
