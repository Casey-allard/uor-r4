from __future__ import annotations

import pytest

from evaluation.benchmark_runner.seed_policy import (
    MAX_SEED,
    canonicalize_seed_input,
    derive_seed,
    derive_seed_from_canonical,
    normalize_optional_seed,
    normalize_seed,
)


def test_canonicalize_seed_input_is_stable_for_equivalent_payloads() -> None:
    first_payload = {
        "run_id": "run-1",
        "benchmark_id": "benchmark-1",
        "actor_ids": ["agent-b", "agent-a"],
        "scenario": {
            "scenario_id": "scenario-1",
            "vars": {"difficulty": "normal", "region": "cave"},
            "objectives": [{"id": "obj-a", "count": 1}],
        },
    }
    second_payload = {
        "benchmark_id": "benchmark-1",
        "scenario": {
            "objectives": [{"count": 1, "id": "obj-a"}],
            "vars": {"region": "cave", "difficulty": "normal"},
            "scenario_id": "scenario-1",
        },
        "actor_ids": ["agent-b", "agent-a"],
        "run_id": "run-1",
    }

    assert canonicalize_seed_input(first_payload) == canonicalize_seed_input(second_payload)


def test_derive_seed_is_deterministic_and_changes_with_input() -> None:
    payload = {"run_id": "run-1", "benchmark_id": "benchmark-1", "scenario_id": "scenario-1"}
    seed_first = derive_seed(payload)
    seed_second = derive_seed(payload)
    seed_other = derive_seed({"run_id": "run-2", "benchmark_id": "benchmark-1", "scenario_id": "scenario-1"})

    assert seed_first == seed_second
    assert seed_first != seed_other
    assert 0 <= seed_first <= MAX_SEED


def test_derive_seed_from_canonical_matches_mapping_derivation() -> None:
    payload = {"a": 1, "b": {"x": 2, "y": [3, 4]}}
    canonical = canonicalize_seed_input(payload)

    assert derive_seed(payload) == derive_seed_from_canonical(canonical)


def test_seed_normalization_rejects_invalid_values() -> None:
    with pytest.raises(ValueError, match="seed must be an integer"):
        normalize_seed(True, field_name="seed")  # type: ignore[arg-type]
    with pytest.raises(ValueError, match=r"seed must be within \[0, 2147483647\]"):
        normalize_seed(-1, field_name="seed")
    with pytest.raises(ValueError, match="max_seed must be a positive integer"):
        derive_seed({"a": 1}, max_seed=0)


def test_canonicalization_rejects_unsupported_payload_shapes() -> None:
    with pytest.raises(ValueError, match="payload must be a mapping"):
        canonicalize_seed_input("bad")  # type: ignore[arg-type]
    with pytest.raises(ValueError, match="contains unsupported value type"):
        canonicalize_seed_input({"obj": object()})


def test_normalize_optional_seed_returns_none_and_valid_seed() -> None:
    assert normalize_optional_seed(None, field_name="seed_override") is None
    assert normalize_optional_seed(7, field_name="seed_override") == 7
