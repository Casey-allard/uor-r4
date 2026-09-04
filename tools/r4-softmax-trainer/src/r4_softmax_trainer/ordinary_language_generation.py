"""Direct source-free generation with the fitted ordinary softmax control.

The command loads the final ordinary artifact from the matched #973 campaign
and recomputes its complete causal prefix for every emitted token. It does not
fit, select a checkpoint, load H4 geometry, or read corpus or teacher data.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import torch
from blake3 import blake3
from tokenizers import Tokenizer

from .language_path_generalization import (
    CONTEXT,
    HEADS,
    LAYERS,
    ORDINARY_POLICY,
    PARAMETER_COUNT,
    VOCAB_SIZE,
    OrdinaryCausalSoftmaxLanguagePathV1,
    work_ledger,
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
from .provenance import canonical_json_bytes, cid_bytes


SCHEMA = "uor-r4.ordinary-language-generation/1"
STATUS = "GENERATED"
ARM_RESULT_SCHEMA = "uor-r4.retained-language-path-arm-result/1"
DEFAULT_MAX_NEW_TOKENS = 16
DEFAULT_SEED = 9_738
TOKENIZER_RELATIVE_PATH = Path("tokenizer/tokenizer.json")
ARTIFACT_RELATIVE_PATH = Path("arms/ordinary/model.safetensors")
RESULT_RELATIVE_PATH = Path("arms/ordinary/result.json")
EXPECTED_ARTIFACT_CID = (
    "blake3:c1cd34b36c7df7c53915785a608ccd353a11de56eebb3ecc58e74092cb5d1933"
)
EXPECTED_ARM_RESULT_CID = (
    "blake3:19e6cd17a704c866afde13d413c8a20af88a969d04d8c1478be61eb5befcc59c"
)
EXPECTED_TOKENIZER_CID = (
    "blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc"
)


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


def _verify_arm_result(result: dict[str, Any], *, artifact_payload: bytes) -> None:
    unsigned = dict(result)
    expected_result_cid = unsigned.pop("arm_result_cid", None)
    if (
        expected_result_cid != EXPECTED_ARM_RESULT_CID
        or expected_result_cid != cid_bytes(canonical_json_bytes(unsigned))
    ):
        raise ValueError("ordinary arm result CID does not reproduce")

    artifact = result.get("artifact")
    expected_artifact = {
        "bytes": len(artifact_payload),
        "cid": EXPECTED_ARTIFACT_CID,
        "path": str(ARTIFACT_RELATIVE_PATH),
    }
    replay = result.get("replay")
    final_validation = result.get("final_validation")
    if (
        result.get("schema") != ARM_RESULT_SCHEMA
        or result.get("status") != "COMPLETE"
        or result.get("issue") != 973
        or result.get("arm") != "ordinary"
        or artifact != expected_artifact
        or result.get("forbidden_reads") != 0
        or not isinstance(replay, dict)
        or replay.get("passed") is not True
        or replay.get("artifact_reload_maximum_logits_delta") != 0.0
        or not isinstance(final_validation, dict)
        or final_validation.get("forbidden_reads") != 0
    ):
        raise ValueError("ordinary arm result does not bind the completed artifact")


def generate_ordinary_language_path(
    root: Path,
    *,
    prompt: str,
    max_new_tokens: int = DEFAULT_MAX_NEW_TOKENS,
    seed: int = DEFAULT_SEED,
) -> dict[str, Any]:
    """Generate one continuation through the fitted ordinary control."""

    if not isinstance(prompt, str) or not prompt:
        raise ValueError("prompt must be a nonempty string")
    if isinstance(max_new_tokens, bool) or not isinstance(max_new_tokens, int):
        raise TypeError("max_new_tokens must be an integer")
    if max_new_tokens < 1:
        raise ValueError("max_new_tokens must be positive")

    resolved_root = root.expanduser().resolve()
    tokenizer_path = resolved_root / TOKENIZER_RELATIVE_PATH
    artifact_path = resolved_root / ARTIFACT_RELATIVE_PATH
    result_path = resolved_root / RESULT_RELATIVE_PATH
    tokenizer_payload = tokenizer_path.read_bytes()
    artifact_payload = artifact_path.read_bytes()
    result_payload = result_path.read_bytes()
    if (
        _record(tokenizer_path, tokenizer_payload)["cid"]
        != EXPECTED_TOKENIZER_CID
    ):
        raise ValueError("language-path tokenizer differs from the fitted control")
    if _record(artifact_path, artifact_payload)["cid"] != EXPECTED_ARTIFACT_CID:
        raise ValueError("ordinary artifact differs from the completed control")
    arm_result = json.loads(result_payload)
    if not isinstance(arm_result, dict):
        raise ValueError("ordinary arm result must be a JSON object")
    _verify_arm_result(arm_result, artifact_payload=artifact_payload)

    tokenizer = Tokenizer.from_file(str(tokenizer_path))
    if tokenizer.get_vocab_size(with_added_tokens=True) != VOCAB_SIZE:
        raise ValueError("language-path tokenizer vocabulary differs")
    raw_decoder = ByteLevelRawDecoder.from_tokenizer_json(tokenizer_path)
    prompt_token_ids = list(tokenizer.encode(prompt, add_special_tokens=False).ids)
    if not prompt_token_ids:
        raise ValueError("prompt produced no tokenizer IDs")
    if any(token < 0 or token >= VOCAB_SIZE for token in prompt_token_ids):
        raise ValueError("prompt produced an out-of-vocabulary token ID")
    if raw_decoder.decode_bytes(prompt_token_ids) != prompt.encode("utf-8"):
        raise ValueError("prompt does not round-trip through the language tokenizer")

    input_token_ids = [BOS_TOKEN_ID, *prompt_token_ids]
    processed_ceiling = len(input_token_ids) + max_new_tokens - 1
    if processed_ceiling > CONTEXT:
        raise ValueError(
            "prompt plus generated prefix exceeds the model's 120-token context"
        )

    model = OrdinaryCausalSoftmaxLanguagePathV1()
    model.load_learned_artifact(artifact_payload)
    model.to(device=torch.device("cpu"), dtype=torch.float32)
    model.eval()
    torch.use_deterministic_algorithms(True)

    sampler = SplitMix64(seed)
    generated: list[int] = []
    prefix_lengths: list[int] = []
    work = {
        "token_steps": 0,
        "materialized_attention_scores": 0,
        "admitted_attention_scores": 0,
        "attention_value_reads": 0,
        "vocabulary_scores": 0,
    }
    stop: str | dict[str, Any] = "maximum_new_tokens"

    with torch.inference_mode():
        for decision in range(max_new_tokens):
            prefix = [*input_token_ids, *generated]
            output = model(
                torch.tensor([prefix], dtype=torch.long), attention_off=False
            )
            audit = output.audit
            expected_work = work_ledger(
                "ordinary", batch_size=1, time=len(prefix)
            )
            if (
                output.loss is not None
                or audit.batch_size != 1
                or audit.layers != LAYERS
                or audit.heads != HEADS
                or audit.work_signature()
                != (
                    1,
                    expected_work.token_steps,
                    LAYERS,
                    HEADS,
                    expected_work.materialized_attention_scores,
                    expected_work.admitted_attention_scores,
                    expected_work.attention_value_reads,
                    expected_work.vocabulary_scores,
                    0,
                )
                or audit.attention_off
            ):
                raise RuntimeError("generation left the ordinary causal execution path")
            logits = (
                output.logits[0, -1]
                .detach()
                .cpu()
                .to(dtype=torch.float32)
                .contiguous()
            )
            if logits.shape != (VOCAB_SIZE,) or not bool(
                torch.isfinite(logits).all()
            ):
                raise RuntimeError("ordinary causal path produced invalid next-token logits")

            prefix_lengths.append(len(prefix))
            for field in work:
                work[field] += int(getattr(audit, field))
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

    response_ids = (
        generated[:-1]
        if generated and generated[-1] == EOS_TOKEN_ID
        else generated
    )
    continuation, utf8_decodable = _decode_utf8(
        raw_decoder.decode_bytes(response_ids)
    )

    return {
        "schema": SCHEMA,
        "status": STATUS,
        "model": {
            "policy": ORDINARY_POLICY,
            "parameter_count": PARAMETER_COUNT,
            "context_tokens": CONTEXT,
        },
        "inputs": {
            "model_artifact": _record(artifact_path, artifact_payload),
            "artifact_result": _record(result_path, result_payload),
            "tokenizer": _record(tokenizer_path, tokenizer_payload),
            "corpus_files_read": 0,
            "teacher_files_read": 0,
            "geometry_files_read": 0,
            "checkpoint_files_read": 0,
        },
        "fit_provenance": {
            "arm_result_cid": arm_result["arm_result_cid"],
            "completed_steps": arm_result.get("completed_steps"),
            "presentations": arm_result.get("presentations"),
            "train_order_cid": arm_result.get("train_order_cid"),
            "final_validation": arm_result.get("final_validation"),
        },
        "execution": {
            "implementation": "full-prefix-recomputation",
            "forward_calls": len(prefix_lengths),
            "maximum_prefix_tokens": max(prefix_lengths),
            "evaluated_token_positions": sum(prefix_lengths),
            "attention_off": False,
            "forbidden_reads": 0,
            "work": work,
        },
        "prompt": prompt,
        "prompt_token_ids": prompt_token_ids,
        "seed": seed,
        "sampler": {
            "type": "top-k-q32-splitmix64",
            "top_k": TOP_K,
            "temperature": TEMPERATURE,
        },
        "max_new_tokens": max_new_tokens,
        "generated_token_ids": generated,
        "continuation": continuation,
        "text": prompt + continuation,
        "utf8_decodable": utf8_decodable,
        "stop": stop,
    }


__all__ = [
    "DEFAULT_MAX_NEW_TOKENS",
    "DEFAULT_SEED",
    "SCHEMA",
    "STATUS",
    "generate_ordinary_language_path",
]
