//! D29 bounded static vendor check; std only, no Cargo or model execution.
//! rustc --edition=2021 scripts/check_native_cuda_vendor.rs -o local/check-native-cuda-vendor
//! local/check-native-cuda-vendor --repo . [--ptx-dir VENDOR_OUT_DIR]
//!     [--binary-imports SAVED_READELF_OUTPUT] [--features SAVED_CARGO_TREE_OUTPUT]
//! local/check-native-cuda-vendor --self-test
//! The optional files are independently captured evidence, not commands this tool runs.
//! Static absence is not runtime qualification or proof against arbitrary renamed operations.
use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
    process,
};

#[path = "../crates/uor-r4-discrete/src/decomposition_source.rs"]
mod decomposition_source;
#[path = "../third_party/candle-kernels-0.9.2/entry_allowlist.rs"]
mod entry_allowlist;
mod native_dependency_boundary;

const MODULES: [&str; 9] = [
    "affine", "binary", "cast", "fill", "indexing", "reduce", "sort", "ternary", "unary",
];
const TRAINING_COMPILED_CU: [&str; 3] = [
    "crates/uor-r4-training/src/cuda_stack.cu",
    "crates/uor-r4-training/src/spiralcore_fbs.cu",
    "crates/uor-r4-training/src/starling_native.cu",
];
const DISCRETE_ENTRIES: [&str; 11] = [
    "discrete_init",
    "discrete_encode",
    "discrete_forward",
    "discrete_counterfactual",
    "discrete_choose",
    "discrete_apply",
    "discrete_validate",
    "discrete_evaluate",
    "discrete_evaluation_aggregate",
    "discrete_count",
    "discrete_count_score",
];
const BANNED_FEATURES: [&str; 8] = [
    "cublas",
    "cublaslt",
    "nvrtc",
    "cudnn",
    "cutensor",
    "cusolver",
    "cusolvermg",
    "cusparse",
];
const STANDALONE_INSTRUMENT_MANIFESTS: [&str; 5] = [
    "docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/Cargo.toml",
    "docs/labs/admission-redirect-packet-2026-10-15/instrument/Cargo.toml",
    "docs/labs/fresh-scoring-eval-2026-10-15/instrument/Cargo.toml",
    "docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml",
    "docs/labs/stage-a-admission-instrument-2026-10-15/instrument/Cargo.toml",
];
const APPROVED_BUILD_SCRIPTS: [&str; 3] = [
    "crates/uor-r4-training/build.rs",
    "crates/uor-r4-discrete/build.rs",
    "docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/build.rs",
];
const MAX_FILE: u64 = 64 * 1024 * 1024;

fn read(path: &Path) -> Result<String, String> {
    let m = fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if !m.is_file() || m.len() > MAX_FILE {
        return Err(format!(
            "{}: expected regular UTF-8 file <=64MiB",
            path.display()
        ));
    }
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SourceLang {
    Cuda,
    Ptx,
    Rust,
    Auto,
}

// Preserve offsets/newlines while excluding comments. Keep strings: CUDA
// inline PTX assembly is executable even when spelled inside a string.
fn code_for(text: &str, lang: SourceLang) -> String {
    let mut out = text.as_bytes().to_vec();
    let mut i = 0;
    while i < out.len() {
        if i + 1 < out.len() && &out[i..i + 2] == b"//" {
            while i < out.len() && out[i] != b'\n' {
                out[i] = b' ';
                i += 1;
            }
        } else if i + 1 < out.len() && &out[i..i + 2] == b"/*" {
            out[i] = b' ';
            out[i + 1] = b' ';
            i += 2;
            while i < out.len() {
                if i + 1 < out.len() && &out[i..i + 2] == b"*/" {
                    out[i] = b' ';
                    out[i + 1] = b' ';
                    i += 2;
                    break;
                }
                if out[i] != b'\n' {
                    out[i] = b' ';
                }
                i += 1;
            }
        } else if lang != SourceLang::Ptx
            && (out[i] == b'r' || (out[i] == b'b' && i + 1 < out.len() && out[i + 1] == b'r'))
            && (i == 0 || (!out[i - 1].is_ascii_alphanumeric() && out[i - 1] != b'_'))
        {
            let r_pos = if out[i] == b'b' { i + 1 } else { i };
            let mut hashes = 0usize;
            while r_pos + 1 + hashes < out.len() && out[r_pos + 1 + hashes] == b'#' {
                hashes += 1;
            }
            if r_pos + 1 + hashes < out.len() && out[r_pos + 1 + hashes] == b'"' {
                i = r_pos + 2 + hashes;
                while i < out.len() {
                    if out[i] == b'"'
                        && i + 1 + hashes <= out.len()
                        && out[i + 1..i + 1 + hashes].iter().all(|&b| b == b'#')
                    {
                        i += 1 + hashes;
                        break;
                    }
                    i += 1;
                }
            } else {
                i += 1;
            }
        } else if out[i] == b'\'' {
            if lang == SourceLang::Ptx {
                i += 1;
                continue;
            }
            let mut tok_start = i;
            while tok_start > 0
                && (out[tok_start - 1].is_ascii_alphanumeric() || out[tok_start - 1] == b'_')
            {
                tok_start -= 1;
            }
            let is_digit_sep = lang != SourceLang::Rust
                && tok_start < i
                && out[tok_start].is_ascii_digit()
                && i + 1 < out.len()
                && out[i + 1].is_ascii_hexdigit();
            if is_digit_sep {
                i += 1;
                continue;
            }
            let mut k = i + 1;
            if k < out.len() && (out[k].is_ascii_alphabetic() || out[k] == b'_') {
                while k < out.len() && (out[k].is_ascii_alphanumeric() || out[k] == b'_') {
                    k += 1;
                }
            }
            let mut j = i + 1;
            let mut close = None;
            while j < out.len() && out[j] != b'\n' {
                if out[j] == b'\\' {
                    j = (j + 2).min(out.len());
                } else if out[j] == b'\'' {
                    close = Some(j + 1);
                    break;
                } else {
                    j += 1;
                }
            }
            let is_char_literal = match (lang, close) {
                (_, None) | (SourceLang::Ptx, _) => false,
                (SourceLang::Cuda, Some(_)) => true,
                (SourceLang::Rust, Some(end)) => {
                    let body = &out[i + 1..end - 1];
                    body.first() == Some(&b'\\')
                        || std::str::from_utf8(body).is_ok_and(|s| s.chars().count() == 1)
                }
                (SourceLang::Auto, Some(end)) => {
                    let quote_idx = end - 1;
                    if k == i + 1 || quote_idx == i + 2 {
                        true
                    } else {
                        let tail = &out[k..quote_idx];
                        let line_end = out[end..]
                            .iter()
                            .position(|&b| b == b'\n')
                            .map_or(out.len(), |off| end + off);
                        let rest_line = &out[end..line_end];
                        let unquoted_comment_close = tail.windows(2).any(|w| w == b"/*")
                            && rest_line
                                .windows(2)
                                .position(|w| w == b"*/")
                                .is_some_and(|pos| !rest_line[..pos].contains(&b'\''));
                        !out.get(end).is_some_and(|b| b.is_ascii_alphabetic())
                            && !tail.iter().any(|&b| b == b' ' || b == b'\t')
                            && !tail.windows(2).any(|w| w == b"*/")
                            && !unquoted_comment_close
                    }
                }
            };
            if let Some(end) = close.filter(|_| is_char_literal) {
                i = end;
            } else if k > i + 1 {
                i = k;
            } else {
                i += 1;
            }
        } else if out[i] == b'"' {
            i += 1;
            while i < out.len() {
                if out[i] == b'\\' {
                    i += 1;
                    if i < out.len() {
                        i += 1;
                    }
                } else if out[i] == b'"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
        } else {
            i += 1;
        }
    }
    String::from_utf8(out).unwrap_or_default()
}

fn code_cuda(text: &str) -> String {
    code_for(text, SourceLang::Cuda)
}

fn code_ptx(text: &str) -> String {
    code_for(text, SourceLang::Ptx)
}

fn code_rust(text: &str) -> String {
    code_for(text, SourceLang::Rust)
}

fn code(text: &str) -> String {
    code_for(text, SourceLang::Auto)
}

fn forbidden_with_lang(text: &str, lang: SourceLang) -> Vec<String> {
    let c = code_for(text, lang).to_ascii_lowercase();
    let mut found = BTreeSet::new();
    for word in c.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_') {
        for family in [
            "softmax",
            "softplus",
            "sigmoid",
            "tanh",
            "gelu",
            "silu",
            "uelu_",
            "rope",
            "ropei",
            "rotary",
            "cublas",
            "cudnn",
            "nvrtc",
            "matmul",
            "gemm",
            "wmma",
            "wgmma",
            "tcgen05",
            "mma_sync",
            "uexp_",
            "ulog_",
            "__nv_exp",
            "__nv_log",
            "__nv_tanh",
            "__nv_erf",
        ] {
            if word.contains(family) {
                found.insert(word.to_string());
            }
        }
        if [
            "exp", "expf", "exp2", "exp2f", "expm1", "expm1f", "__expf", "erf", "erff", "erfc",
            "erfcf", "erfinv", "erfinvf", "tanhf", "elu", "sgemm", "dgemm", "hgemm", "gemv",
        ]
        .contains(&word)
        {
            found.insert(word.to_string());
        }
    }
    for instruction in [
        "mma.",
        "wmma.",
        "wgmma.",
        "tcgen05.",
        "ex2.",
        "lg2.",
        "tanh.",
        "sin.approx",
        "cos.approx",
    ] {
        if c.contains(instruction) {
            found.insert(instruction.to_string());
        }
    }
    found.into_iter().collect()
}

fn forbidden(text: &str) -> Vec<String> {
    forbidden_with_lang(text, SourceLang::Auto)
}

fn forbidden_cuda(text: &str) -> Vec<String> {
    forbidden_with_lang(text, SourceLang::Cuda)
}

fn strip_ptx_predicates_and_labels(mut line: &str) -> &str {
    loop {
        line = line.trim_start();
        if let Some(rest) = line.strip_prefix('@') {
            let rest = rest.trim_start();
            let rest = rest.strip_prefix('!').unwrap_or(rest).trim_start();
            let idx = rest
                .find(|c: char| c.is_ascii_whitespace())
                .unwrap_or(rest.len());
            line = &rest[idx..];
            continue;
        }
        if let Some(colon) = line.find(':') {
            let head = &line[..colon];
            let next_is_colon = line[colon + 1..].starts_with(':');
            if !head.is_empty()
                && !next_is_colon
                && head
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
            {
                line = &line[colon + 1..];
                continue;
            }
        }
        break;
    }
    line.trim_start()
}

fn forbidden_ptx(text: &str) -> Vec<String> {
    let mut found: BTreeSet<String> = forbidden_cuda(text).into_iter().collect();
    let cleaned = code_cuda(text);
    for stmt in cleaned.split(['\n', ';', '{', '}']) {
        let stripped = strip_ptx_predicates_and_labels(stmt).to_ascii_lowercase();
        if stripped.is_empty() {
            continue;
        }
        for candidate in std::iter::once(stripped.as_str()).chain(
            stripped
                .split_whitespace()
                .map(strip_ptx_predicates_and_labels),
        ) {
            for prefix in [
                "ex2.",
                "lg2.",
                "sin.",
                "cos.",
                "tanh.",
                "rcp.approx",
                "sqrt.approx",
                "rsqrt.approx",
                "mma.",
                "wmma.",
                "wgmma.",
                "tcgen05.",
            ] {
                if candidate.starts_with(prefix) {
                    found.insert(prefix.to_string());
                }
            }
        }
    }
    found.into_iter().collect()
}

fn forbidden_training_cuda(text: &str) -> Vec<String> {
    let mut found: BTreeSet<String> = forbidden_cuda(text).into_iter().collect();
    let c = code_cuda(text).to_ascii_lowercase();
    for word in c.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_') {
        if [
            "log", "logf", "log2", "log2f", "log10", "log10f", "log1p", "log1pf", "__logf",
            "nvcuda",
        ]
        .contains(&word)
        {
            found.insert(word.to_string());
        }
    }
    found.into_iter().collect()
}

