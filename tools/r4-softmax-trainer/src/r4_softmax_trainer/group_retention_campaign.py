"""Construction-only campaign harness for issue #973.

This module stops at the cheap authorization gate.  It can prepare the frozen
population, validate the Rust-owned group artifact, and exercise disposable
models, but it cannot open held-out bytes or start the one-shot main fit.
"""

from __future__ import annotations

import copy
import gc
import json
import math
import os
import statistics
import struct
import time
from collections import Counter
from collections.abc import Callable, Mapping, Sequence
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any, Protocol

import torch
from torch import Tensor

from .group_retention import (
    GEOMETRY_ARMS,
    POLICY,
    PRODUCTION_CONTEXT,
    PRODUCTION_GROUP_SIZE,
    PRODUCTION_INITIALIZATION_SEED,
    PRODUCTION_MAX_CANDIDATE_LEAVES,
    PRODUCTION_VOCAB_SIZE,
    GroupAddressArtifact,
    GroupRetentionConfig,
    R4GroupAddressedRetentionLMV1,
)
from .group_retention_data import (
    FIT_STORY_COUNT,
    FIT_TOKENS_RELATIVE_PATH,
    HELDOUT_STORY_COUNT,
    TOKENS_PER_STORY,
    TRAINING_VIEW_MANIFEST_NAME,
    VOCAB_SIZE,
    load_group_retention_training_view,
    prepare_group_retention_population,
)
from .provenance import (
    artifact_records,
    canonical_json_bytes,
    cid_bytes,
    trainer_implementation_contract,
    tree_cid,
    verify_bound_manifest,
)
from .train import require_mps


ISSUE = 973
TERMINAL_UNAVAILABLE = "UNAVAILABLE_FRAME_POPULATION_OR_LOCAL_BUDGET"
MAIN_NOT_RUN = "NOT_RUN"
MAIN_AUTHORIZED_NOT_RUN = "AUTHORIZED_NOT_RUN"

GEOMETRY_SCHEMA = 1
GEOMETRY_DOMAIN = "uor-r4.group-addressed-retention-geometry/1"
LEAF_SCHEMA = 1
LEAF_DOMAIN = "uor-r4.group-addressed-retention-prime-leaf-map/1"
LEAF_POLICY = (
    "BOS token 0 maps to the exact H4 identity; token t>0 maps to "
    "zero-based prime p_(t-1) mod 120"
)
SCRAMBLE_SCHEMA = 1
SCRAMBLE_DOMAIN = "uor-r4.group-addressed-retention-transport-scramble/1"
SCRAMBLE_POLICY = (
    "identity-fixing deterministic rotation within each exact H4 element-order "
    "class; candidate leaves remain true and only transport actions use pi(leaf)"
)

GEOMETRY_RELATIVE_PATH = "geometry/r4-group-address-geometry.json"
PREPARATION_MANIFEST_NAME = "group-retention-preparation-manifest.json"
STARTED_RELATIVE_PATH = "preflight/group-retention-preflight-started.json"
RESULT_RELATIVE_PATH = "preflight/group-retention-preflight-result.json"
AUTHORIZATION_RELATIVE_PATH = "preflight/group-retention-main-authorization.json"

PREPARATION_SCHEMA = "uor-r4.group-addressed-retention-preparation/1"
STARTED_SCHEMA = "uor-r4.group-addressed-retention-preflight-started/1"
RESULT_SCHEMA = "uor-r4.group-addressed-retention-preflight-result/1"
AUTHORIZATION_SCHEMA = "uor-r4.group-addressed-retention-main-authorization/1"
STRUCTURAL_SCHEMA = "uor-r4.group-addressed-retention-structural-opportunity/1"

MAIN_OPTIMIZER_STEPS_PER_ARM = 256
MAIN_TOTAL_OPTIMIZER_STEPS = 768
MAIN_PRESENTATIONS_PER_ARM = 524_288
MAIN_TOTAL_PRESENTATIONS = 1_572_864
MAIN_WALL_CEILING_SECONDS = 900.0
ETA_CEILING_SECONDS = 720.0
ETA_SAFETY_FACTOR = 1.25
R_ACTION_MINIMUM = 41
PREFLIGHT_EXECUTION_PATH = "exact_stationary_frame_closed_form"
PREFLIGHT_USE_CHECKPOINT = False
REFERENCE_CHECKPOINT_CHUNK = 16
PREFLIGHT_OBSERVED_CURRENT_MEMORY_BYTES = 1_597_398_528
PREFLIGHT_OBSERVED_DRIVER_MEMORY_BYTES = 3_521_118_208
PREFLIGHT_OBSERVED_RECOMMENDED_MEMORY_BYTES = 12_713_115_648


class GroupRetentionPreflightUnavailable(RuntimeError):
    """The cheap construction gate cannot authorize the main campaign."""

    terminal = TERMINAL_UNAVAILABLE


def _integer(value: object, *, path: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int):
        raise ValueError(f"{path} must be an integer")
    return value


def _mapping(value: object, *, path: str) -> Mapping[str, Any]:
    if not isinstance(value, Mapping):
        raise ValueError(f"{path} must be an object")
    return value


def _ordered_keys(value: Mapping[str, Any], expected: Sequence[str], *, path: str) -> None:
    if list(value) != list(expected):
        raise ValueError(f"{path} fields or canonical field order differ from the Rust contract")


def _integer_vector(
    value: object,
    *,
    length: int,
    upper_bound: int,
    path: str,
) -> tuple[int, ...]:
    if not isinstance(value, list) or len(value) != length:
        raise ValueError(f"{path} must contain exactly {length} entries")
    result = tuple(_integer(item, path=f"{path}[{offset}]") for offset, item in enumerate(value))
    if any(item < 0 or item >= upper_bound for item in result):
        raise ValueError(f"{path} contains an out-of-range group index")
    return result


def _permutation_rows(value: object, *, order: int, path: str) -> tuple[tuple[int, ...], ...]:
    if not isinstance(value, list) or len(value) != order:
        raise ValueError(f"{path} must contain exactly {order} action rows")
    rows = tuple(
        _integer_vector(row, length=order, upper_bound=order, path=f"{path}[{offset}]")
        for offset, row in enumerate(value)
    )
    expected = tuple(range(order))
    if any(tuple(sorted(row)) != expected for row in rows):
        raise ValueError(f"{path} contains a non-permutation action")
    return rows


def _reject_json_constant(value: str) -> None:
    raise ValueError(f"geometry JSON contains non-finite constant {value}")


def _object_without_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"geometry JSON repeats field {key!r}")
        result[key] = value
    return result


def _rust_json_bytes(value: object) -> bytes:
    """Mirror serde_json::to_vec for this ASCII, integer-only Rust schema."""
    return json.dumps(
        value,
        ensure_ascii=False,
        allow_nan=False,
        separators=(",", ":"),
    ).encode("utf-8")


def _first_primes(count: int) -> tuple[int, ...]:
    if count < 0:
        raise ValueError("prime count cannot be negative")
    primes: list[int] = []
    candidate = 2
    while len(primes) < count:
        limit = math.isqrt(candidate)
        if all(candidate % prime for prime in primes if prime <= limit):
            primes.append(candidate)
        candidate += 1 if candidate == 2 else 2
    return tuple(primes)


def _multiply(table: Sequence[int], order: int, left: int, right: int) -> int:
    return table[left * order + right]


def _generated_subgroup(
    table: Sequence[int], *, order: int, identity: int, generators: Sequence[int]
) -> tuple[int, ...]:
    reached = {identity}
    frontier = [identity]
    while frontier:
        current = frontier.pop()
        for generator in generators:
            product = _multiply(table, order, current, generator)
            if product not in reached:
                reached.add(product)
                frontier.append(product)
    return tuple(sorted(reached))


