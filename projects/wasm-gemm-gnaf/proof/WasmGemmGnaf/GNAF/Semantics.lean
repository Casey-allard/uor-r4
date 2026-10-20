import WasmGemmGnaf.GNAF.Plan
set_option autoImplicit false

/-!
# GNAF: plan behaviour, independently of compilation (SPEC §11.2)

SPEC §11.2 requires plan behaviour to be defined *independently of
compilation*.  Nothing in this file mentions a module, an encoder, a decoder or
an emitter: a plan denotes a total function on machine configurations, in the
sense of authority §5.3 (a configuration is typed memory, resource state and
control state) and authority §5.4 (plans operate on configurations, not
isolated scalars: multiple inputs and outputs, destructive updates, prepared
state).

`Plan.eval` is that denotation and `Eval` is its SPEC-facing name.  It is a
*total structurally recursive function*, so the semantics is deterministic and
total by construction; `GNAF/Resource.lean` supplies the explicit well-founded
measure and the resource bound.

The raw-input classifier `Machine.classify` is likewise a total function of the
configuration alone, following the SPEC §8.3 precedence order (header, then
kind/mode tags, then compatibility, then the resource predicate).
-/

namespace WasmGemmGnaf.GNAF

open WasmGemmGnaf.Foundation

/-! ## Machine configurations -/

/-- A machine configuration (authority §5.3): typed memory, private scratch,
the scalar and vector register files, the code/data tables, the status word and
the constructed output. -/
structure Machine where
  mem : List Nat
  scratch : List Nat
  regs : List Nat
  vregs : List (List Nat)
  tables : List (List Nat)
  status : Nat
  out : List Nat
  deriving DecidableEq, Repr, Inhabited

namespace Machine

/-- The value of a scalar register (unset registers read as zero). -/
def reg (m : Machine) (i : Nat) : Nat := m.regs.getD i 0

/-- Writing a scalar register. -/
def withReg (m : Machine) (i v : Nat) : Machine := { m with regs := m.regs.set i v }

/-- The value of a vector register. -/
def vreg (m : Machine) (i : Nat) : List Nat := m.vregs.getD i []

/-- Writing a vector register. -/
def withVreg (m : Machine) (i : Nat) (v : List Nat) : Machine :=
  { m with vregs := m.vregs.set i v }

/-- A byte of the raw invocation. -/
def byteAt (m : Machine) (i : Nat) : Nat := m.mem.getD i 0

/-- A little-endian 16-bit field of the raw invocation (SPEC §8.3). -/
def u16At (m : Machine) (i : Nat) : Nat := m.byteAt i + 256 * m.byteAt (i + 1)

@[simp] theorem reg_withReg_same (m : Machine) (i v : Nat) (h : i < m.regs.length) :
    (m.withReg i v).reg i = v := by
  simp [reg, withReg, List.getD_eq_getElem?_getD, h]

@[simp] theorem regs_length_withReg (m : Machine) (i v : Nat) :
    (m.withReg i v).regs.length = m.regs.length := by
  simp [withReg]

@[simp] theorem vregs_length_withVreg (m : Machine) (i : Nat) (v : List Nat) :
    (m.withVreg i v).vregs.length = m.vregs.length := by
  simp [withVreg]

@[simp] theorem mem_withReg (m : Machine) (i v : Nat) : (m.withReg i v).mem = m.mem := rfl

@[simp] theorem status_withReg (m : Machine) (i v : Nat) :
    (m.withReg i v).status = m.status := rfl

@[simp] theorem scratch_withReg (m : Machine) (i v : Nat) :
    (m.withReg i v).scratch = m.scratch := rfl

end Machine

/-! ## Raw-input classification (SPEC §8.3) -/

/-- Decoding a stored-kind tag (SPEC §8.3 tag table). -/
def kindOfTag : Nat → Option ScalarKind
  | 0 => some .i8 | 1 => some .u8 | 2 => some .i16 | 3 => some .u16
  | 4 => some .i32 | 5 => some .u32 | 6 => some .i64 | 7 => some .u64
  | 8 => some .binary16 | 9 => some .bfloat16 | 10 => some .binary32
  | 11 => some .binary64 | 12 => some .exactDyadic
  | _ => none

/-- Decoding an arithmetic-mode tag (SPEC §8.3). -/
def modeOfTag : Nat → Option ArithmeticMode
  | 0 => some .modular | 1 => some .checked | 2 => some .strictFloat
  | 3 => some .exactDyadicRoundOnce
  | _ => none

theorem kindOfTag_tag (k : ScalarKind) : kindOfTag k.tag = some k := by
  cases k <;> rfl

theorem modeOfTag_tag (m : ArithmeticMode) : modeOfTag m.tag = some m := by
  cases m <;> rfl

/-- The released ABI header size (SPEC §8.3). -/
def headerSize : Nat := 256

/-- The released raw-invocation extent limit (SPEC §8.2 resource constants). -/
def maxRawExtent : Nat := 4294967295

namespace Machine

/-- The exact ABI magic `WGNG` is present (SPEC §8.3). -/
def magicOk (m : Machine) : Bool :=
  m.byteAt 0 == 87 && m.byteAt 1 == 71 && m.byteAt 2 == 78 && m.byteAt 3 == 71

/-- The version and header-size fields are exactly the released literals. -/
def headerOk (m : Machine) : Bool :=
  m.magicOk && m.u16At 4 == 1 && m.u16At 6 == headerSize

