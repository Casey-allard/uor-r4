//! Clean-room reader and integer-only evaluator for discrete SpiralCore/FBS
//! checkpoints exported in the `UORLUT01` (`uor-r4.lut-discrete/1`) container.
//!
//! Serving arithmetic uses only signed-axis permutation, $L_1$ and Hamming root
//! comparisons, ternary gate accumulation in `{-1, 0, 1}`, and exact `snap_n`
//! orbit-root projection by repeated addition over the 16-root `Cl(0,6)` anchor
//! orbit. No exponent or squashing table, no floating-point value, and no dense
//! matrix product exist on this path.
#![forbid(unsafe_code)]

use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{invalid, Result};

/// Container magic (`UORLUT01`).
pub const MAGIC: &[u8; 8] = b"UORLUT01";
/// Header schema of a discrete SpiralCore/FBS `UORLUT01` artifact.
pub const DISCRETE_SCHEMA: &str = "uor-r4.lut-discrete/1";
/// Source snapshot schema recorded by `uor-r4-discrete`.
pub const SNAPSHOT_SCHEMA: &str = "uor-r4.discrete-spiralcore-fbs/2";

const ALIGN: usize = 64;
const MAX_HEADER: u64 = 64 << 20;
const MAX_CONTEXT: usize = 243;
const MAX_KEPT: usize = 8;
const MAX_SWEEPS: u32 = 5;
const EDGE_TYPES: usize = 5;

const AXES: [u8; 512] = [
    0, 1, 2, 3, 4, 5, 6, 7, 12, 10, 1, 14, 0, 7, 3, 13, 15, 11, 6, 1, 5, 12, 10, 0, 2, 12, 8, 13,
    1, 3, 15, 6, 14, 13, 15, 4, 11, 1, 0, 2, 5, 14, 11, 2, 7, 8, 1, 12, 13, 14, 11, 2, 15, 0, 1, 4,
    9, 0, 12, 7, 2, 6, 13, 11, 3, 7, 13, 8, 14, 2, 4, 9, 15, 3, 14, 9, 5, 12, 2, 0, 14, 5, 15, 12,
    3, 9, 0, 2, 10, 12, 0, 13, 1, 3, 7, 14, 4, 10, 1, 14, 8, 15, 3, 5, 15, 3, 6, 9, 13, 4, 10, 0,
    11, 15, 13, 0, 14, 2, 4, 1, 9, 0, 4, 7, 10, 14, 5, 11, 8, 9, 10, 11, 12, 13, 14, 15, 7, 3, 14,
    9, 13, 4, 2, 8, 10, 4, 0, 5, 9, 11, 7, 14, 6, 5, 7, 12, 3, 9, 8, 10, 13, 6, 3, 10, 15, 0, 9, 4,
    11, 7, 5, 8, 14, 2, 12, 1, 9, 8, 12, 15, 10, 14, 13, 11, 0, 9, 10, 11, 4, 5, 14, 7, 5, 14, 3,
    2, 15, 0, 9, 12, 6, 5, 15, 12, 11, 1, 0, 10, 2, 12, 0, 13, 9, 11, 7, 6, 5, 6, 3, 10, 7, 8, 9,
    12, 3, 15, 13, 0, 6, 10, 4, 9, 1, 0, 4, 7, 2, 6, 5, 3, 8, 1, 2, 3, 12, 13, 6, 15, 8, 1, 10, 3,
    4, 5, 14, 15, 9, 8, 4, 15, 2, 6, 5, 11, 3, 15, 5, 0, 14, 2, 12, 9, 1, 8, 4, 15, 10, 14, 5, 3,
    6, 13, 7, 4, 11, 1, 8, 10, 13, 6, 11, 10, 7, 8, 1, 4, 14, 13, 7, 4, 3, 9, 8, 2, 0, 9, 2, 11,
    12, 13, 6, 7, 1, 0, 12, 7, 10, 14, 13, 3, 4, 2, 1, 6, 0, 7, 3, 5, 11, 15, 5, 0, 6, 10, 12, 1,
    2, 4, 8, 5, 9, 11, 15, 6, 7, 11, 14, 1, 5, 12, 2, 8, 10, 4, 8, 5, 1, 3, 15, 14, 11, 7, 13, 8,
    6, 10, 4, 1, 12, 10, 9, 14, 8, 15, 11, 13, 7, 11, 6, 1, 13, 4, 10, 8, 12, 2, 9, 6, 0, 7, 11,
    13, 3, 7, 5, 8, 6, 10, 12, 9, 1, 8, 12, 15, 2, 6, 13, 3, 4, 2, 9, 6, 8, 15, 11, 5, 12, 2, 1, 6,
    8, 15, 3, 13, 10, 12, 8, 13, 9, 11, 15, 14, 6, 13, 15, 4, 3, 9, 0, 10, 4, 10, 9, 14, 0, 7, 11,
    5, 2, 4, 0, 5, 1, 3, 7, 6, 8, 9, 2, 11, 4, 5, 6, 15, 14, 5, 7, 12, 11, 1, 8, 2, 0, 1, 10, 3,
    12, 13, 14, 7, 13, 14, 3, 2, 7, 8, 9, 4, 5, 6, 11, 10, 15, 0, 1, 12, 15, 11, 14, 1, 13, 4, 2,
    0, 7, 3, 6, 9, 5, 12, 10, 8,
];

