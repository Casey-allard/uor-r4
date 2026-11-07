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

const MAX_EXACT_TOOL_PROMPT_BYTES: usize = 160;
const MAX_EXACT_TOOL_OPERAND: u64 = 9_999_999_999;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactArithmeticToolAnswer {
    pub expression: String,
    pub result: String,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExactToolOperator {
    Add,
    Subtract,
}

fn exact_tool_operand_start(rest: &str) -> bool {
    rest.trim_start().starts_with(|c: char| {
        matches!(
            c,
            '0'..='9'
                | '+'
                | '-'
                | '.'
                | '('
                | '{'
                | '['
                | '%'
                | '^'
                | '*'
                | '/'
                | '×'
                | '÷'
                | '&'
                | '|'
                | '='
                | '<'
                | '>'
                | '~'
                | '−'
                | '≤'
                | '≥'
                | '≠'
                | '·'
        )
    })
}

fn exact_tool_compact_subtraction_shape(expression: &str) -> bool {
    let mut fields = expression.split('-');
    let (Some(left), Some(right), None) = (fields.next(), fields.next(), fields.next()) else {
        return false;
    };
    let left_raw = left;
    let right_raw = right;
    let left = left.trim();
    let right = right.trim();
    if left.is_empty()
        || right.is_empty()
        || !left.chars().all(|c| c.is_ascii_digit())
        || !right.chars().all(|c| c.is_ascii_digit())
    {
        return false;
    }
    let adjacent = left_raw == left && right_raw == right;
    let no_space_date_pair = adjacent
        && if left.len() == 4 && right.len() == 2 {
            right
                .parse::<u8>()
                .is_ok_and(|month| (1..=12).contains(&month))
        } else if left.len() == 2 && right.len() == 2 {
            left.parse::<u8>()
                .is_ok_and(|month| (1..=12).contains(&month))
                && right.parse::<u8>().is_ok_and(|day| (1..=31).contains(&day))
        } else {
            false
        };
    !no_space_date_pair
}

fn exact_tool_count_prose(text: &str) -> bool {
    let alphabetic_words = text
        .split_whitespace()
        .filter(|word| word.chars().any(char::is_alphabetic))
        .count();
    let required_words = if text.trim_start().starts_with(|c: char| c.is_ascii_digit()) {
        1
    } else {
        2
    };
    if alphabetic_words < required_words {
        return false;
    }
    !text.chars().any(|c| {
        matches!(
            c,
            '+' | '-'
                | '*'
                | '/'
                | '%'
                | '^'
                | '&'
                | '|'
                | '='
                | '<'
                | '>'
                | '~'
                | '−'
                | '≤'
                | '≥'
                | '≠'
                | '·'
                | '('
                | '{'
                | '['
        )
    })
}

fn exact_tool_compact_range_prose(expression: &str, suffix: &str) -> bool {
    if !exact_tool_compact_subtraction_shape(expression) || suffix.is_empty() {
        return false;
    }
    if suffix.chars().any(|c| {
        matches!(
            c,
            '+' | '-'
                | '*'
                | '/'
                | '%'
                | '^'
                | '&'
                | '|'
                | '='
                | '<'
                | '>'
                | '~'
                | '−'
                | '≤'
                | '≥'
                | '≠'
                | '·'
                | '('
                | '{'
                | '['
        )
    }) {
        return false;
    }
    if suffix.chars().any(|c| c.is_ascii_digit()) {
        return exact_tool_count_prose(suffix);
    }
    exact_tool_ratio_suffix_allowed(&suffix.to_ascii_lowercase())
}

fn exact_tool_arithmetic_shaped(rest: &str) -> bool {
    let rest = rest.trim_start();
    let signed_literal = rest.trim_end_matches(|c| matches!(c, '?' | '.' | '!' | ' ' | '\t'));
    if (signed_literal.starts_with('-') || signed_literal.starts_with('+'))
        && signed_literal[1..].chars().all(|c| c.is_ascii_digit())
    {
        return false;
    }
    if !rest.starts_with(|c: char| {
        matches!(
            c,
            '0'..='9'
                | '+'
                | '-'
                | '.'
                | '('
                | '{'
                | '['
                | '%'
                | '^'
                | '*'
                | '/'
                | '×'
                | '÷'
                | '&'
                | '|'
                | '='
                | '<'
                | '>'
                | '~'
                | '−'
                | '≤'
                | '≥'
                | '≠'
                | '·'
        )
    }) || !rest.chars().any(|c| c.is_ascii_digit())
    {
        return false;
    }
    for (index, c) in rest.char_indices() {
        if index == 0
            && matches!(c, '+' | '-')
            && rest[1..]
                .chars()
                .next()
                .is_some_and(|next| next.is_ascii_digit())
        {
            continue;
        }
        if matches!(
            c,
            '+' | '*'
                | '/'
                | '%'
                | '^'
                | '×'
                | '÷'
                | '&'
                | '|'
                | '='
                | '<'
                | '>'
                | '~'
                | '−'
                | '≤'
                | '≥'
                | '≠'
                | '·'
                | '('
                | '{'
                | '['
        ) {
            return true;
        }
        if c == '-' {
            let previous = rest[..index].chars().next_back();
            let next = rest[index + 1..].chars().next();
            if (previous.is_some_and(char::is_whitespace) || previous == Some(','))
                && (next.is_none()
                    || next.is_some_and(|next| next.is_ascii_digit())
                    || (next.is_some_and(char::is_whitespace)
                        && exact_tool_operand_start(&rest[index + 1..])))
            {
                return true;
            }
        }
    }
    false
}

fn exact_tool_time_atom(text: &str) -> bool {
    let lower = text
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    let (clock, meridiem) = if let Some(clock) = lower.strip_suffix("am") {
        (clock, true)
    } else if let Some(clock) = lower.strip_suffix("pm") {
        (clock, true)
    } else {
        (lower.as_str(), false)
    };
    let fields: Vec<&str> = clock.split(':').collect();
    if fields.len() > 3 || (fields.len() == 1 && !meridiem) {
        return false;
    }
    if fields[0].is_empty() || fields[0].len() > 2 || !fields[0].chars().all(|c| c.is_ascii_digit())
    {
        return false;
    }
    let Ok(hour) = fields[0].parse::<u8>() else {
        return false;
    };
    let valid_hour = if meridiem {
        (1..=12).contains(&hour)
    } else {
        hour <= 23
    };
    valid_hour
        && (fields.len() == 1
            || (fields[1].len() == 2
                && fields[1].chars().all(|c| c.is_ascii_digit())
                && fields[1].parse::<u8>().is_ok_and(|minute| minute <= 59)))
        && (fields.len() <= 2
            || (fields[2].len() == 2
                && fields[2].chars().all(|c| c.is_ascii_digit())
                && fields[2].parse::<u8>().is_ok_and(|second| second <= 59)))
}

fn exact_tool_time_range_tail_is_prose(text: &str) -> bool {
    !text.chars().any(|c| {
        c.is_ascii_digit()
            || matches!(
                c,
                '+' | '-'
                    | '*'
                    | '/'
                    | '%'
                    | '^'
                    | '×'
                    | '÷'
                    | '&'
                    | '|'
                    | '='
                    | '<'
                    | '>'
                    | '~'
                    | '−'
                    | '≤'
                    | '≥'
                    | '≠'
                    | '·'
                    | '('
                    | '{'
                    | '['
            )
    })
}

fn exact_tool_time_atom_prefix(text: &str) -> Option<&str> {
    for (index, c) in text.char_indices() {
        if index == 0 {
            continue;
        }
        let candidate = text[..index].trim_end();
        if candidate.is_empty() || !exact_tool_time_atom(candidate) {
            continue;
        }
        if c.is_alphanumeric() {
            continue;
        }
        return Some(text[index..].trim_start());
    }
    None
}

fn exact_tool_time_or_prose(text: &str) -> bool {
    let text = text.trim();
    if exact_tool_time_atom(text) {
        return true;
    }
    match exact_tool_time_atom_prefix(text) {
        Some(remainder) => exact_tool_time_range_tail_is_prose(remainder),
        None => false,
    }
}

fn exact_tool_has_meridiem(text: &str) -> bool {
    let compact = text
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    compact.ends_with("am") || compact.ends_with("pm")
}

fn exact_tool_meridiem_time_prefix(text: &str) -> Option<&str> {
    for (index, c) in text.char_indices() {
        if index == 0 {
            continue;
        }
        let candidate = text[..index].trim_end();
        if candidate.is_empty()
            || !exact_tool_time_atom(candidate)
            || !exact_tool_has_meridiem(candidate)
        {
            continue;
        }
        if c.is_alphanumeric() {
            continue;
        }
        return Some(text[index..].trim_start());
    }
    None
}

fn exact_tool_meridiem_time_or_prose(text: &str) -> bool {
    let text = text.trim();
    if exact_tool_time_atom(text) && exact_tool_has_meridiem(text) {
        return true;
    }
    match exact_tool_meridiem_time_prefix(text) {
        Some(remainder) => exact_tool_time_range_tail_is_prose(remainder),
        None => false,
    }
}

fn exact_tool_bare_hour_prose(text: &str) -> bool {
    let text = text.trim_start();
    let hour_len = text.chars().take_while(|c| c.is_ascii_digit()).count();
    if hour_len == 0 || hour_len > 2 {
        return false;
    }
    let Ok(hour) = text[..hour_len].parse::<u8>() else {
        return false;
    };
    if !(1..=12).contains(&hour) {
        return false;
    }
    let mut rest = &text[hour_len..];
    if let Some(after_colon) = rest.strip_prefix(':') {
        let minute_len = after_colon
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .count();
        if minute_len != 2 {
            return false;
        }
        let Ok(minute) = after_colon[..2].parse::<u8>() else {
            return false;
        };
        if minute > 59 {
            return false;
        }
        rest = &after_colon[2..];
        if let Some(after_second_colon) = rest.strip_prefix(':') {
            let second_len = after_second_colon
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .count();
            if second_len != 2 {
                return false;
            }
            let Ok(second) = after_second_colon[..2].parse::<u8>() else {
                return false;
            };
            if second > 59 {
                return false;
            }
            rest = &after_second_colon[2..];
        }
    }
    if rest.chars().next().is_some_and(|c| c.is_alphanumeric()) {
        return false;
    }
    exact_tool_time_range_tail_is_prose(rest)
}

fn exact_tool_is_time_range(rest: &str) -> bool {
    let rest = rest.trim_end_matches(|c| matches!(c, '?' | '.' | '!' | ',' | ' ' | '\t'));
    let Some((left, right)) = rest.split_once(" - ") else {
        return false;
    };
    let left = left.trim();
    let right = right.trim();
    if exact_tool_time_atom(left) && exact_tool_time_or_prose(right) {
        return true;
    }
    if exact_tool_time_atom(left) && exact_tool_bare_hour_prose(right) {
        return true;
    }
    if exact_tool_bare_hour_prose(left) && exact_tool_time_atom(right) {
        return true;
    }
    if exact_tool_meridiem_time_or_prose(left) && exact_tool_bare_hour_prose(right) {
        return true;
    }
    if exact_tool_meridiem_time_or_prose(right) && exact_tool_bare_hour_prose(left) {
        return true;
    }
    false
}

fn exact_tool_phone_digits(text: &str, count: usize) -> Option<&str> {
    let digits = text.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits != count {
        return None;
    }
    Some(&text[count..])
}

fn exact_tool_phone_separator(text: &str) -> Option<&str> {
    if let Some(rest) = text.strip_prefix('-') {
        return Some(rest);
    }
    let spaces = text.chars().take_while(|c| matches!(c, ' ' | '\t')).count();
    if spaces == 0 {
        return None;
    }
    Some(&text[spaces..])
}

fn exact_tool_phone_area(text: &str) -> Option<&str> {
    if let Some(rest) = text.strip_prefix('(') {
        let after = exact_tool_phone_digits(rest, 3)?;
        return after.strip_prefix(')');
    }
    exact_tool_phone_digits(text, 3)
}

fn exact_tool_phone_shape_matches(text: &str) -> bool {
    let Some(after_area) = exact_tool_phone_area(text) else {
        return false;
    };
    let Some(after_area_sep) = exact_tool_phone_separator(after_area) else {
        return false;
    };
    let Some(after_exchange) = exact_tool_phone_digits(after_area_sep, 3) else {
        return false;
    };
    let Some(after_exchange_sep) = exact_tool_phone_separator(after_exchange) else {
        return false;
    };
    let Some(after_subscriber) = exact_tool_phone_digits(after_exchange_sep, 4) else {
        return false;
    };
    after_subscriber.trim().is_empty()
}

fn exact_tool_is_phone_shape(rest: &str) -> bool {
    let rest = rest.trim_end_matches(|c| matches!(c, '?' | '.' | '!' | ',' | ' ' | '\t'));
    let body = rest.strip_prefix('+').unwrap_or(rest);
    if body.contains('+') {
        return false;
    }
    if exact_tool_phone_shape_matches(body) {
        return true;
    }
    if let Some(after_country) = body.strip_prefix('1') {
        if let Some(after_separator) = exact_tool_phone_separator(after_country) {
            return exact_tool_phone_shape_matches(after_separator);
        }
    }
    false
}

fn exact_tool_two_digits(bytes: &[u8], offset: usize) -> Option<u8> {
    let tens = *bytes.get(offset)?;
    let ones = *bytes.get(offset + 1)?;
    if !tens.is_ascii_digit() || !ones.is_ascii_digit() {
        return None;
    }
    Some((tens - b'0') * 10 + ones - b'0')
}

fn exact_tool_is_iso_date_time_core(rest: &str) -> bool {
    let bytes = rest.as_bytes();
    if bytes.len() < 10
        || !bytes[..4].iter().all(u8::is_ascii_digit)
        || bytes[4] != b'-'
        || bytes[7] != b'-'
    {
        return false;
    }
    let Some(month) = exact_tool_two_digits(bytes, 5) else {
        return false;
    };
    let Some(day) = exact_tool_two_digits(bytes, 8) else {
        return false;
    };
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return false;
    }
    if bytes.len() == 10 {
        return true;
    }
    if !matches!(bytes[10], b'T' | b't' | b' ' | b'\t') {
        return false;
    }
    let time = &bytes[11..];
    let Some(hour) = exact_tool_two_digits(time, 0) else {
        return false;
    };
    if hour > 23 {
        return false;
    }
    if time.len() == 2 {
        return true;
    }
    if time[2] != b':' {
        return false;
    }
    let Some(minute) = exact_tool_two_digits(time, 3) else {
        return false;
    };
    if minute > 59 {
        return false;
    }
    let mut cursor = 5;
    if cursor < time.len() && time[cursor] == b':' {
        let Some(second) = exact_tool_two_digits(time, 6) else {
            return false;
        };
        if second > 59 {
            return false;
        }
        cursor = 8;
        if cursor < time.len() && time[cursor] == b'.' {
            cursor += 1;
            let fraction_start = cursor;
            while cursor < time.len() && time[cursor].is_ascii_digit() {
                cursor += 1;
            }
            if cursor == fraction_start {
                return false;
            }
        }
    }
    if cursor == time.len() {
        return true;
    }
    let zone = &time[cursor..];
    match zone[0] {
        b'Z' | b'z' => zone.len() == 1,
        b'+' | b'-' => {
            let Some(zone_hour) = exact_tool_two_digits(zone, 1) else {
                return false;
            };
            if zone_hour > 14 {
                return false;
            }
            match zone.len() {
                3 => true,
                5 => exact_tool_two_digits(zone, 3).is_some_and(|minute| minute <= 59),
                6 if zone[3] == b':' => {
                    exact_tool_two_digits(zone, 4).is_some_and(|minute| minute <= 59)
                }
                _ => false,
            }
        }
        _ => false,
    }
}

