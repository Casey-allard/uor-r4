# Exact NoWrite admission — #1139, 2026-09-05

**Retain exact NoWrite reuse as an execution improvement. Geometric superiority
and the full #1139 handoff remain unestablished.** The selected sparse artifact is
`067adbf0a14d931d2ad98ca28a06a92aa1b963d0587183bf70b8085ffb7a4df6`.
The geometric comparison is `0e9c4e9e591349900176810439857e062d51d8de7db0edec659069c8819d87d9`;
the collapsed-partition control is
`2cf8bd3eae655e46ae574059432d92e21ff3b94b62d1869eecfbefba01b8b449`.
All derive from the accepted #1138 parent `793cb9ad`, delivered by protected
PR #1144 at `39e35c54b5bc6fb6a9f6be164528703bd62f3b9d`.

## Mechanism and information boundaries

The unchanged learned /2 writer was repeatedly scoring NoWrite at almost every
completed source word. Rust construction now collects the most frequent 64
exact negative input signatures from its existing 112 construction prompts.
The writer's learned scores determine NoWrite; frequency chooses retained
shortcuts. There is no new predictive fit, learned route weight, answer table,
semantic parser or change to the reader and exact relation store.

A signature retains the window length and all eight ordered dictionary prime
addresses. The /2 writer consumes those addresses, position-shared participant
masks and their artifact-bound role geometry. Unknown words already map to zero
in that writer. Byte identities of such nonparticipant words are therefore not
distinguished by this signature; this is the parent's existing information loss.
Exact owner/value bytes, source occurrence, revision and conflict remain in the
unchanged relation records. No summary replaces a recoverable fact.

The geometric shortlist composes the known words' fixed H4 elements in source
order and accumulates eight zeta phase channels, retaining four bits per phase.
The full signed H4 element retains orientation, but finite products can collide
and accumulated phases do not retain order. Exact signature guards recover every
distinction relevant to the /2 writer before a shortcut executes. The route is
an address partition, without a semantic distance interpretation.

Two bounded route searches locate a bucket; at most eight exact guards execute.
Larger buckets, unknown signatures and misses fall back to the unchanged full
writer. The sparse comparator binary-searches the same 64 exact signatures in
the same 48-byte entry layout. The destructive control merges all routes into
one bucket, exceeding the cap and forcing fallback. It changes the partition
itself, not just bucket labels.

Artifact validation checks the parent identity, row order, uniqueness, route
construction and every stored NoWrite decision against the original writer.
The old optional-field serialization and old artifact identities are preserved.
No online index maintenance or learned-state update is added. Serving uses
bounded integer comparisons, H4 table reads, phase additions and exact guards.

## Behavior and direct checks

All four arms produce **112/112 OPEN answers and exact write sequences**, then
**28/28 reserved answers and writes** with new names and four times the padding
length (768 filler words). This is the known association/grammar family with
new source vocabulary and longer context, not general conversation or a broader
population. The 112 construction examples are unchanged. All artifacts, the
64-entry/8-guard limits and source selection were fixed before reserved use.

Both useful admission arms preserve all 62 prior responses, 24 role-reader
responses, eight binding outputs and five actual restored/isolated session
reads. Every non-work Generation field and exact relation state agrees with
the parent. All 112 old parent Generation objects and states reproduce exactly
on the changed executable. The actual selected-artifact CLI Generation also
matches the probe. All generated Rust case/binding sources match their earlier
checked files byte-for-byte; no new Rust semantic execution is claimed.

Five focused relation tests and two role/commit tests pass. Tests cover exact
guards, collisions, crowded fallback, source/version persistence and role-path
order. Final review added saturation for the new work counters, matching the
existing telemetry law, with a maximum-counter assertion. This changes no
learned parameters, entries or normal routing decisions. Initial measurements
remain preserved and final timing is repeated after that correction.

## Cost and decision

