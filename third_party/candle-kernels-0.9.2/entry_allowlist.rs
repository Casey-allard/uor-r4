// Frozen from reviewed CUDA source declarations BEFORE PTX generation.
// CUDA12.8+; min_arch uses CUDA __CUDA_ARCH__ units. RoPE withdrawn.
pub const EXPORT_GROUPS: &[(&str, u32, &[&str])] = &[
    (
        "affine",
        0,
        &[
            "affine_f32",
            "affine_f64",
            "affine_i16",
            "affine_i32",
            "affine_i64",
            "affine_u32",
            "affine_u8",
        ],
    ),
    ("affine", 530, &["affine_f16"]),
    ("affine", 800, &["affine_bf16"]),
    ("affine", 890, &["affine_f8_e4m3"]),
    (
        "binary",
        0,
        &[
            "badd_f32",
            "badd_f64",
            "badd_i64",
            "badd_u32",
            "badd_u8",
            "bdiv_f32",
            "bdiv_f64",
            "bdiv_i64",
            "bdiv_u32",
            "bdiv_u8",
            "bmaximum_f32",
            "bmaximum_f64",
            "bmaximum_i64",
            "bmaximum_u32",
            "bmaximum_u8",
            "bminimum_f32",
            "bminimum_f64",
            "bminimum_i64",
            "bminimum_u32",
            "bminimum_u8",
            "bmul_f32",
            "bmul_f64",
            "bmul_i64",
            "bmul_u32",
            "bmul_u8",
            "bsub_f32",
            "bsub_f64",
            "bsub_i64",
            "bsub_u32",
            "bsub_u8",
            "eq_f32",
            "eq_f64",
            "eq_i64",
            "eq_u32",
            "eq_u8",
            "ge_f32",
            "ge_f64",
            "ge_i64",
            "ge_u32",
            "ge_u8",
            "gt_f32",
            "gt_f64",
            "gt_i64",
            "gt_u32",
            "gt_u8",
            "le_f32",
            "le_f64",
            "le_i64",
            "le_u32",
            "le_u8",
            "lt_f32",
            "lt_f64",
            "lt_i64",
            "lt_u32",
            "lt_u8",
            "ne_f32",
            "ne_f64",
            "ne_i64",
            "ne_u32",
            "ne_u8",
        ],
    ),
    (
        "binary",
        530,
        &[
            "badd_f16",
            "bdiv_f16",
            "bmaximum_f16",
            "bminimum_f16",
            "bmul_f16",
            "bsub_f16",
            "eq_f16",
            "ge_f16",
            "gt_f16",
            "le_f16",
            "lt_f16",
            "ne_f16",
        ],
    ),
    (
        "binary",
        800,
        &[
            "badd_bf16",
            "badd_f8_e4m3",
            "bdiv_bf16",
            "bdiv_f8_e4m3",
            "bmaximum_bf16",
            "bmaximum_f8_e4m3",
            "bminimum_bf16",
            "bminimum_f8_e4m3",
            "bmul_bf16",
            "bmul_f8_e4m3",
            "bsub_bf16",
            "bsub_f8_e4m3",
            "eq_bf16",
            "eq_f8_e4m3",
            "ge_bf16",
            "ge_f8_e4m3",
            "gt_bf16",
            "gt_f8_e4m3",
            "le_bf16",
            "le_f8_e4m3",
            "lt_bf16",
            "lt_f8_e4m3",
            "ne_bf16",
            "ne_f8_e4m3",
        ],
    ),
    (
        "cast",
        0,
        &[
            "cast_bf16_f16",
            "cast_bf16_f32",
            "cast_bf16_f64",
            "cast_bf16_f8_e4m3",
            "cast_bf16_u8",
            "cast_f16_bf16",
            "cast_f32_bf16",
            "cast_f32_f32",
            "cast_f32_f64",
            "cast_f32_i64",
            "cast_f32_u32",
            "cast_f32_u8",
            "cast_f64_bf16",
            "cast_f64_f32",
            "cast_f64_f64",
            "cast_f64_i64",
            "cast_f64_u32",
            "cast_f64_u8",
            "cast_i64_f32",
            "cast_i64_f64",
            "cast_i64_i64",
            "cast_i64_u32",
            "cast_i64_u8",
            "cast_u32_f32",
            "cast_u32_f64",
            "cast_u32_i64",
            "cast_u32_u32",
            "cast_u32_u8",
            "cast_u8_bf16",
            "cast_u8_f32",
            "cast_u8_f64",
            "cast_u8_i64",
            "cast_u8_u32",
            "cast_u8_u8",
        ],
    ),
    (
        "cast",
        530,
        &[
            "cast_f16_f16",
            "cast_f16_f32",
            "cast_f16_f64",
            "cast_f16_u32",
            "cast_f16_u8",
            "cast_f32_f16",
            "cast_f64_f16",
            "cast_u32_f16",
            "cast_u8_f16",
        ],
    ),
    (
        "cast",
        800,
        &[
            "cast_bf16_bf16",
            "cast_bf16_u32",
            "cast_f16_f8_e4m3",
            "cast_f32_f8_e4m3",
            "cast_f64_f8_e4m3",
            "cast_f8_e4m3_bf16",
            "cast_f8_e4m3_f16",
            "cast_f8_e4m3_f32",
            "cast_f8_e4m3_f64",
            "cast_f8_e4m3_f8_e4m3",
            "cast_f8_e4m3_i32",
            "cast_f8_e4m3_u8",
            "cast_i32_f8_e4m3",
            "cast_u32_bf16",
            "cast_u8_f8_e4m3",
        ],
    ),
    (
        "fill",
        0,
        &[
            "const_set_f32",
            "const_set_f64",
            "const_set_i64",
            "const_set_u32",
            "const_set_u8",
            "copy2d_f32",
            "copy2d_f64",
            "copy2d_i64",
            "copy2d_u32",
            "copy2d_u8",
            "fill_f32",
            "fill_f64",
            "fill_i64",
            "fill_u32",
            "fill_u8",
        ],
    ),
    ("fill", 530, &["const_set_f16", "copy2d_f16", "fill_f16"]),
    (
        "fill",
        800,
        &[
            "const_set_bf16",
            "const_set_f8_e4m3",
            "copy2d_bf16",
            "copy2d_f8_e4m3",
            "fill_bf16",
            "fill_f8_e4m3",
        ],
    ),
    (
        "indexing",
        0,
        &[
            "gather_i64_f32",
            "gather_i64_f64",
            "gather_i64_i64",
            "gather_i64_u32",
            "gather_i64_u8",
            "gather_u32_f32",
            "gather_u32_f64",
            "gather_u32_i64",
            "gather_u32_u32",
            "gather_u32_u8",
            "gather_u8_f32",
            "gather_u8_f64",
            "gather_u8_i64",
            "gather_u8_u32",
            "gather_u8_u8",
            "ia_i64_f32",
            "ia_i64_f64",
            "ia_i64_i64",
            "ia_i64_u32",
            "ia_i64_u8",
            "ia_u32_f32",
            "ia_u32_f64",
            "ia_u32_i64",
            "ia_u32_u32",
            "ia_u32_u8",
            "ia_u8_f32",
            "ia_u8_f64",
            "ia_u8_i64",
            "ia_u8_u32",
            "ia_u8_u8",
            "is_i64_f32",
            "is_i64_f64",
            "is_i64_i64",
            "is_i64_u32",
            "is_i64_u8",
            "is_u32_f32",
            "is_u32_f64",
            "is_u32_i64",
            "is_u32_u32",
            "is_u32_u8",
            "is_u8_f32",
            "is_u8_f64",
            "is_u8_i64",
            "is_u8_u32",
            "is_u8_u8",
            "s_i64_f32",
            "s_i64_f64",
            "s_i64_i64",
            "s_i64_u32",
            "s_i64_u8",
            "s_u32_f32",
            "s_u32_f64",
            "s_u32_i64",
            "s_u32_u32",
            "s_u32_u8",
            "s_u8_f32",
            "s_u8_f64",
            "s_u8_i64",
            "s_u8_u32",
            "s_u8_u8",
            "sa_i64_f32",
            "sa_i64_f64",
            "sa_i64_i64",
            "sa_i64_u32",
            "sa_i64_u8",
            "sa_u32_f32",
            "sa_u32_f64",
            "sa_u32_i64",
            "sa_u32_u32",
            "sa_u32_u8",
            "sa_u8_f32",
            "sa_u8_f64",
            "sa_u8_i64",
            "sa_u8_u32",
            "sa_u8_u8",
        ],
    ),
    (
        "indexing",
        530,
        &[
            "gather_i64_f16",
            "gather_u32_f16",
            "gather_u8_f16",
            "ia_i64_f16",
            "ia_u32_f16",
            "ia_u8_f16",
            "is_i64_f16",
            "is_u32_f16",
            "is_u8_f16",
            "s_i64_f16",
            "s_u32_f16",
            "s_u8_f16",
            "sa_i64_f16",
            "sa_u32_f16",
            "sa_u8_f16",
        ],
    ),
    (
        "indexing",
        800,
        &[
            "gather_i64_bf16",
            "gather_u32_bf16",
            "gather_u8_bf16",
            "ia_i64_bf16",
            "ia_u32_bf16",
            "ia_u8_bf16",
            "is_i64_bf16",
            "is_u32_bf16",
            "is_u8_bf16",
            "s_i64_bf16",
            "s_u32_bf16",
            "s_u8_bf16",
            "sa_i64_bf16",
            "sa_u32_bf16",
            "sa_u8_bf16",
        ],
    ),
    (
        "indexing",
        890,
        &[
            "gather_i16_f8_e4m3",
            "gather_i32_f8_e4m3",
            "gather_i64_f8_e4m3",
            "gather_u32_f8_e4m3",
            "gather_u8_f8_e4m3",
            "ia_i16_f8_e4m3",
            "ia_i32_f8_e4m3",
            "ia_i64_f8_e4m3",
            "ia_u32_f8_e4m3",
            "ia_u8_f8_e4m3",
            "is_i16_f8_e4m3",
            "is_i32_f8_e4m3",
            "is_i64_f8_e4m3",
            "is_u32_f8_e4m3",
            "is_u8_f8_e4m3",
            "sa_i16_f8_e4m3",
            "sa_i32_f8_e4m3",
            "sa_i64_f8_e4m3",
            "sa_u32_f8_e4m3",
            "sa_u8_f8_e4m3",
        ],
    ),
    (
        "reduce",
        0,
        &[
            "fast_argmax_f32",
            "fast_argmax_f64",
            "fast_argmax_i64",
            "fast_argmax_u32",
            "fast_argmax_u8",
            "fast_argmin_f32",
            "fast_argmin_f64",
            "fast_argmin_i64",
            "fast_argmin_u32",
            "fast_argmin_u8",
            "fast_max_f32",
            "fast_max_f64",
            "fast_max_i64",
            "fast_max_u32",
            "fast_max_u8",
            "fast_min_f32",
            "fast_min_f64",
            "fast_min_i64",
            "fast_min_u32",
            "fast_min_u8",
            "fast_sum_f32",
            "fast_sum_f64",
            "fast_sum_i64",
            "fast_sum_u32",
            "fast_sum_u8",
            "layernorm_f32",
            "layernorm_f64",
            "rmsnorm_f32",
            "rmsnorm_f64",
            "sum_f32",
            "sum_f64",
            "sum_u32",
        ],
    ),
    (
        "reduce",
        530,
        &[
            "fast_argmax_f16",
            "fast_argmin_f16",
            "fast_max_f16",
            "fast_min_f16",
            "fast_sum_f16",
            "layernorm_f16",
            "rmsnorm_f16",
            "sum_f16",
        ],
    ),
    (
        "reduce",
        800,
        &[
            "fast_argmax_bf16",
            "fast_argmin_bf16",
            "fast_max_bf16",
            "fast_min_bf16",
            "fast_sum_bf16",
            "layernorm_bf16",
            "rmsnorm_bf16",
            "sum_bf16",
        ],
    ),
    (
        "sort",
        0,
        &[
            "asort_asc_f32",
            "asort_asc_f64",
            "asort_asc_i64",
            "asort_asc_u32",
            "asort_asc_u8",
            "asort_desc_f32",
            "asort_desc_f64",
            "asort_desc_i64",
            "asort_desc_u32",
            "asort_desc_u8",
        ],
    ),
    ("sort", 530, &["asort_asc_f16", "asort_desc_f16"]),
    ("sort", 800, &["asort_asc_bf16", "asort_desc_bf16"]),
    (
        "ternary",
        0,
        &[
            "where_i64_f32",
            "where_i64_f64",
            "where_i64_i64",
            "where_i64_u32",
            "where_i64_u8",
            "where_u32_f32",
            "where_u32_f64",
            "where_u32_i64",
            "where_u32_u32",
            "where_u32_u8",
            "where_u8_f32",
            "where_u8_f64",
            "where_u8_i64",
            "where_u8_u32",
            "where_u8_u8",
        ],
    ),
    (
        "ternary",
        530,
        &["where_i64_f16", "where_u32_f16", "where_u8_f16"],
    ),
    (
        "ternary",
        800,
        &["where_i64_bf16", "where_u32_bf16", "where_u8_bf16"],
    ),
    (
        "ternary",
        890,
        &[
            "where_i16_fp8_e4m3",
            "where_i32_fp8_e4m3",
            "where_i64_fp8_e4m3",
            "where_u32_fp8_e4m3",
            "where_u8_fp8_e4m3",
        ],
    ),
    (
        "unary",
        0,
        &[
            "uabs_f32",
            "uabs_f64",
            "uceil_f32",
            "uceil_f64",
            "ucopy_f32",
            "ucopy_f64",
            "ucopy_i64",
            "ucopy_u32",
            "ucopy_u8",
            "ucos_f32",
            "ucos_f64",
            "ufloor_f32",
            "ufloor_f64",
            "uneg_f32",
            "uneg_f64",
            "urecip_f32",
            "urecip_f64",
            "urelu_f32",
            "urelu_f64",
            "uround_f32",
            "uround_f64",
            "usign_f32",
            "usign_f64",
            "usin_f32",
            "usin_f64",
            "usqr_f32",
            "usqr_f64",
            "usqrt_f32",
            "usqrt_f64",
        ],
    ),
    (
        "unary",
        530,
        &[
            "uabs_f16",
            "uceil_f16",
            "ucopy_f16",
            "ucos_f16",
            "ufloor_f16",
            "uneg_f16",
            "urecip_f16",
            "urelu_f16",
            "uround_f16",
            "usign_f16",
            "usin_f16",
            "usqr_f16",
            "usqrt_f16",
        ],
    ),
    (
        "unary",
        800,
        &[
            "uabs_bf16",
            "uceil_bf16",
            "ucopy_bf16",
            "ucos_bf16",
            "ufloor_bf16",
            "uneg_bf16",
            "urecip_bf16",
            "urelu_bf16",
            "uround_bf16",
            "usign_bf16",
            "usin_bf16",
            "usqr_bf16",
            "usqrt_bf16",
        ],
    ),
    (
        "unary",
        890,
        &[
            "uabs_fp8_e4m3",
            "uceil_fp8_e4m3",
            "ucopy_f8_e4m3",
            "ucos_fp8_e4m3",
            "ufloor_fp8_e4m3",
            "uneg_fp8_e4m3",
            "urecip_fp8_e4m3",
            "urelu_fp8_e4m3",
            "uround_fp8_e4m3",
            "usign_fp8_e4m3",
            "usin_fp8_e4m3",
            "usqr_fp8_e4m3",
            "usqrt_fp8_e4m3",
        ],
    ),
];