fn cuda_float_literals(cleaned_cuda: &str) -> Vec<String> {
    let bytes = cleaned_cuda.as_bytes();
    let mut found = BTreeSet::new();
    let mut i = 0usize;
    while i < bytes.len() {
        let prev_boundary = i == 0
            || (!bytes[i - 1].is_ascii_alphanumeric()
                && bytes[i - 1] != b'_'
                && bytes[i - 1] != b'.'
                && bytes[i - 1] != b'%');
        if bytes[i] == b'.'
            && prev_boundary
            && bytes.get(i + 1).is_some_and(|b| b.is_ascii_digit())
        {
            let mut k = i + 2;
            while k < bytes.len() && (bytes[k].is_ascii_digit() || bytes[k] == b'\'') {
                k += 1;
            }
            if matches!(bytes.get(k), Some(b'e' | b'E')) {
                k += 1;
                if matches!(bytes.get(k), Some(b'+' | b'-')) {
                    k += 1;
                }
                while k < bytes.len() && (bytes[k].is_ascii_digit() || bytes[k] == b'\'') {
                    k += 1;
                }
            }
            while k < bytes.len() && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_') {
                k += 1;
            }
            found.insert(cleaned_cuda[i..k].replace('\'', "").to_ascii_lowercase());
            i = k;
            continue;
        }
        if bytes[i].is_ascii_digit() && prev_boundary {
            if bytes[i] == b'0' && matches!(bytes.get(i + 1), Some(b'x' | b'X')) {
                let mut k = i + 2;
                while k < bytes.len() && (bytes[k].is_ascii_hexdigit() || bytes[k] == b'\'') {
                    k += 1;
                }
                let has_radix =
                    bytes.get(k) == Some(&b'.') && bytes.get(k + 1) != Some(&b'.');
                if has_radix {
                    k += 1;
                    while k < bytes.len()
                        && (bytes[k].is_ascii_hexdigit() || bytes[k] == b'\'')
                    {
                        k += 1;
                    }
                }
                let has_bin_exp = matches!(bytes.get(k), Some(b'p' | b'P'));
                if has_bin_exp {
                    k += 1;
                    if matches!(bytes.get(k), Some(b'+' | b'-')) {
                        k += 1;
                    }
                    while k < bytes.len() && (bytes[k].is_ascii_digit() || bytes[k] == b'\'') {
                        k += 1;
                    }
                }
                while k < bytes.len() && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_')
                {
                    k += 1;
                }
                if has_radix || has_bin_exp {
                    found.insert(cleaned_cuda[i..k].replace('\'', "").to_ascii_lowercase());
                }
                i = k;
                continue;
            }
            if bytes[i] == b'0' && matches!(bytes.get(i + 1), Some(b'f' | b'F' | b'd' | b'D')) {
                let mut k = i + 2;
                while k < bytes.len() && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_')
                {
                    k += 1;
                }
                found.insert(cleaned_cuda[i..k].to_ascii_lowercase());
                i = k;
                continue;
            }
            let mut k = i;
            while k < bytes.len() && (bytes[k].is_ascii_digit() || bytes[k] == b'\'') {
                k += 1;
            }
            if bytes.get(k) == Some(&b'.')
                && bytes.get(k + 1) != Some(&b'.')
                && !bytes.get(k + 1).is_some_and(|b| {
                    (b.is_ascii_alphabetic() || *b == b'_')
                        && !matches!(b, b'e' | b'E' | b'f' | b'F' | b'l' | b'L')
                })
            {
                k += 1;
                while k < bytes.len() && (bytes[k].is_ascii_digit() || bytes[k] == b'\'') {
                    k += 1;
                }
                if matches!(bytes.get(k), Some(b'e' | b'E')) {
                    k += 1;
                    if matches!(bytes.get(k), Some(b'+' | b'-')) {
                        k += 1;
                    }
                    while k < bytes.len() && (bytes[k].is_ascii_digit() || bytes[k] == b'\'') {
                        k += 1;
                    }
                }
                while k < bytes.len() && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_')
                {
                    k += 1;
                }
                found.insert(cleaned_cuda[i..k].replace('\'', "").to_ascii_lowercase());
                i = k;
                continue;
            }
            if matches!(bytes.get(k), Some(b'e' | b'E')) {
                let mut m = k + 1;
                if matches!(bytes.get(m), Some(b'+' | b'-')) {
                    m += 1;
                }
                if bytes.get(m).is_some_and(|b| b.is_ascii_digit()) {
                    while m < bytes.len() && (bytes[m].is_ascii_digit() || bytes[m] == b'\'') {
                        m += 1;
                    }
                    while m < bytes.len()
                        && (bytes[m].is_ascii_alphanumeric() || bytes[m] == b'_')
                    {
                        m += 1;
                    }
                    found.insert(cleaned_cuda[i..m].replace('\'', "").to_ascii_lowercase());
                    i = m;
                    continue;
                }
            }
            if matches!(bytes.get(k), Some(b'f' | b'F')) {
                while k < bytes.len() && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_')
                {
                    k += 1;
                }
                found.insert(cleaned_cuda[i..k].replace('\'', "").to_ascii_lowercase());
                i = k;
                continue;
            }
            while k < bytes.len() && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_') {
                k += 1;
            }
            i = k;
            continue;
        }
        i += 1;
    }
    found.into_iter().collect()
}

