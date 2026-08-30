"""Standard Hugging Face export for the repository's existing Rust loaders."""

from __future__ import annotations

import os
import shutil
from pathlib import Path
from typing import Any

from safetensors.torch import save_file

from .constants import EXPORT_MANIFEST_SCHEMA, FROZEN_MODEL_CONFIG
from .model import R4SoftmaxForCausalLM, export_state_dict
from .provenance import atomic_write_json, cid_file, write_bound_manifest


def export_hugging_face_snapshot(
    model: R4SoftmaxForCausalLM,
    *,
    output_dir: Path,
    tokenizer_path: Path,
    training_result: dict[str, Any],
    dataset_manifest_cid: str,
    training_view_manifest_cid: str,
    split_policy_cid: str,
    run_contract_cid: str,
    selected_checkpoint_cid: str | None,
) -> dict[str, Any]:
    """Export three standard files plus CID-bound provenance/result files."""
    output_dir.mkdir(parents=True, exist_ok=True)
    config_path = output_dir / "config.json"
    weights_path = output_dir / "model.safetensors"
    exported_tokenizer_path = output_dir / "tokenizer.json"
    result_path = output_dir / "training-result.json"

    atomic_write_json(config_path, FROZEN_MODEL_CONFIG.as_hugging_face_config())
    tokenizer_temporary = output_dir / ".tokenizer.json.part"
    shutil.copyfile(tokenizer_path, tokenizer_temporary)
    os.replace(tokenizer_temporary, exported_tokenizer_path)
    weights_temporary = output_dir / ".model.safetensors.part"
    save_file(export_state_dict(model), str(weights_temporary), metadata={"format": "pt"})
    os.replace(weights_temporary, weights_path)
    atomic_write_json(result_path, training_result)

    weights_cid = cid_file(weights_path)
    payload: dict[str, Any] = {
        "schema": EXPORT_MANIFEST_SCHEMA,
        "dataset_manifest_cid": dataset_manifest_cid,
        "training_view_manifest_cid": training_view_manifest_cid,
        "split_policy_cid": split_policy_cid,
        "run_contract_cid": run_contract_cid,
        "selected_checkpoint_cid": selected_checkpoint_cid or weights_cid,
        "selected_checkpoint_identity": (
            "checkpoint artifact" if selected_checkpoint_cid else "exported smoke weights"
        ),
        "training_result_cid": training_result["result_cid"],
        "config_cid": cid_file(config_path),
        "tokenizer_cid": cid_file(exported_tokenizer_path),
        "weights_cid": weights_cid,
        "model_contract": FROZEN_MODEL_CONFIG.as_contract(),
        "loader_contract": {
            "weights": "HuggingFaceLlamaOracle",
            "tokenizer": "HfBpeTokenizer",
            "tied_lm_head": True,
            "source_dtype": "F32",
        },
    }
    return write_bound_manifest(
        output_dir / "export-manifest.json",
        payload,
        artifact_root=output_dir,
        relative_paths=[
            "config.json",
            "model.safetensors",
            "tokenizer.json",
            "training-result.json",
        ],
    )
