//! Deterministic integer RGBA-to-R4 image decomposition.
//!
//! The primitive accepts one exact top-left-origin RGBA buffer, partitions it
//! into non-empty rectangular patches, and computes signed fixed-point visual
//! statistics. It does not decode images, infer semantics, invoke a model, or
//! claim visual understanding.
//!
//! The default `parallel` feature evaluates independent patches through Rayon.
//! Rayon's indexed collection preserves grid order, so canonical bytes do not
//! depend on the worker count. Disable default features for a serial build.

#![forbid(unsafe_code)]

use std::fmt;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Maximum accepted width and height, in pixels.
pub const MAX_DIMENSION: u32 = 256;
/// Maximum accepted patch columns and rows.
pub const MAX_GRID: u16 = 32;
/// Canonical decomposition header magic and format version.
pub const CANONICAL_MAGIC: &[u8; 8] = b"R4IMG001";

/// Position of an out-of-range source dimension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Width,
    Height,
}

/// A rectangular grid of patches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grid {
    columns: u16,
    rows: u16,
}

impl Grid {
    /// Constructs a grid. [`decompose_rgba`] validates its size and relation to
    /// the source.
    pub const fn new(columns: u16, rows: u16) -> Self {
        Self { columns, rows }
    }

    /// Number of patch columns.
    pub const fn columns(self) -> u16 {
        self.columns
    }

    /// Number of patch rows.
    pub const fn rows(self) -> u16 {
        self.rows
    }
}

/// A rejection before any patch can be published.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VisionError {
    /// A source dimension is zero or exceeds [`MAX_DIMENSION`].
    DimensionOutOfRange { axis: Axis, value: u32, max: u32 },
    /// The RGBA slice is not exactly `width * height * 4` bytes.
    PixelBufferLength { expected: usize, actual: usize },
    /// The grid has a zero axis.
    EmptyGrid,
    /// A grid axis exceeds [`MAX_GRID`].
    GridOutOfRange { columns: u16, rows: u16, max: u16 },
    /// A grid axis would create a zero-area patch.
    GridLargerThanImage {
        columns: u16,
        rows: u16,
        width: u32,
        height: u32,
    },
    /// A bounded size or fixed-point calculation overflowed.
    ArithmeticOverflow,
}

impl fmt::Display for VisionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DimensionOutOfRange { axis, value, max } => {
                write!(f, "{axis:?} {value} is outside 1..={max}")
            }
            Self::PixelBufferLength { expected, actual } => {
                write!(
                    f,
                    "RGBA buffer has {actual} bytes; expected exactly {expected}"
                )
            }
            Self::EmptyGrid => f.write_str("grid columns and rows must both be non-zero"),
            Self::GridOutOfRange { columns, rows, max } => {
                write!(f, "grid {columns}x{rows} exceeds {max}x{max}")
            }
            Self::GridLargerThanImage {
                columns,
                rows,
                width,
                height,
            } => write!(
                f,
                "grid {columns}x{rows} would create an empty patch in {width}x{height} source"
            ),
            Self::ArithmeticOverflow => f.write_str("decomposition arithmetic overflow"),
        }
    }
}

impl std::error::Error for VisionError {}

/// One positioned patch with exact integer visual statistics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Patch {
    /// Zero-based patch column.
    pub column: u16,
    /// Zero-based patch row.
    pub row: u16,
    /// Inclusive left pixel coordinate.
    pub x0: u16,
    /// Inclusive top pixel coordinate.
    pub y0: u16,
    /// Exclusive right pixel coordinate.
    pub x1: u16,
    /// Exclusive bottom pixel coordinate.
    pub y1: u16,
    /// Rounded Q12 mean R, G and B, each in `0..=4096`.
    pub mean_rgb_q12: [u16; 3],
    /// Rec.601 luma at Q12 in `0..=4096`.
    pub luma_q12: u16,
    /// Signed Q12 `[R - luma, B - luma]`.
    pub chroma_q12: [i16; 2],
    /// Signed mean Q12 luma change `[horizontal, vertical]`.
    pub edges_q12: [i16; 2],
    /// Two signed Q12 R4 vectors: centered RGB plus centered luma, then
    /// chroma axes plus horizontal and vertical edges.
    pub r4_q12: [[i16; 4]; 2],
}

