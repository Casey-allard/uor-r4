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

#[path = "../third_party/candle-kernels-0.9.2/entry_allowlist.rs"]
mod entry_allowlist;
mod native_dependency_boundary;

const MODULES: [&str; 9] = [
    "affine", "binary", "cast", "fill", "indexing", "reduce", "sort", "ternary", "unary",
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

// Preserve offsets/newlines while excluding comments. Keep strings: CUDA
// inline PTX assembly is executable even when spelled inside a string.
fn code(text: &str) -> String {
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

fn forbidden(text: &str) -> Vec<String> {
    let c = code(text).to_ascii_lowercase();
    let mut found = BTreeSet::new();
    for word in c.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_') {
        for family in [
            "softmax", "softplus", "sigmoid", "tanh", "gelu", "silu", "cublas", "cudnn", "nvrtc",
            "matmul", "gemm", "wmma", "mma_sync", "uexp_", "__nv_exp",
        ] {
            if word.contains(family) {
                found.insert(word.to_string());
            }
        }
        if ["exp", "expf", "exp2", "exp2f", "expm1", "expm1f", "__expf"].contains(&word) {
            found.insert(word.to_string());
        }
    }
    for instruction in ["mma.", "wmma.", "ex2.", "tanh."] {
        if c.contains(instruction) {
            found.insert(instruction.to_string());
        }
    }
    found.into_iter().collect()
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
        let bad = forbidden(&text);
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
    let lib = code(&read(&vendor.join("src/lib.rs"))?);
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
    checked.push("vendor patches and direct selected CUDA cudarc feature declarations".into());
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
            let bad = forbidden(&text);
            if !bad.is_empty() || !code(&text).contains(".entry") {
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
            "void uexp_f32(){}",
            "float sigmoid(float x){}",
        ] {
            assert!(!forbidden(bad).is_empty(), "missed {bad}");
        }
        assert!(forbidden("// expf(x) mma.sync\nfloat x=a+b; /* sigmoid */").is_empty());
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
