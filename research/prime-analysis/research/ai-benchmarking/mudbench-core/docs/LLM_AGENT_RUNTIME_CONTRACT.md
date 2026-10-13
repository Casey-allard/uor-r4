# LLM Agent Runtime Contract

## Status

- Type: implementation-oriented design note
- Scope: first serious model-facing runtime contract for MUDBench gameplay
- Intent: define the smallest concrete contract that can move MUDBench from deterministic script agents and thin wrapper examples toward a real LLM gameplay loop without weakening benchmark auditability

## 1. Why This Note Exists

MUDBench already has a usable agent boundary:

- canonical `Observation` payloads in `src/agents/protocol/observation.py`
- canonical `ActionSubmission` parsing in `src/agents/protocol/action.py`
- deterministic local-process ingress through `src/agents/local_runner/process_bridge.py`
- optional persistent local-process sessions through `--persistent-agent-session`
- exact observation/action replay logging through the current replay pipeline

What is still underbuilt is the model-facing cognition/runtime layer between:

- MUDBench's strict observation/action contract
- a wrapper process that calls an LLM
- a future direct-provider runtime that removes the extra wrapper process

Today the repo mostly demonstrates:

- deterministic rule agents
- strict JSON wrapper examples
- mock single-shot LLM-style prompting

This note defines the first serious runtime contract for a real model-facing play loop while staying grounded in those existing seams.

## 2. Goals and Non-Goals

### 2.1 Goals

The first LLM runtime contract should:

- reuse the current `Observation` and `ActionSubmission` protocol as the authoritative simulation boundary
- define one canonical model-facing observation payload derived from the existing observation object
- define one strict prompt and output contract for single-turn benchmark play
- define the minimum prompt/runtime differences needed for persistent-session mode
- make invalid output handling explicit and replay-auditable
- bound token, history, and memory behavior so benchmark runs stay comparable
- create a clean migration path from:
  - deterministic script agents
  - wrapper-based local-process LLM agents
  - future direct-provider LLM mode

### 2.2 Non-Goals

This first contract should not:

- redesign the simulation protocol
- replace the current local-process runner
- require network-provider integration yet
- introduce hidden memory stores or retrieval systems
- allow unconstrained free-form tool use inside benchmark runs
- silently broaden benchmark semantics relative to deterministic script agents
- define the full persistent-world cognition stack for long-lived shards

## 3. Agent Classes and Contract Boundaries

MUDBench now needs three distinct runtime categories with a shared action boundary.

### 3.1 Deterministic Script Agents

Examples:

- built-in deterministic scripts embedded in `benchmark_runner/runner.py`
- example rule agents under `examples/agents/`

Properties:

- no model call
- no prompt construction
- action chosen directly from current observation
- deterministic by implementation

These remain the baseline and regression anchor.

### 3.2 Wrapper-Based LLM Agents

Examples:

- `examples/agents/mock_llm_wrapper.py`
- future real wrappers that read one observation JSON line, call a model, then return one action JSON line

Properties:

- MUDBench still launches a local process
- wrapper owns prompt construction, model call, and repair handling
- MUDBench only sees canonical observation in and canonical action out

This is the first serious implementation target.

### 3.3 Future Direct-Provider LLM Mode

Properties:

- MUDBench runtime calls a provider SDK directly
- prompt assembly, token accounting, repair, and retries move into MUDBench-controlled code
- simulation boundary remains the same `Observation` in and `ActionSubmission` out

This mode should inherit the same model-facing contract rather than inventing a different one.

## 4. Canonical Model-Facing Observation Payload

The model-facing observation payload should be a strict superset-style rendering of the current `Observation.to_dict()` payload, not a parallel schema.

### 4.1 Canonical Rule

For the first runtime contract:

- the canonical cognition input is the existing protocol observation
- field names should match the current protocol exactly where possible
- wrapper-level prompt builders may add presentation text, but not change meaning

Required base fields remain:

- `run_id`
- `step`
- `location`
- `description`
- `exits`
- `entities`
- `inventory`
- `health`
- `messages`
- `action_space`
- `remaining_steps`
- `protocol_version`

### 4.2 Optional Runtime Metadata

The first serious LLM runtime may derive a small runtime envelope around that observation for prompting, but it should remain explicitly non-authoritative. Allowed derived metadata:

- `mode`: `benchmark_single_turn` or `persistent_session`
- `actor_id` if the wrapper/runtime already knows the local identity label
- `response_format`: the required action JSON shape
- `action_selection_rule`: reminder that only one action may be returned

These fields should not change simulation semantics. They only help prompt assembly.

### 4.3 Canonical Observation Example

```json
{
  "mode": "benchmark_single_turn",
  "response_format": {
    "type": "json_object",
    "required_fields": ["action"]
  },
  "observation": {
    "run_id": "tiny-fetch-quest-run",
    "step": 2,
    "location": "corridor",
    "description": "A narrow stone corridor.",
    "exits": ["east", "west"],
    "entities": [
      {"type": "npc", "name": "guard-1"},
      {"type": "item", "name": "golden-key"}
    ],
    "inventory": [],
    "health": 100,
    "messages": [
      "The guard blocks part of the hall."
    ],
    "action_space": [
      "move east",
      "move west",
      "attack guard-1",
      "take golden-key",
      "look",
      "wait"
    ],
    "remaining_steps": 3,
    "protocol_version": "1.0"
  }
}
```