fn exact_tool_leading_numeric_dash(text: &str) -> bool {
    let text = text.trim_start();
    let Some(after) = text.strip_prefix('-') else {
        return false;
    };
    after.chars().next().is_some_and(char::is_whitespace)
        && after.trim_start().starts_with(|c: char| c.is_ascii_digit())
}

fn exact_tool_signed_numeric_tail(text: &str) -> bool {
    let text = text.trim();
    let Some(sign) = text.chars().next() else {
        return false;
    };
    if !matches!(sign, '+' | '-') {
        return false;
    }
    let digits = text[sign.len_utf8()..].trim_start();
    !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
}

fn exact_tool_is_iso_date_time(rest: &str) -> bool {
    let rest = rest.trim_end_matches(|c| matches!(c, '?' | '.' | '!' | ',' | ' ' | '\t'));
    if exact_tool_is_iso_date_time_core(rest) {
        return true;
    }
    if let Some((head, tail)) = rest.split_once(" - ") {
        if exact_tool_is_iso_date_time_core(head.trim_end()) && exact_tool_time_or_prose(tail) {
            return true;
        }
    }
    for (index, c) in rest.char_indices() {
        if !matches!(c, ',' | ' ' | '\t') {
            continue;
        }
        let head = &rest[..index];
        let tail = rest[index + 1..].trim_start_matches(|c| matches!(c, ',' | ' ' | '\t'));
        if exact_tool_is_iso_date_time_core(head) {
            if exact_tool_is_time_range(tail) {
                return true;
            }
            if !tail.starts_with('+')
                && !exact_tool_signed_numeric_tail(tail)
                && !exact_tool_leading_operator_minus(tail)
                && !exact_tool_leading_numeric_dash(tail)
                && !exact_tool_arithmetic_shaped(tail)
            {
                return true;
            }
        }
    }
    false
}

