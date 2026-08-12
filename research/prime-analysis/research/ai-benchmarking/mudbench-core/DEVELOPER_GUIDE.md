# MUDBench Developer Guide

## Document Status
Version: 0.1
Status: Draft
Purpose: Provide practical guidance for developers building agents, extending the simulator, or contributing to MUDBench.

---

# 1. Overview

MUDBench is a benchmark environment designed to evaluate AI agents in a structured text-world simulation.

The system combines:

- a deterministic simulation engine
- a structured world environment
- an agent interaction protocol
- a capability-based scoring system
- replay and telemetry infrastructure

---

# 2. Repository Structure

Typical repository layout:

/mudbench
  core
  world
  agents
  evaluation
  replay
  cli
  scenarios
  docs

---

# 3. Running the Simulator

Use the repository root when invoking the built-in CLI entrypoint:

```bash
python -m src.cli.main run --scenario tiny-fetch-quest --output json
```

Expected output:

- machine-readable JSON
- single-scenario lifecycle summary
- scorecard aggregate score and metadata
- replay artifact refs and parity hashes

To run a scenario through an external local-process wrapper, supply `--agent-command`:

```bash
python -m src.cli.main run \
  --scenario tiny-fetch-quest \
  --actor-id agent-a \
  --agent-command "python examples/agents/deterministic_rule_agent.py" \
  --output json
```

Use this default single-shot mode when your wrapper is easiest to implement as one request per process.

You can also run the mock LLM-style example wrapper the same way:

```bash
python -m src.cli.main run \
  --scenario tiny-hidden-key \
  --actor-id agent-a \
  --agent-command "python examples/agents/mock_llm_wrapper.py" \
  --output json
```

Use `--persistent-agent-session` when the same external local process should stay alive across multiple turns in one run:

```bash
python -m src.cli.main run \
  --scenario tiny-fetch-quest \
  --actor-id agent-a \
  --agent-command "python examples/agents/mock_llm_wrapper.py" \
  --persistent-agent-session \
  --output json
```

Default single-shot mode works like this:

1. MUDBench starts the configured local command
2. MUDBench writes one observation JSON object to the process `stdin`
3. The wrapper writes one action JSON object to `stdout`
4. The process exits

Persistent mode uses the same stdin/stdout contract, but keeps the process alive across turns:

1. MUDBench starts the configured local command once for the run
2. MUDBench writes one observation JSON object per turn to `stdin`
3. The wrapper writes one action JSON object per turn to `stdout`
4. MUDBench closes the process when the run ends or the session breaks

In both modes, the wrapper must speak newline-delimited JSON and return a supported action object like `{"action":"move east"}`. Persistent mode does not add any new protocol fields.

Shipped wrapper examples:

- `examples/agents/deterministic_rule_agent.py`
- `examples/agents/mock_llm_wrapper.py`
- `examples/agents/strict_wrapper_example.py`

To generate a compact benchmark summary across the five canonical tiny scenarios:

```bash
python -m src.cli.main suite --suite tiny --output json
```

Expected output:

- machine-readable JSON
- `report.entries` with one entry per configured agent per scenario
- `scenario_id`, `agent_id`, `aggregate_score`, and `composite_score`
- existing scorecard `normalized_metrics` and `contributions`
- `replay_ref`
- `parity_ref` with terminal/applied/score summary hashes

To compare the two built-in actor profiles across the same tiny suite:

```bash
python -m src.cli.main suite --suite tiny --baseline-agent agent-a --candidate-agent agent-b --output json
```

Expected output:

- machine-readable JSON
- `report.comparisons` with one entry per scenario
- `baseline` and `candidate` entries containing the existing suite report fields
- `composite_score_difference` for each scenario (`candidate - baseline`)
- `report.summary` with total and average composite-score differences across the suite

---

# 4. Writing an Agent

A local agent wrapper for the current CLI hook always uses the same basic protocol:

1. MUDBench writes one observation JSON object to `stdin`
2. the wrapper chooses one valid action from `action_space`
3. the wrapper writes one action JSON object to `stdout`

Choose the process model explicitly:

- Default `--agent-command` mode: MUDBench starts the process for one request, reads one response, and the process exits.
- `--agent-command ... --persistent-agent-session`: MUDBench keeps the same process alive and repeats the same observation-in/action-out exchange across turns.

Persistent-session mode is the current way to keep a local wrapper alive across steps. No broader long-lived session protocol is supported beyond repeated newline-delimited JSON exchanges on stdin/stdout.

---

# 5. Minimal Python Agent

```python
import json
import sys

line = sys.stdin.readline()
observation = json.loads(line)
action_space = observation.get("action_space", [])
action = action_space[0] if action_space else "wait"
print(json.dumps({"action": action}))
sys.stdout.flush()
```

This example shows the minimal supported single-shot shape:

- read one observation object from `stdin`
- emit one JSON action object like `{"action": "move east"}`
- exit after responding

For persistent mode, keep the same observation/action shape but read and respond in a loop:

```python
import json
import sys

for line in sys.stdin:
    observation = json.loads(line)
    action_space = observation.get("action_space", [])
    action = action_space[0] if action_space else "wait"
    print(json.dumps({"action": action}))
    sys.stdout.flush()
```

This is still the same current local CLI contract: newline-delimited observation JSON in, newline-delimited action JSON out. Persistent-session mode only changes process lifetime, not the payload schema.

For complete runnable examples, see:

- `examples/agents/deterministic_rule_agent.py`
- `examples/agents/mock_llm_wrapper.py`
- `examples/agents/strict_wrapper_example.py`

---

# 6. Simulation Loop

1. observation
2. action
3. validation
4. world update
5. scoring
6. replay logging

---

# 7. Benchmark Runs

max_steps = 1000  
time_limit = 5s  
seed = fixed  

---

# 8. Replay Logs

/replay/run_x.json

---

# 9. Scoring Output

navigation, memory, planning, tactical, social, efficiency

---

# 10. Scenarios

/scenarios/

---

# 11. Rules

- deterministic
- observable
- replay-safe

---

# 12. Debugging

- replay inspection
- step tracing

---

# 13. Performance

- fast loop
- low overhead

---

# 14. Contributions

- modular
- tested
- documented

---

# 15. Local Validation Commands

Run local validation hooks from the repository root:

```bash
make lint-local
make test-local
make determinism-local
make ci-local
```

Notes:

- `lint-local` uses `ruff` when available, falls back to `flake8`, and fails explicitly if neither is installed.
- `determinism-local` runs the real benchmark-runtime determinism gate suite (`tests/benchmark/test_real_determinism_gate.py`) with fail-fast behavior.
- `ci-local` chains lint, tests, and determinism checks in fail-fast order.

---

# 16. Closing

MUDBench evaluates real agent capability, not tricks.