/-- The declared stored kind, accumulator kind and arithmetic mode, when all
three tags are known (SPEC §8.3). -/
def declaredTags (m : Machine) : Option (ScalarKind × ScalarKind × ArithmeticMode) :=
  match kindOfTag (m.byteAt 8), kindOfTag (m.byteAt 11), modeOfTag (m.byteAt 12) with
  | some stored, some acc, some mode => some (stored, acc, mode)
  | _, _, _ => none

/-- The total raw-input classifier of SPEC §8.3, in the fixed precedence order:
truncation or malformed header first, then unknown kind/mode tags, then
incompatible tags, then the resource predicate. -/
def classify (m : Machine) : Classification :=
  if m.mem.length < headerSize then .invalid
  else if ¬ m.headerOk then .invalid
  else
    match m.declaredTags with
    | none => .unsupported
    | some (stored, acc, mode) =>
        if compatible mode stored acc then
          if m.mem.length ≤ maxRawExtent then .valid else .resourceExhausted
        else .invalid

/-- Classification is total: it is a function, so every raw configuration has
exactly one class. -/
theorem classify_total (m : Machine) : ∃ c, m.classify = c := ⟨_, rfl⟩

/-- A truncated invocation is `invalid`. -/
theorem classify_of_short {m : Machine} (h : m.mem.length < headerSize) :
    m.classify = .invalid := by
  simp [classify, h]

/-- A `valid` classification forces the exact released header. -/
theorem headerOk_of_classify_valid {m : Machine} (h : m.classify = .valid) :
    m.headerOk = true := by
  unfold classify at h
  by_cases h1 : m.mem.length < headerSize
  · rw [if_pos h1] at h; exact absurd h (by decide)
  · rw [if_neg h1] at h
    by_cases h2 : m.headerOk = true
    · exact h2
    · rw [if_pos (by simpa using h2)] at h; exact absurd h (by decide)

/-- A `valid` classification forces a compatible kind/mode triple: the plan
language cannot enter its valid continuation on an incompatible descriptor. -/
theorem compatible_of_classify_valid {m : Machine} (h : m.classify = .valid) :
    ∃ stored acc mode,
      m.declaredTags = some (stored, acc, mode) ∧ compatible mode stored acc = true := by
  unfold classify at h
  by_cases h1 : m.mem.length < headerSize
  · rw [if_pos h1] at h; exact absurd h (by decide)
  rw [if_neg h1] at h
  by_cases h2 : ¬ m.headerOk = true
  · rw [if_pos h2] at h; exact absurd h (by decide)
  rw [if_neg h2] at h
  cases hk : m.declaredTags with
  | none => simp only [hk] at h; exact absurd h (by simp)
  | some t =>
    obtain ⟨stored, acc, mode⟩ := t
    simp only [hk] at h
    by_cases hc : compatible mode stored acc = true
    · exact ⟨stored, acc, mode, by first | exact hk | rfl, hc⟩
    · rw [if_neg hc] at h; exact absurd h (by simp)

/-- The stored element width declared by the header. -/
def storedWidth (m : Machine) : Nat :=
  match kindOfTag (m.byteAt 8) with
  | some k => k.byteWidth
  | none => 0

/-- The total layout-class dispatch of SPEC §11.1: a function of the declared A
view, never inferred from an address. -/
def layoutClass (m : Machine) : LayoutClass :=
  if m.u16At 64 == m.storedWidth then .rowMajor
  else if m.u16At 56 == m.storedWidth then .colMajor
  else .general

theorem layoutClass_rowMajor {m : Machine} (h : m.u16At 64 = m.storedWidth) :
    m.layoutClass = .rowMajor := by
  simp [layoutClass, h]

end Machine

/-! ## Condition evaluation -/

namespace Cond

/-- The exact meaning of a branch condition on a configuration. -/
def eval (c : Cond) (m : Machine) : Bool :=
  match c with
  | statusIs s => m.status == s.code
  | regEq r v => m.reg r == v
  | regLt r v => decide (m.reg r < v)
  | scratchAtLeast n => decide (n ≤ m.scratch.length)

@[simp] theorem eval_statusIs (s : Status) (m : Machine) :
    (statusIs s).eval m = (m.status == s.code) := rfl

end Cond

/-! ## Primitive store transformers -/

/-- Copy `n` elements into `dst` starting at `dstBase`, reading `src` through
the address function `f`.  This is the exact meaning of `pack` and `unpack`. -/
def gather (dst : List Nat) (dstBase : Nat) (src : List Nat) (f : Nat → Nat) :
    Nat → Nat → List Nat
  | 0, _ => dst
  | n + 1, i => gather (dst.set (dstBase + i) (src.getD (f i) 0)) dstBase src f n (i + 1)

@[simp] theorem gather_zero (dst : List Nat) (dstBase : Nat) (src : List Nat)
    (f : Nat → Nat) (i : Nat) : gather dst dstBase src f 0 i = dst := rfl

/-- Copying never changes the size of the destination store. -/
theorem gather_length (src : List Nat) (f : Nat → Nat) :
    ∀ (n : Nat) (dst : List Nat) (dstBase i : Nat),
      (gather dst dstBase src f n i).length = dst.length := by
  intro n
  induction n with
  | zero => intro dst dstBase i; rfl
  | succ n ih =>
    intro dst dstBase i
    rw [gather, ih]
    simp

/-- Copying leaves every destination cell outside the written range alone.  In
particular an out-of-range address writes nothing at all: `List.set` is the
identity past the end of the list. -/
theorem gather_getD_outside (src : List Nat) (f : Nat → Nat) :
    ∀ (n : Nat) (dst : List Nat) (dstBase i x : Nat),
      (x < dstBase + i ∨ dstBase + i + n ≤ x) →
      (gather dst dstBase src f n i).getD x 0 = dst.getD x 0 := by
  intro n
  induction n with
  | zero => intro dst dstBase i x _; rfl
  | succ n ih =>
    intro dst dstBase i x hx
    rw [gather, ih _ _ _ _ (by omega)]
    have hne : dstBase + i ≠ x := by omega
    simp [List.getD_eq_getElem?_getD, List.getElem?_set_ne hne]

