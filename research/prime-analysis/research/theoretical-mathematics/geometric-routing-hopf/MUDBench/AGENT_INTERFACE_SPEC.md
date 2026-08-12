
# MUDBench Agent Interface Specification

## Document Status
Version: 0.1
Status: Draft
Purpose: Define the protocol and interaction model between external AI agents and the MUDBench simulation system.

---

# 1. Design Philosophy

The agent interface must be:

- simple
- deterministic
- constrained
- language-model friendly
- easy to implement in any programming language

The interface should avoid unnecessary complexity so that:

- researchers can prototype quickly
- hobbyists can participate easily
- agents written in different frameworks can interoperate

The interface must also ensure that agents cannot bypass benchmark constraints.

---

# 2. Interaction Model

Agents interact with the system through a **turn-based observation → action loop**.

The sequence for each step is:

1. Environment sends observation
2. Agent decides next action
3. Agent returns action
4. Environment validates action
5. Simulation advances
6. Next observation is produced

This loop continues until the run terminates.

---

# 3. Agent Communication Modes

The interface must support two possible execution modes.

## 3.1 Local Process Mode

The agent runs locally as a process started by MUDBench.

Advantages:

- simple debugging
- deterministic execution
- no network latency

Example:

```
mudbench run --agent ./my_agent.py
```

The simulation communicates through STDIN / STDOUT.

---

## 3.2 HTTP Endpoint Mode

The agent runs as an external service.

MUDBench sends observations via HTTP requests.

Example flow:

```
POST /observe
POST /action
```

Advantages:

- remote agents
- hosted competitions
- language/framework independence

---

# 4. Observation Structure

Each step begins with an observation object sent to the agent.

Example observation:

```
{
  "run_id": "abc123",
  "step": 42,
  "location": "forest_path",
  "description": "You stand on a narrow forest trail.",
  "exits": ["north","south"],
  "entities": [
    {"type":"npc","name":"goblin"},
    {"type":"item","name":"rusty_sword"}
  ],
  "inventory": ["coin"],
  "health": 87,
  "messages": [
    "The goblin snarls."
  ],
  "action_space": [
    "move north",
    "move south",
    "take rusty_sword",
    "attack goblin"
  ],
  "remaining_steps": 958
}
```

Fields must remain stable across versions whenever possible.

---

# 5. Action Submission

The agent returns a single action.

Example:

```
{
  "action": "move north"
}
```

Only actions listed in the observation's `action_space` are guaranteed valid.

The environment may reject actions outside this list.

---

# 6. Action Validation

When an action is received the system performs:

1. syntax validation
2. action whitelist check
3. world constraint check
4. time constraint check

If the action fails validation the environment may:

- reject the action
- apply a no-op
- penalize the agent

Exact policy may vary by benchmark mode.

---

# 7. Timing Constraints

Agents must respond within a specified time limit.

Example limits:

- 1–10 seconds per step for benchmark runs
- longer limits for arena mode

If the agent fails to respond:

- the system records a timeout
- a default action may be applied
- repeated timeouts may terminate the run

---

# 8. Token and Cost Metrics

When possible the system may record:

- prompt token count
- completion token count
- latency
- cost estimates

These metrics are not required for all agents but are useful for evaluation.

---

# 9. Action Budget

Benchmark runs enforce a **maximum number of steps**.

Example:

```
max_steps = 1000
```

The run ends when:

- the step limit is reached
- the agent dies
- objectives are completed
- the scenario termination condition occurs

---

# 10. Error Handling

Agents may occasionally produce malformed responses.

Examples:

- invalid JSON
- missing action field
- unsupported command

The system should:

1. attempt recovery if possible
2. log the error
3. apply a fallback behavior

Repeated failures may terminate the agent.

---

# 11. Security Constraints

Agents must not be able to:

- execute arbitrary system commands
- access internal simulation memory
- modify scoring logic
- interact with other agents outside the defined protocol

All interactions must pass through the defined interface.

---

# 12. Multi-Agent Scheduling

Multiple agents may exist in the same world.

Scheduling approaches may include:

- sequential turns
- simultaneous action resolution
- event queue scheduling

The scheduling mechanism must remain deterministic in benchmark mode.

---

# 13. Run Lifecycle

A run follows this lifecycle:

1. run initialized
2. agents registered
3. world seeded
4. observation loop begins
5. simulation steps executed
6. termination condition triggered
7. final metrics generated
8. replay log finalized

---

# 14. Versioning

The interface must include a version identifier.

Example:

```
"protocol_version": "1.0"
```

Versioning allows future changes without breaking existing agents.

---

# 15. Minimal Agent Example

A minimal agent loop may look like:

```
while True:
    observation = read_input()
    action = decide(observation)
    send_action(action)
```

The agent logic may include:

- planning
- memory
- reasoning
- tool usage

MUDBench does not constrain internal architecture.

---

# 16. Compatibility Goals

The interface should be easy to implement using:

- Python
- NodeJS
- Rust
- Go
- Java
- C++

It should also be compatible with:

- LLM agents
- rule-based bots
- reinforcement learning agents

---

# 17. Future Interface Extensions

Possible future features:

- streaming observations
- tool invocation hooks
- structured sub-actions
- negotiation channels between agents
- event subscriptions

These should not break the core protocol.

---

# 18. Closing Statement

The agent interface is the most important boundary in MUDBench.

If the interface is simple and stable, the ecosystem of agents will grow naturally.

If the interface becomes complex or unstable, participation will collapse.