def _element_orders(table: Sequence[int], *, order: int, identity: int) -> tuple[int, ...]:
    result: list[int] = []
    for element in range(order):
        value = identity
        for exponent in range(1, order + 1):
            value = _multiply(table, order, value, element)
            if value == identity:
                result.append(exponent)
                break
        else:
            raise ValueError(f"group element {element} has no finite order <= {order}")
    return tuple(result)


def _validate_group(
    table: Sequence[int], inverses: Sequence[int], *, order: int, identity: int, path: str
) -> None:
    expected = tuple(range(order))
    if tuple(table[identity * order : (identity + 1) * order]) != expected:
        raise ValueError(f"{path} has no left identity at the declared offset")
    if tuple(table[offset * order + identity] for offset in range(order)) != expected:
        raise ValueError(f"{path} has no right identity at the declared offset")
    for element, inverse in enumerate(inverses):
        if (
            _multiply(table, order, element, inverse) != identity
            or _multiply(table, order, inverse, element) != identity
        ):
            raise ValueError(f"{path} inverse table fails at element {element}")
    for left in range(order):
        left_row = table[left * order : (left + 1) * order]
        if tuple(sorted(left_row)) != expected:
            raise ValueError(f"{path} left row {left} is not a permutation")
        for right in range(order):
            product_row = table[left_row[right] * order : (left_row[right] + 1) * order]
            right_row = table[right * order : (right + 1) * order]
            for third in range(order):
                if product_row[third] != left_row[right_row[third]]:
                    raise ValueError(f"{path} is not associative")


def _histogram_records(
    support: Sequence[int], element_orders: Sequence[int]
) -> list[dict[str, int]]:
    counts = Counter(element_orders[element] for element in set(support))
    return [
        {"element_order": element_order, "distinct_actions": counts[element_order]}
        for element_order in sorted(counts)
    ]


@dataclass(frozen=True, slots=True)
class GroupGeometryBundle:
    """Validated Rust geometry plus the three equal-shape experiment arms."""

    exact_h4: GroupAddressArtifact
    cyclic_120: GroupAddressArtifact
    scrambled_h4: GroupAddressArtifact
    artifact_cid: str
    geometry_file_cid: str
    direct_support: tuple[int, ...]
    h4_generated_count: int
    c120_generated_count: int
    scrambled_generated_count: int

    @property
    def arms(self) -> dict[str, GroupAddressArtifact]:
        return {
            "exact_h4": self.exact_h4,
            "cyclic_120": self.cyclic_120,
            "scrambled_h4": self.scrambled_h4,
        }

    def population_signatures(
        self,
        *,
        fit_stories: tuple[tuple[int, ...], ...],
        heldout_stories: tuple[tuple[int, ...], ...],
    ) -> Mapping[str, Any]:
        if len(fit_stories) != FIT_STORY_COUNT or len(heldout_stories) != HELDOUT_STORY_COUNT:
            raise ValueError("#973 structural census requires exactly 256 fit and 64 held-out stories")
        fit = _partition_signature_census(
            fit_stories,
            arms=self.arms,
            direct_support=self.direct_support,
            expected_story_tokens=TOKENS_PER_STORY,
        )
        heldout = _partition_signature_census(
            heldout_stories,
            arms=self.arms,
            direct_support=self.direct_support,
            expected_story_tokens=TOKENS_PER_STORY,
        )
        passed = (
            self.h4_generated_count == PRODUCTION_GROUP_SIZE
            and self.c120_generated_count == PRODUCTION_GROUP_SIZE
            and self.scrambled_generated_count == PRODUCTION_GROUP_SIZE
            and heldout["r_action"] >= R_ACTION_MINIMUM
            and heldout["stories_with_r_action"] == HELDOUT_STORY_COUNT
        )
        return {
            "schema": STRUCTURAL_SCHEMA,
            "policy": (
                "at each actual-next row, transport only prior write identities by the "
                "current observed token action; compare their readable true-candidate "
                "relative slots without opening or using the next token"
            ),
            "geometry_artifact_cid": self.artifact_cid,
            "geometry_file_cid": self.geometry_file_cid,
            "group_order": PRODUCTION_GROUP_SIZE,
            "direct_candidate_support_count": len(self.direct_support),
            "generated_state_coverage": {
                "exact_h4": self.h4_generated_count,
                "cyclic_120": self.c120_generated_count,
                "scrambled_h4": self.scrambled_generated_count,
            },
            "fit": fit,
            "heldout": heldout,
            "next_token_reads": 0,
            "passed": passed,
        }