/-- Copying really writes: inside the written range the destination cell holds
the source value the address function selected. -/
theorem gather_getD_inside (src : List Nat) (f : Nat → Nat) :
    ∀ (n : Nat) (dst : List Nat) (dstBase i x : Nat),
      dstBase + i ≤ x → x < dstBase + i + n → dstBase + i + n ≤ dst.length →
      (gather dst dstBase src f n i).getD x 0 = src.getD (f (x - dstBase)) 0 := by
  intro n
  induction n with
  | zero => intro dst dstBase i x h1 h2 _; omega
  | succ n ih =>
    intro dst dstBase i x h1 h2 h3
    rw [gather]
    by_cases hx : x = dstBase + i
    · subst hx
      rw [gather_getD_outside _ _ _ _ _ _ _ (Or.inl (by omega))]
      have hi : dstBase + i - dstBase = i := by omega
      have hlt : dstBase + i < dst.length := by omega
      rw [hi]
      simp [List.getD_eq_getElem?_getD, hlt]
    · refine ih (dst.set (dstBase + i) (src.getD (f i) 0)) dstBase (i + 1) x
        (by omega) (by omega) ?_
      simpa using (by omega : dstBase + (i + 1) + n ≤ dst.length)

/-! ### Little-endian machine words

`Machine.u16At` reads two cells little-endian.  `leBytes` and `leWord` are the
same convention at an arbitrary width: they are the byte image a register store
deposits and the value a load of that image reads back. -/

/-- The little-endian byte image of a value, least significant byte first. -/
def leBytes (v : Nat) : Nat → List Nat
  | 0 => []
  | n + 1 => v % 256 :: leBytes (v / 256) n

@[simp] theorem leBytes_zero (v : Nat) : leBytes v 0 = [] := rfl

theorem leBytes_succ (v n : Nat) : leBytes v (n + 1) = v % 256 :: leBytes (v / 256) n := rfl

theorem leBytes_length : ∀ (n v : Nat), (leBytes v n).length = n := by
  intro n
  induction n with
  | zero => intro v; rfl
  | succ n ih => intro v; simp [leBytes_succ, ih]

/-- The little-endian value of `n` cells of a store, starting at `addr`. -/
def leWord (l : List Nat) (addr : Nat) : Nat → Nat
  | 0 => 0
  | n + 1 => l.getD addr 0 + 256 * leWord l (addr + 1) n

@[simp] theorem leWord_zero (l : List Nat) (addr : Nat) : leWord l addr 0 = 0 := rfl

theorem leWord_succ (l : List Nat) (addr n : Nat) :
    leWord l addr (n + 1) = l.getD addr 0 + 256 * leWord l (addr + 1) n := rfl

/-- A little-endian read depends only on the cells it reads. -/
theorem leWord_ext : ∀ (n : Nat) (l l' : List Nat) (a a' : Nat),
    (∀ k, k < n → l.getD (a + k) 0 = l'.getD (a' + k) 0) → leWord l a n = leWord l' a' n := by
  intro n
  induction n with
  | zero => intro l l' a a' _; rfl
  | succ n ih =>
    intro l l' a a' h
    have h0 := h 0 (by omega)
    rw [Nat.add_zero, Nat.add_zero] at h0
    rw [leWord_succ, leWord_succ, h0]
    refine congrArg (fun z => l'.getD a' 0 + 256 * z) (ih l l' (a + 1) (a' + 1) ?_)
    intro k hk
    have hs := h (k + 1) (by omega)
    have e1 : a + (k + 1) = a + 1 + k := by omega
    have e2 : a' + (k + 1) = a' + 1 + k := by omega
    rwa [e1, e2] at hs

theorem leWord_cons : ∀ (n : Nat) (x : Nat) (l : List Nat) (a : Nat),
    leWord (x :: l) (a + 1) n = leWord l a n := by
  intro n
  induction n with
  | zero => intro x l a; rfl
  | succ n ih =>
    intro x l a
    rw [leWord_succ, leWord_succ, ih x l (a + 1)]
    simp [List.getD_eq_getElem?_getD]

/-- The arithmetic of one little-endian digit. -/
theorem mod_pow_succ (v n : Nat) :
    v % 256 ^ (n + 1) = v % 256 + 256 * (v / 256 % 256 ^ n) := by
  have hpow : (256 : Nat) ^ (n + 1) = 256 * 256 ^ n := by
    rw [Nat.pow_succ, Nat.mul_comm]
  have e1 : 256 * (v / 256) + v % 256 = v := Nat.div_add_mod v 256
  have e2 : 256 ^ n * (v / 256 / 256 ^ n) + v / 256 % 256 ^ n = v / 256 :=
    Nat.div_add_mod (v / 256) (256 ^ n)
  have hlt : v % 256 < 256 := Nat.mod_lt _ (by omega)
  have hM : 0 < 256 ^ n := Nat.pow_pos (by omega)
  have hB : v / 256 % 256 ^ n < 256 ^ n := Nat.mod_lt _ hM
  have hB' : 256 * (v / 256 % 256 ^ n + 1) ≤ 256 * 256 ^ n :=
    Nat.mul_le_mul_left 256 hB
  have hsmall : 256 * (v / 256 % 256 ^ n) + v % 256 < 256 * 256 ^ n := by
    rw [Nat.mul_add] at hB'
    omega
  have key : (256 * 256 ^ n) * (v / 256 / 256 ^ n) +
      (256 * (v / 256 % 256 ^ n) + v % 256) = v := by
    have h1 : (256 * 256 ^ n) * (v / 256 / 256 ^ n) +
        (256 * (v / 256 % 256 ^ n) + v % 256) =
        256 * (256 ^ n * (v / 256 / 256 ^ n) + v / 256 % 256 ^ n) + v % 256 := by
      rw [Nat.mul_add, Nat.mul_assoc]
      exact (Nat.add_assoc _ _ _).symm
    rw [h1, e2, e1]
  calc v % 256 ^ (n + 1)
      = v % (256 * 256 ^ n) := by rw [hpow]
    _ = ((256 * 256 ^ n) * (v / 256 / 256 ^ n) +
          (256 * (v / 256 % 256 ^ n) + v % 256)) % (256 * 256 ^ n) := by rw [key]
    _ = (256 * (v / 256 % 256 ^ n) + v % 256) % (256 * 256 ^ n) := Nat.mul_add_mod _ _ _
    _ = 256 * (v / 256 % 256 ^ n) + v % 256 := Nat.mod_eq_of_lt hsmall
    _ = v % 256 + 256 * (v / 256 % 256 ^ n) := by omega

