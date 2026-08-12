# Prime Transport Router Memory Status

## Current Architectural Status

This note freezes the current status of the router-memory branch after the
first stateful task-loop success and the first promoted-query reduction test.

## What Is Now Validated

The router-memory branch has now validated all of the following on bounded
research-side workloads:

- structured read/write memory behavior
- stable carried-state behavior
- bounded multi-step task-loop success
- high retrieval accuracy under the current read path

The current validated picture is:

- the memory layer supports explicit state initialization, update, read, write,
  and selective promotion
- repeated route-class visits produce reliable carried-state retrieval
- task-loop reads on reused addresses hit at `1.0`
- aggregate task retrieval accuracy under the current read path is
  `0.9300522166708776`
- route decision instability remains `0.0`

So this branch is no longer only a routing prototype. It is now a viable small
stateful memory substrate.

## Main Remaining Cost

The main remaining practical cost is:

- promoted-query burden on deeper lift families

That burden is real and not just a cosmetic nuisance:

- aggregate promoted-query fraction on reuse: `0.48887856062924245`
- aggregate promotion step fraction: `0.45274725274725275`

And it is overwhelmingly a deeper-family effect:

- share of promoted queries from the `30030 -> 510510` family:
  `0.9701540436456996`

So the current memory layer is viable, but not yet cheap.

## What Was Tested And Rejected

The first tested reduction attempt was:

- a one-key `phi0` read-side refinement

Why it was attractive:

- it was extremely small
- it stayed within the current architecture
- it did materially lower promoted-query burden

Why it was rejected:

- promoted-query fraction on reuse dropped from `0.48887856062924245` to
  `0.21055889917899204`
- but task retrieval accuracy fell from `0.9300522166708776` to
  `0.6832439966572033`
- effective task resolution also fell from `0.7386524819618124` to
  `0.6152483719549754`

So the refinement reduced burden by collapsing too much useful memory
distinction. That is not a good trade.

## Current Interpretation

The correct current interpretation is:

- promoted-query burden should be treated as a real architectural cost
- it should **not** be treated as a solved nuisance
- the current memory branch is viable, but not yet cheap

This is actually a healthy status point:

- the architecture works
- the cost is visible
- the failed cheap refinement clarifies that the next improvement will need to
  be qualitatively stronger than a one-key hint

## Next Decision

The next nontrivial architectural decision is now explicit:

1. continue toward a larger router-native architecture experiment with the
   current read path unchanged

or

2. attempt one larger, qualitatively different read/refinement design

The current recommendation is:

- continue toward the larger router-native architecture experiment with the
  current read path unchanged

Reason:

- the current path is already validated as a stable memory substrate
- the first cheap refinement failed cleanly
- there is not yet evidence that another tiny refinement will solve the cost
  without breaking retrieval quality

So the branch should move forward with the current read path unless a clearly
different refinement design is justified in advance.