def load_group_geometry_artifacts(path: Path) -> GroupGeometryBundle:
    """Load the canonical Rust export and construct all three Python arms."""
    path = path.resolve()
    if path.is_symlink() or not path.is_file():
        raise ValueError("#973 geometry must be an existing regular, non-symlink file")
    raw = path.read_bytes()
    try:
        value = json.loads(
            raw.decode("utf-8", errors="strict"),
            object_pairs_hook=_object_without_duplicate_keys,
            parse_constant=_reject_json_constant,
        )
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError("#973 geometry is not strict UTF-8 JSON") from error
    root = _mapping(value, path="geometry")
    top_keys = (
        "schema",
        "domain",
        "max_token_id",
        "group_order",
        "h4_root_table_kappa",
        "h4_multiplication_table_kappa",
        "identity_index",
        "inverse_indices",
        "h4_multiplication_indices",
        "c120_inverse_indices",
        "c120_multiplication_indices",
        "h4_left_regular_permutations",
        "c120_left_regular_permutations",
        "leaf_map",
        "scramble",
        "censuses",
        "artifact_cid",
    )
    _ordered_keys(root, top_keys, path="geometry")
    if _rust_json_bytes(root) != raw:
        raise ValueError("#973 geometry bytes are not canonical Rust JSON")

    if (
        _integer(root["schema"], path="geometry.schema") != GEOMETRY_SCHEMA
        or root["domain"] != GEOMETRY_DOMAIN
        or _integer(root["max_token_id"], path="geometry.max_token_id")
        != PRODUCTION_VOCAB_SIZE - 1
        or _integer(root["group_order"], path="geometry.group_order")
        != PRODUCTION_GROUP_SIZE
    ):
        raise ValueError("#973 geometry schema/domain/bounds differ from the frozen contract")
    for field_name in ("h4_root_table_kappa", "h4_multiplication_table_kappa"):
        if not isinstance(root[field_name], str) or not root[field_name]:
            raise ValueError(f"geometry.{field_name} must be a nonempty source witness")

    order = PRODUCTION_GROUP_SIZE
    identity = _integer(root["identity_index"], path="geometry.identity_index")
    if not 0 <= identity < order:
        raise ValueError("geometry identity index is outside the group")
    inverses = _integer_vector(
        root["inverse_indices"], length=order, upper_bound=order, path="geometry.inverse_indices"
    )
    c120_inverses = _integer_vector(
        root["c120_inverse_indices"],
        length=order,
        upper_bound=order,
        path="geometry.c120_inverse_indices",
    )
    h4_table = _integer_vector(
        root["h4_multiplication_indices"],
        length=order * order,
        upper_bound=order,
        path="geometry.h4_multiplication_indices",
    )
    c120_table = _integer_vector(
        root["c120_multiplication_indices"],
        length=order * order,
        upper_bound=order,
        path="geometry.c120_multiplication_indices",
    )
    h4_actions = _permutation_rows(
        root["h4_left_regular_permutations"], order=order, path="geometry.h4_actions"
    )
    c120_actions = _permutation_rows(
        root["c120_left_regular_permutations"], order=order, path="geometry.c120_actions"
    )
    if tuple(item for row in h4_actions for item in row) != h4_table:
        raise ValueError("H4 action rows disagree with the multiplication table")
    if tuple(item for row in c120_actions for item in row) != c120_table:
        raise ValueError("C120 action rows disagree with the multiplication table")
    expected_c120 = tuple(
        (identity + ((left - identity) % order) + ((right - identity) % order)) % order
        for left in range(order)
        for right in range(order)
    )
    if c120_table != expected_c120:
        raise ValueError("C120 is not the frozen identity-reindexed cyclic law")
    _validate_group(h4_table, inverses, order=order, identity=identity, path="H4 table")
    _validate_group(c120_table, c120_inverses, order=order, identity=identity, path="C120 table")

    leaf = _mapping(root["leaf_map"], path="geometry.leaf_map")
    leaf_keys = (
        "schema",
        "domain",
        "policy",
        "max_token_id",
        "leaf_indices",
        "direct_support_indices",
        "direct_support_count",
        "leaf_cid",
    )
    _ordered_keys(leaf, leaf_keys, path="geometry.leaf_map")
    if (
        _integer(leaf["schema"], path="leaf.schema") != LEAF_SCHEMA
        or leaf["domain"] != LEAF_DOMAIN
        or leaf["policy"] != LEAF_POLICY
        or _integer(leaf["max_token_id"], path="leaf.max_token_id")
        != PRODUCTION_VOCAB_SIZE - 1
    ):
        raise ValueError("#973 leaf schema/domain/policy differs from the frozen rule")
    leaves = _integer_vector(
        leaf["leaf_indices"],
        length=PRODUCTION_VOCAB_SIZE,
        upper_bound=order,
        path="geometry.leaf_map.leaf_indices",
    )
    expected_primes = _first_primes(PRODUCTION_VOCAB_SIZE - 1)
    expected_leaves = (identity,) + tuple(
        expected_primes[token_id - 1] % order
        for token_id in range(1, PRODUCTION_VOCAB_SIZE)
    )
    if leaves != expected_leaves:
        raise ValueError("#973 leaves do not reproduce zero-based p_(token-1) mod 120")
    support = tuple(sorted(set(leaves)))
    support_value = _integer_vector(
        leaf["direct_support_indices"],
        length=len(support),
        upper_bound=order,
        path="geometry.leaf_map.direct_support_indices",
    )
    if support_value != support or _integer(
        leaf["direct_support_count"], path="leaf.direct_support_count"
    ) != len(support):
        raise ValueError("#973 direct leaf support census does not reproduce")
    if len(support) > PRODUCTION_MAX_CANDIDATE_LEAVES:
        raise ValueError("#973 direct candidate support exceeds the frozen bound")
    leaf_seed = copy.deepcopy(dict(leaf))
    leaf_cid = leaf_seed["leaf_cid"]
    leaf_seed["leaf_cid"] = ""
    if leaf_cid != cid_bytes(_rust_json_bytes(leaf_seed)):
        raise ValueError("#973 leaf content identity does not reproduce")

    scramble = _mapping(root["scramble"], path="geometry.scramble")
    scramble_keys = (
        "schema",
        "domain",
        "policy",
        "permutation",
        "transport_leaf_indices",
        "moved_count",
        "element_orders",
        "identity_fixed",
        "element_orders_preserved",
        "used_leaf_order_histogram",
        "scrambled_used_action_order_histogram",
        "nonhomomorphism_witness",
        "used_action_generated_subgroup_count",
    )
    _ordered_keys(scramble, scramble_keys, path="geometry.scramble")
    if (
        _integer(scramble["schema"], path="scramble.schema") != SCRAMBLE_SCHEMA
        or scramble["domain"] != SCRAMBLE_DOMAIN
        or scramble["policy"] != SCRAMBLE_POLICY
    ):
        raise ValueError("#973 scramble schema/domain/policy differs")
    permutation = _integer_vector(
        scramble["permutation"], length=order, upper_bound=order, path="scramble.permutation"
    )
    if tuple(sorted(permutation)) != tuple(range(order)) or permutation[identity] != identity:
        raise ValueError("#973 scramble must be an identity-fixing bijection")
    moved_count = sum(index != target for index, target in enumerate(permutation))
    if moved_count < 100 or _integer(scramble["moved_count"], path="scramble.moved_count") != moved_count:
        raise ValueError("#973 transport scramble is not the frozen broad permutation")
    if scramble["identity_fixed"] is not True or scramble["element_orders_preserved"] is not True:
        raise ValueError("#973 scramble does not declare its required invariants")
    observed_orders = _integer_vector(
        scramble["element_orders"],
        length=order,
        upper_bound=order + 1,
        path="scramble.element_orders",
    )
    expected_orders = _element_orders(h4_table, order=order, identity=identity)
    if observed_orders != expected_orders or any(
        expected_orders[element] != expected_orders[permutation[element]] for element in range(order)
    ):
        raise ValueError("#973 scramble does not preserve exact H4 element orders")
    transport_leaves = _integer_vector(
        scramble["transport_leaf_indices"],
        length=PRODUCTION_VOCAB_SIZE,
        upper_bound=order,
        path="scramble.transport_leaf_indices",
    )
    if transport_leaves != tuple(permutation[leaf_index] for leaf_index in leaves):
        raise ValueError("#973 transport leaves do not equal pi(true leaf)")
    for name, expected_histogram in (
        ("used_leaf_order_histogram", _histogram_records(support, expected_orders)),
        (
            "scrambled_used_action_order_histogram",
            _histogram_records(tuple(sorted(set(transport_leaves))), expected_orders),
        ),
    ):
        histogram = scramble[name]
        if not isinstance(histogram, list):
            raise ValueError(f"scramble.{name} must be a list")
        for offset, record in enumerate(histogram):
            record = _mapping(record, path=f"scramble.{name}[{offset}]")
            _ordered_keys(
                record,
                ("element_order", "distinct_actions"),
                path=f"scramble.{name}[{offset}]",
            )
        if histogram != expected_histogram:
            raise ValueError(f"scramble.{name} does not reproduce")

    witness = _mapping(scramble["nonhomomorphism_witness"], path="scramble.witness")
    witness_keys = (
        "left",
        "right",
        "true_product",
        "permuted_product",
        "product_of_permuted",
    )
    _ordered_keys(witness, witness_keys, path="scramble.witness")
    witness_values = {
        name: _integer(witness[name], path=f"scramble.witness.{name}") for name in witness_keys
    }
    if any(not 0 <= item < order for item in witness_values.values()):
        raise ValueError("#973 nonhomomorphism witness contains an out-of-range index")
    true_product = _multiply(
        h4_table, order, witness_values["left"], witness_values["right"]
    )
    permuted_product = permutation[true_product]
    product_of_permuted = _multiply(
        h4_table,
        order,
        permutation[witness_values["left"]],
        permutation[witness_values["right"]],
    )
    if (
        witness_values["true_product"] != true_product
        or witness_values["permuted_product"] != permuted_product
        or witness_values["product_of_permuted"] != product_of_permuted
        or permuted_product == product_of_permuted
    ):
        raise ValueError("#973 scramble witness does not prove non-homomorphism")

    h4_generated = _generated_subgroup(h4_table, order=order, identity=identity, generators=support)
    c120_generated = _generated_subgroup(
        c120_table, order=order, identity=identity, generators=support
    )
    scrambled_support = tuple(sorted(set(transport_leaves)))
    scrambled_generated = _generated_subgroup(
        h4_table, order=order, identity=identity, generators=scrambled_support
    )
    censuses = _mapping(root["censuses"], path="geometry.censuses")
    census_keys = (
        "direct_leaf_support_indices",
        "direct_leaf_support_count",
        "direct_nonidentity_leaf_support_count",
        "identity_token_count",
        "h4_generated_subgroup_indices",
        "h4_generated_subgroup_count",
        "c120_generated_subgroup_indices",
        "c120_generated_subgroup_count",
        "scrambled_h4_generated_subgroup_indices",
        "scrambled_h4_generated_subgroup_count",
    )
    _ordered_keys(censuses, census_keys, path="geometry.censuses")
    expected_censuses: dict[str, Any] = {
        "direct_leaf_support_indices": list(support),
        "direct_leaf_support_count": len(support),
        "direct_nonidentity_leaf_support_count": len(set(support) - {identity}),
        "identity_token_count": sum(leaf_index == identity for leaf_index in leaves),
        "h4_generated_subgroup_indices": list(h4_generated),
        "h4_generated_subgroup_count": len(h4_generated),
        "c120_generated_subgroup_indices": list(c120_generated),
        "c120_generated_subgroup_count": len(c120_generated),
        "scrambled_h4_generated_subgroup_indices": list(scrambled_generated),
        "scrambled_h4_generated_subgroup_count": len(scrambled_generated),
    }
    if dict(censuses) != expected_censuses:
        raise ValueError("#973 group-coverage censuses do not reproduce")
    if (
        len(h4_generated) != order
        or len(c120_generated) != order
        or len(scrambled_generated) != order
        or _integer(
            scramble["used_action_generated_subgroup_count"],
            path="scramble.used_action_generated_subgroup_count",
        )
        != order
    ):
        raise ValueError("#973 geometry does not generate all 120 actions in every arm")

    artifact_seed = copy.deepcopy(dict(root))
    artifact_cid = artifact_seed["artifact_cid"]
    artifact_seed["artifact_cid"] = ""
    if artifact_cid != cid_bytes(_rust_json_bytes(artifact_seed)):
        raise ValueError("#973 geometry artifact content identity does not reproduce")
    if not isinstance(artifact_cid, str):
        raise ValueError("#973 geometry artifact CID must be a string")

    token_leaves = torch.tensor(leaves, dtype=torch.long)
    h4_tensor = torch.tensor(h4_actions, dtype=torch.long)
    c120_tensor = torch.tensor(c120_actions, dtype=torch.long)
    scrambled_tensor = torch.tensor(
        tuple(h4_actions[permutation[leaf_index]] for leaf_index in range(order)),
        dtype=torch.long,
    )
    artifacts = {
        "exact_h4": GroupAddressArtifact(
            arm="exact_h4",
            identity_offset=identity,
            token_leaves=token_leaves,
            left_actions=h4_tensor,
            artifact_cid=artifact_cid,
        ),
        "cyclic_120": GroupAddressArtifact(
            arm="cyclic_120",
            identity_offset=identity,
            token_leaves=token_leaves.clone(),
            left_actions=c120_tensor,
            artifact_cid=artifact_cid,
        ),
        "scrambled_h4": GroupAddressArtifact(
            arm="scrambled_h4",
            identity_offset=identity,
            token_leaves=token_leaves.clone(),
            left_actions=scrambled_tensor,
            artifact_cid=artifact_cid,
        ),
    }
    for artifact in artifacts.values():
        artifact.validate(
            group_size=PRODUCTION_GROUP_SIZE,
            vocab_size=PRODUCTION_VOCAB_SIZE,
            max_candidate_leaves=PRODUCTION_MAX_CANDIDATE_LEAVES,
            require_cid=True,
        )
    if any(
        not torch.equal(artifact.token_leaves, artifacts["exact_h4"].token_leaves)
        for artifact in artifacts.values()
    ):
        raise ValueError("#973 matched controls do not retain byte-identical raw candidate leaves")
    return GroupGeometryBundle(
        exact_h4=artifacts["exact_h4"],
        cyclic_120=artifacts["cyclic_120"],
        scrambled_h4=artifacts["scrambled_h4"],
        artifact_cid=artifact_cid,
        geometry_file_cid=cid_bytes(raw),
        direct_support=support,
        h4_generated_count=len(h4_generated),
        c120_generated_count=len(c120_generated),
        scrambled_generated_count=len(scrambled_generated),
    )