fn exact_tool_is_iso_date_range(rest: &str) -> bool {
    let rest = rest.trim_end_matches(|c| matches!(c, '?' | '.' | '!' | ',' | ' ' | '\t'));
    let Some((left, right)) = rest.split_once(" - ") else {
        return false;
    };
    let left = left.trim();
    let right = right.trim();
    if exact_tool_is_iso_date_time_core(left) && exact_tool_is_iso_date_time_core(right) {
        return true;
    }
    for (index, c) in right.char_indices() {
        if !matches!(c, ',' | ' ' | '\t') {
            continue;
        }
        let head = &right[..index];
        let tail = right[index + 1..].trim_start();
        if exact_tool_is_iso_date_time_core(left)
            && exact_tool_is_iso_date_time_core(head)
            && exact_tool_count_prose(tail)
        {
            return true;
        }
    }
    false
}

fn exact_tool_leading_operator_minus(expression: &str) -> bool {
    let expression = expression.trim_end_matches(|c| matches!(c, '?' | '.' | '!' | ' ' | '\t'));
    if !expression.starts_with('-') || expression.chars().any(char::is_alphabetic) {
        return false;
    }
    let after_sign = &expression[1..];
    let separated = after_sign.chars().next().is_some_and(char::is_whitespace);
    let remainder = after_sign.trim_start();
    if remainder.is_empty() {
        return true;
    }
    if !separated && remainder.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return false;
    }
    remainder.starts_with(|c: char| {
        matches!(
            c,
            '0'..='9'
                | '.'
                | '?'
                | '!'
                | '('
                | '{'
                | '['
                | '+'
                | '-'
                | '%'
                | '^'
                | '*'
                | '/'
                | '×'
                | '÷'
        )
    })
}

fn exact_tool_ratio_unit_prefix(text: &str, unit: &str) -> bool {
    let Some(rest) = text.strip_prefix(unit) else {
        return false;
    };
    rest.is_empty() || !rest.chars().next().is_some_and(|c| c.is_alphanumeric())
}

fn exact_tool_ratio_suffix_allowed(text: &str) -> bool {
    [
        "support",
        "of",
        "cup",
        "cups",
        "gallon",
        "gallons",
        "gram",
        "grams",
        "liter",
        "liters",
        "dozen",
        "dozens",
        "ml",
        "l",
        "g",
        "kg",
        "mg",
        "km",
        "kilometer",
        "kilometers",
        "cm",
        "centimeter",
        "centimeters",
        "min",
        "student",
        "students",
        "dollar",
        "dollars",
        "oz",
        "lb",
        "lbs",
        "tablespoon",
        "tablespoons",
        "teaspoon",
        "teaspoons",
        "hour",
        "hours",
        "minute",
        "minutes",
        "second",
        "seconds",
        "day",
        "days",
        "week",
        "weeks",
        "year",
        "years",
        "people",
        "person",
        "item",
        "items",
        "part",
        "parts",
        "piece",
        "pieces",
        "time",
        "times",
    ]
    .iter()
    .any(|unit| exact_tool_ratio_unit_prefix(text, unit))
}

fn exact_tool_is_ratio_prose(rest: &str) -> bool {
    let rest = rest.trim();
    let Some((numerator, tail)) = rest.split_once('/') else {
        return false;
    };
    if numerator.is_empty()
        || numerator.len() > 10
        || !numerator.chars().all(|c| c.is_ascii_digit())
    {
        return false;
    }
    if tail.contains('/') {
        return false;
    }
    let tail = tail.trim_start();
    let denominator_len = tail.chars().take_while(|c| c.is_ascii_digit()).count();
    if denominator_len == 0 || denominator_len > 10 {
        return false;
    }
    let after = tail[denominator_len..].trim_start();
    if after.is_empty() || !after.chars().next().is_some_and(char::is_alphabetic) {
        return false;
    }
    let lower_after = after.to_ascii_lowercase();
    if !exact_tool_ratio_suffix_allowed(&lower_after) {
        return false;
    }
    if after.chars().any(|c| {
        matches!(
            c,
            '+' | '-'
                | '*'
                | '/'
                | '%'
                | '^'
                | '×'
                | '÷'
                | '&'
                | '|'
                | '='
                | '<'
                | '>'
                | '~'
                | '−'
                | '≤'
                | '≥'
                | '≠'
                | '·'
                | '('
                | '{'
                | '['
        )
    }) {
        return false;
    }
    if after.chars().any(|c| c.is_ascii_digit()) {
        return exact_tool_count_prose(after);
    }
    true
}

fn exact_tool_number_leading_prose(text: &str) -> bool {
    let text = text.trim();
    let number_len = text.chars().take_while(|c| c.is_ascii_digit()).count();
    if number_len == 0 {
        return false;
    }
    let remainder = text[number_len..].trim_start();
    if remainder.is_empty() || !remainder.chars().any(|c| c.is_alphabetic()) {
        return false;
    }
    !remainder.chars().any(|c| {
        c.is_ascii_digit()
            || matches!(
                c,
                '+' | '-'
                    | '*'
                    | '/'
                    | '%'
                    | '^'
                    | '×'
                    | '÷'
                    | '&'
                    | '|'
                    | '='
                    | '<'
                    | '>'
                    | '~'
                    | '−'
                    | '≤'
                    | '≥'
                    | '≠'
                    | '·'
                    | '('
                    | '{'
                    | '['
                    | ')'
                    | '}'
                    | ']'
            )
    })
}

