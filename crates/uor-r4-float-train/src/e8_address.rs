//! The E8 address memory (one-model-end-to-end, claude-346): a learned table inside the trunk,
//! read by the state's own address.
//!
//! Before this, the trunk's learned values outside the token table were a few thousand rotations,
//! margins and weights: nothing learned stood between a state and the next. In
//! `StackModel::trunk`, after each layer's Hamilton block, with `x` the residual of one position
//! (`width` coordinates, `width / 8` blocks of 8):
//!
//! ```text
//! x ← x + Σ_blocks V[layer][block][cell(block of x)]
//! ```
//!
//! 1. **Address.** A block is a point of R⁸ and its cell is the nearest of E8's 240 roots: the
//!    112 integer roots ±eᵢ ± eⱼ and the 128 half roots (±½)⁸ with an even number of minus signs.
//!    Every root has squared length 2, so the nearest root is the one with the largest inner
//!    product, and that is found from the block's sorted magnitudes alone ([`e8_address`]): no
//!    projection is learned and no state is multiplied by a table.
//! 2. **Rows.** One table per layer of `blocks × cells` rows, each `width` wide. At width 256 and
//!    240 cells that is 1,966,080 values a layer. A position reads one row per block.
//! 3. **Credit.** A row read takes the gradient of the state it was added to. The state takes a
//!    gradient through the address only near a wall. With `d` the block's distance to the wall
//!    between its cell and the runner-up, `ρ` the position's RMS coordinate (detached) and `τ`
//!    the band, `λ = ½·max(0, 1 − d/(τρ))` and the forward adds
//!    `(V[across] − V[cell])·(λ − λ.detach())`: exactly zero forward. Its gradient is that of the
//!    read blended linearly across the wall, so descent moves a block toward the wall when the row
//!    across it would have served the loss better.
//! 4. **Served form.** Rows are 16-bit integers on the residual stream's own 2⁻¹⁶ grid
//!    ([`ROW_SCALE`]): serving a row is `x_i32 += row_i16`. [`AddressMemory::round_as_exported`]
//!    puts the float rows on that grid and [`AddressMemory::hold_in_exported_range`] keeps the
//!    float master inside what 16 bits hold.
//!
//! The arms ([`AddressArm`]) keep the table and change the address: `sign` (the block's 8 signs,
//! 256 cells), `collapse` (one cell) and `token` (a seeded hash of the position's own input token,
//! so no state enters the address). The memory is position-local: a position's read depends on
//! its own state and token only, so the decode cache steps through it with nothing carried.
//!
//! The memory is active for the thread that sets it ([`set_address_memory`]); every caller of
//! `StackModel::hidden` on that thread gets it. Its optimizer is
//! [`crate::flock_read::FlockAdamW`], stepped at [`AddressMemory::rate`] times the trunk's rate.
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

use candle_core::{DType, Device, Tensor, Var};
use serde::Serialize;

use crate::{invalid, Result};

/// Coordinates in one block: a point of R⁸.
pub const BLOCK: usize = 8;
/// E8's roots: a block's cells under [`AddressArm::E8`] and [`AddressArm::Token`].
pub const E8_CELLS: usize = 240;
/// The integer roots ±eᵢ ± eⱼ come first: 28 pairs by 4 sign choices.
const INTEGER_CELLS: usize = 112;
/// A block's sign patterns: its cells under [`AddressArm::Sign`].
const SIGN_CELLS: usize = 256;
/// The served grid: a row is a 16-bit integer over this scale, the residual stream's own Q16.
pub const ROW_SCALE: f64 = 65_536.0;
/// The largest row step 16 bits hold, kept symmetric.
const ROW_STEPS: f64 = 32_767.0;
/// The TRAIN windows the trainer forwards for each checkpoint's census.
pub const CENSUS_WINDOWS: usize = 16;
const FILE: &str = "address_memory.safetensors";

thread_local! {
    static ACTIVE: RefCell<Option<AddressMemory>> = const { RefCell::new(None) };
    static CAPTURE: RefCell<Option<Vec<Reading>>> = const { RefCell::new(None) };
}

/// Make `memory` the address memory `StackModel::hidden` applies on this thread, or none.
pub fn set_address_memory(memory: Option<AddressMemory>) {
    ACTIVE.with(|cell| *cell.borrow_mut() = memory);
}

/// The address memory `StackModel::hidden` applies on this thread, if one is set. It shares its
/// variables with every other copy.
pub fn address_memory() -> Option<AddressMemory> {
    ACTIVE.with(|cell| cell.borrow().clone())
}

/// What names a block's cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AddressArm {
    /// The nearest of E8's 240 roots: the mechanism.
    E8,
    /// The block's 8 signs: the plainest partition of R⁸, 256 cells.
    Sign,
    /// One cell: a learned offset per layer, with no address.
    Collapse,
    /// A seeded hash of the position's input token, the layer and the block, into 240 cells: the
    /// same table with no state in the address.
    Token,
}

impl AddressArm {
    /// Every arm, in the order of their file codes.
    pub const ALL: [Self; 4] = [Self::E8, Self::Sign, Self::Collapse, Self::Token];

    /// `e8`, `sign`, `collapse` or `token`.
    pub fn parse(text: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|arm| arm.name() == text)
            .ok_or_else(|| {
                invalid(format!(
                    "--address-memory wants e8, sign, collapse or token, not {text}"
                ))
            })
    }

    /// The arm's name on the command line and in `config.json`.
    pub fn name(self) -> &'static str {
        match self {
            Self::E8 => "e8",
            Self::Sign => "sign",
            Self::Collapse => "collapse",
            Self::Token => "token",
        }
    }

    /// The cells one block chooses among.
    pub fn cells(self) -> usize {
        match self {
            Self::E8 | Self::Token => E8_CELLS,
            Self::Sign => SIGN_CELLS,
            Self::Collapse => 1,
        }
    }

    /// The band `τ` when none is given: the median of a block's wall distance over `ρ` when the
    /// state's coordinates are independent Gaussians, so half the blocks of such a state sit
    /// inside the band (`the_default_bands_are_the_gaussian_medians`). Zero, no wall, for the arms
    /// whose address has none.
    pub fn default_band(self) -> f64 {
        match self {
            Self::E8 => E8_BAND,
            Self::Sign => SIGN_BAND,
            Self::Collapse | Self::Token => 0.0,
        }
    }

    fn code(self) -> u32 {
        Self::ALL.iter().position(|&arm| arm == self).unwrap_or(0) as u32
    }

    fn from_code(code: u32) -> Result<Self> {
        Self::ALL
            .get(code as usize)
            .copied()
            .ok_or_else(|| invalid(format!("an address memory of unknown arm {code}")))
    }
}

/// [`AddressArm::default_band`] for the E8 address.
const E8_BAND: f64 = 0.141;
/// [`AddressArm::default_band`] for the sign address.
const SIGN_BAND: f64 = 0.104;

/// The memory: one table of rows per layer, and how a block names its row.
#[derive(Clone)]
pub struct AddressMemory {
    arm: AddressArm,
    width: usize,
    band: f64,
    seed: u64,
    /// Per layer, `(blocks × cells, width)`: block `b`'s cell `c` is row `b × cells + c`.
    rows: Vec<Var>,
}

/// A batch of blocks' addresses: each block's cell as an f32 whole number, and its wall when the
/// address has one.
struct Address {
    cell: Tensor,
    wall: Option<Wall>,
}

/// The nearest wall of a block's cell: the cell across it, and the block's live distance to it.
struct Wall {
    across: Tensor,
    distance: Tensor,
}

/// One forward's addresses at one layer, taken while a census is open.
struct Reading {
    layer: usize,
    index: Vec<u32>,
    in_band: usize,
}