const COMPOSE: [u8; 4096] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 1, 16, 6, 7, 8, 9, 17, 18, 19, 20, 21,
    22, 23, 24, 25, 26, 51, 27, 34, 41, 47, 35, 42, 48, 43, 49, 50, 2, 10, 11, 12, 52, 53, 54, 3,
    28, 13, 14, 55, 56, 57, 4, 29, 36, 15, 58, 59, 5, 30, 37, 44, 0, 38, 39, 45, 31, 32, 46, 33,
    40, 62, 63, 61, 60, 2, 27, 16, 10, 11, 12, 1, 28, 29, 30, 18, 19, 20, 31, 32, 33, 17, 0, 35,
    42, 48, 7, 8, 9, 55, 56, 58, 51, 34, 41, 47, 43, 49, 50, 21, 3, 52, 53, 13, 14, 60, 22, 4, 38,
    54, 15, 61, 23, 5, 39, 45, 6, 24, 25, 26, 36, 37, 63, 44, 62, 46, 40, 57, 59, 3, 34, 35, 16,
    13, 14, 21, 1, 36, 37, 2, 38, 39, 19, 20, 40, 18, 10, 0, 43, 49, 27, 52, 53, 8, 9, 59, 28, 6,
    55, 56, 11, 12, 61, 51, 17, 41, 47, 42, 48, 50, 24, 31, 4, 57, 60, 15, 25, 32, 5, 46, 7, 29,
    30, 62, 22, 23, 26, 63, 44, 33, 45, 58, 54, 4, 41, 42, 43, 16, 15, 22, 24, 1, 44, 31, 2, 45, 3,
    46, 20, 19, 11, 13, 0, 50, 55, 27, 54, 34, 57, 9, 29, 52, 6, 58, 35, 60, 12, 36, 38, 7, 59, 10,
    61, 14, 51, 17, 18, 47, 48, 49, 26, 33, 40, 5, 8, 21, 63, 30, 28, 62, 37, 23, 25, 39, 32, 53,
    56, 5, 47, 48, 49, 50, 16, 23, 25, 26, 1, 32, 33, 2, 40, 3, 4, 20, 12, 14, 15, 0, 56, 58, 27,
    59, 34, 41, 30, 53, 54, 6, 61, 35, 42, 37, 39, 57, 7, 60, 10, 43, 44, 45, 46, 8, 11, 13, 51,
    17, 18, 19, 9, 62, 21, 22, 63, 28, 24, 29, 36, 31, 38, 55, 52, 6, 2, 51, 21, 22, 23, 16, 10,
    11, 12, 34, 41, 47, 52, 53, 54, 27, 1, 28, 29, 30, 18, 19, 20, 31, 32, 33, 0, 3, 4, 5, 36, 37,
    44, 35, 7, 38, 39, 24, 25, 62, 42, 8, 55, 45, 26, 63, 48, 9, 56, 58, 17, 43, 49, 50, 13, 14,
    60, 15, 61, 59, 57, 46, 40, 7, 3, 28, 51, 24, 25, 35, 16, 13, 14, 6, 55, 56, 41, 47, 57, 34,
    21, 1, 36, 37, 2, 38, 39, 19, 20, 40, 10, 17, 31, 32, 22, 23, 63, 0, 27, 4, 5, 29, 30, 44, 43,
    52, 8, 46, 62, 26, 49, 53, 9, 59, 18, 11, 12, 61, 42, 48, 50, 60, 15, 54, 58, 33, 45, 8, 4, 29,
    36, 51, 26, 42, 43, 16, 15, 52, 6, 58, 7, 59, 47, 41, 22, 24, 1, 44, 31, 2, 45, 3, 46, 20, 11,
    38, 17, 33, 28, 62, 23, 13, 55, 18, 40, 21, 63, 25, 0, 27, 34, 5, 30, 37, 50, 54, 57, 9, 19,
    35, 60, 12, 10, 61, 14, 48, 49, 56, 53, 39, 32, 9, 5, 30, 37, 44, 51, 48, 49, 50, 16, 53, 54,
    6, 57, 7, 8, 47, 23, 25, 26, 1, 32, 33, 2, 40, 3, 4, 12, 39, 45, 17, 63, 28, 29, 14, 56, 46,
    18, 62, 21, 36, 15, 58, 59, 19, 22, 24, 0, 27, 34, 41, 20, 61, 35, 42, 60, 10, 43, 11, 13, 52,
    55, 31, 38, 10, 21, 3, 17, 31, 32, 7, 27, 52, 53, 16, 13, 14, 42, 48, 60, 35, 18, 2, 38, 39,
    51, 24, 25, 29, 30, 62, 34, 1, 36, 37, 19, 20, 40, 6, 0, 22, 23, 4, 5, 45, 55, 43, 11, 63, 46,
    33, 56, 49, 12, 61, 28, 41, 47, 57, 8, 9, 58, 59, 54, 50, 15, 44, 26, 11, 22, 4, 38, 17, 33, 8,
    55, 27, 54, 43, 16, 15, 10, 61, 48, 42, 19, 31, 2, 45, 36, 51, 26, 21, 63, 30, 41, 24, 1, 44,
    3, 46, 20, 52, 13, 28, 62, 18, 40, 32, 6, 0, 35, 23, 5, 39, 58, 50, 60, 12, 29, 7, 59, 47, 34,
    57, 53, 9, 56, 14, 49, 25, 37, 12, 23, 5, 39, 45, 17, 9, 56, 58, 27, 49, 50, 16, 60, 10, 11,
    48, 20, 32, 33, 2, 37, 44, 51, 62, 21, 22, 47, 25, 26, 1, 40, 3, 4, 53, 14, 63, 28, 46, 18, 38,
    54, 15, 61, 29, 19, 31, 6, 0, 35, 42, 30, 57, 7, 8, 59, 34, 55, 41, 52, 43, 13, 36, 24, 13, 24,
    31, 4, 18, 40, 52, 8, 34, 57, 11, 35, 60, 16, 15, 49, 43, 38, 19, 3, 46, 22, 28, 62, 51, 26,
    37, 55, 29, 21, 63, 17, 33, 39, 41, 42, 1, 44, 2, 45, 20, 7, 10, 0, 25, 32, 5, 59, 61, 50, 14,
    36, 27, 54, 56, 6, 58, 47, 53, 9, 48, 12, 30, 23, 14, 25, 32, 5, 46, 18, 53, 9, 59, 34, 12, 61,
    35, 50, 16, 13, 49, 39, 20, 40, 3, 23, 63, 28, 44, 51, 24, 56, 30, 62, 21, 45, 17, 31, 47, 48,
    26, 1, 33, 2, 4, 57, 60, 15, 36, 38, 19, 7, 10, 0, 43, 37, 58, 27, 52, 54, 6, 8, 55, 41, 11,
    42, 22, 29, 15, 26, 33, 40, 5, 19, 54, 57, 9, 41, 60, 12, 42, 14, 43, 16, 50, 45, 46, 20, 4,
    62, 23, 29, 25, 36, 51, 58, 63, 30, 22, 32, 38, 17, 59, 61, 37, 24, 39, 31, 18, 47, 48, 49, 1,
    2, 3, 8, 11, 13, 0, 44, 53, 55, 27, 56, 52, 34, 6, 7, 35, 10, 28, 21, 16, 51, 17, 18, 19, 20,
    27, 34, 41, 47, 35, 42, 48, 43, 49, 50, 0, 2, 3, 4, 5, 28, 29, 30, 36, 37, 44, 6, 21, 22, 23,
    38, 39, 45, 7, 10, 24, 25, 31, 32, 46, 8, 11, 13, 26, 33, 40, 9, 12, 14, 15, 1, 55, 56, 58, 52,
    53, 59, 54, 57, 61, 60, 63, 62, 17, 6, 0, 35, 42, 48, 51, 21, 22, 23, 3, 4, 5, 38, 39, 45, 2,
    16, 10, 11, 12, 34, 41, 47, 52, 53, 54, 1, 7, 8, 9, 13, 14, 15, 28, 18, 55, 56, 43, 49, 61, 29,
    19, 31, 58, 50, 60, 30, 20, 32, 33, 27, 36, 37, 44, 24, 25, 62, 26, 63, 40, 46, 59, 57, 18, 7,
    10, 0, 43, 49, 28, 51, 24, 25, 17, 31, 32, 4, 5, 46, 3, 35, 16, 13, 14, 6, 55, 56, 41, 47, 57,
    21, 27, 52, 53, 42, 48, 60, 1, 2, 8, 9, 11, 12, 15, 36, 38, 19, 59, 61, 50, 37, 39, 20, 40, 34,
    22, 23, 63, 29, 30, 44, 62, 26, 45, 33, 54, 58, 19, 8, 11, 13, 0, 50, 29, 36, 51, 26, 38, 17,
    33, 18, 40, 5, 4, 42, 43, 16, 15, 52, 6, 58, 7, 59, 47, 22, 55, 27, 54, 10, 61, 48, 24, 31, 34,
    57, 35, 60, 49, 1, 2, 3, 9, 12, 14, 44, 45, 46, 20, 41, 28, 62, 23, 21, 63, 25, 30, 37, 32, 39,
    56, 53, 20, 9, 12, 14, 15, 0, 30, 37, 44, 51, 39, 45, 17, 46, 18, 19, 5, 48, 49, 50, 16, 53,
    54, 6, 57, 7, 8, 23, 56, 58, 27, 60, 10, 11, 25, 32, 59, 34, 61, 35, 13, 26, 33, 40, 41, 42,
    43, 1, 2, 3, 4, 47, 63, 28, 29, 62, 21, 36, 22, 24, 38, 31, 52, 55, 21, 35, 7, 27, 52, 53, 18,
    2, 38, 39, 51, 24, 25, 29, 30, 62, 28, 34, 6, 55, 56, 0, 43, 49, 11, 12, 61, 3, 16, 13, 14, 41,
    47, 57, 17, 1, 42, 48, 8, 9, 58, 31, 36, 22, 60, 59, 54, 32, 37, 23, 63, 10, 4, 5, 46, 19, 20,
    33, 40, 45, 44, 26, 15, 50, 22, 42, 8, 55, 27, 54, 19, 31, 2, 45, 36, 51, 26, 21, 63, 30, 29,
    41, 52, 6, 58, 13, 0, 50, 35, 60, 12, 4, 43, 16, 15, 7, 59, 47, 38, 24, 10, 61, 34, 57, 53, 17,
    1, 28, 48, 9, 56, 33, 44, 62, 23, 11, 18, 40, 5, 3, 46, 39, 20, 32, 25, 37, 49, 14, 23, 48, 9,
    56, 58, 27, 20, 32, 33, 2, 37, 44, 51, 62, 21, 22, 30, 47, 53, 54, 6, 14, 15, 0, 61, 35, 42, 5,
    49, 50, 16, 57, 7, 8, 39, 25, 60, 10, 59, 34, 55, 45, 26, 63, 11, 41, 52, 17, 1, 28, 29, 12,
    46, 18, 19, 40, 3, 31, 4, 38, 36, 24, 13, 43, 24, 43, 52, 8, 34, 57, 38, 19, 3, 46, 22, 28, 62,
    51, 26, 37, 36, 55, 41, 7, 59, 42, 10, 61, 0, 50, 14, 31, 11, 35, 60, 27, 54, 56, 4, 29, 16,
    15, 6, 58, 47, 18, 21, 1, 49, 53, 9, 40, 63, 44, 25, 13, 2, 45, 32, 17, 33, 5, 39, 20, 30, 23,
    12, 48, 25, 49, 53, 9, 59, 34, 39, 20, 40, 3, 23, 63, 28, 44, 51, 24, 37, 56, 47, 57, 7, 48,
    60, 10, 15, 0, 43, 32, 12, 61, 35, 58, 27, 52, 5, 30, 50, 16, 54, 6, 8, 46, 62, 26, 13, 55, 41,
    18, 21, 1, 36, 14, 33, 2, 38, 45, 17, 19, 31, 4, 22, 29, 42, 11, 26, 50, 54, 57, 9, 41, 45, 46,
    20, 4, 62, 23, 29, 25, 36, 51, 44, 58, 59, 47, 8, 61, 48, 11, 49, 13, 0, 33, 60, 12, 42, 53,
    55, 27, 40, 63, 14, 43, 56, 52, 34, 5, 30, 37, 16, 6, 7, 19, 22, 24, 1, 15, 39, 31, 2, 32, 38,
    3, 17, 18, 28, 21, 10, 35, 27, 17, 1, 28, 29, 30, 0, 35, 42, 48, 7, 8, 9, 55, 56, 58, 6, 51,
    21, 22, 23, 3, 4, 5, 38, 39, 45, 16, 18, 19, 20, 24, 25, 26, 10, 34, 31, 32, 36, 37, 63, 11,
    41, 52, 33, 44, 62, 12, 47, 53, 54, 2, 13, 14, 15, 43, 49, 61, 50, 60, 57, 59, 40, 46, 28, 10,
    34, 6, 55, 56, 3, 17, 31, 32, 1, 36, 37, 22, 23, 63, 21, 7, 27, 52, 53, 16, 13, 14, 42, 48, 60,
    18, 0, 43, 49, 8, 9, 59, 2, 51, 11, 12, 41, 47, 54, 38, 24, 29, 61, 57, 58, 39, 25, 30, 62, 35,
    19, 20, 40, 4, 5, 45, 46, 33, 26, 44, 50, 15, 29, 11, 41, 52, 6, 58, 4, 38, 17, 33, 24, 1, 44,
    28, 62, 23, 22, 8, 55, 27, 54, 43, 16, 15, 10, 61, 48, 19, 13, 0, 50, 34, 57, 9, 31, 36, 35,
    60, 7, 59, 56, 2, 51, 21, 12, 47, 53, 45, 26, 63, 30, 42, 3, 46, 20, 18, 40, 32, 5, 39, 37, 25,
    14, 49, 30, 12, 47, 53, 54, 6, 5, 39, 45, 17, 25, 26, 1, 63, 28, 29, 23, 9, 56, 58, 27, 49, 50,
    16, 60, 10, 11, 20, 14, 15, 0, 59, 34, 41, 32, 37, 61, 35, 57, 7, 52, 33, 44, 62, 42, 8, 55, 2,
    51, 21, 22, 48, 40, 3, 4, 46, 18, 38, 19, 31, 24, 36, 43, 13, 31, 55, 43, 11, 35, 60, 24, 29,
    21, 63, 19, 3, 46, 17, 33, 39, 38, 13, 42, 10, 61, 8, 34, 57, 6, 58, 53, 36, 41, 7, 59, 0, 50,
    14, 22, 4, 27, 54, 16, 15, 48, 28, 18, 2, 56, 49, 12, 62, 40, 45, 32, 52, 51, 26, 37, 1, 44,
    23, 25, 30, 5, 20, 47, 9, 32, 56, 49, 12, 61, 35, 25, 30, 62, 21, 20, 40, 3, 45, 17, 31, 39,
    14, 48, 60, 10, 9, 59, 34, 54, 6, 55, 37, 47, 57, 7, 15, 0, 43, 23, 5, 58, 27, 50, 16, 11, 63,
    46, 33, 52, 13, 42, 28, 18, 2, 38, 53, 44, 51, 24, 26, 1, 29, 36, 22, 19, 4, 8, 41, 33, 58, 50,
    60, 12, 42, 26, 63, 30, 22, 46, 20, 4, 32, 38, 17, 45, 15, 61, 48, 11, 57, 9, 41, 56, 52, 6,
    44, 59, 47, 8, 49, 13, 0, 62, 40, 53, 55, 14, 43, 35, 23, 5, 39, 27, 16, 10, 29, 19, 31, 2, 54,
    25, 36, 51, 37, 24, 21, 1, 28, 3, 18, 34, 7, 34, 18, 21, 1, 36, 37, 10, 0, 43, 49, 27, 52, 53,
    8, 9, 59, 7, 28, 51, 24, 25, 17, 31, 32, 4, 5, 46, 35, 2, 38, 39, 29, 30, 62, 16, 6, 19, 20,
    22, 23, 26, 13, 55, 41, 40, 63, 44, 14, 56, 47, 57, 3, 42, 48, 60, 11, 12, 15, 61, 50, 58, 54,
    45, 33, 35, 28, 18, 2, 38, 39, 34, 6, 55, 56, 0, 43, 49, 11, 12, 61, 10, 3, 17, 31, 32, 1, 36,
    37, 22, 23, 63, 7, 51, 24, 25, 4, 5, 46, 27, 16, 29, 30, 19, 20, 33, 52, 13, 42, 62, 40, 45,
    53, 14, 48, 60, 21, 8, 9, 59, 41, 47, 54, 57, 58, 15, 50, 26, 44, 36, 13, 55, 41, 7, 59, 31, 4,
    18, 40, 29, 21, 63, 1, 44, 25, 24, 52, 8, 34, 57, 11, 35, 60, 16, 15, 49, 38, 42, 10, 61, 6,
    58, 53, 19, 22, 0, 50, 27, 54, 9, 3, 28, 51, 14, 56, 47, 46, 62, 26, 37, 43, 17, 33, 39, 2, 45,
    20, 32, 5, 23, 30, 48, 12, 37, 14, 56, 47, 57, 7, 32, 5, 46, 18, 30, 62, 21, 26, 1, 36, 25, 53,
    9, 59, 34, 12, 61, 35, 50, 16, 13, 39, 48, 60, 10, 54, 6, 55, 20, 23, 15, 0, 58, 27, 41, 40,
    63, 44, 43, 52, 8, 3, 28, 51, 24, 49, 45, 17, 31, 33, 2, 4, 38, 19, 29, 22, 11, 42, 38, 52, 13,
    42, 10, 61, 36, 22, 28, 62, 4, 18, 40, 2, 45, 32, 31, 43, 11, 35, 60, 41, 7, 59, 27, 54, 56,
    24, 8, 34, 57, 16, 15, 49, 29, 19, 6, 58, 0, 50, 12, 21, 3, 17, 53, 14, 48, 63, 46, 33, 39, 55,
    1, 44, 25, 51, 26, 30, 37, 23, 20, 5, 9, 47, 39, 53, 14, 48, 60, 10, 37, 23, 63, 28, 5, 46, 18,
    33, 2, 38, 32, 49, 12, 61, 35, 47, 57, 7, 58, 27, 52, 25, 9, 59, 34, 50, 16, 13, 30, 20, 54, 6,
    15, 0, 42, 62, 40, 45, 55, 43, 11, 21, 3, 17, 31, 56, 26, 1, 36, 44, 51, 22, 24, 29, 4, 19, 41,
    8, 40, 59, 61, 50, 14, 43, 62, 26, 37, 24, 33, 39, 31, 20, 4, 18, 46, 60, 15, 49, 13, 58, 53,
    55, 9, 41, 7, 63, 54, 56, 52, 12, 42, 10, 44, 45, 47, 8, 48, 11, 0, 25, 32, 5, 34, 35, 16, 36,
    38, 19, 3, 57, 30, 22, 28, 23, 29, 51, 21, 1, 17, 2, 6, 27, 41, 19, 22, 24, 1, 44, 11, 13, 0,
    50, 55, 27, 54, 34, 57, 9, 8, 29, 36, 51, 26, 38, 17, 33, 18, 40, 5, 42, 31, 2, 45, 21, 63, 30,
    43, 52, 3, 46, 28, 62, 37, 16, 6, 7, 20, 23, 25, 15, 58, 59, 47, 4, 10, 61, 48, 35, 60, 49, 12,
    14, 53, 56, 32, 39, 42, 29, 19, 31, 2, 45, 41, 52, 6, 58, 13, 0, 50, 35, 60, 12, 11, 4, 38, 17,
    33, 24, 1, 44, 28, 62, 23, 8, 36, 51, 26, 18, 40, 5, 55, 43, 21, 63, 3, 46, 39, 27, 16, 10, 30,
    20, 32, 54, 15, 61, 48, 22, 34, 57, 9, 7, 59, 56, 47, 53, 49, 14, 37, 25, 43, 36, 38, 19, 3,
    46, 55, 41, 7, 59, 42, 10, 61, 0, 50, 14, 13, 31, 4, 18, 40, 29, 21, 63, 1, 44, 25, 52, 22, 28,
    62, 2, 45, 32, 8, 11, 51, 26, 17, 33, 5, 34, 35, 16, 37, 39, 20, 57, 60, 15, 49, 24, 6, 58, 53,
    27, 54, 9, 56, 47, 12, 48, 23, 30, 44, 15, 58, 59, 47, 8, 33, 40, 5, 19, 63, 30, 22, 37, 24, 1,
    26, 54, 57, 9, 41, 60, 12, 42, 14, 43, 16, 45, 61, 48, 11, 56, 52, 6, 46, 62, 49, 13, 53, 55,
    7, 20, 23, 25, 0, 27, 34, 4, 29, 36, 51, 50, 32, 38, 17, 39, 31, 18, 2, 3, 21, 28, 35, 10, 45,
    54, 15, 61, 48, 11, 44, 62, 23, 29, 40, 5, 19, 39, 31, 2, 33, 50, 60, 12, 42, 59, 47, 8, 53,
    55, 27, 26, 57, 9, 41, 14, 43, 16, 63, 46, 56, 52, 49, 13, 10, 30, 20, 32, 6, 0, 35, 22, 4, 38,
    17, 58, 37, 24, 1, 25, 36, 28, 51, 21, 18, 3, 7, 34, 46, 57, 60, 15, 49, 13, 63, 44, 25, 36,
    45, 32, 38, 5, 19, 3, 40, 61, 50, 14, 43, 54, 56, 52, 47, 8, 34, 62, 58, 53, 55, 48, 11, 35,
    26, 33, 9, 41, 12, 42, 16, 37, 39, 20, 7, 10, 0, 24, 31, 4, 18, 59, 23, 29, 21, 30, 22, 1, 28,
    51, 2, 17, 27, 6, 47, 20, 23, 25, 26, 1, 12, 14, 15, 0, 56, 58, 27, 59, 34, 41, 9, 30, 37, 44,
    51, 39, 45, 17, 46, 18, 19, 48, 32, 33, 2, 62, 21, 22, 49, 53, 40, 3, 63, 28, 24, 50, 54, 57,
    4, 29, 36, 16, 6, 7, 8, 5, 60, 10, 11, 61, 35, 13, 42, 43, 55, 52, 38, 31, 48, 30, 20, 32, 33,
    2, 47, 53, 54, 6, 14, 15, 0, 61, 35, 42, 12, 5, 39, 45, 17, 25, 26, 1, 63, 28, 29, 9, 37, 44,
    51, 46, 18, 19, 56, 49, 62, 21, 40, 3, 31, 58, 50, 60, 22, 4, 38, 27, 16, 10, 11, 23, 59, 34,
    41, 57, 7, 52, 8, 55, 13, 43, 24, 36, 49, 37, 39, 20, 40, 3, 56, 47, 57, 7, 48, 60, 10, 15, 0,
    43, 14, 32, 5, 46, 18, 30, 62, 21, 26, 1, 36, 53, 23, 63, 28, 33, 2, 38, 9, 12, 44, 51, 45, 17,
    19, 59, 61, 50, 24, 31, 4, 34, 35, 16, 13, 25, 54, 6, 55, 58, 27, 41, 52, 8, 42, 11, 29, 22,
    50, 44, 45, 46, 20, 4, 58, 59, 47, 8, 61, 48, 11, 49, 13, 0, 15, 33, 40, 5, 19, 63, 30, 22, 37,
    24, 1, 54, 62, 23, 29, 39, 31, 2, 57, 60, 25, 36, 32, 38, 3, 9, 12, 14, 51, 17, 18, 41, 42, 43,
    16, 26, 56, 52, 6, 53, 55, 7, 27, 34, 10, 35, 21, 28, 51, 0, 27, 34, 41, 47, 2, 3, 4, 5, 28,
    29, 30, 36, 37, 44, 1, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 17, 35, 42, 48, 55, 56, 58, 18, 21,
    43, 49, 52, 53, 59, 19, 22, 24, 50, 54, 57, 20, 23, 25, 26, 16, 31, 32, 33, 38, 39, 40, 45, 46,
    63, 62, 60, 61, 52, 31, 36, 22, 28, 62, 43, 11, 35, 60, 41, 7, 59, 27, 54, 56, 55, 24, 29, 21,
    63, 19, 3, 46, 17, 33, 39, 13, 4, 18, 40, 1, 44, 25, 42, 8, 2, 45, 51, 26, 30, 10, 34, 6, 32,
    37, 23, 61, 57, 58, 53, 38, 0, 50, 14, 16, 15, 48, 49, 12, 9, 47, 5, 20, 53, 32, 37, 23, 63,
    28, 49, 12, 61, 35, 47, 57, 7, 58, 27, 52, 56, 25, 30, 62, 21, 20, 40, 3, 45, 17, 31, 14, 5,
    46, 18, 26, 1, 36, 48, 9, 33, 2, 44, 51, 22, 60, 59, 54, 38, 24, 29, 10, 34, 6, 55, 39, 15, 0,
    43, 50, 16, 11, 13, 42, 41, 8, 19, 4, 54, 33, 44, 62, 23, 29, 50, 60, 12, 42, 59, 47, 8, 53,
    55, 27, 58, 26, 63, 30, 22, 46, 20, 4, 32, 38, 17, 15, 40, 5, 19, 37, 24, 1, 61, 57, 39, 31,
    25, 36, 28, 48, 9, 56, 2, 51, 21, 11, 41, 52, 6, 45, 49, 13, 0, 14, 43, 35, 16, 10, 7, 34, 3,
    18, 55, 38, 24, 29, 21, 63, 13, 42, 10, 61, 8, 34, 57, 6, 58, 53, 52, 36, 22, 28, 62, 4, 18,
    40, 2, 45, 32, 43, 19, 3, 46, 51, 26, 37, 11, 41, 17, 33, 1, 44, 23, 35, 7, 27, 39, 25, 30, 60,
    59, 54, 56, 31, 16, 15, 49, 0, 50, 12, 14, 48, 47, 9, 20, 5, 56, 39, 25, 30, 62, 21, 14, 48,
    60, 10, 9, 59, 34, 54, 6, 55, 53, 37, 23, 63, 28, 5, 46, 18, 33, 2, 38, 49, 20, 40, 3, 44, 51,
    24, 12, 47, 45, 17, 26, 1, 29, 61, 57, 58, 31, 36, 22, 35, 7, 27, 52, 32, 50, 16, 13, 15, 0,
    42, 43, 11, 8, 41, 4, 19, 57, 40, 63, 44, 25, 36, 61, 50, 14, 43, 54, 56, 52, 47, 8, 34, 59,
    62, 26, 37, 24, 33, 39, 31, 20, 4, 18, 60, 45, 32, 38, 23, 29, 21, 15, 58, 5, 19, 30, 22, 1,
    49, 53, 9, 3, 28, 51, 13, 55, 41, 7, 46, 12, 42, 10, 48, 11, 0, 35, 16, 27, 6, 17, 2, 58, 45,
    26, 63, 30, 22, 15, 61, 48, 11, 57, 9, 41, 56, 52, 6, 54, 44, 62, 23, 29, 40, 5, 19, 39, 31, 2,
    50, 46, 20, 4, 25, 36, 51, 60, 59, 32, 38, 37, 24, 21, 12, 47, 53, 17, 1, 28, 42, 8, 55, 27,
    33, 14, 43, 16, 49, 13, 10, 0, 35, 34, 7, 18, 3, 59, 46, 62, 26, 37, 24, 60, 15, 49, 13, 58,
    53, 55, 9, 41, 7, 57, 63, 44, 25, 36, 45, 32, 38, 5, 19, 3, 61, 33, 39, 31, 30, 22, 28, 50, 54,
    20, 4, 23, 29, 51, 14, 56, 47, 18, 21, 1, 43, 52, 8, 34, 40, 48, 11, 35, 12, 42, 16, 10, 0, 6,
    27, 2, 17, 60, 62, 40, 45, 32, 38, 57, 58, 53, 55, 50, 14, 43, 48, 11, 35, 61, 46, 33, 39, 31,
    44, 25, 36, 30, 22, 28, 59, 26, 37, 24, 20, 4, 18, 54, 15, 23, 29, 5, 19, 2, 56, 49, 12, 21, 3,
    17, 52, 13, 42, 10, 63, 47, 8, 34, 9, 41, 6, 7, 27, 0, 16, 1, 51, 61, 63, 46, 33, 39, 31, 59,
    54, 56, 52, 15, 49, 13, 12, 42, 10, 60, 40, 45, 32, 38, 26, 37, 24, 23, 29, 21, 57, 44, 25, 36,
    5, 19, 3, 58, 50, 30, 22, 20, 4, 17, 53, 14, 48, 28, 18, 2, 55, 43, 11, 35, 62, 9, 41, 7, 47,
    8, 27, 34, 6, 16, 0, 51, 1, 62, 61, 57, 58, 53, 55, 46, 33, 39, 31, 44, 25, 36, 30, 22, 28, 63,
    59, 54, 56, 52, 15, 49, 13, 12, 42, 10, 40, 50, 14, 43, 47, 8, 34, 45, 26, 48, 11, 9, 41, 6,
    32, 37, 23, 35, 7, 27, 38, 24, 29, 21, 60, 5, 19, 3, 20, 4, 17, 18, 2, 1, 51, 16, 0, 63, 60,
    59, 54, 56, 52, 40, 45, 32, 38, 26, 37, 24, 23, 29, 21, 62, 57, 58, 53, 55, 50, 14, 43, 48, 11,
    35, 46, 15, 49, 13, 9, 41, 7, 33, 44, 12, 42, 47, 8, 27, 39, 25, 30, 10, 34, 6, 31, 36, 22, 28,
    61, 20, 4, 18, 5, 19, 2, 3, 17, 51, 1, 0, 16,
];