## 5. Prompt Structure for Single-Turn Benchmark Play

The first benchmark-mode prompt should remain intentionally narrow.

### 5.1 Prompt Goals

The prompt should:

- describe the task as choosing exactly one next action
- serialize the current observation faithfully
- remind the model that only actions in `action_space` are valid
- require strict JSON output
- avoid hidden multi-step planning state beyond the current turn

### 5.2 Prompt Shape

Recommended structure:

1. system/runtime instruction
2. action validity rule
3. output schema rule
4. serialized observation
5. final response instruction

### 5.3 Benchmark Prompt Example

```text
You are controlling one MUDBench benchmark agent for exactly one step.

Rules:
- Choose exactly one next action.
- Your action must be a string taken exactly from observation.action_space.
- Do not explain your reasoning.
- Return JSON only.
- The JSON object must contain exactly one required field: "action".

Observation:
{
  "run_id": "tiny-fetch-quest-run",
  "step": 2,
  "location": "corridor",
  "description": "A narrow stone corridor.",
  "exits": ["east", "west"],
  "entities": [
    {"type": "npc", "name": "guard-1"},
    {"type": "item", "name": "golden-key"}
  ],
  "inventory": [],
  "health": 100,
  "messages": ["The guard blocks part of the hall."],
  "action_space": [
    "move east",
    "move west",
    "attack guard-1",
    "take golden-key",
    "look",
    "wait"
  ],
  "remaining_steps": 3,
  "protocol_version": "1.0"
}

Return the action now.
```

## 6. Prompt and Runtime Differences for Persistent-Session Mode

The current codebase already supports persistent local-process sessions. That is enough to define the first persistent-session prompt delta even before full shard gameplay exists.

### 6.1 What Changes

Persistent-session mode differs from single-turn benchmark mode in that:

- the wrapper process may stay alive across turns
- the model-facing loop may include limited prior-turn context
- the session may represent an ongoing actor identity rather than a bounded benchmark run only

### 6.2 What Must Not Change Yet

Even in persistent-session mode:

- the world-facing contract is still one observation in, one action out
- the runtime should not depend on hidden external memory for correctness
- the prompt should not assume open-ended autonomous tool use
- replay must still be able to explain which observation led to which action

### 6.3 Prompt Delta

Persistent-session prompting may add:

- session continuity banner
- prior accepted action summary
- short rolling memory summary owned by the wrapper/runtime

It should not yet add:

- free-form diary memory
- unbounded transcript history
- off-protocol world state not present in observed or explicitly runtime-derived data

### 6.4 Persistent-Session Prompt Sketch

```text
You are controlling one MUDBench agent in a persistent local session.

Current session rules:
- Choose exactly one next action for this turn.
- Your action must match one entry in observation.action_space exactly.
- Use the short memory summary only as a convenience; the current observation is authoritative.
- Return JSON only with field "action".

Session memory summary:
- Prior accepted action: "move east"
- Last known goal-relevant fact: "A locked gate was seen south of entry."

Current observation:
{
  "run_id": "dev-shard-session",
  "step": 18,
  "location": "lock-ante",
  "description": "A narrow ante-chamber with a barred northern door and a passage south.",
  "exits": ["south"],
  "entities": [],
  "inventory": ["brass-key"],
  "health": 100,
  "messages": ["The barred door is still shut."],
  "action_space": [
    "move south",
    "use brass-key",
    "look",
    "wait"
  ],
  "remaining_steps": 982,
  "protocol_version": "1.0"
}
```

## 7. Strict Action Output Contract

The first LLM runtime contract should keep the output contract as strict as the current parser.

### 7.1 Canonical Output

Required:

- JSON object
- required field `action`
- `action` must be a non-empty string

Optional in wrapper/direct mode but ignored by the simulation boundary:

- `protocol_version`

No other fields should be required for the first serious runtime.

### 7.2 Valid Output Example

```json
{
  "action": "use brass-key"
}
```

### 7.3 Why the Contract Stays Small

This keeps alignment with:

- `ActionSubmission.from_dict(...)`
- `parse_action_submission_payload(...)`
- current local-process STDIN/STDOUT exchange

The model contract should not require a richer plan object before MUDBench has a stable need for it.

## 8. Invalid Output Handling, Retry, and Repair

This is the most important underbuilt part of the current wrapper path.

### 8.1 Failure Types

The first runtime should distinguish:

- invalid JSON
- non-object JSON
- missing `action`
- non-string `action`
- action not in `action_space`
- timeout / no response

### 8.2 Repair Rule

The first serious repair policy should be single-repair and bounded:

1. attempt primary model completion
2. parse result against strict output contract
3. if parsing fails, send one repair prompt using the same observation plus the parser error
4. if repair fails, return a deterministic runtime failure to MUDBench

Benchmark mode should not allow unbounded retries because that weakens comparability and timing fairness.