/// A validated image decomposition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decomposition {
    width: u16,
    height: u16,
    grid: Grid,
    source_hash: [u8; 32],
    patches: Vec<Patch>,
}

impl Decomposition {
    /// Source width in pixels.
    pub const fn width(&self) -> u16 {
        self.width
    }

    /// Source height in pixels.
    pub const fn height(&self) -> u16 {
        self.height
    }

    /// Validated patch grid.
    pub const fn grid(&self) -> Grid {
        self.grid
    }

    /// Patches in row-major grid order.
    pub fn patches(&self) -> &[Patch] {
        &self.patches
    }

    /// Blake3 digest of the exact RGBA input bytes.
    pub const fn source_hash(&self) -> [u8; 32] {
        self.source_hash
    }

    /// Lowercase hexadecimal source digest.
    pub fn source_hash_hex(&self) -> String {
        use fmt::Write as _;

        let mut hex = String::with_capacity(64);
        for byte in self.source_hash {
            write!(&mut hex, "{byte:02x}").expect("writing to String cannot fail");
        }
        hex
    }

    /// Versioned canonical bytes in row-major patch order.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(52 + self.patches.len() * 40);
        bytes.extend_from_slice(CANONICAL_MAGIC);
        bytes.extend_from_slice(&self.width.to_be_bytes());
        bytes.extend_from_slice(&self.height.to_be_bytes());
        bytes.extend_from_slice(&self.grid.columns.to_be_bytes());
        bytes.extend_from_slice(&self.grid.rows.to_be_bytes());
        bytes.extend_from_slice(&(self.patches.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&self.source_hash);
        for patch in &self.patches {
            for value in [patch.x0, patch.y0, patch.x1, patch.y1] {
                bytes.extend_from_slice(&value.to_be_bytes());
            }
            for value in patch.mean_rgb_q12.into_iter().chain([patch.luma_q12]) {
                bytes.extend_from_slice(&value.to_be_bytes());
            }
            for value in patch
                .chroma_q12
                .into_iter()
                .chain(patch.edges_q12)
                .chain(patch.r4_q12[0])
                .chain(patch.r4_q12[1])
            {
                bytes.extend_from_slice(&value.to_be_bytes());
            }
        }
        bytes
    }
}

/// Computes the exact RGBA byte count, rejecting arithmetic overflow.
pub fn rgba_byte_len(width: u32, height: u32) -> Result<usize, VisionError> {
    usize::try_from(width)
        .ok()
        .and_then(|w| usize::try_from(height).ok().and_then(|h| w.checked_mul(h)))
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(VisionError::ArithmeticOverflow)
}

/// Validates one RGBA buffer and decomposes it into deterministic R4 patches.
///
/// Pixels are top-left origin. Every accepted source pixel belongs to exactly
/// one patch. The alpha channel is covered by the source digest but does not
/// contribute to the declared RGB visual statistics.
pub fn decompose_rgba(
    width: u32,
    height: u32,
    rgba: &[u8],
    grid: Grid,
) -> Result<Decomposition, VisionError> {
    validate_dimension(Axis::Width, width)?;
    validate_dimension(Axis::Height, height)?;
    let expected = rgba_byte_len(width, height)?;
    if rgba.len() != expected {
        return Err(VisionError::PixelBufferLength {
            expected,
            actual: rgba.len(),
        });
    }
    validate_grid(grid, width, height)?;

    let positions: Vec<(u16, u16)> = (0..grid.rows)
        .flat_map(|row| (0..grid.columns).map(move |column| (column, row)))
        .collect();
    let evaluate = |(column, row)| patch_for(width, height, rgba, grid, column, row);

    #[cfg(feature = "parallel")]
    let patches = {
        let min_len = positions.len().div_ceil(rayon::current_num_threads());
        positions
            .into_par_iter()
            .with_min_len(min_len)
            .map(evaluate)
            .collect::<Result<Vec<_>, VisionError>>()?
    };
    #[cfg(not(feature = "parallel"))]
    let patches = positions
        .into_iter()
        .map(evaluate)
        .collect::<Result<Vec<_>, VisionError>>()?;

    Ok(Decomposition {
        width: width as u16,
        height: height as u16,
        grid,
        source_hash: *blake3::hash(rgba).as_bytes(),
        patches,
    })
}