/-- Reading back the little-endian image of a value gives the value, truncated
to the stored width. -/
theorem leWord_leBytes : ∀ (n v : Nat), leWord (leBytes v n) 0 n = v % 256 ^ n := by
  intro n
  induction n with
  | zero => intro v; rw [Nat.pow_zero, Nat.mod_one]; rfl
  | succ n ih =>
    intro v
    rw [leBytes_succ, leWord_succ]
    have hc : leWord (v % 256 :: leBytes (v / 256) n) (0 + 1) n =
        leWord (leBytes (v / 256) n) 0 n := leWord_cons n _ _ 0
    rw [hc, ih (v / 256), mod_pow_succ]
    simp [List.getD_eq_getElem?_getD]

/-- The exact dot-product reduction of an arithmetic contract (SPEC §8.2).  The
second component records a checked-mode overflow, in which case the accumulator
is left unchanged. -/
def reduceFold (c : ArithmeticContract) (src : List Nat) :
    Nat → Nat → Nat → Nat → Nat × Bool
  | 0, _, _, acc => (acc, false)
  | n + 1, ba, bb, acc =>
      let a := src.getD ba 0
      let b := src.getD bb 0
      if c.overflows acc a b then (acc, true)
      else reduceFold c src n (ba + 1) (bb + 1) (c.step acc a b)

@[simp] theorem reduceFold_zero (c : ArithmeticContract) (src : List Nat)
    (ba bb acc : Nat) : reduceFold c src 0 ba bb acc = (acc, false) := rfl

/-- Outside checked mode no reduction ever reports an overflow: the overflow
result is a property of the *declared contract*, not of the schedule. -/
theorem reduceFold_no_overflow (c : ArithmeticContract) (h : c.mode ≠ .checked)
    (src : List Nat) :
    ∀ (n ba bb acc : Nat), (reduceFold c src n ba bb acc).2 = false := by
  intro n
  induction n with
  | zero => intro ba bb acc; rfl
  | succ n ih =>
    intro ba bb acc
    rw [reduceFold]
    have hov : c.overflows acc (src.getD ba 0) (src.getD bb 0) = false := by
      unfold ArithmeticContract.overflows
      cases hm : c.mode <;> simp_all
    simp only [hov, if_false, Bool.false_eq_true]
    exact ih _ _ _

/-- A modular reduction keeps the accumulator inside the accumulator's pattern
space at every step. -/
theorem reduceFold_modular_lt (c : ArithmeticContract) (h : c.mode = .modular)
    (src : List Nat) :
    ∀ (n ba bb acc : Nat), 0 < n →
      (reduceFold c src n ba bb acc).1 < c.accModulus := by
  intro n
  induction n with
  | zero => intro _ _ _ hn; exact absurd hn (by omega)
  | succ n ih =>
    intro ba bb acc _
    rw [reduceFold]
    have hov : c.overflows acc (src.getD ba 0) (src.getD bb 0) = false := by
      unfold ArithmeticContract.overflows
      rw [h]
    simp only [hov, if_false, Bool.false_eq_true]
    cases n with
    | zero =>
      simp only [reduceFold]
      exact c.step_modular_lt h _ _ _
    | succ k => exact ih _ _ _ (by omega)

/-- The scratch store produced by a `pack` node. -/
def packedScratch (m : Machine) (src dst : RegionRef) (map : IndexMap) : List Nat :=
  gather m.scratch dst.base m.mem (fun i => src.base + map.apply i 0 0) src.count 0

/-- The memory store produced by an `unpack` node. -/
def unpackedMem (m : Machine) (src dst : RegionRef) (map : IndexMap) : List Nat :=
  gather m.mem dst.base m.scratch (fun i => src.base + map.apply i 0 0) src.count 0

/-- The address a `storeReg` node writes.  It is the destination region's base
offset by the index map evaluated at the machine's loop indices, which the
evaluator publishes in the first three scalar registers: `Plan.eval` of `tiled`
writes the tile index into register `0`, and a GEMM loop nest carries its two
traversal indices in registers `1` and `2` (the `b`, `i`, `j` slots of
`IndexMap.apply`).  This is the same `base + map` addressing `unpack` uses, with
the loop indices in place of `unpack`'s copy counter. -/
def storeAddr (m : Machine) (dst : RegionRef) (map : IndexMap) : Nat :=
  dst.base + map.apply (m.reg 0) (m.reg 1) (m.reg 2)

