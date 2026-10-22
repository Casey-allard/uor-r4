//! Exact arithmetic on the geometric path (`m6-exact-arithmetic-mechanism`, `#2864` / `#2034`).
//!
//! Represents multi-digit integers in radix `B` (`2 <= B <= 16`, including decimal `B = 10`
//! and full-orbit hexadecimal `B = 16`) as place-indexed 8D integer atoms on the 16-root `E_8`
//! anchor orbit (`ROOTS[0..16]`), binds operands and intermediate partial sums into
//! [`IntegerAddressedMemory`] under `(role_prime, place_prime)` [`PrimeFactorKey`] addresses,
//! orders place leaves across the ternary FBS binder (`fbs_binder_summary`), and executes
//! single-place digit transitions via the 64-state `Cl(0,6)` signed coordinate-permutation
//! table (`BFS`, `COMPOSE`, `AXES`) with exact inverse `g^{-1} = g^3` and a three-valued
//! carry/borrow trit switch in `{-1, 0, +1}`.
//!
//! Every step is multiplier-free and integer-only under D11, with zero squashing tables and
//! zero dense matrix products. Supports [`AblationArm`] causal knockouts (`#2863`) to verify
//! that multi-step arithmetic depends causally on both `Cl(0,6)` state transport and
//! exact-addressed memory.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::discrete_d11::{
    cl06_action, cl06_bfs, cl06_compose, cl06_inverse, fbs_binder_summary, orbit_root_coords,
    orbit_root_id, orbit_snap,
};
use crate::m6_ablation::{
    AblationArm, IntegerAddressedMemory, MemoryLookupOutcome, MemoryVersionSelector, PrimeFactorKey,
};
use crate::{invalid, Result};

/// Schema identifier for M6 exact geometric arithmetic traces.
pub const M6_EXACT_ARITHMETIC_SCHEMA: &str = "uor-r4.m6-exact-arithmetic/1";

/// Maximum number of place-value digits supported in one operand or result (fits inside the
/// 243-leaf ternary FBS binder cap and `u64` integer range).
pub const MAX_ARITHMETIC_PLACES: usize = 32;

/// Number of distinct [`OperandRole`] slots (`OperandA`, `OperandB`, `OperandC`, `Accumulator`,
/// and `Register(0..4)`).
const ROLE_COUNT: usize = 8;

/// Return the `n`-th odd prime (`n = 0 -> 3, n = 1 -> 5, ...`) by deterministic trial-free
/// odd-divisor addition scan (used only to form small static coprime `(role, place)` keys).
fn nth_odd_prime(target_idx: usize) -> u32 {
    let mut count = 0usize;
    let mut candidate = 3u32;
    loop {
        let mut is_prime = true;
        let mut d = 3u32;
        while repeated_add_u64(u64::from(d), d as u8).unwrap_or(u64::MAX) <= u64::from(candidate) {
            // Check divisibility by repeated addition/subtraction (shift-subtract remainder).
            let mut rem = candidate;
            let mut shift = 0u32;
            while (d << shift) <= rem && shift < 20 {
                shift += 1;
            }
            loop {
                let scaled = d << shift;
                if rem >= scaled {
                    rem -= scaled;
                }
                if shift == 0 {
                    break;
                }
                shift -= 1;
            }
            if rem == 0 {
                is_prime = false;
                break;
            }
            d += 2;
        }
        if is_prime {
            if count == target_idx {
                return candidate;
            }
            count += 1;
        }
        candidate += 2;
    }
}

/// Operand or accumulator role used to form exact pairwise-coprime `(role, place)` addresses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperandRole {
    OperandA,
    OperandB,
    OperandC,
    Accumulator,
    Register(u8),
}

impl OperandRole {
    /// Return the canonical role index `0..8` for this operand role.
    pub fn role_index(self) -> Result<usize> {
        match self {
            Self::OperandA => Ok(0),
            Self::OperandB => Ok(1),
            Self::OperandC => Ok(2),
            Self::Accumulator => Ok(3),
            Self::Register(r) if (r as usize) + 4 < ROLE_COUNT => Ok(4 + usize::from(r)),
            Self::Register(_) => Err(invalid("OperandRole::Register index must be in 0..4")),
        }
    }

    /// Build the exact pairwise-coprime two-prime [`PrimeFactorKey`] for `(self, place)`.
    /// Every distinct `(role, place)` pair receives a disjoint prime pair `(p_a, p_b)` so
    /// `gcd(q_{r1,p1}, A_{r2,p2}) > 1` if and only if `(r1, p1) == (r2, p2)`.
    pub fn place_key(self, place: u32) -> Result<PrimeFactorKey> {
        let p_idx = usize::try_from(place).map_err(|_| invalid("place index overflow"))?;
        if p_idx >= MAX_ARITHMETIC_PLACES {
            return Err(invalid(
                "place index exceeds MAX_ARITHMETIC_PLACES (32 places)",
            ));
        }
        let r_idx = self.role_index()?;
        let slot_idx = (r_idx << 5) + p_idx;
        let p_a = nth_odd_prime(slot_idx << 1);
        let p_b = nth_odd_prime((slot_idx << 1) + 1);
        PrimeFactorKey::new(vec![p_a, p_b], vec![(p_a, p_b)])
    }
}

/// One digit at place exponent `place` (LSB-first, `0` = ones place) represented as an
/// 8D integer atom on the 16-root `E_8` anchor orbit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrbitDigitAtom {
    pub place: u32,
    pub radix: u8,
    pub digit: u8,
    pub coords: [i32; 8],
}

