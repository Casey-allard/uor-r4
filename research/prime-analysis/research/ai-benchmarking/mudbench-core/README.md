# MUDBench

MUDBench includes a deterministic CLI for running single scenarios and for producing a compact baseline summary across the starter tiny-suite scenarios.

## CLI Examples

Run a single built-in scenario and emit structured JSON:

```bash
python -m src.cli.main run --scenario tiny-fetch-quest --output json
```

Run a single built-in scenario with an external local-process wrapper:

```bash
python -m src.cli.main run \
  --scenario tiny-fetch-quest \
  --actor-id agent-a \
  --agent-command "python examples/agents/deterministic_rule_agent.py" \
  --output json
```

Use the default mode when your wrapper is simplest as a one-request process:

1. MUDBench starts the configured local command
2. MUDBench writes one observation JSON object to `stdin`
3. The wrapper writes one action JSON object like `{"action":"move east"}` to `stdout`
4. The wrapper exits

Use persistent mode when the same local wrapper should stay alive across turns in a run:

```bash
python -m src.cli.main run \
  --scenario tiny-fetch-quest \
  --actor-id agent-a \
  --agent-command "python examples/agents/mock_llm_wrapper.py" \
  --persistent-agent-session \
  --output json
```

In persistent mode, MUDBench keeps the configured local command running, sends one observation JSON object per turn on `stdin`, and expects one action JSON object per turn on `stdout`.

Additional example wrappers are available in:

- `examples/agents/deterministic_rule_agent.py`
- `examples/agents/mock_llm_wrapper.py`
- `examples/agents/strict_wrapper_example.py`

Run the five canonical tiny scenarios and emit a compact machine-readable suite summary:

```bash
python -m src.cli.main suite --suite tiny --output json
```

Compare the two built-in tiny-suite actor profiles and emit a compact machine-readable comparison:

```bash
python -m src.cli.main suite --suite tiny --baseline-agent agent-a --candidate-agent agent-b --output json
```

The tiny-suite report includes one entry per configured agent per scenario with:

- `scenario_id`
- `agent_id`
- scenario `aggregate_score`
- actor `composite_score`
- existing scorecard `normalized_metrics`
- existing scorecard `contributions`
- `replay_ref`
- `parity_ref`

The comparison report includes one per-scenario comparison entry with:

- `baseline`
- `candidate`
- `composite_score_difference` (`candidate - baseline`)

It also includes an aggregate `summary` with total and average composite-score differences across the five scenarios.

## Additional Docs

- `PROJECT_CHARTER.md`
- `PRODUCT_SPEC.md`
- `DEVELOPER_GUIDE.md`