const ROOTS: [[i8; 8]; 16] = [
    [2, 2, 0, 0, 0, 0, 0, 0],
    [-2, -2, 0, 0, 0, 0, 0, 0],
    [-2, 2, 0, 0, 0, 0, 0, 0],
    [0, 0, -2, 0, -2, 0, 0, 0],
    [0, 0, -2, 0, 2, 0, 0, 0],
    [0, 0, 0, -2, 0, 0, 0, -2],
    [0, 0, 0, -2, 0, 0, 0, 2],
    [0, 0, 0, 0, 0, -2, -2, 0],
    [0, 0, 0, 0, 0, -2, 2, 0],
    [0, 0, 0, 0, 0, 2, -2, 0],
    [0, 0, 0, 0, 0, 2, 2, 0],
    [0, 0, 0, 2, 0, 0, 0, -2],
    [0, 0, 0, 2, 0, 0, 0, 2],
    [0, 0, 2, 0, -2, 0, 0, 0],
    [0, 0, 2, 0, 2, 0, 0, 0],
    [2, -2, 0, 0, 0, 0, 0, 0],
];

const BFS: [u8; 256] = [
    0, 16, 7, 42, 3, 8, 9, 27, 5, 24, 4, 47, 2, 12, 1, 34, 16, 0, 34, 1, 12, 2, 47, 4, 24, 5, 27,
    9, 8, 3, 42, 7, 34, 7, 0, 26, 11, 9, 14, 10, 6, 4, 5, 2, 47, 1, 3, 16, 1, 42, 3, 0, 15, 4, 5,
    8, 13, 9, 2, 10, 27, 7, 16, 48, 12, 3, 1, 7, 0, 5, 6, 9, 2, 41, 13, 4, 35, 16, 15, 11, 2, 8,
    47, 27, 24, 0, 7, 1, 3, 12, 42, 34, 16, 5, 4, 9, 47, 9, 2, 10, 4, 34, 0, 3, 11, 1, 26, 16, 7,
    6, 5, 14, 4, 27, 5, 2, 13, 42, 48, 0, 7, 15, 16, 3, 1, 9, 8, 10, 35, 5, 4, 9, 41, 12, 1, 15, 0,
    16, 7, 11, 3, 2, 13, 6, 5, 35, 6, 13, 2, 3, 11, 7, 16, 0, 15, 1, 12, 41, 9, 4, 27, 4, 10, 8, 9,
    1, 3, 16, 15, 7, 0, 48, 42, 13, 2, 5, 9, 47, 14, 5, 6, 7, 16, 26, 1, 11, 3, 0, 34, 4, 10, 2, 8,
    2, 9, 4, 5, 16, 34, 42, 12, 3, 1, 7, 0, 24, 27, 47, 3, 12, 11, 15, 16, 35, 4, 13, 41, 2, 9, 6,
    5, 0, 7, 1, 42, 1, 48, 16, 7, 27, 10, 2, 9, 13, 8, 5, 4, 15, 0, 3, 7, 34, 16, 3, 1, 47, 2, 5,
    4, 6, 10, 14, 9, 11, 26, 0,
];

