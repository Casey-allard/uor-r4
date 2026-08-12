# Local Agent Wrapper Examples

These examples show how an external local-process agent plugs into MUDBench through the current `run --agent-command` hook.
MUDBench also now supports one bounded direct-provider benchmark path through `run --direct-provider openai-chat-completions`, which reuses the same canonical prompt, parse, repair, and fail-closed helpers internally.

## Current contract

The current local-process bridge is single-shot per request:

1. MUDBench starts the configured local command
2. MUDBench writes one observation JSON object to the process `stdin`
3. The process writes one action JSON object to `stdout`
4. The process exits

The wrapper must not invent extra protocol fields. The action payload must be:

```json
{"action":"move east"}
```

The observation payload includes the current protocol fields used by MUDBench, including:

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

## Example wrappers

- `strict_wrapper_example.py`
  - validates the expected observation shape
  - returns a deterministic valid action
- `deterministic_rule_agent.py`
  - simple rule-based policy for local testing
  - prefers `take` then `move` then `attack` then `look` then `wait`
- `mock_llm_wrapper.py`
  - exercises the canonical benchmark-mode LLM runtime path
  - builds the canonical model-facing payload, session frame, and guarded benchmark prompt
  - uses strict JSON-only action parsing plus one bounded repair pass
  - fails closed to a deterministic fallback action if both model attempts are invalid
  - uses a deterministic in-process mock instead of a real API

## Example commands

Run a tiny canonical scenario with the deterministic rule wrapper:

```bash
python -m src.cli.main run \
  --scenario tiny-fetch-quest \
  --actor-id agent-a \
  --agent-command "python examples/agents/deterministic_rule_agent.py"
```

Run the mock LLM-style wrapper:

```bash
python -m src.cli.main run \
  --scenario tiny-hidden-key \
  --actor-id agent-a \
  --agent-command "python examples/agents/mock_llm_wrapper.py"
```

Run the bounded direct-provider path against the same benchmark loop:

```bash
export MUDBENCH_OPENAI_API_KEY="..."
export MUDBENCH_OPENAI_MODEL="gpt-4.1-mini"
python -m src.cli.main run \
  --scenario tiny-hidden-key \
  --actor-id agent-a \
  --direct-provider openai-chat-completions
```

Optional:

- `MUDBENCH_OPENAI_BASE_URL` overrides the default `https://api.openai.com/v1/chat/completions`
- `--direct-provider-model` overrides `MUDBENCH_OPENAI_MODEL` for one run

Run the same wrapper while forcing one deterministic malformed first response so the bounded repair path is exercised:

```bash
python -m src.cli.main run \
  --scenario tiny-hidden-key \
  --actor-id agent-a \
  --agent-command "python examples/agents/mock_llm_wrapper.py --emit-invalid-first"
```

Run the same wrapper while forcing malformed output on both attempts so the fail-closed fallback action is exercised:

```bash
python -m src.cli.main run \
  --scenario tiny-hidden-key \
  --actor-id agent-a \
  --agent-command "python examples/agents/mock_llm_wrapper.py --emit-invalid-always"
```

For quick manual play against the same deterministic scenario surface, launch the text-only console client:

```bash
PYTHONPATH=src python -m cli.main play --scenario tiny-guarded-relic
```

The console client renders the current observation, the exact accepted actions, and derived target hints, then accepts either a literal action string or a 1-based action number.

For a compact comparison pass across the richer playable slices with built-in and mock-wrapper modes:

```bash
PYTHONPATH=src python -m cli.main compare-playable-slices
```

To include the bounded direct-provider path when it is configured:

```bash
MUDBENCH_OPENAI_API_KEY="..." MUDBENCH_OPENAI_MODEL="gpt-4.1-mini" \
PYTHONPATH=src python -m cli.main compare-playable-slices \
  --direct-provider openai-chat-completions
```

Use the strict example when you want a small reference implementation of the observation-in / action-out contract:

```bash
python examples/agents/strict_wrapper_example.py
```
