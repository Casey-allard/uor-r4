# Attention preflight Battery2: staging stop — 17 October 2026

**Observed outcome: operational data-staging failure before the first project
executable.** Battery2 exited1 with zero attempted command steps. This is an
instrument/setup defect, not evidence against Relation or native attention.
The released attempt is closed; its permission cannot be replayed.

The protected line remains **count0/3**, with headline **reply37/232 SUSPENDED**
and attributable learned Relation **NOT_RUN**, per
[PM count ruling6242715607](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6242715607).
Preparation units and this D31 staging failure do not consume the executed-test
line count. No
learning, mechanism negative, KEEP, progress reset or milestone transition is
claimed. Under D31, this instrument failure does not add a mechanism negative
or reset the existing D21/D22 history. Training/M2/Stage B remain HELD and
Q4 NOT_EXECUTABLE.

## Authority and exact artifacts

- [PM battery FIT/cost/sensitivity disposition6241890920](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6241890920).
- [AGY actual packet APPROVE6242095986](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6242095986).
- [Actual finite-preflight dispatch6242232012](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6242232012).
- [Actual closure report6242397583](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6242397583).
- [PM operational closure/repair disposition6242429565](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6242429565).
- Protected source `d46e050f4dfe4177f599410f34e1647cc9b2fc4f`, reviewed delivery
  `ca70766067adbf58409ac5d590f84636339377ff`, constructed source
  `96ad8a4750398a8ecbf89e711f351de1580d7991` and binary SHA256
  `5abe54c3ce64dd66006b5ff50697d0c416071ba50c473014a0053e6bd63099ed`
  retain their distinct identities. The discrete crate is byte-identical across
  those source commits; no runtime identity was observed.
- Frozen44-member packet manifest
  `8e995d5b7d84823653852c5588b6f2a00eec059339ea7e603e2e44a7a13dbacd`;
  actual dispatch SHA
  `fa571c489b1c12a494f7ecddedd763ec2646fe16e7d20b7e5e756c7cfeb7c826`;
  installed actual AGY verdict SHA
  `2e625f75b3f9a4bd0d617593ba19dbbd8aad4e873fb8451d88f7d6108e3b5d37`.
- Attempt `uor-cf1-b2-fa3ccb55425d6091c63b7beb62b8367e`, own pod
  `vast-56460012`, session `codex-attention-preflight-battery2`.

The [token-cell source/construction record](../attention-token-cell-export-2026-10-17/README.md)
retains Gate9's five construction PASS results. This failure neither invalidates
those scoped results nor promotes the uninvoked executable to runtime evidence.

## Actual boundary

Fresh free disk50960684KiB exceeded the full32014336KiB admission reserve.
The native runner selected one RTX5090 at a recorded rounded$0.964/hour.
Setup completed, with SSH readiness after24s, and recorded driver595.99.02.
OS utility setup, source/input staging, retained-D34 metadata checks and NVIDIA
environment queries occurred. They are not model qualification.

The native job started and ended at22:33:10Z with exit1. Its scope file describes
the admitted finite-model plan; it does not prove that model work began.
`job-outcome.json` reports `attempted_command_steps:0`, and `commands/` is empty.
The raw data SHA check precedes the first numbered `run()` call and every
project-binary invocation.

| Phase | Observed status |
| --- | --- |
| Source/OS setup and staging adapter | Completed; adapter completeness claim disproved below |
| Raw data check | TRAIN tokens OK; TRAIN mask, DEV tokens, DEV mask and tokenizer missing |
| Binary metadata identity/panel | NOT_RUN |
| Initialization / checkpoint creation | NOT_RUN |
| Device token-cell export / restore | NOT_RUN |
| DEV evaluation / B3 admission | NOT_RUN |
| Gate22 / Gate0 proposals | NOT_RUN |
| All nine Relation qualification controls | NOT_RUN |
| Apply / learning / dose / fresh access | NOT_RUN; zero applies |

The relevant retained hashes are data-check
`016213f9f7c2c2d1b03b9fd0a704060453782e864ff9e6c3a67c6db469dee217`
and job-outcome
`78a47a0cb4872e467200f00743e7a3cf538defccb5b1794756579ddb48fe7dc7`.
No B3, sensitivity, export or mechanism-quality conclusion is available.

## Bounded diagnosis and repair boundary

