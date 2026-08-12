# Prime Transport Router Memory Task Loop

## Purpose

This note records the first bounded multi-step task loop that uses the
prime-transport router memory layer as an actual read/write memory substrate
rather than only as a replayed routing structure.

## Chosen Task Family

The chosen task family is:

- bounded structured state editing on the existing tightly matched exact trace
  families

At each step, the loop:

1. forms the current router-memory address from the active `R_min` state
2. queries what payload was previously stored at that address
3. writes a new deterministic symbolic payload for the current step
4. promotes only if the current route class is still unresolved

The payloads are small symbolic state edits derived deterministically from the
existing trace state:

- field name
- current gap
- leading phase coordinate
- short spin mass

This is a more honest memory test than replay-only route evaluation because the
loop now requires:

- repeated writes into persistent memory cells
- later reads of earlier stored state
- selective promotion when coarse route memory is insufficient
- correctness scoring on carried state, not just on route partition behavior

## Success / Failure Criteria

Success means:

- high read-hit rate on reused addresses
- high retrieval accuracy on later reads
- stable promotion behavior
- zero decision instability

Failure would mean:

- low carry-over accuracy after addresses are revisited
- unstable route decisions
- or collapse into indiscriminate promotion just to preserve correctness

## Implemented Driver

- `tools/prime_transport/run_router_memory_task_loop.py`

The driver uses the existing:

- `tools/prime_transport/router_memory_layer.py`

and the existing bounded exact trace family already used by earlier routing
prototype work.

## Result Artifact

- `results/prime_transport_recursive_system/prime_transport_router_memory_task_loop.csv`

## Aggregate Result

Across the bounded task-loop family:

- route reuse fraction: `0.8151848151848151`
- query hit fraction on reuse: `1.0`
- task retrieval accuracy: `0.9300522166708776`
- promoted query fraction on reuse: `0.48887856062924245`
- promotion step fraction: `0.45274725274725275`
- effective task resolution fraction: `0.7386524819618124`
- route decision instability: `0.0`

## Reading

The result is structurally encouraging.

What is now demonstrated:

- once route classes are reused, the memory layer does carry state forward
- reads on reused addresses hit reliably
- carried symbolic payloads are retrieved correctly at a high rate
- promotion remains selective and stable rather than chaotic

What remains limited:

- deeper lift families still require substantial promoted querying
- so the current memory layer is already a coherent substrate, but not yet a
  cheap universal memory solution

This is exactly the architectural pressure that should be visible at this
stage: the router-memory layer works, but its cost still rises where coarse
memory classes are too entangled.

## Conclusion

Yes: the router memory layer is strong enough to continue toward a larger
router-native architecture experiment.

The reason is now stronger than before:

- the layer no longer only routes cleanly
- it now supports actual state carry, later retrieval, selective promotion,
  and stable multi-step task execution

The main caution is unchanged in form but clearer in meaning:

- the larger next experiment should target how to reduce promoted-query burden
  on deeper families without losing the stability and retrieval accuracy
  already demonstrated here
