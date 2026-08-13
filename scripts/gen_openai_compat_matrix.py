#!/usr/bin/env python3
"""Generate the pinned OpenAI compatibility matrix for the `r4-openai-profile`
(#654 phase A).

Deterministic and dependency-free: it parses the vendored
`profiles/openai/openapi.yaml` by a simple indentation line scan (the file is
pinned and byte-verified, so the scan is stable), classifies every operation as
`supported` / `unsupported` / `not-applicable`, and writes
`profiles/openai/compatibility_matrix.json`.

The Rust drift gate `tests/openai_profile_pin.rs` re-derives the operation set
from the same vendored spec and fails CI if the matrix and the spec disagree,
so a spec bump or a matrix edit cannot silently widen or drift the profile.

The classification is intentionally small and explicit: only the four phase-1
text-serving operations are `supported`; a recorded set of text/inference
adjacent operations is `unsupported` (out of the phase-1 profile, never implied
by omission); everything else is `not-applicable` platform surface.
"""

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SPEC = ROOT / "profiles" / "openai" / "openapi.yaml"
OUT = ROOT / "profiles" / "openai" / "compatibility_matrix.json"

# The pinned upstream identity (recorded, byte-verified below and in the Rust
# drift gate). git blob sha1 is `git hash-object openapi.yaml`; blake3 is the
# content hash the Rust gate re-verifies (blake3 is a workspace dependency).
SPEC_PIN = {
    "upstream_repo": "openai/openai-openapi",
    "commit": "11854aef674352d3f9cd5c0a7038f079a7bbac06",
    "openapi_yaml_git_blob_sha1": "b4e4080c0baf909bc6bfa293dd0efa553dfb0a29",
    "openapi_yaml_blake3": "blake3:396df55705eaca49b0f87c606a150443c4c0bd291efc3347cb8497f11d6e60f6",
    "openapi_version": "3.1.0",
    "api_version": "2.3.0",
    "license": "MIT",
    "retrieval": "curl -sSL https://raw.githubusercontent.com/openai/openai-openapi/11854aef674352d3f9cd5c0a7038f079a7bbac06/openapi.yaml",
}

# Phase-1 R4 text-serving profile: the operations the profile supports.
SUPPORTED = {
    "createChatCompletion": "POST /chat/completions — the phase-1 chat text-generation operation.",
    "listModels": "GET /models — model discovery over the loadable R4 models.",
    "retrieveModel": "GET /models/{model} — single-model discovery, agreeing with the list.",
    "createResponse": "POST /responses — the recommended text-generation operation; routed through the same internal generation adapter as chat completions.",
}

# Text/inference-adjacent operations explicitly OUT of the phase-1 profile.
# Recorded so support is never implied by omission (#654).
UNSUPPORTED = {
    "createCompletion": "Legacy POST /completions — explicitly out of the phase-1 profile (recorded decision, not implied by omission).",
    "createEmbedding": "Embeddings are not a text-generation operation in this profile.",
    "listChatCompletions": "Stored-completions management; R4 serving is stateless.",
    "getChatCompletion": "Stored-completions management; R4 serving is stateless.",
    "updateChatCompletion": "Stored-completions management; R4 serving is stateless.",
    "deleteChatCompletion": "Stored-completions management; R4 serving is stateless.",
    "getChatCompletionMessages": "Stored-completions management; R4 serving is stateless.",
    "getResponse": "Stateful response retrieval; phase-1 generation is stateless.",
    "deleteResponse": "Stateful response management; phase-1 generation is stateless.",
    "cancelResponse": "Background/stateful response cancellation; phase-1 generation is synchronous.",
    "listInputItems": "Stateful response input-item listing; phase-1 generation is stateless.",
    "Getinputtokencounts": "Response input-token-count endpoint; not in the phase-1 profile.",
    "Compactconversation": "Stateful conversation compaction; phase-1 generation is stateless.",
}

# Beta channel (`?beta=true`) duplicates of the responses surface.
BETA_PREFIX = "beta_"


def classify(operation_id: str):
    if operation_id in SUPPORTED:
        return "supported", SUPPORTED[operation_id]
    if operation_id in UNSUPPORTED:
        return "unsupported", UNSUPPORTED[operation_id]
    if operation_id.startswith(BETA_PREFIX):
        return (
            "unsupported",
            "Beta channel of the responses surface; the stable phase-1 profile does not include beta variants.",
        )
    return (
        "not-applicable",
        "OpenAI platform operation outside the local R4 text-model profile (see #654 non-goals).",
    )


def git_blob_sha1(data: bytes) -> str:
    header = f"blob {len(data)}\0".encode()
    return hashlib.sha1(header + data).hexdigest()


def main() -> None:
    raw = SPEC.read_bytes()
    computed = git_blob_sha1(raw)
    assert computed == SPEC_PIN["openapi_yaml_git_blob_sha1"], (
        f"vendored openapi.yaml git blob {computed} != pinned "
        f"{SPEC_PIN['openapi_yaml_git_blob_sha1']}"
    )

    path = method = None
    ops = []
    for line in raw.decode().splitlines():
        m = re.match(r"^  (/\S*):\s*$", line)
        if m:
            path, method = m.group(1), None
            continue
        m = re.match(r"^    (get|post|put|delete|patch):\s*$", line)
        if m:
            method = m.group(1)
            continue
        m = re.match(r"^      operationId:\s*(\S+)\s*$", line)
        if m and path and method:
            ops.append((m.group(1), path, method))

    ids = [o[0] for o in ops]
    assert len(ids) == len(set(ids)), "operationIds are not unique in the spec"

    operations = []
    for operation_id, path, method in sorted(ops):
        classification, rationale = classify(operation_id)
        entry = {
            "operation_id": operation_id,
            "path": path,
            "method": method,
            "classification": classification,
            "rationale": rationale,
        }
        if classification != "not-applicable":
            entry["owning_issue"] = 654
            entry["phase"] = (
                "A (profile definition); wire implementation in later phases"
                if classification == "supported"
                else "A (classified out of the phase-1 profile)"
            )
        operations.append(entry)

    counts = {
        c: sum(1 for o in operations if o["classification"] == c)
        for c in ("supported", "unsupported", "not-applicable")
    }
    counts["total"] = len(operations)

    doc = {
        "_note": "r4-openai-profile compatibility matrix (#654 phase A). Regenerate with scripts/gen_openai_compat_matrix.py; the Rust drift gate tests/openai_profile_pin.rs verifies this stays consistent with the vendored spec.",
        "profile": "r4-openai-profile",
        "profile_version": 1,
        "spec_pin": SPEC_PIN,
        "counts": counts,
        "operations": operations,
    }
    OUT.write_text(json.dumps(doc, indent=2) + "\n")
    print(f"wrote {OUT.relative_to(ROOT)} — {counts}")


if __name__ == "__main__":
    main()
