Geometric Routed Prompt Engine for MUDBench

Overview

MUDBench currently evaluates agents in bounded text worlds with partial observation, explicit action constraints, temporal continuity, and shared state. These properties make it a useful environment not only for agent benchmarking, but also for testing prompt-synthesis strategies under consequential interaction.

This note proposes an optional geometric routed prompt engine for MUDBench.

The key idea is that model-facing context should not be treated as one flat prompt blob. Instead, context should be organized as a set of layered state loops, and a routing layer should decide how to synthesize, weight, order, or compress those layers at each turn.

This makes MUDBench useful for two purposes at once:
	1.	evaluating agent behavior in shared consequential worlds
	2.	evaluating whether a routing engine improves model performance under layered structural prompting

The proposed routed prompt engine is an optional experimental path, not a replacement for the current baseline runtime.

⸻

Motivation

The problem with flat prompting

In benchmark and game settings, the effective context seen by an agent is rarely one-dimensional. A model acting inside MUDBench is influenced simultaneously by multiple live loops:
	•	immediate local affordances
	•	objective and dependency state
	•	world tick / timing pressure
	•	resource preservation or expenditure
	•	multi-agent interaction
	•	persistence across turns

A flat prompt can include all of this, but it does not distinguish which layer matters most right now. This makes several failure modes more likely:
	•	irrelevant context dominates the turn
	•	the model overfocuses on visible local text and ignores delayed constraints
	•	temporal or shared-world pressure is buried
	•	prompt growth becomes noisy rather than structured
	•	the model produces valid but strategically poor actions

Why routing may help

A routing layer can treat prompt assembly as a structured synthesis problem rather than a concatenation problem.

Instead of asking:
	•	“what text do we include?”

it asks:
	•	“which loop layers are most behaviorally important on this turn?”
	•	“what ordering and weighting should those layers receive?”
	•	“how should low-priority context be compressed or omitted?”

This is especially relevant in MUDBench because the benchmark already contains multiple distinct reasoning pressures:
	•	dependency and unlock reasoning
	•	route and hazard tradeoffs
	•	delayed-cost planning
	•	shared-world timing effects
	•	multi-participant world continuity

These give the routing engine something meaningful to optimize against.

⸻

Core idea

The routed prompt engine treats agent context as a set of layered state spaces or interaction loops, each corresponding to a different domain of relevance and a different persistence timescale.

The routing engine does not change the action contract, parser, repair logic, or fail-closed behavior. It only changes how the model-facing prompt is assembled from available context.

Baseline path
	•	observation payload is assembled
	•	prompt is built in a fixed canonical order
	•	model responds
	•	parser, repair, and fail-closed logic apply

Routed path
	•	observation payload is assembled
	•	context is decomposed into loop layers
	•	the router scores, phases, or ranks those layers
	•	prompt is synthesized using routed ordering/weighting/compression
	•	model responds
	•	same parser, repair, and fail-closed logic apply

This allows A/B comparison between:
	•	canonical baseline prompting
	•	routed prompting

under otherwise identical world and agent conditions.

⸻

Layered structural prompting

The routed engine is based on the existing MUDBench idea of layered structural prompting.

A turn can be decomposed into at least the following loop layers.

1. Immediate action loop

This layer captures what is legal and visible right now.

Examples:
	•	current room description
	•	visible exits
	•	legal actions
	•	legal targets
	•	immediate hazard notices

This layer is critical when the action space is narrow or tightly constrained.

2. Local objective loop

This layer captures what the agent is currently trying to accomplish and what dependencies remain.

Examples:
	•	retrieve relic
	•	unlock vault
	•	conserve item for later gate
	•	cross blocked corridor at the right time

This layer is critical when local affordances are not enough to determine good action.

3. Temporal world loop

This layer captures changing state that is driven by time or world ticks rather than direct agent action.