impl AddressMemory {
    /// A memory for a trunk of `layers` layers and `width` coordinates, every row zero, so the
    /// trunk starts unchanged. `band` is `τ`, the arm's default when none; 0 is no wall. `seed`
    /// keys [`AddressArm::Token`]'s hash.
    pub fn new(
        arm: AddressArm,
        layers: usize,
        width: usize,
        band: Option<f64>,
        seed: u64,
        device: &Device,
    ) -> Result<Self> {
        if layers == 0 || width == 0 || width % BLOCK != 0 {
            return Err(invalid(format!(
                "the address memory wants at least one layer and a width in whole blocks of \
                 {BLOCK}, not {layers} layers of width {width}"
            )));
        }
        let band = band.unwrap_or(arm.default_band());
        if !band.is_finite() || band < 0.0 {
            return Err(invalid(format!(
                "--address-band wants a band of at least 0, not {band}"
            )));
        }
        let shape = (width / BLOCK * arm.cells(), width);
        let rows = (0..layers)
            .map(|_| Ok(Var::zeros(shape, DType::F32, device)?))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            arm,
            width,
            band,
            seed,
            rows,
        })
    }

    /// The address arm.
    pub fn arm(&self) -> AddressArm {
        self.arm
    }

    /// The band `τ`; 0 is no wall.
    pub fn band(&self) -> f64 {
        self.band
    }

    /// The layers the memory holds a table for.
    pub fn layers(&self) -> usize {
        self.rows.len()
    }

    /// Every learned value: the layers' tables.
    pub fn variables(&self) -> Vec<&Var> {
        self.rows.iter().collect()
    }

    /// The rows' step over the trunk's: each of a position's rows, one per block, takes the same
    /// gradient, so at one over the block count their sum moves at the trunk's rate.
    pub fn rate(&self) -> f64 {
        1.0 / (self.width / BLOCK) as f64
    }

    /// The residual `(batch, time, width)` of the windows `ids` with each position's rows added
    /// at `layer`. Returns a residual of the same shape.
    pub fn apply(&self, layer: usize, residual: &Tensor, ids: &[u32]) -> Result<Tensor> {
        let (batch, time, width) = residual.dims3()?;
        let positions = batch * time;
        if width != self.width {
            return Err(invalid(format!(
                "the address memory holds rows of width {}, and the residual is {width} wide",
                self.width
            )));
        }
        if residual.dtype() != DType::F32 {
            return Err(invalid("the address memory reads an f32 residual"));
        }
        if ids.len() != positions {
            return Err(invalid("the address memory needs every position's token"));
        }
        let table = self
            .rows
            .get(layer)
            .ok_or_else(|| invalid(format!("the address memory has no table for layer {layer}")))?
            .as_tensor();
        let device = residual.device();
        let blocks = width / BLOCK;
        let cells = self.arm.cells();
        let entries = positions * blocks;
        let points = residual.reshape((positions, width))?;
        let address = match self.arm {
            AddressArm::E8 => e8_address(&points.reshape((entries, BLOCK))?)?,
            AddressArm::Sign => sign_address(&points.reshape((entries, BLOCK))?)?,
            AddressArm::Collapse => Address {
                cell: Tensor::zeros(entries, DType::F32, device)?,
                wall: None,
            },
            AddressArm::Token => Address {
                cell: token_cells(self.seed, layer, ids, blocks, device)?,
                wall: None,
            },
        };
        let read = read_rows(table, &address.cell, positions, blocks, cells)?;
        let mut added = read.sum(1)?;
        let mut in_band = None;
        if let (Some(wall), true) = (&address.wall, self.band > 0.0) {
            // τρ, with ρ the position's RMS coordinate. The small constant keeps a zero state's
            // quotient finite.
            let reach = points
                .detach()
                .sqr()?
                .mean_keepdim(1)?
                .sqrt()?
                .affine(self.band, 1e-12)?;
            // λ = ½·max(0, 1 − d/(τρ)): ½ on the wall, 0 from τρ away.
            let credit = wall
                .distance
                .reshape((positions, blocks))?
                .broadcast_div(&reach)?
                .affine(-0.5, 0.5)?
                .relu()?;
            // λ − λ̄ is exactly 0 forward, and its gradient is λ's.
            let live = credit.sub(&credit.detach())?;
            let swing = read_rows(table, &wall.across, positions, blocks, cells)?
                .sub(&read)?
                .detach();
            added = added.add(&swing.broadcast_mul(&live.unsqueeze(2)?)?.sum(1)?)?;
            in_band = Some(credit);
        }
        if CAPTURE.with(|cell| cell.borrow().is_some()) {
            let in_band = match in_band {
                Some(credit) => credit
                    .gt(0f64)?
                    .to_dtype(DType::F32)?
                    .sum_all()?
                    .to_scalar::<f32>()? as usize,
                None => 0,
            };
            // Block b's cell c is row b × cells + c. Whole numbers below 2^24 are exact in f32.
            let offsets: Vec<f32> = (0..entries)
                .map(|entry| ((entry % blocks) * cells) as f32)
                .collect();
            let offsets = Tensor::from_vec(offsets, entries, device)?;
            let reading = Reading {
                layer,
                index: address
                    .cell
                    .add(&offsets)?
                    .to_dtype(DType::U32)?
                    .to_vec1::<u32>()?,
                in_band,
            };
            CAPTURE.with(|cell| {
                if let Some(readings) = cell.borrow_mut().as_mut() {
                    readings.push(reading);
                }
            });
        }
        Ok(points.add(&added)?.reshape((batch, time, width))?)
    }

    /// Keep every float row inside what the served 16 bits hold, so the float master cannot run
    /// on past the export's range.
    pub fn hold_in_exported_range(&self) -> Result<()> {
        let limit = ROW_STEPS / ROW_SCALE;
        for rows in &self.rows {
            rows.set(&rows.as_tensor().detach().clamp(-limit, limit)?)?;
        }
        Ok(())
    }

    /// Put every row on the served grid, with the residual stream's own rounding
    /// (`hamilton_sheet::q16_residual`): `floor(v × 2¹⁶ + ½) / 2¹⁶`, held to 16 bits.
    pub fn round_as_exported(&self) -> Result<()> {
        for rows in &self.rows {
            rows.set(&Self::steps(rows)?.affine(1.0 / ROW_SCALE, 0.0)?)?;
        }
        Ok(())
    }

    /// One layer's rows in whole steps of the served grid: `floor(v × 2¹⁶ + ½)`, held to 16 bits.
    fn steps(rows: &Var) -> Result<Tensor> {
        Ok(rows
            .as_tensor()
            .detach()
            .affine(ROW_SCALE, 0.5)?
            .floor()?
            .clamp(-ROW_STEPS, ROW_STEPS)?)
    }

    /// The table as a served stack holds it (claude-366): every row in the whole steps
    /// [`Self::round_as_exported`] puts it on, taken by the same arithmetic, so the artifact's
    /// rows are the rows the served evaluation read. The memory itself is left as it is. Only
    /// the `e8` and `token` arms have a served form. The band is not carried: it shapes the
    /// gradient at a wall and adds nothing to the value `apply` returns, so the served read has
    /// none.
    pub fn image(&self) -> Result<uor_r4_integer::fbs_address::AddressImage> {
        use uor_r4_integer::fbs_address as served;
        let arm = match self.arm {
            AddressArm::E8 => served::AddressArm::E8,
            AddressArm::Token => served::AddressArm::Token,
            other => {
                return Err(invalid(format!(
                    "the {} arm has no served form",
                    other.name()
                )))
            }
        };
        let mut layers = Vec::with_capacity(self.rows.len());
        for rows in &self.rows {
            let steps = Self::steps(rows)?.flatten_all()?.to_vec1::<f32>()?;
            layers.push(steps.into_iter().map(|step| step as i16).collect());
        }
        served::AddressImage::checked(arm, self.seed, self.width, layers).map_err(invalid)
    }

    /// Write the memory to `directory/address_memory.safetensors`: each layer's table as
    /// `rows.NN`, with the arm, the band and the seed.
    pub fn save(&self, directory: &Path) -> Result<()> {
        let host = Device::Cpu;
        let mut tensors: HashMap<String, Tensor> = HashMap::new();
        for (layer, rows) in self.rows.iter().enumerate() {
            tensors.insert(format!("rows.{layer:02}"), rows.as_tensor().detach());
        }
        tensors.insert("arm".to_owned(), Tensor::new(&[self.arm.code()], &host)?);
        tensors.insert("band".to_owned(), Tensor::new(&[self.band], &host)?);
        let seed = [(self.seed >> 32) as u32, self.seed as u32];
        tensors.insert("seed".to_owned(), Tensor::new(&seed, &host)?);
        candle_core::safetensors::save(&tensors, directory.join(FILE))?;
        Ok(())
    }

    /// The memory saved in `directory`, or none when the checkpoint has no address memory.
    pub fn load(directory: &Path, device: &Device) -> Result<Option<Self>> {
        let path = directory.join(FILE);
        if !path.exists() {
            return Ok(None);
        }
        let tensors = candle_core::safetensors::load(&path, device)?;
        let named = |name: &str| {
            tensors
                .get(name)
                .ok_or_else(|| invalid(format!("{}: no tensor {name}", path.display())))
        };
        let short = || invalid(format!("{}: a short header tensor", path.display()));
        let arm = named("arm")?.to_vec1::<u32>()?;
        let arm = AddressArm::from_code(*arm.first().ok_or_else(short)?)?;
        let band = *named("band")?.to_vec1::<f64>()?.first().ok_or_else(short)?;
        let seed = named("seed")?.to_vec1::<u32>()?;
        let &[high, low] = seed.as_slice() else {
            return Err(short());
        };
        let mut rows = Vec::new();
        while let Some(table) = tensors.get(&format!("rows.{:02}", rows.len())) {
            rows.push(Var::from_tensor(table)?);
        }
        let Some(first) = rows.first() else {
            return Err(invalid(format!("{}: no layer's rows", path.display())));
        };
        let (count, width) = first.as_tensor().dims2()?;
        if width == 0 || width % BLOCK != 0 || count != width / BLOCK * arm.cells() {
            return Err(invalid(format!(
                "{}: rows of shape ({count}, {width}) are not the {} arm's",
                path.display(),
                arm.name()
            )));
        }
        if rows
            .iter()
            .any(|table| table.as_tensor().dims() != [count, width])
        {
            return Err(invalid(format!(
                "{}: the layers' tables differ in shape",
                path.display()
            )));
        }
        if !band.is_finite() || band < 0.0 {
            return Err(invalid(format!("{}: band {band}", path.display())));
        }
        Ok(Some(Self {
            arm,
            width,
            band,
            seed: (u64::from(high) << 32) | u64::from(low),
            rows,
        }))
    }

    /// The memory with every row drawn from `seed` on the served grid, within ±1/16: a test's
    /// stand-in for a trained table.
    #[cfg(test)]
    pub(crate) fn randomized(self, seed: u64) -> Result<Self> {
        let mut state = seed;
        for rows in &self.rows {
            let (count, width) = rows.as_tensor().dims2()?;
            let values: Vec<f32> = (0..count * width)
                .map(|_| {
                    state = mix(state);
                    ((state % 8_193) as f32 - 4_096.0) / ROW_SCALE as f32
                })
                .collect();
            rows.set(&Tensor::from_vec(
                values,
                (count, width),
                rows.as_tensor().device(),
            )?)?;
        }
        Ok(self)
    }
}