### 8.3 Repair Prompt Requirements

Repair prompts should:

- quote the exact parser/validation failure
- restate the allowed `action_space`
- demand JSON-only output
- avoid adding new world facts

### 8.4 Invalid Output Repair Example

Initial invalid model output:

```text
I would attack the guard first.
```

Repair prompt:

```text
Your previous response was invalid for MUDBench.

Failure:
- response was not a JSON object with a string field named "action"

Return JSON only.
The action must exactly match one item from this action_space:
["move east", "move west", "attack guard-1", "take golden-key", "look", "wait"]
```

Repaired valid output:

```json
{
  "action": "attack guard-1"
}
```

### 8.5 Runtime Consequences

Wrapper-based mode:

- wrapper may implement the one-repair policy internally
- MUDBench still receives either one valid action or one runner/protocol failure

Future direct-provider mode:

- MUDBench runtime should implement the same bounded repair policy directly
- repair attempts and failures should be logged as runtime telemetry

## 9. Token, History, and Memory Policy Boundaries

### 9.1 Benchmark Mode

For official benchmark-oriented prompting:

- default to current-turn observation only
- allow a fixed instruction scaffold
- do not include unbounded prior transcript
- do not include hidden persistent memory
- record token and latency metrics when available

The benchmark contract should prefer comparability over maximal capability.

### 9.2 Persistent-Session Mode

For persistent local sessions:

- allow a small bounded rolling summary
- keep the current observation authoritative
- bound history by count or token budget
- require deterministic truncation order

Recommended first boundary:

- one fixed system scaffold
- current observation
- optional short runtime-owned memory summary
- optional last accepted action summary

### 9.3 Memory Ownership Rule

The first runtime contract should distinguish:

- simulation truth: current observation payload
- runtime-owned convenience memory: bounded summary derived from prior accepted steps
- future world memory systems: not part of this first contract

That prevents wrappers from quietly inventing a second state system that replay cannot explain.

## 10. Replay, Reporting, and Observer Implications

The runtime contract should be observable through existing MUDBench surfaces.

### 10.1 Replay Requirements

Replay should continue to capture:

- exact observation delivered
- accepted action submitted
- runtime failure if no valid action was produced

Future enhancement, still consistent with this note:

- optional prompt/runtime telemetry sidecar
- invalid-output repair count
- latency/token usage summary

### 10.2 Reporting Requirements

The first LLM runtime contract should make it possible to compare:

- deterministic script baselines
- wrapper-based LLM agents
- future direct-provider agents

using the same benchmark scorecards and replay semantics.

### 10.3 Observer Requirements

The existing observer/reporting stack should not need a new action schema.
If richer LLM runtime telemetry is added later, it should appear as additive metadata, not a replacement for observation/action replay drilldowns.

## 11. Benchmark Prompting vs Future Persistent-World Prompting

The two regimes should share the same action boundary but differ in runtime posture.

### 11.1 Benchmark Prompting

Benchmark prompting should be:

- turn-local
- strict
- low-memory
- action-space bounded
- easy to replay and audit

### 11.2 Future Persistent-World Prompting

Persistent-world prompting will likely need:

- identity/session framing
- bounded medium-horizon memory
- persistent-session context
- shard/season-safe policy reminders

But the first persistent-world runtime should still inherit the same strict output contract and bounded retry policy.

### 11.3 Architectural Rule

The benchmark contract is the narrow core.
Persistent-world prompting should extend that core, not fork it into an incompatible protocol.

## 12. Phased Implementation Path

### Phase 1: Wrapper Contract Hardening

Implement around the current local-process path:

- publish this contract note
- add one real LLM wrapper example that follows it
- standardize one-repair invalid output handling inside the wrapper
- log wrapper runtime failures distinctly from world validation failures

### Phase 2: Runtime Telemetry Surfacing

Add optional telemetry capture for wrapper-based LLM calls:

- latency
- token counts when available
- repair count
- timeout/error category

This should flow into replay/report artifacts as additive metadata.

### Phase 3: Direct LLM Runtime Mode

Add a direct-provider runner inside MUDBench that:

- consumes the same canonical observation payload
- assembles the same prompt contract
- enforces the same strict output parsing
- applies the same bounded repair policy

This is the first true model-native gameplay mode.

### Phase 4: Persistent Session Runtime Layer

Extend the direct runtime for persistent local/world sessions:

- bounded rolling memory summary
- explicit session-mode prompt variant
- stable telemetry for long-lived sessions

### Phase 5: Shard/World Integration

Integrate the same runtime contract into shard/world mode:

- session-aware actor identity
- persistent shard session orchestration
- shard export and observer summaries for model runtime health

This phase should still preserve the benchmark contract as the narrow audited core.

## 13. Recommended First Implementation Boundary

The first code-level implementation should target:

- local-process wrapper-based LLM agents
- current protocol version `1.0`
- one-step benchmark play
- optional persistent local-process session continuity
- strict JSON action output
- one bounded repair attempt

That is the smallest serious runtime contract that fits the current MUDBench codebase and can later be lifted into a direct-provider runtime without redoing the action boundary.