fn forbidden_discrete_cuda(text: &str) -> Vec<String> {
    let mut found: BTreeSet<String> = forbidden_ptx(text).into_iter().collect();
    let cleaned = code_cuda(text);
    for lit in cuda_float_literals(&cleaned) {
        found.insert(lit);
    }
    let c = cleaned.to_ascii_lowercase();
    for word in c.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_') {
        if [
            "float",
            "double",
            "__half",
            "__nv_bfloat16",
            "f16",
            "f16x2",
            "f32",
            "f64",
            "bf16",
            "bf16x2",
            "tf32",
            "e4m3",
            "e5m2",
            "mma",
            "wmma",
            "wgmma",
            "tcgen05",
            "tensor",
            "tensormap",
            "log",
            "logf",
            "log2",
            "log2f",
            "log10",
            "log10f",
            "log1p",
            "log1pf",
        ]
        .contains(&word)
        {
            found.insert(word.to_string());
        }
    }
    for instruction in [
        "sin.approx",
        "cos.approx",
        "rcp.approx",
        "sqrt.approx",
        "rsqrt.approx",
    ] {
        if c.contains(instruction) {
            found.insert(instruction.to_string());
        }
    }
    found.into_iter().collect()
}

fn check_training_build_sources(training_build: &str) -> Result<(), String> {
    if training_build.contains("native_context")
        || training_build.contains("selected_potential")
        || training_build.contains("historical")
        || training_build.contains("quarantined")
        || training_build.contains("glob(")
        || training_build.contains("read_dir(")
        || training_build.contains("bindgen_cuda")
    {
        return Err(
            "crates/uor-r4-training/build.rs must not reference quarantined/historical modules or discover extra CUDA files"
                .into(),
        );
    }
    let mod_start = training_build
        .find("const MODULES:")
        .ok_or("crates/uor-r4-training/build.rs missing const MODULES")?;
    let tail = &training_build[mod_start..];
    let begin = tail
        .find("= [")
        .ok_or("crates/uor-r4-training/build.rs MODULES is not an explicit array")?
        + 3;
    let end = tail[begin..]
        .find("];")
        .ok_or("crates/uor-r4-training/build.rs unclosed MODULES")?
        + begin;
    let modules_body = &tail[begin..end];
    if training_build[..mod_start].contains("sources: &[")
        || tail[end..].contains("sources:")
    {
        return Err(
            "crates/uor-r4-training/build.rs declares Module.sources outside const MODULES"
                .into(),
        );
    }
    if modules_body.matches("Module").count() != 3 {
        return Err(
            "crates/uor-r4-training/build.rs const MODULES must contain exactly 3 Module entries"
                .into(),
        );
    }
    let expected_training_cu: BTreeSet<String> =
        TRAINING_COMPILED_CU.iter().map(|s| s.to_string()).collect();
    let mut declared_sources = BTreeSet::new();
    let mut module_count = 0usize;
    let mut scan = modules_body;
    while let Some(pos) = scan.find("sources:") {
        let after_key = scan[pos + "sources:".len()..].trim_start();
        let after_open = after_key.strip_prefix("&[").ok_or(
            "crates/uor-r4-training/build.rs Module.sources must immediately start with a literal &[...] slice",
        )?;
        let arr_close = after_open
            .find(']')
            .ok_or("crates/uor-r4-training/build.rs unclosed Module.sources slice")?;
        let slice_body = &after_open[..arr_close];
        let mut entries = Vec::new();
        for raw_elem in slice_body.split(',') {
            let elem = raw_elem.trim();
            if elem.is_empty() {
                continue;
            }
            if !elem.starts_with('"')
                || !elem.ends_with('"')
                || elem.len() < 2
                || elem[1..elem.len() - 1].contains('"')
                || elem[1..elem.len() - 1].contains('\\')
            {
                return Err(format!(
                    "crates/uor-r4-training/build.rs Module.sources element `{elem}` must be a plain string literal"
                ));
            }
            entries.push(elem[1..elem.len() - 1].to_string());
        }
        if entries.is_empty() {
            return Err("crates/uor-r4-training/build.rs empty Module.sources slice".into());
        }
        for src_rel in entries {
            if !src_rel.starts_with("src/") || src_rel.contains("..") {
                return Err(format!(
                    "crates/uor-r4-training/build.rs Module.sources path `{src_rel}` outside src/"
                ));
            }
            let repo_rel = format!("crates/uor-r4-training/{src_rel}");
            if !expected_training_cu.contains(&repo_rel) {
                return Err(format!(
                    "crates/uor-r4-training/build.rs Module.sources path `{repo_rel}` not in TRAINING_COMPILED_CU"
                ));
            }
            declared_sources.insert(repo_rel);
        }
        module_count += 1;
        scan = &after_open[arr_close + 1..];
    }
    if module_count != 3 {
        return Err(format!(
            "crates/uor-r4-training/build.rs expected 3 Module.sources entries, found {module_count}"
        ));
    }
    if declared_sources != expected_training_cu {
        return Err(format!(
            "crates/uor-r4-training/build.rs declared Module.sources set differs from TRAINING_COMPILED_CU: {declared_sources:?}"
        ));
    }
    for lit in quoted(training_build)? {
        if (lit.ends_with(".cu") || lit.ends_with(".cuh")) && lit != ".cu" {
            let repo_rel = format!("crates/uor-r4-training/{lit}");
            if !expected_training_cu.contains(&repo_rel) {
                return Err(format!(
                    "crates/uor-r4-training/build.rs references unapproved CUDA source literal `{lit}`"
                ));
            }
        }
    }
    Ok(())
}