/// One layer of a census ([`Census::read`]).
#[derive(Clone, Debug, Serialize)]
pub struct LayerCensus {
    pub layer: usize,
    /// The cells read at least once, a mean over blocks.
    pub cells_used_mean: f64,
    /// The same for the block that used the fewest.
    pub cells_used_min: usize,
    /// The largest share of one block's reads that went to a single cell.
    pub largest_cell_share: f64,
    /// The share of reads whose cell differs from the last census's, at the same position and
    /// block; none at the first census.
    pub moved_share: Option<f64>,
    /// The share of reads inside the band.
    pub in_band_share: f64,
    /// The mean size of a row value.
    pub row_mean_abs: f64,
}

/// The address memory's occupancy, movement and stability over the same windows, checkpoint
/// after checkpoint.
#[derive(Default)]
pub struct Census {
    previous: Vec<Vec<u32>>,
}

impl Census {
    /// Run `forward`, which forwards the same windows in the same order at every call, and
    /// describe the rows `memory` read at each layer.
    pub fn read(
        &mut self,
        memory: &AddressMemory,
        forward: impl FnOnce() -> Result<()>,
    ) -> Result<Vec<LayerCensus>> {
        CAPTURE.with(|cell| *cell.borrow_mut() = Some(Vec::new()));
        let ran = forward();
        let readings = CAPTURE
            .with(|cell| cell.borrow_mut().take())
            .unwrap_or_default();
        ran?;
        let cells = memory.arm.cells();
        let blocks = memory.width / BLOCK;
        let mut layers = Vec::with_capacity(memory.rows.len());
        let mut now = Vec::with_capacity(memory.rows.len());
        for (layer, rows) in memory.rows.iter().enumerate() {
            let mut index = Vec::new();
            let mut in_band = 0usize;
            for reading in readings.iter().filter(|reading| reading.layer == layer) {
                index.extend_from_slice(&reading.index);
                in_band += reading.in_band;
            }
            if index.is_empty() {
                return Err(invalid(format!(
                    "the address census forwarded nothing through layer {layer}"
                )));
            }
            let row_mean_abs = f64::from(
                rows.as_tensor()
                    .detach()
                    .abs()?
                    .mean_all()?
                    .to_scalar::<f32>()?,
            );
            let before = self.previous.get(layer).map(Vec::as_slice);
            layers.push(layer_census(
                layer,
                &index,
                in_band,
                blocks,
                cells,
                before,
                row_mean_abs,
            ));
            now.push(index);
        }
        self.previous = now;
        Ok(layers)
    }
}

/// One layer's census from the rows its reads named: `index` holds row `block × cells + cell`
/// for every read, `in_band` counts the reads inside the band, and `before` is the last census's
/// `index`, compared read for read when it covers the same reads.
fn layer_census(
    layer: usize,
    index: &[u32],
    in_band: usize,
    blocks: usize,
    cells: usize,
    before: Option<&[u32]>,
    row_mean_abs: f64,
) -> LayerCensus {
    let mut counts = vec![0usize; blocks * cells];
    for &row in index {
        counts[row as usize] += 1;
    }
    let per_block = index.len() / blocks;
    let (mut used_sum, mut used_min, mut largest) = (0usize, usize::MAX, 0usize);
    for block in counts.chunks(cells) {
        let used = block.iter().filter(|&&count| count > 0).count();
        used_sum += used;
        used_min = used_min.min(used);
        largest = largest.max(block.iter().copied().max().unwrap_or(0));
    }
    let moved_share = before
        .filter(|before| before.len() == index.len())
        .map(|before| {
            let moved = before.iter().zip(index).filter(|(a, b)| a != b).count();
            moved as f64 / index.len().max(1) as f64
        });
    LayerCensus {
        layer,
        cells_used_mean: used_sum as f64 / blocks as f64,
        cells_used_min: used_min,
        largest_cell_share: largest as f64 / per_block.max(1) as f64,
        moved_share,
        in_band_share: in_band as f64 / index.len().max(1) as f64,
        row_mean_abs,
    }
}

/// Each block's coordinates by size.
struct Sorted {
    /// `(n, 8)`: the coordinates' magnitudes, live.
    magnitude: Tensor,
    /// `(n, 8)`: 1 where a coordinate is below zero.
    negative: Tensor,
    /// `(n, 8)`: each coordinate's place by magnitude, 0 the largest. Equal magnitudes take
    /// their places in coordinate order, so the places are always a permutation.
    rank: Tensor,
}

impl Sorted {
    /// The blocks `(n, 8)` ranked. The places come from compares against every other coordinate,
    /// 64 a block: no sort.
    fn of(blocks: &Tensor) -> Result<Self> {
        let magnitude = blocks.abs()?;
        let negative = blocks.lt(0f64)?.to_dtype(DType::F32)?;
        let fixed = magnitude.detach();
        let mine = fixed.unsqueeze(2)?;
        let other = fixed.unsqueeze(1)?;
        let larger = other.broadcast_gt(&mine)?.to_dtype(DType::F32)?;
        let equal = other.broadcast_eq(&mine)?.to_dtype(DType::F32)?;
        let earlier: Vec<f32> = (0..BLOCK)
            .flat_map(|i| (0..BLOCK).map(move |j| if j < i { 1.0 } else { 0.0 }))
            .collect();
        let earlier = Tensor::from_vec(earlier, (1, BLOCK, BLOCK), blocks.device())?;
        let rank = larger.add(&equal.broadcast_mul(&earlier)?)?.sum(2)?;
        Ok(Self {
            magnitude,
            negative,
            rank,
        })
    }

    /// `(n, 8)`: 1 at the coordinate holding `place`, 0 elsewhere.
    fn at(&self, place: usize) -> Result<Tensor> {
        Ok(self.rank.eq(place as f64)?.to_dtype(DType::F32)?)
    }
}

/// `(n)`: `values` at each block's coordinate marked by `hot`.
fn pick(values: &Tensor, hot: &Tensor) -> Result<Tensor> {
    Ok(values.broadcast_mul(hot)?.sum(1)?)
}

/// `mask·a + (1 − mask)·b` for a mask of zeros and ones: exactly `a` or `b`.
fn choose(mask: &Tensor, a: &Tensor, b: &Tensor) -> Result<Tensor> {
    Ok(mask.mul(a)?.add(&mask.affine(-1.0, 1.0)?.mul(b)?)?)
}

/// The cell of the integer root on coordinates `ca` and `cb`, with `na` and `nb` 1 where its
/// entry is −1. With `lo < hi` the two coordinates, the pair's number counts the pairs before it
/// in order, `lo·(15 − lo)/2 + hi − lo − 1`, and the cell is `4·pair + 2·n_lo + n_hi`.
fn integer_cell(ca: &Tensor, na: &Tensor, cb: &Tensor, nb: &Tensor) -> Result<Tensor> {
    let a_first = ca.lt(cb)?.to_dtype(DType::F32)?;
    let lo = choose(&a_first, ca, cb)?;
    let hi = ca.add(cb)?.sub(&lo)?;
    let n_lo = choose(&a_first, na, nb)?;
    let n_hi = na.add(nb)?.sub(&n_lo)?;
    let pair = lo
        .mul(&lo.affine(-1.0, 15.0)?)?
        .affine(0.5, 0.0)?
        .add(&hi)?
        .sub(&lo)?
        .affine(1.0, -1.0)?;
    Ok(pair
        .affine(4.0, 0.0)?
        .add(&n_lo.affine(2.0, 0.0)?)?
        .add(&n_hi)?)
}

