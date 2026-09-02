"""Source attribution and recursive implementation binding for issue #1047.

The MQAR construction in :mod:`r4_softmax_trainer.zoology_control.data` is a
bounded-memory adaptation of HazyResearch/Zoology's ICLR24 ``_mqar`` routine.
The upstream work is Apache-2.0 licensed.  This module records the exact
upstream revisions and source-file content hashes rather than treating the
later repository head as the released executable oracle.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

from ..provenance import (
    artifact_records,
    canonical_json_bytes,
    cid_bytes,
    tree_cid,
)


ISSUE = 1047
POLICY = "ZoologyMQARControlV1"
ZOOLOGY_REPOSITORY = "https://github.com/HazyResearch/zoology"
ZOOLOGY_LICENSE = "Apache-2.0"
ZOOLOGY_RELEASE_REVISION = "de4e258784224e09909c257ff3ea040f089ed660"
ZOOLOGY_LATER_REVISION = "1ad20d193b6113cae1e8f3c655c300d7b4b3f4bb"

ZOOLOGY_RELEASE_MQAR_SOURCE_CID = (
    "blake3:09fc47699144ba3b4f50093661be960d6881e1cbb4ed3d8f24bc53663fde9204"
)
ZOOLOGY_RELEASE_DATA_UTILS_SOURCE_CID = (
    "blake3:1fb44d9d431b03e8eb7e40ee062c798dbfc8a06f06d9fb03714f4b59d00b1584"
)
ZOOLOGY_RELEASE_ATTENTION_SOURCE_CID = (
    "blake3:c67f69d373302d019aafcd273e8b367143eaf7463e727ec1fc32e26eb9cd9502"
)
ZOOLOGY_RELEASE_MODEL_SOURCE_CID = (
    "blake3:e1b894edf594730dcc305e92acd78e9c752ec63a33413cbe7dd0f642fceac3f9"
)
ZOOLOGY_RELEASE_TRAIN_SOURCE_CID = (
    "blake3:adf492f3ac0dbfd53c4899fe33737ac92ec75ff6aa755341b95cadc23c078aac"
)
ZOOLOGY_RELEASE_FIGURE2_SOURCE_CID = (
    "blake3:3d4bbf2c2a1f2bf2fa74fc3828b9e3b43569268b45680f5c9ad9a3c026df841b"
)
ZOOLOGY_RELEASE_LICENSE_CID = (
    "blake3:6f1967ff88a71b26c1e9b2ae83c073256e76aa00dd0ca5145f99715d13695878"
)
ZOOLOGY_LATER_MQAR_SOURCE_CID = (
    "blake3:af676426a692ad514f2296fed7011196f97142203918191186d7b676bb604863"
)


def _source_url(revision: str, path: str) -> str:
    return f"{ZOOLOGY_REPOSITORY}/blob/{revision}/{path}"


def zoology_source_attribution() -> dict[str, Any]:
    """Return a canonical, self-addressed record of copied source authority."""

    body: dict[str, Any] = {
        "schema": "uor-r4/zoology-source-attribution/v1",
        "issue": ISSUE,
        "policy": POLICY,
        "repository": ZOOLOGY_REPOSITORY,
        "license": ZOOLOGY_LICENSE,
        "release_oracle": {
            "revision": ZOOLOGY_RELEASE_REVISION,
            "files": [
                {
                    "path": "zoology/data/associative_recall.py",
                    "bytes": 14_111,
                    "cid": ZOOLOGY_RELEASE_MQAR_SOURCE_CID,
                    "url": _source_url(
                        ZOOLOGY_RELEASE_REVISION,
                        "zoology/data/associative_recall.py",
                    ),
                },
                {
                    "path": "zoology/data/utils.py",
                    "bytes": 6_580,
                    "cid": ZOOLOGY_RELEASE_DATA_UTILS_SOURCE_CID,
                    "url": _source_url(
                        ZOOLOGY_RELEASE_REVISION,
                        "zoology/data/utils.py",
                    ),
                },
                {
                    "path": "zoology/mixers/attention.py",
                    "bytes": 2_207,
                    "cid": ZOOLOGY_RELEASE_ATTENTION_SOURCE_CID,
                    "url": _source_url(
                        ZOOLOGY_RELEASE_REVISION,
                        "zoology/mixers/attention.py",
                    ),
                },
                {
                    "path": "zoology/model.py",
                    "bytes": 7_162,
                    "cid": ZOOLOGY_RELEASE_MODEL_SOURCE_CID,
                    "url": _source_url(
                        ZOOLOGY_RELEASE_REVISION,
                        "zoology/model.py",
                    ),
                },
                {
                    "path": "zoology/train.py",
                    "bytes": 6_285,
                    "cid": ZOOLOGY_RELEASE_TRAIN_SOURCE_CID,
                    "url": _source_url(
                        ZOOLOGY_RELEASE_REVISION,
                        "zoology/train.py",
                    ),
                },
                {
                    "path": "zoology/experiments/paper/figure2.py",
                    "bytes": 5_311,
                    "cid": ZOOLOGY_RELEASE_FIGURE2_SOURCE_CID,
                    "url": _source_url(
                        ZOOLOGY_RELEASE_REVISION,
                        "zoology/experiments/paper/figure2.py",
                    ),
                },
                {
                    "path": "LICENSE.md",
                    "bytes": 11_346,
                    "cid": ZOOLOGY_RELEASE_LICENSE_CID,
                    "url": _source_url(ZOOLOGY_RELEASE_REVISION, "LICENSE.md"),
                },
            ],
        },
        "later_provenance_only": {
            "revision": ZOOLOGY_LATER_REVISION,
            "files": [
                {
                    "path": "zoology/data/multiquery_ar.py",
                    "bytes": 5_983,
                    "cid": ZOOLOGY_LATER_MQAR_SOURCE_CID,
                    "url": _source_url(
                        ZOOLOGY_LATER_REVISION,
                        "zoology/data/multiquery_ar.py",
                    ),
                }
            ],
        },
        "adaptations": [
            "local legacy NumPy RandomState with the released integer stream",
            "bounded row-wise sampling instead of materialized tiled vocabularies",
            "query-position-only batching",
            "deterministic explicit release-style epoch shuffle",
            "device-neutral CPU tensors and create-once UOR provenance containers",
        ],
        "claim_boundary": (
            "scaled source-derived CPU control; not byte-identical and not a full "
            "published Figure 2 reproduction"
        ),
    }
    return {
        **body,
        "attribution_cid": cid_bytes(canonical_json_bytes(body)),
    }


def zoology_control_implementation_contract(
    root: Path | None = None,
) -> dict[str, Any]:
    """Recursively bind the nested control package, tests, and locked inputs."""

    trainer_root = (
        Path(__file__).resolve().parents[3] if root is None else root.resolve()
    )
    package_root = trainer_root / "src" / "r4_softmax_trainer" / "zoology_control"
    tests_root = trainer_root / "tests"
    if not package_root.is_dir() or not tests_root.is_dir():
        raise FileNotFoundError("Zoology control package or test root is missing")

    paths = [
        str(path.relative_to(trainer_root))
        for path in sorted(package_root.rglob("*.py"))
        if path.is_file() and "__pycache__" not in path.parts
    ]
    paths.extend(
        str(path.relative_to(trainer_root))
        for path in sorted(tests_root.rglob("test_zoology_control*.py"))
        if path.is_file()
    )
    for support in (
        "src/r4_softmax_trainer/zoology_control/NOTICE.md",
        "src/r4_softmax_trainer/zoology_control/LICENSE-APACHE-2.0.md",
        "pyproject.toml",
        "uv.lock",
    ):
        if (trainer_root / support).is_file():
            paths.append(support)
    if not paths:
        raise ValueError("Zoology control implementation tree is empty")

    records = artifact_records(trainer_root, paths)
    source = zoology_source_attribution()
    body = {
        "schema": "uor-r4/zoology-control-implementation/v1",
        "issue": ISSUE,
        "policy": POLICY,
        "source_attribution_cid": source["attribution_cid"],
        "files": records,
        "tree_cid": tree_cid(records),
    }
    return {
        **body,
        "implementation_cid": cid_bytes(canonical_json_bytes(body)),
    }
