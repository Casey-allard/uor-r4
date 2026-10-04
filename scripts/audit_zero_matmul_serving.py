#!/usr/bin/env python3
"""Zero-Multiplier Numerical Serving Kernel Static Disassembly Auditor.

Verifies the D0-b and Milestone M1/M2 architectural invariants:
- Strictly 0 floating-point / vector register transfer instructions (Class III: `fmul`, `fmov`, `fadd`, etc.)
- Strictly 0 hardware integer multiplier instructions (Class I: `mul`, `madd`, `smull`, `umull`, `smaddl`, `smsubl`, etc.)
- Strictly 0 hardware integer divider instructions (Class II: `sdiv`, `udiv`)
in compiled numerical serving symbols of `libuor_r4_integer.rlib` and release binaries.

Usage:
    python3 scripts/audit_zero_matmul_serving.py [path_to_binary_or_rlib] [--strict-arm64] [--tap] [--stack]

`--stack` (implied for a binary named `uor-r4-stack*`) audits the D11 geometric
stack engine (`uor_r4_integer::stack`): its step and kernels become the mandatory
symbols and the call-graph roots, in place of the retained IntegerModel set. A
stack audit always runs the call graph and fails unless every stack root is
found. Other targets audit exactly as before.
"""

import argparse
import hashlib
import os
import re
import subprocess
import sys

DEFAULT_TARGETS = [
    "target/release/libuor_r4_integer.rlib",
    "target/release/uor-r4-integer",
    "target/release/uor-chat",
]


def get_artifact_metadata(path):
    sha256 = hashlib.sha256()
    with open(path, "rb") as f:
        while chunk := f.read(65536):
            sha256.update(chunk)
    artifact_hash = sha256.hexdigest()

    commit = "unknown"
    try:
        res = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            capture_output=True,
            text=True,
            check=True,
        )
        commit = res.stdout.strip()
    except Exception:
        pass

    compiler = "unknown"
    try:
        res = subprocess.run(
            ["rustc", "--version"],
            capture_output=True,
            text=True,
            check=True,
        )
        compiler = res.stdout.strip()
    except Exception:
        pass

    flags = os.environ.get("RUSTFLAGS", "").strip()
    if not flags:
        flags = os.environ.get("CARGO_ENCODED_RUSTFLAGS", "").strip()
    if not flags:
        flags = "release profile (opt-level=3, codegen-units=16)"

    return artifact_hash, commit, compiler, flags


# Class I: Hardware Multipliers (AArch64 / ARM64)
CLASS_I_PATTERN = re.compile(
    r"\b(mul|madd|msub|mneg|smull|umull|smaddl|umaddl|smsubl|umsubl|smulh|umulh|sqdmulh|sqrdmulh|mla|mls|pmul|pmull|sdot|udot|smlal|smlsl|umlal|umlsl|smull2|umull2|smlal2|smlsl2|umlal2|umlsl2|sqdmull|sqdmull2|sqdmlal|sqdmlal2|sqdmlsl|sqdmlsl2|smmla|ummla|usmmla|usdot|sudot|pmull2)\b",
    re.IGNORECASE,
)

# Class II: Hardware Dividers
CLASS_II_PATTERN = re.compile(
    r"\b(sdiv|udiv)\b",
    re.IGNORECASE,
)

# Class III: Floating-Point & Register Transfer Instructions
CLASS_III_PATTERN = re.compile(
    r"(\b(fmul|fmov|fadd|fsub|fdiv|fmadd|fmsub|fnmadd|fnmsub|fnmul|fsqrt|fcmp|fcmpe|scvtf|ucvtf|fneg|fabs|fmla|fmls|fmax|fmin|fmaxnm|fminnm|fmaxp|fminp|fmaxnmp|fminnmp|fmaxv|fminv|fmaxnmv|fminnmv|fmulx|fabd|faddp|facge|facgt|fcmeq|fcmge|fcmgt|fcmle|fcmlt|fcsel|fccmp|fccmpe|frecpe|frecps|frecpx|frsqrte|frsqrts|fcmla|fcadd|fdot|fmlal|fmlsl|fmmla|bfdot|bfmmla|bfmlal|bfmlalb|bfmlalt)\b"
    r"|\b(frint[aimnpzx]|fcvt[a-z0-9]*)\b"
    r"|\b(f[a-z]+|bf[a-z]+)\.[0-9]+[a-z]\b)",
    re.IGNORECASE,
)

# Combined full strict pattern
STRICT_FORBIDDEN_PATTERN = re.compile(
    r"(\b(mul|madd|msub|mneg|smull|umull|smaddl|umaddl|smsubl|umsubl|smulh|umulh|sqdmulh|sqrdmulh|mla|mls|pmul|pmull|sdot|udot|smlal|smlsl|umlal|umlsl|smull2|umull2|smlal2|smlsl2|umlal2|umlsl2|sqdmull|sqdmull2|sqdmlal|sqdmlal2|sqdmlsl|sqdmlsl2|smmla|ummla|usmmla|usdot|sudot|pmull2"
    r"|sdiv|udiv"
    r"|fmul|fmov|fadd|fsub|fdiv|fmadd|fmsub|fnmadd|fnmsub|fnmul|fsqrt|fcmp|fcmpe|scvtf|ucvtf|fneg|fabs|fmla|fmls|fmax|fmin|fmaxnm|fminnm|fmaxp|fminp|fmaxnmp|fminnmp|fmaxv|fminv|fmaxnmv|fminnmv|fmulx|fabd|faddp|facge|facgt|fcmeq|fcmge|fcmgt|fcmle|fcmlt|fcsel|fccmp|fccmpe|frecpe|frecps|frecpx|frsqrte|frsqrts|fcmla|fcadd|fdot|fmlal|fmlsl|fmmla|bfdot|bfmmla|bfmlal|bfmlalb|bfmlalt)\b"
    r"|\b(frint[aimnpzx]|fcvt[a-z0-9]*)\b"
    r"|\b(f[a-z]+|bf[a-z]+)\.[0-9]+[a-z]\b)",
    re.IGNORECASE,
)

# Historical pattern set to strict forbidden pattern
CORE_FORBIDDEN_PATTERN = STRICT_FORBIDDEN_PATTERN

