# Prime Transport Router Memory Record Family

## Purpose

This note defines the first larger bounded task family used to test the
router-memory layer as a genuine structured memory substrate.

## Task Family

The chosen task family is:

- bounded structured record maintenance over the existing tightly matched exact
  trace families

Each reusable route class now carries a small multi-field record:

- `goal`
- `world`
- `combat`
- `pressure`
- `revision`

At each step:

1. the current record is queried from memory
2. exactly one field is updated
3. the full updated record snapshot is written back
4. promotion is triggered only if the existing route class remains unresolved

So this task family is richer than the earlier bounded state-editing loop in
one key way:

- correctness now depends on retrieving and preserving a structured record that
  is repeatedly edited over time, not just a single symbolic payload

## Why This Is A Better Router-Native Test

This family exercises the architecture more honestly than replay-only routing
or one-payload memory loops because it requires:

- repeated writes
- repeated reads
- multi-step state carry
- partial updates to existing memory
- structured correctness across time

It therefore probes whether the current memory layer can behave like a small
stateful working memory rather than just a route-local cache.

## Success / Failure

Success means:

- high structured-record retrieval accuracy
- stable read-hit behavior on reused addresses
- bounded promotion burden
- zero instability

Failure would mean:

- record corruption under partial updates
- loss of carried state across revisits
- or instability in the promotion/read path

## Current Use

This task family is used by:

- `tools/prime_transport/run_larger_router_memory_experiment.py`

with results written to:

- `results/prime_transport_recursive_system/prime_transport_larger_router_memory_experiment.csv`