Examples:
	•	stance phase
	•	route availability windows
	•	timed hazards
	•	delayed unlock opportunities

This layer becomes critical when a correct action depends on when the world is in a particular state.

4. Multi-agent loop

This layer captures the influence of other participants on the same shared world.

Examples:
	•	another participant occupying the route
	•	another participant taking the item first
	•	turn sequencing
	•	cooperative or interfering action patterns

This layer matters most in shared-shard or mixed human/agent play.

5. Persistence or self loop

This layer captures what should remain coherent across repeated turns.

Examples:
	•	items already collected
	•	earlier commitments
	•	recent failed attempts
	•	current strategic posture
	•	remembered timing or phase behavior

This layer matters whenever one-turn local correctness is insufficient.

⸻

Hypothesis

The main working hypothesis is:

Prompt quality in shared consequential environments can improve when context is routed according to the currently dominant loop layers rather than always presented in a fixed flat order.

A stronger but still bounded version is:

Different scenario types and world states activate different loop priorities, and an effective routing engine should be able to shift prompt emphasis accordingly.

Examples:
	•	in a dependency scenario, objective/dependency context should dominate
	•	in a timing scenario, temporal world context should dominate
	•	in a delayed-cost scenario, persistence/resource context should dominate
	•	in shared play, multi-agent context should matter more than in single-agent slices

⸻

Proposed routing model

The first version should stay simple and deterministic enough to evaluate cleanly.

Input

The router sees the same canonical structured observation payload already available to the runtime.

Output

The router produces a prompt assembly plan, such as:
	•	loop priority order
	•	inclusion/exclusion choices
	•	compression levels
	•	emphasis tags
	•	routing metadata for evaluation

Minimal routing plan shape

An initial routing plan might include:
	•	dominant_loop
	•	secondary_loops
	•	prompt_section_order
	•	compressed_sections
	•	reasoning_pressure_tag

For example:

{
  "dominant_loop": "temporal_world",
  "secondary_loops": ["immediate_action", "local_objective"],
  "prompt_section_order": [
    "temporal_world",
    "immediate_action",
    "local_objective",
    "persistence",
    "multi_agent"
  ],
  "compressed_sections": ["multi_agent"],
  "reasoning_pressure_tag": "phase_timing"
}

This would allow the actual prompt builder to remain deterministic while still being routed.

⸻

Geometric routing intuition

The routing engine is motivated by a geometric view of context relevance.

Rather than thinking of prompt sections as a list, the router can think of them as points or regions in a relevance landscape shaped by:
	•	current world constraints
	•	phase/timing state
	•	objective dependencies
	•	recent action history
	•	shared participation patterns

In this framing:
	•	context blocks that lie “closer” to the current decision pressure are elevated
	•	context blocks that are structurally distant are compressed or deferred
	•	phase state can act like a field that changes the routing geometry

This is the point where MUDBench becomes useful to the routing project itself: it provides repeated, measurable, world-grounded decision points where routing pressure is not arbitrary.

⸻

Initial implementation boundary

The first routed prompt engine should be strictly bounded.

It should do:
	•	operate as an optional runtime mode
	•	reuse the same observation payload
	•	reuse the same action parser
	•	reuse the same repair and fail-closed logic
	•	produce comparable outputs against the baseline path

It should not do:
	•	replace the whole runtime
	•	introduce provider-specific prompt logic
	•	change the observation/action contract
	•	add free-form natural language planning layers
	•	mutate world rules
	•	hide invalid-output behavior

The goal is to isolate the routing layer as the experimental variable.

⸻

Candidate evaluation metrics

The routed path should be compared against the baseline path using existing MUDBench signals wherever possible.

Behavioral metrics
	•	objective completion rate
	•	aggregate/composite score
	•	resource-use correctness
	•	route-choice correctness
	•	timing correctness

Runtime metrics
	•	invalid output count
	•	repair-used count
	•	fail-closed count
	•	final parse success rate