const BRANCH: [[u8; 3]; 16] = [
    [0, 14, 3],
    [1, 3, 14],
    [2, 13, 4],
    [3, 0, 1],
    [4, 2, 15],
    [5, 7, 10],
    [6, 9, 8],
    [7, 12, 5],
    [8, 6, 11],
    [9, 11, 6],
    [10, 5, 12],
    [11, 8, 9],
    [12, 10, 7],
    [13, 15, 2],
    [14, 1, 0],
    [15, 4, 13],
];

const IDENTITY: u8 = 0;
const OPERATOR_KAPPA: &str =
    "blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b";
const COMPOSITION_KAPPA: &str =
    "blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0";

/// Read operator of a discrete checkpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscreteReadMode {
    Route,
    Flock { kept: u32 },
    Relation { kept: u32, sweeps: u32 },
}

/// Objective policy recorded in the discrete header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscreteObjectivePolicy {
    NearestAddressMarginV1,
    OccupiedCellRankHingeV1,
}

/// Shape and read/writer metadata of a discrete `UORLUT01` artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscreteLutShape {
    pub vocabulary: u32,
    pub blocks: u32,
    pub context: u32,
    pub kept_cells: u32,
    pub stages: u32,
    pub read_mode: String,
    pub read_kept: u32,
    pub read_sweeps: u32,
    pub objective_policy: String,
    pub read_initialization: String,
    pub seed: u32,
    pub epoch: u64,
}