impl OrbitDigitAtom {
    /// Construct an 8D orbit atom for `digit` in `0..radix` at `place`.
    pub fn new(digit: u8, place: u32, radix: u8) -> Result<Self> {
        validate_radix(radix)?;
        if digit >= radix {
            return Err(invalid(format!(
                "digit {digit} is outside radix 0..{radix}"
            )));
        }
        if place as usize >= MAX_ARITHMETIC_PLACES {
            return Err(invalid("digit place exceeds MAX_ARITHMETIC_PLACES"));
        }
        let coords = orbit_root_coords(usize::from(digit));
        Ok(Self {
            place,
            radix,
            digit,
            coords,
        })
    }

    /// Recover and validate an [`OrbitDigitAtom`] from an 8D coordinate block using both
    /// strict orbit membership (`orbit_root_id`) and nearest-root `snap(coords, 1)`.
    pub fn from_root_vector(coords: [i32; 8], place: u32, radix: u8) -> Result<Self> {
        validate_radix(radix)?;
        if place as usize >= MAX_ARITHMETIC_PLACES {
            return Err(invalid("digit place exceeds MAX_ARITHMETIC_PLACES"));
        }
        let exact_root = orbit_root_id(&coords)
            .ok_or_else(|| invalid("8D vector is not on the 16-root SpiralCore E8 orbit"))?;
        let snapped = orbit_snap(&coords, 1);
        if exact_root != snapped {
            return Err(invalid(
                "orbit_snap(coords, 1) disagreed with exact root_id",
            ));
        }
        let digit = u8::try_from(exact_root).map_err(|_| invalid("root index overflow"))?;
        if digit >= radix {
            return Err(invalid(format!(
                "orbit root {digit} exceeds declared radix {radix}"
            )));
        }
        Ok(Self {
            place,
            radix,
            digit,
            coords,
        })
    }
}

/// Record of one place-value arithmetic step executed through the 64-state `Cl(0,6)` action table
/// and a ternary carry/borrow trit in `{-1, 0, +1}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaceStepRecord {
    pub place: u32,
    pub left_digit: u8,
    pub right_digit: u8,
    pub carry_in_trit: i8,
    pub cl06_action_id: u8,
    pub cl06_inverse_id: u8,
    pub output_digit: u8,
    pub output_coords: [i32; 8],
    pub carry_out_trit: i8,
}

impl PlaceStepRecord {
    /// Verify that `cl06_inverse_id` (`g^3`) reverses `cl06_action_id` (`g`) exactly on the
    /// 8D `E_8` orbit root, recovering `ROOTS[left_digit]`.
    pub fn verify_exact_inverse(&self) -> bool {
        let inv_coords = cl06_action(usize::from(self.cl06_inverse_id), &self.output_coords);
        let expected_left = orbit_root_coords(usize::from(self.left_digit));
        let composed_id = cl06_compose(
            usize::from(self.cl06_inverse_id),
            usize::from(self.cl06_action_id),
        );
        inv_coords == expected_left
            && cl06_action(composed_id, &expected_left) == expected_left
            && orbit_snap(&self.output_coords, 1) == usize::from(self.output_digit)
    }
}

/// Complete trace of a multi-digit geometric arithmetic operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeometricArithmeticTrace {
    pub schema: String,
    pub radix: u8,
    pub ablation_arm: AblationArm,
    pub steps: Vec<PlaceStepRecord>,
    pub result_atoms: Vec<OrbitDigitAtom>,
    pub fbs_nodes: usize,
    pub fbs_frontier: usize,
    pub fbs_stages: u32,
}

impl GeometricArithmeticTrace {
    /// Verify that every step in the trace satisfies exact `Cl(0,6)` invertibility (`g^3 = g^{-1}`)
    /// and that every result atom sits on its declared `E_8` orbit root.
    pub fn verify_all_inverses(&self) -> bool {
        self.steps.iter().all(PlaceStepRecord::verify_exact_inverse)
            && self.result_atoms.iter().all(|a| {
                orbit_root_id(&a.coords) == Some(usize::from(a.digit))
                    && orbit_snap(&a.coords, 1) == usize::from(a.digit)
            })
    }

    /// Convert the LSB-first `result_atoms` into a `u64` integer using only multiplier-free
    /// repeated addition across radix places.
    pub fn to_u64(&self) -> Result<u64> {
        atoms_to_u64(&self.result_atoms, self.radix)
    }
}

/// Multiplier-free repeated addition of `value` `times` times (`value + value + ...`).
fn repeated_add_u64(value: u64, times: u8) -> Result<u64> {
    let mut acc = 0u64;
    for _ in 0..times {
        acc = acc
            .checked_add(value)
            .ok_or_else(|| invalid("arithmetic u64 overflow in repeated_add_u64"))?;
    }
    Ok(acc)
}

/// Convert LSB-first [`OrbitDigitAtom`] slice to `u64` without hardware multiply instructions.
pub fn atoms_to_u64(atoms: &[OrbitDigitAtom], radix: u8) -> Result<u64> {
    validate_radix(radix)?;
    if atoms.is_empty() {
        return Err(invalid("atoms slice must not be empty"));
    }
    let mut total = 0u64;
    for atom in atoms.iter().rev() {
        if atom.radix != radix || atom.digit >= radix {
            return Err(invalid("atom radix or digit mismatch in atoms_to_u64"));
        }
        if orbit_root_id(&atom.coords) != Some(usize::from(atom.digit)) {
            return Err(invalid("atom coords do not match its orbit digit root"));
        }
        total = repeated_add_u64(total, radix)?;
        total = total
            .checked_add(u64::from(atom.digit))
            .ok_or_else(|| invalid("arithmetic u64 overflow in atoms_to_u64"))?;
    }
    Ok(total)
}