fn exact_tool_is_number_leading_dash_prose(rest: &str) -> bool {
    let Some((left, right)) = rest.split_once(" - ") else {
        return false;
    };
    exact_tool_number_leading_prose(left) && exact_tool_number_leading_prose(right)
}

fn exact_tool_numeric_range(expression: &str, has_prefix: bool) -> bool {
    if has_prefix {
        return false;
    }
    let fields: Vec<&str> = expression.split_whitespace().collect();
    if fields.len() != 3 || fields[1] != "-" {
        return false;
    }
    if !fields[0].chars().all(|c| c.is_ascii_digit())
        || !fields[2].chars().all(|c| c.is_ascii_digit())
    {
        return false;
    }
    let (Ok(left), Ok(right)) = (fields[0].parse::<u64>(), fields[2].parse::<u64>()) else {
        return false;
    };
    left < right
}

fn exact_tool_expression(prompt: &str) -> Result<Option<(u64, ExactToolOperator, u64)>> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let lower = trimmed.to_ascii_lowercase();
    let mut rest = trimmed;
    let mut has_prefix = false;
    for prefix in ["what is", "compute", "calculate"] {
        if lower.starts_with(prefix) {
            rest = trimmed[prefix.len()..].trim_start();
            has_prefix = true;
            break;
        }
    }
    if !has_prefix && trimmed.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return Ok(None);
    }
    if exact_tool_is_phone_shape(rest) {
        return Ok(None);
    }
    if exact_tool_is_time_range(rest) {
        return Ok(None);
    }
    if exact_tool_is_iso_date_time(rest) || exact_tool_is_iso_date_range(rest) {
        return Ok(None);
    }
    if exact_tool_is_ratio_prose(rest) {
        return Ok(None);
    }
    if !has_prefix && exact_tool_is_number_leading_dash_prose(rest) {
        return Ok(None);
    }
    if rest.trim_end_matches(|c| matches!(c, '?' | '.' | '!' | ' ' | '\t')) == "+" {
        return Err(invalid(
            "exact arithmetic expects one decimal addition or subtraction",
        ));
    }
    if exact_tool_leading_operator_minus(rest) {
        return Err(invalid(
            "exact arithmetic does not accept an operator-only leading minus",
        ));
    }
    let expression_end = rest
        .char_indices()
        .find(|(_, c)| !matches!(c, '0'..='9' | '+' | '-' | ' ' | '\t'))
        .map_or(rest.len(), |(index, _)| index);
    let expression = rest[..expression_end].trim();
    let raw_suffix = rest[expression_end..].trim();
    if !raw_suffix.is_empty()
        && raw_suffix
            .chars()
            .all(|c| matches!(c, '?' | '.' | '!' | ',' | ' ' | '\t'))
        && exact_tool_numeric_range(expression, has_prefix)
    {
        return Ok(None);
    }
    let mut suffix = raw_suffix;
    if suffix.starts_with(|c| matches!(c, '?' | '.' | '!' | ',')) {
        suffix = suffix[1..].trim_start();
    }
    suffix = suffix.trim_end_matches(|c| matches!(c, '?' | '.' | '!' | ',' | ' '));
    let continued_arithmetic = suffix.chars().any(|c| {
        c.is_ascii_digit()
            || matches!(
                c,
                '+' | '-' | '*' | '^' | '&' | '|' | '=' | '<' | '>' | '~' | '−'
            )
    });
    if !suffix.is_empty()
        && !continued_arithmetic
        && exact_tool_numeric_range(expression, has_prefix)
    {
        return Ok(None);
    }
    let normalized_suffix = suffix.to_ascii_lowercase();
    let unsupported_suffix = !normalized_suffix.is_empty()
        && ![
            "answer with the number first",
            "answer with number first",
            "show your work",
            "show the carry trace",
            "please show your work",
            "please",
        ]
        .contains(&normalized_suffix.as_str());
    if prompt.len() > MAX_EXACT_TOOL_PROMPT_BYTES {
        let trailing_operator = expression.trim_end().ends_with(|c| matches!(c, '+' | '-'));
        if exact_tool_arithmetic_shaped(rest)
            || exact_tool_compact_subtraction_shape(expression)
            || (!unsupported_suffix && trailing_operator)
        {
            return Err(invalid(
                "exact arithmetic prompt exceeds the 160-byte tool bound",
            ));
        }
        return Ok(None);
    }
    if unsupported_suffix {
        if exact_tool_compact_range_prose(expression, suffix) {
            return Ok(None);
        }
        if exact_tool_arithmetic_shaped(rest) || exact_tool_compact_subtraction_shape(expression) {
            return Err(invalid(
                "exact arithmetic supports only one unsigned decimal + or - expression",
            ));
        }
        return Ok(None);
    }

    let expression_bytes = expression.as_bytes();
    let is_iso_date = expression_bytes.len() >= 10
        && expression_bytes[..4].iter().all(u8::is_ascii_digit)
        && expression_bytes[4] == b'-'
        && expression_bytes[5..7].iter().all(u8::is_ascii_digit)
        && expression_bytes[7] == b'-'
        && expression_bytes[8..10].iter().all(u8::is_ascii_digit)
        && (expression_bytes.len() == 10
            || (expression_bytes[10].is_ascii_whitespace()
                && expression_bytes[11..].iter().all(|byte| {
                    byte.is_ascii_digit() || *byte == b':' || byte.is_ascii_whitespace()
                })));
    if is_iso_date {
        return Ok(None);
    }
    if exact_tool_leading_operator_minus(expression) {
        return Err(invalid(
            "exact arithmetic does not accept an operator-only leading minus",
        ));
    }
    if (expression.starts_with('-') || expression.starts_with('+'))
        && expression[1..].chars().all(|c| c.is_ascii_digit())
    {
        return Ok(None);
    }
    if !expression.contains('+') && expression.contains('-') {
        let has_spaced_subtraction = expression.char_indices().any(|(index, c)| {
            if c != '-' {
                return false;
            }
            let previous = expression[..index].chars().next_back();
            let next = expression[index + 1..].chars().next();
            previous.is_some_and(char::is_whitespace)
                && (next.is_none()
                    || next.is_some_and(|next| next.is_ascii_digit())
                    || (next.is_some_and(char::is_whitespace)
                        && exact_tool_operand_start(&expression[index + 1..])))
        });
        if !has_spaced_subtraction && !exact_tool_compact_subtraction_shape(expression) {
            let digits_and_hyphens = expression.chars().all(|c| c.is_ascii_digit() || c == '-');
            if !digits_and_hyphens {
                return Ok(None);
            }
            if expression.ends_with('-') || expression.contains("--") {
                return Err(invalid(
                    "exact arithmetic expects one decimal addition or subtraction",
                ));
            }
            return Ok(None);
        }
    }
    let mut normalized = String::with_capacity(expression.len() + 4);
    let mut operator_count = 0usize;
    for c in expression.chars() {
        if matches!(c, '+' | '-') {
            operator_count += 1;
            normalized.push(' ');
            normalized.push(c);
            normalized.push(' ');
        } else {
            normalized.push(c);
        }
    }
    if operator_count == 0 {
        return Ok(None);
    }
    if operator_count > 1 {
        return Err(invalid(
            "exact arithmetic expects one decimal addition or subtraction",
        ));
    }
    let fields: Vec<&str> = normalized.split_whitespace().collect();
    if fields.len() != 3 && operator_count == 1 && expression.starts_with('+') {
        return Ok(None);
    }
    if fields.len() != 3 {
        return Err(invalid(
            "exact arithmetic expects one decimal addition or subtraction",
        ));
    }
    let parse_operand = |value: &str| -> Result<u64> {
        if value.len() > 10 {
            return Err(invalid(
                "exact arithmetic operand exceeds the 10-digit tool bound",
            ));
        }
        let parsed = value
            .parse::<u64>()
            .map_err(|_| invalid("exact arithmetic operand is not an unsigned decimal integer"))?;
        if parsed > MAX_EXACT_TOOL_OPERAND {
            return Err(invalid(
                "exact arithmetic operand exceeds the 10-digit tool bound",
            ));
        }
        Ok(parsed)
    };
    let left = parse_operand(fields[0])?;
    let right = parse_operand(fields[2])?;
    let operator = match fields[1] {
        "+" => ExactToolOperator::Add,
        "-" => ExactToolOperator::Subtract,
        _ => {
            return Err(invalid(
                "exact arithmetic supports only one decimal + or - operator",
            ))
        }
    };
    Ok(Some((left, operator, right)))
}

