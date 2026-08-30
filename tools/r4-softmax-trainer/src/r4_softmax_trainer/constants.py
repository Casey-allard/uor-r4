"""Frozen #1014 model, data, and run contract."""

from __future__ import annotations

from dataclasses import asdict, dataclass


SCHEMA_PREFIX = "uor-r4-softmax-trainer"
TRAINER_SCHEMA = f"{SCHEMA_PREFIX}/1"
DATASET_MANIFEST_SCHEMA = f"{SCHEMA_PREFIX}-dataset/1"
TRAINING_VIEW_MANIFEST_SCHEMA = f"{SCHEMA_PREFIX}-training-view/1"
EXPORT_MANIFEST_SCHEMA = f"{SCHEMA_PREFIX}-export/1"
SELECTION_MANIFEST_SCHEMA = f"{SCHEMA_PREFIX}-selection/1"
CHECKPOINT_SCHEMA = f"{SCHEMA_PREFIX}-checkpoint/1"
SMOKE_SCHEMA = f"{SCHEMA_PREFIX}-overfit-smoke/1"
SMOKE_MANIFEST_SCHEMA = f"{SCHEMA_PREFIX}-smoke-manifest/1"
MAIN_ADMISSION_MANIFEST_SCHEMA = f"{SCHEMA_PREFIX}-main-admission/1"
RUST_QUALIFICATION_REPORT_SCHEMA = "uor-r4.r4-softmax-local-qualification/1"
PYTHON_PREFIX_LOGITS_SCHEMA = "uor-r4.r4-softmax-python-prefix-logits/1"
PREFIX_PARITY_TOKENS = 32
PREFIX_LOGIT_ABS_TOLERANCE = 0.005

LLAMA2_C_REPOSITORY = "https://github.com/karpathy/llama2.c"
LLAMA2_C_REVISION = "350e04fe35433e6d2941dce5a1f53308f87058eb"
TINYSTORIES_REPOSITORY = "roneneldan/TinyStories"
TINYSTORIES_REVISION = "f54c09fd23315a6f9c86f9dc80f725de7d8f9c64"
TINYSTORIES_FILENAME = "TinyStoriesV2-GPT4-train.txt"
TINYSTORIES_URL = (
    "https://huggingface.co/datasets/roneneldan/TinyStories/resolve/"
    f"{TINYSTORIES_REVISION}/{TINYSTORIES_FILENAME}"
)
TINYSTORIES_BYTES = 2_227_753_162
TINYSTORIES_SHA256 = "6418d412de72888f52b5142c761ac21a582f7d1166f0bfbdb5f03ccfdec90443"

BOS_TOKEN = "<|bos|>"
EOS_TOKEN = "<|eos|>"
UNK_TOKEN = "<|unk|>"
BOS_TOKEN_ID = 0
EOS_TOKEN_ID = 1
UNK_TOKEN_ID = 2
STORY_DELIMITER = b"<|endoftext|>"

SPLIT_BUCKETS = 100
TRAIN_BUCKETS = range(0, 90)
DEV_BUCKETS = range(90, 95)
TEST_BUCKETS = range(95, 100)

TOKENIZER_TRAIN_BYTES = 64 * 1024 * 1024
TRAIN_TOKEN_CAP = 30_000_000
DEV_TOKEN_CAP = 250_000
SEALED_PROMPT_COUNT = 5
SEALED_PROMPT_TOKENS_PER_STORY = 24
SEALED_PROMPT_TOKEN_COUNT = SEALED_PROMPT_COUNT * SEALED_PROMPT_TOKENS_PER_STORY
TEST_REVEAL_TOTAL_CAP = 250_000
# Reserve the separate global-lowest-CID prompt fixture inside the same hard
# reveal budget; the scored NLL store receives the remainder.
TEST_TOKEN_CAP = TEST_REVEAL_TOTAL_CAP - SEALED_PROMPT_TOKEN_COUNT


@dataclass(frozen=True, slots=True)
class ModelConfig:
    """The one architecture authorized by #1014.

    A 48-wide head is twelve R4 coordinate blocks. No configuration search is
    exposed: changing these values means opening a different experiment.
    """

    vocab_size: int = 4096
    hidden_size: int = 288
    intermediate_size: int = 768
    num_hidden_layers: int = 6
    num_attention_heads: int = 6
    num_key_value_heads: int = 6
    head_dim: int = 48
    r4_blocks_per_head: int = 12
    max_position_embeddings: int = 256
    rms_norm_eps: float = 1e-5
    rope_theta: float = 10_000.0
    bos_token_id: int = BOS_TOKEN_ID
    eos_token_id: int = EOS_TOKEN_ID
    tie_word_embeddings: bool = True

    def validate(self) -> None:
        if self.hidden_size != self.num_attention_heads * self.head_dim:
            raise ValueError("hidden_size must equal num_attention_heads * head_dim")
        if self.num_key_value_heads != self.num_attention_heads:
            raise ValueError("#1014 freezes ordinary multi-head Q/K/V attention")
        if self.head_dim != 4 * self.r4_blocks_per_head:
            raise ValueError("each head must decompose into exact R4 coordinate blocks")
        if self.head_dim % 2:
            raise ValueError("RoPE requires an even head width")
        if self.vocab_size != 4096 or self.max_position_embeddings != 256:
            raise ValueError("#1014 freezes vocabulary 4096 and context 256")
        if not self.tie_word_embeddings:
            raise ValueError("#1014 requires a tied embedding/language-model head")
        if not 0 <= self.bos_token_id < self.vocab_size:
            raise ValueError("BOS id outside vocabulary")
        if not 0 <= self.eos_token_id < self.vocab_size:
            raise ValueError("EOS id outside vocabulary")

    def as_contract(self) -> dict[str, object]:
        self.validate()
        return asdict(self)

    def as_hugging_face_config(self) -> dict[str, object]:
        """Return the config accepted by `HuggingFaceLlamaOracle`."""
        self.validate()
        return {
            "architectures": ["LlamaForCausalLM"],
            "attention_bias": False,
            "bos_token_id": self.bos_token_id,
            "eos_token_id": self.eos_token_id,
            "hidden_act": "silu",
            "hidden_size": self.hidden_size,
            "initializer_range": 0.02,
            "intermediate_size": self.intermediate_size,
            "max_position_embeddings": self.max_position_embeddings,
            "mlp_bias": False,
            "model_type": "llama",
            "num_attention_heads": self.num_attention_heads,
            "num_hidden_layers": self.num_hidden_layers,
            "num_key_value_heads": self.num_key_value_heads,
            "pretraining_tp": 1,
            "rms_norm_eps": self.rms_norm_eps,
            "rope_interleaved": False,
            "rope_scaling": None,
            "rope_theta": self.rope_theta,
            "tie_word_embeddings": self.tie_word_embeddings,
            "torch_dtype": "float32",
            "transformers_version": "not-required",
            "use_cache": True,
            "vocab_size": self.vocab_size,
        }


FROZEN_MODEL_CONFIG = ModelConfig()
FROZEN_MODEL_CONFIG.validate()