/-- The memory store produced by a `storeReg` node: the little-endian byte image
of the register, deposited by exactly the primitive `unpack` uses.  An address
outside the store therefore behaves exactly as an out-of-range `unpack` does —
`gather` is `List.set`, which is the identity past the end of the list — and no
new failure behaviour is introduced. -/
def storedRegMem (m : Machine) (dst : RegionRef) (map : IndexMap) (width src : Nat) :
    List Nat :=
  gather m.mem (storeAddr m dst map) (leBytes (m.reg src) width) (fun i => i) width 0

/-- The value a `loadReg` node reads: the little-endian word of `width` cells at
exactly the address `storeReg` writes.  The two nodes therefore agree on
addressing by construction, not by coincidence. -/
def loadedReg (m : Machine) (src : RegionRef) (map : IndexMap) (width : Nat) : Nat :=
  leWord m.mem (storeAddr m src map) width

@[simp] theorem storedRegMem_length (m : Machine) (dst : RegionRef) (map : IndexMap)
    (width src : Nat) : (storedRegMem m dst map width src).length = m.mem.length :=
  gather_length _ _ _ _ _ _

@[simp] theorem packedScratch_length (m : Machine) (src dst : RegionRef) (map : IndexMap) :
    (packedScratch m src dst map).length = m.scratch.length :=
  gather_length _ _ _ _ _ _

@[simp] theorem unpackedMem_length (m : Machine) (src dst : RegionRef) (map : IndexMap) :
    (unpackedMem m src dst map).length = m.mem.length :=
  gather_length _ _ _ _ _ _

/-! ## Evaluation -/

namespace Plan

/-- Run a loop body `f` for `n` iterations, publishing the iteration index in
register `r`. -/
def runLoop (f : Machine → Machine) (r : Nat) : Nat → Nat → Machine → Machine
  | 0, _, m => m
  | n + 1, i, m => runLoop f r n (i + 1) (f (m.withReg r i))

@[simp] theorem runLoop_zero (f : Machine → Machine) (r i : Nat) (m : Machine) :
    runLoop f r 0 i m = m := rfl

theorem runLoop_succ (f : Machine → Machine) (r n i : Nat) (m : Machine) :
    runLoop f r (n + 1) i m = runLoop f r n (i + 1) (f (m.withReg r i)) := rfl

/-- The behaviour of a plan on a configuration (SPEC §11.2).  It is defined by
structural recursion on the plan and mentions no compilation artefact. -/
def eval : Plan → Machine → Machine
  | nop, m => m
  | seq a b, m => b.eval (a.eval m)
  | classifyRaw v i u e, m =>
      match m.classify with
      | .valid => v.eval m
      | .invalid => i.eval m
      | .unsupported => u.eval m
      | .resourceExhausted => e.eval m
  | dispatchLayout r c g, m =>
      match m.layoutClass with
      | .rowMajor => r.eval m
      | .colMajor => c.eval m
      | .general => g.eval m
  | branch co t f, m => if co.eval m then t.eval m else f.eval m
  | pack src dst map _, m => { m with scratch := packedScratch m src dst map }
  | unpack src dst map _, m => { m with mem := unpackedMem m src dst map }
  | storeReg dst map width src, m => { m with mem := storedRegMem m dst map width src }
  | loadReg dst src map width, m => m.withReg dst (loadedReg m src map width)
  | loopNest axis body, m => runLoop body.eval axis.indexReg axis.extent 0 m
  | loopReg indexReg extentReg _ body, m =>
      runLoop body.eval indexReg (Nat.min (m.reg extentReg) loopRegMaxTrips) 0 m
  | tiled _ tiling extents body, m =>
      runLoop body.eval 0 (tiling.totalTiles extents.m extents.n extents.k) 0 m
  | reduce contract acc lhs rhs, m =>
      let r := reduceFold contract m.mem (Nat.min lhs.count rhs.count) lhs.base rhs.base
        (m.reg acc)
      if r.2 then { m with status := Status.checkedOverflow.code }
      else m.withReg acc r.1
  | allocScratch bytes body, m =>
      body.eval { m with scratch := m.scratch ++ List.replicate bytes 0 }
  | setReg dst value, m => m.withReg dst value
  | scalarOp op dst lhs rhs, m => m.withReg dst (op.apply (m.reg lhs) (m.reg rhs))
  | vectorOp op lanes dst lhs rhs, m =>
      m.withVreg dst
        ((List.zipWith op.lane.apply (m.vreg lhs) (m.vreg rhs)).take lanes)
  | emitTable index data, m => { m with tables := m.tables.set index data }
  | tableLoad table index dst, m => m.withReg dst ((m.tables.getD table []).getD index 0)
  | setStatus s, m => { m with status := s.code }
  | buildOutput src, m => { m with out := (m.mem.drop src.base).take src.count }
  | opaqueProcess _ body, m => body.eval m

/-! ### Defining equations -/

@[simp] theorem eval_nop (m : Machine) : nop.eval m = m := rfl

@[simp] theorem eval_seq (a b : Plan) (m : Machine) :
    (seq a b).eval m = b.eval (a.eval m) := rfl

@[simp] theorem eval_branch (co : Cond) (t f : Plan) (m : Machine) :
    (branch co t f).eval m = if co.eval m then t.eval m else f.eval m := rfl

@[simp] theorem eval_loopNest (axis : LoopAxis) (body : Plan) (m : Machine) :
    (loopNest axis body).eval m = runLoop body.eval axis.indexReg axis.extent 0 m := by
  simp [eval]

