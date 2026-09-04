"""Direct source-free generation with the contextual retained value write.

The command loads the already fitted compact retained artifact and changes only
the recurrent value source used by the versioned contextual-write model.  It
does not train, select a checkpoint, or read corpus data.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

import torch
from blake3 import blake3
from tokenizers import Tokenizer

from .group_retention_campaign import load_group_geometry_artifacts
from .language_path_generalization import (
    CONTEXT,
    VOCAB_SIZE,
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
POLICY = "R4ContextualValueWriteLanguagePathV1"
DEFAULT_MAX_NEW_TOKENS = 32
DEFAULT_SEED = 9_738
TOKENIZER_RELATIVE_PATH = Path("tokenizer/tokenizer.json")
ARTIFACT_RELATIVE_PATH = Path("arms/retained/model.safetensors")


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
    artifact_path = resolved_root / ARTIFACT_RELATIVE_PATH
    tokenizer_payload = tokenizer_path.read_bytes()
    artifact_payload = artifact_path.read_bytes()
    geometry_payload = resolved_geometry.read_bytes()

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
    model = R4ContextualValueWriteLanguagePathV1(geometry)
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
    return {
        "schema": SCHEMA,
        "status": "GENERATED",
        "model": {
            "policy": POLICY,
            "value_write": "Wv(RMSNorm(x_t + strict_prior_retained_residual))",
            "parameters_added": 0,
            "fitted_under_value_write": "Wv(RMSNorm(x_t))",
        },
        "inputs": {
            "model_artifact": _record(artifact_path, artifact_payload),
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