/// Decompose a `u64` integer into LSB-first radix-`B` digits using shift-and-subtract
/// division-free extraction, returning at least one [`OrbitDigitAtom`].
pub fn u64_to_atoms(mut value: u64, radix: u8) -> Result<Vec<OrbitDigitAtom>> {
    validate_radix(radix)?;
    if value == 0 {
        return Ok(vec![OrbitDigitAtom::new(0, 0, radix)?]);
    }
    let mut atoms = Vec::new();
    let mut place = 0u32;
    let b = u64::from(radix);
    while value > 0 {
        if place as usize >= MAX_ARITHMETIC_PLACES {
            return Err(invalid("u64 decomposition exceeded MAX_ARITHMETIC_PLACES"));
        }
        // Division-free quotient and remainder by binary shift-and-subtract.
        let mut q = 0u64;
        let mut rem = value;
        let mut shift = 0u32;
        while (b << shift) <= rem && shift < 60 {
            shift += 1;
        }
        loop {
            let scaled = b << shift;
            if rem >= scaled {
                rem -= scaled;
                q += 1u64 << shift;
            }
            if shift == 0 {
                break;
            }
            shift -= 1;
        }
        let digit = u8::try_from(rem).map_err(|_| invalid("remainder digit overflow"))?;
        atoms.push(OrbitDigitAtom::new(digit, place, radix)?);
        value = q;
        place += 1;
    }
    Ok(atoms)
}

fn validate_radix(radix: u8) -> Result<()> {
    if !(2..=16).contains(&radix) {
        return Err(invalid(format!(
            "radix {radix} is outside the 16-root E8 orbit range 2..=16"
        )));
    }
    Ok(())
}

/// Exact geometric arithmetic engine over the 16-root `E_8` orbit, 64-state `Cl(0,6)` actions,
/// ternary carry/borrow trits, and [`IntegerAddressedMemory`].
#[derive(Clone, Debug)]
pub struct GeometricArithmeticEngine {
    radix: u8,
    max_places: usize,
    ablation_arm: AblationArm,
}

impl GeometricArithmeticEngine {
    /// Create a new engine for `radix` (`2..=16`) and `max_places` (`1..=MAX_ARITHMETIC_PLACES`).
    pub fn new(radix: u8, max_places: usize) -> Result<Self> {
        validate_radix(radix)?;
        if !(1..=MAX_ARITHMETIC_PLACES).contains(&max_places) {
            return Err(invalid(
                "max_places must be in 1..=MAX_ARITHMETIC_PLACES (32)",
            ));
        }
        Ok(Self {
            radix,
            max_places,
            ablation_arm: AblationArm::None,
        })
    }

    /// Return a copy of this engine configured with an [`AblationArm`] causal knockout.
    pub fn with_ablation_arm(&self, arm: AblationArm) -> Self {
        let mut copy = self.clone();
        copy.ablation_arm = arm;
        copy
    }

    pub fn radix(&self) -> u8 {
        self.radix
    }

    pub fn max_places(&self) -> usize {
        self.max_places
    }

    pub fn ablation_arm(&self) -> AblationArm {
        self.ablation_arm
    }

    /// Write an LSB-first digit slice into [`IntegerAddressedMemory`] under `role`.
    pub fn write_digits(
        &self,
        mem: &mut IntegerAddressedMemory,
        role: OperandRole,
        digits_lsb: &[u8],
    ) -> Result<Vec<OrbitDigitAtom>> {
        if digits_lsb.is_empty() {
            return Err(invalid("digits_lsb must not be empty"));
        }
        if digits_lsb.len() > self.max_places {
            return Err(invalid(
                "operand digit length exceeds engine max_places bound",
            ));
        }
        let mut atoms = Vec::with_capacity(digits_lsb.len());
        for (idx, &d) in digits_lsb.iter().enumerate() {
            let place = u32::try_from(idx).map_err(|_| invalid("place index overflow"))?;
            let atom = OrbitDigitAtom::new(d, place, self.radix)?;
            let key = role.place_key(place)?;
            mem.write(key, vec![u32::from(atom.digit)], place)?;
            atoms.push(atom);
        }
        Ok(atoms)
    }

    /// Decompose `value` into LSB-first radix-`B` atoms and write them into `mem` under `role`.
    pub fn write_u64(
        &self,
        mem: &mut IntegerAddressedMemory,
        role: OperandRole,
        value: u64,
    ) -> Result<Vec<OrbitDigitAtom>> {
        let atoms = u64_to_atoms(value, self.radix)?;
        if atoms.len() > self.max_places {
            return Err(invalid("u64 operand exceeds engine max_places bound"));
        }
        for atom in &atoms {
            let key = role.place_key(atom.place)?;
            mem.write(key, vec![u32::from(atom.digit)], atom.place)?;
        }
        Ok(atoms)
    }

    /// Read an operand of `num_places` LSB-first digits from [`IntegerAddressedMemory`].
    ///
    /// - Under [`AblationArm::KnockoutAddressedMemory`] or [`AblationArm::KnockoutBoth`], memory
    ///   lookups are suppressed to [`MemoryLookupOutcome::Absent`], which this reader surfaces as
    ///   an explicit error so memory-dependent compositions fail honestly rather than guessing.
    /// - If any place slot was overwritten ([`MemoryLookupOutcome::Evicted`]), returns an explicit
    ///   eviction error.
    pub fn read_operand(
        &self,
        mem: &IntegerAddressedMemory,
        role: OperandRole,
        num_places: u32,
        version: MemoryVersionSelector,
    ) -> Result<Vec<OrbitDigitAtom>> {
        if num_places == 0 || (num_places as usize) > self.max_places {
            return Err(invalid("num_places must be in 1..=max_places"));
        }
        if self.ablation_arm.knocks_out_addressed_memory() {
            return Err(invalid(
                "addressed memory read knocked out by AblationArm (Absent)",
            ));
        }
        let mut atoms = Vec::with_capacity(num_places as usize);
        for place in 0..num_places {
            let key = role.place_key(place)?;
            match mem.lookup(&key, version) {
                MemoryLookupOutcome::Hit { value_tokens, .. } => {
                    let raw = *value_tokens
                        .first()
                        .ok_or_else(|| invalid("empty value_tokens in addressed memory slot"))?;
                    let digit = u8::try_from(raw)
                        .map_err(|_| invalid("stored digit token out of u8 range"))?;
                    let coords = orbit_root_coords(usize::from(digit));
                    atoms.push(OrbitDigitAtom::from_root_vector(coords, place, self.radix)?);
                }
                MemoryLookupOutcome::Evicted {
                    overwritten_by_turn,
                } => {
                    return Err(invalid(format!(
                        "addressed memory operand place {place} was evicted at turn {overwritten_by_turn}"
                    )));
                }
                MemoryLookupOutcome::Absent => {
                    return Err(invalid(format!(
                        "addressed memory operand place {place} is absent"
                    )));
                }
            }
        }
        Ok(atoms)
    }