@[simp] theorem eval_tiled (o : TraversalOrder) (t : Tiling) (e : Extents)
    (body : Plan) (m : Machine) :
    (tiled o t e body).eval m =
      runLoop body.eval 0 (t.totalTiles e.m e.n e.k) 0 m := by
  simp [eval]

@[simp] theorem eval_allocScratch (bytes : Nat) (body : Plan) (m : Machine) :
    (allocScratch bytes body).eval m =
      body.eval { m with scratch := m.scratch ++ List.replicate bytes 0 } := rfl

@[simp] theorem eval_opaqueProcess (spec : OpaqueSpec) (body : Plan) (m : Machine) :
    (opaqueProcess spec body).eval m = body.eval m := rfl

@[simp] theorem eval_vectorOp (op : VectorOp) (lanes dst lhs rhs : Nat) (m : Machine) :
    (vectorOp op lanes dst lhs rhs).eval m =
      m.withVreg dst ((List.zipWith op.lane.apply (m.vreg lhs) (m.vreg rhs)).take lanes) :=
  rfl

@[simp] theorem eval_emitTable (index : Nat) (data : List Nat) (m : Machine) :
    (emitTable index data).eval m = { m with tables := m.tables.set index data } := rfl

@[simp] theorem eval_setStatus (s : Status) (m : Machine) :
    (setStatus s).eval m = { m with status := s.code } := rfl

theorem eval_classifyRaw (v i u e : Plan) (m : Machine) :
    (classifyRaw v i u e).eval m =
      match m.classify with
      | .valid => v.eval m
      | .invalid => i.eval m
      | .unsupported => u.eval m
      | .resourceExhausted => e.eval m := rfl

theorem eval_dispatchLayout (r c g : Plan) (m : Machine) :
    (dispatchLayout r c g).eval m =
      match m.layoutClass with
      | .rowMajor => r.eval m
      | .colMajor => c.eval m
      | .general => g.eval m := rfl

/-- An empty loop is exactly the empty plan. -/
theorem eval_loopNest_extent_zero (axis : LoopAxis) (body : Plan) (m : Machine)
    (h : axis.extent = 0) : (loopNest axis body).eval m = m := by
  rw [eval_loopNest, h, runLoop_zero]

/-- Allocating no scratch is exactly running the body. -/
theorem eval_allocScratch_zero (body : Plan) (m : Machine) :
    (allocScratch 0 body).eval m = body.eval m := by
  rw [eval_allocScratch]
  cases m
  simp

end Plan

/-! ## The register store law (SPEC §11.1 output construction)

`storeReg` is the only constructor that moves a value out of the register file
and into the observable store, so it is the one that has to satisfy a
load-after-store law.  All four statements below are about `Plan.eval` alone:
nothing here mentions compilation. -/

/-- The evaluation equation of a register store. -/
@[simp] theorem eval_storeReg (dst : RegionRef) (map : IndexMap) (width src : Nat)
    (m : Machine) :
    (Plan.storeReg dst map width src).eval m =
      { m with mem := storedRegMem m dst map width src } := rfl

/-- A register store never changes the size of the observable memory. -/
@[simp] theorem storeReg_mem_length (dst : RegionRef) (map : IndexMap)
    (width src : Nat) (m : Machine) :
    ((Plan.storeReg dst map width src).eval m).mem.length = m.mem.length := by
  rw [eval_storeReg]
  exact storedRegMem_length m dst map width src

/-- **Frame.**  Every cell outside the written range keeps its value. -/
theorem storeReg_outside (dst : RegionRef) (map : IndexMap) (width src : Nat)
    (m : Machine) (x : Nat)
    (hx : x < storeAddr m dst map ∨ storeAddr m dst map + width ≤ x) :
    ((Plan.storeReg dst map width src).eval m).mem.getD x 0 = m.mem.getD x 0 := by
  rw [eval_storeReg]
  refine gather_getD_outside _ _ width m.mem (storeAddr m dst map) 0 x ?_
  omega

/-- **Load after store.**  Reading the written address back, little-endian and at
the stored width, returns the value the register held — truncated to the width
the node declared, exactly as a `width`-byte machine store truncates it. -/
theorem storeReg_reads_back (dst : RegionRef) (map : IndexMap) (width src : Nat)
    (m : Machine) (hfit : storeAddr m dst map + width ≤ m.mem.length) :
    leWord ((Plan.storeReg dst map width src).eval m).mem (storeAddr m dst map) width =
      m.reg src % 256 ^ width := by
  rw [eval_storeReg]
  rw [← leWord_leBytes width (m.reg src)]
  refine leWord_ext width _ _ _ _ ?_
  intro k hk
  show (storedRegMem m dst map width src).getD (storeAddr m dst map + k) 0 =
    (leBytes (m.reg src) width).getD (0 + k) 0
  rw [Nat.zero_add]
  unfold storedRegMem
  have h := gather_getD_inside (leBytes (m.reg src) width) (fun i => i) width m.mem
    (storeAddr m dst map) 0 (storeAddr m dst map + k) (by omega) (by omega) (by omega)
  rw [h]
  have he : storeAddr m dst map + k - storeAddr m dst map = k := by omega
  rw [he]

/-- The exact form: a register that fits the declared width is read back
unchanged. -/
theorem storeReg_reads_back_exact (dst : RegionRef) (map : IndexMap) (width src : Nat)
    (m : Machine) (hfit : storeAddr m dst map + width ≤ m.mem.length)
    (hv : m.reg src < 256 ^ width) :
    leWord ((Plan.storeReg dst map width src).eval m).mem (storeAddr m dst map) width =
      m.reg src := by
  rw [storeReg_reads_back dst map width src m hfit, Nat.mod_eq_of_lt hv]

