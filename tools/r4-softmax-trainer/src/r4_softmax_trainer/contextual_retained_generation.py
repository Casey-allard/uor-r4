"""Direct source-free generation with the contextual retained value write.

The command loads the already fitted compact retained artifact and changes only
the recurrent value source used by the versioned contextual-write model.  It
does not train, select a checkpoint, or read corpus data.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import torch
from blake3 import blake3
from tokenizers import Tokenizer

from .contextual_retained_fit import (
    CONTEXTUAL_KEY_VALUE_SCHEMA,
    CONTEXTUAL_KEY_VALUE_STATUS,
    EXPECTED_GEOMETRY_CID,
    EXPECTED_INITIAL_ARTIFACT_CID,
    FULL_EPOCH_STATUS,
    SCHEMA as CONTEXTUAL_FIT_SCHEMA,
    STATUS as CONTEXTUAL_FIT_STATUS,
)
from .group_retention_campaign import load_group_geometry_artifacts
from .language_path_generalization import (
    CONTEXTUAL_KEY_VALUE_WRITE_POLICY,
    CONTEXTUAL_VALUE_WRITE_POLICY,
    CONTEXT,
    VOCAB_SIZE,
    R4ContextualKeyValueWriteLanguagePathV1,
    R4ContextualValueWriteLanguagePathV1,
)
from .language_path_generation import (
    BOS_TOKEN_ID,
    EOS_TOKEN_ID,
    TEMPERATURE,
    TOP_K,
    ByteLevelRawDecoder,
    SplitMix64,
    sample_top_k_q32,
    short_cycle_period,
)


SCHEMA = "uor-r4.contextual-retained-generation/1"
POLICY = CONTEXTUAL_VALUE_WRITE_POLICY
DEFAULT_MAX_NEW_TOKENS = 32
DEFAULT_SEED = 9_738
TOKENIZER_RELATIVE_PATH = Path("tokenizer/tokenizer.json")
ARTIFACT_RELATIVE_PATH = Path("arms/retained/model.safetensors")
CONTEXTUAL_KEY_WRITE = "Wk(RMSNorm(x_t + strict_prior_retained_residual))"
CONTEXTUAL_VALUE_WRITE = "Wv(RMSNorm(x_t + strict_prior_retained_residual))"
TOKEN_LOCAL_VALUE_WRITE = "Wv(RMSNorm(x_t))"


def _record(path: Path, payload: bytes) -> dict[str, Any]:
    return {
        "path": str(path),
        "bytes": len(payload),
        "cid": f"blake3:{blake3(payload).hexdigest()}",
    }


def _decode_utf8(payload: bytes) -> tuple[str, bool]:
    try:
        return payload.decode("utf-8"), True
    except UnicodeDecodeError:
        return payload.decode("utf-8", errors="replace"), False


def generate_contextual_retained(
    root: Path,
    *,
    geometry_path: Path,
    prompt: str,
    artifact_path: Path | None = None,
    max_new_tokens: int = DEFAULT_MAX_NEW_TOKENS,
    seed: int = DEFAULT_SEED,
) -> dict[str, Any]:
    """Generate one continuation through the contextual-write direct cell."""

    if not isinstance(prompt, str) or not prompt:
        raise ValueError("prompt must be a nonempty string")
    if isinstance(max_new_tokens, bool) or not isinstance(max_new_tokens, int):
        raise TypeError("max_new_tokens must be an integer")
    if max_new_tokens < 1:
        raise ValueError("max_new_tokens must be positive")

    resolved_root = root.expanduser().resolve()
    resolved_geometry = geometry_path.expanduser().resolve()
    tokenizer_path = resolved_root / TOKENIZER_RELATIVE_PATH
    resolved_artifact = (
        resolved_root / ARTIFACT_RELATIVE_PATH
        if artifact_path is None
        else artifact_path.expanduser().resolve()
    )
    tokenizer_payload = tokenizer_path.read_bytes()
    artifact_payload = resolved_artifact.read_bytes()
    geometry_payload = resolved_geometry.read_bytes()
    artifact_cid = f"blake3:{blake3(artifact_payload).hexdigest()}"
    geometry_cid = f"blake3:{blake3(geometry_payload).hexdigest()}"
    fit_result_path = resolved_artifact.with_name("fit.json")
    fit_summary: dict[str, Any] | None = None
    selected_policy = POLICY
    model_type = R4ContextualValueWriteLanguagePathV1
    fitted_under_key_write: str | None = None
    fitted_under_value_write = TOKEN_LOCAL_VALUE_WRITE
    if fit_result_path.is_file():
        fit_result = json.loads(fit_result_path.read_text(encoding="utf-8"))
        if fit_result.get("artifact", {}).get("cid") != artifact_cid:
            raise ValueError("contextual fit metadata does not match the selected artifact")
        if fit_result.get("inputs", {}).get("geometry", {}).get("cid") != geometry_cid:
            raise ValueError("contextual fit metadata names a different geometry")
        fit_model = fit_result.get("model", {})
        fit_policy = fit_model.get("policy")
        if fit_policy == POLICY:
            if (
                fit_result.get("schema") != CONTEXTUAL_FIT_SCHEMA
                or fit_result.get("status")
                not in {CONTEXTUAL_FIT_STATUS, FULL_EPOCH_STATUS}
            ):
                raise ValueError(
                    "contextual fit metadata has an unsupported schema or status"
                )
            if fit_model.get("value_write") != CONTEXTUAL_VALUE_WRITE:
                raise ValueError("contextual fit metadata names a different value write")
        elif fit_policy == CONTEXTUAL_KEY_VALUE_WRITE_POLICY:
            if (
                fit_result.get("schema") != CONTEXTUAL_KEY_VALUE_SCHEMA
                or fit_result.get("status") != CONTEXTUAL_KEY_VALUE_STATUS
            ):
                raise ValueError(
                    "contextual key/value fit metadata has an unsupported schema or status"
                )
            if fit_model.get("key_write") != CONTEXTUAL_KEY_WRITE:
                raise ValueError("contextual fit metadata names a different key write")
            if fit_model.get("value_write") != CONTEXTUAL_VALUE_WRITE:
                raise ValueError("contextual fit metadata names a different value write")
            selected_policy = CONTEXTUAL_KEY_VALUE_WRITE_POLICY
            model_type = R4ContextualKeyValueWriteLanguagePathV1
            fitted_under_key_write = fit_model["key_write"]
        else:
            raise ValueError("contextual fit metadata names an unsupported model policy")
        fitted_under_value_write = fit_model["value_write"]
        fit_summary = {
            "updates": fit_result.get("fit", {}).get("updates"),
            "causal_targets": fit_result.get("fit", {}).get("causal_targets"),
            "elapsed_seconds": fit_result.get("fit", {}).get("elapsed_seconds"),
            "result_path": str(fit_result_path),
        }
    else:
        canonical_artifact = (resolved_root / ARTIFACT_RELATIVE_PATH).resolve()
        if resolved_artifact != canonical_artifact:
            raise ValueError("a noncanonical retained artifact requires adjacent fit metadata")
        if artifact_cid != EXPECTED_INITIAL_ARTIFACT_CID:
            raise ValueError("canonical retained artifact differs from the pinned V1 artifact")
        if geometry_cid != EXPECTED_GEOMETRY_CID:
            raise ValueError("canonical retained generation requires the pinned geometry")

    tokenizer = Tokenizer.from_file(str(tokenizer_path))
    raw_decoder = ByteLevelRawDecoder.from_tokenizer_json(tokenizer_path)
    prompt_token_ids = list(tokenizer.encode(prompt, add_special_tokens=False).ids)
    if not prompt_token_ids:
        raise ValueError("prompt produced no tokenizer IDs")
    if any(token < 0 or token >= VOCAB_SIZE for token in prompt_token_ids):
        raise ValueError("prompt produced an out-of-vocabulary token ID")
    if raw_decoder.decode_bytes(prompt_token_ids) != prompt.encode("utf-8"):
        raise ValueError("prompt does not round-trip through the retained tokenizer")

    input_token_ids = [BOS_TOKEN_ID, *prompt_token_ids]
    processed_ceiling = len(input_token_ids) + max_new_tokens - 1
    if processed_ceiling > CONTEXT:
        raise ValueError(
            "prompt plus generated prefix exceeds the model's 120-token context"
        )

    geometry = load_group_geometry_artifacts(resolved_geometry).exact_h4
    model = model_type(geometry)
    model.load_learned_artifact(artifact_payload)
    model.to(device=torch.device("cpu"), dtype=torch.float32)
    model.eval()
    torch.use_deterministic_algorithms(True)

    state = model.initial_state(1, device=torch.device("cpu"), dtype=torch.float32)
    sampler = SplitMix64(seed)
    generated: list[int] = []
    logits: torch.Tensor | None = None
    processed_tokens = 0
    stop: str | dict[str, Any] = "maximum_new_tokens"

    with torch.inference_mode():
        for token in input_token_ids:
            step = model.step(
                torch.tensor([token], dtype=torch.long),
                state,
                attention_off=False,
            )
            if (
                step.audit.implementation != "direct"
                or step.audit.state_off
                or step.audit.forbidden_reads != 0
            ):
                raise RuntimeError("generation left the direct retained execution path")
            state = step.final_state
            logits = step.logits[0].detach().cpu().to(dtype=torch.float32).contiguous()
            processed_tokens += 1

        if logits is None:
            raise RuntimeError("prompt produced no next-token logits")
        for decision in range(max_new_tokens):
            token = int(sample_top_k_q32(logits, sampler))
            if not 0 <= token < VOCAB_SIZE:
                raise RuntimeError("sampler selected a token outside the vocabulary")
            generated.append(token)
            if token == EOS_TOKEN_ID:
                stop = "eos"
                break
            period = short_cycle_period(generated)
            if period is not None:
                stop = {"short_cycle": {"period": period}}
                break
            if decision + 1 == max_new_tokens:
                break
            step = model.step(
                torch.tensor([token], dtype=torch.long),
                state,
                attention_off=False,
            )
            if (
                step.audit.implementation != "direct"
                or step.audit.state_off
                or step.audit.forbidden_reads != 0
            ):
                raise RuntimeError("generation left the direct retained execution path")
            state = step.final_state
            logits = step.logits[0].detach().cpu().to(dtype=torch.float32).contiguous()
            processed_tokens += 1

    response_ids = generated[:-1] if generated and generated[-1] == EOS_TOKEN_ID else generated
    continuation, utf8_decodable = _decode_utf8(raw_decoder.decode_bytes(response_ids))
    model_report: dict[str, Any] = {
        "policy": selected_policy,
        "value_write": CONTEXTUAL_VALUE_WRITE,
        "parameters_added": 0,
        "fitted_under_value_write": fitted_under_value_write,
        "fit": fit_summary,
    }
    if fitted_under_key_write is not None:
        model_report["key_write"] = CONTEXTUAL_KEY_WRITE
        model_report["fitted_under_key_write"] = fitted_under_key_write

    return {
        "schema": SCHEMA,
        "status": "GENERATED",
        "model": model_report,
        "inputs": {
            "model_artifact": _record(resolved_artifact, artifact_payload),
            "tokenizer": _record(tokenizer_path, tokenizer_payload),
            "geometry": _record(resolved_geometry, geometry_payload),
            "corpus_files_read": 0,
            "teacher_files_read": 0,
        },
        "prompt": prompt,
        "prompt_token_ids": prompt_token_ids,
        "seed": seed,
        "sampler": {"type": "top-k-q32-splitmix64", "top_k": TOP_K, "temperature": TEMPERATURE},
        "max_new_tokens": max_new_tokens,
        "generated_token_ids": generated,
        "continuation": continuation,
        "text": prompt + continuation,
        "utf8_decodable": utf8_decodable,
        "stop": stop,
        "processed_tokens": processed_tokens,
    }


__all__ = [
    "DEFAULT_MAX_NEW_TOKENS",
    "DEFAULT_SEED",
    "POLICY",
    "SCHEMA",
    "generate_contextual_retained",
]