    /// Execute one place step (`place`) for addition (`op_sign = 1`) or subtraction (`op_sign = -1`).
    fn execute_place_step(
        &self,
        place: u32,
        left_digit: u8,
        right_digit: u8,
        carry_in_trit: i8,
        op_sign: i8,
    ) -> Result<PlaceStepRecord> {
        if left_digit >= self.radix || right_digit >= self.radix {
            return Err(invalid("operand digit exceeds engine radix"));
        }
        if !(-1..=1).contains(&carry_in_trit) {
            return Err(invalid("carry_in_trit must be in -1..=1"));
        }
        let left_coords = orbit_root_coords(usize::from(left_digit));
        if self.ablation_arm.knocks_out_state_transport() {
            // Under state-transport knockout, Cl(0,6) action is projected to identity A_0 = 0
            // and the ternary carry/borrow gate is zeroed.
            let action_id = 0u8;
            let inverse_id = 0u8;
            let out_coords = cl06_action(usize::from(action_id), &left_coords);
            let out_digit = u8::try_from(orbit_snap(&out_coords, 1))
                .map_err(|_| invalid("snapped root overflow"))?;
            return Ok(PlaceStepRecord {
                place,
                left_digit,
                right_digit,
                carry_in_trit,
                cl06_action_id: action_id,
                cl06_inverse_id: inverse_id,
                output_digit: out_digit,
                output_coords: out_coords,
                carry_out_trit: 0,
            });
        }

        let b = i32::from(self.radix);
        let raw = i32::from(left_digit)
            + (if op_sign >= 0 {
                i32::from(right_digit)
            } else {
                -i32::from(right_digit)
            })
            + i32::from(carry_in_trit);

        let (out_val, carry_out_trit) = if raw >= b {
            (raw - b, 1i8)
        } else if raw < 0 {
            (raw + b, -1i8)
        } else {
            (raw, 0i8)
        };
        let output_digit =
            u8::try_from(out_val).map_err(|_| invalid("output digit out of u8 range"))?;
        let action_usize = cl06_bfs(usize::from(left_digit), usize::from(output_digit));
        let inverse_usize = cl06_inverse(action_usize);
        let output_coords = cl06_action(action_usize, &left_coords);

        if orbit_root_id(&output_coords) != Some(usize::from(output_digit)) {
            return Err(invalid(
                "Cl(0,6) BFS action did not land on expected E8 orbit root",
            ));
        }

        Ok(PlaceStepRecord {
            place,
            left_digit,
            right_digit,
            carry_in_trit,
            cl06_action_id: u8::try_from(action_usize)
                .map_err(|_| invalid("action_id overflow"))?,
            cl06_inverse_id: u8::try_from(inverse_usize)
                .map_err(|_| invalid("inverse_id overflow"))?,
            output_digit,
            output_coords,
            carry_out_trit,
        })
    }

    /// Multi-digit ripple-carry addition over 8D `E_8` orbit atoms and `Cl(0,6)` actions.
    /// Optionally records the resulting digits under [`OperandRole::Accumulator`] in `mem`.
    pub fn add_atoms(
        &self,
        left: &[OrbitDigitAtom],
        right: &[OrbitDigitAtom],
        mut mem: Option<&mut IntegerAddressedMemory>,
    ) -> Result<GeometricArithmeticTrace> {
        self.validate_atom_slice(left)?;
        self.validate_atom_slice(right)?;
        let places = left.len().max(right.len());
        let mut steps = Vec::with_capacity(places + 1);
        let mut result_atoms = Vec::with_capacity(places + 1);
        let mut carry_trit = 0i8;

        for idx in 0..places {
            let place = u32::try_from(idx).map_err(|_| invalid("place overflow"))?;
            let l_dig = left.get(idx).map_or(0, |a| a.digit);
            let r_dig = right.get(idx).map_or(0, |a| a.digit);
            let step = self.execute_place_step(place, l_dig, r_dig, carry_trit, 1)?;
            carry_trit = step.carry_out_trit;
            result_atoms.push(OrbitDigitAtom::from_root_vector(
                step.output_coords,
                place,
                self.radix,
            )?);
            steps.push(step);
        }

        if carry_trit > 0 {
            if places + 1 > self.max_places {
                return Err(invalid("addition carry overflowed engine max_places bound"));
            }
            let place = u32::try_from(places).map_err(|_| invalid("place overflow"))?;
            let step = self.execute_place_step(place, 0, 0, carry_trit, 1)?;
            result_atoms.push(OrbitDigitAtom::from_root_vector(
                step.output_coords,
                place,
                self.radix,
            )?);
            steps.push(step);
        }

        if let Some(m) = mem.as_deref_mut() {
            for atom in &result_atoms {
                let key = OperandRole::Accumulator.place_key(atom.place)?;
                m.write(key, vec![u32::from(atom.digit)], atom.place)?;
            }
        }

        let (fbs_nodes, fbs_frontier, fbs_stages) = fbs_binder_summary(result_atoms.len())?;
        Ok(GeometricArithmeticTrace {
            schema: M6_EXACT_ARITHMETIC_SCHEMA.to_string(),
            radix: self.radix,
            ablation_arm: self.ablation_arm,
            steps,
            result_atoms,
            fbs_nodes,
            fbs_frontier,
            fbs_stages,
        })
    }

