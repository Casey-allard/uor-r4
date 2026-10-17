# Independent baseline-reuse timing startup audit — run1

Reviewer: Codex independent reviewer `review_2551`.

**Verdict: APPROVE the retained outcome as BLOCKED_SETUP_DRIVER, before any instrument binary invocation.** The admitted driver check stopped the attempt correctly. This is an operational admission blocker, not a resource/parity observation, speed result, or model/mechanism negative. Release6227893756 is consumed; this audit grants no retry, driver broadening, timing or training authority.

## Reviewed retained evidence

Archive: `/Users/casey.allard/uor-r4-local/codex-cf1-reuse-timing-run1/reuse-timing-run1-archive`.

Independently verified20 remote-manifest entries,24 operator-manifest entries and10 transferred input identities. The input checks were reviewer post-run checks; the stopped wrapper did not reach its own input-check command.

Final pin SHA256 `dea48273933cbacb869f41fe5cbba63e0dcfc3cd625cdbc3570b4a1958514f70` differs from the reviewed pin only by execution_release_reference, which is `https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6227893756`.

The retained remote script is unchanged, SHA256 `678f0e61979ba900ccf43b5dfebe55f30a3469abbe884697cd4ffde158dece02`. Retained executable SHA256 is `671917acb13b541c148b6b8122b92ae613ac0cdff8577a3f72fb6c05faa06209`, with the same pinned source5f533cbfd49ecef9a4ea2b217bb31a93002b3dc2. This is transferred-byte verification, not an actual runtime identity observation: the binary was never invoked.

## Actual execution boundary

- Podvast-56325466; sessioncodex-cf1-reuse-timing-run1.
- Actual RTX5090, driver595.71.05, GPU UUID `GPU-1daf8397-88e7-c3e5-e50c-0dbc1459969c`,32607MiB.
- Admitted driver580.95.05 only; actual595.71.05 is outside that admission.
- Command started and ended2026-10-17T01:22:42Z, exit1.
- Command log records successful remote disk-floor metadata then: `BLOCKED: timing admits only finite-qualified driver580.95.05; no retry`.

The unchanged script checks the GPU/driver before ldd, input verification, reuse-identity, reuse-check, reuse-plan, the model-command marker and reuse-timing. The complete archive contains no ldd.txt, actual-identity.json, checked-plan.json, execution-plan.json, model-command-invocation-count.txt or timing-output directory. The admitted-runner job is the wrapper invocation; it is not an instrument/model invocation.

Therefore instrument binary invocations, instrument CUDA/model initialization, proposal attempts, warmups, timed proposals/pairs, apply and training are all zero for this attempt. No checkpoint bridge, checkpoint model evaluation or runtime U/R resource gate was exercised. The accepted finite result remains separate and unchanged. Absence is assessed against the exact reviewed script and its observed stopping log, not inferred from missing files alone.

## Preservation and closure

Local operator log records cloud archive `icloud:UOR-R4/results/codex/reuse-timing-run1-archive.tar`,9,547,264bytes, full-readback MD5 `ef4197f63542ecd58a366b80316b308d`, before lease release2026-10-17T01:23:20Z and pod deletion01:23:25Z. Final fleet0; operator exit1 preserves the actual blocked command outcome.

Creation01:21:57Z, deletion01:23:25Z gives88s lifecycle; ledger separately records86s provider uptime. Actual rate$0.908/h. Keep lifecycle and provider estimates distinct; parent owns the final cumulative ledger reconciliation. No authentication failure is shown.

## Disposition and limits

Protect-deliver this stopped operational record and preserve its consumed release. No automatic retry or driver allowlist expansion follows. A future action requires separately reviewed prospective disposition, exact packet and distinct release; this audit proposes none. Do not report INCONCLUSIVE timing, regression, speedup, checkpoint parity or a model negative: no timing/model dose ran.

This review performed local metadata/hash inspection only: no archived executable, model/reference, pod operation, publication or repository mutation. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; seed2 and production activation excluded.