Shared-world metrics
	•	coherence across repeated turns
	•	successful use of world phase timing
	•	reduced confusion under shared-state pressure
	•	possible cooperation or conflict outcomes

Slice-specific metrics
	•	guarded relic: dependency/unlock success
	•	hazard route: safe-vs-risky route handling
	•	delayed cost: save-now vs spend-later success
	•	shared shard: handling of phase-linked route constraints

⸻

Why MUDBench is a strong testbed for the router

MUDBench is well suited to routing evaluation because it combines several things that most prompt experiments lack:

1. Bounded action contract

The model cannot hide failure behind prose.

2. Shared consequential state

Wrong decisions have persistent consequences.

3. Temporal structure

The world can change across turns even when the agent does not act.

4. Multiple reasoning pressures

The current slice set already includes dependency, route, timing, and delayed-cost pressures.

5. Human + agent participation

The same world loop can host both human and agent participants, making shared-world routing more meaningful.

This gives the router a real environment to succeed or fail in, rather than a purely textual benchmark.

⸻

Suggested phased implementation

Phase 1: Design-only
	•	define loop layers formally
	•	define routing-plan schema
	•	define baseline vs routed comparison method

Phase 2: Optional routed prompt builder
	•	add routed prompt assembly mode
	•	keep canonical baseline prompt builder unchanged
	•	no provider changes beyond existing path reuse

Phase 3: Slice comparison

Run baseline vs routed across:
	•	guarded relic
	•	hazard route
	•	delayed cost

Measure:
	•	completion
	•	parse quality
	•	repair rate
	•	fail-closed rate

Phase 4: Shared-shard comparison

Test routed prompts in:
	•	human + agent shared loop
	•	human + external wrapper shared loop
	•	human + direct-provider shared loop

Evaluate whether routing improves:
	•	timing against phase-linked world pressure
	•	persistence across turns
	•	action boundedness under richer shared-state context

⸻

Safety relevance

The routed prompt engine is interesting not only for performance but for safety.

If a routing layer can improve:
	•	protocol adherence
	•	bounded action selection
	•	temporal coherence
	•	resource discipline
	•	multi-agent handling

then it may help reduce the gap between:
	•	one-shot intelligence
and
	•	safe behavior inside evolving environments

This is especially relevant because many practical failures in agentic systems arise not from raw inability, but from context mis-weighting:
	•	focusing on the wrong part of the state
	•	forgetting delayed constraints
	•	ignoring shared-world effects
	•	overreacting to immediate local cues

A routing layer may help correct those failures.

That said, MUDBench would test only a reduced form of this problem. It would not prove general safe agency.

⸻

Current limitations

This proposal should stay grounded.

Current limitations include:
	•	the existing world is still small and deterministic
	•	world-side phase behavior is still narrow
	•	shared-shard behavior is local and bounded
	•	no full open-world scheduling exists
	•	no broad persistence/restore framework exists yet
	•	the direct-provider path is still narrow
	•	routing success in MUDBench would not automatically transfer to physical robotics or unrestricted environments

So the router should be presented as:
	•	a bounded experimental synthesis layer
not
	•	a universal solution to intelligence

⸻

Conclusion

MUDBench has reached the point where it can serve not only as a benchmark for agent behavior in shared consequential text worlds, but also as an experimental environment for a geometric or phase-aware prompt routing engine.

The key value is that MUDBench already exposes layered structural prompting in practice:
	•	immediate action pressure
	•	objective pressure
	•	temporal world pressure
	•	multi-agent pressure
	•	persistence pressure

A routed prompt engine can therefore be tested against real behavioral outcomes rather than aesthetic intuitions about prompt quality.

If successful, this would make MUDBench useful for two aligned purposes:
	1.	benchmarking agents in consequential shared worlds
	2.	evaluating whether geometric routing improves coherence and bounded behavior under layered interaction