    /// Multi-digit borrow subtraction (`left - right`) over 8D `E_8` orbit atoms and `Cl(0,6)`
    /// actions. Refuses unsigned underflow (`left < right`, terminal borrow trit `-1`) with an
    /// explicit error.
    pub fn sub_atoms(
        &self,
        left: &[OrbitDigitAtom],
        right: &[OrbitDigitAtom],
        mut mem: Option<&mut IntegerAddressedMemory>,
    ) -> Result<GeometricArithmeticTrace> {
        self.validate_atom_slice(left)?;
        self.validate_atom_slice(right)?;
        let places = left.len().max(right.len());
        let mut steps = Vec::with_capacity(places);
        let mut result_atoms = Vec::with_capacity(places);
        let mut borrow_trit = 0i8;

        for idx in 0..places {
            let place = u32::try_from(idx).map_err(|_| invalid("place overflow"))?;
            let l_dig = left.get(idx).map_or(0, |a| a.digit);
            let r_dig = right.get(idx).map_or(0, |a| a.digit);
            let step = self.execute_place_step(place, l_dig, r_dig, borrow_trit, -1)?;
            borrow_trit = step.carry_out_trit;
            result_atoms.push(OrbitDigitAtom::from_root_vector(
                step.output_coords,
                place,
                self.radix,
            )?);
            steps.push(step);
        }

        if borrow_trit < 0 {
            return Err(invalid(
                "unsigned subtraction underflow: terminal borrow trit is -1 (left < right)",
            ));
        }

        // Trim leading zero places above the ones place while keeping canonical LSB-first order.
        while result_atoms.len() > 1 && result_atoms.last().is_some_and(|a| a.digit == 0) {
            result_atoms.pop();
        }

        if let Some(m) = mem.as_deref_mut() {
            for atom in &result_atoms {
                let key = OperandRole::Accumulator.place_key(atom.place)?;
                m.write(key, vec![u32::from(atom.digit)], atom.place)?;
            }
        }

        let (fbs_nodes, fbs_frontier, fbs_stages) = fbs_binder_summary(result_atoms.len())?;
        Ok(GeometricArithmeticTrace {
            schema: M6_EXACT_ARITHMETIC_SCHEMA.to_string(),
            radix: self.radix,
            ablation_arm: self.ablation_arm,
            steps,
            result_atoms,
            fbs_nodes,
            fbs_frontier,
            fbs_stages,
        })
    }

    /// Multiplier-free multi-digit multiplication (`left * right`) executed by place-shifted
    /// repeated addition over the `Cl(0,6)` ripple-carry adder, writing each intermediate
    /// partial sum to [`OperandRole::Accumulator`] in `mem`.
    pub fn mul_atoms_shift_and_add(
        &self,
        left: &[OrbitDigitAtom],
        right: &[OrbitDigitAtom],
        mem: &mut IntegerAddressedMemory,
    ) -> Result<GeometricArithmeticTrace> {
        self.validate_atom_slice(left)?;
        self.validate_atom_slice(right)?;

        let mut acc_atoms = vec![OrbitDigitAtom::new(0, 0, self.radix)?];
        let mut all_steps = Vec::new();

        // Write initial zero accumulator to memory so version history tracks [initial, previous, current].
        for atom in &acc_atoms {
            let key = OperandRole::Accumulator.place_key(atom.place)?;
            mem.write(key, vec![u32::from(atom.digit)], 0)?;
        }

        for (shift_idx, r_atom) in right.iter().enumerate() {
            if r_atom.digit == 0 {
                continue;
            }
            // Construct `left` shifted by `shift_idx` radix places (prepending `shift_idx` zero atoms).
            if left.len() + shift_idx > self.max_places {
                return Err(invalid(
                    "shifted multiplicand exceeds engine max_places bound",
                ));
            }
            let mut shifted = Vec::with_capacity(left.len() + shift_idx);
            for p in 0..shift_idx {
                let place = u32::try_from(p).map_err(|_| invalid("place overflow"))?;
                shifted.push(OrbitDigitAtom::new(0, place, self.radix)?);
            }
            for l_atom in left {
                let place = u32::try_from(shifted.len()).map_err(|_| invalid("place overflow"))?;
                shifted.push(OrbitDigitAtom::new(l_atom.digit, place, self.radix)?);
            }

            // Add `shifted` to `acc_atoms` `r_atom.digit` times using the Cl(0,6) ripple-carry adder.
            for _ in 0..r_atom.digit {
                let trace = self.add_atoms(&acc_atoms, &shifted, Some(mem))?;
                all_steps.extend(trace.steps);
                acc_atoms = trace.result_atoms;
            }
        }

        while acc_atoms.len() > 1 && acc_atoms.last().is_some_and(|a| a.digit == 0) {
            acc_atoms.pop();
        }

        let (fbs_nodes, fbs_frontier, fbs_stages) = fbs_binder_summary(acc_atoms.len())?;
        Ok(GeometricArithmeticTrace {
            schema: M6_EXACT_ARITHMETIC_SCHEMA.to_string(),
            radix: self.radix,
            ablation_arm: self.ablation_arm,
            steps: all_steps,
            result_atoms: acc_atoms,
            fbs_nodes,
            fbs_frontier,
            fbs_stages,
        })
    }