fn validate_dimension(axis: Axis, value: u32) -> Result<(), VisionError> {
    if (1..=MAX_DIMENSION).contains(&value) {
        Ok(())
    } else {
        Err(VisionError::DimensionOutOfRange {
            axis,
            value,
            max: MAX_DIMENSION,
        })
    }
}

fn validate_grid(grid: Grid, width: u32, height: u32) -> Result<(), VisionError> {
    if grid.columns == 0 || grid.rows == 0 {
        return Err(VisionError::EmptyGrid);
    }
    if grid.columns > MAX_GRID || grid.rows > MAX_GRID {
        return Err(VisionError::GridOutOfRange {
            columns: grid.columns,
            rows: grid.rows,
            max: MAX_GRID,
        });
    }
    if u32::from(grid.columns) > width || u32::from(grid.rows) > height {
        return Err(VisionError::GridLargerThanImage {
            columns: grid.columns,
            rows: grid.rows,
            width,
            height,
        });
    }
    Ok(())
}

fn patch_for(
    width: u32,
    height: u32,
    rgba: &[u8],
    grid: Grid,
    column: u16,
    row: u16,
) -> Result<Patch, VisionError> {
    let x0 = partition(column, grid.columns, width)?;
    let x1 = partition(column + 1, grid.columns, width)?;
    let y0 = partition(row, grid.rows, height)?;
    let y1 = partition(row + 1, grid.rows, height)?;
    let area = (x1 - x0)
        .checked_mul(y1 - y0)
        .ok_or(VisionError::ArithmeticOverflow)?;

    let mut rgb_sum = [0u64; 3];
    let mut edge_h_sum = 0i64;
    let mut edge_h_pairs = 0u64;
    let mut edge_v_sum = 0i64;
    let mut edge_v_pairs = 0u64;
    let mut luma = [0u16; MAX_DIMENSION as usize];
    let mut next_luma = [0u16; MAX_DIMENSION as usize];
    for y in y0..y1 {
        let row_base = (y * width * 4) as usize;
        for x in x0..x1 {
            let at = row_base + (x * 4) as usize;
            let source_pixel = [rgba[at], rgba[at + 1], rgba[at + 2], rgba[at + 3]];
            for (sum, value) in rgb_sum.iter_mut().zip(&source_pixel[..3]) {
                *sum += u64::from(*value);
            }
            luma[(x - x0) as usize] = raw_luma(source_pixel);
        }
        let patch_width = (x1 - x0) as usize;
        for x in 0..patch_width.saturating_sub(1) {
            edge_h_sum += i64::from(luma[x + 1]) - i64::from(luma[x]);
            edge_h_pairs += 1;
        }
        if y + 1 < y1 {
            let next_base = ((y + 1) * width * 4) as usize;
            for (index, x) in (x0..x1).enumerate() {
                let at = next_base + (x * 4) as usize;
                next_luma[index] = raw_luma([rgba[at], rgba[at + 1], rgba[at + 2], rgba[at + 3]]);
            }
            for (current, below) in luma[..patch_width].iter().zip(&next_luma[..patch_width]) {
                edge_v_sum += i64::from(*below) - i64::from(*current);
                edge_v_pairs += 1;
            }
        }
    }

    let mean_rgb_q12 = [
        mean_q12(rgb_sum[0], area)?,
        mean_q12(rgb_sum[1], area)?,
        mean_q12(rgb_sum[2], area)?,
    ];
    let luma_q12 = weighted_luma_q12(mean_rgb_q12)?;
    let chroma_q12 = [
        signed_i16(i32::from(mean_rgb_q12[0]) - i32::from(luma_q12))?,
        signed_i16(i32::from(mean_rgb_q12[2]) - i32::from(luma_q12))?,
    ];
    let edges_q12 = [
        edge_q12(edge_h_sum, edge_h_pairs)?,
        edge_q12(edge_v_sum, edge_v_pairs)?,
    ];
    let r4_q12 = [
        [
            centered(mean_rgb_q12[0])?,
            centered(mean_rgb_q12[1])?,
            centered(mean_rgb_q12[2])?,
            centered(luma_q12)?,
        ],
        [chroma_q12[0], chroma_q12[1], edges_q12[0], edges_q12[1]],
    ];

    Ok(Patch {
        column,
        row,
        x0: x0 as u16,
        y0: y0 as u16,
        x1: x1 as u16,
        y1: y1 as u16,
        mean_rgb_q12,
        luma_q12,
        chroma_q12,
        edges_q12,
        r4_q12,
    })
}