/-! ## The register load law (SPEC §11.1, and SPEC §3 input totality)

`loadReg` is the dual constructor: it is the only one that moves a value *out*
of the observable store and into the register file, so it is what lets a plan
compute with the descriptor it was handed instead of merely branching on it.
Everything below is about `Plan.eval` alone; nothing here mentions
compilation. -/

/-- The evaluation equation of a register load. -/
@[simp] theorem eval_loadReg (dst : Nat) (src : RegionRef) (map : IndexMap)
    (width : Nat) (m : Machine) :
    (Plan.loadReg dst src map width).eval m =
      m.withReg dst (loadedReg m src map width) := rfl

/-- A register load leaves the observable store alone: it reads, it does not
write. -/
@[simp] theorem loadReg_mem (dst : Nat) (src : RegionRef) (map : IndexMap)
    (width : Nat) (m : Machine) :
    ((Plan.loadReg dst src map width).eval m).mem = m.mem := rfl

/-- A register load never changes the size of the register file. -/
@[simp] theorem loadReg_regs_length (dst : Nat) (src : RegionRef) (map : IndexMap)
    (width : Nat) (m : Machine) :
    ((Plan.loadReg dst src map width).eval m).regs.length = m.regs.length := by
  simp

/-- **Store then load.**  A `loadReg` reading the address and width a `storeReg`
just wrote returns the value the register held, truncated to the declared width
exactly as a `width`-byte machine store truncates it.  This is the round trip
that makes the pair usable: a plan may deposit a computed value and read it back,
and — the point of the constructor — may read a value it never wrote, since the
law holds for the raw invocation's own bytes. -/
theorem loadReg_reads_stored (r : RegionRef) (map : IndexMap) (width dst src : Nat)
    (m : Machine) (hfit : storeAddr m r map + width ≤ m.mem.length)
    (hdst : dst < m.regs.length) :
    ((Plan.loadReg dst r map width).eval ((Plan.storeReg r map width src).eval m)).reg dst =
      m.reg src % 256 ^ width := by
  have hregs : ((Plan.storeReg r map width src).eval m).regs.length = m.regs.length := rfl
  have haddr :
      storeAddr ((Plan.storeReg r map width src).eval m) r map = storeAddr m r map := rfl
  rw [eval_loadReg, Machine.reg_withReg_same _ _ _ (by rw [hregs]; exact hdst)]
  unfold loadedReg
  rw [haddr]
  exact storeReg_reads_back r map width src m hfit

/-- The exact form of the round trip: a register that fits the declared width is
loaded back unchanged. -/
theorem loadReg_reads_stored_exact (r : RegionRef) (map : IndexMap)
    (width dst src : Nat) (m : Machine)
    (hfit : storeAddr m r map + width ≤ m.mem.length) (hdst : dst < m.regs.length)
    (hv : m.reg src < 256 ^ width) :
    ((Plan.loadReg dst r map width).eval ((Plan.storeReg r map width src).eval m)).reg dst =
      m.reg src := by
  rw [loadReg_reads_stored r map width dst src m hfit hdst, Nat.mod_eq_of_lt hv]

/-! ## The register-driven loop law

`loopReg` is the constructor that makes a *trip count* a function of the input.
The three equations below are its complete unfolding theory, and
`loopReg_eq_loopNest` is what transfers every fixed-extent result of the
development to it. -/

/-- The evaluation equation of a register-driven loop: the extent is read once,
before the loop, and the loop is then the ordinary `runLoop` on that natural
number. -/
@[simp] theorem eval_loopReg (ir er : Nat) (map : IndexMap) (body : Plan) (m : Machine) :
    (Plan.loopReg ir er map body).eval m =
      Plan.runLoop body.eval ir (Nat.min (m.reg er) loopRegMaxTrips) 0 m := rfl

/-- The clamp is inert below the released `i32` ceiling. -/
theorem min_loopRegMaxTrips {n : Nat} (h : n ≤ loopRegMaxTrips) :
    Nat.min n loopRegMaxTrips = n := Nat.min_eq_left h

/-- **An empty extent register runs the body not at all.** -/
theorem loopReg_zero (ir er : Nat) (map : IndexMap) (body : Plan) (m : Machine)
    (h : m.reg er = 0) : (Plan.loopReg ir er map body).eval m = m := by
  rw [eval_loopReg, h, min_loopRegMaxTrips (Nat.zero_le _), Plan.runLoop_zero]

/-- **The unfolding equation.**  An extent register holding `n + 1` runs the body
once at index `0` and then continues as a loop of `n` further iterations. -/
theorem loopReg_succ (ir er : Nat) (map : IndexMap) (body : Plan) (m : Machine)
    (n : Nat) (h : m.reg er = n + 1) (hb : n + 1 ≤ loopRegMaxTrips) :
    (Plan.loopReg ir er map body).eval m =
      Plan.runLoop body.eval ir n 1 (body.eval (m.withReg ir 0)) := by
  rw [eval_loopReg, h, min_loopRegMaxTrips hb]
  rfl

/-- **The transfer law.**  When the extent register holds `n`, a register-driven
loop is exactly the fixed-extent loop nest at literal extent `n`.  Every result
proved of `loopNest` at a literal extent therefore applies to the register-driven
loop that reaches that extent at run time. -/
theorem loopReg_eq_loopNest (ir er : Nat) (map : IndexMap) (body : Plan) (m : Machine)
    (n : Nat) (hn : m.reg er = n) (hb : n ≤ loopRegMaxTrips) :
    (Plan.loopReg ir er map body).eval m =
      (Plan.loopNest { indexReg := ir, extent := n, map := map } body).eval m := by
  rw [eval_loopReg, Plan.eval_loopNest, hn, min_loopRegMaxTrips hb]