use std::collections::BTreeSet;

/// Static source-derived names, not discovered from compiler output. The
/// thresholds mirror reviewed __CUDA_ARCH__ conditional declarations. This
/// inventory requires CUDA12.8+ (the source's CUDA_VERSION>=11000 branch).
pub fn expected_entries(module: &str, arch: u32) -> Result<BTreeSet<&'static str>, String> {
    if !EXPORT_GROUPS.iter().any(|(name, _, _)| *name == module) {
        return Err(format!("unknown vendor PTX module {module}"));
    }
    let mut entries = BTreeSet::new();
    for (name, minimum, names) in EXPORT_GROUPS {
        if *name == module && arch >= *minimum {
            for entry in *names {
                if !entries.insert(*entry) {
                    return Err(format!("duplicate frozen export {module}/{entry}"));
                }
            }
        }
    }
    Ok(entries)
}

// Tokenize PTX declarations; comments and quoted file/debug strings do not
// declare kernels. This checks exported names, not arbitrary body semantics.
fn tokens(text: &str) -> Vec<&str> {
    let b = text.as_bytes();
    let mut result = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i..].starts_with(b"//") {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
        } else if b[i..].starts_with(b"/*") {
            i += 2;
            while i + 1 < b.len() && !b[i..].starts_with(b"*/") {
                i += 1;
            }
            i = (i + 2).min(b.len());
        } else if b[i] == b'"' {
            i += 1;
            while i < b.len() {
                if b[i] == b'\\' {
                    i = (i + 2).min(b.len());
                } else if b[i] == b'"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
        } else if b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'.' {
            let start = i;
            i += 1;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'.') {
                i += 1;
            }
            result.push(&text[start..i]);
        } else {
            i += 1;
        }
    }
    result
}