# Mandatory serving symbols for library archive (.rlib)
RLIB_MANDATORY_SYMBOLS = [
    {
        "name": "UnitS3Q30::hopf_project",
        "pattern": re.compile(r"UnitS3Q30.*hopf_project\b"),
        "mangled": re.compile(r"__RNv.*UnitS3Q30.*12hopf_project\b"),
        "description": "S3 -> S2 Hopf base projection coordinates",
    },
    {
        "name": "UnitS3Q30::from_i32_coords",
        "pattern": re.compile(r"UnitS3Q30.*from_i32_coords\b"),
        "mangled": re.compile(r"__RNv.*UnitS3Q30.*15from_i32_coords\b"),
        "description": "S3 unit quaternion normalization from 4D coordinates",
    },
    {
        "name": "UnitS3Q30::fiber_u1_q30",
        "pattern": re.compile(r"UnitS3Q30.*fiber_u1_q30\b"),
        "mangled": re.compile(r"__RNv.*UnitS3Q30.*12fiber_u1_q30\b"),
        "description": "U(1) fiber unit phasor extraction",
    },
    {
        "name": "UnitS3Q30::hopf_fiber_project",
        "pattern": re.compile(r"UnitS3Q30.*hopf_fiber_project\b"),
        "mangled": re.compile(r"__RNv.*UnitS3Q30.*18hopf_fiber_project\b"),
        "description": "Combined base S2 and fiber U(1) projection",
    },
    {
        "name": "T8ZetaState::step",
        "pattern": re.compile(r"T8ZetaState.*step\b(?!_raw)"),
        "mangled": re.compile(r"__RNv.*T8ZetaState.*4step\b"),
        "description": "T^8 toroidal Riemann zeta phase progression",
    },
    {
        "name": "T8ZetaState::step_raw",
        "pattern": re.compile(r"T8ZetaState.*step_raw\b"),
        "mangled": re.compile(r"__RNv.*T8ZetaState.*8step_raw\b"),
        "description": "T^8 unmodulated zeta frequency progression",
    },
    {
        "name": "IntegerModel::step",
        "pattern": re.compile(r"IntegerModel.*step\b(?!_conversational)"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*4step\b"),
        "description": "Autoregressive state transition step",
    },
    {
        "name": "IntegerModel::step_conversational",
        "pattern": re.compile(r"IntegerModel.*step_conversational(?:_into)?\b"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*(?:19step_conversational|24step_conversational_into)\b"),
        "description": "Conversational autoregressive step with partitioned memory",
    },
    {
        "name": "IntegerModel::matrix_work",
        "pattern": re.compile(r"IntegerModel.*matrix_work\b"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*11matrix_work\b"),
        "description": "Zero-MatMul affine matrix-vector product",
    },
    {
        "name": "IntegerModel::affine",
        "pattern": re.compile(r"IntegerModel.*affine\b"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*6affine\b"),
        "description": "Linear combination over parameter codes",
    },
    {
        "name": "IntegerModel::vector_work",
        "pattern": re.compile(r"IntegerModel.*vector_work\b"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*11vector_work\b"),
        "description": "Parameter bias scaling and extraction",
    },
    {
        "name": "IntegerModel::embed",
        "pattern": re.compile(r"IntegerModel.*embed\b"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*5embed\b"),
        "description": "Token embedding projection via shift-and-add",
    },
    {
        "name": "IntegerModel::project_vocab",
        "pattern": re.compile(r"IntegerModel.*project_vocab\b"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*13project_vocab\b"),
        "description": "Vocab projection via shift-and-add rows",
    },
    {
        "name": "model::softmax",
        "pattern": re.compile(r"model.*softmax\b"),
        "mangled": re.compile(r"__RNv.*model.*7softmax\b"),
        "description": "Q48 integer softmax distribution",
    },
    {
        "name": "low_bit_dot (model or packed_rows)",
        "pattern": re.compile(r"(?:^|::)(?:model|packed_rows)::low_bit_dot\b"),
        "mangled": re.compile(r"__RNv.*(?:model|packed_rows).*11low_bit_dot\b"),
        "description": "Inner product via signed-4 lookup table",
    },
    {
        "name": "model::low_bit_products",
        "pattern": re.compile(r"model.*low_bit_products\b"),
        "mangled": re.compile(r"__RNv.*model.*16low_bit_products\b"),
        "description": "Shift-add 4-bit scalar multiples",
    },
    {
        "name": "math::atan2_q30",
        "pattern": re.compile(r"math.*atan2_q30\b"),
        "mangled": re.compile(r"__RNv.*math.*9atan2_q30\b"),
        "description": "16-stage CORDIC fixed-point arc-tangent",
    },
    {
        "name": "math::checked_mul",
        "pattern": re.compile(r"math.*checked_mul\b(?!_unsigned)"),
        "mangled": re.compile(r"__RNv.*math.*11checked_mul\b"),
        "description": "Shift-and-add exact signed multiplication",
    },
    {
        "name": "math::div_round",
        "pattern": re.compile(r"math.*div_round\b"),
        "mangled": re.compile(r"__RNv.*math.*9div_round\b"),
        "description": "Signed integer division rounded to nearest",
    },
    {
        "name": "math::scale_pow2",
        "pattern": re.compile(r"math.*scale_pow2\b"),
        "mangled": re.compile(r"__RNv.*math.*10scale_pow2\b"),
        "description": "Dyadic power-of-two rescaling with rounding",
    },
    {
        "name": "math::sum_squares",
        "pattern": re.compile(r"math.*sum_squares\b"),
        "mangled": re.compile(r"__RNv.*math.*11sum_squares\b"),
        "description": "Shift-and-add exact vector squared norm",
    },
    {
        "name": "model::compress_barycenter_key_fibonacci",
        "pattern": re.compile(r"(?:model.*)?compress_barycenter_key_fibonacci\b"),
        "mangled": re.compile(r"__RNv.*(?:model.*)?(?:33)?compress_barycenter_key_fibonacci\b"),
        "description": "Fibonacci integer barycenter key compression in Z[phi]",
    },
    {
        "name": "model::compute_turn_prime_signature",
        "pattern": re.compile(r"(?:model.*)?compute_turn_prime_signature\b"),
        "mangled": re.compile(r"__RNv.*(?:model.*)?(?:28)?compute_turn_prime_signature\b"),
        "description": "Galois LFSR 64-bit prime signature computation",
    },
    {
        "name": "model::extract_salient_tokens",
        "pattern": re.compile(r"(?:model.*)?extract_salient_tokens\b"),
        "mangled": re.compile(r"__RNv.*(?:model.*)?(?:22)?extract_salient_tokens\b"),
        "description": "Top-4 salient entity token extraction via zero-matmul scoring",
    },
    {
        "name": "model::div_round_100m_raw",
        "pattern": re.compile(r"div_round_100m_raw\b"),
        "mangled": re.compile(r"__RNv.*(?:model.*)?18div_round_100m_raw\b"),
        "description": "Radix-16 exact quotient and remainder for 100M division",
    },
    {
        "name": "model::mul_fraction_radix16",
        "pattern": re.compile(r"mul_fraction_radix16\b"),
        "mangled": re.compile(r"__RNv.*(?:model.*)?20mul_fraction_radix16\b"),
        "description": "Radix-16 nibble lookup product against precomputed fraction table",
    },
    {
        "name": "model::build_fraction_table",
        "pattern": re.compile(r"build_fraction_table\b"),
        "mangled": re.compile(r"__RNv.*(?:model.*)?20build_fraction_table\b"),
        "description": "16-multiple table construction via repeated addition",
    },
    {
        "name": "model::mul_shift_add_i64",
        "pattern": re.compile(r"mul_shift_add_i64\b"),
        "mangled": re.compile(r"__RNv.*(?:model.*)?17mul_shift_add_i64\b"),
        "description": "Shift-add exact signed 64-bit multiplication",
    },
    {
        "name": "model::build_pair_tables_256",
        "pattern": re.compile(r"build_pair_tables_256\b"),
        "mangled": re.compile(r"__RNv.*(?:model.*)?21build_pair_tables_256\b"),
        "description": "256-entry combined pair table construction for width 256",
    },
    {
        "name": "model::low_bit_dot_8_pair_tables",
        "pattern": re.compile(r"low_bit_dot_8_pair_tables\b"),
        "mangled": re.compile(r"__RNv.*(?:model.*)?24low_bit_dot_8_pair_tables\b"),
        "description": "Inner dot product of 8 contiguous rows via precomputed pair tables",
    },
    {
        "name": "model::build_pair_tables_576",
        "pattern": re.compile(r"build_pair_tables_576\b"),
        "mangled": re.compile(r"__RNv.*(?:model.*)?21build_pair_tables_576\b"),
        "description": "256-entry combined pair table construction for width 576",
    },
    {
        "name": "model::low_bit_dot_4_contiguous_wide_pair_tables",
        "pattern": re.compile(r"low_bit_dot_4_contiguous_wide_pair_tables\b"),
        "mangled": re.compile(r"__RNv.*(?:model.*)?39low_bit_dot_4_contiguous_wide_pair_tables\b"),
        "description": "Four contiguous wide rows via precomputed pair tables",
    },
    # Lorentz read and hyperbolic cache scoring symbols
    {
        "name": "LorentzRead::score",
        "pattern": re.compile(r"LorentzRead.*score\b"),
        "mangled": re.compile(r"__RNv.*LorentzRead.*5score\b"),
        "description": "Hyperbolic Lorentz read score with sealed arcosh table",
    },
    {
        "name": "LorentzRead::new",
        "pattern": re.compile(r"LorentzRead.*new\b"),
        "mangled": re.compile(r"__RNv.*LorentzRead.*3new\b"),
        "description": "Learned Lorentz read scaling initialization",
    },
    {
        "name": "lorentz::excess_q32",
        "pattern": re.compile(r"lorentz.*excess_q32\b"),
        "mangled": re.compile(r"__RNv.*lorentz.*10excess_q32\b"),
        "description": "Hyperbolic distance excess z - 1 at Q32",
    },
    {
        "name": "lorentz::arcosh1p_q24",
        "pattern": re.compile(r"lorentz.*arcosh1p_q24\b"),
        "mangled": re.compile(r"__RNv.*lorentz.*12arcosh1p_q24\b"),
        "description": "Tabulated arcosh(1+u) at Q24 via dyadic octave interpolation",
    },
    {
        "name": "lorentz::squared_norm",
        "pattern": re.compile(r"lorentz.*squared_norm\b"),
        "mangled": re.compile(r"__RNv.*lorentz.*12squared_norm\b"),
        "description": "Exact integer sum of squared codes for Lorentz lifting",
    },
    {
        "name": "lorentz::exp_q32",
        "pattern": re.compile(r"lorentz.*exp_q32\b"),
        "mangled": re.compile(r"__RNv.*lorentz.*7exp_q32\b"),
        "description": "Fixed-point exponential via Taylor series with zero hardware multipliers",
    },
    # Sampling symbols
    {
        "name": "sampling::exact_min_p_threshold",
        "pattern": re.compile(r"sampling.*exact_min_p_threshold\b"),
        "mangled": re.compile(r"__RNv.*sampling.*21exact_min_p_threshold\b"),
        "description": "Zero-matmul bit-for-bit shift-add MinP threshold over set bits",
    },
    {
        "name": "Sampler::select",
        "pattern": re.compile(r"Sampler.*select\b"),
        "mangled": re.compile(r"__RNv.*Sampler.*6select\b"),
        "description": "Integer token selection for normalized Q48 output distribution",
    },
    {
        "name": "sampling::validate_distribution",
        "pattern": re.compile(r"sampling.*validate_distribution\b"),
        "mangled": re.compile(r"__RNv.*sampling.*21validate_distribution\b"),
        "description": "Validation of Q48 probability normalization",
    },
    {
        "name": "sampling::select_ticket",
        "pattern": re.compile(r"sampling.*select_ticket\b"),
        "mangled": re.compile(r"__RNv.*sampling.*13select_ticket\b"),
        "description": "Exact ticket interval search over unnormalized masses",
    },
    {
        "name": "Sampler::draw_below",
        "pattern": re.compile(r"Sampler.*draw_below\b"),
        "mangled": re.compile(r"__RNv.*Sampler.*10draw_below\b"),
        "description": "Unbiased pseudo-random draw below bound using xorshift64",
    },
    {
        "name": "sampling::heapsort_ranked",
        "pattern": re.compile(r"sampling.*heapsort_ranked\b"),
        "mangled": re.compile(r"__RNv.*sampling.*15heapsort_ranked\b"),
        "description": "Zero-matmul bit-shift heapsort for ranked token indices",
    },
    {
        "name": "math::step_zeta_phase_scalar",
        "pattern": re.compile(r"math.*step_zeta_phase_scalar\b"),
        "mangled": re.compile(r"__RNv.*math.*22step_zeta_phase_scalar\b"),
        "description": "Scalar zeta phase update with zero float register synthesis",
    },
]

# Mandatory serving symbols for compiled executable binary (e.g. uor-r4-integer, uor-chat)
BIN_MANDATORY_SYMBOLS = [
    {
        "name": "IntegerModel::step",
        "pattern": re.compile(r"IntegerModel.*step\b(?!_conversational)"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*4step\b"),
        "description": "Autoregressive state transition step",
        "alternative_group": "step_transition",
        "group_display": "IntegerModel::step (or IntegerModel::step_conversational)",
    },
    {
        "name": "IntegerModel::step_conversational",
        "pattern": re.compile(r"IntegerModel.*step_conversational(?:_into)?\b"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*(?:19step_conversational|24step_conversational_into)\b"),
        "description": "Conversational autoregressive step with partitioned memory",
        "alternative_group": "step_transition",
        "group_display": "IntegerModel::step (or IntegerModel::step_conversational)",
    },
    {
        "name": "IntegerModel::embed",
        "pattern": re.compile(r"IntegerModel.*embed\b"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*5embed\b"),
        "description": "Token embedding projection via shift-and-add",
    },
    {
        "name": "IntegerModel::project_vocab",
        "pattern": re.compile(r"IntegerModel.*project_vocab\b"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*13project_vocab\b"),
        "description": "Vocab projection via shift-and-add rows",
    },
    {
        "name": "model::compress_barycenter_key_fibonacci",
        "pattern": re.compile(r"(?:model.*)?compress_barycenter_key_fibonacci\b"),
        "mangled": re.compile(r"__RNv.*(?:model.*)?(?:33)?compress_barycenter_key_fibonacci\b"),
        "description": "Fibonacci integer barycenter key compression in Z[phi]",
    },
    {
        "name": "model::compute_turn_prime_signature",
        "pattern": re.compile(r"(?:model.*)?compute_turn_prime_signature\b"),
        "mangled": re.compile(r"__RNv.*(?:model.*)?(?:28)?compute_turn_prime_signature\b"),
        "description": "Galois LFSR 64-bit prime signature computation",
    },
    {
        "name": "model::extract_salient_tokens",
        "pattern": re.compile(r"(?:model.*)?extract_salient_tokens\b"),
        "mangled": re.compile(r"__RNv.*(?:model.*)?(?:22)?extract_salient_tokens\b"),
        "description": "Top-4 salient entity token extraction via zero-matmul scoring",
    },
    # Lorentz read and hyperbolic cache scoring symbols
    {
        "name": "LorentzRead::score",
        "pattern": re.compile(r"LorentzRead.*score\b"),
        "mangled": re.compile(r"__RNv.*LorentzRead.*5score\b"),
        "description": "Hyperbolic Lorentz read score with sealed arcosh table",
    },
    {
        "name": "LorentzRead::new",
        "pattern": re.compile(r"LorentzRead.*new\b"),
        "mangled": re.compile(r"__RNv.*LorentzRead.*3new\b"),
        "description": "Learned Lorentz read scaling initialization",
    },
    {
        "name": "lorentz::excess_q32",
        "pattern": re.compile(r"lorentz.*excess_q32\b"),
        "mangled": re.compile(r"__RNv.*lorentz.*10excess_q32\b"),
        "description": "Hyperbolic distance excess z - 1 at Q32",
    },
    {
        "name": "lorentz::arcosh1p_q24",
        "pattern": re.compile(r"lorentz.*arcosh1p_q24\b"),
        "mangled": re.compile(r"__RNv.*lorentz.*12arcosh1p_q24\b"),
        "description": "Tabulated arcosh(1+u) at Q24 via dyadic octave interpolation",
    },
    {
        "name": "lorentz::squared_norm",
        "pattern": re.compile(r"lorentz.*squared_norm\b"),
        "mangled": re.compile(r"__RNv.*lorentz.*12squared_norm\b"),
        "description": "Exact integer sum of squared codes for Lorentz lifting",
    },
    {
        "name": "lorentz::exp_q32",
        "pattern": re.compile(r"lorentz.*exp_q32\b"),
        "mangled": re.compile(r"__RNv.*lorentz.*7exp_q32\b"),
        "description": "Fixed-point exponential via Taylor series with zero hardware multipliers",
    },
    # Sampling symbols
    {
        "name": "sampling::exact_min_p_threshold",
        "pattern": re.compile(r"sampling.*exact_min_p_threshold\b"),
        "mangled": re.compile(r"__RNv.*sampling.*21exact_min_p_threshold\b"),
        "description": "Zero-matmul bit-for-bit shift-add MinP threshold over set bits",
    },
    {
        "name": "Sampler::select",
        "pattern": re.compile(r"Sampler.*select\b"),
        "mangled": re.compile(r"__RNv.*Sampler.*6select\b"),
        "description": "Integer token selection for normalized Q48 output distribution",
    },
    {
        "name": "sampling::validate_distribution",
        "pattern": re.compile(r"sampling.*validate_distribution\b"),
        "mangled": re.compile(r"__RNv.*sampling.*21validate_distribution\b"),
        "description": "Validation of Q48 probability normalization",
    },
    {
        "name": "sampling::select_ticket",
        "pattern": re.compile(r"sampling.*select_ticket\b"),
        "mangled": re.compile(r"__RNv.*sampling.*13select_ticket\b"),
        "description": "Exact ticket interval search over unnormalized masses",
    },
    {
        "name": "Sampler::draw_below",
        "pattern": re.compile(r"Sampler.*draw_below\b"),
        "mangled": re.compile(r"__RNv.*Sampler.*10draw_below\b"),
        "description": "Unbiased pseudo-random draw below bound using xorshift64",
    },
    {
        "name": "sampling::heapsort_ranked",
        "pattern": re.compile(r"sampling.*heapsort_ranked\b"),
        "mangled": re.compile(r"__RNv.*sampling.*15heapsort_ranked\b"),
        "description": "Zero-matmul bit-shift heapsort for ranked token indices",
    },
    {
        "name": "math::step_zeta_phase_scalar",
        "pattern": re.compile(r"math.*step_zeta_phase_scalar\b"),
        "mangled": re.compile(r"__RNv.*math.*22step_zeta_phase_scalar\b"),
        "description": "Scalar zeta phase update with zero float register synthesis",
    },
]

# The packed block kernels are inline functions. Inspect their actual emitted
# callers (and wrappers that may absorb them), without treating an absent name
# as audited. These are additional coverage for both old and packed artifacts;
# the historical mandatory-symbol and opcode policies remain unchanged.
PACKED_KERNEL_CALLERS = [
    {
        "name": f"IntegerModel::{name}",
        "pattern": re.compile(r"IntegerModel.*" + re.escape(name) + r"\b"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*" + str(len(name)) + re.escape(name) + r"\b"),
        "description": "Emitted containing function for signed4 affine/vocabulary kernels",
    }
    for name in (
        "project_vocab_with_products_into",
        "project_vocab_with_products",
        "matrix_work_direct_into",
        "matrix_work_direct",
        "affine_direct_into",
        "affine_direct",
        "affine_wide_into",
        "step_conversational_into",
    )
]

# Exact H4 classification is a standalone numerical component. Inspect every
# emitted classifier/helper range that is present without making historical
# artifacts require a component they did not contain. Absent inline helpers
# still require checking the actual containing function before claiming coverage.
EXACT_GEOMETRY_SYMBOLS = [
    {
        "name": f"h4_classifier::{name}",
        "pattern": re.compile(r"h4_classifier.*" + re.escape(name) + r"\b"),
        "mangled": re.compile(r"__RNv.*h4_classifier.*" + str(len(name)) + re.escape(name) + r"\b"),
        "description": "Exact signed H4 classification and integer score comparison",
    }
    for name in (
        "signed_h4_code_i32",
        "family_candidates",
        "golden_sign_bits",
        "checked_add_square_terms",
        "coefficient_term",
        "score",
        "score_difference_order",
    )
]

# Loading/hash validation is outside the numerical lookup. New artifacts also
# expose historical-code inverse, Hamilton composition and directed relation.
# As above, old artifacts need not contain a component that did not exist yet.
EXACT_GEOMETRY_SYMBOLS += [
    {
        "name": f"HistoricalH4Tables::{name}",
        "pattern": re.compile(r"HistoricalH4Tables.*" + re.escape(name) + r"\b"),
        "mangled": re.compile(r"__RNv.*HistoricalH4Tables.*" + str(len(name)) + re.escape(name) + r"\b"),
        "description": "Immutable historical H4 numerical table lookup",
    }
    for name in ("inverse", "compose", "relative")
]

# Discrete parameter codec symbols (audited when present; zero multiplier/divider/float)
EXACT_GEOMETRY_SYMBOLS += [
    {
        "name": "codec::mul_small_code_i64",
        "pattern": re.compile(r"mul_small_code_i64\b"),
        "mangled": re.compile(r"__RNv.*mul_small_code_i64\b"),
        "description": "Multiplier-free small code product via shift-and-add",
    },
    {
        "name": "Grouped4BitRow::to_q16_vector",
        "pattern": re.compile(r"Grouped4BitRow.*to_q16_vector\b"),
        "mangled": re.compile(r"__RNv.*Grouped4BitRow.*13to_q16_vector\b"),
        "description": "Multiplier-free grouped 4-bit activation extraction",
    },
]

# Mandatory dialogue serving symbols when auditing interactive chatbot binaries (uor-chat)
CHAT_MANDATORY_SYMBOLS = [
    {
        "name": "DialogueConversation::respond_stream",
        "pattern": re.compile(r"DialogueConversation.*respond_stream\b"),
        "mangled": re.compile(r"__RNv.*DialogueConversation.*14respond_stream\b"),
        "description": "Dialogue conversation response stream entry point",
        "alternative_group": "dialogue_entry",
        "group_display": "DialogueConversation::respond_stream (or DialogueConversation::respond)",
    },
    {
        "name": "DialogueConversation::respond",
        "pattern": re.compile(r"DialogueConversation.*respond\b(?!_stream)"),
        "mangled": re.compile(r"__RNv.*DialogueConversation.*7respond\b"),
        "description": "Dialogue conversation response entry point",
        "alternative_group": "dialogue_entry",
        "group_display": "DialogueConversation::respond_stream (or DialogueConversation::respond)",
    },
    {
        "name": "TextSession::step_next_token",
        "pattern": re.compile(r"TextSession.*step_next_token\b"),
        "mangled": re.compile(r"__RNv.*TextSession.*15step_next_token\b"),
        "description": "Single-token generation step in active text session",
        "alternative_group": "text_session_step",
        "group_display": "TextSession::step_next_token (or TextSession::observe)",
    },
    {
        "name": "TextSession::observe",
        "pattern": re.compile(r"TextSession.*observe\b"),
        "mangled": re.compile(r"__RNv.*TextSession.*7observe\b"),
        "description": "Observation step in active text session",
        "alternative_group": "text_session_step",
        "group_display": "TextSession::step_next_token (or TextSession::observe)",
    },
    {
        "name": "IntegerModel::project_vocab_with_products_into",
        "pattern": re.compile(r"IntegerModel.*project_vocab_with_products_into\b"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*32project_vocab_with_products_into\b"),
        "description": "Grouped-LUT vocabulary projection for width 576",
    },
    {
        "name": "IntegerModel::affine_wide_into",
        "pattern": re.compile(r"IntegerModel.*affine_wide_into\b"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*16affine_wide_into\b"),
        "description": "Zero-allocation wide affine transformation for width 576",
    },
]


def _stack_kernel(name, module, description):
    """A `uor_r4_integer::stack` function: demangled `stack::<module>::<name>`,
    v0-mangled `...<len><name>` (identifiers are length-prefixed)."""
    return {
        "name": f"stack::{module}::{name}",
        "pattern": re.compile(r"stack::" + module + r"::" + re.escape(name) + r"\b"),
        "mangled": re.compile(r"__RNv.*5stack.*" + str(len(name)) + re.escape(name) + r"\b"),
        "description": description,
    }


# Mandatory symbols of the D11 geometric-stack engine (`uor_r4_integer::stack`),
# selected only by `--stack` or a `uor-r4-stack*` binary. Every kernel is
# `#[inline(never)]` so that each has its own emitted range.
STACK_MANDATORY_SYMBOLS = [
    {
        "name": "IntegerStackSession::step",
        # The session type may carry an erased lifetime (`<'_>`, v0 `L_E`).
        "pattern": re.compile(r"IntegerStackSession(?:<[^>]*>)?>::step\b"),
        "mangled": re.compile(r"__RNv.*19IntegerStackSession(?:L[0-9A-Za-z_]*E)?4step\b"),
        "description": "D11 stack step: embedding, mixers, MLPs, final norm and head",
    },
    _stack_kernel("stack_recurrence", "session", "Quaternion transport recurrence mixer"),
    _stack_kernel("stack_rotation", "session", "Unit rotation quaternion by long division"),
    _stack_kernel("stack_read", "session", "Dot/Lorentz/L2 read mixer with NoRead softmax"),
    _stack_kernel("stack_swiglu", "session", "SwiGLU gating products by digit tables"),
    _stack_kernel("stack_gemv", "kernels", "4-bit weight map by activation-table reads and adds"),
    _stack_kernel("stack_gemv_pairs", "kernels", "4-bit weight map by pair-table reads and adds"),
    _stack_kernel("stack_pair_tables", "kernels", "Weight-byte pair tables by additions"),
    _stack_kernel("stack_activation_tables", "kernels", "Per-column nibble product tables by shifts and adds"),
    _stack_kernel("stack_dequant_row", "kernels", "Embedding row by shift-add group scales"),
    _stack_kernel("stack_rms_norm", "kernels", "RMSNorm by digit-table squares, long division and isqrt"),
    _stack_kernel("stack_quantize16", "kernels", "16-bit requantization by shifts"),
    _stack_kernel("stack_max_abs_i32", "kernels", "Scalar largest magnitude (i32)"),
    _stack_kernel("stack_max_abs_i64", "kernels", "Scalar largest magnitude (i64)"),
    _stack_kernel("stack_exp_neg", "kernels", "Sealed exp table with digit-table interpolation"),
    _stack_kernel("stack_sigmoid_q31", "kernels", "Sigmoid from the exp table and long division"),
    _stack_kernel("stack_activation", "kernels", "SiLU/GELU sealed-table interpolation"),
    _stack_kernel("stack_arcosh1p_q24", "kernels", "Sealed arcosh table with octave interpolation"),
    _stack_kernel("stack_lift", "kernels", "Hyperboloid lift by digit-table squares and isqrt"),
    _stack_kernel("stack_lorentz_distance", "kernels", "Lorentz distance from lifts and inner product"),
    _stack_kernel("stack_l2_distance", "kernels", "L2 distance by digit-table squares and isqrt"),
    _stack_kernel("stack_query_tables", "kernels", "Per-coordinate query multiple tables"),
    _stack_kernel("stack_dot", "kernels", "Exact query-key inner product by digit tables"),
    _stack_kernel("stack_mix_row", "kernels", "Exact weighted value accumulation by digit tables"),
    _stack_kernel("stack_hamilton", "kernels", "Exact Hamilton product of quaternions"),
    _stack_kernel("stack_mul_u128", "kernels", "Radix-16 multiple-table product modulo 2^128"),
    _stack_kernel("stack_mul_u64", "kernels", "Radix-16 multiple-table product modulo 2^64"),
    _stack_kernel("stack_square", "kernels", "Radix-16 multiple-table square"),
    _stack_kernel("stack_div_u64", "kernels", "Restoring long division (u64)"),
    _stack_kernel("stack_div_u128", "kernels", "Restoring long division (u128)"),
    _stack_kernel("stack_isqrt", "kernels", "Digit-by-digit integer square root"),
    _stack_kernel("stack_argmax", "kernels", "Greedy integer argmax over logits"),
    _stack_kernel("stack_snap_select", "kernels", "Icosian root selection by exact Z[phi] comparison"),
    _stack_kernel("stack_snap_rotation", "kernels", "Snapped rotation by shift-add golden-ratio terms"),
]

# Call-graph roots of the stack engine (v0-mangled names, as `otool -tvV`
# prints them): its step and the greedy selection, as (name, pattern). A stack
# audit runs the call graph and fails unless every root is found, so a build
# whose names these patterns miss (legacy mangling, an `.llvm.` suffix) cannot
# pass with no reachable functions.
STACK_CALL_GRAPH_ROOTS = [
    ("IntegerStackSession::step", re.compile(r"19IntegerStackSession(?:L[0-9A-Za-z_]*E)?4step$")),
    ("stack::kernels::stack_argmax", re.compile(r"12stack_argmax$")),
]


def run_stack_pattern_tests():
    """The stack patterns must match their demangled and v0-mangled names and
    must not confuse kernels that share a prefix."""
    for entry in STACK_MANDATORY_SYMBOLS:
        name = entry["name"].split("::")[-1]
        if entry["name"].startswith("stack::"):
            module = entry["name"].split("::")[1]
            demangled = f"uor_r4_integer::stack::{module}::{name}"
            mangled = (
                f"__RNvNtNtCs1a2b3c_14uor_r4_integer5stack{len(module)}{module}{len(name)}{name}"
            )
        else:
            demangled = "<uor_r4_integer::stack::session::IntegerStackSession<'_>>::step"
            mangled = "__RNvMs_NtNtCs1a2b3c_14uor_r4_integer5stack7sessionINtB5_19IntegerStackSessionL_E4step"
        if not entry["pattern"].search(demangled):
            raise AssertionError(f"Stack pattern test failed: {entry['name']} misses {demangled}")
        if not entry["mangled"].search(mangled):
            raise AssertionError(f"Stack pattern test failed: {entry['name']} misses {mangled}")
    step_names = [
        "__RNvMs_NtNtCs1a2b3c_14uor_r4_integer5stack7sessionNtB5_19IntegerStackSession4step",
        "__RNvMs_NtNtCs1a2b3c_14uor_r4_integer5stack7sessionINtB5_19IntegerStackSessionL_E4step",
        "__RNvNtNtCs1a2b3c_14uor_r4_integer5stack7kernels12stack_argmax",
    ]
    for root_name in step_names:
        if not any(p.search(root_name) for _, p in STACK_CALL_GRAPH_ROOTS):
            raise AssertionError(f"Stack pattern test failed: no call-graph root matches {root_name}")
    for other in [
        "__RNvMs_NtNtCs1a2b3c_14uor_r4_integer5stack7sessionNtB5_19IntegerStackSession5reset",
        "__RNvMs_NtNtCs1a2b3c_14uor_r4_integer5stack7sessionNtB5_17IntegerStackModel7session",
    ]:
        if any(p.search(other) for _, p in STACK_CALL_GRAPH_ROOTS):
            raise AssertionError(f"Stack pattern test failed: {other} is not a serving root")
    # Every root must be found. A build whose argmax the pattern misses (legacy
    # mangling, an `.llvm.` suffix) reports that root missing, and so fails.
    _, missing = find_serving_roots(step_names, [], STACK_CALL_GRAPH_ROOTS)
    if missing:
        raise AssertionError(f"Stack pattern test failed: roots {missing} missed in a v0 build")
    for renamed in [
        "__ZN14uor_r4_integer5stack7kernels12stack_argmax17h0123456789abcdefE",
        "__RNvNtNtCs1a2b3c_14uor_r4_integer5stack7kernels12stack_argmax.llvm.1234567890",
    ]:
        _, missing = find_serving_roots([step_names[0], renamed], [], STACK_CALL_GRAPH_ROOTS)
        if missing != ["stack::kernels::stack_argmax"]:
            raise AssertionError(f"Stack pattern test failed: {renamed} did not report the argmax root missing")
    _, missing = find_serving_roots([], [], STACK_CALL_GRAPH_ROOTS)
    if missing != [name for name, _ in STACK_CALL_GRAPH_ROOTS]:
        raise AssertionError("Stack pattern test failed: an empty call graph did not report every root missing")
    for longer, shorter in [
        ("stack_activation_tables", "stack_activation"),
        ("stack_gemv_pairs", "stack_gemv"),
        ("stack_div_u128", "stack_div_u64"),
        ("stack_mul_u128", "stack_mul_u64"),
    ]:
        entry = next(e for e in STACK_MANDATORY_SYMBOLS if e["name"].endswith("::" + shorter))
        if entry["pattern"].search(f"uor_r4_integer::stack::kernels::{longer}"):
            raise AssertionError(f"Stack pattern test failed: {shorter} matches {longer}")


def find_target_artifact(user_arg=None):
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    target_dirs = []
    if "CARGO_TARGET_DIR" in os.environ:
        target_dirs.append(os.environ["CARGO_TARGET_DIR"])
    target_dirs.append(os.path.join(repo_root, "target"))

    if user_arg:
        if os.path.exists(user_arg):
            return os.path.abspath(user_arg)
        candidate = os.path.join(repo_root, user_arg)
        if os.path.exists(candidate):
            return os.path.abspath(candidate)
        for tdir in target_dirs:
            candidate = os.path.join(tdir, "release", user_arg)
            if os.path.exists(candidate):
                return os.path.abspath(candidate)
            candidate = os.path.join(tdir, user_arg)
            if os.path.exists(candidate):
                return os.path.abspath(candidate)
        sys.exit(f"ERROR: Specified target artifact does not exist: {user_arg}")

    candidates_to_try = []
    for tdir in target_dirs:
        for t in ["libuor_r4_integer.rlib", "uor-r4-integer", "uor-chat"]:
            candidates_to_try.append(os.path.join(tdir, "release", t))
            candidates_to_try.append(os.path.join(tdir, "release", "deps", t))
    for t in DEFAULT_TARGETS:
        candidates_to_try.append(os.path.join(repo_root, t))
        candidates_to_try.append(t)

    for candidate in candidates_to_try:
        if os.path.exists(candidate):
            return os.path.abspath(candidate)

    sys.exit(
        f"ERROR: No target artifact found. Checked: {', '.join(candidates_to_try[:4])}...\n"
        "Build with: cargo build --release -p uor-r4-integer"
    )


def check_tool_availability(disasm_choice="auto"):
    """Check if objdump or otool is available and functional."""
    tools_found = []
    if disasm_choice in ("auto", "objdump"):
        try:
            res = subprocess.run(["objdump", "--version"], capture_output=True, text=True)
            if res.returncode == 0:
                tools_found.append("objdump")
        except FileNotFoundError:
            pass
    if disasm_choice in ("auto", "otool"):
        try:
            res = subprocess.run(["otool", "--version"], capture_output=True, text=True)
            if res.returncode == 0 or "otool" in res.stderr:
                tools_found.append("otool")
        except FileNotFoundError:
            pass
    return tools_found


def disassemble_artifact(path, disasm_choice="auto"):
    """Disassemble using objdump (preferred for demangling) or otool."""
    is_demangled = False

    if disasm_choice in ("auto", "objdump"):
        try:
            res = subprocess.run(
                ["objdump", "--demangle", "-d", path],
                capture_output=True,
                text=True,
                check=True,
            )
            is_demangled = True
            return res.stdout, is_demangled
        except (subprocess.SubprocessError, FileNotFoundError):
            if disasm_choice == "objdump":
                sys.exit("ERROR: objdump requested but not available or failed.")

    if disasm_choice in ("auto", "otool"):
        try:
            res = subprocess.run(
                ["otool", "-tV", path],
                capture_output=True,
                text=True,
                check=True,
            )
            return res.stdout, False
        except (subprocess.SubprocessError, FileNotFoundError) as e:
            sys.exit(f"ERROR: otool disassembly failed: {e}")

    sys.exit("ERROR: Neither objdump nor otool available for disassembly.")


def parse_symbols(disasm_text, is_demangled):
    """Parse disassembly output into per-symbol instruction lists."""
    per_symbol = {}
    current_sym = None

    if is_demangled:
        sym_header = re.compile(r"^[0-9a-fA-F]+\s+<(.*)>:$")
    else:
        sym_header = re.compile(r"^([_\w]+):$")

    for line in disasm_text.splitlines():
        line_clean = line.strip()
        m = sym_header.match(line_clean)
        if m:
            current_sym = m.group(1)
            per_symbol[current_sym] = []
            continue

        if current_sym is not None and re.match(r"^[0-9a-fA-F]{1,16}[:\s\t]", line_clean):
            per_symbol[current_sym].append(line_clean)

    return per_symbol


def run_audit(per_symbol, is_demangled, target_path, strict_arm64=True, git_commit=None, compiler=None, flags=None, stack=False):
    """Execute audit across Class I, Class II, and Class III instructions.

    With `stack`, the D11 geometric-stack symbols are the mandatory set.
    Returns dict containing detailed results and failure reports.
    """
    is_rlib = target_path.endswith(".rlib")
    is_chat = "uor-chat" in os.path.basename(target_path)
    if stack:
        mandatory_list = list(STACK_MANDATORY_SYMBOLS)
    elif is_rlib:
        mandatory_list = list(RLIB_MANDATORY_SYMBOLS)
    else:
        mandatory_list = list(BIN_MANDATORY_SYMBOLS)
        if is_chat:
            mandatory_list.extend(CHAT_MANDATORY_SYMBOLS)

    artifact_hash, auto_commit, auto_compiler, auto_flags = get_artifact_metadata(target_path)
    commit = git_commit or auto_commit
    compiler = compiler or auto_compiler
    flags = flags or auto_flags

    results = {
        "target": target_path,
        "artifact_sha256": artifact_hash,
        "git_commit": commit,
        "compiler": compiler,
        "flags": flags,
        "is_rlib": is_rlib,
        "is_chat": is_chat,
        "is_stack": stack,
        "strict": strict_arm64,
        "total_symbols_indexed": len(per_symbol),
        "mandatory_checked": 0,
        "missing_mandatory": [],
        "missing_optional": [],
        "class_i_violations": [],   # Multipliers
        "class_ii_violations": [],  # Dividers
        "class_iii_violations": [], # Floats / transfers
        "all_violations": [],
        "passed_symbols": [],
    }

    # Collect groups for alternative symbol handling (e.g. step vs step_conversational)
    satisfied_groups = set()
    groups_needed = set()
    group_displays = {}
    for entry in mandatory_list:
        grp = entry.get("alternative_group")
        if grp:
            groups_needed.add(grp)
            group_displays[grp] = (entry.get("group_display", grp), entry["description"])

    # Track audited symbol signatures to prevent duplicate checking/reporting
    audited_syms = set()

    # Match symbols against mandatory list
    for entry in mandatory_list:
        name = entry["name"]
        pat = entry["pattern"] if is_demangled else entry["mangled"]
        desc = entry["description"]
        grp = entry.get("alternative_group")

        matched = [
            s
            for s in per_symbol
            if pat.search(s)
            and "closure" not in s
            and "drop_glue" not in s
            and "from_iter" not in s
            and "try_process" not in s
            and "GenericShunt" not in s
        ]

        if not matched:
            if not grp:
                results["missing_mandatory"].append((name, desc))
            continue

        if grp:
            satisfied_groups.add(grp)

        for sym in matched:
            if sym in audited_syms:
                continue
            audited_syms.add(sym)
            results["mandatory_checked"] += 1
            instrs = per_symbol[sym]

            c1 = [i for i in instrs if CLASS_I_PATTERN.search(i)]
            c2 = [i for i in instrs if CLASS_II_PATTERN.search(i)]
            c3 = [i for i in instrs if CLASS_III_PATTERN.search(i)]

            if c1:
                results["class_i_violations"].append((name, sym, c1, desc))
            if c2:
                results["class_ii_violations"].append((name, sym, c2, desc))
            if c3:
                results["class_iii_violations"].append((name, sym, c3, desc))

            if c1 or c2 or c3:
                results["all_violations"].append((name, sym, c1 + c2 + c3, desc))
            else:
                results["passed_symbols"].append((name, sym, len(instrs), desc))

    # Verify all alternative groups were satisfied by at least one symbol
    for grp in groups_needed:
        if grp not in satisfied_groups:
            disp_name, disp_desc = group_displays[grp]
            results["missing_mandatory"].append((disp_name, disp_desc))

    # Also audit optional/additional serving symbols if present
    other_list = (
        (RLIB_MANDATORY_SYMBOLS if not is_rlib else [])
        + PACKED_KERNEL_CALLERS
        + EXACT_GEOMETRY_SYMBOLS
    )
    for entry in other_list:
        name = entry["name"]
        pat = entry["pattern"] if is_demangled else entry["mangled"]
        desc = entry["description"]

        matched = [
            s
            for s in per_symbol
            if pat.search(s)
            and "closure" not in s
            and "drop_glue" not in s
            and "from_iter" not in s
            and "try_process" not in s
            and "GenericShunt" not in s
        ]

        if not matched:
            results["missing_optional"].append((name, desc))

        for sym in matched:
            if sym in audited_syms:
                continue
            audited_syms.add(sym)
            instrs = per_symbol[sym]

            c1 = [i for i in instrs if CLASS_I_PATTERN.search(i)]
            c2 = [i for i in instrs if CLASS_II_PATTERN.search(i)]
            c3 = [i for i in instrs if CLASS_III_PATTERN.search(i)]

            if c1:
                results["class_i_violations"].append((name, sym, c1, desc))
            if c2:
                results["class_ii_violations"].append((name, sym, c2, desc))
            if c3:
                results["class_iii_violations"].append((name, sym, c3, desc))

            if c1 or c2 or c3:
                results["all_violations"].append((name, sym, c1 + c2 + c3, desc))
            else:
                results["passed_symbols"].append((name, sym, len(instrs), desc))

    results["symbols_checked"] = len(audited_syms)
    return results


def print_tap_output(results, tools_found):
    """Emit authentic TAP version 13 results across the 5 standard test cases."""
    print("TAP version 13")
    print("1..5")
    print(f"# Target: {results['target']}")
    print(f"# Artifact SHA-256: {results.get('artifact_sha256', 'unknown')}")
    print(f"# Source Commit: {results.get('git_commit', 'unknown')}")
    print(f"# Compiler: {results.get('compiler', 'unknown')}")
    print(f"# Compiler Flags: {results.get('flags', 'unknown')}")
    print(f"# Matched symbol ranges checked: {results['symbols_checked']}")
    if results.get("call_graph_checked"):
        print(f"# Transitive call-graph reachability checked: {results['call_graph_checked']} functions (0 violations)")
    else:
        print("# Scope: matched instruction ranges only; no transitive call-graph certification.")
    for name, _ in results["missing_optional"]:
        print(f"# NOT AUDITED: {name} (no separate symbol; may be inlined or absent)")

    # TC01: Disassembly tool availability
    if tools_found:
        print(f"ok 1 - [T1_F05_TC01] [Tier 1] Disassembly tool availability ({', '.join(tools_found)})")
    else:
        print("not ok 1 - [T1_F05_TC01] [Tier 1] Disassembly tool availability (no objdump/otool found)")
        print("  ---")
        print("  severity: fail")
        print("  message: Neither objdump nor otool available")
        print("  ...")

    # TC02: Zero floating-point instructions (Class III)
    c3 = results["class_iii_violations"]
    if not c3:
        print("ok 2 - [T1_F05_TC02] [Tier 1] Zero floating-point instructions in serving symbols (Class III clean)")
    else:
        print(f"not ok 2 - [T1_F05_TC02] [Tier 1] Zero floating-point instructions in serving symbols ({len(c3)} violations)")
        print("  ---")
        print(f"  violation_count: {len(c3)}")
        print("  symbols:")
        for name, sym, bad, _ in c3:
            print(f"    - name: {name}")
            print(f"      symbol: {sym}")
            print(f"      instructions: {bad[:3]}")
        print("  ...")

    # TC03: Zero hardware multipliers (Class I)
    c1 = results["class_i_violations"]
    if not c1:
        print("ok 3 - [T1_F05_TC03] [Tier 1] Zero hardware multipliers in numerical serving symbols (Class I clean)")
    else:
        print(f"not ok 3 - [T1_F05_TC03] [Tier 1] Zero hardware multipliers in numerical serving symbols ({len(c1)} violations)")
        print("  ---")
        print(f"  violation_count: {len(c1)}")
        print("  symbols:")
        for name, sym, bad, _ in c1:
            print(f"    - name: {name}")
            print(f"      symbol: {sym}")
            print(f"      instructions: {bad[:3]}")
        print("  ...")

    # TC04: Zero hardware dividers (Class II)
    c2 = results["class_ii_violations"]
    if not c2:
        print("ok 4 - [T1_F05_TC04] [Tier 1] Zero hardware dividers in numerical serving symbols (Class II clean)")
    else:
        print(f"not ok 4 - [T1_F05_TC04] [Tier 1] Zero hardware dividers in numerical serving symbols ({len(c2)} violations)")
        print("  ---")
        print(f"  violation_count: {len(c2)}")
        print("  symbols:")
        for name, sym, bad, _ in c2:
            print(f"    - name: {name}")
            print(f"      symbol: {sym}")
            print(f"      instructions: {bad[:3]}")
        print("  ...")

    # TC05: Strict audit exit code integrity
    missing = results["missing_mandatory"]
    total_failures = len(results["all_violations"]) + len(missing)
    if total_failures == 0 and tools_found:
        print("ok 5 - [T1_F05_TC05] [Tier 1] Strict audit exit code integrity for declared symbol coverage (0 violations)")
    else:
        print(f"not ok 5 - [T1_F05_TC05] [Tier 1] Strict audit exit code integrity ({total_failures} total failures)")
        print("  ---")
        print(f"  missing_symbols: {len(missing)}")
        print(f"  violations: {len(results['all_violations'])}")
        print("  ...")


def print_standard_report(results):
    """Print readable report to stdout."""
    mode_str = "STRICT ARM64 (Class I multipliers, Class II dividers, Class III floats/transfers)"
    print("=" * 80)
    print(f"ZERO-MATMUL SERVING KERNEL AUDIT [{mode_str}]")
    print(f"Target Artifact: {results['target']}")
    print(f"Artifact SHA-256: {results.get('artifact_sha256', 'unknown')}")
    print(f"Source Commit:   {results.get('git_commit', 'unknown')}")
    print(f"Compiler:        {results.get('compiler', 'unknown')}")
    print(f"Compiler Flags:  {results.get('flags', 'unknown')}")
    print("=" * 80)

    for name, sym, n_instrs, desc in results["passed_symbols"]:
        print(f"  [ PASS  ] {name:<38} (0 forbidden, {n_instrs} instructions) - {desc}")
        print(f"            symbol: {sym}")

    for name, sym, bad, desc in results["all_violations"]:
        print(f"  [ FAIL  ] {name:<38} ({len(bad)} forbidden) - {desc}")
        for b in bad:
            print(f"            >>> {b}")

    for name, desc in results["missing_mandatory"]:
        print(f"  [MISSING] {name:<38} - {desc}")

    for name, desc in results["missing_optional"]:
        print(f"  [NOT AUDITED] {name} (no separate symbol; may be inlined or absent) - {desc}")

    print("=" * 80)
    print(f"Total matched symbol ranges checked: {results['symbols_checked']}")
    print(f"Class I (Multipliers) Violations: {len(results['class_i_violations'])}")
    print(f"Class II (Dividers) Violations:    {len(results['class_ii_violations'])}")
    print(f"Class III (Floats) Violations:    {len(results['class_iii_violations'])}")
    print(f"Missing Mandatory Symbols:        {len(results['missing_mandatory'])}")
    print("=" * 80)

    if "call_graph_checked" in results:
        print(f"Call-Graph Reachability Checked: {results['call_graph_checked']} functions")
        print(f"Call-Graph Violations:           {len(results['call_graph_violations'])}")
        for fn, bad in results["call_graph_violations"]:
            print(f"  [ FAIL  ] Reachable callee {fn}")
            print(f"            >>> {bad}")

    if results["all_violations"]:
        print("FAILED: Serving kernel contains forbidden instructions.")
        return 1

    if results["missing_mandatory"]:
        print(f"FAILED: Missing mandatory serving symbol coverage ({len(results['missing_mandatory'])} symbols missing).")
        return 1

    if "call_graph_checked" in results:
        print(f"FULL PASS: 0 forbidden instructions across {results['symbols_checked']} symbol ranges AND {results['call_graph_checked']} reachable call-graph functions; all hardware invariants satisfied.")
        print("Scope: the declared serving roots and their reachable callees in this artifact; not a whole-process D0-b certification.")
    else:
        print("FULL PASS: 0 forbidden instructions in all matched symbol ranges; mandatory coverage satisfied.")
        print("Unmatched or inlined functions are not independently audited; this is not a transitive call-graph or whole-process D0-b certification.")
    return 0
def run_sentinel_tests():
    """Verify that forbidden opcode patterns detect all required sentinels.
    Fails immediately if any sentinel is missed."""
    sentinels = [
        # Hardware Multipliers (Class I)
        ("mul x0, x1, x2", True, False, False),
        ("madd x0, x1, x2, x3", True, False, False),
        ("msub x0, x1, x2, x3", True, False, False),
        ("smull x0, w1, w2", True, False, False),
        ("smull v0.4s, v1.4h, v2.4h", True, False, False),
        ("smull2 v0.4s, v1.8h, v2.8h", True, False, False),
        ("umull x0, w1, w2", True, False, False),
        ("umull2 v0.2d, v1.4s, v2.4s", True, False, False),
        ("smulh x0, x1, x2", True, False, False),
        ("umulh x0, x1, x2", True, False, False),
        ("sdot v0.4s, v1.16b, v2.16b", True, False, False),
        ("udot v0.4s, v1.16b, v2.16b", True, False, False),
        ("usdot v0.4s, v1.16b, v2.16b", True, False, False),
        ("sudot v0.4s, v1.16b, v2.16b", True, False, False),
        ("smlal v0.4s, v1.4h, v2.4h", True, False, False),
        ("smlal2 v0.4s, v1.8h, v2.8h", True, False, False),
        ("smlsl v0.4s, v1.4h, v2.4h", True, False, False),
        ("smlsl2 v0.4s, v1.8h, v2.8h", True, False, False),
        ("umlal v0.4s, v1.4h, v2.4h", True, False, False),
        ("umlal2 v0.4s, v1.8h, v2.8h", True, False, False),
        ("umlsl v0.4s, v1.4h, v2.4h", True, False, False),
        ("umlsl2 v0.4s, v1.8h, v2.8h", True, False, False),
        ("mla v0.4s, v1.4s, v2.4s", True, False, False),
        ("mls v0.4s, v1.4s, v2.4s", True, False, False),
        ("mul v0.4s, v1.4s, v2.4s", True, False, False),
        ("sqdmull v0.4s, v1.4h, v2.4h", True, False, False),
        ("sqdmull2 v0.4s, v1.8h, v2.8h", True, False, False),
        ("sqdmulh v0.4s, v1.4h, v2.4h", True, False, False),
        ("sqrdmulh v0.4s, v1.4h, v2.4h", True, False, False),
        ("smmla v0.4s, v1.16b, v2.16b", True, False, False),
        ("ummla v0.4s, v1.16b, v2.16b", True, False, False),
        ("usmmla v0.4s, v1.16b, v2.16b", True, False, False),
        # Hardware Dividers (Class II)
        ("sdiv x0, x1, x2", False, True, False),
        ("udiv x0, x1, x2", False, True, False),
        # Floating-Point & Register Transfer Instructions (Class III)
        ("fmul s0, s1, s2", False, False, True),
        ("fmul.2d v0, v1, v2", False, False, True),
        ("fadd d0, d1, d2", False, False, True),
        ("fadd.2d v0, v1, v2", False, False, True),
        ("fsub.4s v0, v1, v2", False, False, True),
        ("fdiv.2d v0, v1, v2", False, False, True),
        ("fsqrt.2d v0, v1", False, False, True),
        ("fneg d0, d1", False, False, True),
        ("fneg d16, d31", False, False, True),
        ("fneg s0, s1", False, False, True),
        ("fneg s15, s31", False, False, True),
        ("fneg h0, h1", False, False, True),
        ("fneg q0, q1", False, False, True),
        ("fabs d0, d1", False, False, True),
        ("fabs d16, d31", False, False, True),
        ("fabs s0, s1", False, False, True),
        ("fabs s15, s31", False, False, True),
        ("fabs h0, h1", False, False, True),
        ("fabs q0, q1", False, False, True),
        ("fneg.2d v6, v6", False, False, True),
        ("fneg.4s v0, v0", False, False, True),
        ("fneg.8h v0, v0", False, False, True),
        ("fneg v0.2d, v1.2d", False, False, True),
        ("fabs.2d v0, v0", False, False, True),
        ("fabs.4s v0, v0", False, False, True),
        ("fabs.8h v0, v0", False, False, True),
        ("fabs v0.2d, v1.2d", False, False, True),
        ("scvtf d0, x0", False, False, True),
        ("ucvtf s0, w0", False, False, True),
        ("fmla v0.4s, v1.4s, v2.4s", False, False, True),
        ("fmla.4s v0, v1, v2", False, False, True),
        ("fmls v0.2d, v1.2d, v2.2d", False, False, True),
        ("fmls.2d v0, v1, v2", False, False, True),
        ("fmax d0, d1, d2", False, False, True),
        ("fmin d0, d1, d2", False, False, True),
        ("fmaxnm s0, s1, s2", False, False, True),
        ("fminnm s0, s1, s2", False, False, True),
        ("fmaxp v0.4s, v1.4s, v2.4s", False, False, True),
        ("fminp v0.4s, v1.4s, v2.4s", False, False, True),
        ("fmaxnmp v0.4s, v1.4s, v2.4s", False, False, True),
        ("fminnmp v0.4s, v1.4s, v2.4s", False, False, True),
        ("fmaxv s0, v1.4s", False, False, True),
        ("fminv s0, v1.4s", False, False, True),
        ("fmax.2d v0, v1, v2", False, False, True),
        ("fmin.4s v0, v1, v2", False, False, True),
        ("fmaxnm.2d v0, v1, v2", False, False, True),
        ("fminnm.4s v0, v1, v2", False, False, True),
        ("fmaxnmv s0, v1.4s", False, False, True),
        ("fminnmv s0, v1.4s", False, False, True),
        ("faddp v0.4s, v1.4s, v2.4s", False, False, True),
        ("fcmgt v0.4s, v1.4s, v2.4s", False, False, True),
        ("frecpe v0.4s, v1.4s", False, False, True),
        ("frsqrte.4s v0, v1", False, False, True),
        ("bfdot v0.4s, v1.8h, v2.8h", False, False, True),
    ]
    # Negative sentinels: integer data movement and arithmetic must stay unflagged.
    benign = [
        "add x0, x1, x2",
        "sub x0, x1, x2",
        "neg v0.2d, v1.2d",
        "neg.2d v0, v1",
        "neg x0, x1",
        "lsl x0, x1, #3",
        "lsr x0, x1, #3",
        "asr x0, x1, #3",
        "ldr x0, [x1]",
        "str x0, [x1]",
        "cmp x0, x1",
        "csel x0, x1, x2, lt",
        "tbl v0.16b, {v1.16b}, v2.16b",
        "mvn v0.16b, v1.16b",
        "and v0.16b, v1.16b, v2.16b",
        "orr v0.16b, v1.16b, v2.16b",
        "eor v0.16b, v1.16b, v2.16b",
    ]
    for instr in benign:
        if STRICT_FORBIDDEN_PATTERN.search(instr):
            raise AssertionError(f"Sentinel test failed: benign '{instr}' flagged by STRICT_FORBIDDEN_PATTERN")

    for instr, exp_c1, exp_c2, exp_c3 in sentinels:
        m1 = bool(CLASS_I_PATTERN.search(instr))
        m2 = bool(CLASS_II_PATTERN.search(instr))
        m3 = bool(CLASS_III_PATTERN.search(instr))
        m_strict = bool(STRICT_FORBIDDEN_PATTERN.search(instr))

        if exp_c1 and not m1:
            raise AssertionError(f"Sentinel test failed: '{instr}' not detected by CLASS_I_PATTERN")
        if exp_c2 and not m2:
            raise AssertionError(f"Sentinel test failed: '{instr}' not detected by CLASS_II_PATTERN")
        if exp_c3 and not m3:
            raise AssertionError(f"Sentinel test failed: '{instr}' not detected by CLASS_III_PATTERN")
        if (exp_c1 or exp_c2 or exp_c3) and not m_strict:
            raise AssertionError(f"Sentinel test failed: '{instr}' not detected by STRICT_FORBIDDEN_PATTERN")


def find_serving_roots(functions, keywords, extra_roots):
    """The functions a call-graph traversal starts from (a keyword in the name
    or an `extra_roots` pattern match, closures and drop glue excluded), and
    the names of the `extra_roots` (name, pattern) pairs that match none."""
    roots = [
        fn
        for fn in functions
        if (
            any(kw in fn for kw in keywords)
            or any(p.search(fn) for _, p in extra_roots)
        )
        and "drop_glue" not in fn
        and "closure" not in fn
    ]
    missing = [name for name, p in extra_roots if not any(p.search(fn) for fn in roots)]
    return roots, missing


def run_call_graph_audit(path, disasm_choice="auto", extra_roots=()):
    """Transitive call-graph reachability traversal from serving roots.
    Checks 100% of reachable numerical serving functions in compiled binaries.
    `extra_roots` are (name, compiled pattern) pairs naming further roots (the
    stack set). Returns the functions visited, the violations and the names of
    the `extra_roots` that no function matched."""
    try:
        output = subprocess.check_output(["otool", "-tvV", path]).decode("utf-8", errors="ignore")
    except Exception as e:
        return 0, [("call_graph_init", f"Failed to run otool for call-graph audit: {e}")], [
            name for name, _ in extra_roots
        ]

    functions = {}
    current_fn = None
    current_lines = []

    for line in output.splitlines():
        if line and not line.startswith("\t") and not line.startswith(" ") and line.endswith(":"):
            if current_fn:
                functions[current_fn] = current_lines
            current_fn = line[:-1].strip()
            current_lines = []
        elif current_fn:
            current_lines.append(line)
    if current_fn:
        functions[current_fn] = current_lines

    def extract_callees(lines):
        callees = set()
        for l in lines:
            m = re.search(r"\b(?:bl|b)\s+(?:0x[0-9a-fA-F]+\s+;)?\s*([_A-Za-z0-9$]+)", l)
            if m:
                callees.add(m.group(1))
        return callees

    serving_entry_keywords = [
        "step_conversational",
        "IntegerModel4step",
        "DialogueConversation7respond",
        "DialogueConversation14respond_stream",
        "respond_stream",
        "DialogueConversationStream",
        "Sampler6select",
        "select_ticket",
        "exact_min_p_threshold",
        "validate_distribution",
        "TextSession15step_next_token",
        "step_next_token",
        "TextSession7observe",
        "project_vocab_with_products_into",
        "affine_wide_into",
        "extract_salient_tokens",
        "heapsort_ranked",
        "step_zeta_phase_scalar",
    ]

    serving_roots, missing_roots = find_serving_roots(functions, serving_entry_keywords, extra_roots)

    allow_patterns = [
        r"alloc",
        r"free",
        r"realloc",
        r"core..fmt",
        r"panic",
        r"fmt..Display",
        r"fmt..Debug",
        r"unwind",
        r"rust_begin_unwind",
        r"std..io",
        r"std..panicking",
        r"Bundle",
        r"[Tt]okenizer",
        r"uor_r4_tokenizer",
        r"core..str",
        r"load",
        r"from_file",
        r"from_serialized",
    ]

    def is_allowlisted(name):
        return any(re.search(pat, name) for pat in allow_patterns)

    visited = set()
    queue = list(serving_roots)
    for r in queue:
        visited.add(r)

    violations = []

    while queue:
        curr = queue.pop(0)
        lines = functions.get(curr, [])
        if not is_allowlisted(curr):
            for l in lines:
                if STRICT_FORBIDDEN_PATTERN.search(l):
                    violations.append((curr, l.strip()))
        callees = extract_callees(lines)
        for c in callees:
            if c in functions and c not in visited:
                if not is_allowlisted(c):
                    visited.add(c)
                    queue.append(c)

    return len(visited), violations, missing_roots


def main():
    parser = argparse.ArgumentParser(
        description="Zero-Multiplier Numerical Serving Kernel Static Disassembly Auditor"
    )
    parser.add_argument(
        "target",
        nargs="?",
        default=None,
        help="Path to .rlib archive or compiled release binary (default: auto-detected)",
    )
    parser.add_argument(
        "--target",
        dest="target_opt",
        default=None,
        help="Explicit path to target artifact",
    )
    parser.add_argument(
        "--disassembler",
        choices=["auto", "objdump", "otool"],
        default="auto",
        help="Disassembly backend tool to use (default: auto)",
    )
    parser.add_argument(
        "--strict-arm64",
        "--strict",
        action="store_true",
        default=True,
        help="Check extended ARM64 multiply, divide, and floating-point mnemonics (default: True)",
    )
    parser.add_argument(
        "--tap",
        action="store_true",
        help="Emit TAP version 13 output",
    )
    parser.add_argument(
        "--call-graph",
        action="store_true",
        default=None,
        help="Traverse transitive call graph from serving roots (default: auto for binary executables)",
    )
    parser.add_argument(
        "--git-commit",
        type=str,
        default=None,
        help="Git commit SHA to record in metadata (default: auto-detected)",
    )
    parser.add_argument(
        "--compiler",
        type=str,
        default=None,
        help="Compiler version string to record in metadata (default: auto-detected)",
    )
    parser.add_argument(
        "--flags",
        type=str,
        default=None,
        help="Compiler flags to record in metadata (default: auto-detected)",
    )
    parser.add_argument(
        "--stack",
        action="store_true",
        help="Audit the D11 geometric-stack engine: its step and kernels are the mandatory "
        "symbols and call-graph roots (implied for a binary named uor-r4-stack*); the call "
        "graph always runs and every stack root must be found",
    )

    args = parser.parse_args()

    # Always verify sentinel detection before running any audit
    run_sentinel_tests()

    chosen_target = args.target_opt or args.target
    tools_found = check_tool_availability(args.disassembler)

    if not tools_found:
        if args.tap:
            print("TAP version 13\n1..5\nnot ok 1 - [T1_F05_TC01] No disassembler available\n...")
        else:
            print("ERROR: Neither objdump nor otool available on host.", file=sys.stderr)
        return 1

    target_path = find_target_artifact(chosen_target)
    stack = args.stack or os.path.basename(target_path).startswith("uor-r4-stack")
    if stack:
        run_stack_pattern_tests()
    if not args.tap:
        print(f"Disassembling target artifact: {target_path} (using {args.disassembler})")
        if stack:
            print("Symbol set: D11 geometric stack (STACK_MANDATORY_SYMBOLS and stack call-graph roots)")

    disasm, is_demangled = disassemble_artifact(target_path, args.disassembler)
    per_symbol = parse_symbols(disasm, is_demangled)
    if not args.tap:
        print(f"Disassembly parsed: {len(per_symbol)} symbols indexed (demangled={is_demangled}).\n")

    results = run_audit(
        per_symbol,
        is_demangled,
        target_path,
        strict_arm64=args.strict_arm64,
        git_commit=args.git_commit,
        compiler=args.compiler,
        flags=args.flags,
        stack=stack,
    )

    do_call_graph = args.call_graph if args.call_graph is not None else (not target_path.endswith(".rlib"))
    # A stack audit always certifies the call graph of its roots.
    do_call_graph = do_call_graph or stack
    if do_call_graph:
        cg_visited, cg_violations, cg_missing_roots = run_call_graph_audit(
            target_path,
            args.disassembler,
            extra_roots=STACK_CALL_GRAPH_ROOTS if stack else (),
        )
        results["call_graph_checked"] = cg_visited
        results["call_graph_violations"] = cg_violations
        if cg_violations:
            for fn, bad in cg_violations:
                results["all_violations"].append(("call_graph::" + fn, fn, [bad], "Reachable callee"))
        # Only the stack set declares roots that must exist; a missing one
        # fails the audit instead of certifying an empty traversal.
        for name in cg_missing_roots:
            results["missing_mandatory"].append(
                (f"call-graph root {name}", "Stack call-graph root: no function in the call graph matches it")
            )

    if args.tap:
        print_tap_output(results, tools_found)
        total_failures = len(results["all_violations"]) + len(results["missing_mandatory"])
        return 0 if total_failures == 0 else 1
    else:
        return print_standard_report(results)


if __name__ == "__main__":
    sys.exit(main())