/// Answer one strict decimal expression through the existing geometric place-value engine.
pub fn exact_arithmetic_tool(prompt: &str) -> Option<Result<ExactArithmeticToolAnswer>> {
    let (left, operator, right) = match exact_tool_expression(prompt) {
        Ok(Some(parsed)) => parsed,
        Ok(None) => return None,
        Err(error) => return Some(Err(error)),
    };
    Some((|| {
        let left_atoms = u64_to_atoms(left, 10)?;
        let right_atoms = u64_to_atoms(right, 10)?;
        let engine = GeometricArithmeticEngine::new(10, 11)?;
        let trace = match operator {
            ExactToolOperator::Add => engine.add_atoms(&left_atoms, &right_atoms, None)?,
            ExactToolOperator::Subtract => engine.sub_atoms(&left_atoms, &right_atoms, None)?,
        };
        if !trace.verify_all_inverses() {
            return Err(invalid("exact arithmetic trace failed its inverse check"));
        }
        let result = trace.to_u64()?;
        let expression = format!(
            "{left} {} {right}",
            match operator {
                ExactToolOperator::Add => "+",
                ExactToolOperator::Subtract => "-",
            }
        );
        let mut text = format!(
            "{result}\n\nR4 geometric exact arithmetic — tool, not model generation\n- {expression} = {result}"
        );
        for step in &trace.steps {
            match operator {
                ExactToolOperator::Add => text.push_str(&format!(
                    "\n- p{}: {} + {} + {} -> {}, carry {}",
                    step.place,
                    step.left_digit,
                    step.right_digit,
                    step.carry_in_trit,
                    step.output_digit,
                    step.carry_out_trit
                )),
                ExactToolOperator::Subtract => text.push_str(&format!(
                    "\n- p{}: {} - {} - {} -> {}, borrow {}",
                    step.place,
                    step.left_digit,
                    step.right_digit,
                    -step.carry_in_trit,
                    step.output_digit,
                    -step.carry_out_trit
                )),
            }
        }
        Ok(ExactArithmeticToolAnswer {
            expression,
            result: result.to_string(),
            text,
        })
    })())
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

    #[test]
    fn exact_arithmetic_tool_answers_strict_decimal_prompts() {
        let answer = exact_arithmetic_tool("What is 17 + 25? Answer with the number first.")
            .expect("addition should be recognized")
            .expect("addition should be exact");
        assert_eq!(answer.expression, "17 + 25");
        assert_eq!(answer.result, "42");
        assert!(answer.text.starts_with("42\n"));
        assert!(answer.text.contains("tool, not model generation"));
        assert!(answer.text.contains("p0: 7 + 5 + 0 -> 2, carry 1"));
        assert!(answer.text.contains("p1: 1 + 2 + 1 -> 4, carry 0"));

        let carry = exact_arithmetic_tool("Calculate 483 + 259")
            .expect("second addition should be recognized")
            .expect("second addition should be exact");
        assert_eq!(carry.result, "742");

        let borrow = exact_arithmetic_tool("50 - 12")
            .expect("subtraction should be recognized")
            .expect("subtraction should be exact");
        assert_eq!(borrow.expression, "50 - 12");
        assert_eq!(borrow.result, "38");
        assert!(borrow.text.contains("p0: 0 - 2 - 0 -> 8, borrow 1"));
    }

    #[test]
    fn exact_arithmetic_tool_falls_through_and_refuses_invalid_input() {
        let no_space = exact_arithmetic_tool("17+25")
            .expect("no-space addition should be recognized")
            .expect("no-space addition should be exact");
        assert_eq!(no_space.expression, "17 + 25");
        assert_eq!(no_space.result, "42");
        assert!(exact_arithmetic_tool("42").is_none());
        assert!(exact_arithmetic_tool("+42").is_none());
        assert!(exact_arithmetic_tool("-42").is_none());
        assert!(exact_arithmetic_tool("11-05").is_none());
        assert!(exact_arithmetic_tool("2026-11").is_none());
        assert!(exact_arithmetic_tool("2026-11-05").is_none());
        assert!(exact_arithmetic_tool("2026-11-05 17:25").is_none());
        assert!(exact_arithmetic_tool("2026-11-05 17").is_none());
        assert!(exact_arithmetic_tool("What is 11-05?").is_none());
        assert!(exact_arithmetic_tool("What is 2026-11?").is_none());
        for prompt in ["50-12", "What is 50-12?"] {
            let compact = exact_arithmetic_tool(prompt)
                .expect("compact subtraction should be recognized")
                .expect("compact subtraction should be exact");
            assert_eq!(compact.result, "38", "wrong compact result: {prompt}");
        }
        let compact_underflow = exact_arithmetic_tool("What is 17-25?")
            .expect("compact underflow should be recognized")
            .expect_err("compact underflow should be an explicit error");
        assert!(compact_underflow.to_string().contains("underflow"));
        let compact_over_limit = format!("{}50-12", " ".repeat(200));
        let compact_over_limit_error = exact_arithmetic_tool(&compact_over_limit)
            .expect("overlong compact subtraction should be recognized")
            .expect_err("overlong compact subtraction should be an explicit error");
        assert!(compact_over_limit_error.to_string().contains("160-byte"));
        let no_space_subtraction = exact_arithmetic_tool("What is 50 - 12?")
            .expect("prefixed spaced subtraction should be recognized")
            .expect("prefixed spaced subtraction should be exact");
        assert_eq!(no_space_subtraction.result, "38");
        assert!(exact_arithmetic_tool("Explain in two sentences what you are.").is_none());
        assert!(exact_arithmetic_tool("What is the capital of Japan?").is_none());
        assert!(exact_arithmetic_tool("What is 17 plus 25?").is_none());
        assert!(exact_arithmetic_tool("What is 17 + ?")
            .expect("malformed arithmetic should be recognized")
            .is_err());
        assert!(exact_arithmetic_tool("1 - 2")
            .expect("underflow should be recognized")
            .is_err());
        assert!(exact_arithmetic_tool("What is 999999999999999999999 + 1?")
            .expect("oversized operand should be recognized")
            .is_err());
    }

    #[test]
    fn exact_arithmetic_tool_rejects_unsupported_and_overlong_arithmetic() {
        for prompt in [
            "What is 17 * 25?",
            "What is 17.5 + 25?",
            "What is (17 + 25)?",
            "What is 17 % 25?",
            "What is 17 ^ 25?",
            "What is ((17 + 25))?",
            "What is {17 + 25}?",
            "What is +17 * 25?",
            "What is -17 % 25?",
            "What is +17 ^ 25?",
            "What is +17 / 25?",
            "What is +17 × 25?",
            "What is +17 ÷ 25?",
            "What is .5 + 25?",
            "What is -17.5 + 25?",
            "What is (-17 * 25)?",
            "What is % 25?",
            "What is - 25?",
            "What is -?",
            "What is [17 + 25]?",
            "2026-11-05 + 1",
            "2026-11-05 - 1",
            "1 -",
            "What is 1 -?",
            "2026-11-05 -",
        ] {
            assert!(
                exact_arithmetic_tool(prompt)
                    .expect("unsupported arithmetic should be recognized")
                    .is_err(),
                "unsupported arithmetic fell through: {prompt}"
            );
        }

        let within_limit = format!("1{}+ 1", " ".repeat(156));
        assert_eq!(within_limit.len(), 160);
        assert_eq!(
            exact_arithmetic_tool(&within_limit)
                .expect("160-byte arithmetic should be recognized")
                .expect("160-byte arithmetic should be exact")
                .result,
            "2"
        );
        for spaces in [157, 159, 160] {
            let over_limit = format!("1{}+ 1", " ".repeat(spaces));
            assert!(
                exact_arithmetic_tool(&over_limit)
                    .expect("overlong arithmetic should be recognized")
                    .is_err(),
                "overlong arithmetic with operator after {spaces} spaces fell through"
            );
        }
        let prefixed_over_limit = format!("What is 1{}+ 1?", " ".repeat(151));
        assert_eq!(prefixed_over_limit.len(), 164);
        assert!(exact_arithmetic_tool(&prefixed_over_limit)
            .expect("prefixed overlong arithmetic should be recognized")
            .is_err());
        let signed_over_limit = format!("+1{}* 1", " ".repeat(159));
        assert!(exact_arithmetic_tool(&signed_over_limit)
            .expect("signed overlong arithmetic should be recognized")
            .is_err());
        let trailing_over_limit = format!("{}1 -", " ".repeat(200));
        let trailing_error = exact_arithmetic_tool(&trailing_over_limit)
            .expect("overlong incomplete subtraction should be recognized")
            .err()
            .expect("overlong incomplete subtraction should be an error");
        assert!(trailing_error.to_string().contains("160-byte"));
        for suffix in ["?", "Show your work"] {
            let prompt = format!("{}1 - {}", " ".repeat(200), suffix);
            let error = exact_arithmetic_tool(&prompt)
                .expect("overlong incomplete subtraction with suffix should be recognized")
                .err()
                .expect("overlong incomplete subtraction with suffix should be an error");
            assert!(error.to_string().contains("160-byte"), "{prompt}");
        }
        let date_over_limit = format!("{}2026-11-05 - ?", " ".repeat(200));
        let date_error = exact_arithmetic_tool(&date_over_limit)
            .expect("overlong date-prefixed incomplete subtraction should be recognized")
            .err()
            .expect("overlong date-prefixed incomplete subtraction should be an error");
        assert!(date_error.to_string().contains("160-byte"));
        let digit_over_limit = format!("{} - ?", "1".repeat(200));
        let digit_error = exact_arithmetic_tool(&digit_over_limit)
            .expect("overlong digits with subtraction suffix should be recognized")
            .err()
            .expect("overlong digits with subtraction suffix should be an error");
        assert!(digit_error.to_string().contains("160-byte"));
        for prompt in ["1 - ?", "1 - Show your work"] {
            assert!(
                exact_arithmetic_tool(prompt)
                    .expect("incomplete subtraction with suffix should be recognized")
                    .is_err(),
                "incomplete subtraction with suffix fell through: {prompt}"
            );
        }
        assert!(exact_arithmetic_tool(&"a".repeat(161)).is_none());
        for prompt in ["50 -12", "50- 12", "What is 50 -12?", "What is 50- 12?"] {
            let compact = exact_arithmetic_tool(prompt)
                .expect("one-sided-space subtraction should be recognized")
                .expect("one-sided-space subtraction should be exact");
            assert_eq!(
                compact.result, "38",
                "wrong one-sided-space result: {prompt}"
            );
        }
        assert!(exact_arithmetic_tool("What is +42?").is_none());
        assert!(exact_arithmetic_tool("What is -42?").is_none());
        for prompt in [
            "-42 is the answer",
            "+42 is the answer",
            "- 1. Introduction",
            "1. Introduction",
            "1) Introduction",
            "1 - Introduction",
            "2026 - Year in review",
            "2026-11-05 - Year in review",
            "What is +42, really?",
            "-42,",
            "2026-11-05T17:25+01:00,",
            "2026-11-05T17:25-01:00,",
            "2026-11-05T17:25+01:00, what day is it?",
            "2026-11-05T17:25-01:00, what day is it?",
            "What is 2026-11-05T17:25+01:00 in UTC?",
            "What is 2026-11-05T17:25-01:00 in UTC?",
            "What is 2026-11-05T17:25Z in UTC?",
            "2026-11-05T17:25+01:00, 5pm",
            "2026-11-05T17:25-01:00, 5pm",
            "2026-11-05T17:25Z, 5pm",
        ] {
            assert!(
                exact_arithmetic_tool(prompt).is_none(),
                "ordinary signed/date prose was intercepted: {prompt}"
            );
        }
        for prompt in [
            "2026-11-05T17:25Z",
            "2026-11-05T17:25+01:00",
            "2026-11-05T17:25-01:00",
            "2026-11-05 17:25+01:00",
            "2026-11-05T17:25Z?",
            "2026-11-05T17:25+01:00?",
            "2026-11-05T17:25-01:00?",
            "What is 2026-11-05T17:25+01:00?",
            "What is 2026-11-05T17:25Z?",
        ] {
            assert!(
                exact_arithmetic_tool(prompt).is_none(),
                "ISO date/time was intercepted: {prompt}"
            );
        }
        for prompt in [
            format!("{}1 + 1", " ".repeat(200)),
            format!("1 + 1{}", " ".repeat(200)),
        ] {
            assert!(exact_arithmetic_tool(&prompt)
                .expect("whitespace-padded overlong arithmetic should be recognized")
                .is_err());
        }
        for expression in ["-", "- 25", "- 2.5", "-?"] {
            for prompt in [
                format!("{}{}", " ".repeat(200), expression),
                format!("{}{}", expression, " ".repeat(200)),
            ] {
                let error = exact_arithmetic_tool(&prompt)
                    .expect("padded leading-minus arithmetic should be recognized")
                    .err()
                    .expect("padded leading-minus arithmetic should be an error");
                assert!(error.to_string().contains("leading minus"));
            }
        }
        for prompt in [
            "- this is ordinary prose".to_string(),
            format!("{}- this is ordinary prose", " ".repeat(200)),
            format!("- this is ordinary prose{}", " ".repeat(200)),
        ] {
            assert!(
                exact_arithmetic_tool(&prompt).is_none(),
                "dash prose was intercepted: {prompt}"
            );
        }
        let overflow = exact_arithmetic_tool("What is 999999999999999999999 + 1?")
            .expect("oversized operand should be recognized")
            .err()
            .expect("oversized operand should be an error");
        assert!(overflow.to_string().contains("10-digit tool bound"));
    }

    #[test]
    fn exact_arithmetic_tool_review18_boundary_regressions() {
        for prompt in [
            "2026-11-05  - 1",
            "2026-11-05, - 1",
            "2026-11-05T17:25+01:00, - 1",
            "2026-11-05  -",
            "2026-11-05 +",
            "What is 17 & 25?",
            "17 | 25",
            "17 << 2",
            "17 = 25",
            "17 (25)",
        ] {
            assert!(
                exact_arithmetic_tool(prompt)
                    .expect("review arithmetic boundary should be recognized")
                    .is_err(),
                "review arithmetic boundary fell through: {prompt}"
            );
        }
        for prompt in [
            "2026-11-05  - 1",
            "2026-11-05, - 1",
            "2026-11-05T17:25+01:00, - 1",
            "2026-11-05 +",
        ] {
            let overlong = format!("{}{}", " ".repeat(200), prompt);
            let error = exact_arithmetic_tool(&overlong)
                .expect("overlong review arithmetic boundary should be recognized")
                .err()
                .expect("overlong review arithmetic boundary should be an error");
            assert!(error.to_string().contains("160-byte"), "{overlong}");
        }
        for prompt in ["+1 800 555 0100", "+1 2 3", "10 - 20 minutes"] {
            assert!(
                exact_arithmetic_tool(prompt).is_none(),
                "ordinary signed/range prose was intercepted: {prompt}"
            );
        }
    }

    #[test]
    fn exact_arithmetic_tool_review20_boundary_regressions() {
        for prompt in ["~17", "17 ~ 25", "~17 + 25", "17 − 25"] {
            assert!(
                exact_arithmetic_tool(prompt)
                    .expect("unsupported review20 arithmetic should be recognized")
                    .is_err(),
                "unsupported review20 arithmetic fell through: {prompt}"
            );
        }
        for prompt in [
            "10 - 20 °C",
            "10 - 20 km/h",
            "10 - 20%",
            "10 - 20 (approx)",
            "10 - 20 min, then rest",
            "5pm - 6pm",
            "9am - 5pm",
            "5:00 - 6:00",
            "2026-11-05T17:25+01:00, 5pm - 6pm",
        ] {
            assert!(
                exact_arithmetic_tool(prompt).is_none(),
                "ordinary range/time prose was intercepted: {prompt}"
            );
        }
        for prompt in ["10 - 20 minutes + 3", "10 - 20 (25)"] {
            assert!(
                exact_arithmetic_tool(prompt)
                    .expect("continued review20 arithmetic should be recognized")
                    .is_err(),
                "continued review20 arithmetic fell through: {prompt}"
            );
        }
    }

    #[test]
    fn exact_arithmetic_tool_review23_boundary_regressions() {
        for prompt in [
            "17 ≤ 25",
            "17 ≥ 25",
            "17 ≠ 25",
            "17 · 25",
            "+",
            "+?",
            "+1-800-555-0100 + 2",
        ] {
            assert!(
                exact_arithmetic_tool(prompt)
                    .expect("review23 arithmetic boundary should be recognized")
                    .is_err(),
                "review23 arithmetic boundary fell through: {prompt}"
            );
        }
        for prompt in [
            "5 pm - 6 pm",
            "5:00 PM - 6:00 PM",
            "5:00:00 - 6:00:00",
            "2026-11-05T17:25+01:00, 5 pm - 6 pm",
            "2026-11-05T17:25+01:00, 5:00:00 - 6:00:00",
            "10 - 20?",
            "10 - 20.",
            "10 - 20°",
            "10 - 20$",
            "10 - 20€",
            "+1-800-555-0100",
            "+1 (800) 555-0100",
        ] {
            assert!(
                exact_arithmetic_tool(prompt).is_none(),
                "ordinary review23 prose was intercepted: {prompt}"
            );
        }
        let subtraction = exact_arithmetic_tool("50 - 12?")
            .expect("valid subtraction with sentence punctuation should be recognized")
            .expect("valid subtraction should be exact");
        assert_eq!(subtraction.result, "38");
    }

    #[test]
    fn exact_arithmetic_tool_review27_boundary_regressions() {
        for prompt in ["17-", "What is 17-?", "17--25", "2026-11-05 - 1 minute"] {
            assert!(
                exact_arithmetic_tool(prompt)
                    .expect("review27 arithmetic boundary should be recognized")
                    .is_err(),
                "review27 arithmetic boundary fell through: {prompt}"
            );
        }
        for prompt in [
            "24/7 support",
            "1/2 cup",
            "11-05-2026",
            "05-11-2026",
            "1-800-555-0100",
            "123-45-6789",
        ] {
            assert!(
                exact_arithmetic_tool(prompt).is_none(),
                "ordinary ratio/date/telephone prose was intercepted: {prompt}"
            );
        }
    }

    #[test]
    fn exact_arithmetic_tool_review29_prose_and_range_regressions() {
        for prompt in [
            "3/4 of the students passed",
            "5-10 minutes",
            "50-12 minutes",
            "2026-11-05 - 2026-11-10",
        ] {
            assert!(
                exact_arithmetic_tool(prompt).is_none(),
                "ordinary ratio/range prose was intercepted: {prompt}"
            );
        }
        for prompt in [
            "17/25 of 3",
            "5-10 minutes + 3",
            "2026-11-05 - 2026-11-10 + 3",
            "17/25 in decimal",
        ] {
            assert!(
                exact_arithmetic_tool(prompt)
                    .expect("review29 arithmetic boundary should be recognized")
                    .is_err(),
                "review29 arithmetic boundary fell through: {prompt}"
            );
        }
        for prompt in [
            "What is 10 -20?",
            "What is 12 -25?",
            "2026-11-05 -1",
            "2026-11-05, -1",
            "2026-11-05,-1",
            "2026-11-05,- 1",
            "2026-11-05 , -1",
            "2026-11-05T17:25Z,-1",
            "2026-11-05T17:25-01:00,-1",
            "2026-11-05T17:25+01:00, -1",
            "2026-11-05 +1",
        ] {
            assert!(
                exact_arithmetic_tool(prompt)
                    .expect("review15 arithmetic boundary should be recognized")
                    .is_err(),
                "review15 arithmetic boundary fell through: {prompt}"
            );
        }
        for (prompt, expected) in [("11 -05", "6"), ("2026 -11", "2015")] {
            let answer = exact_arithmetic_tool(prompt)
                .expect("one-sided date-like subtraction should be recognized")
                .expect("one-sided date-like subtraction should compute exactly");
            assert_eq!(answer.result, expected, "wrong result: {prompt}");
        }
        for prompt in [
            "12:00 - 1, then lunch",
            "5:00 - 6",
            "12 - 1:00",
            "5 - 6:00",
            "3/4 of the 20 students passed",
            "5-10 minutes, 3 times",
            "5-10 km",
            "5-10 cm",
            "5-10 min",
            "5-10 students",
            "5-10 dollars",
            "2026-11-05 - 2026-11-10, 3 days",
        ] {
            assert!(
                exact_arithmetic_tool(prompt).is_none(),
                "ordinary count/clock prose was intercepted: {prompt}"
            );
        }
        let compact = exact_arithmetic_tool("50-12")
            .expect("compact subtraction should still be recognized")
            .expect("compact subtraction should still compute exactly");
        assert_eq!(compact.result, "38");
        assert!(exact_arithmetic_tool("17-25")
            .expect("compact underflow should still be recognized")
            .is_err());
    }

    #[test]
    fn exact_arithmetic_tool_final_review_range_and_punctuation_regressions() {
        let answer = |prompt: &str| {
            exact_arithmetic_tool(prompt)
                .unwrap_or_else(|| panic!("tool did not recognize {prompt}"))
                .unwrap_or_else(|error| panic!("tool errored for {prompt}: {error}"))
                .result
        };
        assert_eq!(answer("What is 10 - 2?"), "8");
        assert_eq!(answer("What is 100 - 20?"), "80");
        assert_eq!(answer("What is 50 - 12,"), "38");
        assert_eq!(answer("What is 50 - 12 please"), "38");
        assert_eq!(
            answer("What is 50 - 12? Answer with the number first."),
            "38"
        );
        for prompt in [
            "What is 17 - 25?",
            "What is 17 - 25,",
            "What is 10 - 20?",
            "What is 50 - 12 + 1",
            "10 - 20 minutes + 3",
        ] {
            assert!(
                exact_arithmetic_tool(prompt)
                    .expect("final-review arithmetic boundary should be recognized")
                    .is_err(),
                "final-review arithmetic boundary fell through: {prompt}"
            );
        }
        for prompt in [
            "10 - 20?",
            "10 - 20 minutes",
            "5 pm - 6 pm?",
            "9am - 5pm?",
            "5:00 PM - 6:00 PM?",
            "+1-800-555-0100?",
        ] {
            assert!(
                exact_arithmetic_tool(prompt).is_none(),
                "ordinary final-review prose was intercepted: {prompt}"
            );
        }
    }

    #[test]
    fn exact_arithmetic_tool_final_review_time_and_telephone_punctuation() {
        for prompt in [
            "5 pm - 6 pm?",
            "9am - 5pm?",
            "5:00 PM - 6:00 PM?",
            "+1-800-555-0100?",
        ] {
            assert!(
                exact_arithmetic_tool(prompt).is_none(),
                "punctuated time/telephone prose was intercepted: {prompt}"
            );
        }
    }

    #[test]
    fn exact_arithmetic_tool_final_review_false_answer_regressions() {
        for prompt in [
            "What is 20 + 5 %",
            "What is 20 + 5 / two",
            "What is 20 + 5 (two)",
            "What is 17 + 25 in binary",
            "What is 20 + 10, then double it",
            "What is 20 - 10 in binary",
        ] {
            assert!(
                exact_arithmetic_tool(prompt)
                    .expect("false-answer arithmetic boundary should be recognized")
                    .is_err(),
                "false-answer arithmetic request returned an exact answer: {prompt}"
            );
        }
    }

    #[test]
    fn exact_arithmetic_tool_final_review_supported_suffix_range_regressions() {
        for prompt in [
            "10 - 20 please",
            "10 - 20, please",
            "10 - 20, show your work",
            "10 - 20, answer with the number first",
            "10 - 20, please show your work",
        ] {
            assert!(
                exact_arithmetic_tool(prompt).is_none(),
                "no-prefix range with a supported suffix was intercepted: {prompt}"
            );
        }
    }

    #[test]
    fn exact_arithmetic_tool_final_review_time_range_prose_regressions() {
        for prompt in ["5 pm - 6 pm, then dinner", "2026-11-05T17:25 - 18:00"] {
            assert!(
                exact_arithmetic_tool(prompt).is_none(),
                "ordinary time-range prose was intercepted: {prompt}"
            );
        }
        for prompt in ["5 pm - 6 pm + 3", "2026-11-05T17:25 - 1"] {
            assert!(
                exact_arithmetic_tool(prompt)
                    .expect("time-range arithmetic boundary should be recognized")
                    .is_err(),
                "time-range arithmetic boundary fell through: {prompt}"
            );
        }
    }

    #[test]
    fn exact_arithmetic_tool_final_review_prose_boundary_regressions() {
        let answer = |prompt: &str| {
            exact_arithmetic_tool(prompt)
                .unwrap_or_else(|| panic!("tool did not recognize {prompt}"))
                .unwrap_or_else(|error| panic!("tool errored for {prompt}: {error}"))
                .result
        };
        let intercepted: Vec<&str> = [
            "5 pm - 6, then dinner",
            "12 - 1 pm, then lunch",
            "5 people attended - 3 left",
            "1 (800) 555-0100",
            "(800) 555-0100",
            "1-800-555-0100",
            "+1-800-555-0100",
            "+1 800 555 0100",
        ]
        .into_iter()
        .filter(|prompt| exact_arithmetic_tool(prompt).is_some())
        .collect();
        assert!(
            intercepted.is_empty(),
            "ordinary prose was intercepted: {intercepted:?}"
        );
        assert_eq!(answer("2026 - 11"), "2015");
        assert_eq!(answer("What is 50 - 12 please"), "38");
        for prompt in [
            "2026 - 11 - 05",
            "5 pm - 6 pm + 3",
            "2026-11-05T17:25 - 1",
            "1 (800) 555-0100 + 2",
        ] {
            assert!(
                exact_arithmetic_tool(prompt)
                    .expect("prose-boundary arithmetic control should be recognized")
                    .is_err(),
                "prose-boundary arithmetic control fell through: {prompt}"
            );
        }
        let missing_errors: Vec<&str> = [
            "What is (1234567-8)?",
            "(1234567-8)",
            "1 (800) 555-0100 -2",
            "(800) 555-0100 -2",
            "5 people attended - 3 left + 2",
        ]
        .into_iter()
        .filter(|prompt| match exact_arithmetic_tool(prompt) {
            Some(Err(error)) => error.to_string().is_empty(),
            _ => true,
        })
        .collect();
        assert!(
            missing_errors.is_empty(),
            "grouped or continued arithmetic must be explicit non-empty errors: {missing_errors:?}"
        );
        assert!(
            exact_arithmetic_tool("5 - 3 left").is_some(),
            "5 - 3 left must remain tool-handled rather than prose"
        );
    }

    #[test]
    fn exact_arithmetic_tool_final_review_ratio_instruction_regressions() {
        for prompt in ["24/7 support", "1/2 cup", "1/2 cup sugar"] {
            assert!(
                exact_arithmetic_tool(prompt).is_none(),
                "ordinary ratio prose was intercepted: {prompt}"
            );
        }
        let intercepted_units: Vec<&str> = ["1/2 gallon", "1/2 gram", "1/2 liter", "1/2 dozen"]
            .into_iter()
            .filter(|prompt| exact_arithmetic_tool(prompt).is_some())
            .collect();
        assert!(
            intercepted_units.is_empty(),
            "ordinary unit ratio prose was intercepted: {intercepted_units:?}"
        );
        let mut failures: Vec<String> = Vec::new();
        for prompt in [
            "17/25 show your work",
            "17/25 please",
            "17/25 answer with the number first",
            "17/25 in binary",
            "17/25 in hex",
            "17/25 in decimal",
            "17/25 as a decimal",
            "17/25 to decimal",
            "17/25 percent",
            "17/25 as a percentage",
            "17/25 simplified",
            "1/2 cup + 3",
        ] {
            match exact_arithmetic_tool(prompt) {
                Some(Err(error)) if !error.to_string().is_empty() => {}
                _ => failures.push(prompt.to_string()),
            }
        }
        let padded = format!("{}17/25 show your work", " ".repeat(200));
        match exact_arithmetic_tool(&padded) {
            Some(Err(error)) if error.to_string().contains("160-byte") => {}
            _ => failures.push("over-160-byte instruction-suffixed division".to_string()),
        }
        let padded_in_decimal = format!("{}17/25 in decimal", " ".repeat(200));
        match exact_arithmetic_tool(&padded_in_decimal) {
            Some(Err(error)) if error.to_string().contains("160-byte") => {}
            _ => failures.push("over-160-byte in-decimal division".to_string()),
        }
        assert!(
            failures.is_empty(),
            "instruction-suffixed division must be an explicit error: {failures:?}"
        );
    }
}