def _inverse_action_rows(artifact: GroupAddressArtifact) -> tuple[tuple[int, ...], ...]:
    rows = artifact.left_actions.tolist()
    inverse_rows: list[tuple[int, ...]] = []
    for row in rows:
        inverse = [0] * len(row)
        for new_slot, old_slot in enumerate(row):
            inverse[old_slot] = new_slot
        inverse_rows.append(tuple(inverse))
    return tuple(inverse_rows)


def _partition_signature_census(
    stories: Sequence[Sequence[int]],
    *,
    arms: Mapping[str, GroupAddressArtifact],
    direct_support: Sequence[int],
    expected_story_tokens: int | None,
) -> dict[str, Any]:
    if tuple(arms) != GEOMETRY_ARMS:
        raise ValueError(f"structural census arms must be ordered exactly as {GEOMETRY_ARMS}")
    support = frozenset(direct_support)
    inverse_rows = {name: _inverse_action_rows(artifact) for name, artifact in arms.items()}
    leaves = {name: artifact.token_leaves.tolist() for name, artifact in arms.items()}
    identity = {name: artifact.identity_offset for name, artifact in arms.items()}
    group_sizes = {artifact.left_actions.shape[0] for artifact in arms.values()}
    vocab_sizes = {artifact.token_leaves.shape[0] for artifact in arms.values()}
    if len(group_sizes) != 1 or len(vocab_sizes) != 1:
        raise ValueError("structural census arms do not share group/vocabulary sizes")
    if any(leaves[name] != leaves["exact_h4"] for name in arms):
        raise ValueError("structural census controls changed the raw candidate-leaf map")
    vocab_size = int(next(iter(vocab_sizes)))

    pairwise = {"exact_vs_cyclic": 0, "exact_vs_scrambled": 0}
    story_rows: list[int] = []
    total_rows = 0
    for story in stories:
        if expected_story_tokens is not None and len(story) != expected_story_tokens:
            raise ValueError(f"structural story must contain exactly {expected_story_tokens} tokens")
        if len(story) < 2 or any(
            isinstance(token, bool) or not isinstance(token, int) or not 0 <= token < vocab_size
            for token in story
        ):
            raise ValueError("structural story contains invalid current-token inputs")
        positions: dict[str, list[int]] = {name: [] for name in arms}
        story_r_action = 0
        # story[-1] is never opened as a target by this loop.  The final scored
        # row consumes story[-2] and commits story[-1] only outside this census.
        for current_token in story[:-1]:
            signatures: dict[str, tuple[int, ...]] = {}
            for name in arms:
                action = inverse_rows[name][leaves[name][current_token]]
                positions[name] = [action[slot] for slot in positions[name]]
                signatures[name] = tuple(
                    slot if slot in support else -1 for slot in positions[name]
                )
            exact_vs_cyclic = signatures["exact_h4"] != signatures["cyclic_120"]
            exact_vs_scrambled = signatures["exact_h4"] != signatures["scrambled_h4"]
            pairwise["exact_vs_cyclic"] += int(exact_vs_cyclic)
            pairwise["exact_vs_scrambled"] += int(exact_vs_scrambled)
            if exact_vs_cyclic and exact_vs_scrambled:
                story_r_action += 1
            for name in arms:
                positions[name].append(identity[name])
            total_rows += 1
        story_rows.append(story_r_action)
    return {
        "stories": len(stories),
        "rows": total_rows,
        "r_action": sum(story_rows),
        "stories_with_r_action": sum(value > 0 for value in story_rows),
        "minimum_r_action_per_story": min(story_rows) if story_rows else 0,
        "maximum_r_action_per_story": max(story_rows) if story_rows else 0,
        "pairwise_difference_rows": pairwise,
        "signature_contents": "ordered prior-write identities at true-candidate-readable relative slots",
        "next_token_reads": 0,
    }


