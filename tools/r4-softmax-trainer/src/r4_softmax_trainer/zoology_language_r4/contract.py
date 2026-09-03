"""Immutable source, frame, opportunity and phase bindings for inference #1079."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import torch

from ..provenance import artifact_records, canonical_json_bytes, cid_bytes, tree_cid
from ..zoology_language_interface import contract as previous
from ..zoology_language_interface import data
from ..zoology_language_interface.model import MODEL_CONFIG, MODEL_POLICY
from ..zoology_r4_inference import contract as prior
from ..zoology_r4_inference.frames import load_frames

ISSUE = 1079
POLICY = "FrozenLearnedLanguageR4InferenceV1"
SCHEMA = "uor-r4.zoology-language-r4-preparation/1"
SOURCE_FILE_COUNT = 295
SOURCE_CIDS = {
    "preparation": "blake3:0395b826049dbeed351a647960c7b66cc4d65fc19b65eb3c522fcdd807aaad69",
    "fit": "blake3:7c5a46f0b044ee3a9da3aa2126b4fb31f3088e239a5fd6b9f4e276334a97770d",
    "result": "blake3:294fe7f488237c196525a3470f48c3b55f5a14232e23c3243b88f579da85e1c1",
    "replay": "blake3:8e5b1f11be99835ec3dddd197357da462353e6bb9c7f5b8f603e1db766d0770f",
}
CONTROLS = ("token_source_frame_permuted", "fact_source_frame_permuted")
EVALUATION = {
    "batch_size": 256,
    "threads": 4,
    "interop_threads": 1,
    "max_elapsed_seconds": 900,
    "max_rss_bytes": 4 * 1024**3,
    "logit_atol": 0.005,
    "attention_atol": 1e-5,
    "role_vector_atol": 1e-5,
    "nll_atol": 1e-5,
    "role_attention_exact": True,
    "role_predictions_exact": True,
    "strong_control_drop": 0.5,
    "construction_views": [0, 1],
    "development_views": [0, 1, 2, 3],
    "construction_rows_per_view": 10240,
    "development_rows_per_view": 1280,
    "decision_rows_per_arm": 25600,
    "ordinary_reproduction": "exact #1077 all/supported/unknown records, role attention/positions, groups, syntax and work before R4",
}
INTERVENTION = {
    "source_issue": 1077,
    "reference": "frozen learned ordinary soft-role and binding path; canonical hard-field oracle is not the preservation reference",
    "frame_assignment": "native cumulative fold across all five valid clauses without reset; padding omitted; each pooling destination is its clause end",
    "role_transport": "all valid token embeddings, all fifteen role mixtures, all sixteen four-lane blocks; original reader probabilities unchanged",
    "binding_transport": "four learned fact K/V entries plus identity-frame null into the final question frame; full softmax mixture and unchanged head",
    "token_control": "true token encoding; source frame at (i+1) modulo valid clause length only in token transport",
    "fact_control": "true fact encoding; source end frames [1,2,3,0] only in fact transport; null fixed",
    "controls": list(CONTROLS),
    "control_admission": "all six ordinary and coherent primary views pass; each control is isolated and has equal work",
    "weak_control": "retain preservation separately; no strong sensitivity attribution for an invalid or weak arm",
    "optimizer_updates": 0,
    "new_parameters": 0,
    "new_population_generation": 0,
    "native_exports": 0,
    "geometry_changes": 0,
    "model_label_arguments": 0,
    "checkpoint_optimizer_rng_payload_reads": 0,
    "scope": "preservation on observed #1077 renderings; supplied boundaries, known lexicon and observed worlds; no geometry advantage or new generalization claim",
}


def _repo() -> Path:
    return Path(__file__).resolve().parents[5]


def _source_contract(root: Path) -> dict[str, Any]:
    paths = {
        name: f"docs/r4_zoology_language_interface_1077_{name}.json"
        for name in SOURCE_CIDS
    }
    documents = {}
    for name, path in paths.items():
        public = prior._envelope(_repo() / path, f"{name}_cid", SOURCE_CIDS[name])
        if (
            prior._envelope(root / f"{name}.json", f"{name}_cid", SOURCE_CIDS[name])
            != public
        ):
            raise ValueError("retained #1077 envelope differs from publication")
        documents[name] = public
    preparation, fitted, result, replay = (documents[name] for name in SOURCE_CIDS)
    # Do not call the old validate_preparation: its worktree root is historical.
    # Reproduce content in this checkout and bind the retained core through its
    # existing lineage helper, without rerunning an older frame campaign.
    lineage = previous._lineage(
        Path(preparation["source"]["root"]), Path(preparation["prior"]["root"])
    )
    if any(
        {k: v for k, v in lineage[name].items() if k != "root"}
        != {k: v for k, v in preparation[name].items() if k != "root"}
        for name in ("source", "prior")
    ):
        raise ValueError("retained core lineage differs from #1077")
    core, reader, evidence = lineage["source"], fitted["artifact"], result["evidence"]
    if (
        preparation["issue"] != 1077
        or preparation["model_config"] != MODEL_CONFIG
        or preparation["model_policy"] != MODEL_POLICY
        or preparation["data_policy"] != data.DATA_POLICY
        or preparation["training"] != previous.TRAINING
        or fitted["status"] != "FIT_COMPLETE"
        or fitted["optimizer_updates"] != 512
        or fitted["row_presentations"] != 65536
        or fitted["role_label_presentations"] != 917504
        or fitted["core_optimizer_updates"] != 0
        or fitted["development_tensor_reads"] != 0
        or fitted["core_file_cid"] != core["model"]["cid"]
        or fitted["core_state_cid"] != core["model"]["state_cid"]
        or reader["path"] != "fit/reader.safetensors"
        or reader["bytes"] != 566692
        or reader["parameter_count"] != 141571
        or result["reader"] != reader
        or result["core"] != core["model"]
        or result["fit_cid"] != fitted["fit_cid"]
        or result["runtime"] != fitted["runtime"]
        or result["runtime"]["threads"] != EVALUATION["threads"]
        or result["runtime"]["interop_threads"] != EVALUATION["interop_threads"]
        or evidence["status"] != "LANGUAGE_INTERFACE_HELDOUT_PASSED"
        or not evidence["passed"]
        or evidence["reader_state_before"] != reader["state_cid"]
        or evidence["reader_state_after"] != reader["state_cid"]
        or evidence["core_state_before"] != core["model"]["state_cid"]
        or evidence["core_state_after"] != core["model"]["state_cid"]
        or evidence["evaluation_optimizer_updates"] != 0
        or evidence["core_optimizer_updates"] != 0
        or evidence["r4_forwards"] != 0
        or result["evidence_cid"] != cid_bytes(canonical_json_bytes(evidence))
        or not replay["exact_replay"]
        or not replay["fresh_process"]
        or replay["optimizer_updates"] != 0
        or replay["result_cid"] != result["result_cid"]
        or replay["process_id"] == result["process_id"]
        or any(
            document["issue"] != 1077
            or document["preparation_cid"] != preparation["preparation_cid"]
            or document["implementation_cid"]
            != preparation["implementation"]["tree_cid"]
            for document in (fitted, result, replay)
        )
        or any(
            replay[key] != result[key]
            for key in ("fit_cid", "reader", "core", "runtime", "evidence_cid")
        )
    ):
        raise ValueError(
            "qualified #1077 reader/core/result/replay relationship differs"
        )
    for population, expected in (
        ("construction", [0, 1]),
        ("development", [0, 1, 2, 3]),
    ):
        if [view["view_id"] for view in evidence[population]] != expected or not all(
            view["qualification"]["passed"] for view in evidence[population]
        ):
            raise ValueError("qualified #1077 views are incomplete")
    files = artifact_records(
        _repo(), [row["path"] for row in preparation["implementation"]["files"]]
    )
    if (
        len(files) != SOURCE_FILE_COUNT
        or files != preparation["implementation"]["files"]
        or tree_cid(files) != preparation["implementation"]["tree_cid"]
    ):
        raise ValueError("historical #1077 implementation changed")
    actual = prior._record(root, reader["path"], cid=reader["cid"])
    if any(reader[key] != value for key, value in actual.items()):
        raise ValueError("retained reader bytes changed")
    dataset = data.validate(root / "data", inspect_development=False)
    if dataset != preparation["dataset"]:
        raise ValueError("retained observed renderings changed")
    return {
        "root": str(root),
        **{f"{name}_cid": SOURCE_CIDS[name] for name in SOURCE_CIDS},
        "reader": reader,
        "core": core,
        "runtime": result["runtime"],
        "dataset": dataset,
        "baseline_history": evidence,
        "evidence_cid": result["evidence_cid"],
        "frame_tree_cid": lineage["prior"]["frame_tree_cid"],
        "documents": artifact_records(root, [f"{name}.json" for name in SOURCE_CIDS]),
        "public_documents": artifact_records(_repo(), paths.values()),
        "implementation_files": files,
    }


def load_source_model(preparation: dict):
    """Load each bound frozen artifact once, only when the evaluator requests it."""
    from ..zoology_compound_binding.model import load_model
    from ..zoology_language_interface.campaign import _load_fit
    from ..zoology_language_interface.model import LanguageInterfaceModel

    source = preparation["source"]
    root = Path(source["root"])
    original = prior._envelope(
        root / "preparation.json", "preparation_cid", source["preparation_cid"]
    )
    fitted, reader = _load_fit(root, original)
    if fitted["fit_cid"] != source["fit_cid"] or fitted["artifact"] != source["reader"]:
        raise ValueError("reader changed before loading")
    core = load_model({"source": source["core"]})
    return LanguageInterfaceModel(core, reader).eval().requires_grad_(False)


def _frames(root: Path, source: dict) -> dict:
    frames = prior._frame_contract(root)
    if (
        frames["tree_cid"] != source["frame_tree_cid"]
        or frames["tree_cid"] != previous.FRAME_TREE_CID
    ):
        raise ValueError("native frame bundle differs from preserved source")
    return frames


def _opportunity(
    changed: torch.Tensor, shifted: torch.Tensor, supported: torch.Tensor
) -> dict:
    rows_changed = changed.flatten(1).any(1)
    count = int(supported.sum())
    if not count:
        raise ValueError("frame opportunity requires supported rows")
    eligible = int(rows_changed[supported].sum())
    return {
        "source_frame_positions_changed": int(shifted.sum()),
        "source_frame_matrices_changed": int(changed.sum()),
        "rows_with_changed_source_frame": int(rows_changed.sum()),
        "supported_rows_with_changed_source_frame": eligible,
        "supported_loss_reachability_ceiling": eligible / count,
        "passed": eligible / count >= EVALUATION["strong_control_drop"],
    }


def _frame_view(inputs, lengths, supported, frames) -> dict:
    from .attention import frame_assignment

    tokens, ends = frame_assignment(inputs, lengths, frames)
    offsets = torch.arange(inputs.shape[-1]).reshape(1, 1, -1).expand_as(tokens)
    valid = offsets < lengths.unsqueeze(-1)
    next_offsets = (offsets + 1) % lengths.unsqueeze(-1)
    matrix_changes = (
        frames.frame_matrices[:, None] != frames.frame_matrices[None, :]
    ).any(dim=(-2, -1))
    token_changes = matrix_changes[tokens, tokens.gather(2, next_offsets)] & valid
    fact_changes = matrix_changes[ends[:, :4], ends[:, [1, 2, 3, 0]]]
    token_reached = tokens[valid].unique(sorted=True)
    clause_reached = ends.reshape(-1).unique(sorted=True)
    reached = torch.cat(
        (token_reached, clause_reached, torch.tensor([frames.identity_index]))
    ).unique(sorted=True)
    controls = {
        CONTROLS[0]: _opportunity(
            token_changes, (offsets != next_offsets) & valid, supported
        ),
        CONTROLS[1]: _opportunity(
            fact_changes, torch.ones_like(fact_changes), supported
        ),
    }
    return {
        "rows": len(inputs),
        "supported_rows": int(supported.sum()),
        "unknown_rows": len(inputs) - int(supported.sum()),
        "valid_tokens": int(valid.sum()),
        "reached_frame_indices": reached.tolist(),
        "future_token_reads": 0,
        "reached_token_frame_indices": token_reached.tolist(),
        "reached_clause_frame_indices": clause_reached.tolist(),
        "controls": controls,
        "passed": all(control["passed"] for control in controls.values()),
    }


def structural_preflight(source: dict, frame_info: dict) -> dict:
    frames = load_frames(Path(frame_info["root"]))
    views = []
    for population, loader, identifiers in (
        ("construction", data.load_construction, [0, 1]),
        ("development", data.load_development, [0, 1, 2, 3]),
    ):
        tensors = loader(Path(source["root"]) / "data")
        for identifier in identifiers:
            mask = tensors["view_ids"] == identifier
            inputs, lengths = tensors["inputs"][mask], tensors["lengths"][mask]
            if len(inputs) != EVALUATION[f"{population}_rows_per_view"]:
                raise ValueError("observed frame population is incomplete")
            views.append(
                {
                    "population": population,
                    "view_id": identifier,
                    **_frame_view(
                        inputs, lengths, tensors["variant_ids"][mask] < 4, frames
                    ),
                }
            )
    return {
        "passed": all(
            control["passed"] for view in views for control in view["controls"].values()
        ),
        "views": views,
        "frame_count": len(frames.frame_matrices),
        "frame_map_tokens": frames.token_leaf_indices.numel(),
        "decision_rows_per_arm": sum(view["rows"] for view in views),
        "model_forwards": 0,
        "new_populations": 0,
        "scope": "label-free matrix-change opportunity on observed inputs; an upper bound, not a predicted loss",
    }


def _bindings(source_root: Path, frame_root: Path) -> dict:
    source = _source_contract(source_root)
    frames = _frames(frame_root, source)
    preflight = structural_preflight(source, frames)
    if not preflight["passed"]:
        raise ValueError("one control lacks the declared frame-change opportunity")
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
            "test_zoology_language_r4*.py"
        )
    )
    files = artifact_records(_repo(), sorted(paths))
    return {
        "source": source,
        "frames": frames,
        "preflight": preflight,
        "implementation": {
            "root": str(_repo()),
            "files": files,
            "tree_cid": tree_cid(files),
        },
    }


def prepare(root: Path, source_root: Path, frame_root: Path) -> dict:
    root, source_root, frame_root = (
        path.resolve() for path in (root, source_root, frame_root)
    )
    root.mkdir(parents=True, exist_ok=True)
    if (root / "preparation.json").exists():
        raise FileExistsError("language R4 preparation already exists")
    start = previous._exclusive(
        root / "preparation-started.json",
        {
            "issue": ISSUE,
            "source_root": str(source_root),
            "frame_root": str(frame_root),
        },
        "started_cid",
    )
    body = {
        "schema": SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "evaluation": EVALUATION,
        "intervention": INTERVENTION,
        "preparation_started_cid": start["started_cid"],
        **_bindings(source_root, frame_root),
    }
    return previous._exclusive(root / "preparation.json", body, "preparation_cid")


def validate_preparation(root: Path) -> dict:
    root = root.resolve()
    body = prior._envelope(root / "preparation.json", "preparation_cid")
    start = prior._envelope(root / "preparation-started.json", "started_cid")
    if (
        any(
            body.get(key) != value
            for key, value in {
                "schema": SCHEMA,
                "issue": ISSUE,
                "policy": POLICY,
                "evaluation": EVALUATION,
                "intervention": INTERVENTION,
            }.items()
        )
        or start["issue"] != ISSUE
        or start["started_cid"] != body["preparation_started_cid"]
        or start["source_root"] != body["source"]["root"]
        or start["frame_root"] != body["frames"]["root"]
    ):
        raise ValueError("frozen language R4 policy or preparation phase differs")
    current = _bindings(Path(body["source"]["root"]), Path(body["frames"]["root"]))
    if any(body.get(key) != value for key, value in current.items()):
        raise ValueError("language R4 source/frame/implementation/preflight changed")
    return body