fn target_arch(target: &str) -> Result<u32, String> {
    let tail = target
        .strip_prefix("sm_")
        .ok_or_else(|| format!("invalid PTX target {target}"))?;
    let digits: String = tail.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty()
        || !tail[digits.len()..]
            .chars()
            .all(|c| c.is_ascii_alphabetic())
    {
        return Err(format!("invalid PTX target {target}"));
    }
    digits
        .parse::<u32>()
        .ok()
        .and_then(|cap| cap.checked_mul(10))
        .ok_or_else(|| format!("invalid PTX target capability {target}"))
}

pub fn check_ptx_exports(module: &str, text: &str) -> Result<(), String> {
    let words = tokens(text);
    let targets: Vec<_> = words
        .windows(2)
        .filter(|pair| pair[0] == ".target")
        .collect();
    if targets.len() != 1 {
        return Err(format!("{module}: expected one PTX .target declaration"));
    }
    let expected = expected_entries(module, target_arch(targets[0][1])?)?;
    let mut actual = BTreeSet::new();
    for pair in words.windows(2).filter(|pair| pair[0] == ".entry") {
        let name = pair[1];
        if name.is_empty() || !name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_') {
            return Err(format!("{module}: invalid PTX entry name {name}"));
        }
        if !actual.insert(name) {
            return Err(format!("{module}: duplicate PTX entry {name}"));
        }
    }
    if actual != expected {
        let missing: Vec<_> = expected.difference(&actual).copied().collect();
        let extra: Vec<_> = actual.difference(&expected).copied().collect();
        return Err(format!("{module}: PTX exports differ from frozen source inventory; missing={missing:?}; extra={extra:?}"));
    }
    Ok(())
}