/-- **The two constructors together: a trip count read out of the input.**
Loading a memory word into register `dst` and then running a `loopReg` whose
extent register is `dst` runs the body exactly as many times as *that memory
word* says — the loop is the fixed-extent nest at the extent the configuration
supplied, and the plan text names no extent at all.

This is the statement the fixed-extent fragment of the language cannot make, and
it is what a general GEMM needs: the descriptor's `m`, `n`, `k` reach the loop
bounds through the register file. -/
theorem loopReg_trips_from_memory (dst ir : Nat) (src : RegionRef) (map : IndexMap)
    (width : Nat) (body : Plan) (m : Machine) (hdst : dst < m.regs.length)
    (hb : loadedReg m src map width ≤ loopRegMaxTrips) :
    (Plan.seq (Plan.loadReg dst src map width) (Plan.loopReg ir dst map body)).eval m =
      (Plan.loopNest
        { indexReg := ir, extent := loadedReg m src map width, map := map } body).eval
        (m.withReg dst (loadedReg m src map width)) := by
  rw [Plan.eval_seq, eval_loadReg]
  exact loopReg_eq_loopNest ir dst map body _ (loadedReg m src map width)
    (Machine.reg_withReg_same m dst _ hdst) hb

/-! ## The SPEC-facing semantics -/

/-- `GNAF.Eval` of SPEC §11.3: the behaviour of a plan as a configuration
transformer, independent of compilation. -/
def Eval (p : Plan) : Machine → Machine := p.eval

@[simp] theorem Eval_apply (p : Plan) (m : Machine) : Eval p m = p.eval m := rfl

/-- The plan accepts an input/output configuration pair. -/
def Accepts (p : Plan) (input output : Machine) : Prop := Eval p input = output

/-- Plan behaviour is total: every configuration has an outcome. -/
theorem accepts_total (p : Plan) (input : Machine) : ∃ output, Accepts p input output :=
  ⟨Eval p input, rfl⟩

/-- Plan behaviour is deterministic. -/
theorem accepts_deterministic {p : Plan} {input o o' : Machine}
    (h : Accepts p input o) (h' : Accepts p input o') : o = o' := by
  unfold Accepts at h h'
  rw [← h, ← h']

/-- Sequential composition of behaviour (authority §5.7). -/
theorem accepts_seq {a b : Plan} {m n o : Machine}
    (ha : Accepts a m n) (hb : Accepts b n o) : Accepts (Plan.seq a b) m o := by
  unfold Accepts Eval at *
  simp only [Plan.eval_seq]
  rw [ha, hb]

theorem accepts_nop (m : Machine) : Accepts Plan.nop m m := rfl

/-- Semantic equivalence of plans: equality of the complete behaviour, as
authority §6.2 requires (behaviour, not syntax). -/
def SemanticallyEquivalent (a b : Plan) : Prop := Eval a = Eval b

namespace SemanticallyEquivalent

theorem refl (a : Plan) : SemanticallyEquivalent a a := rfl

theorem symm {a b : Plan} (h : SemanticallyEquivalent a b) :
    SemanticallyEquivalent b a := Eq.symm h

theorem trans {a b c : Plan} (hab : SemanticallyEquivalent a b)
    (hbc : SemanticallyEquivalent b c) : SemanticallyEquivalent a c := Eq.trans hab hbc

/-- Semantic equivalence is a congruence for sequential composition. -/
theorem seq {a a' b b' : Plan} (ha : SemanticallyEquivalent a a')
    (hb : SemanticallyEquivalent b b') :
    SemanticallyEquivalent (Plan.seq a b) (Plan.seq a' b') := by
  unfold SemanticallyEquivalent Eval at *
  funext m
  simp only [Plan.eval_seq]
  rw [congrFun ha m, congrFun hb (a'.eval m)]

end SemanticallyEquivalent

/-! ## Observations (authority §3.4, §5.5) -/

/-- The declared observation of a configuration: the released ABI makes the
status, the constructed output and the observable memory visible, and masks the
private scratch (SPEC §8.3). -/
structure Observation where
  status : Nat
  out : List Nat
  mem : List Nat
  deriving DecidableEq, Repr, Inhabited

/-- The observation projection. -/
def observe (m : Machine) : Observation := ⟨m.status, m.out, m.mem⟩

/-- Two plans are observationally equivalent when their declared observations
agree on every configuration. -/
def ObservationallyEquivalent (a b : Plan) : Prop :=
  ∀ m : Machine, observe (Eval a m) = observe (Eval b m)

/-- Equal behaviour implies equal observation.  The converse is deliberately
*not* claimed: authority §3.4 forbids promoting an observation agreement to a
semantic equality without a separate warrant. -/
theorem observationallyEquivalent_of_semanticallyEquivalent {a b : Plan}
    (h : SemanticallyEquivalent a b) : ObservationallyEquivalent a b := by
  intro m
  unfold SemanticallyEquivalent at h
  rw [congrFun h m]

theorem observationallyEquivalent_refl (a : Plan) : ObservationallyEquivalent a a :=
  fun _ => rfl

theorem observationallyEquivalent_trans {a b c : Plan}
    (hab : ObservationallyEquivalent a b) (hbc : ObservationallyEquivalent b c) :
    ObservationallyEquivalent a c := fun m => (hab m).trans (hbc m)

/-- The private scratch is not observable: two configurations differing only in
scratch have the same observation. -/
theorem observe_scratch_irrelevant (m : Machine) (s : List Nat) :
    observe { m with scratch := s } = observe m := rfl

end WasmGemmGnaf.GNAF