/// Each block's nearest E8 root and the runner-up. With `m1 ≥ … ≥ m8` the block's magnitudes
/// and `S` their sum, the best roots' inner products with the block are:
/// - `I1 = m1 + m2` and `I2 = m1 + m3`: the integer roots on the two largest coordinates, and
///   on the first and third, with the block's signs;
/// - `H1 = S/2 − odd·m8`: the half root with the block's signs, its smallest coordinate flipped
///   when the block has an odd number of minus signs;
/// - `H2 = S/2 − m7 − (1 − odd)·m8`: the next half root, which flips the second smallest
///   instead, or both when the count was even.
///
/// The nearest is `I1` when `I1 ≥ H1`, else `H1`; the runner-up is the larger of the two that
/// remain beside it. A half root's cell is `112 + Σ_{i<7} negᵢ·2ⁱ`: the eighth sign is fixed by
/// the even count. Two roots this close differ by a root, of length √2, so the distance to the
/// wall between them is the difference of their inner products over √2.
fn e8_address(blocks: &Tensor) -> Result<Address> {
    let device = blocks.device();
    let sorted = Sorted::of(blocks)?;
    let (first, second, third) = (sorted.at(0)?, sorted.at(1)?, sorted.at(2)?);
    let (seventh, eighth) = (sorted.at(BLOCK - 2)?, sorted.at(BLOCK - 1)?);
    let coordinate: Vec<f32> = (0..BLOCK).map(|i| i as f32).collect();
    let coordinate = Tensor::from_vec(coordinate, (1, BLOCK), device)?;
    let bits: Vec<f32> = (0..BLOCK)
        .map(|i| {
            if i + 1 < BLOCK {
                (1u32 << i) as f32
            } else {
                0.0
            }
        })
        .collect();
    let bits = Tensor::from_vec(bits, (1, BLOCK), device)?;
    let magnitude = &sorted.magnitude;
    let negative = &sorted.negative;
    let m1 = pick(magnitude, &first)?;
    let m2 = pick(magnitude, &second)?;
    let m3 = pick(magnitude, &third)?;
    let m7 = pick(magnitude, &seventh)?;
    let m8 = pick(magnitude, &eighth)?;
    let count = negative.sum(1)?;
    let odd = count.sub(&count.affine(0.5, 0.0)?.floor()?.affine(2.0, 0.0)?)?;
    let even = odd.affine(-1.0, 1.0)?;
    let half = magnitude.sum(1)?.affine(0.5, 0.0)?;
    let i1 = m1.add(&m2)?;
    let i2 = m1.add(&m3)?;
    let h1 = half.sub(&odd.mul(&m8)?)?;
    let h2 = half.sub(&m7)?.sub(&even.mul(&m8)?)?;
    let (c1, n1) = (pick(&coordinate, &first)?, pick(negative, &first)?);
    let (c2, n2) = (pick(&coordinate, &second)?, pick(negative, &second)?);
    let (c3, n3) = (pick(&coordinate, &third)?, pick(negative, &third)?);
    let cell_i1 = integer_cell(&c1, &n1, &c2, &n2)?;
    let cell_i2 = integer_cell(&c1, &n1, &c3, &n3)?;
    // Flipping a coordinate's sign moves the code by ±its bit: + when the coordinate was positive.
    let signed_bits = negative.affine(-2.0, 1.0)?.broadcast_mul(&bits)?;
    let flip7 = pick(&signed_bits, &seventh)?;
    let flip8 = pick(&signed_bits, &eighth)?;
    let code = pick(&bits, negative)?.affine(1.0, INTEGER_CELLS as f64)?;
    let cell_h1 = code.add(&odd.mul(&flip8)?)?;
    let cell_h2 = code.add(&flip7)?.add(&even.mul(&flip8)?)?;
    let integer = i1.ge(&h1)?.to_dtype(DType::F32)?;
    let next_integer = i2.ge(&h1)?.to_dtype(DType::F32)?;
    let next_half = h2.ge(&i1)?.to_dtype(DType::F32)?;
    let cell = choose(&integer, &cell_i1, &cell_h1)?;
    let value = choose(&integer, &i1, &h1)?;
    let across = choose(
        &integer,
        &choose(&next_integer, &cell_i2, &cell_h1)?,
        &choose(&next_half, &cell_h2, &cell_i1)?,
    )?;
    let runner = choose(
        &integer,
        &choose(&next_integer, &i2, &h1)?,
        &choose(&next_half, &h2, &i1)?,
    )?;
    let distance = value
        .sub(&runner)?
        .affine(std::f64::consts::FRAC_1_SQRT_2, 0.0)?;
    Ok(Address {
        cell,
        wall: Some(Wall { across, distance }),
    })
}

/// Each block's sign pattern, `Σ negᵢ·2ⁱ`. Its nearest wall is the coordinate plane of its
/// smallest coordinate: the cell across flips that sign, at that coordinate's magnitude.
fn sign_address(blocks: &Tensor) -> Result<Address> {
    let sorted = Sorted::of(blocks)?;
    let eighth = sorted.at(BLOCK - 1)?;
    let bits: Vec<f32> = (0..BLOCK).map(|i| (1u32 << i) as f32).collect();
    let bits = Tensor::from_vec(bits, (1, BLOCK), blocks.device())?;
    let cell = pick(&bits, &sorted.negative)?;
    let signed_bits = sorted.negative.affine(-2.0, 1.0)?.broadcast_mul(&bits)?;
    let across = cell.add(&pick(&signed_bits, &eighth)?)?;
    let distance = pick(&sorted.magnitude, &eighth)?;
    Ok(Address {
        cell,
        wall: Some(Wall { across, distance }),
    })
}

/// `(positions, blocks, width)`: block b of position p reads row `cell[p × blocks + b]` of block
/// b's own `cells` rows; in the flat table that is row `b × cells + c`.
///
/// The table is read as `(blocks, cells, width)` and gathered along its cells. candle's kernels
/// for the gradient of an indexed read run one serial loop over the indexed dimension for every
/// element of the others. Indexed flat by row, that was `width` loops over every block entry of
/// the batch, and a training step took four times as long as without the memory (measured 13 Nov
/// 2026). Gathered a block, it is `blocks × width` loops over the positions, summed in the same
/// order.
///
/// The index is cut from the graph: it never takes a gradient. Read flat with `index_select`, an
/// index that kept the decode's history gave the rows' gradient an op (candle's `index_add` takes
/// one from its index), and the optimizer's running means then held every step's forward pass for
/// the rest of the run: the first training run ran out of GPU memory in a few steps. The gathered
/// read's scatter takes no op from its index; the cut stays so that no path of that shape returns.
fn read_rows(
    table: &Tensor,
    cell: &Tensor,
    positions: usize,
    blocks: usize,
    cells: usize,
) -> Result<Tensor> {
    let width = table.dim(1)?;
    let index = cell
        .reshape((positions, blocks))?
        .transpose(0, 1)?
        .to_dtype(DType::U32)?
        .detach()
        .unsqueeze(2)?
        .broadcast_as((blocks, positions, width))?
        .contiguous()?;
    Ok(table
        .reshape((blocks, cells, width))?
        .gather(&index, 1)?
        .transpose(0, 1)?)
}

/// `(positions × blocks)`: each block's cell under [`AddressArm::Token`], a hash of the seed, the
/// layer, the block and the position's input token.
fn token_cells(
    seed: u64,
    layer: usize,
    ids: &[u32],
    blocks: usize,
    device: &Device,
) -> Result<Tensor> {
    let mut cells = Vec::with_capacity(ids.len() * blocks);
    for &id in ids {
        for block in 0..blocks {
            let key = seed ^ ((layer as u64) << 48) ^ ((block as u64) << 32) ^ u64::from(id);
            cells.push((mix(key) % E8_CELLS as u64) as f32);
        }
    }
    Ok(Tensor::from_vec(cells, ids.len() * blocks, device)?)
}

