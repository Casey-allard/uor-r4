# Independent finite run3 execution-readiness review

**APPROVE — conditional readiness for one separately released finite environment qualification.** This is an independent source/metadata/operational review, not an execution release or a measured driver qualification. Training/M2/Stage B remain HELD; Q4 remains NOT_EXECUTABLE.

## Immutable packet reviewed

Directory: `/Users/casey.allard/uor-r4-local/codex-cf1-reuse-finite-run3`.

- REVIEW-SHA256SUMS: `31e6ea40df5d6b173fd027e2fd264e0cbded4743fef60ff5de63ec893c0ff2de`; all 16 listed files verified.
- Review pin: `3fd484da479e3e5eb4e374289d5852ba0b069c0545bc455250ec129c121103b6`.
- lease.sh: `554969b944f5c85ec189cafa351e6f1c590df9c4602f49247511f1c58824dc8e`.
- run-remote.sh: `ac64d2878abcce75663109bb6e5a02695c655c5dea302cb0ed4484e3d625217b`.
- expected-identity.json: `e61d7eedd490b56d9ce31d2117e54da3dd3b92e2912cdf3c87c792e425a2ace5`.
- Delivered runner: `1ae8e6021dc6853d2cf62571842c9d429b5d319acad8aa7d18a42ea889cc8fd9`.
- Provider helper: `cb524800bf583d1bffc956516f430e74128dc999224b5970b294588521cdc13d`.
- Instrument binary: `671917acb13b541c148b6b8122b92ae613ac0cdff8577a3f72fb6c05faa06209`.
- Cloud-store: `2e07a318d58b2f5d2f4ec73a32073b657a9d26d5263bd9398e7a9a4663ad3319`.

All three external SHA bindings verified. The helper SHA also equals the tested CLI SHA retained in protected #2567. All five materialized runner/dependency files match the exact git objects at protected runner commit `48478154ea4f0ff3d2b6fd28e76fd2292d7a75ed`, tree `bd703b6e19092e9c8af7b8d5e217150e988969ef`. The discrete crate and standalone instrument are unchanged between retained instrument source `5f533cbfd49ecef9a4ea2b217bb31a93002b3dc2` and that main commit. Bash syntax checks passed for both operators and the delivered shell dependencies.

## Scientific and operational scope

The review pin differs from finite run2's review pin only in out_dir. Expected identity is byte-identical. Composite `aca299c484d5434291f7f65cc9315d6b52b976952bcd68f39a45cd3341bf2825`, images/reports, geometry and fixture inventory remain unchanged. The one finite invocation retains exactly 64 proposal attempts (56 expected successes/eight expected status8) plus 16 typed refusals. No timing, endpoint rerun, checkpoint execution, apply, training or CPU reference is added.

Explicit `UOR_POD_VAST_DRIVER_VERSION=595.71.05` applies the protected server-query, local filtering and exactly-one-object precreate receipt guards. The operator uses the materialized delivered runner and pinned helper, config `/dev/null`, explicit Vast/5090/count1, no provider fallback/wait or owner override. The wrapper requires exactly one RTX 5090 with actual driver 595.71.05 before ldd or any instrument invocation. It verifies transferred inputs and exact no-context identity before the single finite command. Advertised metadata is not treated as runtime qualification.

Strict runtime own-AOT admission remains unchanged: U 88 registers/21536 local/shared0; R 115/21536/static shared512; dynamic0, zero spills and block16 launchability. No subtraction from ELF1536/reservedcap1024 is admitted. Constructor validation can precede resource refusal, so a future resource stop must disclose that reached GPU boundary rather than call it zero model compute.

Trap installation precedes creation. Any attempted submission requires a valid atomic terminal marker before preservation/teardown. Unknown/live state, SSH, renewal or preservation uncertainty retains the pod with a renewal/recovery instruction. The remote finish writes ended.txt before atomically publishing command.exit. Live monitoring renews every ten minutes; the 30-minute lease is not a healthy-run kill deadline. Local/remote 30GiB floors, local archive plus cloud-stage margin, fleet/rate admission, complete remote/input/operator/review SHA checks and pinned full-cloud MD5 receipt precede release/down. Final fleet status is retained for inspection. No on-lease repair or automatic retry is authorized.

The observed raw quote is explicitly distinct from the legacy admission-rounded quote. The prospective 30-minute allowance uses the raw $0.9083333333333332/h quote, approximately $0.454166667, rather than $0.908/h. Fresh selected metadata and full actual lifecycle must determine final accounting; the captured offer is not reserved and need not remain available.

## Conditions before launch

At review, readiness-verdict.md, hygiene-complete.json, execution-release.md, finite-pin.json and FINAL-PIN-SHA256SUMS were absent from the prospective packet. Their absence correctly prevents provider access through this operator.

1. Parent must supply and preserve actual successful hygiene75611 completion evidence; it has not been assumed here.
2. Parent must record this independent readiness/prospective environment FIT disposition and publish a new distinct finite execution release binding the immutable packet. Prior finite/timing releases remain consumed.
3. Final pin may differ only by replacing execution_release_reference:null with that new actual release URL; verify and publish the final SHA/manifest before launch.
4. Fresh disk, fleet, ownership, actual offer/price and exact advertised/remote driver gates must pass. Preserve any setup/resource/parity refusal and stop without an opportunistic dose or bar change.
5. Only independent actual-outcome acceptance and protected finite-result delivery could qualify the observed 595.71.05 environment for a future separately reviewed timing packet. This approval does not expand any timing allowlist or release timing.

No substantive readiness blocker found in the sealed packet. This review used retained files, exact git objects, SHA/cmp/JSON comparisons and shell syntax inspection only. No runner/helper/instrument execution, provider API, lease, pod, GPU/model/reference compute, source mutation or GitHub publication was performed. This verdict is outside the frozen packet.