/// Metadata controls only: fixture PTX text is never compiled or loaded.
pub fn self_test() {
    for arch in [500, 530, 800, 890, 1200] {
        for module in [
            "affine", "binary", "cast", "fill", "indexing", "reduce", "sort", "ternary", "unary",
        ] {
            let names = expected_entries(module, arch).expect("known metadata module");
            assert!(!names.is_empty());
            let mut fixture = format!(".version 7.0\n.target sm_{}\n.address_size 64\n", arch / 10);
            for name in &names {
                fixture.push_str(&format!(".visible .entry {name}() {{ ret; }}\n"));
            }
            fixture
                .push_str("// .entry ignored_comment() {}\n.file 1 \".entry ignored_filename\"\n");
            assert!(check_ptx_exports(module, &fixture).is_ok());
            assert!(check_ptx_exports(
                module,
                &(fixture.clone() + ".visible .entry harmless_renamed_extra() { ret; }\n")
            )
            .is_err());
            let first = names.first().expect("nonempty");
            let missing = fixture.replace(&format!(".visible .entry {first}() {{ ret; }}\n"), "");
            assert!(check_ptx_exports(module, &missing).is_err());
            assert!(check_ptx_exports(
                module,
                &(fixture.clone() + &format!(".visible .entry {first}() {{ ret; }}\n"))
            )
            .is_err());
            assert!(names.iter().all(|name| !name.starts_with("rope")));
        }
    }
    assert!(check_ptx_exports("unknown", ".target sm_80\n.entry placeholder() {}").is_err());
    assert!(check_ptx_exports("affine", ".entry placeholder() {}").is_err());
    assert!(check_ptx_exports("affine", ".target sm_80\n.target sm_80\n").is_err());
}

#[cfg(test)]
mod tests {
    #[test]
    fn frozen_exports_reject_missing_extra_duplicate_and_architecture_drift() {
        super::self_test();
    }
}
