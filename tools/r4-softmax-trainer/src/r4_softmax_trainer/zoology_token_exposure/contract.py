"""Bind the prior result and construction-only diagnostic before outcome access."""

from __future__ import annotations

from pathlib import Path

import torch

from ..provenance import artifact_records, canonical_json_bytes, cid_bytes, tree_cid
from ..zoology_compound_binding.campaign import _tensor_cid
from ..zoology_language_interface import campaign as ordinary
from ..zoology_language_interface import data
from ..zoology_language_r4 import contract as historical
from ..zoology_language_r4.attention import frame_assignment
from ..zoology_r4_inference.campaign import _write_exclusive
from ..zoology_r4_inference.frames import load_frames
from .diagnostic import METRICS, ROLE_NAMES

ISSUE = 1082
POLICY = {
    "name": "FrozenConstructionTokenExposureV1",
    "construction_views": [0, 1],
    "rows_per_view": 10240,
    "used_roles_per_row": 14,
    "computed_roles_per_row": 15,
    "threads": 4,
    "interop_threads": 1,
    "batch_size": 256,
    "max_elapsed_seconds_per_phase": 120,
    "max_rss_bytes": 3 * 1024**3,
    "max_disk_bytes": 256 * 1024**2,
    "budget_enforcement": "cooperative checks between bindings, batches and views; external process timeout also required for the 120-second wall limit",
    "role_order": ROLE_NAMES,
    "metric_order": list(METRICS),
    "control": "unchanged #1079 next-token-source frame within each valid clause; true local encoding",
    "value_coordinates": "decode each individual coherent/controlled f64 token value to the shared embedding coordinates",
    "attention": "original f32 reader weights promoted to f64; no renormalization of A or D",
    "mass": "M=sum a_i*changed_matrix_i; fraction=M/sum a_i",
    "displacement": "A=sum a_i*norm(delta_i); D=norm(sum a_i*delta_i); norm is Euclidean over all 64 lanes",
    "used_displacement": "norm(controlled_f32_role.double()-coherent_f32_role.double()) before original norms/projections",
    "ratio": "D/A when A>0; zero placeholder at A=0, excluded from ratio summaries",
    "zero_cases": "M=0 implies A=D=0; changed matrices can act identically on a value, so M>0 need not imply A>0",
    "float_checks": "attention sum within1e-5; D<=A+1e-12*(1+A); pool closure within1e-12*(1+max_abs_pool)",
    "summaries": "each used role, each view; supported/unknown x all/changed/retained recorded answers",
    "quantiles": [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0],
    "summary_units": "mass and ratio dimensionless; all value norms in original embedding coordinates",
    "threshold_selection": "none; descriptive distributions do not identify a causal mechanism or choose a replacement control",
    "artifact": "C-order little-endian f64 [view_row,14,7]; recorded view selection order; ratio mask is A>0",
    "head_forwards": 0,
    "development_tensor_reads": 0,
    "development_identity_hashes": "retained lineage may hash files; no development tensors are decoded or scored",
    "new_predictions": 0,
    "new_fits": 0,
    "new_population": 0,
    "new_controls": 0,
    "geometry_changes": 0,
    "generation": 0,
}
SOURCE_CIDS = {
    "preparation": "blake3:d9c8ad8448365b2039276fdeda6b70da53ef63fde24e02dd1dd8dea437b546a4",
    "result": "blake3:dee107190172afcb7637d52469662ecab217847271e4bbdb0721514fcfbdc3a5",
    "replay": "blake3:eaa17433d5cd150a2a0c52adab6104bda4c4dae26221944fcde112ef841ca597",
}


def _repo():
    return Path(__file__).resolve().parents[5]


def _source(root):
    paths = {
        name: f"docs/r4_zoology_language_r4_1079_{name}.json" for name in SOURCE_CIDS
    }
    documents = {}
    for name, cid in SOURCE_CIDS.items():
        value = historical.prior._envelope(root / f"{name}.json", f"{name}_cid", cid)
        if value != historical.prior._envelope(
            _repo() / paths[name], f"{name}_cid", cid
        ):
            raise ValueError("#1079 local/public envelopes differ")
        documents[name] = value
    preparation, result, replay = (documents[name] for name in SOURCE_CIDS)
    source = historical._source_contract(Path(preparation["source"]["root"]))
    frames = historical._frames(Path(preparation["frames"]["root"]), source)
    files = artifact_records(
        _repo(), [row["path"] for row in preparation["implementation"]["files"]]
    )
    if (
        len(files) != 307
        or files != preparation["implementation"]["files"]
        or tree_cid(files) != preparation["implementation"]["tree_cid"]
    ):
        raise ValueError("frozen #1079 implementation changed")
    if source != preparation["source"] or frames != preparation["frames"]:
        raise ValueError("frozen reader/core/frame/data binding changed")
    if (
        result["preparation_cid"] != preparation["preparation_cid"]
        or result["implementation_cid"] != preparation["implementation"]["tree_cid"]
        or result["evidence_cid"] != cid_bytes(canonical_json_bytes(result["evidence"]))
        or result["evidence"]["status"] != "LANGUAGE_R4_PRESERVED_CONTROL_WEAK"
        or not replay["exact_replay"]
        or not replay["fresh_process"]
        or replay["result_cid"] != result["result_cid"]
        or replay["evidence_cid"] != result["evidence_cid"]
        or replay["process_id"] == result["process_id"]
    ):
        raise ValueError("#1079 result/replay relationship changed")
    views = []
    for view_id in POLICY["construction_views"]:
        primary = next(
            row
            for row in result["evidence"]["primary"]["views"]
            if row["population"] == "construction" and row["view_id"] == view_id
        )
        controlled = next(
            row
            for row in result["evidence"]["controls"]["views"]
            if row["population"] == "construction"
            and row["view_id"] == view_id
            and row["execution"] == "token_source_frame_permuted"
        )
        coherent = primary["r4"]
        if (
            not primary["passed"]
            or not controlled["valid"]
            or coherent["role_attention_cid"] != controlled["role_attention_cid"]
        ):
            raise ValueError(
                "construction reference/control is not the preserved valid pair"
            )
        views.append(
            {"view_id": view_id, "coherent": coherent, "controlled": controlled}
        )
    return {
        "root": str(root),
        "cids": SOURCE_CIDS,
        "documents": artifact_records(root, [f"{name}.json" for name in SOURCE_CIDS]),
        "public_documents": artifact_records(_repo(), paths.values()),
        "construction": views,
        "source": source,
        "frames": frames,
        "implementation_files": files,
    }