/// Provenance recorded in `source` of a discrete `UORLUT01` artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscreteLutSource {
    pub snapshot_schema: String,
    pub snapshot_sha256: String,
    pub operator_kappa: String,
    pub composition_kappa: String,
    pub kernel_sha256: String,
    pub source_sha256: String,
    pub geometry_sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SectionSpan {
    offset: u64,
    bytes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SectionKind {
    I16,
    I32,
    U32,
}

impl SectionKind {
    fn width(self) -> usize {
        match self {
            Self::I16 => 2,
            Self::I32 | Self::U32 => 4,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SectionSpec {
    name: String,
    kind: SectionKind,
    len: usize,
    span: SectionSpan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct DiscreteLutHeader {
    schema: String,
    shape: DiscreteLutShape,
    matrices: Vec<serde_json::Value>,
    tables: Vec<SectionSpec>,
    source: DiscreteLutSource,
}

/// Output of evaluating a causal prefix on an [`IntegerDiscreteModel`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscretePrefixEvaluation {
    /// Read vector `q` of length `blocks * 8` (`[i32; 8]` per block).
    pub read_vector: Vec<i32>,
    /// Occupied writer cells admitted in `(address_distance, cell_id)` order (`1..=kept_cells`).
    pub admitted_cells: Vec<u32>,
    /// Up to 8 globally top-ranked admitted tokens in `(token_distance, token_id)` order.
    pub top8: Vec<u32>,
    /// Top-1 predicted token (`top8[0]`).
    pub prediction: u32,
    /// Per-node relation scores (`blocks * nodes_count`) when `read_mode` is `Relation`.
    pub relation_scores: Option<Vec<i32>>,
}

/// Target-supervised writer loss and rival counts for one prefix evaluation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscreteTargetObservation {
    pub evaluation: DiscretePrefixEvaluation,
    pub token_rivals: u32,
    pub address_rivals: u32,
    pub token_hinge: u64,
    pub address_hinge: u64,
    pub rank_hinge: u64,
    pub loss_ta: u64,
    pub loss_tr: u64,
    pub selected_loss: u64,
}

/// Loaded discrete SpiralCore/FBS model served from a `UORLUT01` (`uor-r4.lut-discrete/1`)
/// integer container.
#[derive(Clone, Debug)]
pub struct IntegerDiscreteModel {
    shape: DiscreteLutShape,
    source: DiscreteLutSource,
    read_mode: DiscreteReadMode,
    objective_policy: DiscreteObjectivePolicy,
    artifact_sha256: String,
    /// `[token][block]` 8D orbit-root coordinates.
    token_blocks: Vec<Vec<[i32; 8]>>,
    /// Pre-verified writer cell `0..256` per token.
    cells: Vec<u32>,
    /// `[block][stage]` route action index `0..64`.
    routes: Vec<Vec<u32>>,
    /// `[block][stage]` level gate trits in `-1..=1`.
    level_gates: Vec<Vec<i32>>,
    /// `[stage][edge_type]` relation-edge gate trits in `-1..=1` (empty unless `Relation`).
    relation_gates: Vec<[i32; EDGE_TYPES]>,
    /// When `true`, projects intermediate tree/frame state transport to the identity frame
    /// (`A_0 = I_8`) on the local query token block and zeroes all level and relation gates.
    state_transport_knockout: bool,
}

fn expected_stages(context: u32) -> Option<u32> {
    if !(1..=MAX_CONTEXT as u32).contains(&context) {
        return None;
    }
    let mut stages = 1u32;
    let mut span = 3u32;
    while span <= context {
        stages += 1;
        span += span + span;
    }
    Some(stages)
}

fn root_id(block: &[i32; 8]) -> Option<usize> {
    (0..16).find(|&r| (0..8).all(|d| block[d] == i32::from(ROOTS[r][d])))
}

fn action(a: usize, x: &[i32; 8]) -> [i32; 8] {
    let mut y = [0i32; 8];
    let base = a << 3;
    for d in 0..8 {
        let axis = usize::from(AXES[base + d]);
        let v = x[axis & 7];
        y[d] = if (axis & 8) != 0 { -v } else { v };
    }
    y
}

fn compose(a: usize, b: usize) -> usize {
    usize::from(COMPOSE[(a << 6) + b])
}

fn root_cost(x: &[i32; 8], root: usize) -> (u32, u32) {
    let mut distance = 0u32;
    let mut hamming = 0u32;
    for d in 0..8 {
        let y = i32::from(ROOTS[root][d]);
        distance += (x[d] - y).unsigned_abs();
        hamming += u32::from(x[d] != y);
    }
    (distance, hamming)
}

fn snap(x: &[i32; 8], n: usize) -> usize {
    let mut best = 0usize;
    let mut best_distance = u32::MAX;
    let mut best_hamming = u32::MAX;
    for r in 0..16 {
        let mut distance = 0u32;
        let mut hamming = 0u32;
        for d in 0..8 {
            let mut y = 0i32;
            let v = i32::from(ROOTS[r][d]);
            for _ in 0..n {
                y += v;
            }
            distance += (x[d] - y).unsigned_abs();
            hamming += u32::from(x[d] != y);
        }
        if distance < best_distance || (distance == best_distance && hamming < best_hamming) {
            best = r;
            best_distance = distance;
            best_hamming = hamming;
        }
    }
    best
}

fn checked_scaled_add(mut value: u64, mut times: u32) -> Option<u64> {
    let mut total = 0u64;
    while times != 0 {
        if (times & 1) != 0 {
            total = total.checked_add(value)?;
        }
        times >>= 1;
        if times != 0 {
            if value > (u64::MAX >> 1) {
                return None;
            }
            value <<= 1;
        }
    }
    Some(total)
}

#[derive(Clone, Debug)]
struct BinderNode {
    children: Option<[usize; 3]>,
    atom: Option<usize>,
    level: usize,
    first: usize,
    last: usize,
}

#[derive(Clone, Debug)]
struct RowPlan {
    nodes: Vec<BinderNode>,
    frontier: Vec<usize>,
}

fn build_row_plan(length: usize) -> Result<RowPlan> {
    if !(1..=MAX_CONTEXT).contains(&length) {
        return Err(invalid("prefix length must be in 1..=context"));
    }
    let mut nodes: Vec<BinderNode> = Vec::new();
    let mut frontier: Vec<usize> = Vec::new();
    for atom in 0..length {
        frontier.push(nodes.len());
        nodes.push(BinderNode {
            children: None,
            atom: Some(atom),
            level: 0,
            first: atom,
            last: atom,
        });
        loop {
            let n = frontier.len();
            if n < 3 {
                break;
            }
            let c0 = frontier[n - 3];
            let c1 = frontier[n - 2];
            let c2 = frontier[n - 1];
            let lv = nodes[c0].level;
            if nodes[c1].level != lv || nodes[c2].level != lv {
                break;
            }
            let parent = nodes.len();
            let first = nodes[c0].first;
            let last = nodes[c2].last;
            nodes.push(BinderNode {
                children: Some([c0, c1, c2]),
                atom: None,
                level: lv + 1,
                first,
                last,
            });
            frontier.truncate(n - 3);
            frontier.push(parent);
        }
    }
    Ok(RowPlan { nodes, frontier })
}

pub(crate) fn orbit_root_coords(root: usize) -> [i32; 8] {
    let r = &ROOTS[root];
    [
        i32::from(r[0]),
        i32::from(r[1]),
        i32::from(r[2]),
        i32::from(r[3]),
        i32::from(r[4]),
        i32::from(r[5]),
        i32::from(r[6]),
        i32::from(r[7]),
    ]
}

pub(crate) fn orbit_root_id(block: &[i32; 8]) -> Option<usize> {
    root_id(block)
}

pub(crate) fn orbit_snap(block: &[i32; 8], n: usize) -> usize {
    snap(block, n)
}

pub(crate) fn cl06_action(a: usize, x: &[i32; 8]) -> [i32; 8] {
    action(a, x)
}

/// The eight signed-axis codes of Cl(0,6) state `a`: `code & 7` is the source
/// axis, `code & 8` negates.
pub(crate) fn cl06_axes(a: usize) -> [u8; 8] {
    let mut codes = [0u8; 8];
    codes.copy_from_slice(&AXES[a * 8..a * 8 + 8]);
    codes
}

pub(crate) fn cl06_compose(a: usize, b: usize) -> usize {
    compose(a, b)
}

pub(crate) fn cl06_bfs(src: usize, dst: usize) -> usize {
    usize::from(BFS[(src << 4) + dst])
}

pub(crate) fn cl06_inverse(a: usize) -> usize {
    compose(a, compose(a, a))
}

pub(crate) fn fbs_binder_summary(length: usize) -> Result<(usize, usize, u32)> {
    let plan = build_row_plan(length)?;
    let stages =
        expected_stages(length as u32).ok_or_else(|| invalid("invalid FBS binder length"))?;
    Ok((plan.nodes.len(), plan.frontier.len(), stages))
}

impl IntegerDiscreteModel {
    /// Load and validate a `UORLUT01` (`uor-r4.lut-discrete/1`) file from disk.
    pub fn load(path: &Path) -> Result<Self> {
        Self::from_bytes(&std::fs::read(path)?)
    }

    /// Parse and validate a `UORLUT01` (`uor-r4.lut-discrete/1`) byte buffer.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 16 || &bytes[..8] != MAGIC {
            return Err(invalid("missing UORLUT01 magic"));
        }
        let mut len_bytes = [0u8; 8];
        len_bytes.copy_from_slice(&bytes[8..16]);
        let header_len = u64::from_le_bytes(len_bytes);
        if header_len > MAX_HEADER || 16 + (header_len as usize) > bytes.len() {
            return Err(invalid("UORLUT01 header length out of bounds"));
        }
        let header: DiscreteLutHeader =
            serde_json::from_slice(&bytes[16..16 + (header_len as usize)])?;
        if header.schema != DISCRETE_SCHEMA || !header.matrices.is_empty() {
            return Err(invalid(
                "discrete UORLUT01 artifact requires schema uor-r4.lut-discrete/1 and empty matrices",
            ));
        }
        if header.source.snapshot_schema != SNAPSHOT_SCHEMA
            || header.source.operator_kappa != OPERATOR_KAPPA
            || header.source.composition_kappa != COMPOSITION_KAPPA
        {
            return Err(invalid(
                "discrete UORLUT01 snapshot_schema or geometry kappa mismatch",
            ));
        }
        let unaligned = 16 + (header_len as usize);
        let data_start = unaligned + (ALIGN - unaligned % ALIGN) % ALIGN;
        if data_start > bytes.len() || bytes[unaligned..data_start].iter().any(|&b| b != 0) {
            return Err(invalid("UORLUT01 header padding is missing or non-zero"));
        }
        let data = &bytes[data_start..];
        let s = &header.shape;
        if !(2..=8192).contains(&s.vocabulary)
            || !(2..=16).contains(&s.blocks)
            || !(1..=MAX_CONTEXT as u32).contains(&s.context)
            || !(1..=16).contains(&s.kept_cells)
            || s.kept_cells > s.vocabulary
            || expected_stages(s.context) != Some(s.stages)
        {
            return Err(invalid("inconsistent or unsupported discrete shape"));
        }
        let read_mode = match (s.read_mode.as_str(), s.read_kept, s.read_sweeps) {
            ("route", 0, 0) => DiscreteReadMode::Route,
            ("flock", kept, 0) if (1..=MAX_KEPT as u32).contains(&kept) => {
                DiscreteReadMode::Flock { kept }
            }
            ("relation", kept, sweeps)
                if (1..=MAX_KEPT as u32).contains(&kept) && (1..=MAX_SWEEPS).contains(&sweeps) =>
            {
                DiscreteReadMode::Relation { kept, sweeps }
            }
            _ => return Err(invalid("unsupported discrete read mode or parameters")),
        };
        let objective_policy = match s.objective_policy.as_str() {
            "nearest_address_margin_v1" => DiscreteObjectivePolicy::NearestAddressMarginV1,
            "occupied_cell_rank_hinge_v1" => DiscreteObjectivePolicy::OccupiedCellRankHingeV1,
            _ => return Err(invalid("unsupported discrete objective policy")),
        };
        let init_ok = match s.read_initialization.as_str() {
            "zero_gates_v1" => true,
            "active_leaf_v1" => !matches!(read_mode, DiscreteReadMode::Route),
            _ => false,
        };
        if !init_ok {
            return Err(invalid("unsupported discrete read initialization"));
        }

        if header.tables.len() != 4 {
            return Err(invalid("discrete UORLUT01 requires exactly four tables"));
        }
        let read_section =
            |name: &str, expected_kind: SectionKind, expected_len: usize| -> Result<&[u8]> {
                let entry = header
                    .tables
                    .iter()
                    .find(|t| t.name == name)
                    .ok_or_else(|| invalid(format!("missing table {name}")))?;
                if entry.kind != expected_kind || entry.len != expected_len {
                    return Err(invalid(format!("table {name} kind or length mismatch")));
                }
                let expected_bytes = expected_len
                    .checked_mul(expected_kind.width())
                    .ok_or_else(|| invalid("table byte size overflow"))?;
                if (entry.span.offset as usize) % ALIGN != 0
                    || entry.span.bytes != expected_bytes as u64
                {
                    return Err(invalid(format!(
                        "table {name} alignment or byte span mismatch"
                    )));
                }
                let start = usize::try_from(entry.span.offset)
                    .map_err(|_| invalid("table offset overflow"))?;
                let end = start
                    .checked_add(expected_bytes)
                    .ok_or_else(|| invalid("table end overflow"))?;
                data.get(start..end)
                    .ok_or_else(|| invalid(format!("table {name} span out of bounds")))
            };

        let vocab = s.vocabulary as usize;
        let blocks = s.blocks as usize;
        let stages = s.stages as usize;
        let base_gates = blocks * stages;
        let total_gates = if matches!(read_mode, DiscreteReadMode::Relation { .. }) {
            base_gates + EDGE_TYPES * stages
        } else {
            base_gates
        };

        let raw_table: Vec<i32> =
            read_section("discrete.table", SectionKind::I32, vocab * blocks * 8)?
                .chunks_exact(4)
                .map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect();
        let cells: Vec<u32> = read_section("discrete.cells", SectionKind::U32, vocab)?
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();
        let raw_routes: Vec<u32> = read_section("discrete.routes", SectionKind::U32, base_gates)?
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();
        let raw_gates: Vec<i32> = read_section("discrete.gates", SectionKind::I32, total_gates)?
            .chunks_exact(4)
            .map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();

        if raw_routes.iter().any(|&r| r >= 64) {
            return Err(invalid("discrete.routes contains action index >= 64"));
        }
        if raw_gates.iter().any(|&g| !(-1..=1).contains(&g)) {
            return Err(invalid("discrete.gates contains trit outside -1..=1"));
        }

        let mut token_blocks = Vec::with_capacity(vocab);
        for token in 0..vocab {
            let mut tb = Vec::with_capacity(blocks);
            let mut first_root = 0u32;
            let mut last_root = 0u32;
            for block in 0..blocks {
                let start = (token * blocks + block) * 8;
                let mut coords = [0i32; 8];
                coords.copy_from_slice(&raw_table[start..start + 8]);
                let root = root_id(&coords).ok_or_else(|| {
                    invalid("discrete.table block is outside the 16-root SpiralCore orbit")
                })? as u32;
                if block == 0 {
                    first_root = root;
                }
                if block + 1 == blocks {
                    last_root = root;
                }
                tb.push(coords);
            }
            let expected_cell = (first_root << 4) + last_root;
            if cells[token] != expected_cell {
                return Err(invalid(
                    "discrete.cells entry disagrees with discrete.table endpoint roots",
                ));
            }
            token_blocks.push(tb);
        }

        let routes: Vec<Vec<u32>> = (0..blocks)
            .map(|b| raw_routes[b * stages..(b + 1) * stages].to_vec())
            .collect();
        let level_gates: Vec<Vec<i32>> = (0..blocks)
            .map(|b| raw_gates[b * stages..(b + 1) * stages].to_vec())
            .collect();
        let mut relation_gates = Vec::new();
        if matches!(read_mode, DiscreteReadMode::Relation { .. }) {
            for stage in 0..stages {
                let mut rg = [0i32; EDGE_TYPES];
                let base = base_gates + stage * EDGE_TYPES;
                rg.copy_from_slice(&raw_gates[base..base + EDGE_TYPES]);
                relation_gates.push(rg);
            }
        }

        let artifact_sha256 = hex::encode(Sha256::digest(bytes));
        Ok(Self {
            shape: header.shape,
            source: header.source,
            read_mode,
            objective_policy,
            artifact_sha256,
            token_blocks,
            cells,
            routes,
            level_gates,
            relation_gates,
            state_transport_knockout: false,
        })
    }

    /// Return a copy of this model with intermediate state transport knocked out:
    /// every binder-route action is projected to the identity element (`A_0 = 0`),
    /// level transport gates and relation transport gates are zeroed, and `Route`
    /// reads return the un-transported local query token blocks on the 16-root orbit.
    pub fn with_state_transport_knockout(&self) -> Self {
        let mut copy = self.clone();
        for b in &mut copy.routes {
            for r in b.iter_mut() {
                *r = u32::from(IDENTITY);
            }
        }
        for b in &mut copy.level_gates {
            for g in b.iter_mut() {
                *g = 0;
            }
        }
        for s in &mut copy.relation_gates {
            *s = [0; EDGE_TYPES];
        }
        copy.state_transport_knockout = true;
        copy
    }

    pub fn is_state_transport_knocked_out(&self) -> bool {
        self.state_transport_knockout
    }

    pub fn shape(&self) -> &DiscreteLutShape {
        &self.shape
    }

    pub fn source(&self) -> &DiscreteLutSource {
        &self.source
    }

    pub fn read_mode(&self) -> DiscreteReadMode {
        self.read_mode
    }

    pub fn artifact_sha256(&self) -> &str {
        &self.artifact_sha256
    }

    pub fn cells(&self) -> &[u32] {
        &self.cells
    }

    fn route_read(&self, prefix: &[u32], plan: &RowPlan) -> Result<Vec<[i32; 8]>> {
        let blocks = self.shape.blocks as usize;
        let stages = self.shape.stages as usize;
        let last_token = prefix[prefix.len() - 1] as usize;
        if self.state_transport_knockout {
            return Ok((0..blocks)
                .map(|block| self.token_blocks[last_token][block])
                .collect());
        }
        let mut out = Vec::with_capacity(blocks);
        for block in 0..blocks {
            let query = &self.token_blocks[last_token][block];
            let mut node = 0usize;
            let mut root = 0usize;
            let mut total = usize::from(IDENTITY);
            let mut resolved = None;
            for stage in 0..stages {
                let frame = self.routes[block][stage] as usize;
                let q = action(frame, query);
                let count = if stage == 0 { plan.frontier.len() } else { 3 };
                let mut best = 0usize;
                let mut best_distance = u32::MAX;
                let mut best_hamming = u32::MAX;
                let mut goal = 0usize;
                for port in 0..count {
                    let r = if stage == 0 {
                        port
                    } else {
                        usize::from(BRANCH[root][port])
                    };
                    let (distance, hamming) = root_cost(&q, r);
                    if distance < best_distance
                        || (distance == best_distance && hamming < best_hamming)
                    {
                        best = port;
                        best_distance = distance;
                        best_hamming = hamming;
                        goal = r;
                    }
                }
                total = compose(usize::from(BFS[(root << 4) + goal]), total);
                root = goal;
                let next_node = if stage == 0 {
                    plan.frontier[best]
                } else {
                    let children = plan.nodes[node]
                        .children
                        .ok_or_else(|| invalid("route stepped past an atom leaf"))?;
                    children[best]
                };
                if let Some(atom) = plan.nodes[next_node].atom {
                    let carrier_token = prefix[atom] as usize;
                    let carrier = &self.token_blocks[carrier_token][block];
                    resolved = Some(action(total, carrier));
                    break;
                }
                node = next_node;
            }
            out.push(resolved.ok_or_else(|| invalid("route did not reach an atom leaf"))?);
        }
        Ok(out)
    }

    fn flock_or_relation_read(
        &self,
        prefix: &[u32],
        plan: &RowPlan,
        kept: usize,
        sweeps: u32,
        relation_active: bool,
    ) -> Result<(Vec<[i32; 8]>, Option<Vec<i32>>)> {
        let blocks = self.shape.blocks as usize;
        let stages = self.shape.stages as usize;
        let length = prefix.len();
        let nodes_count = plan.nodes.len();
        let last_token = prefix[length - 1] as usize;
        let mut out = Vec::with_capacity(blocks);
        let mut all_rel_scores = if relation_active {
            Some(Vec::with_capacity(blocks * nodes_count))
        } else {
            None
        };

        for b in 0..blocks {
            let f = if b == 0 { blocks - 1 } else { 0 };
            let query_frame = &self.token_blocks[last_token][f];
            let mut u = self.token_blocks[last_token][b];
            let fq = root_id(query_frame)
                .ok_or_else(|| invalid("query frame outside SpiralCore orbit"))?;

            let mut frame = vec![0usize; nodes_count];
            let mut payload = vec![[0i32; 8]; nodes_count];
            for (node_idx, node) in plan.nodes.iter().enumerate() {
                if let Some(atom) = node.atom {
                    let tok = prefix[atom] as usize;
                    frame[node_idx] = root_id(&self.token_blocks[tok][f])
                        .ok_or_else(|| invalid("atom frame outside SpiralCore orbit"))?;
                    payload[node_idx] = self.token_blocks[tok][b];
                } else {
                    let children = node
                        .children
                        .ok_or_else(|| invalid("non-atom node missing children"))?;
                    let c0 = children[0];
                    frame[node_idx] = frame[c0];
                    let mut sum = [0i32; 8];
                    for &child in &children {
                        let act = usize::from(BFS[(frame[child] << 4) + frame[c0]]);
                        let carried = action(act, &payload[child]);
                        for d in 0..8 {
                            sum[d] += carried[d];
                        }
                    }
                    let r = snap(&sum, 3);
                    payload[node_idx] = ROOTS[r].map(i32::from);
                }
            }

            let mut rel = vec![0i32; nodes_count];
            if relation_active {
                let mut prev: Vec<i32> = (0..nodes_count)
                    .map(|n| 8 - (root_cost(query_frame, frame[n]).0 as i32))
                    .collect();
                for _ in 0..sweeps {
                    for i in 0..nodes_count {
                        if plan.nodes[i].last >= length {
                            continue;
                        }
                        let li = plan.nodes[i].level;
                        for e in 0..EDGE_TYPES {
                            let gt = self.relation_gates[li][e];
                            if gt == 0 {
                                continue;
                            }
                            let mut esum = 0i32;
                            if e == 0 {
                                if let Some(children) = plan.nodes[i].children {
                                    for ch in children {
                                        if plan.nodes[ch].last < length {
                                            esum += prev[ch];
                                        }
                                    }
                                }
                                for p in (i + 1)..nodes_count {
                                    if let Some(children) = plan.nodes[p].children {
                                        if plan.nodes[p].last < length
                                            && (children[0] == i
                                                || children[1] == i
                                                || children[2] == i)
                                        {
                                            esum += prev[p];
                                        }
                                    }
                                }
                            } else if e == 1 {
                                for p in (i + 1)..nodes_count {
                                    let Some([c0, c1, c2]) = plan.nodes[p].children else {
                                        continue;
                                    };
                                    if plan.nodes[p].last >= length {
                                        continue;
                                    }
                                    if i == c0 && plan.nodes[c1].last < length {
                                        esum += prev[c1];
                                    } else if i == c1 {
                                        if plan.nodes[c0].last < length {
                                            esum += prev[c0];
                                        }
                                        if plan.nodes[c2].last < length {
                                            esum += prev[c2];
                                        }
                                    } else if i == c2 && plan.nodes[c1].last < length {
                                        esum += prev[c1];
                                    }
                                }
                            } else if e == 2 {
                                for j in 0..i {
                                    if plan.nodes[j].last < length
                                        && plan.nodes[j].level == li
                                        && plan.nodes[j].last + 1 == plan.nodes[i].first
                                    {
                                        esum += prev[j];
                                    }
                                }
                            } else if e == 3 {
                                if li == 0 {
                                    let tid = prefix[plan.nodes[i].last];
                                    for j in 0..i {
                                        if plan.nodes[j].last < length
                                            && plan.nodes[j].level == 0
                                            && prefix[plan.nodes[j].last] == tid
                                        {
                                            esum = prev[j];
                                        }
                                    }
                                }
                            } else if e == 4 {
                                for j in 0..i {
                                    if plan.nodes[j].last < length
                                        && plan.nodes[j].level == li
                                        && frame[j] == frame[i]
                                    {
                                        esum = prev[j];
                                    }
                                }
                            }
                            rel[i] += if gt > 0 { esum } else { -esum };
                        }
                    }
                    prev.copy_from_slice(&rel);
                }
                if let Some(scores) = &mut all_rel_scores {
                    scores.extend_from_slice(&rel);
                }
            }

            let mut terms = 1usize;
            for l in 0..stages {
                let gate = self.level_gates[b][l];
                if gate == 0 {
                    continue;
                }
                let mut top = [0usize; MAX_KEPT];
                let mut key_r = [0i32; MAX_KEPT];
                let mut key_d = [0u32; MAX_KEPT];
                let mut key_h = [0u32; MAX_KEPT];
                let mut found = 0usize;
                for node in 0..nodes_count {
                    if plan.nodes[node].level != l || plan.nodes[node].last + 1 >= length {
                        continue;
                    }
                    let (distance, hamming) = root_cost(query_frame, frame[node]);
                    let rscore = rel[node];
                    let mut at = found;
                    while at > 0 {
                        let o = top[at - 1];
                        let better = rscore > key_r[at - 1]
                            || (rscore == key_r[at - 1]
                                && (distance < key_d[at - 1]
                                    || (distance == key_d[at - 1]
                                        && (hamming < key_h[at - 1]
                                            || (hamming == key_h[at - 1]
                                                && (plan.nodes[node].last
                                                    > plan.nodes[o].last
                                                    || (plan.nodes[node].last
                                                        == plan.nodes[o].last
                                                        && node < o)))))));
                        if !better {
                            break;
                        }
                        if at < kept {
                            top[at] = top[at - 1];
                            key_r[at] = key_r[at - 1];
                            key_d[at] = key_d[at - 1];
                            key_h[at] = key_h[at - 1];
                        }
                        at -= 1;
                    }
                    if at < kept {
                        top[at] = node;
                        key_r[at] = rscore;
                        key_d[at] = distance;
                        key_h[at] = hamming;
                        if found < kept {
                            found += 1;
                        }
                    }
                }
                if found == 0 {
                    continue;
                }
                let mut sum = [0i32; 8];
                for &node in &top[..found] {
                    let act = usize::from(BFS[(frame[node] << 4) + fq]);
                    let carried = action(act, &payload[node]);
                    for d in 0..8 {
                        sum[d] += carried[d];
                    }
                }
                let m = snap(&sum, found);
                for d in 0..8 {
                    let v = i32::from(ROOTS[m][d]);
                    u[d] += if gate > 0 { v } else { -v };
                }
                terms += 1;
            }
            let out_root = snap(&u, terms);
            out.push(ROOTS[out_root].map(i32::from));
        }
        Ok((out, all_rel_scores))
    }

    fn address_distance(&self, q: &[[i32; 8]], cell: u32) -> u32 {
        let a = root_cost(&q[0], (cell >> 4) as usize).0;
        let b = root_cost(&q[q.len() - 1], (cell & 15) as usize).0;
        a + b
    }

    fn token_distance(&self, q: &[[i32; 8]], token: usize) -> u32 {
        let mut sum = 0u32;
        for (block_q, block_v) in q.iter().zip(&self.token_blocks[token]) {
            for d in 0..8 {
                sum += (block_q[d] - block_v[d]).unsigned_abs();
            }
        }
        sum
    }

    /// Evaluate a causal token prefix (`1..=context`) and return the read vector,
    /// admitted cells, top-8 ranked tokens, and top-1 prediction.
    pub fn evaluate_prefix(&self, prefix: &[u32]) -> Result<DiscretePrefixEvaluation> {
        let vocab = self.shape.vocabulary;
        let context = self.shape.context as usize;
        if prefix.is_empty() || prefix.len() > context {
            return Err(invalid("prefix length must be in 1..=context"));
        }
        if prefix.iter().any(|&tok| tok >= vocab) {
            return Err(invalid("prefix token ID outside vocabulary"));
        }
        let plan = build_row_plan(prefix.len())?;
        let (q_blocks, relation_scores) = match self.read_mode {
            DiscreteReadMode::Route => (self.route_read(prefix, &plan)?, None),
            DiscreteReadMode::Flock { kept } => {
                self.flock_or_relation_read(prefix, &plan, kept as usize, 0, false)?
            }
            DiscreteReadMode::Relation { kept, sweeps } => {
                self.flock_or_relation_read(prefix, &plan, kept as usize, sweeps, true)?
            }
        };

        let mut occupied = [false; 256];
        for &cell in &self.cells {
            occupied[cell as usize] = true;
        }
        let mut used = [false; 256];
        let mut admitted_cells = Vec::with_capacity(self.shape.kept_cells as usize);
        for _ in 0..self.shape.kept_cells {
            let mut best_cell = None;
            let mut best_cost = u32::MAX;
            for c in 0..256u32 {
                if occupied[c as usize] && !used[c as usize] {
                    let d = self.address_distance(&q_blocks, c);
                    if d < best_cost || (d == best_cost && best_cell.is_none_or(|bc| c < bc)) {
                        best_cell = Some(c);
                        best_cost = d;
                    }
                }
            }
            let Some(cell) = best_cell else {
                break;
            };
            used[cell as usize] = true;
            admitted_cells.push(cell);
        }

        let mut ranked = [u32::MAX; 8];
        let mut distance = [u32::MAX; 8];
        let mut found = 0usize;
        for token in 0..vocab as usize {
            let cell = self.cells[token];
            if !admitted_cells.contains(&cell) {
                continue;
            }
            let d = self.token_distance(&q_blocks, token);
            let tok = token as u32;
            let mut at = found;
            while at > 0 {
                let prev = at - 1;
                if !(d < distance[prev] || (d == distance[prev] && tok < ranked[prev])) {
                    break;
                }
                if at < 8 {
                    ranked[at] = ranked[prev];
                    distance[at] = distance[prev];
                }
                at -= 1;
            }
            if at < 8 {
                ranked[at] = tok;
                distance[at] = d;
                if found < 8 {
                    found += 1;
                }
            }
        }
        if found == 0 {
            return Err(invalid("writer admitted zero tokens"));
        }
        let top8 = ranked[..found].to_vec();
        let prediction = top8[0];
        let read_vector = q_blocks.iter().flatten().copied().collect();
        Ok(DiscretePrefixEvaluation {
            read_vector,
            admitted_cells,
            top8,
            prediction,
            relation_scores,
        })
    }

    /// Evaluate a causal token prefix and compute the exact writer loss numerators
    /// and rival counts against `target` (`0..vocabulary`).
    pub fn observe_target(&self, prefix: &[u32], target: u32) -> Result<DiscreteTargetObservation> {
        if target >= self.shape.vocabulary {
            return Err(invalid("target token ID outside vocabulary"));
        }
        let evaluation = self.evaluate_prefix(prefix)?;
        let blocks = self.shape.blocks as usize;
        let mut q_blocks = Vec::with_capacity(blocks);
        for b in 0..blocks {
            let mut coords = [0i32; 8];
            coords.copy_from_slice(&evaluation.read_vector[(b << 3)..((b + 1) << 3)]);
            q_blocks.push(coords);
        }

        let target_cell = self.cells[target as usize];
        let mut token_rivals = 0u32;
        let mut address_rivals = 0u32;
        let mut rival_distance = u32::MAX;
        let mut rival_address = u32::MAX;
        for &cell in &evaluation.admitted_cells {
            let cost = self.address_distance(&q_blocks, cell);
            if cell != target_cell {
                address_rivals += 1;
                if cost < rival_address {
                    rival_address = cost;
                }
            }
            for token in 0..self.shape.vocabulary as usize {
                if self.cells[token] == cell {
                    let d = self.token_distance(&q_blocks, token);
                    if (token as u32) != target {
                        token_rivals += 1;
                        if d < rival_distance {
                            rival_distance = d;
                        }
                    }
                }
            }
        }

        let width = self.shape.blocks << 3;
        let td = self.token_distance(&q_blocks, target as usize);
        let ad = self.address_distance(&q_blocks, target_cell);
        let token_hinge = if rival_distance == u32::MAX || width + td <= rival_distance {
            0
        } else {
            u64::from(width + td - rival_distance)
        };
        let address_hinge = if rival_address == u32::MAX || 8 + ad <= rival_address {
            0
        } else {
            u64::from(8 + ad - rival_address)
        };

        let mut occupied = [false; 256];
        for &cell in &self.cells {
            occupied[cell as usize] = true;
        }
        let mut rank = 1u32;
        for cell in 0..256u32 {
            if occupied[cell as usize] {
                let d = self.address_distance(&q_blocks, cell);
                if d < ad || (d == ad && cell < target_cell) {
                    rank += 1;
                }
            }
        }
        let rank_hinge = if rank > self.shape.kept_cells {
            u64::from(rank - self.shape.kept_cells)
        } else {
            0
        };

        let weighted_token = checked_scaled_add(token_hinge, 8)
            .ok_or_else(|| invalid("weighted token hinge overflow"))?;
        let weighted_address = checked_scaled_add(address_hinge, width)
            .ok_or_else(|| invalid("weighted address hinge overflow"))?;
        let weighted_rank = checked_scaled_add(rank_hinge, width)
            .ok_or_else(|| invalid("weighted rank hinge overflow"))?;
        let loss_ta = weighted_token
            .checked_add(weighted_address)
            .ok_or_else(|| invalid("loss_ta overflow"))?;
        let loss_tr = weighted_token
            .checked_add(weighted_rank)
            .ok_or_else(|| invalid("loss_tr overflow"))?;
        let selected_loss = match self.objective_policy {
            DiscreteObjectivePolicy::NearestAddressMarginV1 => loss_ta,
            DiscreteObjectivePolicy::OccupiedCellRankHingeV1 => loss_tr,
        };

        Ok(DiscreteTargetObservation {
            evaluation,
            token_rivals,
            address_rivals,
            token_hinge,
            address_hinge,
            rank_hinge,
            loss_ta,
            loss_tr,
            selected_loss,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: [i32; 8] = [2, 2, 0, 0, 0, 0, 0, 0];
    const NEG_A: [i32; 8] = [-2, -2, 0, 0, 0, 0, 0, 0];
    const C_ROOT: [i32; 8] = [-2, 2, 0, 0, 0, 0, 0, 0];
    const NEG_B: [i32; 8] = [0, 0, -2, 0, -2, 0, 0, 0];
    const B: [i32; 8] = [0, 0, 2, 0, 2, 0, 0, 0];
    const D_ROOT: [i32; 8] = [2, -2, 0, 0, 0, 0, 0, 0];

    fn push_section(blob: &mut Vec<u8>, bytes: &[u8]) -> SectionSpan {
        let pad = (ALIGN - blob.len() % ALIGN) % ALIGN;
        blob.resize(blob.len() + pad, 0);
        let offset = blob.len() as u64;
        blob.extend_from_slice(bytes);
        SectionSpan {
            offset,
            bytes: bytes.len() as u64,
        }
    }

    fn build_discrete_uorlut01(
        shape: DiscreteLutShape,
        tokens: &[[[i32; 8]; 2]],
        routes: &[u32],
        gates: &[i32],
    ) -> Vec<u8> {
        let table: Vec<i32> = tokens.iter().flatten().flatten().copied().collect();
        let cells: Vec<u32> = tokens
            .iter()
            .map(|blocks| {
                let r0 = root_id(&blocks[0]).unwrap() as u32;
                let r1 = root_id(&blocks[1]).unwrap() as u32;
                (r0 << 4) + r1
            })
            .collect();
        let mut blob = Vec::new();
        let mut tables = Vec::new();
        let table_bytes: Vec<u8> = table.iter().flat_map(|x| x.to_le_bytes()).collect();
        tables.push(SectionSpec {
            name: "discrete.table".to_owned(),
            kind: SectionKind::I32,
            len: table.len(),
            span: push_section(&mut blob, &table_bytes),
        });
        let cells_bytes: Vec<u8> = cells.iter().flat_map(|x| x.to_le_bytes()).collect();
        tables.push(SectionSpec {
            name: "discrete.cells".to_owned(),
            kind: SectionKind::U32,
            len: cells.len(),
            span: push_section(&mut blob, &cells_bytes),
        });
        let routes_bytes: Vec<u8> = routes.iter().flat_map(|x| x.to_le_bytes()).collect();
        tables.push(SectionSpec {
            name: "discrete.routes".to_owned(),
            kind: SectionKind::U32,
            len: routes.len(),
            span: push_section(&mut blob, &routes_bytes),
        });
        let gates_bytes: Vec<u8> = gates.iter().flat_map(|x| x.to_le_bytes()).collect();
        tables.push(SectionSpec {
            name: "discrete.gates".to_owned(),
            kind: SectionKind::I32,
            len: gates.len(),
            span: push_section(&mut blob, &gates_bytes),
        });
        let header = DiscreteLutHeader {
            schema: DISCRETE_SCHEMA.to_owned(),
            shape,
            matrices: Vec::new(),
            tables,
            source: DiscreteLutSource {
                snapshot_schema: SNAPSHOT_SCHEMA.to_owned(),
                snapshot_sha256: "00".repeat(32),
                operator_kappa: OPERATOR_KAPPA.to_owned(),
                composition_kappa: COMPOSITION_KAPPA.to_owned(),
                kernel_sha256: "11".repeat(32),
                source_sha256: "22".repeat(32),
                geometry_sha256: "33".repeat(32),
            },
        };
        let json = serde_json::to_vec(&header).unwrap();
        let mut out = Vec::with_capacity(16 + json.len() + ALIGN + blob.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&(json.len() as u64).to_le_bytes());
        out.extend_from_slice(&json);
        let pad = (ALIGN - out.len() % ALIGN) % ALIGN;
        out.resize(out.len() + pad, 0);
        out.extend_from_slice(&blob);
        out
    }

    #[test]
    fn route_writer_and_context_transport_match_cuda_qualification_witnesses() {
        // 1. Two-token exact writer witness (`qualification.rs::exact_writer`).
        let shape_2 = DiscreteLutShape {
            vocabulary: 2,
            blocks: 2,
            context: 1,
            kept_cells: 2,
            stages: 1,
            read_mode: "route".to_owned(),
            read_kept: 0,
            read_sweeps: 0,
            objective_policy: "nearest_address_margin_v1".to_owned(),
            read_initialization: "zero_gates_v1".to_owned(),
            seed: 7,
            epoch: 0,
        };
        let bytes_2 =
            build_discrete_uorlut01(shape_2.clone(), &[[A, A], [NEG_A, NEG_A]], &[0, 0], &[0, 0]);
        let model_2 = IntegerDiscreteModel::from_bytes(&bytes_2).unwrap();

        let obs0 = model_2.observe_target(&[0], 0).unwrap();
        assert_eq!(obs0.evaluation.prediction, 0);
        assert_eq!(obs0.evaluation.admitted_cells, vec![0, 17]);
        assert_eq!(obs0.evaluation.top8, vec![0, 1]);
        assert_eq!(obs0.token_rivals, 1);
        assert_eq!(obs0.address_rivals, 1);
        assert_eq!(obs0.selected_loss, 0);

        let obs1 = model_2.observe_target(&[1], 0).unwrap();
        assert_eq!(obs1.evaluation.prediction, 1);
        assert_eq!(obs1.evaluation.admitted_cells, vec![17, 0]);
        assert_eq!(obs1.evaluation.top8, vec![1, 0]);
        assert_eq!(obs1.token_rivals, 1);
        assert_eq!(obs1.address_rivals, 1);
        assert_eq!(obs1.selected_loss, 640);

        // Negative control: editing `discrete.table` (replacing token 0's endpoint block
        // with `NEG_A` and token 1's with `A`) changes the integer read vector, admitted cells,
        // and loss on the same prefix `[0]` and target `0`.
        let edited_bytes_2 =
            build_discrete_uorlut01(shape_2, &[[A, NEG_A], [NEG_A, A]], &[0, 0], &[0, 0]);
        let edited_model_2 = IntegerDiscreteModel::from_bytes(&edited_bytes_2).unwrap();
        let edited_obs0 = edited_model_2.observe_target(&[0], 0).unwrap();
        assert_ne!(
            edited_obs0.evaluation.read_vector,
            obs0.evaluation.read_vector
        );
        assert_eq!(edited_obs0.evaluation.admitted_cells, vec![1, 16]);
        assert_ne!(
            edited_obs0.evaluation.admitted_cells,
            obs0.evaluation.admitted_cells
        );

        // 2. Three-token context transport witness (`qualification.rs::context_transport`).
        let shape_3 = DiscreteLutShape {
            vocabulary: 3,
            blocks: 2,
            context: 3,
            kept_cells: 3,
            stages: 2,
            read_mode: "route".to_owned(),
            read_kept: 0,
            read_sweeps: 0,
            objective_policy: "nearest_address_margin_v1".to_owned(),
            read_initialization: "zero_gates_v1".to_owned(),
            seed: 7,
            epoch: 0,
        };
        let bytes_3 = build_discrete_uorlut01(
            shape_3,
            &[[A, A], [NEG_A, NEG_A], [B, B]],
            &[0, 0, 0, 0],
            &[0, 0, 0, 0],
        );
        let model_3 = IntegerDiscreteModel::from_bytes(&bytes_3).unwrap();

        let eval_a = model_3.evaluate_prefix(&[0, 1, 2]).unwrap();
        let expected_a: Vec<i32> = [NEG_B, NEG_B].iter().flatten().copied().collect();
        assert_eq!(eval_a.read_vector, expected_a);

        let eval_b = model_3.evaluate_prefix(&[0, 0, 2]).unwrap();
        let expected_b: Vec<i32> = [B, B].iter().flatten().copied().collect();
        assert_eq!(eval_b.read_vector, expected_b);
    }

    #[test]
    fn flock_and_relation_copy_successor_match_cuda_qualification_witnesses() {
        // C2 copy-successor witness (`relation_qualification.rs::relation_copy_successor_fixture`).
        let tokens = [
            [B, A],
            [A, D_ROOT],
            [NEG_A, NEG_B],
            [C_ROOT, A],
            [D_ROOT, A],
        ];
        let prefix = [0u32, 1, 2, 3, 4, 1];

        let flock_shape = DiscreteLutShape {
            vocabulary: 5,
            blocks: 2,
            context: 6,
            kept_cells: 2,
            stages: 2,
            read_mode: "flock".to_owned(),
            read_kept: 1,
            read_sweeps: 0,
            objective_policy: "nearest_address_margin_v1".to_owned(),
            read_initialization: "active_leaf_v1".to_owned(),
            seed: 7,
            epoch: 0,
        };
        let flock_bytes =
            build_discrete_uorlut01(flock_shape, &tokens, &[0, 0, 0, 0], &[0, 0, 1, 0]);
        let flock_model = IntegerDiscreteModel::from_bytes(&flock_bytes).unwrap();
        let flock_obs = flock_model.observe_target(&prefix, 2).unwrap();
        let expected_base_vec: Vec<i32> = [A, D_ROOT].iter().flatten().copied().collect();
        assert_eq!(flock_obs.evaluation.read_vector, expected_base_vec);
        assert_eq!(flock_obs.selected_loss, 640);

        let rel_shape = DiscreteLutShape {
            vocabulary: 5,
            blocks: 2,
            context: 6,
            kept_cells: 2,
            stages: 2,
            read_mode: "relation".to_owned(),
            read_kept: 1,
            read_sweeps: 1,
            objective_policy: "nearest_address_margin_v1".to_owned(),
            read_initialization: "active_leaf_v1".to_owned(),
            seed: 7,
            epoch: 0,
        };
        let mut base_gates = [0i32; 14];
        base_gates[2] = 1;
        let rel_base_bytes =
            build_discrete_uorlut01(rel_shape.clone(), &tokens, &[0, 0, 0, 0], &base_gates);
        let rel_base_model = IntegerDiscreteModel::from_bytes(&rel_base_bytes).unwrap();
        let rel_base_obs = rel_base_model.observe_target(&prefix, 2).unwrap();
        assert_eq!(rel_base_obs.evaluation.read_vector, expected_base_vec);
        assert_eq!(rel_base_obs.selected_loss, 640);

        // Activate SequentialSuccessor at level 0 (`gates[6] = +1`): selects node 2 (`tok_B`),
        // transports to `[A, B]`, and reduces loss on target 2 from 640 to 448.
        let mut active_gates = base_gates;
        active_gates[6] = 1;
        let rel_active_bytes =
            build_discrete_uorlut01(rel_shape, &tokens, &[0, 0, 0, 0], &active_gates);
        let rel_active_model = IntegerDiscreteModel::from_bytes(&rel_active_bytes).unwrap();
        let rel_active_obs = rel_active_model.observe_target(&prefix, 2).unwrap();
        let expected_active_vec: Vec<i32> = [A, B].iter().flatten().copied().collect();
        assert_eq!(rel_active_obs.evaluation.read_vector, expected_active_vec);
        assert_eq!(rel_active_obs.selected_loss, 448);
        let scores = rel_active_obs
            .evaluation
            .relation_scores
            .as_ref()
            .expect("relation scores");
        let b1 = &scores[8..16];
        assert_eq!([b1[0], b1[1], b1[2], b1[4], b1[5]], [0, 0, 8, 0, 4]);
    }

    #[test]
    fn loader_and_evaluator_refuse_corrupted_containers_and_invalid_prefixes() {
        let tokens = [[A, A], [NEG_A, NEG_A]];
        let shape = DiscreteLutShape {
            vocabulary: 2,
            blocks: 2,
            context: 2,
            kept_cells: 2,
            stages: 1,
            read_mode: "route".to_owned(),
            read_kept: 0,
            read_sweeps: 0,
            objective_policy: "occupied_cell_rank_hinge_v1".to_owned(),
            read_initialization: "zero_gates_v1".to_owned(),
            seed: 7,
            epoch: 0,
        };
        let valid = build_discrete_uorlut01(shape.clone(), &tokens, &[0, 0], &[0, 0]);
        let model = IntegerDiscreteModel::from_bytes(&valid).unwrap();
        assert!(model.evaluate_prefix(&[]).is_err());
        assert!(model.evaluate_prefix(&[0, 1, 0]).is_err());
        assert!(model.evaluate_prefix(&[2]).is_err());
        assert!(model.observe_target(&[0], 2).is_err());

        // Bad magic.
        let mut bad_magic = valid.clone();
        bad_magic[0] ^= 1;
        assert!(IntegerDiscreteModel::from_bytes(&bad_magic).is_err());

        // Non-zero header padding.
        let header_len = u64::from_le_bytes(valid[8..16].try_into().unwrap()) as usize;
        let unaligned = 16 + header_len;
        let data_start = unaligned + (ALIGN - unaligned % ALIGN) % ALIGN;
        if unaligned < data_start {
            let mut bad_pad = valid.clone();
            bad_pad[unaligned] = 1;
            assert!(IntegerDiscreteModel::from_bytes(&bad_pad).is_err());
        }

        // Corrupted cell index in discrete.cells.
        let mut bad_cells = valid.clone();
        let header: DiscreteLutHeader =
            serde_json::from_slice(&valid[16..16 + header_len]).unwrap();
        let cells_offset = data_start + (header.tables[1].span.offset as usize);
        bad_cells[cells_offset] ^= 1;
        assert!(IntegerDiscreteModel::from_bytes(&bad_cells).is_err());

        // Invalid stages in shape.
        let mut bad_shape = shape;
        bad_shape.stages = 2;
        let bad_stages = build_discrete_uorlut01(bad_shape, &tokens, &[0, 0], &[0, 0]);
        assert!(IntegerDiscreteModel::from_bytes(&bad_stages).is_err());
    }
}
