# Prime Transport Multi-Entity Router Memory Family

## Purpose

This note defines the first bounded multi-entity task family used to test the
current router-memory layer as a structured memory substrate for interacting
records.

## Task Family

The chosen family is:

- bounded multi-entity world-state coordination

Each bundle contains three active entities. Each entity carries its own
structured record with fields:

- `goal`
- `world`
- `combat`
- `pressure`
- `revision`

At each global step:

1. every active entity queries its current carried record
2. the system checks whether all active entity reads are jointly correct
3. each entity applies one partial record update
4. each updated record is written back through the unchanged router-memory
   layer

So the task requires:

- multiple records active at once
- partial updates to different records over time
- later reads that depend on the current state of more than one record
- bounded reproducible evaluation

## Why This Is The Right Next Step

This is the smallest larger router-native systems test compatible with the
current branch because it adds:

- concurrent active records
- coordinated multi-record reads
- persistent partial updates across entities

without changing:

- the router-memory state
- the read path
- the promotion logic

## Success / Failure

Success means:

- exact multi-record retrieval on reused addresses
- exact coordinated snapshot reads across active entities
- bounded promoted-query burden
- zero instability

Failure would mean:

- carried-state corruption when several entities remain active together
- collapse of coordination reads
- or unstable promotion behavior