    /// Evaluate the two-step memory-bound composition `(OperandA + OperandB) - OperandC`:
    /// 1. Reads `OperandA` and `OperandB` from `mem` via the sorted-merge GCD sieve;
    /// 2. Computes `T_1 = OperandA + OperandB` and stores `T_1` in `Register(0)`;
    /// 3. Reads `Register(0)` and `OperandC` from `mem`, computes `T_2 = T_1 - OperandC`,
    ///    and stores `T_2` in `Accumulator`.
    pub fn eval_memory_add_sub(
        &self,
        mem: &mut IntegerAddressedMemory,
        places_a: u32,
        places_b: u32,
        places_c: u32,
    ) -> Result<GeometricArithmeticTrace> {
        let a = self.read_operand(
            mem,
            OperandRole::OperandA,
            places_a,
            MemoryVersionSelector::Current,
        )?;
        let b = self.read_operand(
            mem,
            OperandRole::OperandB,
            places_b,
            MemoryVersionSelector::Current,
        )?;
        let t1 = self.add_atoms(&a, &b, None)?;
        let t1_digits: Vec<u8> = t1.result_atoms.iter().map(|x| x.digit).collect();
        self.write_digits(mem, OperandRole::Register(0), &t1_digits)?;

        let t1_len = u32::try_from(t1_digits.len()).map_err(|_| invalid("length overflow"))?;
        let t1_read = self.read_operand(
            mem,
            OperandRole::Register(0),
            t1_len,
            MemoryVersionSelector::Current,
        )?;
        let c = self.read_operand(
            mem,
            OperandRole::OperandC,
            places_c,
            MemoryVersionSelector::Current,
        )?;
        let mut t2 = self.sub_atoms(&t1_read, &c, Some(mem))?;
        let mut combined_steps = t1.steps;
        combined_steps.extend(t2.steps);
        t2.steps = combined_steps;
        Ok(t2)
    }

    /// Evaluate the two-step memory-bound composition `(OperandA * OperandB) + OperandC`:
    /// 1. Reads `OperandA` and `OperandB` from `mem` via the sorted-merge GCD sieve;
    /// 2. Computes `T_1 = OperandA * OperandB` by shift-and-add and stores `T_1` in `Register(1)`;
    /// 3. Reads `Register(1)` and `OperandC` from `mem`, computes `T_2 = T_1 + OperandC`,
    ///    and stores `T_2` in `Accumulator`.
    pub fn eval_memory_mul_add(
        &self,
        mem: &mut IntegerAddressedMemory,
        places_a: u32,
        places_b: u32,
        places_c: u32,
    ) -> Result<GeometricArithmeticTrace> {
        let a = self.read_operand(
            mem,
            OperandRole::OperandA,
            places_a,
            MemoryVersionSelector::Current,
        )?;
        let b = self.read_operand(
            mem,
            OperandRole::OperandB,
            places_b,
            MemoryVersionSelector::Current,
        )?;
        let t1 = self.mul_atoms_shift_and_add(&a, &b, mem)?;
        let t1_digits: Vec<u8> = t1.result_atoms.iter().map(|x| x.digit).collect();
        self.write_digits(mem, OperandRole::Register(1), &t1_digits)?;

        let t1_len = u32::try_from(t1_digits.len()).map_err(|_| invalid("length overflow"))?;
        let t1_read = self.read_operand(
            mem,
            OperandRole::Register(1),
            t1_len,
            MemoryVersionSelector::Current,
        )?;
        let c = self.read_operand(
            mem,
            OperandRole::OperandC,
            places_c,
            MemoryVersionSelector::Current,
        )?;
        let mut t2 = self.add_atoms(&t1_read, &c, Some(mem))?;
        let mut combined_steps = t1.steps;
        combined_steps.extend(t2.steps);
        t2.steps = combined_steps;
        Ok(t2)
    }