fn forbidden_runtime_compiler_rust(cleaned_rust: &str) -> Vec<String> {
    let mut found = BTreeSet::new();
    for needle in [
        "nvrtc::",
        "compile_ptx",
        "cuLinkCreate",
        "cuLinkAddData",
        "cuLinkAddFile",
        "cuLinkComplete",
        "\"nvcc\"",
        "/nvcc\"",
        "\"NVCC\"",
        "\"UOR_NVCC_ARCH\"",
        "\"ptxas\"",
        "/ptxas\"",
        "\"PTXAS\"",
        "\"nvlink\"",
        "/nvlink\"",
        "\"NVLINK\"",
        "\"cudafe++\"",
        "\"cicc\"",
        "\"-ptx\"",
        "\"--ptx\"",
        "\"-cubin\"",
        "\"--cubin\"",
        "\"-fatbin\"",
        "\"--fatbin\"",
    ] {
        if cleaned_rust.contains(needle) {
            found.insert(needle.to_string());
        }
    }
    for word in cleaned_rust.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_') {
        if ["nvcc", "NVCC", "UOR_NVCC_ARCH"].contains(&word) {
            found.insert(word.to_string());
        }
    }
    found.into_iter().collect()
}

fn check_standalone_instrument_inventory(repo: &Path) -> Result<(), String> {
    let labs_dir = repo.join("docs/labs");
    let mut discovered = BTreeSet::new();
    let mut discovered_build_scripts = BTreeSet::new();
    for entry in fs::read_dir(&labs_dir).map_err(|e| format!("{}: {e}", labs_dir.display()))? {
        let lab_path = entry.map_err(|e| e.to_string())?.path();
        if !lab_path.is_dir() {
            continue;
        }
        let instr_dir = lab_path.join("instrument");
        if !instr_dir.exists() {
            continue;
        }
        let manifest_path = instr_dir.join("Cargo.toml");
        if !manifest_path.is_file() {
            return Err(format!(
                "{}: standalone instrument directory missing Cargo.toml",
                instr_dir.display()
            ));
        }
        let rel = manifest_path
            .strip_prefix(repo)
            .unwrap_or(&manifest_path)
            .to_string_lossy()
            .replace('\\', "/");
        discovered.insert(rel);
        let build_rs = instr_dir.join("build.rs");
        if build_rs.exists() {
            let build_rel = build_rs
                .strip_prefix(repo)
                .unwrap_or(&build_rs)
                .to_string_lossy()
                .replace('\\', "/");
            discovered_build_scripts.insert(build_rel);
        }
    }
    let expected: BTreeSet<String> = STANDALONE_INSTRUMENT_MANIFESTS
        .iter()
        .map(|s| s.to_string())
        .collect();
    if discovered != expected {
        return Err(format!(
            "standalone instrument manifest inventory differs from frozen 5-root set: {discovered:?}"
        ));
    }
    let expected_build_scripts: BTreeSet<String> = [
        "docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/build.rs".to_string(),
    ]
    .into_iter()
    .collect();
    if discovered_build_scripts != expected_build_scripts {
        return Err(format!(
            "standalone instrument build.rs inventory differs: {discovered_build_scripts:?}"
        ));
    }
    Ok(())
}