fn partition(index: u16, parts: u16, size: u32) -> Result<u32, VisionError> {
    let numerator = u32::from(index)
        .checked_mul(size)
        .ok_or(VisionError::ArithmeticOverflow)?;
    Ok(numerator / u32::from(parts))
}

fn mean_q12(sum: u64, area: u32) -> Result<u16, VisionError> {
    let denominator = u64::from(area)
        .checked_mul(255)
        .ok_or(VisionError::ArithmeticOverflow)?;
    let numerator = sum
        .checked_mul(4096)
        .and_then(|value| value.checked_add(denominator / 2))
        .ok_or(VisionError::ArithmeticOverflow)?;
    u16::try_from(numerator / denominator).map_err(|_| VisionError::ArithmeticOverflow)
}

fn weighted_luma_q12(rgb: [u16; 3]) -> Result<u16, VisionError> {
    let numerator = 299u64
        .checked_mul(u64::from(rgb[0]))
        .and_then(|value| value.checked_add(587 * u64::from(rgb[1])))
        .and_then(|value| value.checked_add(114 * u64::from(rgb[2])))
        .and_then(|value| value.checked_add(500))
        .ok_or(VisionError::ArithmeticOverflow)?;
    u16::try_from(numerator / 1000).map_err(|_| VisionError::ArithmeticOverflow)
}

fn raw_luma(pixel: [u8; 4]) -> u16 {
    ((299 * u32::from(pixel[0]) + 587 * u32::from(pixel[1]) + 114 * u32::from(pixel[2])) / 1000)
        as u16
}

fn edge_q12(sum: i64, pairs: u64) -> Result<i16, VisionError> {
    if pairs == 0 {
        return Ok(0);
    }
    let denominator = i64::try_from(pairs).map_err(|_| VisionError::ArithmeticOverflow)? * 255;
    let numerator = sum
        .checked_mul(4096)
        .ok_or(VisionError::ArithmeticOverflow)?;
    let rounded = if numerator >= 0 {
        numerator
            .checked_add(denominator / 2)
            .ok_or(VisionError::ArithmeticOverflow)?
            / denominator
    } else {
        -((-numerator)
            .checked_add(denominator / 2)
            .ok_or(VisionError::ArithmeticOverflow)?
            / denominator)
    };
    i16::try_from(rounded).map_err(|_| VisionError::ArithmeticOverflow)
}

fn centered(value: u16) -> Result<i16, VisionError> {
    signed_i16(i32::from(value) - 2048)
}

fn signed_i16(value: i32) -> Result<i16, VisionError> {
    i16::try_from(value).map_err(|_| VisionError::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_length_overflow_is_rejected() {
        assert_eq!(
            rgba_byte_len(u32::MAX, u32::MAX),
            Err(VisionError::ArithmeticOverflow)
        );
    }
}
