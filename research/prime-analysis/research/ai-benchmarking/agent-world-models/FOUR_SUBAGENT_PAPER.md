# Four Sub-Agent Capability Paper

Date: 2026-06-10

Project: SHD-CCP v2 / LongLongMoa Architecture

## Abstract

This paper defines the intended capability profile for four constrained sub-agents inside the current SHD-CCP v2 architecture. The four-agent split matches the existing routing model used by the web control room:

- factual
- creative
- analytical
- technical

These are not meant to be unconstrained autonomous agents. They are specialized execution roles inside a governed multi-agent pipeline. The system should keep them small, auditable, and safe. Each sub-agent must produce structured output, expose uncertainty, and remain within its allowed scope.

The design goal is simple: improve usefulness without losing control.

## 1. Scope and Design Intent

The current app already has a control-room UI, telemetry ingestion, governance reporting, rollout selection, and a tokenizer abstraction. The four sub-agents proposed here are the next logical specialization layer on top of that scaffold.

They should help the system do four things well:

1. understand the user request
2. explore solution space
3. reason about tradeoffs
4. produce a safe final output

The system should not treat them as free-roaming processes. They are bounded workers with clear interfaces, not independent general-purpose assistants.

## 2. Shared Safety Requirements

Every sub-agent must obey the same base safety rules.

### 2.1 Capability boundaries

Each sub-agent must:

- stay within its assigned role
- decline requests outside that role
- report uncertainty instead of inventing certainty
- preserve provenance for any retrieved or inferred content
- return structured outputs that the selector can compare

### 2.2 Operational boundaries

Each sub-agent must not:

- execute arbitrary commands
- read or write files without explicit orchestration
- access network resources unless the outer system allows it
- modify its own code or configuration
- bypass governance, telemetry, or selector checks

### 2.3 Output rules

Every response from a sub-agent should include, when possible:

- short summary
- confidence estimate
- evidence or source note
- assumptions
- failure mode or abstention reason

This structure makes the final selector safer and more explainable.

## 3. Sub-Agent 1: Factual Agent

### 3.1 Purpose

The factual agent is responsible for grounding the system in known information. It is the closest thing to a retrieval and verification worker.

In the current app, this role aligns with:

- prompt routing for factual requests
- chunk store lookup
- telemetry-backed status reporting
- raw data parsing

### 3.2 Required capabilities

The factual agent must be able to:

- extract explicit facts from a prompt
- locate relevant stored chunks or cached state
- distinguish known facts from guesses
- identify contradictions
- cite the origin of retrieved material
- summarize raw telemetry or runtime state in plain language

### 3.3 Safe behavior

The factual agent must:

- refuse to fabricate sources
- avoid claiming a fact if evidence is missing
- return "insufficient evidence" when retrieval fails
- keep quotations short and faithful
- mark any inference as inference, not fact

### 3.4 Output format

Recommended output fields:

- `summary`
- `facts`
- `evidence`
- `confidence`
- `unknowns`
- `safety_notes`

### 3.5 What success looks like

The factual agent is successful when it can answer:

- "What do we know?"
- "Where did it come from?"
- "What is confirmed versus uncertain?"

## 4. Sub-Agent 2: Creative Agent

### 4.1 Purpose

The creative agent explores alternative formulations, analogies, structures, and conceptual links. Its job is not truth certification. Its job is synthesis and generation.

In the current app, this role aligns with:

- prompt routing for creative requests
- conceptual expansion
- drafting alternate explanations
- generating candidate phrasing for human-readable output

### 4.2 Required capabilities

The creative agent must be able to:

- produce multiple valid variants of a response
- explain the same idea in different styles
- generate metaphors or analogies when helpful
- simplify complex content for human readers
- propose naming, framing, or presentation alternatives

### 4.3 Safe behavior

The creative agent must:

- label speculative content clearly
- avoid presenting invented details as fact
- avoid unsafe persuasion, manipulation, or deception
- stay within the user’s requested scope
- preserve important technical constraints when simplifying

### 4.4 Output format

Recommended output fields:

- `main_draft`
- `alternatives`
- `style_notes`
- `assumptions`
- `confidence`
- `risk_flags`

### 4.5 What success looks like

The creative agent is successful when it can answer:

- "How else can this be expressed?"
- "Which version is clearest?"
- "What explanation is most usable for a human reader?"

## 5. Sub-Agent 3: Analytical Agent

### 5.1 Purpose

The analytical agent decomposes the problem, compares options, and reasons about tradeoffs. It is the system’s structured reasoning layer.

