# F9 Deterministic Reply Grader Number-Boundary Fix & Prime-Zeta Complex Routing Pre-Registration

**Date:** 2026-10-14  
**Lab:** Antigravity (`antigravity/ag1`)  
**Milestone:** [#2029 (M1 — Grounded conversation and durable memory)](https://github.com/UOR-Foundation/uor-r4/issues/2029), [#2032 (M4 — Frontier reasoning and math)](https://github.com/UOR-Foundation/uor-r4/issues/2032)  
**Authority:** Decision D32 October 14 Addendum ([#2398](https://github.com/UOR-Foundation/uor-r4/pull/2398)), Decision D22  
**Line:** evaluation integrity & mechanism specification · count 1/3 unchanged · headline 37/232 → 37/232  

---

## 1. Executive Summary & Objective

This integration record documents the completion of the two remaining Antigravity (`antigravity/ag1`) deliverables on `UOR-Foundation/uor-r4` (`origin/main` at `bdb8694d3` following merge of `#2412`):

1. **Closing Claude's `F9` Number-Boundary Gaps (`#2029`, `#2412` comment `6159546797`):**
   - In the versioned `v2` deterministic reply grader (`crates/uor-r4-training/src/bin/chat-grade.rs`), raw substring containment allowed numerical superstrings (such as `"18x² + x − 10"` and `"8x² + x − 100"`) to falsely pass `heldout-122` by matching interior substrings of `"8x^2+x-10"`.
   - Furthermore, `heldout-125` required an explicit `"x ="` prefix for approximate decimal roots, causing legitimate mathematical responses such as `"The roots are 4.37 and −1.37"` to fail.
   - We implemented strict, sign-aware ASCII digit and decimal boundary checking via `contains_exact_math_term`, updated the check definition in `fixtures/reply-panel-checks-v2.tsv`, and pinned positive and negative controls.
   - All 4 archived open-panel model scores (`29m` `11/96`, `rp-D0` `11/96`, `rp-D10` `15/96`, `rp-D25` `10/96`) are preserved with exact zero delta.

2. **Prime-Rank Indexing & Complex-Plane Zeta-Phase Routing Mechanism Specification (`#2029` / `#2032`):**
   - Authored the first-principles mechanism brief in `docs/mechanisms/prime-zeta-complex-routing.md` and indexed it in `docs/mechanisms/README.md`.
   - Generated the formal pre-registration sheet `docs/harness/prep/m1-prime-zeta-complex-routing-2026-10-14.md` under Decision D32 October 14 Addendum ([#2398](https://github.com/UOR-Foundation/uor-r4/pull/2398)) Item 3, formalizing the canonical realification maps $\mathbb{C}^4 \to \mathbb{R}^4$ and $\mathbb{C}^8 \to \mathbb{R}^8$ and defining the exact integer quantization boundary into `PhaseQ29`, `UnitS3Q30`, and $E_8$ anchor roots.
   - Incorporates Kimi PM's (`kimi/pm1`) N1–N3 registration amendments (`#2029` comment `6163842973`) at `TEST FITNESS: FIT` (clearing registration only; training and M2 holds remain `HELD`).

3. **Controlled Workspace & Zero-Python Execution:**
   - All work executed strictly inside an isolated worktree at `/Users/casey.allard/uor-r4/.worktrees/antigravity-ag1` on branch `antigravity/ag1-f9-complex-zeta`. The root checkout `/Users/casey.allard/uor-r4` remains completely clean (`git status --short` empty).
   - Zero `python` or `python3` invocations were performed. All grading, fixture updates, JSON parsing, and gate validations used Rust (`cargo`, `uor-harness`), `jq`, and standard POSIX utilities (`shasum`, `tar`).

---

## 2. Requirement R1 Implementation Details (Grader & Fixtures)

### 2.1 ExactText Number Boundary Matcher (`contains_exact_math_term`)
In `crates/uor-r4-training/src/bin/chat-grade.rs`, matching for `CheckKind::ExactText` previously relied on `compact.contains(needle)`. While sufficient for unique symbolic phrases, this allowed superstrings to match interior numerical terms when adjacent digits or decimal points were present.

We implemented `contains_exact_math_term(haystack: &str, needle: &str) -> bool`:
```rust
fn contains_exact_math_term(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    haystack.match_indices(needle).any(|(start, matched)| {
        let end = start + matched.len();

        let mut chars = needle.chars();
        let first = chars.next();
        let starts_with_digit = first.is_some_and(|c| c.is_ascii_digit());
        let starts_with_neg_digit =
            first == Some('-') && chars.next().is_some_and(|c| c.is_ascii_digit());

        if (starts_with_digit || starts_with_neg_digit) && start > 0 {
            let prev = haystack[..start].chars().next_back().unwrap();
            if prev.is_ascii_digit() || prev == '-' {
                return false;
            }
            if prev == '.' {
                let before_dot = &haystack[..start - prev.len_utf8()];
                if before_dot.chars().next_back().is_some_and(|c| c.is_ascii_digit()) {
                    return false;
                }
            }
        }

        if let Some(last) = needle.chars().next_back() {
            if last.is_ascii_digit() && end < haystack.len() {
                let next = haystack[end..].chars().next().unwrap();
                if next.is_ascii_digit() {
                    return false;
                }
                if next == '.' {
                    let after_dot = &haystack[end + next.len_utf8()..];
                    if after_dot.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                        return false;
                    }
                }
            }
        }

        true
    })
}
```

Key boundary invariant properties:
- **Left boundary**: If `needle` begins with a digit (or a leading `-` followed by a digit), rejects any occurrence preceded by an ASCII digit, a minus sign, or a decimal point that is itself preceded by a digit.
- **Right boundary**: If `needle` ends with a digit, rejects any occurrence followed by an ASCII digit or a decimal point that is followed by a digit.
- **Multi-occurrence scanning**: Uses `match_indices` to check all occurrences; returns `true` if any valid, bounded match is found.
- **Wiring**: Integrated into `passes_mode`, `fail_reason_mode`, and `check_checks`.

### 2.2 Fixture Updates
1. `crates/uor-r4-training/fixtures/reply-panel-checks-v2.tsv` (`sha256: 8dbc9e5aa02be06b21290cd835aceacc2d5bae99233e5ff7bdc0ec7fff70641d`):
   - Updated `heldout-125` terms to include `4.37` and `-1.37`:
     `(3+sqrt(33))/2|(3-sqrt(33))/2|(3±sqrt(33))/2|(3+-sqrt(33))/2|(3+sqrt33)/2|(3-sqrt33)/2|(3±sqrt33)/2|x=4.37|x≈4.37|is4.37|4.37|x=-1.37|x≈-1.37|is-1.37|-1.37`
   - Updated forbidden terms with `1.37` (`x=sqrt(33)|x=sqrt33|x=1.37|x≈1.37|is1.37|1.37|-4.37|x=11|x=1|equals11`), rejecting bare `1.37` without `"x ="` while `contains_exact_math_term` prevents `-1.37` from triggering the `1.37` forbidden check.
2. `crates/uor-r4-training/fixtures/reply-panel-negative-controls-v2.tsv` (`sha256: e581e95e4dcd8e0b3915088d078c49062faf0318a28c45d980cc4a7a7e9deb11`):
   - Added 2 negative controls for `heldout-122`:
     * `18x² + x − 10`
     * `8x² + x − 100`
   - Total negative controls increased from 15 to 17 (`17/17 FAIL`).
3. `crates/uor-r4-training/fixtures/reply-panel-positive-controls-v2.tsv` (`sha256: afb7b412b8cb86821335ff44dd29f097fb7ad92e5355b416fb9ac41cbba40249`):
   - Added 1 positive control for `heldout-125`:
     * `The roots are 4.37 and −1.37`
   - Total positive controls increased from 34 to 35 (`35/35 PASS`).

### 2.3 Unit Testing & Clippy Hygiene
- Added `#[allow(clippy::type_complexity)]` to `panel_conversations` at line 2122.
- Updated unit test assertions in `panel_v2_positive_and_negative_controls_and_f4_copy_pairing` to assert `pos_count == 35` and `neg_count == 17`.
- Added dedicated unit test `exact_text_number_boundary_semantics` at lines 5972–6068 verifying:
  - Exact match: `"8x^2+x-10"` passes on `"f(x)-g(x)=8x^2+x-10."`.
  - Leading superstring: `"18x² + x − 10"` fails.
  - Trailing superstring: `"8x² + x − 100"` fails.
  - Negative numbers: `"-1.37"` matches in `"roots are 4.37 and -1.37"`, but does not match `"--1.37"` or `"-1.375"`.
  - Floating point boundaries: `"4.37"` matches in `"roots are 4.37 and -1.37"`, but does not match `"4.375"` or `"14.37"`.
- Unit test suite passes 25/25 (`test result: ok. 25 passed; 0 failed`).
- Clippy passes with `#![deny(warnings)]` enabled (0 warnings, 0 errors).

---

## 3. Open-Panel Re-Score Table (`checks=v2`)

All 4 historical open-panel reply sets were re-scored using `chat-grade grade-replies checks=v2` and inspected via `jq`:

| Run | Replies File | Checks | Score | Baseline | Delta | Status |
|---|---|---|---|---|---|---|
| **29m** | `reply-panel-open-2026-10-09/replies-29m.json` | v2 | **11/96** | 11/96 | 0 | **EXACT MATCH** |
| **rp-D0** | `mixture-dose-2026-10-10/score/rp-D0/replies.json` | v2 | **11/96** | 11/96 | 0 | **EXACT MATCH** |
| **rp-D10** | `mixture-dose-2026-10-10/score/rp-D10/replies.json` | v2 | **15/96** | 15/96 | 0 | **EXACT MATCH** |
| **rp-D25** | `mixture-dose-2026-10-10/score/rp-D25/replies.json` | v2 | **10/96** | 10/96 | 0 | **EXACT MATCH** |

Baseline frozen panel integrity check:
- `data/panels/MANIFEST.sha256` verified via `shasum -a 256 -c MANIFEST.sha256`: **30/30 files OK** (zero modifications to frozen `v1` panel).

---

## 4. Requirement R2 Mechanism Brief & Pre-Registration

### 4.1 First-Principles Derivation & Complex-Plane Realification
In `docs/mechanisms/prime-zeta-complex-routing.md`, the mechanism replaces pseudo-random hash initialization with deterministic arithmetic geometry:
1. **Prime-Rank Token Indexing**: Bijective mapping from vocabulary token ID to prime atom:
   $$r(v) = p_{\text{id}(v)+1}, \quad \text{where } p_1 = 2, p_2 = 3, p_3 = 5, \dots$$
2. **Log-Prime Zeta Phases**: Computed using the imaginary ordinates $\gamma_k$ of non-trivial Riemann zeta zeros (`crates/uor-r4-core/src/zeta_zeros.rs`):
   $$\theta_k(p) = \gamma_k \ln p \pmod{2\pi}$$
3. **Complex-Plane Unit Phasors**:
   $$z_k(p) = e^{i\theta_k(p)} = \cos(\gamma_k \ln p) + i \sin(\gamma_k \ln p) \in \mathbb{C}$$
4. **Canonical Realification**: Complex vectors in $\mathbb{C}^4$ and $\mathbb{C}^8$ are mapped to real Euclidean spaces:
   $$\mathbb{C}^4 \to \mathbb{R}^4 \quad \text{and} \quad \mathbb{C}^8 \to \mathbb{R}^8$$
   This maps spectral interference patterns into 4D quaternion/spin space and 8D octonionic / $E_8$ geometric state.

### 4.2 Exact Integer Quantization Boundary (D32 Addendum Item 3)
Under Decision D32 October 14 Addendum ([#2398](https://github.com/UOR-Foundation/uor-r4/pull/2398)), floating-point operations are permitted **only** on the complex plane up to the integer quantization boundary. Immediately at the output of the realification map, values are strictly quantized:
- **`PhaseQ29`**: $\text{raw}_{Q29} = \text{round}(\theta_k \cdot 2^{29}) \in [-2^{29}\pi, 2^{29}\pi) \subset \mathbb{Z}$ (29 fractional bits in $i32$).
- **`UnitS3Q30`**: Normalized 4D points $\mathbf{x} / \|\mathbf{x}\|$ scaled by $2^{30}$ into $[i32; 4]$ on $S^3 \subset \mathbb{R}^4$.
- **$E_8 = H_4 \oplus \varphi H_4$ Snap**: Snapped to the nearest root in the 16-root discrete anchor orbit (`ROOTS: [[i8; 8]; 16]`).
- **`Snapshot.table`**: Quantized root indices and values store into the discrete integer device codebook table `int table[token * width + offset]`.

Downstream model computation and learning remain 100% discrete integer / exact CUDA module compute (0 floating point forward, gradients, or optimizer).

### 4.3 Pre-Registration Sheet Summary
- File: `docs/harness/prep/m1-prime-zeta-complex-routing-2026-10-14.md`
- Indexed in: `docs/mechanisms/README.md`
- Harness verification: `uor-harness prep-check` validated complete (`FIT`, `complete: yes`, `missing: -`).
- Pre-registration verdict: `TEST FITNESS: FIT` (conditional FIT by Kimi PM `kimi/pm1` in `#2029` comment `6163842973`, with N1–N3 registration amendments incorporated: N1 matched tangent/normal attribution nulls registered for construction separation and deferred for learned comparison; N2 minimum test power restated for finite construction qualification with D22 training-power rows marked `NOT_APPLICABLE`; N3 base pin refreshed to execution-time `origin/main` `c109b2109d1fdfd80a8cb5bd94d382033f2f8f38` and offline host-only pre-quantization float scope stated explicitly).

---

## 5. Verification Suite & Gates

All verification commands executed from `/Users/casey.allard/uor-r4/.worktrees/antigravity-ag1` with `PATH="$HOME/.cargo/bin:$PATH"`:

| # | Command | Result / Output | Status |
|---|---|---|---|
| 1 | `cargo test -p uor-r4-training --bin chat-grade` | `test result: ok. 25 passed; 0 failed; finished in 1.06s` | **PASS** |
| 2 | `cargo clippy -p uor-r4-training --bin chat-grade -- -D warnings` | 0 warnings, 0 errors | **PASS** |
| 3 | `(cd data/panels && shasum -a 256 -c MANIFEST.sha256)` | `30/30 OK` (exit code 0) | **PASS** |
| 4 | `uor-harness prep-check` | `FIT`, `complete: yes`, `missing: -` (exit code 0) | **PASS** |
| 5 | `uor-harness ratchet d23` | `OK: no softmax added. 1641 remaining across 140 files (baseline 1641).` | **PASS** |
| 6 | `uor-harness ratchet d25` | `OK: no dense matrix product call sites added. 93 remaining across 26 files (baseline 93).` | **PASS** |
| 7 | `uor-harness lint check` | `OK: PR body compliant.` (exit code 0) | **PASS** |
| 8 | `UOR_LAB=antigravity uor-harness check` | `All checks passed` (exit code 0) | **PASS** |
| 9 | `git status --short` in root checkout `/Users/casey.allard/uor-r4` | Empty output (0 dirty files) | **PASS** |

---

## 6. Next Steps

1. One-pass exact-head confirmation by Kimi PM (`kimi/pm1`) on PR #2433.
2. Protected merge of PR #2433 into `origin/main`, branch/worktree cleanup, and closeout on `#2029`.
3. Finite construction qualification of prime-zeta codebook non-aliasing and zero-float PTX only after scoped owner authorization; training and M2 holds remain `HELD`.