The representative OPEN prompt has 202 word boundaries and 200 NoWrites.
Both useful arms skip 191 boundaries and reduce writer score-row comparisons
from **2,054,052 to 91,884**. Exact writes, dictionary work, source observation,
store maintenance, response commitment and copying remain. The geometric arm
adds 7,627 route scalar comparisons, 1,721 exact scalar comparisons and 20,243
logical metadata bytes; sparse adds 10,056 exact scalar comparisons and 35,982
logical metadata bytes. Geometry also incurs its separately reported context
probes, H4 reads and phase additions. These byte counters are logical scalar
accesses, not physical cache/DRAM traffic.

The [compact evidence](evidence/native_geometric_relation_admission_1139.json)
records final whole-generation timings and startup cost. Timing includes
encoding, observation, routing, scoring, gathering, operators, state writes and
decoding. File reading, deserialization and validation are measured separately;
OS caches are not flushed. Five rotated rounds use seven longer cases, giving
35 complete generations per arm. The collapse control retains quality but
loses the saving. Sparse and geometric timing overlap; no geometric speed
advantage is established. Reuse is useful for an already loaded model; startup
overhead must be amortized and is not hidden behind warm timing.

Final measurements on the corrected executable:

| Arm | Warm median | Warm p95 | Startup load/validate | One load + 35 generations |
|---|---:|---:|---:|---:|
| Parent | 23.301 ms | 24.164 ms | 2.180 s | 2.998 s |
| Geometric | 1.151 ms | 1.240 ms | 2.444 s | 2.485 s |
| Sparse, retained | 1.110 ms | 1.150 ms | 2.451 s | 2.490 s |
| Collapsed partition | 23.391 ms | 24.046 ms | 2.458 s | 3.279 s |

The sparse warm median is about 21 times faster on this repeated-filler workload.
Including one measured load and all 35 generations reduces total time by about
17%. A single startup-plus-response is slower; the final observed startup
increment is paid back after approximately 13 such responses. The initial timing
sample implied 16, illustrating startup variability. One startup sample per arm
cannot rank small load-time differences. Sparse warm execution is slightly
faster; this experiment does not establish geometric superiority. It does not
establish a 21-fold speedup on arbitrary language or coding workloads.

The selected JSON artifact grows by **21,555 bytes** to 10,728,283 bytes. Its
64 entries occupy 3,072 bytes, with construction provenance and container
overhead additional. The exact relation state remains 2,840 bytes. The compile
report's `persistent_state_bytes_added=0` refers to relation contents, not work
telemetry: eight u64 counters add 64 bytes per RelationWork, or 128 bytes across
the two session work records on the changed executable, equally for all arms.
JSON trace sizes also change. Direct model RSS and retained disk accounting are
reported by the existing monitor. An admission-specific allocation census,
hardware traffic census and general latency distribution are `NOT_RUN`.

Keep #1139 open and #1140 dependent on its actual handoff. This work compiles
exact previously learned decisions; it does not learn a query-dependent route
over persistent relations or establish geometric abstraction transfer. The
next bounded candidate is a conservative upper-bound admission operator for
the residual writer scoring, conditioned on the learned role/H4/zeta features.
It should reject provable NoWrites beyond exact signature hits and fall back
when unresolved. Compare with a plain sparse bound at the same quality and
complete cost. Do not widen this cache or optimize the tiny 16-probe reader
merely to claim another routing result.

## Reproduction and preserved material

Local artifact root: `.uor-models/native-typed-value-2026-09-05/`.
`relation-admission-models/{geometric,sparse,collapsed}.json` are the compared
artifacts. `relation-admission-source.json`, `relation-admission-selection.json`,
`relation-admission-compile-report.json`, OPEN/reserved reports, both timing
reports, preservation/session reports and `relation-admission-checkpoint.json`
retain the inputs, boundaries, results, costs and source identities. Old #1138
artifacts and negatives remain intact. There is no deletion or paid compute.

The existing Rust `native_geometric_value_probe` adds `prepare-admission-source`,
`compile-admission` and `time-admission`; `evaluate-relations` and the existing
preservation/session checks perform the actual generation. The cumulative model
ledger and existing command wrapper charge preparation, compilation, evaluation,
tests and CLI execution. See the checkpoint for final resource usage and the
protected PR for delivery. Queue acknowledgements are not model tests.