    fn validate_atom_slice(&self, atoms: &[OrbitDigitAtom]) -> Result<()> {
        if atoms.is_empty() {
            return Err(invalid("operand atom slice must not be empty"));
        }
        if atoms.len() > self.max_places {
            return Err(invalid("operand atom slice exceeds engine max_places"));
        }
        for (idx, atom) in atoms.iter().enumerate() {
            if atom.radix != self.radix || atom.digit >= self.radix {
                return Err(invalid("operand atom radix or digit out of bounds"));
            }
            if atom.place as usize != idx {
                return Err(invalid(
                    "operand atoms must be ordered LSB-first with contiguous place indices 0..len",
                ));
            }
            if orbit_root_id(&atom.coords) != Some(usize::from(atom.digit)) {
                return Err(invalid(
                    "operand atom coordinates do not match its 16-root E8 orbit index",
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Control 1: Bijective orbit digit mapping (`0..10` and `0..16`), `snap(x, 1)` recovery,
    /// `g^4 = I` across all 64 `Cl(0,6)` states, and exact inverse `g^3 = g^{-1}` across all
    /// `16 x 16 = 256` root transitions.
    #[test]
    fn control_orbit_digit_and_cl06_inverse() {
        for d in 0..10u8 {
            let atom = OrbitDigitAtom::new(d, u32::from(d), 10).unwrap();
            let recovered =
                OrbitDigitAtom::from_root_vector(atom.coords, u32::from(d), 10).unwrap();
            assert_eq!(atom, recovered);
        }
        for d in 0..16u8 {
            let atom = OrbitDigitAtom::new(d, u32::from(d), 16).unwrap();
            let recovered =
                OrbitDigitAtom::from_root_vector(atom.coords, u32::from(d), 16).unwrap();
            assert_eq!(atom, recovered);
        }

        // Verify g^4 = I (action 0) for all 64 Cl(0,6) operators.
        for g in 0..64usize {
            let g2 = cl06_compose(g, g);
            let g3 = cl06_compose(g, g2);
            let g4 = cl06_compose(g, g3);
            assert_eq!(g3, cl06_inverse(g));
            assert_eq!(g4, 0, "g^4 must equal identity state 0 for g={g}");
        }

        // Verify all 16 x 16 = 256 root transitions and their exact inverses.
        let mut verified_pairs = 0usize;
        for src in 0..16usize {
            let src_coords = orbit_root_coords(src);
            for dst in 0..16usize {
                let dst_coords = orbit_root_coords(dst);
                let g = cl06_bfs(src, dst);
                let inv = cl06_inverse(g);
                let transported = cl06_action(g, &src_coords);
                assert_eq!(transported, dst_coords);
                assert_eq!(orbit_snap(&transported, 1), dst);
                let reversed = cl06_action(inv, &transported);
                assert_eq!(reversed, src_coords);
                verified_pairs += 1;
            }
        }
        assert_eq!(verified_pairs, 256);
    }

    /// Control 2: Multi-digit decimal (`radix = 10`) and hexadecimal (`radix = 16`) ripple-carry
    /// addition and borrow subtraction across 32 deterministic cases, verifying exact 8D orbit
    /// atoms, ternary FBS binder structure, and `Cl(0,6)` inverse traces.
    #[test]
    fn control_ripple_carry_add_and_sub() {
        let dec_engine = GeometricArithmeticEngine::new(10, 12).unwrap();
        let dec_pairs: [(u64, u64); 12] = [
            (0, 0),
            (7, 8),
            (9, 1),
            (99, 1),
            (999, 1),
            (4789, 5643),
            (12345, 67890),
            (50000, 50000),
            (999_999, 1),
            (314_159, 271_828),
            (808_080, 191_920),
            (987_654, 12_346),
        ];

        let mut passed = 0usize;
        for (a, b) in dec_pairs {
            let a_atoms = u64_to_atoms(a, 10).unwrap();
            let b_atoms = u64_to_atoms(b, 10).unwrap();
            let add_trace = dec_engine.add_atoms(&a_atoms, &b_atoms, None).unwrap();
            assert!(add_trace.verify_all_inverses());
            assert_eq!(add_trace.to_u64().unwrap(), a + b);
            assert!(add_trace.fbs_nodes >= add_trace.result_atoms.len());
            passed += 1;

            let sub_trace = dec_engine
                .sub_atoms(&add_trace.result_atoms, &b_atoms, None)
                .unwrap();
            assert!(sub_trace.verify_all_inverses());
            assert_eq!(sub_trace.to_u64().unwrap(), a);
            passed += 1;
        }

        let hex_engine = GeometricArithmeticEngine::new(16, 12).unwrap();
        let hex_pairs: [(u64, u64); 4] = [
            (0x00FF, 0x0001),
            (0xFFFF, 0x0001),
            (0x1A2B, 0x3C4D),
            (0xDEAD, 0xBEEF),
        ];
        for (a, b) in hex_pairs {
            let a_atoms = u64_to_atoms(a, 16).unwrap();
            let b_atoms = u64_to_atoms(b, 16).unwrap();
            let add_trace = hex_engine.add_atoms(&a_atoms, &b_atoms, None).unwrap();
            assert!(add_trace.verify_all_inverses());
            assert_eq!(add_trace.to_u64().unwrap(), a + b);
            passed += 1;

            let sub_trace = hex_engine
                .sub_atoms(&add_trace.result_atoms, &b_atoms, None)
                .unwrap();
            assert!(sub_trace.verify_all_inverses());
            assert_eq!(sub_trace.to_u64().unwrap(), a);
            passed += 1;
        }
        assert_eq!(passed, 32);
    }

    /// Control 3: Multi-step compositions (`(A + B) - C` and `(A * B) + C`) through
    /// `IntegerAddressedMemory` with `[initial, previous, current]` version history verification
    /// across 16 multi-step items.
    #[test]
    fn control_addressed_memory_composition_and_multiply() {
        let engine = GeometricArithmeticEngine::new(10, 16).unwrap();

        let add_sub_cases: [(u64, u64, u64); 8] = [
            (999, 1, 500),
            (4789, 5643, 1432),
            (12000, 8500, 20500),
            (765, 432, 197),
            (99999, 1, 88888),
            (54321, 12345, 66660),
            (1024, 2048, 512),
            (3333, 7777, 1110),
        ];

        let mut passed = 0usize;
        for (a, b, c) in add_sub_cases {
            let mut mem = IntegerAddressedMemory::new(64).unwrap();
            let a_atoms = engine
                .write_u64(&mut mem, OperandRole::OperandA, a)
                .unwrap();
            let b_atoms = engine
                .write_u64(&mut mem, OperandRole::OperandB, b)
                .unwrap();
            let c_atoms = engine
                .write_u64(&mut mem, OperandRole::OperandC, c)
                .unwrap();
            let trace = engine
                .eval_memory_add_sub(
                    &mut mem,
                    a_atoms.len() as u32,
                    b_atoms.len() as u32,
                    c_atoms.len() as u32,
                )
                .unwrap();
            assert!(trace.verify_all_inverses());
            assert_eq!(trace.to_u64().unwrap(), (a + b) - c);
            passed += 1;
        }

        let mul_add_cases: [(u64, u64, u64); 8] = [
            (12, 34, 56),
            (99, 99, 1),
            (125, 8, 1000),
            (37, 27, 1),
            (256, 16, 4096),
            (405, 19, 305),
            (700, 13, 900),
            (999, 9, 9),
        ];

        for (a, b, c) in mul_add_cases {
            let mut mem = IntegerAddressedMemory::new(128).unwrap();
            let a_atoms = engine
                .write_u64(&mut mem, OperandRole::OperandA, a)
                .unwrap();
            let b_atoms = engine
                .write_u64(&mut mem, OperandRole::OperandB, b)
                .unwrap();
            let c_atoms = engine
                .write_u64(&mut mem, OperandRole::OperandC, c)
                .unwrap();
            let trace = engine
                .eval_memory_mul_add(
                    &mut mem,
                    a_atoms.len() as u32,
                    b_atoms.len() as u32,
                    c_atoms.len() as u32,
                )
                .unwrap();
            assert!(trace.verify_all_inverses());
            assert_eq!(trace.to_u64().unwrap(), a * b + c);

            // Verify that Accumulator place 0 in IntegerAddressedMemory recorded initial and current versions.
            let acc_p0 = OperandRole::Accumulator.place_key(0).unwrap();
            match mem.lookup(&acc_p0, MemoryVersionSelector::Initial) {
                MemoryLookupOutcome::Hit { value_tokens, .. } => {
                    assert_eq!(value_tokens, vec![0]);
                }
                other => panic!("expected Initial Hit on Accumulator place 0, got {other:?}"),
            }
            match mem.lookup(&acc_p0, MemoryVersionSelector::Current) {
                MemoryLookupOutcome::Hit { value_tokens, .. } => {
                    assert_eq!(value_tokens, vec![((a * b + c) % 10) as u32]);
                }
                other => panic!("expected Current Hit on Accumulator place 0, got {other:?}"),
            }
            passed += 1;
        }
        assert_eq!(passed, 16);
    }

    /// Control 4: Mandatory failure / refusal controls and causal intermediate-state knockouts.
    #[test]
    fn control_refusal_and_causal_knockouts() {
        // 1. Invalid radix (< 2 or > 16) and out-of-radix digit must fail.
        assert!(GeometricArithmeticEngine::new(1, 8).is_err());
        assert!(GeometricArithmeticEngine::new(17, 8).is_err());
        assert!(OrbitDigitAtom::new(10, 0, 10).is_err());

        // 2. Off-orbit 8D coordinate vector must fail.
        assert!(OrbitDigitAtom::from_root_vector([1, 1, 1, 1, 0, 0, 0, 0], 0, 10).is_err());

        // 3. Unsigned subtraction underflow (A < B) must fail.
        let engine = GeometricArithmeticEngine::new(10, 4).unwrap();
        let a = u64_to_atoms(42, 10).unwrap();
        let b = u64_to_atoms(99, 10).unwrap();
        assert!(engine.sub_atoms(&a, &b, None).is_err());

        // 4. Fixed-width carry overflow beyond max_places must fail.
        let narrow = GeometricArithmeticEngine::new(10, 3).unwrap();
        let n999 = u64_to_atoms(999, 10).unwrap();
        let n1 = u64_to_atoms(1, 10).unwrap();
        assert!(narrow.add_atoms(&n999, &n1, None).is_err());

        // 5. Evicted operand slot in IntegerAddressedMemory must fail (never silently read 0).
        let mut tiny_mem = IntegerAddressedMemory::new(2).unwrap();
        engine
            .write_u64(&mut tiny_mem, OperandRole::OperandA, 99)
            .unwrap();
        // Overwrite the 2-slot ring with OperandB so OperandA's slots are evicted.
        engine
            .write_u64(&mut tiny_mem, OperandRole::OperandB, 77)
            .unwrap();
        let evicted_err = engine
            .read_operand(
                &tiny_mem,
                OperandRole::OperandA,
                2,
                MemoryVersionSelector::Current,
            )
            .unwrap_err();
        assert!(evicted_err.to_string().contains("evicted"));

        // 6. Causal ablation knockouts: KnockoutStateTransport and KnockoutAddressedMemory
        // drop multi-step carry/memory accuracy from 16/16 to 0/16.
        let wide_engine = GeometricArithmeticEngine::new(10, 8).unwrap();
        let cases: [(u64, u64, u64); 16] = [
            (99, 1, 10),
            (199, 2, 50),
            (299, 3, 100),
            (399, 4, 150),
            (499, 5, 200),
            (599, 6, 250),
            (699, 7, 300),
            (799, 8, 350),
            (899, 9, 400),
            (999, 1, 500),
            (4789, 5643, 1432),
            (5555, 6666, 2221),
            (7777, 8888, 6665),
            (1234, 9876, 1110),
            (9009, 1091, 100),
            (9999, 1, 9000),
        ];

        let mut none_correct = 0usize;
        let mut knockout_transport_correct = 0usize;
        let mut knockout_memory_correct = 0usize;

        for (va, vb, vc) in cases {
            let expected = (va + vb) - vc;

            let mut mem_none = IntegerAddressedMemory::new(64).unwrap();
            let pa = wide_engine
                .write_u64(&mut mem_none, OperandRole::OperandA, va)
                .unwrap()
                .len() as u32;
            let pb = wide_engine
                .write_u64(&mut mem_none, OperandRole::OperandB, vb)
                .unwrap()
                .len() as u32;
            let pc = wide_engine
                .write_u64(&mut mem_none, OperandRole::OperandC, vc)
                .unwrap()
                .len() as u32;

            let res_none = wide_engine
                .eval_memory_add_sub(&mut mem_none, pa, pb, pc)
                .unwrap();
            if res_none.to_u64().unwrap() == expected {
                none_correct += 1;
            }

            let transport_ko = wide_engine.with_ablation_arm(AblationArm::KnockoutStateTransport);
            if let Ok(res_t) = transport_ko.eval_memory_add_sub(&mut mem_none.clone(), pa, pb, pc) {
                if res_t.to_u64().unwrap() == expected {
                    knockout_transport_correct += 1;
                }
            }

            let memory_ko = wide_engine.with_ablation_arm(AblationArm::KnockoutAddressedMemory);
            if let Ok(res_m) = memory_ko.eval_memory_add_sub(&mut mem_none.clone(), pa, pb, pc) {
                if res_m.to_u64().unwrap() == expected {
                    knockout_memory_correct += 1;
                }
            }
        }

        assert_eq!(none_correct, 16);
        assert_eq!(knockout_transport_correct, 0);
        assert_eq!(knockout_memory_correct, 0);
    }
}
