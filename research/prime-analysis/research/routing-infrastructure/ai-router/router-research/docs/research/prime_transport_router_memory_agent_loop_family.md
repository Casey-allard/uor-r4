# Prime Transport Router Memory Agent Loop Family

## Task Family

This task family is a bounded dependency-aware workspace control loop.

Each step requires:

- reading one entity's local record
- reading the shared task ledger
- reading the shared world-status ledger
- reading the shared dependency ledger
- choosing one bounded next action
- writing the resulting local and shared updates back into the same workspace

The action set is intentionally small and deterministic:

- `claim_goal`
- `sync_world`
- `engage_combat`
- `stabilize_pressure`
- `mark_complete`
- `hold_claim`
- `hold_combat`
- `idle`

## Why It Is Stronger Than Passive Workspace Maintenance

Passive workspace maintenance only tests whether local and shared state can be
carried and retrieved correctly.

This control loop is stronger because each step now depends on:

- retrieving current local state
- retrieving current shared state
- choosing an action from that retrieved state
- then updating both local and shared workspace objects consistently

So correctness is no longer just memory retrieval. It is:

- memory retrieval
- decision selection
- and state-transition consistency across repeated steps

## Success / Failure

This task family counts as a success if all of the following hold:

- local retrieval remains accurate
- shared-ledger retrieval remains accurate
- route reuse remains high enough to support repeated reads
- route decision instability stays `0.0`
- bounded action correctness stays strong enough that the loop remains
  interpretable as a coherent control system

This task family counts as a failure if:

- retrieval accuracy collapses
- shared ledgers drift from local state
- instability becomes nonzero
- or action correctness collapses so far that the loop is only a memory test,
  not a bounded control test