The archived staging function uploads each file with stdin redirected from the
artifact, then invokes a verification SSH child without redirecting its stdin.
The surrounding `while read` loop reads the registered-data manifest on FD0.
The archived native SSH adapter execs SSH without `-n`. Its verification child
can therefore consume the remaining manifest rows, leaving only the first data
file staged while the shell loop still exits successfully.

Four missing files are a direct observation. The precise SSH byte consumption
was not syscall-traced. An independent offline stub using the exact archived
`stage_file` function reproduced one-of-five staging with a child that consumes
stdin, and five-of-five with manifest input isolated from child stdin. Both
cases passed without SSH, model, provider or corpus-scoring execution.

The prior25+11 metadata tests did not exercise this transport edge. Both the
author and independent reviewer acknowledge that review miss. A prospective
bounded repair isolates child stdin, reads the manifest on a separate descriptor
and requires all five staged data roles to complete. Its regression must invoke
the actual staging code with a consuming transport stub and verify transfer
bytes, destinations, ordering and immediate SHA checks. This is the same failed
staging contract, not a new model test programme.

Battery2 and its evidence remain unchanged. Any successor packet requires its
own exact review and a new distinct PM release; no on-lease patch or automatic
paid retry is authorized. The law, config, data, source/binary identities,
scientific predicates, frozen timing3 and learning-dose hold are unchanged.

## Preservation and independent audit

The [independent actual-outcome verdict6242413809](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6242413809)
accepts a preserved `BLOCKED_STAGING_DATA` receipt. Its report SHA is
`7b56233609cd21bd5c8ea903f8672d6d38ce94f25385b3e2d78795a5c4867726`.
It independently checks31 collected,71 operator,20 original-job,44 reviewed-packet
and5 late-closure manifest entries, all PASS. These inventories overlap and are
not summed as unique files. Private provider/CREATE/up/lease logs are retained
privately and are not published here.

Main archive: `icloud:UOR-R4/results/codex/attention-preflight-battery2-archive.tar`,
6309888bytes, MD5 `7e2afaa3df2943dc9cd98b87c5ef0b01`. The completed cloud receipt
precedes normal release/down in the reviewed operator. Cloud receipt completion
is22:33:58Z; native release also has second-resolution timestamp22:33:58Z, so
no finer order is inferred from equal timestamps alone. Release output completed
22:34:00Z; native down is22:34:03Z, with output completed22:34:04Z. Source order
and complete receipt support archive-before-normal-down. Retained fleet-after
reports0pods; this is a dated observation, not authority over peer jobs.

Late closure: `icloud:UOR-R4/results/codex/attention-preflight-battery2-closure.tar`,
23552bytes, MD5 `c95491c56bc36bf83bcadd4b387fc104`, completed22:34:34Z after down.
It remains a separate lifecycle supplement. The reviewed operator exited1;
both cloud operations completed0. The independent local transport fixture SHA
is `de0e8d24d5b745da940bc29a9307d5177db43ff0a4619c11d26afb43d9f579b1`.
The post-outcome review report, fixture, stdout and manifest-check outputs are
separately preserved at
`icloud:UOR-R4/results/codex/attention-battery2-outcome-audit.tar`,48128bytes,
MD5 `d0bc0caef1d8f78a43ed4a7877afe93a`. This later audit bundle is not represented
as evidence that existed before pod teardown.

## Cost and next dependency

Provider-recorded uptime is188s. The raw quoted rate$0.9635111111111112/hour
gives a$0.050316691 uptime proxy; rounded recorded$0.964/hour gives
$0.050342222. The separate191s up/down wall interval gives a$0.051119617
raw-rate proxy or$0.051145556 rounded-rate proxy. These are alternatives, not
summed charges or invoices. The carried mixed Codex estimate, adding the
rounded-rate uptime proxy, is$4.644607777 plus PM; storage/transfer UNMEASURED.

Battery3 is **conditionally ready for distinct PM dispatch; NOT_RELEASED /
NOT_RUN**, per [independent prospective verdict6242500425](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6242500425)
and [final frozen-packet handoff6242520855](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6242520855).
[AGY confined packet APPROVE6242609372](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6242609372)
completes the required packet reviews. Next is a new distinct actual PM dispatch
and fresh admission, then execution of the same reviewed battery phases.
No new compiler run, timing series, mechanism bar or host model reconstruction
is implied. A successful battery must still precede any separately released
paired learning dose; a token map alone cannot establish same-cell attribution.