fn global_entries(cleaned_code: &str, require_extern_c: bool) -> Result<Vec<String>, String> {
    let bytes = cleaned_code.as_bytes();
    let needle = b"__global__";
    let mut entries = Vec::new();
    let mut i = 0usize;
    while i + needle.len() <= bytes.len() {
        if &bytes[i..i + needle.len()] == needle {
            let prev_ok =
                i == 0 || (!bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_');
            let next_pos = i + needle.len();
            let next_ok = next_pos == bytes.len()
                || (!bytes[next_pos].is_ascii_alphanumeric() && bytes[next_pos] != b'_');
            if prev_ok && next_ok {
                if require_extern_c {
                    let before = cleaned_code[..i].trim_end();
                    let Some(before_c) = before.strip_suffix("\"C\"") else {
                        return Err("CUDA __global__ kernel missing extern \"C\" linkage".into());
                    };
                    let before_extern = before_c.trim_end();
                    let Some(prefix) = before_extern.strip_suffix("extern") else {
                        return Err("CUDA __global__ kernel missing extern \"C\" linkage".into());
                    };
                    if prefix
                        .as_bytes()
                        .last()
                        .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
                    {
                        return Err("CUDA __global__ kernel missing extern \"C\" linkage".into());
                    }
                }
                let mut rest = cleaned_code[next_pos..].trim_start();
                while rest.starts_with("__launch_bounds__") || rest.starts_with("__maxnreg__") {
                    let open = rest
                        .find('(')
                        .ok_or("malformed CUDA launch attribute before kernel name")?;
                    let mut depth = 0usize;
                    let mut end_attr = None;
                    for (idx, ch) in rest[open..].char_indices() {
                        match ch {
                            '(' => depth += 1,
                            ')' => {
                                depth -= 1;
                                if depth == 0 {
                                    end_attr = Some(open + idx + 1);
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    let end_idx =
                        end_attr.ok_or("unclosed CUDA launch attribute before kernel name")?;
                    rest = rest[end_idx..].trim_start();
                }
                let after_void = rest
                    .strip_prefix("void")
                    .ok_or("expected void return type after __global__")?;
                if !after_void
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_whitespace())
                {
                    return Err("expected whitespace after __global__ void".into());
                }
                let after_void = after_void.trim_start();
                let paren = after_void
                    .find('(')
                    .ok_or("missing '(' after __global__ void kernel name")?;
                let name = after_void[..paren].trim();
                if name.is_empty()
                    || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                {
                    return Err(format!("invalid __global__ kernel identifier: `{name}`"));
                }
                entries.push(name.to_string());
                i = next_pos;
                continue;
            }
        }
        i += 1;
    }
    Ok(entries)
}

fn quoted(text: &str) -> Result<Vec<String>, String> {
    let mut parts = text.split('"');
    let mut out = Vec::new();
    parts.next();
    while let Some(value) = parts.next() {
        out.push(value.to_string());
        if parts.next().is_none() {
            return Err("unclosed quoted field".into());
        }
    }
    Ok(out)
}

fn section<'a>(text: &'a str, header: &str) -> Result<&'a str, String> {
    let start = text
        .find(header)
        .ok_or_else(|| format!("missing {header}"))?
        + header.len();
    let tail = &text[start..];
    Ok(tail.split("\n[").next().unwrap_or(tail))
}

fn array_field(text: &str, key: &str) -> Result<Vec<String>, String> {
    let start = text
        .lines()
        .scan(0usize, |offset, line| {
            let here = *offset;
            *offset += line.len() + 1;
            Some((here, line))
        })
        .find(|(_, line)| line.trim_start().starts_with(&format!("{key} =")))
        .map(|(offset, _)| offset)
        .ok_or_else(|| format!("missing {key} array"))?;
    let tail = &text[start..];
    let begin = tail
        .find('[')
        .ok_or_else(|| format!("{key} is not an array"))?;
    let end = tail[begin..]
        .find(']')
        .ok_or_else(|| format!("unclosed {key} array"))?
        + begin;
    quoted(&tail[begin + 1..end])
}

fn verify(
    repo: &Path,
    ptx: Option<&Path>,
    imports: Option<&Path>,
    features: Option<&Path>,
) -> Result<Vec<String>, String> {
    let vendor = repo.join("third_party/candle-kernels-0.9.2");
    let build = read(&vendor.join("build.rs"))?;
    let list_start = build
        .find("const MODULES:")
        .ok_or("missing explicit MODULES constant")?;
    let list = &build[list_start..];
    let begin = list
        .find("= &[")
        .ok_or("MODULES is not an explicit array")?
        + 4;
    let end = list[begin..].find("];").ok_or("unclosed MODULES")? + begin;
    let actual = quoted(&list[begin..end])?;
    if actual != MODULES {
        return Err(format!("compiler module list differs: {actual:?}"));
    }
    if build.contains("glob(") || build.contains("read_dir(") || build.contains("bindgen_cuda") {
        return Err("build must not discover extra CUDA modules".into());
    }
    let mut checked = vec!["explicit nine-module compiler list".into()];
    let mut actual_sources = BTreeSet::new();
    for entry in fs::read_dir(vendor.join("src")).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        let ext = path.extension().and_then(|v| v.to_str()).unwrap_or("");
        if ext != "cu" && ext != "cuh" {
            continue;
        }
        let text = read(&path)?;
        let bad = forbidden_cuda(&text);
        if !bad.is_empty() {
            return Err(format!("{} forbidden code tokens: {bad:?}", path.display()));
        }
        for line in text
            .lines()
            .filter(|l| l.trim_start().starts_with("#include"))
        {
            if line.contains("historical")
                || line.contains("moe")
                || line.contains("conv")
                || line.contains("quantized")
                || line.contains("../")
            {
                return Err(format!(
                    "{} excluded source include: {line}",
                    path.display()
                ));
            }
        }
        if ext == "cu" {
            actual_sources.insert(
                path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }
    let expected: BTreeSet<_> = MODULES.iter().map(|v| v.to_string()).collect();
    if actual_sources != expected {
        return Err(format!("active .cu set differs: {actual_sources:?}"));
    }
    let lib = code_rust(&read(&vendor.join("src/lib.rs"))?);
    if !lib.contains("include!(concat!(env!(\"OUT_DIR\"), \"/ptx.rs\"))") {
        return Err("kernel bindings must include only generated OUT_DIR/ptx.rs".into());
    }
    if lib.contains("mod ffi") || lib.contains("mod moe") || lib.contains("extern \"C\"") {
        return Err("active kernel crate links historical FFI/MOE".into());
    }
    if !build.contains("pub const CONV: &str = \\\"\\\";")
        || !build.contains("pub const QUANTIZED: &str = \\\"\\\";")
    {
        return Err("historical conv/quantized descriptors must be empty".into());
    }
    checked.push("active vendor CUDA source/header token census and no historical FFI".into());
    let training_build = code_rust(&read(&repo.join("crates/uor-r4-training/build.rs"))?);
    check_training_build_sources(&training_build)?;
    let mut training_cu_files = BTreeSet::new();
    for entry in
        fs::read_dir(repo.join("crates/uor-r4-training/src")).map_err(|e| e.to_string())?
    {
        let path = entry.map_err(|e| e.to_string())?.path();
        let ext = path.extension().and_then(|v| v.to_str()).unwrap_or("");
        if ext == "cu" || ext == "cuh" {
            let rel = path
                .strip_prefix(repo)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            training_cu_files.insert(rel);
        }
    }
    let expected_training_cu: BTreeSet<String> =
        TRAINING_COMPILED_CU.iter().map(|s| s.to_string()).collect();
    if training_cu_files != expected_training_cu {
        return Err(format!(
            "crates/uor-r4-training/src CUDA file set differs: {training_cu_files:?}"
        ));
    }
    let historical = read(&repo.join(
        "docs/history/d29-excluded-kernels-2026-10-13/cuda_stack_historical.cu",
    ))?;
    let historical_names = global_entries(&code_cuda(&historical), true)?;
    if historical_names.len() < 30 {
        return Err("historical D29 evidence file lost its kernels".into());
    }
    for rel in TRAINING_COMPILED_CU {
        let cu_path = repo.join(rel);
        let cu_text = read(&cu_path)?;
        let bad_training = forbidden_training_cuda(&cu_text);
        if !bad_training.is_empty() {
            return Err(format!(
                "{} forbidden training CUDA tokens: {bad_training:?}",
                cu_path.display()
            ));
        }
        for line in cu_text
            .lines()
            .filter(|l| l.trim_start().starts_with("#include"))
        {
            if line.contains("historical")
                || line.contains("quarantined")
                || line.contains("native_context")
                || line.contains("selected_potential")
                || line.contains("moe")
                || line.contains("conv")
                || line.contains("quantized")
                || line.contains("../")
            {
                return Err(format!(
                    "{} excluded source include: {line}",
                    cu_path.display()
                ));
            }
        }
        let cleaned = code_cuda(&cu_text);
        let defined_entries = global_entries(&cleaned, false)?;
        for name in &historical_names {
            if defined_entries.contains(name)
                || cleaned.contains(&format!("__global__ void {name}("))
            {
                return Err(format!(
                    "{} defines excluded historical kernel `{name}`",
                    cu_path.display()
                ));
            }
        }
    }
    for (module, evidence) in [
        ("native_context", "native_context_quarantined.cu"),
        ("selected_potential", "selected_potential_quarantined.cu"),
    ] {
        if repo
            .join(format!("crates/uor-r4-training/src/{module}.cu"))
            .exists()
        {
            return Err(format!("quarantined {module}.cu returned to training src"));
        }
        let q_src = read(&repo.join(format!(
            "docs/history/d29-excluded-kernels-2026-10-13/{evidence}"
        )))?;
        let q_names = global_entries(&code_cuda(&q_src), false)?;
        if q_names.is_empty() {
            return Err(format!(
                "quarantined D29 evidence file {evidence} lost its kernels"
            ));
        }
        for rel in TRAINING_COMPILED_CU {
            let cleaned = code_cuda(&read(&repo.join(rel))?);
            let defined_entries = global_entries(&cleaned, false)?;
            for name in &q_names {
                if defined_entries.contains(name)
                    || cleaned.contains(&format!("__global__ void {name}("))
                {
                    return Err(format!(
                        "{rel} defines quarantined dense-contraction kernel `{name}`"
                    ));
                }
            }
        }
    }
    checked.push(
        "training compiled CUDA sources (.cu) D23/D25/D29 token census and historical/quarantined exclusion"
            .into(),
    );
    let mut discrete_cu_files = BTreeSet::new();
    for entry in
        fs::read_dir(repo.join("crates/uor-r4-discrete/src")).map_err(|e| e.to_string())?
    {
        let path = entry.map_err(|e| e.to_string())?.path();
        let ext = path.extension().and_then(|v| v.to_str()).unwrap_or("");
        if ext == "cu" || ext == "cuh" {
            let rel = path
                .strip_prefix(repo)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            discrete_cu_files.insert(rel);
        }
    }
    let expected_discrete_cu: BTreeSet<String> =
        ["crates/uor-r4-discrete/src/discrete.cu".to_string()]
            .into_iter()
            .collect();
    if discrete_cu_files != expected_discrete_cu {
        return Err(format!(
            "crates/uor-r4-discrete/src CUDA file set differs: {discrete_cu_files:?}"
        ));
    }
    let discrete_cu_path = repo.join("crates/uor-r4-discrete/src/discrete.cu");
    let discrete_cu = read(&discrete_cu_path)?;
    for line in discrete_cu
        .lines()
        .filter(|l| l.trim_start().starts_with("#include"))
    {
        if line.contains("historical") || line.contains("quarantined") || line.contains("../") {
            return Err(format!(
                "{} excluded source include: {line}",
                discrete_cu_path.display()
            ));
        }
    }
    let bad_discrete = forbidden_discrete_cuda(&discrete_cu);
    if !bad_discrete.is_empty() {
        return Err(format!(
            "{} forbidden discrete/D32 CUDA tokens: {bad_discrete:?}",
            discrete_cu_path.display()
        ));
    }
    let discrete_code = code_cuda(&discrete_cu);
    let discrete_entries = global_entries(&discrete_code, true)?;
    if discrete_entries != DISCRETE_ENTRIES {
        return Err(format!(
            "{} global entry list differs: {discrete_entries:?}",
            discrete_cu_path.display()
        ));
    }
    let decomp_cu = decomposition_source::generate(&discrete_cu)?;
    let bad_decomp = forbidden_discrete_cuda(&decomp_cu);
    if !bad_decomp.is_empty() {
        return Err(format!(
            "generated discrete_counterfactual_decomposition.cu forbidden discrete/D32 CUDA tokens: {bad_decomp:?}"
        ));
    }
    let decomp_code = code_cuda(&decomp_cu);
    let decomp_entries = global_entries(&decomp_code, true)?;
    if decomp_entries != ["discrete_counterfactual_decomposition"] {
        return Err(format!(
            "generated discrete_counterfactual_decomposition.cu global entry list differs: {decomp_entries:?}"
        ));
    }
    checked.push(
        "discrete CUDA kernel sources (discrete.cu + generated decomposition module) D23/D25/D32 integer-only token and 11+1 entry census"
            .into(),
    );
    check_standalone_instrument_inventory(repo)?;
    for rel in APPROVED_BUILD_SCRIPTS {
        let path = repo.join(rel);
        let cleaned = code_rust(&read(&path)?);
        for rtc in [
            "nvrtc::",
            "compile_ptx",
            "cuLinkCreate",
            "cuLinkAddData",
            "cuLinkAddFile",
            "cuLinkComplete",
        ] {
            if cleaned.contains(rtc) {
                return Err(format!("{} calls runtime compiler `{rtc}`", path.display()));
            }
        }
    }
    let mut rs_stack = vec![
        repo.join("crates/uor-r4-training/src"),
        repo.join("crates/uor-r4-discrete/src"),
    ];
    for manifest_rel in STANDALONE_INSTRUMENT_MANIFESTS {
        let src_rel = manifest_rel.trim_end_matches("/Cargo.toml").to_string() + "/src";
        rs_stack.push(repo.join(src_rel));
    }
    while let Some(dir) = rs_stack.pop() {
        for entry in fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.is_dir() {
                rs_stack.push(path);
            } else if path.extension().and_then(|v| v.to_str()) == Some("rs") {
                let cleaned = code_rust(&read(&path)?);
                let bad_rtc = forbidden_runtime_compiler_rust(&cleaned);
                if !bad_rtc.is_empty() {
                    return Err(format!(
                        "{} calls runtime compiler or subprocess: {bad_rtc:?}",
                        path.display()
                    ));
                }
            }
        }
    }
    checked.push(
        "no runtime CUDA compiler calls or compiler subprocesses across training, discrete, and frozen five-root standalone instrument Rust sources and build scripts"
            .into(),
    );
    let root_manifest = read(&repo.join("Cargo.toml"))?;
    let patches = section(&root_manifest, "[patch.crates-io]")?;
    for (name, directory) in [
        ("candle-core", "third_party/candle-core-0.9.2"),
        ("candle-kernels", "third_party/candle-kernels-0.9.2"),
        ("candle-nn", "third_party/candle-nn-0.9.2"),
        ("cudarc", "third_party/cudarc-0.19.10"),
        ("float8", "third_party/float8-0.6.1"),
    ] {
        let line = patches
            .lines()
            .find(|l| l.starts_with(&format!("{name} =")))
            .ok_or_else(|| format!("missing vendor patch for {name}"))?;
        if !line.contains(directory) {
            return Err(format!("wrong patch path: {line}"));
        }
    }
    for rel in [
        "third_party/candle-core-0.9.2/Cargo.toml",
        "third_party/float8-0.6.1/Cargo.toml",
        "crates/uor-r4-training/Cargo.toml",
        "crates/uor-r4-discrete/Cargo.toml",
        "docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/Cargo.toml",
    ] {
        let manifest = read(&repo.join(rel))?;
        let dep = if manifest.contains("[dependencies.cudarc]") {
            section(&manifest, "[dependencies.cudarc]")?.to_string()
        } else {
            manifest
                .lines()
                .find(|l| l.starts_with("cudarc ="))
                .ok_or_else(|| format!("{rel}: no cudarc dependency"))?
                .to_string()
        };
        if !dep.contains("default-features = false") {
            return Err(format!("{rel}: cudarc defaults not disabled"));
        }
        let explicit = array_field(&dep, "features").or_else(|_| {
            let start = dep
                .find("features = [")
                .ok_or("no explicit cudarc features")?;
            let tail = &dep[start..];
            let end = tail.find(']').ok_or("unclosed features")?;
            quoted(&tail[..end])
        })?;
        for feature in explicit {
            if BANNED_FEATURES.contains(&feature.as_str()) || feature == "default" {
                return Err(format!("{rel}: enabled cudarc/{feature}"));
            }
        }
        let fs = section(&manifest, "[features]")?;
        for feature in array_field(fs, "cuda")? {
            if BANNED_FEATURES
                .iter()
                .any(|b| feature.ends_with(&format!("/{b}")))
                || feature == "cudarc/default"
            {
                return Err(format!("{rel}: cuda enables {feature}"));
            }
        }
    }
    for rel in STANDALONE_INSTRUMENT_MANIFESTS {
        let manifest = read(&repo.join(rel))?;
        let instr_patches = section(&manifest, "[patch.crates-io]")?;
        let cudarc_patch = instr_patches
            .lines()
            .find(|l| l.starts_with("cudarc ="))
            .ok_or_else(|| format!("{rel}: missing vendor patch for cudarc"))?;
        if !cudarc_patch.contains("third_party/cudarc-0.19.10") {
            return Err(format!("{rel}: wrong cudarc patch path: {cudarc_patch}"));
        }
        if manifest.contains("[features]") {
            let fs = section(&manifest, "[features]")?;
            for feature in array_field(fs, "cuda")? {
                if BANNED_FEATURES
                    .iter()
                    .any(|b| feature.ends_with(&format!("/{b}")))
                    || feature == "cudarc/default"
                {
                    return Err(format!("{rel}: cuda enables {feature}"));
                }
            }
        }
    }
    checked.push(
        "vendor patches and direct/standalone-instrument selected CUDA cudarc feature declarations"
            .into(),
    );
    if let Some(dir) = ptx {
        let mut names = BTreeSet::new();
        for entry in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.extension().and_then(|v| v.to_str()) != Some("ptx") {
                continue;
            }
            let text = read(&path)?;
            let module = path
                .file_stem()
                .and_then(|name| name.to_str())
                .ok_or_else(|| format!("{} invalid PTX module name", path.display()))?;
            entry_allowlist::check_ptx_exports(module, &text)?;
            let bad = forbidden_ptx(&text);
            if !bad.is_empty() || !code_ptx(&text).contains(".entry") {
                return Err(format!("{} invalid/forbidden PTX: {bad:?}", path.display()));
            }
            names.insert(
                path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            );
        }
        if names != expected {
            return Err(format!("vendor OUT_DIR PTX set differs: {names:?}"));
        }
        checked.push(
            "supplied nine generated vendor PTX modules and exact frozen source-derived exports"
                .into(),
        );
    }
    if let Some(path) = features {
        let text = read(path)?;
        native_dependency_boundary::verify(&text)?;
        if !text.contains("cudarc") || !text.contains("candle-core") {
            return Err("feature evidence lacks selected CUDA packages".into());
        }
        for b in BANNED_FEATURES {
            if text.contains(&format!("cudarc feature \"{b}\""))
                || text.contains(&format!("cudarc/{b}"))
            {
                return Err(format!("resolved feature evidence enables {b}"));
            }
        }
        checked.push(
            "supplied resolved-feature text (producer/source binding checked separately)".into(),
        );
    }
    if let Some(path) = imports {
        let text = read(path)?;
        if !text.contains("NEEDED") && !text.contains("Symbol table") {
            return Err("expected saved readelf -dW / --dyn-syms output".into());
        }
        let lower = text.to_ascii_lowercase();
        for b in BANNED_FEATURES {
            if lower.contains(b) {
                return Err(format!("binary import evidence contains {b}"));
            }
        }
        checked.push("supplied dynamic-import census; indirect dlopen NOT_PROVEN".into());
    }
    Ok(checked)
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args == ["--self-test"] {
        entry_allowlist::self_test();
        native_dependency_boundary::self_test();
        for bad in [
            "float x=expf(y);",
            "float x=__expf(y);",
            "call __nv_expf;",
            "mma.sync.aligned.m16n8k16;",
            "asm(\"mma.sync.aligned.m16n8k16;\");",
            "wgmma.mma_async.sync.aligned.m64n64k16.f32.f16.f16;",
            "tcgen05.mma.cta_group::1.kind::f16;",
            "@%p1 ex2.approx.ftz.f32 %f1, %f0;",
            "@!%p0 lg2.approx.f32 %f1, %f0;",
            "void uexp_f32(){}",
            "void ulog_f32(){}",
            "void rope_f32(){}",
            "void rotary_kernel(){}",
            "float sigmoid(float x){}",
            "float softplus(float x){}",
            "float y = elu(x);",
            "char q = '\"'; const char* a = \"/*\"; float x = expf(y); const char* b = \"*/\"; char q2 = '\"';",
            "let s = r#\"\\\"/*\"#; let y = expf(x); let t = r#\"*/\"#;",
            "int a = '/*'; float y = expf(x); int b = '*/';",
            "int a = 'a/*'; float y = expf(x); int b = '*/';",
            "int a = 'a+/*'; float y = expf(x); int b = '*/';",
            "fn f<'a>(x: &'a /* ' \" */ u32) { let a = \"/*\"; let y = expf(z); let b = \"*/\"; }",
            "type T<'a> = &'a /* \"\n*/ u32; let a = \"/*\"; let y = expf(x); let b = \"*/\";",
            "int x = 0xdead'0000; // '/*\nfloat y = expf(x);\n// */",
        ] {
            assert!(!forbidden(bad).is_empty(), "missed {bad}");
        }
        assert!(
            forbidden("int a = '/*'; float y = expf(x); int b = '*/';")
                .contains(&"expf".to_string()),
            "multicharacter literal comment-delimiter pair hid expf"
        );
        assert!(
            forbidden("int a = 'a+/*'; float y = expf(x); int b = '*/';")
                .contains(&"expf".to_string()),
            "punctuation multicharacter literal hid expf"
        );
        for clean_lifetime in [
            "fn f<'a>(x: &'a str) { /* expf(x) */ }",
            "fn f<'a /* expf(x) */, 'b>(x: &'a str, y: &'b str) {}",
            "int n = 1'000; /* expf(x) */",
            "int n = 0xdead'beef; /* expf(x) */",
        ] {
            assert!(
                forbidden(clean_lifetime).is_empty() && !code(clean_lifetime).contains("expf"),
                "comment leaked across lifetime/digit-separator handling: {}",
                code(clean_lifetime)
            );
        }
        assert!(forbidden("// expf(x) mma.sync\nfloat x=a+b; /* sigmoid */").is_empty());
        assert!(forbidden("void urelu_f32(int x){}").is_empty());
        for bad_ptx in [
            "  @%p1 ex2.approx.ftz.f32 %f1, %f0;",
            "  $L__BB0_1: @!%p0 lg2.approx.f32 %f1, %f0;",
            "  @%p1 sin.approx.f32 %f1, %f0;",
            "  $L__BB0_1: @!%p0 cos.approx.f32 %f1, %f0;",
            "  $L__BB0_2: @%p0 rcp.approx.f32 %f1, %f0;",
            "  $L__BB0_3: sqrt.approx.f32 %f1, %f0;",
            "  rsqrt.approx.f32 %f1, %f0;",
            "  add.u32 %r1, %r2, 1; @%p0 ex2.approx.ftz.f32 %f1, %f0;",
            "  add.u32 %r1, %r2, 1; $L1: @!%p0 lg2.approx.f32 %f1, %f0;",
            "  .visible .entry k() { add.u32 %r1, %r2, 1; @%p1 sin.approx.f32 %f1, %f0; }",
            "  .visible .entry k() { $L__BB0_1: @!%p0 cos.approx.f32 %f1, %f0; ret; }",
            "  .visible .entry k() { @ %p0 tanh.approx.f32 %f1, %f0; ret; }",
            "  add.u32 %r1, %r2, 1; @%p0 rcp.approx.f32 %f1, %f0;",
            "  .visible .entry k() { add.u32 %r1, %r2, 1; $L__BB0_3: sqrt.approx.f32 %f1, %f0; }",
            "  add.u32 %r1, %r2, 1; $L__BB0_4: @!%p0 rsqrt.approx.f32 %f1, %f0;",
            "  $L__BB0_4: @!%p1 wgmma.mma_async.sync.aligned.m64n64k16.f32.f16.f16;",
            "  add.u32 %r1, %r2, 1; tcgen05.mma.cta_group::1.kind::f16;",
        ] {
            assert!(
                !forbidden_ptx(bad_ptx).is_empty(),
                "missed predicated/labeled/statement PTX instruction: {bad_ptx}"
            );
        }
        assert!(
            forbidden_ptx(
                "  .visible .entry k() { $L__BB0_1: @%p0 mul.lo.u32 %r1, %r2, 8; add.u32 %r3, %r1, 1; ret; }"
            )
            .is_empty()
        );
        for bad_training in [
            "float y = logf(x);",
            "float y = log1pf(x);",
            "using namespace nvcuda;",
        ] {
            assert!(
                !forbidden_training_cuda(bad_training).is_empty(),
                "missed training CUDA violation: {bad_training}"
            );
        }
        for bad_discrete in [
            "extern \"C\" __global__ void discrete_init(float* x) {}",
            "__half h;",
            "__nv_bfloat16 b;",
            "auto x = 1.0f; auto y = 2.0; x += y;",
            "decltype(1.0f) x = 0;",
            "auto z = .5f;",
            "auto w = 1e-3;",
            "auto h = 0x1.0p0f;",
            "  @%p1 sin.approx.f32 %f1, %f0;",
            "  $L__BB0_1: @!%p0 cos.approx.f32 %f1, %f0;",
            "  add.u32 %r1, %r2, 1; @%p0 rcp.approx.f32 %f1, %f0;",
            "  .visible .entry k() { $L__BB0_3: sqrt.approx.f32 %f1, %f0; }",
            "  add.u32 %r1, %r2, 1; $L__BB0_4: rsqrt.approx.f32 %f1, %f0;",
        ] {
            assert!(
                !forbidden_discrete_cuda(bad_discrete).is_empty(),
                "missed discrete D32 violation: {bad_discrete}"
            );
        }
        assert!(
            forbidden_discrete_cuda(
                "extern \"C\" __global__ void discrete_init(unsigned int* x) { unsigned int s = 0x9e3779b9u; x[0] = s ^ 1u; }"
            )
            .is_empty()
        );
        let valid_training_build = "const MODULES: [Module; 3] = [\n\
            Module { sources: &[\"src/cuda_stack.cu\", \"src/spiralcore_fbs.cu\"], ptx: \"cuda_stack_f32.ptx\", options: &PRECISE },\n\
            Module { sources: &[\"src/cuda_stack.cu\", \"src/spiralcore_fbs.cu\"], ptx: \"cuda_stack_bf16.ptx\", options: &PRECISE },\n\
            Module { sources: &[\"src/starling_native.cu\"], ptx: \"starling_native.ptx\", options: &[\"--fmad=false\"] },\n\
        ];";
        assert!(
            check_training_build_sources(&code_rust(valid_training_build)).is_ok(),
            "valid training build.rs Module.sources must pass"
        );
        let historical_training_build = valid_training_build.replace(
            "\"src/cuda_stack.cu\"",
            "\"docs/history/d29-excluded-kernels-2026-10-13/cuda_stack_historical.cu\"",
        );
        assert!(
            check_training_build_sources(&code_rust(&historical_training_build)).is_err(),
            "historical Module.sources path must fail training build check"
        );
        let unapproved_training_build =
            valid_training_build.replace("\"src/starling_native.cu\"", "\"src/extra.cu\"");
        assert!(
            check_training_build_sources(&code_rust(&unapproved_training_build)).is_err(),
            "unapproved src/extra.cu Module.sources path must fail training build check"
        );
        let nonliteral_training_build = "const INPUTS: &[&str] = &[concat!(\"src/\", \"extra\", \".cu\")];\n\
            const MODULES: [Module; 3] = [\n\
                Module { sources: INPUTS, ptx: \"a.ptx\", options: &[\"src/cuda_stack.cu\", \"src/spiralcore_fbs.cu\"] },\n\
                Module { sources: &[\"src/cuda_stack.cu\", \"src/spiralcore_fbs.cu\"], ptx: \"b.ptx\", options: &[] },\n\
                Module { sources: &[\"src/starling_native.cu\"], ptx: \"c.ptx\", options: &[] },\n\
            ];";
        assert!(
            check_training_build_sources(&code_rust(nonliteral_training_build)).is_err(),
            "nonliteral Module.sources variable must not consume later options array"
        );
        let concat_training_build = valid_training_build.replace(
            "\"src/starling_native.cu\"",
            "concat!(\"src/\", \"starling_native.cu\")",
        );
        assert!(
            check_training_build_sources(&code_rust(&concat_training_build)).is_err(),
            "nonliteral concat! element inside Module.sources must fail"
        );
        for bad_rtc_rust in [
            "std::process::Command::new(\"nvcc\").arg(\"-ptx\");",
            "let c = std::env::var(\"NVCC\").unwrap_or_else(|_| \"nvcc\".to_string()); std::process::Command::new(&c);",
            "std::process::Command::new(\"ptxas\").arg(\"-arch=sm_80\");",
            "cudarc::nvrtc::compile_ptx(\"extern \\\"C\\\" __global__ void k() {}\");",
        ] {
            assert!(
                !forbidden_runtime_compiler_rust(&code_rust(bad_rtc_rust)).is_empty(),
                "missed runtime compiler subprocess/API in Rust: {bad_rtc_rust}"
            );
        }
        assert!(
            forbidden_runtime_compiler_rust(&code_rust(
                "// nvcc ran at build time in build.rs\nlet ptx = Ptx::from_file(&cubin_path); let r = parse_ptxas(log, \"discrete_counterfactual\");"
            ))
            .is_empty(),
            "admitted cubin loader and ptxas log parser must pass"
        );
        let multiline_decl = "extern \"C\" __global__ void\nextra(unsigned int* x) { x[0] = 0u; }";
        assert_eq!(
            global_entries(&code(multiline_decl), true).expect("valid multiline entry"),
            vec!["extra".to_string()]
        );
        assert!(
            global_entries(
                &code("__global__ void\nextra(unsigned int* x) { x[0] = 0u; }"),
                true
            )
            .is_err(),
            "missing extern \"C\" must be rejected"
        );
        let mutated_discrete = format!(
            "{}\nextern \"C\" __global__ void\nextra(unsigned int* x) {{ x[0] = 0u; }}\n",
            DISCRETE_ENTRIES
                .iter()
                .map(|name| format!(
                    "extern \"C\" __global__ void {name}(unsigned int* x) {{ x[0] = 0u; }}"
                ))
                .collect::<Vec<_>>()
                .join("\n")
        );
        let mutated_entries =
            global_entries(&code(&mutated_discrete), true).expect("mutated discrete entries");
        assert_ne!(
            mutated_entries, DISCRETE_ENTRIES,
            "multiline extra kernel must fail exact 11-entry census"
        );
        println!("PASS: missing, renamed-extra, duplicate and architecture-conditioned PTX exports rejected by frozen source inventory; planted exponential, squash and tensor-core tokens rejected; comments excluded. Static checker only.");
        return;
    }
    let mut repo = PathBuf::from(".");
    let mut ptx = None;
    let mut imports = None;
    let mut features = None;
    let mut i = 0;
    while i < args.len() {
        if !["--repo", "--ptx-dir", "--binary-imports", "--features"].contains(&args[i].as_str())
            || i + 1 >= args.len()
        {
            eprintln!("usage: check-native-cuda-vendor [--repo PATH] [--ptx-dir VENDOR_OUT_DIR] [--binary-imports FILE] [--features FILE] | --self-test");
            process::exit(2);
        }
        let value = PathBuf::from(&args[i + 1]);
        match args[i].as_str() {
            "--repo" => repo = value,
            "--ptx-dir" => ptx = Some(value),
            "--binary-imports" => imports = Some(value),
            "--features" => features = Some(value),
            _ => {}
        }
        i += 2;
    }
    match verify(
        &repo,
        ptx.as_deref(),
        imports.as_deref(),
        features.as_deref(),
    ) {
        Ok(checks) => {
            for check in checks {
                println!("PASS: {check}");
            }
            println!("Static scope PASS. Runtime qualification UNVERIFIED: run frozen named GPU controls separately; this tool runs no model. No proof against arbitrary renamed/semantic equivalents. Omitted optional artifacts NOT_CHECKED.");
        }
        Err(error) => {
            eprintln!("FAIL: {error}");
            process::exit(1);
        }
    }
}