def _construction(source):
    tensors = data.load_construction(Path(source["source"]["root"]) / "data")
    if sorted(tensors["view_ids"].unique().tolist()) != POLICY["construction_views"]:
        raise ValueError("construction views differ")
    frames = load_frames(Path(source["frames"]["root"]))
    matrix_changes = (
        frames.frame_matrices[:, None] != frames.frame_matrices[None, :]
    ).any(dim=(-2, -1))
    result = []
    for reference in source["construction"]:
        view_id = reference["view_id"]
        view = ordinary._view(tensors, view_id)
        if len(view["inputs"]) != POLICY["rows_per_view"]:
            raise ValueError("construction row count differs")
        # No learned artifact, role weight or alternate control is accessed here.
        tokens, _ = frame_assignment(view["inputs"], view["lengths"], frames)
        offsets = torch.arange(tokens.shape[-1]).reshape(1, 1, -1).expand_as(tokens)
        valid = offsets < view["lengths"].unsqueeze(-1)
        shifted = tokens.gather(2, (offsets + 1) % view["lengths"].unsqueeze(-1))
        changed_mask = matrix_changes[tokens, shifted] & valid
        changed = int(changed_mask.sum())
        if (
            changed
            != reference["controlled"]["audit"]["token_source_frame_matrices_changed"]
        ):
            raise ValueError(
                "construction token control differs from recorded matrices"
            )
        for arm in ("coherent", "controlled"):
            if len(reference[arm]["prediction_ids"]) != len(view["inputs"]):
                raise ValueError("recorded construction answers are incomplete")
        result.append(
            {
                "view_id": view_id,
                "rows": len(view["inputs"]),
                "input_cid": _tensor_cid(view["inputs"]),
                "lengths_cid": _tensor_cid(view["lengths"]),
                "group_ids_cid": _tensor_cid(view["group_ids"]),
                "variant_ids_cid": _tensor_cid(view["variant_ids"]),
                "valid_tokens": int(view["lengths"].sum()),
                "changed_source_matrices": changed,
                "supported_rows_with_changed_matrix": int(
                    (changed_mask.any(dim=(1, 2)) & (view["variant_ids"] < 4)).sum()
                ),
                "used_role_rows": 14 * len(view["inputs"]),
                "supported_rows": int((view["variant_ids"] < 4).sum()),
                "unknown_rows": int((view["variant_ids"] == 4).sum()),
            }
        )
    return result


def _bindings(source_root):
    source = _source(source_root)
    paths = {
        row["path"]
        for row in source["implementation_files"] + source["public_documents"]
    }
    paths.update(
        str(path.relative_to(_repo())) for path in Path(__file__).parent.glob("*.py")
    )
    paths.update(
        str(path.relative_to(_repo()))
        for path in (_repo() / "tools/r4-softmax-trainer/tests").glob(
            "test_zoology_token_exposure*.py"
        )
    )
    files = artifact_records(_repo(), paths)
    return {
        "historical": source,
        "construction": _construction(source),
        "implementation": {
            "root": str(_repo()),
            "files": files,
            "tree_cid": tree_cid(files),
        },
    }


def prepare(root: Path, source_root: Path):
    root, source_root = root.resolve(), source_root.resolve()
    root.mkdir(parents=True, exist_ok=True)
    started = _write_exclusive(
        root / "preparation-started.json",
        {"issue": ISSUE, "source_root": str(source_root)},
        "started_cid",
    )
    body = {
        "schema": "uor-r4.token-exposure-preparation/1",
        "issue": ISSUE,
        "policy": POLICY,
        "preparation_started_cid": started["started_cid"],
        **_bindings(source_root),
    }
    return _write_exclusive(root / "preparation.json", body, "preparation_cid")


def validate_preparation(root):
    body = historical.prior._envelope(root / "preparation.json", "preparation_cid")
    started = historical.prior._envelope(
        root / "preparation-started.json", "started_cid"
    )
    if (
        body["schema"] != "uor-r4.token-exposure-preparation/1"
        or body["issue"] != ISSUE
        or body["policy"] != POLICY
        or started["issue"] != ISSUE
        or started["started_cid"] != body["preparation_started_cid"]
        or started["source_root"] != body["historical"]["root"]
    ):
        raise ValueError("diagnostic policy or preparation marker changed")
    current = _bindings(Path(body["historical"]["root"]))
    if any(body[key] != value for key, value in current.items()):
        raise ValueError("diagnostic source/input/implementation binding changed")
    return body