def _write_exclusive(path: Path, value: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
    descriptor = os.open(path, flags, 0o600)
    try:
        with os.fdopen(descriptor, "wb", closefd=True) as target:
            target.write(value)
            target.flush()
            os.fsync(target.fileno())
        descriptor = -1
        directory = os.open(path.parent, os.O_RDONLY)
        try:
            os.fsync(directory)
        finally:
            os.close(directory)
    except BaseException:
        if descriptor >= 0:
            os.close(descriptor)
        raise


def _write_exclusive_json(path: Path, value: Mapping[str, Any]) -> None:
    _write_exclusive(path, canonical_json_bytes(value))


def _write_exclusive_bound_manifest(
    path: Path,
    payload: Mapping[str, Any],
    *,
    artifact_root: Path,
    relative_paths: Sequence[str],
) -> dict[str, Any]:
    records = artifact_records(artifact_root, relative_paths)
    value = dict(payload)
    value["artifacts"] = records
    value["tree_cid"] = tree_cid(records)
    value["manifest_cid"] = cid_bytes(canonical_json_bytes(value))
    _write_exclusive_json(path, value)
    return value


def _with_cid(value: Mapping[str, Any], field_name: str) -> dict[str, Any]:
    if field_name in value:
        raise ValueError(f"self-CID field already exists: {field_name}")
    result = dict(value)
    result[field_name] = cid_bytes(canonical_json_bytes(value))
    return result


def prepare_group_retention_data(
    root: Path, source_root: Path, geometry_path: Path
) -> dict[str, Any]:
    """Freeze geometry, structural opportunity, and the sealed population."""
    root = root.resolve()
    source_root = source_root.resolve()
    geometry_path = geometry_path.resolve()
    if (root / PREPARATION_MANIFEST_NAME).exists() or (root / GEOMETRY_RELATIVE_PATH).exists():
        raise FileExistsError("#973 campaign preparation is create-once; use a new empty root")
    geometry_bytes = geometry_path.read_bytes()
    geometry = load_group_geometry_artifacts(geometry_path)
    prepared = prepare_group_retention_population(root, source_root, geometry=geometry)
    destination = root / GEOMETRY_RELATIVE_PATH
    _write_exclusive(destination, geometry_bytes)
    training_view = prepared["training_view"]
    manifest = _write_exclusive_bound_manifest(
        root / PREPARATION_MANIFEST_NAME,
        {
            "schema": PREPARATION_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "population_manifest_cid": training_view["population_manifest_cid"],
            "training_view_manifest_cid": training_view["manifest_cid"],
            "geometry_artifact_cid": geometry.artifact_cid,
            "geometry_file_cid": geometry.geometry_file_cid,
            "main": {"status": MAIN_NOT_RUN, "authorized": False},
        },
        artifact_root=root,
        relative_paths=(GEOMETRY_RELATIVE_PATH, TRAINING_VIEW_MANIFEST_NAME),
    )
    return {**prepared, "preparation": manifest}


def _load_prepared(root: Path) -> tuple[dict[str, Any], dict[str, Any], GroupGeometryBundle]:
    root = root.resolve()
    preparation = verify_bound_manifest(
        root / PREPARATION_MANIFEST_NAME, artifact_root=root
    )
    if (
        preparation.get("schema") != PREPARATION_SCHEMA
        or preparation.get("issue") != ISSUE
        or preparation.get("policy") != POLICY
        or preparation.get("main") != {"status": MAIN_NOT_RUN, "authorized": False}
    ):
        raise ValueError("#973 preparation manifest differs from the frozen contract")
    training_view = load_group_retention_training_view(root)
    geometry = load_group_geometry_artifacts(root / GEOMETRY_RELATIVE_PATH)
    geometry_record = training_view.get("geometry")
    summary = geometry_record.get("summary") if isinstance(geometry_record, Mapping) else None
    if (
        preparation.get("population_manifest_cid")
        != training_view.get("population_manifest_cid")
        or preparation.get("training_view_manifest_cid") != training_view.get("manifest_cid")
        or preparation.get("geometry_artifact_cid") != geometry.artifact_cid
        or preparation.get("geometry_file_cid") != geometry.geometry_file_cid
        or not isinstance(summary, Mapping)
        or geometry_record.get("status") != "COMPUTED"
        or summary.get("geometry_artifact_cid") != geometry.artifact_cid
        or summary.get("geometry_file_cid") != geometry.geometry_file_cid
    ):
        raise ValueError("#973 preparation, population, and geometry identities disagree")
    return preparation, training_view, geometry


def _load_fit_sequences(root: Path) -> Tensor:
    """Read only the verified fit u16 store; no held-out path is named or opened."""
    raw = (root / FIT_TOKENS_RELATIVE_PATH).read_bytes()
    expected_values = FIT_STORY_COUNT * TOKENS_PER_STORY
    if len(raw) != expected_values * 2:
        raise ValueError("#973 fit store does not contain exactly 256 x 257 u16 tokens")
    values = tuple(item[0] for item in struct.iter_unpack("<H", raw))
    if len(values) != expected_values or any(value >= VOCAB_SIZE for value in values):
        raise ValueError("#973 fit store contains an invalid token")
    return torch.tensor(values, dtype=torch.long).view(FIT_STORY_COUNT, TOKENS_PER_STORY)


def _tensor_artifact_cid(artifact: Mapping[str, Tensor]) -> str:
    value = bytearray()
    for name in sorted(artifact):
        tensor = artifact[name].detach().to(device="cpu").contiguous()
        value.extend(
            canonical_json_bytes(
                {
                    "dtype": str(tensor.dtype),
                    "name": name,
                    "shape": list(tensor.shape),
                }
            )
        )
        value.extend(tensor.view(torch.uint8).numpy().tobytes(order="C"))
    return cid_bytes(bytes(value))


def _initialization_identity(
    arms: Mapping[str, GroupAddressArtifact],
    *,
    config: GroupRetentionConfig,
) -> tuple[dict[str, Any], dict[str, dict[str, Tensor]]]:
    exports: dict[str, dict[str, Tensor]] = {}
    cids: dict[str, str] = {}
    ledgers: dict[str, dict[str, int]] = {}
    for name in GEOMETRY_ARMS:
        model = R4GroupAddressedRetentionLMV1(config, arms[name])
        export = model.export_learned_artifact()
        exports[name] = export
        cids[name] = _tensor_artifact_cid(export)
        ledgers[name] = {
            "parameters": model.parameter_count(),
            "state_values_per_sequence": model.state_value_count(),
            "state_bytes_f32_per_sequence": model.state_value_count() * 4,
            "candidate_leaf_groups": model.candidate_leaf_group_count,
            "banks": config.banks,
            "group_size": config.group_size,
            "hidden_size": config.hidden_size,
            "vocab_size": config.vocab_size,
        }
    reference = exports["exact_h4"]
    if len(set(cids.values())) != 1 or len({tuple(ledger.items()) for ledger in ledgers.values()}) != 1:
        raise GroupRetentionPreflightUnavailable("arm initialization or analytic ledgers differ")
    for name, export in exports.items():
        if set(export) != set(reference) or any(
            not torch.equal(export[tensor_name], reference[tensor_name]) for tensor_name in reference
        ):
            raise GroupRetentionPreflightUnavailable(
                f"{name} does not have byte-identical learned initialization"
            )
    record = {
        "seed": config.initialization_seed,
        "learned_initialization_cid": next(iter(cids.values())),
        "arm_cids": cids,
        "byte_identical": True,
        "ledgers": ledgers,
        "equal_ledgers": True,
    }
    return record, exports


@dataclass(frozen=True, slots=True)
class PreflightExecutionConfig:
    """Injectable loop sizes; the public entrypoint always uses production()."""

    model: GroupRetentionConfig = field(default_factory=GroupRetentionConfig.production_unchecked)
    batch_size: int = 8
    context: int = PRODUCTION_CONTEXT
    warmup_steps: int = 2
    measured_steps: int = 8
    smoke_stories: int = 8
    smoke_steps: int = 64
    learning_rate: float = 3e-4
    weight_decay: float = 0.1
    beta1: float = 0.9
    beta2: float = 0.95
    epsilon: float = 1e-8
    gradient_clip: float = 1.0
    required_loss_reduction: float = 0.80
    required_state_off_delta: float = 0.05
    eta_safety_factor: float = ETA_SAFETY_FACTOR
    eta_total_steps: int = MAIN_TOTAL_OPTIMIZER_STEPS
    eta_ceiling_seconds: float = ETA_CEILING_SECONDS

    @classmethod
    def production(cls) -> PreflightExecutionConfig:
        return cls()

    def validate(self) -> None:
        self.model.validate()
        if (
            self.batch_size < 1
            or self.context < 1
            or self.context > self.model.max_sequence_length
            or self.warmup_steps < 0
            or self.measured_steps < 1
            or self.smoke_stories < 1
            or self.smoke_steps < 1
            or not 0.0 <= self.required_loss_reduction <= 1.0
            or self.required_state_off_delta < 0.0
            or self.eta_safety_factor <= 0.0
            or self.eta_total_steps < 1
            or self.eta_ceiling_seconds <= 0.0
        ):
            raise ValueError("invalid #973 preflight execution contract")

    def validate_production(self) -> None:
        self.validate()
        if self != PreflightExecutionConfig.production():
            raise ValueError("public #973 preflight exposes one frozen construction budget")


class DeviceTelemetry(Protocol):
    def synchronize(self) -> None: ...

    def empty_cache(self) -> None: ...

    def recommended_memory(self) -> int: ...

    def allocated_memory(self) -> int: ...


@dataclass(slots=True)
class _MpsTelemetry:
    def synchronize(self) -> None:
        torch.mps.synchronize()

    def empty_cache(self) -> None:
        torch.mps.empty_cache()

    def recommended_memory(self) -> int:
        return int(torch.mps.recommended_max_memory())

    def allocated_memory(self) -> int:
        return max(
            int(torch.mps.current_allocated_memory()),
            int(torch.mps.driver_allocated_memory()),
        )


def _optimizer(model: R4GroupAddressedRetentionLMV1, config: PreflightExecutionConfig):
    return torch.optim.AdamW(
        model.parameters(),
        lr=config.learning_rate,
        betas=(config.beta1, config.beta2),
        eps=config.epsilon,
        weight_decay=config.weight_decay,
    )


def _gradient_census(model: R4GroupAddressedRetentionLMV1) -> dict[str, Any]:
    values: dict[str, Any] = {}
    for name, parameter in model.named_parameters():
        gradient = parameter.grad
        if gradient is None:
            values[name] = {"finite": False, "nonzero_values": 0, "total_values": parameter.numel()}
            continue
        finite = bool(torch.isfinite(gradient).all().item())
        nonzero = int(torch.count_nonzero(gradient).item())
        record: dict[str, Any] = {
            "finite": finite,
            "nonzero_values": nonzero,
            "total_values": gradient.numel(),
        }
        if name == "query_table":
            record["nonzero_vocab_rows"] = int(
                torch.count_nonzero(torch.count_nonzero(gradient, dim=1)).item()
            )
        values[name] = record
    required = {
        "recurrence": "decay_logits",
        "overwrite": "write_logits",
        "retained_read": "bank_logits",
        "full_vocabulary_scoring": "query_table",
        "value_binding": "value_table",
    }
    paths = {
        concept: bool(values[name]["finite"] and values[name]["nonzero_values"] > 0)
        for concept, name in required.items()
    }
    query = values["query_table"]
    paths["full_vocabulary_scoring"] = bool(
        paths["full_vocabulary_scoring"]
        and query.get("nonzero_vocab_rows") == model.config.vocab_size
    )
    return {"parameters": values, "paths": paths, "passed": all(paths.values())}


def _one_training_step(
    model: R4GroupAddressedRetentionLMV1,
    optimizer: torch.optim.Optimizer,
    inputs: Tensor,
    targets: Tensor,
    config: PreflightExecutionConfig,
) -> tuple[float, Any]:
    optimizer.zero_grad(set_to_none=True)
    output = model(inputs, targets, use_checkpoint=PREFLIGHT_USE_CHECKPOINT)
    if output.loss is None or not bool(torch.isfinite(output.loss).item()):
        raise GroupRetentionPreflightUnavailable("construction loss is missing or non-finite")
    output.loss.backward()
    torch.nn.utils.clip_grad_norm_(model.parameters(), config.gradient_clip)
    optimizer.step()
    return float(output.loss.detach().item()), output.audit


@torch.no_grad()
def _loss(
    model: R4GroupAddressedRetentionLMV1,
    inputs: Tensor,
    targets: Tensor,
    *,
    state_off: bool = False,
) -> tuple[float, Any]:
    output = model(
        inputs,
        targets,
        state_off=state_off,
        use_checkpoint=PREFLIGHT_USE_CHECKPOINT,
    )
    if output.loss is None or not bool(torch.isfinite(output.loss).item()):
        raise GroupRetentionPreflightUnavailable("smoke loss is missing or non-finite")
    return float(output.loss.item()), output.audit


def _release_mps(telemetry: DeviceTelemetry) -> None:
    gc.collect()
    telemetry.empty_cache()
    telemetry.synchronize()


def _execute_construction_preflight(
    fit_sequences: Tensor,
    arms: Mapping[str, GroupAddressArtifact],
    *,
    device: torch.device,
    config: PreflightExecutionConfig,
    telemetry: DeviceTelemetry,
    initial_exports: Mapping[str, Mapping[str, Tensor]],
) -> dict[str, Any]:
    """Execute disposable timing/gradient and overfit loops sequentially."""
    config.validate()
    if fit_sequences.ndim != 2 or fit_sequences.shape[1] < config.context + 1:
        raise ValueError("fit sequences cannot supply the requested construction context")
    if fit_sequences.shape[0] < max(config.batch_size, config.smoke_stories):
        raise ValueError("fit sequences cannot supply the requested construction batch")
    recommended_memory = telemetry.recommended_memory()
    if recommended_memory <= 0:
        raise GroupRetentionPreflightUnavailable("MPS recommended-memory query is unavailable")

    timing_inputs = fit_sequences[: config.batch_size, : config.context].to(device)
    timing_targets = fit_sequences[: config.batch_size, 1 : config.context + 1].to(device)
    timing_arms: dict[str, Any] = {}
    timing_audits: dict[str, tuple[int, ...]] = {}
    all_seconds: list[float] = []
    gradient_pass = True
    memory_pass = True
    for arm_name in GEOMETRY_ARMS:
        _release_mps(telemetry)
        model = R4GroupAddressedRetentionLMV1(config.model, arms[arm_name]).to(device)
        model.load_learned_artifact_(initial_exports[arm_name])
        optimizer = _optimizer(model, config)
        for _ in range(config.warmup_steps):
            _one_training_step(model, optimizer, timing_inputs, timing_targets, config)
            telemetry.synchronize()
        measured: list[float] = []
        peak = telemetry.allocated_memory()
        last_audit = None
        for _ in range(config.measured_steps):
            telemetry.synchronize()
            started = time.perf_counter()
            _, last_audit = _one_training_step(
                model, optimizer, timing_inputs, timing_targets, config
            )
            telemetry.synchronize()
            measured.append(time.perf_counter() - started)
            peak = max(peak, telemetry.allocated_memory())
        if last_audit is None:
            raise RuntimeError("measured construction loop produced no evidence")
        # Inspect the already-materialized final measured gradients outside the
        # stopwatch so the diagnostic scan cannot inflate the main ETA.
        last_gradients = _gradient_census(model)
        mean_seconds = statistics.fmean(measured)
        all_seconds.extend(measured)
        gradient_pass = gradient_pass and bool(last_gradients["passed"])
        arm_memory_pass = peak < recommended_memory
        memory_pass = memory_pass and arm_memory_pass
        timing_audits[arm_name] = last_audit.work_signature()
        timing_arms[arm_name] = {
            "warmup_steps": config.warmup_steps,
            "measured_steps": config.measured_steps,
            "step_seconds": measured,
            "mean_step_seconds": mean_seconds,
            "observed_allocated_memory_bytes": peak,
            "recommended_memory_bytes": recommended_memory,
            "memory_passed": arm_memory_pass,
            "gradients": last_gradients,
            "work_signature": list(last_audit.work_signature()),
        }
        del optimizer, model
        _release_mps(telemetry)

    global_mean = statistics.fmean(all_seconds)
    projected = config.eta_safety_factor * global_mean * config.eta_total_steps
    timing_pass = projected <= config.eta_ceiling_seconds
    equal_work = len(set(timing_audits.values())) == 1

    smoke_inputs = fit_sequences[: config.smoke_stories, : config.context].to(device)
    smoke_targets = fit_sequences[: config.smoke_stories, 1 : config.context + 1].to(device)
    smoke_arms: dict[str, Any] = {}
    smoke_audits: dict[str, tuple[int, ...]] = {}
    smoke_pass = True
    for arm_name in GEOMETRY_ARMS:
        _release_mps(telemetry)
        model = R4GroupAddressedRetentionLMV1(config.model, arms[arm_name]).to(device)
        model.load_learned_artifact_(initial_exports[arm_name])
        optimizer = _optimizer(model, config)
        initial_loss, initial_audit = _loss(model, smoke_inputs, smoke_targets)
        for _ in range(config.smoke_steps):
            _one_training_step(model, optimizer, smoke_inputs, smoke_targets, config)
        telemetry.synchronize()
        final_loss, final_audit = _loss(model, smoke_inputs, smoke_targets)
        reduction = (initial_loss - final_loss) / initial_loss
        state_off_loss = None
        state_off_delta = None
        state_off_work_equal = None
        if arm_name == "exact_h4":
            state_off_loss, state_off_audit = _loss(
                model, smoke_inputs, smoke_targets, state_off=True
            )
            state_off_delta = state_off_loss - final_loss
            state_off_work_equal = (
                final_audit.work_signature() == state_off_audit.work_signature()
            )
        arm_pass = reduction >= config.required_loss_reduction
        if arm_name == "exact_h4":
            arm_pass = bool(
                arm_pass
                and state_off_delta is not None
                and state_off_delta >= config.required_state_off_delta
                and state_off_work_equal
            )
        smoke_pass = smoke_pass and arm_pass
        smoke_audits[arm_name] = initial_audit.work_signature()
        smoke_arms[arm_name] = {
            "stories": config.smoke_stories,
            "optimizer_steps": config.smoke_steps,
            "initial_ce_nats": initial_loss,
            "final_ce_nats": final_loss,
            "ce_reduction_fraction": reduction,
            "required_reduction_fraction": config.required_loss_reduction,
            "state_off_ce_nats": state_off_loss,
            "state_off_delta_nats": state_off_delta,
            "required_state_off_delta_nats": (
                config.required_state_off_delta if arm_name == "exact_h4" else None
            ),
            "state_off_work_equal": state_off_work_equal,
            "passed": arm_pass,
            "work_signature": list(initial_audit.work_signature()),
        }
        del optimizer, model
        _release_mps(telemetry)
    equal_work = equal_work and len(set(smoke_audits.values())) == 1
    passed = timing_pass and memory_pass and gradient_pass and equal_work and smoke_pass
    return {
        "execution_path": PREFLIGHT_EXECUTION_PATH,
        "use_checkpoint": PREFLIGHT_USE_CHECKPOINT,
        "direct_recurrence_parity": "REQUIRED",
        "timing": {
            "arms": timing_arms,
            "global_mean_step_seconds": global_mean,
            "eta_safety_factor": config.eta_safety_factor,
            "projected_main_seconds": projected,
            "ceiling_seconds": config.eta_ceiling_seconds,
            "passed": timing_pass,
        },
        "memory": {
            "recommended_bytes": recommended_memory,
            "measurement": "maximum synchronized MPS current-or-driver allocated bytes",
            "passed": memory_pass,
        },
        "gradients": {"all_arms_passed": gradient_pass, "passed": gradient_pass},
        "smoke": {"arms": smoke_arms, "passed": smoke_pass},
        "equal_operation_and_read_ledgers": equal_work,
        "training_presentations_per_arm": (
            (config.warmup_steps + config.measured_steps) * config.batch_size * config.context
            + config.smoke_steps * config.smoke_stories * config.context
        ),
        "passed": passed,
    }


PreflightExecutor = Callable[..., Mapping[str, Any]]


def _structural_gate(training_view: Mapping[str, Any]) -> dict[str, Any]:
    geometry = training_view.get("geometry")
    summary = geometry.get("summary") if isinstance(geometry, Mapping) else None
    heldout = summary.get("heldout") if isinstance(summary, Mapping) else None
    coverage = summary.get("generated_state_coverage") if isinstance(summary, Mapping) else None
    passed = bool(
        geometry.get("status") == "COMPUTED"
        and isinstance(heldout, Mapping)
        and isinstance(coverage, Mapping)
        and summary.get("passed") is True
        and heldout.get("r_action", -1) >= R_ACTION_MINIMUM
        and heldout.get("stories_with_r_action") == HELDOUT_STORY_COUNT
        and all(coverage.get(arm) == PRODUCTION_GROUP_SIZE for arm in GEOMETRY_ARMS)
        and summary.get("next_token_reads") == 0
    )
    return {
        "r_action": heldout.get("r_action") if isinstance(heldout, Mapping) else None,
        "required_r_action": R_ACTION_MINIMUM,
        "stories_with_r_action": (
            heldout.get("stories_with_r_action") if isinstance(heldout, Mapping) else None
        ),
        "required_stories_with_r_action": HELDOUT_STORY_COUNT,
        "generated_state_coverage": dict(coverage) if isinstance(coverage, Mapping) else None,
        "next_token_reads": summary.get("next_token_reads") if isinstance(summary, Mapping) else None,
        "passed": passed,
    }


def _require_mps_device(backend: str) -> tuple[torch.device, DeviceTelemetry]:
    if backend != "mps":
        raise ValueError("#973 construction permits only backend='mps'; CPU fallback and CUDA are forbidden")
    return require_mps(PRODUCTION_INITIALIZATION_SEED), _MpsTelemetry()


def _production_contract(config: PreflightExecutionConfig) -> dict[str, Any]:
    return {
        "backend": "mps",
        "model": asdict(config.model),
        "construction": {
            "batch_size": config.batch_size,
            "context": config.context,
            "execution_path": PREFLIGHT_EXECUTION_PATH,
            "use_checkpoint": PREFLIGHT_USE_CHECKPOINT,
            "direct_recurrence_parity": "REQUIRED",
            "reference_checkpoint_chunk": REFERENCE_CHECKPOINT_CHUNK,
            "reference_checkpoint_role": (
                "model equivalence reference only; excluded from preflight work"
            ),
            "execution_selection_rationale": {
                "observed_current_memory_bytes": PREFLIGHT_OBSERVED_CURRENT_MEMORY_BYTES,
                "observed_driver_memory_bytes": PREFLIGHT_OBSERVED_DRIVER_MEMORY_BYTES,
                "observed_recommended_memory_bytes": (
                    PREFLIGHT_OBSERVED_RECOMMENDED_MEMORY_BYTES
                ),
                "driver_below_recommended": True,
                "binding_run_requirement": (
                    "query and record fresh synchronized MPS memory; observed allocation "
                    "must remain below the device recommendation"
                ),
            },
            "warmup_steps_per_arm": config.warmup_steps,
            "measured_steps_per_arm": config.measured_steps,
            "smoke_stories": config.smoke_stories,
            "smoke_steps_per_arm": config.smoke_steps,
            "required_ce_reduction_fraction": config.required_loss_reduction,
            "required_h4_state_off_delta_nats": config.required_state_off_delta,
            "eta_formula": "1.25 * mean_step_seconds * 768",
            "eta_ceiling_seconds": config.eta_ceiling_seconds,
        },
        "main": {
            "status": MAIN_NOT_RUN,
            "optimizer_steps_per_arm": MAIN_OPTIMIZER_STEPS_PER_ARM,
            "total_optimizer_steps": MAIN_TOTAL_OPTIMIZER_STEPS,
            "presentations_per_arm": MAIN_PRESENTATIONS_PER_ARM,
            "total_presentations": MAIN_TOTAL_PRESENTATIONS,
            "combined_hard_ceiling_seconds": MAIN_WALL_CEILING_SECONDS,
            "seed": PRODUCTION_INITIALIZATION_SEED,
            "optimizer": {
                "name": "AdamW",
                "learning_rate": 3e-4,
                "minimum_learning_rate": 3e-5,
                "warmup_steps": 16,
                "schedule": "cosine",
                "beta1": 0.9,
                "beta2": 0.95,
                "epsilon": 1e-8,
                "weight_decay": 0.1,
                "gradient_clip": 1.0,
            },
            "retry_or_sweep": "FORBIDDEN",
        },
    }


def run_group_retention_preflight(
    root: Path,
    backend: str = "mps",
    *,
    _executor: PreflightExecutor | None = None,
    _device_provider: Callable[[str], tuple[torch.device, DeviceTelemetry]] | None = None,
    _execution_config: PreflightExecutionConfig | None = None,
) -> dict[str, Any]:
    """Run the sole construction gate and return a frozen main authorization."""
    root = root.resolve()
    config = PreflightExecutionConfig.production() if _execution_config is None else _execution_config
    if _execution_config is None:
        config.validate_production()
    else:
        config.validate()
    if backend != "mps":
        raise ValueError("#973 construction permits only backend='mps'; CPU fallback and CUDA are forbidden")
    for relative in (STARTED_RELATIVE_PATH, RESULT_RELATIVE_PATH, AUTHORIZATION_RELATIVE_PATH):
        if (root / relative).exists() or (root / relative).is_symlink():
            raise FileExistsError("the sole #973 construction preflight already has a terminal output")

    preparation, training_view, geometry = _load_prepared(root)
    fit_sequences = _load_fit_sequences(root)
    structural = _structural_gate(training_view)
    initialization, initial_exports = _initialization_identity(
        geometry.arms, config=config.model
    )
    contract = _production_contract(config)
    implementation = trainer_implementation_contract()
    started = _with_cid(
        {
            "schema": STARTED_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "preparation_manifest_cid": preparation["manifest_cid"],
            "population_manifest_cid": training_view["population_manifest_cid"],
            "geometry_artifact_cid": geometry.artifact_cid,
            "implementation": implementation,
            "initialization": initialization,
            "contract": contract,
            "main": {"status": MAIN_NOT_RUN, "authorized": False},
        },
        "started_cid",
    )
    _write_exclusive_json(root / STARTED_RELATIVE_PATH, started)

    executor = _execute_construction_preflight if _executor is None else _executor
    device_provider = _require_mps_device if _device_provider is None else _device_provider
    execution: Mapping[str, Any] | None = None
    failure: dict[str, str] | None = None
    try:
        if not structural["passed"]:
            raise GroupRetentionPreflightUnavailable(
                "label-free structural opportunity or 120-state coverage gate missed"
            )
        device, telemetry = device_provider(backend)
        execution = executor(
            fit_sequences,
            geometry.arms,
            device=device,
            config=config,
            telemetry=telemetry,
            initial_exports=initial_exports,
        )
        if (
            not isinstance(execution, Mapping)
            or execution.get("passed") is not True
            or execution.get("execution_path") != PREFLIGHT_EXECUTION_PATH
            or execution.get("use_checkpoint") is not PREFLIGHT_USE_CHECKPOINT
            or execution.get("direct_recurrence_parity") != "REQUIRED"
        ):
            raise GroupRetentionPreflightUnavailable(
                "stationary-path, parity, timing, memory, gradient, equal-ledger, "
                "or disposable smoke gate missed"
            )
    except Exception as error:  # the create-once marker makes every run error terminal
        failure = {"type": type(error).__name__, "reason": str(error)}

    passed = failure is None
    result = _with_cid(
        {
            "schema": RESULT_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "started_cid": started["started_cid"],
            "preparation_manifest_cid": preparation["manifest_cid"],
            "population_manifest_cid": training_view["population_manifest_cid"],
            "geometry_artifact_cid": geometry.artifact_cid,
            "initialization": initialization,
            "structural_gate": structural,
            "construction_execution": dict(execution) if execution is not None else None,
            "verdict": "PASS" if passed else TERMINAL_UNAVAILABLE,
            "failure": failure,
            "heldout": {"status": MAIN_NOT_RUN, "reads": 0},
            "main": {
                "status": MAIN_AUTHORIZED_NOT_RUN if passed else MAIN_NOT_RUN,
                "authorized": passed,
            },
        },
        "result_cid",
    )
    _write_exclusive_json(root / RESULT_RELATIVE_PATH, result)
    authorization = None
    if passed:
        authorization = _with_cid(
            {
                "schema": AUTHORIZATION_SCHEMA,
                "issue": ISSUE,
                "policy": POLICY,
                "result_cid": result["result_cid"],
                "started_cid": started["started_cid"],
                "preparation_manifest_cid": preparation["manifest_cid"],
                "population_manifest_cid": training_view["population_manifest_cid"],
                "geometry_artifact_cid": geometry.artifact_cid,
                "learned_initialization_cid": initialization["learned_initialization_cid"],
                "authorization": "ONE_SHOT_MAIN_256_STEPS_PER_ARM",
                "contract": contract["main"],
                "heldout": {"status": MAIN_NOT_RUN, "reads": 0},
            },
            "authorization_cid",
        )
        _write_exclusive_json(root / AUTHORIZATION_RELATIVE_PATH, authorization)
    return {"result": result, "authorization": authorization}