In the current app, this role aligns with:

- coherence evaluation
- telemetry interpretation
- rollout scoring
- decision support

### 5.2 Required capabilities

The analytical agent must be able to:

- break a task into smaller parts
- identify dependencies
- compare competing options
- detect missing prerequisites
- reason about tradeoffs and edge cases
- estimate confidence and failure probability

### 5.3 Safe behavior

The analytical agent must:

- state assumptions explicitly
- separate observation from conclusion
- avoid overfitting to a single signal
- flag contradictions or weak evidence
- recommend abstention when confidence is too low

### 5.4 Output format

Recommended output fields:

- `problem_breakdown`
- `options`
- `tradeoffs`
- `assumptions`
- `confidence`
- `recommendation`

### 5.5 What success looks like

The analytical agent is successful when it can answer:

- "What is the structure of the problem?"
- "What are the viable options?"
- "Which choice is strongest and why?"

## 6. Sub-Agent 4: Technical Agent

### 6.1 Purpose

The technical agent translates intent into implementation detail. It is the practical agent for code, configuration, system behavior, and integration constraints.

In the current app, this role aligns with:

- CLI commands
- API behavior
- runtime configuration
- storage and tokenizer implementation details

### 6.2 Required capabilities

The technical agent must be able to:

- describe implementation steps precisely
- propose code-level changes
- understand runtime and dependency constraints
- produce testable specifications
- identify where code should be edited
- distinguish temporary scaffolds from stable interfaces

### 6.3 Safe behavior

The technical agent must:

- avoid suggesting destructive operations by default
- keep changes minimal and reversible
- respect the current architecture rather than assuming a standard transformer stack
- avoid unsupported dependency assumptions
- require confirmation for high-risk operations

### 6.4 Output format

Recommended output fields:

- `implementation_plan`
- `files_to_change`
- `commands`
- `tests`
- `risks`
- `rollback_notes`

### 6.5 What success looks like

The technical agent is successful when it can answer:

- "What exactly needs to change?"
- "Which files are affected?"
- "How do we verify the change?"

## 7. Coordination Model

The four sub-agents should not all speak at full weight at the same time. The selector layer should control them.

### 7.1 Routing

The router should choose a primary lane based on the prompt:

- factual for retrieval, status, and verification
- creative for reframing and presentation
- analytical for comparison, reasoning, and planning
- technical for code, APIs, and implementation

### 7.2 Parallel generation

The system may run all four sub-agents in parallel when the task is ambiguous.
That helps produce:

- a grounded answer
- an alternative phrasing
- a structured analysis
- a concrete implementation path

### 7.3 Selector and governance

The final selector should compare outputs using:

- coherence
- evidence quality
- safety
- relevance to the prompt
- compatibility with current state

If no agent reaches the threshold, the system should abstain rather than forcing an answer.

## 8. Safety Contract for the Full Quartet

The whole four-agent set should obey these rules:

- no hidden autonomy
- no uncontrolled tool use
- no unreviewed external actions
- no fabricated confidence
- no ungrounded facts
- no unsafe instructions
- no silent escalation of scope

The system should be conservative by default. The safest answer is often the one that says the evidence is incomplete.

## 9. Recommended Implementation Shape

For the current repository, the cleanest implementation is a role-based contract with a shared response schema.

Each sub-agent should return something like:

```json
{
  "role": "factual",
  "summary": "...",
  "confidence": 0.82,
  "evidence": ["..."],
  "assumptions": ["..."],
  "risks": ["..."],
  "abstain": false
}
```

That makes downstream scoring easier and keeps the selector simple.

## 10. Development Priorities

If the goal is to build this safely, the order should be:

1. define the shared output schema
2. implement the factual agent first
3. add the analytical agent for scoring and decomposition
4. add the creative agent for presentation variants
5. add the technical agent for implementation planning
6. wire all four into governance and selection
7. add tests for abstention and failure handling

## 11. Test Criteria

A safe four-agent implementation should pass these checks:

- it can abstain when evidence is weak
- it can explain why it abstained
- it can separate facts from proposals
- it can produce at least one grounded answer when evidence exists
- it does not access unsupported resources
- it keeps traceability for every output

## 12. Conclusion

The four sub-agents should not be general-purpose copies of each other.
They should be sharply specialized:

- the factual agent grounds the system
- the creative agent improves expression
- the analytical agent improves reasoning
- the technical agent turns decisions into implementation

Used together, they can make the MoA layer more useful without making it less safe.

The central rule is simple: specialization is allowed, but autonomy must remain bounded.