/// splitmix64's step: a fixed scramble of 64 bits.
fn mix(state: u64) -> u64 {
    let mut z = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The laptop holds tensors and runs no tensor arithmetic (D24/D29), so the tests come in two
    // layers. The decode's closed form is written a second time in plain Rust (`twin_e8`,
    // `twin_sign`) and checked here against a search over all 240 roots, with the census's
    // counting, the token hash and the file. The tensor code is held to that twin, and everything
    // else that computes is tested, on the GPU (`cuda`), before any training step.

    /// The GPU the tensor tests run on.
    #[cfg(feature = "cuda")]
    fn device() -> Device {
        Device::new_cuda(0).unwrap()
    }

    /// One block near a wall, all positive. Its nearest root is e₀ + e₁ (cell 0) at inner product
    /// 1.6; the runner-up is e₀ + e₂ (cell 4) at 1.5, so the wall is `(x₁ − x₂)/√2 = 0.0707` away.
    const NEAR_A_WALL: [f32; BLOCK] = [1.0, 0.6, 0.5, 0.05, 0.04, 0.03, 0.02, 0.01];

    type Twin = fn(&[f32]) -> (usize, usize, f32);

    /// The root a cell names, by the numbering [`e8_address`] writes.
    fn root(cell: usize) -> [f64; BLOCK] {
        let mut root = [0.0; BLOCK];
        if cell < INTEGER_CELLS {
            let (pair, n_lo, n_hi) = (cell / 4, (cell >> 1) & 1, cell & 1);
            let mut number = 0;
            for lo in 0..BLOCK {
                for hi in lo + 1..BLOCK {
                    if number == pair {
                        root[lo] = if n_lo == 1 { -1.0 } else { 1.0 };
                        root[hi] = if n_hi == 1 { -1.0 } else { 1.0 };
                    }
                    number += 1;
                }
            }
        } else {
            let code = cell - INTEGER_CELLS;
            let mut minus = 0;
            for (i, entry) in root.iter_mut().enumerate().take(BLOCK - 1) {
                let negative = (code >> i) & 1 == 1;
                minus += usize::from(negative);
                *entry = if negative { -0.5 } else { 0.5 };
            }
            root[BLOCK - 1] = if minus % 2 == 1 { -0.5 } else { 0.5 };
        }
        root
    }

    /// Every root's inner product with `block`, the largest first.
    fn scored(block: &[f32]) -> Vec<(f64, usize)> {
        let mut scored: Vec<(f64, usize)> = (0..E8_CELLS)
            .map(|cell| {
                let inner: f64 = root(cell)
                    .iter()
                    .zip(block)
                    .map(|(r, &b)| r * f64::from(b))
                    .sum();
                (inner, cell)
            })
            .collect();
        scored.sort_by(|a, b| b.0.total_cmp(&a.0));
        scored
    }

    /// The nearest root, the runner-up and the distance to the wall between them, by a search
    /// over all 240.
    fn searched(block: &[f32]) -> (usize, usize, f64) {
        let scored = scored(block);
        let distance = (scored[0].0 - scored[1].0) * std::f64::consts::FRAC_1_SQRT_2;
        (scored[0].1, scored[1].1, distance)
    }

    /// Whether the block's three best roots stand further apart than an f32 sum's rounding could
    /// close, so the nearest and the runner-up are the same whatever order the sums are taken in.
    #[cfg(feature = "cuda")]
    fn clear(block: &[f32]) -> bool {
        let scored = scored(block);
        scored[0].0 - scored[1].0 > 1e-4 && scored[1].0 - scored[2].0 > 1e-4
    }

    /// A block's nearest root, the runner-up and the wall's distance by [`e8_address`]'s closed
    /// form in plain Rust: the same places, sums and choices, one block at a time.
    fn twin_e8(block: &[f32]) -> (usize, usize, f32) {
        let magnitude: Vec<f32> = block.iter().map(|value| value.abs()).collect();
        let negative: Vec<usize> = block
            .iter()
            .map(|&value| usize::from(value < 0.0))
            .collect();
        // A coordinate's place: the larger ones, and the equal ones before it.
        let place = |i: usize| {
            (0..BLOCK)
                .filter(|&j| magnitude[j] > magnitude[i] || (magnitude[j] == magnitude[i] && j < i))
                .count()
        };
        let at = |wanted: usize| (0..BLOCK).find(|&i| place(i) == wanted).unwrap();
        let (first, second, third) = (at(0), at(1), at(2));
        let (seventh, eighth) = (at(BLOCK - 2), at(BLOCK - 1));
        let odd = negative.iter().sum::<usize>() % 2 == 1;
        let half = magnitude.iter().sum::<f32>() * 0.5;
        let i1 = magnitude[first] + magnitude[second];
        let i2 = magnitude[first] + magnitude[third];
        let h1 = half - if odd { magnitude[eighth] } else { 0.0 };
        let h2 = half - magnitude[seventh] - if odd { 0.0 } else { magnitude[eighth] };
        let integer_cell = |a: usize, b: usize| {
            let (lo, hi) = (a.min(b), a.max(b));
            4 * (lo * (15 - lo) / 2 + hi - lo - 1) + 2 * negative[lo] + negative[hi]
        };
        let half_cell = |flipped: &[usize]| {
            INTEGER_CELLS
                + (0..BLOCK - 1)
                    .map(|i| (negative[i] ^ usize::from(flipped.contains(&i))) << i)
                    .sum::<usize>()
        };
        let cell_h1 = if odd {
            half_cell(&[eighth])
        } else {
            half_cell(&[])
        };
        let cell_h2 = if odd {
            half_cell(&[seventh])
        } else {
            half_cell(&[seventh, eighth])
        };
        let (cell, value, across, runner) = if i1 >= h1 {
            if i2 >= h1 {
                (
                    integer_cell(first, second),
                    i1,
                    integer_cell(first, third),
                    i2,
                )
            } else {
                (integer_cell(first, second), i1, cell_h1, h1)
            }
        } else if h2 >= i1 {
            (cell_h1, h1, cell_h2, h2)
        } else {
            (cell_h1, h1, integer_cell(first, second), i1)
        };
        (
            cell,
            across,
            (value - runner) * std::f32::consts::FRAC_1_SQRT_2,
        )
    }

    /// A block's sign pattern, the pattern across its smallest coordinate's plane and that
    /// coordinate's magnitude: [`sign_address`] in plain Rust.
    fn twin_sign(block: &[f32]) -> (usize, usize, f32) {
        let cell: usize = (0..BLOCK).map(|i| usize::from(block[i] < 0.0) << i).sum();
        // The last of the smallest: equal magnitudes take their places in coordinate order.
        let mut smallest = 0;
        for i in 1..BLOCK {
            if block[i].abs() <= block[smallest].abs() {
                smallest = i;
            }
        }
        (cell, cell ^ (1 << smallest), block[smallest].abs())
    }

    /// `count` blocks of generic coordinates in (−1, 1), from `seed`.
    fn generic_blocks(count: usize, seed: u64) -> Vec<f32> {
        let mut state = seed;
        (0..count * BLOCK)
            .map(|_| {
                state = mix(state);
                ((state >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0) as f32
            })
            .collect()
    }

    /// [`generic_blocks`] with every second block's coordinates cubed: a few large coordinates
    /// and many small ones, where an integer root is the nearest.
    fn mixed_blocks(count: usize, seed: u64) -> Vec<f32> {
        let mut values = generic_blocks(count, seed);
        for block in values.chunks_mut(BLOCK).skip(1).step_by(2) {
            for value in block {
                *value = *value * *value * *value;
            }
        }
        values
    }

    /// The block with sign pattern `pattern` whose coordinate 3 is the smallest.
    fn sign_block(pattern: usize) -> [f32; BLOCK] {
        let mut block = [0.0; BLOCK];
        for (i, value) in block.iter_mut().enumerate() {
            let size = if i == 3 { 0.125 } else { 0.5 + i as f32 / 16.0 };
            *value = if (pattern >> i) & 1 == 1 { -size } else { size };
        }
        block
    }

    fn whole(tensor: &Tensor) -> Vec<usize> {
        tensor
            .to_vec1::<f32>()
            .unwrap()
            .into_iter()
            .map(|value| {
                assert_eq!(value.fract(), 0.0, "a cell is not a whole number: {value}");
                value as usize
            })
            .collect()
    }

    #[test]
    fn the_240_roots_are_distinct_and_each_decodes_to_its_own_cell() {
        let roots: Vec<[f64; BLOCK]> = (0..E8_CELLS).map(root).collect();
        for (cell, root) in roots.iter().enumerate() {
            let squared: f64 = root.iter().map(|r| r * r).sum();
            assert_eq!(squared, 2.0, "root {cell}");
            assert!(
                roots[..cell].iter().all(|other| other != root),
                "root {cell} repeats"
            );
            let block: Vec<f32> = root.iter().map(|&r| r as f32).collect();
            assert_eq!(searched(&block).0, cell, "root {cell} by the search");
            assert_eq!(twin_e8(&block).0, cell, "root {cell} by the closed form");
        }
    }

    #[test]
    fn the_closed_form_equals_a_search_over_all_240() {
        let count = 4_096;
        let values = mixed_blocks(count, 346);
        let (mut integer, mut half, mut across_integer, mut across_half) = (0, 0, 0, 0);
        for (n, block) in values.chunks(BLOCK).enumerate() {
            let (cell, across, distance) = twin_e8(block);
            let (nearest, runner, gap) = searched(block);
            assert_eq!(cell, nearest, "block {n}: {block:?}");
            assert_eq!(across, runner, "block {n}: {block:?}");
            assert!(
                (f64::from(distance) - gap).abs() < 1e-5,
                "block {n}: {block:?}"
            );
            if cell < INTEGER_CELLS {
                integer += 1;
            } else {
                half += 1;
            }
            if across < INTEGER_CELLS {
                across_integer += 1;
            } else {
                across_half += 1;
            }
        }
        // Both kinds of root are exercised, as the nearest and as the runner-up.
        assert!(
            integer > 500 && half > 500,
            "{integer} integer, {half} half"
        );
        assert!(
            across_integer > 500 && across_half > 500,
            "{across_integer} integer, {across_half} half across the wall"
        );
    }

    #[test]
    fn a_zero_or_tied_block_still_names_a_cell() {
        let (zero, plus, minus) = ([0.0f32; BLOCK], [0.25f32; BLOCK], [-0.25f32; BLOCK]);
        for block in [zero, plus, minus] {
            for (twin, cells) in [(twin_e8 as Twin, E8_CELLS), (twin_sign as Twin, SIGN_CELLS)] {
                let (cell, across, distance) = twin(&block);
                assert!(
                    cell < cells && across < cells && cell != across,
                    "{block:?}"
                );
                assert!(
                    distance.is_finite() && distance >= 0.0,
                    "{block:?}: {distance}"
                );
            }
        }
        // All eight at +¼ is the all-plus half root, cell 112; at −¼, an even count of minus
        // signs, the all-minus half root, whose seven free signs are all set.
        assert_eq!(twin_e8(&plus).0, INTEGER_CELLS);
        assert_eq!(twin_e8(&minus).0, INTEGER_CELLS + 127);
    }

    #[test]
    fn the_sign_twin_reads_the_sign_pattern_and_its_wall_is_the_smallest_coordinate() {
        for pattern in 0..SIGN_CELLS {
            assert_eq!(
                twin_sign(&sign_block(pattern)),
                (pattern, pattern ^ 8, 0.125)
            );
        }
    }

    /// The wall's distance moves as the hand-worked gradient of
    /// `the_rows_take_the_gradient_and_the_state_takes_it_through_the_wall` says: `+1/√2` per unit
    /// of x₁, `−1/√2` per unit of x₂ and nothing for the rest.
    #[test]
    fn the_wall_distance_has_the_hand_worked_slope() {
        use std::f64::consts::FRAC_1_SQRT_2;
        let (cell, across, distance) = twin_e8(&NEAR_A_WALL);
        assert_eq!((cell, across), (0, 4));
        assert!(
            (f64::from(distance) - 0.1 * FRAC_1_SQRT_2).abs() < 1e-6,
            "{distance}"
        );
        let step = 1.0 / 1024.0;
        for i in 0..BLOCK {
            let mut moved = NEAR_A_WALL;
            moved[i] += step;
            let (moved_cell, moved_across, moved_distance) = twin_e8(&moved);
            assert_eq!((moved_cell, moved_across), (0, 4), "coordinate {i}");
            let slope = f64::from(moved_distance - distance) / f64::from(step);
            let want = match i {
                1 => FRAC_1_SQRT_2,
                2 => -FRAC_1_SQRT_2,
                _ => 0.0,
            };
            assert!((slope - want).abs() < 1e-3, "coordinate {i}: {slope}");
        }
    }

    #[test]
    fn the_token_arm_hashes_the_seed_the_layer_the_block_and_the_token() {
        let ids = [5u32, 9, 5, 4_095];
        let cells = |seed: u64, layer: usize| {
            whole(&token_cells(seed, layer, &ids, 32, &Device::Cpu).unwrap())
        };
        let base = cells(11, 0);
        assert_eq!(base.len(), ids.len() * 32);
        assert!(base.iter().all(|&cell| cell < E8_CELLS));
        assert_eq!(base, cells(11, 0));
        assert_eq!(
            base[..32],
            base[64..96],
            "the same token reads the same cells"
        );
        assert_ne!(base[..32], base[32..64], "another token");
        assert!(
            base[..32].iter().any(|&cell| cell != base[0]),
            "the blocks share one cell"
        );
        assert_ne!(base, cells(11, 1), "another layer");
        assert_ne!(base, cells(12, 0), "another seed");
    }

    /// The tensor decode is the twin, block for block: every root, the tied and zero blocks and
    /// every sign pattern (their sums are exact), and the generic blocks that are clear of a tie.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_tensor_decode_is_the_twin() {
        let device = device();
        let mut values: Vec<f32> = (0..E8_CELLS).flat_map(root).map(|r| r as f32).collect();
        values.extend([0.0f32; BLOCK]);
        values.extend([0.25f32; BLOCK]);
        values.extend([-0.25f32; BLOCK]);
        for pattern in 0..SIGN_CELLS {
            values.extend(sign_block(pattern));
        }
        let exact = values.len() / BLOCK;
        values.extend(mixed_blocks(4_096, 346));
        let count = values.len() / BLOCK;
        let blocks = Tensor::from_vec(values.clone(), (count, BLOCK), &device).unwrap();
        let arms: [(&str, Address, Twin); 2] = [
            ("e8", e8_address(&blocks).unwrap(), twin_e8),
            ("sign", sign_address(&blocks).unwrap(), twin_sign),
        ];
        for (name, address, twin) in arms {
            let wall = address.wall.unwrap();
            let (cells, across) = (whole(&address.cell), whole(&wall.across));
            let distance = wall.distance.to_vec1::<f32>().unwrap();
            let mut compared = 0;
            for (n, block) in values.chunks(BLOCK).enumerate() {
                if n >= exact && name == "e8" && !clear(block) {
                    continue;
                }
                compared += 1;
                let (cell, runner, gap) = twin(block);
                assert_eq!(cells[n], cell, "{name} block {n}: {block:?}");
                assert_eq!(across[n], runner, "{name} block {n}: {block:?}");
                assert!(
                    (distance[n] - gap).abs() < 1e-5,
                    "{name} block {n}: {block:?}"
                );
            }
            assert!(
                compared > exact + 4_000,
                "{name}: {compared} blocks compared"
            );
        }
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn zero_rows_leave_the_residual_and_trained_rows_add_one_row_a_block() {
        let device = device();
        let (width, positions) = (2 * BLOCK, 6);
        let values = generic_blocks(positions * 2, 9);
        let residual = Tensor::from_vec(values.clone(), (2, 3, width), &device).unwrap();
        let ids = [5u32, 9, 5, 0, 4_095, 7];
        for arm in AddressArm::ALL {
            let memory = AddressMemory::new(arm, 2, width, None, 11, &device).unwrap();
            let same = memory.apply(1, &residual, &ids).unwrap();
            assert_eq!(
                same.flatten_all().unwrap().to_vec1::<f32>().unwrap(),
                values,
                "{arm:?}: zero rows changed the residual"
            );
            let memory = memory.randomized(31).unwrap();
            let rows = memory.rows[1].as_tensor().to_vec2::<f32>().unwrap();
            let out = memory.apply(1, &residual, &ids).unwrap();
            let out = out.flatten_all().unwrap().to_vec1::<f32>().unwrap();
            let token = token_cells(11, 1, &ids, 2, &device).unwrap();
            let token = whole(&token);
            for position in 0..positions {
                let point = &values[position * width..(position + 1) * width];
                let cell = |block: usize| match arm {
                    AddressArm::E8 => searched(&point[block * BLOCK..(block + 1) * BLOCK]).0,
                    AddressArm::Sign => (0..BLOCK)
                        .map(|i| usize::from(point[block * BLOCK + i] < 0.0) << i)
                        .sum(),
                    AddressArm::Collapse => 0,
                    AddressArm::Token => token[position * 2 + block],
                };
                let (first, second) = (&rows[cell(0)], &rows[arm.cells() + cell(1)]);
                for i in 0..width {
                    // Row values are dyadic, so their sum is exact; one rounding adds it.
                    let want = point[i] + (first[i] + second[i]);
                    assert_eq!(
                        out[position * width + i],
                        want,
                        "{arm:?} position {position}"
                    );
                }
            }
        }
        // The token arm's read is the same wherever the token is, whatever the state.
        let memory = AddressMemory::new(AddressArm::Token, 1, width, None, 11, &device)
            .unwrap()
            .randomized(5)
            .unwrap();
        let zero = Tensor::zeros((2, 3, width), DType::F32, &device).unwrap();
        let read = memory.apply(0, &zero, &ids).unwrap();
        let read = read
            .reshape((positions, width))
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        assert_eq!(read[0], read[2]);
        assert_ne!(read[0], read[1]);
    }

    /// One position of one block: [`NEAR_A_WALL`].
    #[cfg(feature = "cuda")]
    fn near_a_wall(device: &Device) -> Var {
        Var::new(&[[NEAR_A_WALL]], device).unwrap()
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_rows_take_the_gradient_and_the_state_takes_it_through_the_wall() {
        let device = device();
        let band = 0.5;
        let memory = AddressMemory::new(AddressArm::E8, 1, BLOCK, Some(band), 0, &device).unwrap();
        // The row across the wall, cell 4, holds ¼ everywhere; the row read, cell 0, is zero.
        let mut table = vec![0.0f32; E8_CELLS * BLOCK];
        table[4 * BLOCK..5 * BLOCK].fill(0.25);
        memory.rows[0]
            .set(&Tensor::from_vec(table, (E8_CELLS, BLOCK), &device).unwrap())
            .unwrap();
        let state = near_a_wall(&device);
        let out = memory.apply(0, state.as_tensor(), &[0]).unwrap();
        // Forward, the read is cell 0's zero row and the wall term is exactly zero.
        assert_eq!(
            out.flatten_all().unwrap().to_vec1::<f32>().unwrap(),
            state
                .as_tensor()
                .flatten_all()
                .unwrap()
                .to_vec1::<f32>()
                .unwrap()
        );
        let grads = out.sum_all().unwrap().backward().unwrap();
        let rows = grads.get(memory.rows[0].as_tensor()).unwrap();
        let rows = rows.to_vec2::<f32>().unwrap();
        assert_eq!(
            rows[0],
            vec![1.0; BLOCK],
            "the row read takes the output's gradient"
        );
        assert_eq!(
            rows[4],
            vec![0.0; BLOCK],
            "the row across the wall takes none"
        );
        // With L = Σ out and Δ = Σ (V[4] − V[0]) = 2: dL/dx = 1 + Δ·dλ/dx, and
        // dλ/dx₁ = −1/(2·τρ·√2), dλ/dx₂ = +1/(2·τρ·√2), zero elsewhere.
        let rho = (NEAR_A_WALL
            .iter()
            .map(|&v| f64::from(v) * f64::from(v))
            .sum::<f64>()
            / 8.0)
            .sqrt();
        let slope = 2.0 / (2.0 * band * rho * std::f64::consts::SQRT_2);
        let grad = grads.get(state.as_tensor()).unwrap();
        let grad = grad.flatten_all().unwrap().to_vec1::<f32>().unwrap();
        assert!(
            (f64::from(grad[1]) - (1.0 - slope)).abs() < 1e-4,
            "{grad:?}"
        );
        assert!(
            (f64::from(grad[2]) - (1.0 + slope)).abs() < 1e-4,
            "{grad:?}"
        );
        for i in [0, 3, 4, 5, 6, 7] {
            assert_eq!(grad[i], 1.0, "coordinate {i}: {grad:?}");
        }
        // Descent on x raises x₂ and lowers x₁: toward the wall, whose far row lowers this loss
        // less (it adds ¼ to every output), so with the loss's sign turned the block moves away.
        assert!(slope > 1.0);
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn a_band_of_zero_or_a_block_outside_the_band_passes_the_gradient_through() {
        let device = device();
        for band in [0.0, 0.05] {
            let memory =
                AddressMemory::new(AddressArm::E8, 1, BLOCK, Some(band), 0, &device).unwrap();
            let memory = memory.randomized(3).unwrap();
            let state = near_a_wall(&device);
            let out = memory.apply(0, state.as_tensor(), &[0]).unwrap();
            let grads = out.sum_all().unwrap().backward().unwrap();
            let grad = grads.get(state.as_tensor()).unwrap();
            // At τ = 0.05 the band reaches 0.05 × 0.449 = 0.022, short of the wall at 0.0707.
            assert_eq!(
                grad.flatten_all().unwrap().to_vec1::<f32>().unwrap(),
                vec![1.0; BLOCK],
                "band {band}"
            );
        }
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_rows_gradient_carries_no_history_of_the_forward_pass() {
        let device = device();
        // The optimizer keeps running means of this gradient from step to step. A gradient with
        // an op holds its whole forward pass alive in them, one more every step: on 13 Nov 2026
        // that filled a 32 GB card within a few steps of the first training run.
        for arm in [
            AddressArm::E8,
            AddressArm::Sign,
            AddressArm::Token,
            AddressArm::Collapse,
        ] {
            for band in [0.0, 0.5] {
                let memory = AddressMemory::new(arm, 1, BLOCK, Some(band), 0, &device).unwrap();
                let memory = memory.randomized(3).unwrap();
                let state = near_a_wall(&device);
                let out = memory.apply(0, state.as_tensor(), &[0]).unwrap();
                let grads = out.sum_all().unwrap().backward().unwrap();
                let rows = grads.get(memory.rows[0].as_tensor()).unwrap();
                assert!(!rows.track_op(), "{arm:?}, band {band}");
            }
        }
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn rounding_puts_the_rows_on_the_16_bit_grid_and_the_hold_keeps_them_in_range() {
        let device = device();
        let memory = AddressMemory::new(AddressArm::Collapse, 1, BLOCK, None, 0, &device).unwrap();
        let values = [
            0.3f32,
            -0.3,
            0.7,
            -0.7,
            1.0 / 131_072.0,
            -1.0 / 131_072.0,
            0.0,
            0.123_456_7,
        ];
        let set = |values: &[f32]| {
            memory.rows[0]
                .set(&Tensor::from_vec(values.to_vec(), (1, BLOCK), &device).unwrap())
                .unwrap();
        };
        let read = || {
            memory.rows[0]
                .as_tensor()
                .flatten_all()
                .unwrap()
                .to_vec1::<f32>()
                .unwrap()
        };
        set(&values);
        memory.round_as_exported().unwrap();
        let twin: Vec<f32> = values
            .iter()
            .map(|&value| {
                let steps = (f64::from(value) * ROW_SCALE + 0.5).floor();
                (steps.clamp(-ROW_STEPS, ROW_STEPS) / ROW_SCALE) as f32
            })
            .collect();
        assert_eq!(read(), twin);
        for value in read() {
            let steps = f64::from(value) * ROW_SCALE;
            assert_eq!(steps.fract(), 0.0, "{value} is off the grid");
            assert!(steps.abs() <= ROW_STEPS, "{value} is out of 16 bits");
        }
        // A half step rounds up, toward plus infinity, as the residual stream's rounding does.
        assert_eq!(read()[4], 1.0 / 65_536.0);
        assert_eq!(read()[5], 0.0);
        set(&values);
        memory.hold_in_exported_range().unwrap();
        let limit = (ROW_STEPS / ROW_SCALE) as f32;
        assert_eq!(read()[2..4], [limit, -limit]);
        assert_eq!(read()[..2], values[..2]);
    }

    #[test]
    fn a_saved_memory_loads_as_it_was_and_a_checkpoint_without_one_loads_none() {
        // Storage only: the laptop holds and files the rows.
        let device = Device::Cpu;
        let directory = std::env::temp_dir().join(format!(
            "uor-r4-address-memory-{}-{:x}",
            std::process::id(),
            mix(346)
        ));
        std::fs::create_dir_all(&directory).unwrap();
        assert!(AddressMemory::load(&directory, &device).unwrap().is_none());
        let seed = 0xDEAD_BEEF_0000_0346;
        for arm in AddressArm::ALL {
            let memory = AddressMemory::new(arm, 2, 2 * BLOCK, Some(0.375), seed, &device)
                .unwrap()
                .randomized(7)
                .unwrap();
            memory.save(&directory).unwrap();
            let loaded = AddressMemory::load(&directory, &device).unwrap().unwrap();
            assert_eq!(
                (loaded.arm, loaded.width, loaded.band),
                (arm, 2 * BLOCK, 0.375)
            );
            assert_eq!(loaded.seed, seed);
            assert_eq!(loaded.rows.len(), 2);
            for (a, b) in memory.rows.iter().zip(&loaded.rows) {
                assert_eq!(
                    a.as_tensor().to_vec2::<f32>().unwrap(),
                    b.as_tensor().to_vec2::<f32>().unwrap()
                );
            }
        }
        std::fs::remove_file(directory.join(FILE)).unwrap();
        std::fs::remove_dir(&directory).unwrap();
    }

    #[test]
    fn a_layers_census_counts_cells_movement_and_the_band() {
        // Two blocks of four cells over four positions, a position's two reads together: block 0
        // reads its cell 0 every time (row 0), block 1 its cells 1 and 2 by turns (rows 5, 6).
        let index = [0u32, 5, 0, 6, 0, 5, 0, 6];
        let first = layer_census(3, &index, 2, 2, 4, None, 0.25);
        assert_eq!(first.layer, 3);
        assert_eq!(
            first.cells_used_mean, 1.5,
            "block 0 reads one cell, block 1 two"
        );
        assert_eq!(first.cells_used_min, 1);
        assert_eq!(
            first.largest_cell_share, 1.0,
            "all of block 0's reads are one cell"
        );
        assert_eq!(first.moved_share, None);
        assert_eq!(first.in_band_share, 0.25);
        assert_eq!(first.row_mean_abs, 0.25);
        let moved = [0u32, 6, 0, 6, 0, 5, 0, 6];
        let second = layer_census(3, &moved, 0, 2, 4, Some(&index), 0.0);
        assert_eq!(second.moved_share, Some(0.125), "one of eight reads moved");
        assert_eq!(second.in_band_share, 0.0);
        // A census over other reads than the last has nothing to compare.
        assert_eq!(
            layer_census(3, &moved[..4], 0, 2, 4, Some(&index), 0.0).moved_share,
            None
        );
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_census_reads_the_forward_it_is_given() {
        let device = device();
        let width = 2 * BLOCK;
        let memory = AddressMemory::new(AddressArm::Sign, 1, width, Some(0.5), 0, &device).unwrap();
        let ids = [0u32; 4];
        let state = |flip: bool| {
            // Four positions. Block 0 is all positive at every position; block 1's first sign
            // differs by position, and under `flip` position 0's is turned.
            let mut values = vec![1.0f32; 4 * width];
            for position in 0..4 {
                let negative = (position % 2 == 1) != (flip && position == 0);
                values[position * width + BLOCK] = if negative { -1.0 } else { 1.0 };
            }
            Tensor::from_vec(values, (1, 4, width), &device).unwrap()
        };
        let mut census = Census::default();
        let mut read = |flip: bool| {
            census
                .read(&memory, || memory.apply(0, &state(flip), &ids).map(|_| ()))
                .unwrap()
                .remove(0)
        };
        let first = read(false);
        assert_eq!(
            first.cells_used_mean, 1.5,
            "block 0 reads one cell, block 1 two"
        );
        assert_eq!(first.cells_used_min, 1);
        assert_eq!(first.largest_cell_share, 1.0);
        assert_eq!(first.moved_share, None);
        // Every coordinate is ±1 and ρ = 1: every block is a whole unit from its wall.
        assert_eq!(first.in_band_share, 0.0);
        assert_eq!(first.row_mean_abs, 0.0);
        let second = read(true);
        assert_eq!(
            second.moved_share,
            Some(1.0 / 8.0),
            "one of eight reads moved"
        );
        // A forward outside a census is not captured.
        memory.apply(0, &state(false), &ids).unwrap();
        assert!(CAPTURE.with(|cell| cell.borrow().is_none()));
    }

    /// `count` independent standard Gaussians, from `seed` (Box and Muller's transform).
    fn gaussians(count: usize, seed: u64) -> Vec<f32> {
        let mut state = seed;
        let mut uniform = || {
            state = mix(state);
            ((state >> 11) as f64 + 0.5) / (1u64 << 53) as f64
        };
        (0..count)
            .map(|_| {
                let (u, v) = (uniform(), uniform());
                ((-2.0 * u.ln()).sqrt() * (std::f64::consts::TAU * v).cos()) as f32
            })
            .collect()
    }

    #[test]
    fn the_default_bands_are_the_gaussian_medians() {
        let count = 32_768;
        let values = gaussians(count * BLOCK, 346);
        let median = |twin: Twin| {
            let mut distance: Vec<f32> = values.chunks(BLOCK).map(|block| twin(block).2).collect();
            distance.sort_by(f32::total_cmp);
            f64::from(distance[count / 2])
        };
        let (e8, sign) = (median(twin_e8), median(twin_sign));
        assert!((e8 - E8_BAND).abs() < 0.005, "the E8 wall's median is {e8}");
        assert!(
            (sign - SIGN_BAND).abs() < 0.005,
            "the sign wall's median is {sign}"
        );
        assert_eq!(AddressArm::Collapse.default_band(), 0.0);
        assert_eq!(AddressArm::Token.default_band(), 0.0);
    }

    #[test]
    fn an_arm_parses_by_its_name_and_a_bad_shape_is_refused() {
        // Every refusal here comes before the first tensor op.
        let device = Device::Cpu;
        let refused = |result: Result<Tensor>, words: &str| {
            let message = result
                .err()
                .map(|error| error.to_string())
                .unwrap_or_default();
            assert!(
                message.contains(words),
                "wanted \"{words}\", got \"{message}\""
            );
        };
        for arm in AddressArm::ALL {
            assert_eq!(AddressArm::parse(arm.name()).unwrap(), arm);
            assert_eq!(AddressArm::from_code(arm.code()).unwrap(), arm);
            assert_eq!(
                serde_json::to_string(&arm).unwrap(),
                format!("\"{}\"", arm.name())
            );
        }
        assert!(AddressArm::parse("random").is_err());
        assert!(AddressMemory::new(AddressArm::E8, 1, 12, None, 0, &device).is_err());
        assert!(AddressMemory::new(AddressArm::E8, 0, 16, None, 0, &device).is_err());
        assert!(AddressMemory::new(AddressArm::E8, 1, 16, Some(-0.1), 0, &device).is_err());
        let memory = AddressMemory::new(AddressArm::E8, 1, 16, None, 0, &device).unwrap();
        assert_eq!(memory.rate(), 0.5);
        let residual = Tensor::zeros((1, 2, 16), DType::F32, &device).unwrap();
        refused(memory.apply(1, &residual, &[0, 0]), "no table for layer 1");
        refused(memory.apply(0, &residual, &[0]), "every position's token");
        let narrow = Tensor::zeros((1, 2, 8), DType::F32, &device).unwrap();
        refused(memory.apply(0, &narrow, &[0, 0]), "the residual is 8 wide");
    }

    /// The trunk with a zero table is the trunk; a trained table changes it, under every arm.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_trunk_with_zero_rows_is_unchanged_and_trained_rows_change_it() {
        struct Clear;
        impl Drop for Clear {
            fn drop(&mut self) {
                set_address_memory(None);
            }
        }
        let _clear = Clear;
        let device = Device::new_cuda(0).unwrap();
        let model = crate::spiralcore_fbs_sheet::build_fbs_model(
            crate::spiralcore_fbs_arms::FbsArm::OneAtomActive,
            1,
            &device,
        )
        .unwrap();
        let ids: Vec<u32> = (0..16u32).map(|i| (i * 37 + 5) % 4096).collect();
        let hidden = |m: &crate::geometric_stack::StackModel| {
            crate::hamilton_sheet::with_gate_mode(
                crate::hamilton_sheet::HamiltonArm::HamiltonActive,
                1,
                0,
                false,
                || m.hidden(&ids, 1, 16),
            )
            .unwrap()
            .flatten_all()
            .unwrap()
            .to_vec1::<f32>()
            .unwrap()
        };
        let plain = hidden(&model);
        let width = crate::hamilton_sheet::SHEET_WIDTH;
        for arm in AddressArm::ALL {
            let memory = AddressMemory::new(arm, 6, width, None, 346, &device).unwrap();
            set_address_memory(Some(memory.clone()));
            assert_eq!(
                hidden(&model),
                plain,
                "{arm:?}: zero rows changed the trunk"
            );
            set_address_memory(Some(memory.randomized(346).unwrap()));
            assert_ne!(
                hidden(&model),
                plain,
                "{arm:?}: trained rows left the trunk as it was"
            );
        }
    }

    /// claude-366: on the served grid the tensor decode names the cell the served reader names
    /// (`uor_r4_integer::fbs_address::e8_cell`), block for block: the shared blocks, which hold
    /// the zero block and the planted ties, and drawn blocks at four scales. At the smallest
    /// scale most blocks hold equal magnitudes or a tie between the two families of roots, so
    /// the tie rules are compared, not only the clear blocks. Every value and every sum of a
    /// block's magnitudes is below 2²⁴ steps, where the trainer's 32-bit sums are exact.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_tensor_decode_names_the_served_cell() {
        use uor_r4_integer::fbs_address::{e8_cell, ADDRESS_SHARED_BLOCKS};
        let device = device();
        let mut grid: Vec<[i32; BLOCK]> = ADDRESS_SHARED_BLOCKS.to_vec();
        let mut state = 366u64;
        for n in 0..16_384usize {
            let reach = [3i64, 40, 70_000, 2_000_000][n % 4];
            let mut block = [0i32; BLOCK];
            for value in &mut block {
                state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
                *value = ((mix(state) >> 11) as i64 % (2 * reach + 1) - reach) as i32;
            }
            grid.push(block);
        }
        let values: Vec<f32> = grid
            .iter()
            .flatten()
            .map(|step| *step as f32 / ROW_SCALE as f32)
            .collect();
        let blocks = Tensor::from_vec(values, (grid.len(), BLOCK), &device).unwrap();
        let cells = whole(&e8_address(&blocks).unwrap().cell);
        let mut seen = std::collections::BTreeSet::new();
        for (n, block) in grid.iter().enumerate() {
            assert_eq!(cells[n], e8_cell(block) as usize, "block {n}: {block:?}");
            seen.insert(cells[n]);
        }
        assert!(seen.len() > 200, "the blocks named {} cells", seen.len());
    }

    /// claude-366: the token arm's cells are the served reader's hash, token, layer and block.
    #[test]
    fn the_token_cells_are_the_served_hash() {
        use uor_r4_integer::fbs_address::token_cell;
        let ids = [0u32, 5, 9, 4_095, 65_535];
        for (seed, layer) in [(0u64, 0usize), (11, 3), (0x0123_4567_89AB_CDEF, 5)] {
            let cells = whole(&token_cells(seed, layer, &ids, 32, &Device::Cpu).unwrap());
            assert_eq!(cells.len(), ids.len() * 32);
            for (place, id) in ids.iter().enumerate() {
                for block in 0..32 {
                    assert_eq!(
                        cells[place * 32 + block],
                        token_cell(seed, layer, block, *id) as usize,
                        "seed {seed}, layer {layer}, token {id}, block {block}"
                    );
                }
            }
        }
    }

    /// claude-366: the served table holds the rows `round_as_exported` leaves, step for step,
    /// with the memory's arm, seed and width; taking it leaves the memory as it was. The drawn
    /// rows are multiples of 2⁻²⁰ up to ±1, so they hold exact half steps, values between
    /// steps and values past the 16-bit bound, and the plain rule below is exact for them.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_served_image_holds_the_rounded_rows() {
        let device = device();
        for (arm, served) in [
            (AddressArm::E8, uor_r4_integer::fbs_address::AddressArm::E8),
            (
                AddressArm::Token,
                uor_r4_integer::fbs_address::AddressArm::Token,
            ),
        ] {
            let seed = 0x0366_0000_0000_0007u64;
            let memory = AddressMemory::new(arm, 2, 16, None, seed, &device).unwrap();
            let shape = (16 / BLOCK * E8_CELLS, 16);
            let mut state = 366u64;
            let mut drawn = Vec::new();
            for rows in &memory.rows {
                let values: Vec<f32> = (0..shape.0 * shape.1)
                    .map(|_| {
                        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
                        let k = (mix(state) >> 11) as i64 % (2 * (1 << 20) + 1) - (1 << 20);
                        k as f32 / (1u32 << 20) as f32
                    })
                    .collect();
                rows.set(&Tensor::from_vec(values.clone(), shape, &device).unwrap())
                    .unwrap();
                drawn.push(values);
            }
            let image = memory.image().unwrap();
            assert_eq!(
                (image.arm(), image.seed(), image.width(), image.layers()),
                (served, seed, 16, 2)
            );
            let (mut halves, mut held) = (0, 0);
            for (layer, values) in drawn.iter().enumerate() {
                let unchanged = memory.rows[layer]
                    .as_tensor()
                    .flatten_all()
                    .unwrap()
                    .to_vec1::<f32>()
                    .unwrap();
                assert_eq!(
                    &unchanged, values,
                    "{arm:?}: taking the image moved the memory"
                );
                for (n, value) in values.iter().enumerate() {
                    let scaled = f64::from(*value) * ROW_SCALE;
                    let want = (scaled + 0.5).floor().clamp(-ROW_STEPS, ROW_STEPS) as i16;
                    assert_eq!(
                        image.layer(layer)[n],
                        want,
                        "{arm:?} layer {layer} value {n}: {value}"
                    );
                    halves += usize::from((scaled - scaled.floor() - 0.5).abs() < 1e-12);
                    held += usize::from(scaled.abs() > ROW_STEPS + 1.0);
                }
            }
            assert!(
                halves > 100 && held > 1_000,
                "{halves} half steps, {held} held values"
            );
            memory.round_as_exported().unwrap();
            for layer in 0..2 {
                let rounded = memory.rows[layer]
                    .as_tensor()
                    .flatten_all()
                    .unwrap()
                    .to_vec1::<f32>()
                    .unwrap();
                for (n, value) in rounded.iter().enumerate() {
                    assert_eq!(
                        f64::from(*value) * ROW_SCALE,
                        f64::from(image.layer(layer)[n]),
                        "{arm:?} layer {layer} value {n}"
                    );
                }
            }
        }
        // The other arms have no served form.
        for arm in [AddressArm::Sign, AddressArm::Collapse] {
            let memory = AddressMemory::new(arm, 1, 16, None, 1, &device).unwrap();
            let message = memory.image().expect_err("no served form").to_string();
            assert!(message.contains("no served form"), "{message}");
        }
    }
}
