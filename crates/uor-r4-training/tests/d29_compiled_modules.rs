//! D29 interlock: no banned mechanism in any compiled module, and no runtime
//! CUDA compiler (owner, 13 October 2026).
//!
//! Host checks run everywhere: every source build.rs compiles carries no
//! exponential-family, normalised-exponential, squashing or tensor-core token
//! (comments excluded), defines no excluded historical kernel, and nothing in
//! the crate calls a runtime compiler. With the `cuda` feature the built PTX
//! itself is checked for the instructions those operations compile to.

use std::fs;
use std::path::{Path, PathBuf};

/// The sources build.rs compiles (keep in step with its `MODULES`).
const COMPILED: [&str; 3] = [
    "src/cuda_stack.cu",
    "src/spiralcore_fbs.cu",
    "src/starling_native.cu",
];

fn crate_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

/// The source with `//` and `/* */` comments blanked.
fn code(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"//") {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
        } else if bytes[i..].starts_with(b"/*") {
            i += 2;
            while i + 1 < bytes.len() && !bytes[i..].starts_with(b"*/") {
                i += 1;
            }
            i += 2;
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

/// Whole identifier `word` occurs in `text` (ASCII identifier boundaries).
fn has_word(text: &str, word: &str) -> bool {
    let ident = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let bytes = text.as_bytes();
    let mut from = 0;
    while let Some(at) = text[from..].find(word) {
        let start = from + at;
        let end = start + word.len();
        let before = start == 0 || !ident(bytes[start - 1]);
        let after = end >= bytes.len() || !ident(bytes[end]);
        if before && after {
            return true;
        }
        from = start + 1;
    }
    false
}

/// Banned identifiers, assembled so this file does not match its own text.
fn banned_words() -> Vec<String> {
    [
        ["ex", "p"],
        ["ex", "pf"],
        ["__ex", "pf"],
        ["ex", "p2"],
        ["ex", "p2f"],
        ["lo", "g"],
        ["lo", "gf"],
        ["lo", "g1p"],
        ["lo", "g1pf"],
        ["__lo", "gf"],
        ["ta", "nh"],
        ["ta", "nhf"],
        ["er", "f"],
        ["er", "ff"],
        ["sig", "moid_f"],
        ["soft", "plus_d"],
        ["ge", "lu_value"],
        ["wm", "ma"],
        ["mma", "_sync"],
        ["nvc", "uda"],
        ["ro", "pe"],
        ["ro", "pei"],
        ["rot", "ary"],
    ]
    .iter()
    .map(|parts| parts.concat())
    .collect()
}

#[test]
fn compiled_sources_carry_no_banned_operation() {
    let substring = [
        ["soft", "max"].concat(),
        ["sig", "moid"].concat(),
        ["ge", "lu"].concat(),
    ];
    for source in COMPILED {
        let text = code(&fs::read_to_string(crate_path(source)).expect(source)).to_lowercase();
        for word in banned_words() {
            assert!(!has_word(&text, &word), "{source} contains `{word}`");
        }
        for fragment in &substring {
            assert!(
                !text.contains(fragment.as_str()),
                "{source} contains `{fragment}`"
            );
        }
    }
}

#[test]
fn no_excluded_historical_kernel_is_compiled() {
    let historical = fs::read_to_string(crate_path(
        "../../docs/history/d29-excluded-kernels-2026-10-13/cuda_stack_historical.cu",
    ))
    .expect("D29 evidence file");
    let names: Vec<&str> = historical
        .split("extern \"C\" __global__ void ")
        .skip(1)
        .filter_map(|rest| rest.split('(').next())
        .map(str::trim)
        .collect();
    assert!(names.len() >= 30, "evidence file lost its kernels");
    for source in COMPILED {
        let text = code(&fs::read_to_string(crate_path(source)).expect(source));
        for name in &names {
            let definition = format!("__global__ void {name}(");
            assert!(
                !text.contains(&definition),
                "{source} defines excluded `{name}`"
            );
        }
    }
}

#[test]
fn nothing_in_the_crate_calls_a_runtime_compiler() {
    let forbidden = [["nv", "rtc::"].concat(), ["compile", "_ptx"].concat()];
    let mut stack = vec![crate_path("src")];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("src dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let text = code(&fs::read_to_string(&path).expect("source"));
                for word in &forbidden {
                    assert!(
                        !text.contains(word.as_str()),
                        "{} uses `{word}`",
                        path.display()
                    );
                }
            }
        }
    }
}

/// The built PTX carries none of the instructions exp/log/tanh or a
/// tensor-core contraction compile to.
#[cfg(feature = "cuda")]
#[test]
fn built_ptx_modules_carry_no_banned_instruction() {
    let out = Path::new(env!("OUT_DIR"));
    let modules = [
        "cuda_stack_f32.ptx",
        "cuda_stack_bf16.ptx",
        "starling_native.ptx",
    ];
    let instructions = [
        ["ex2", "."].concat(),
        ["lg2", "."].concat(),
        ["tanh", "."].concat(),
        ["mma", "."].concat(),
        ["wm", "ma."].concat(),
    ];
    for module in modules {
        let ptx = fs::read_to_string(out.join(module)).expect(module);
        assert!(ptx.contains(".entry"), "{module} has no kernel");
        for line in ptx.lines() {
            let line = line.trim_start();
            if line.starts_with("//") {
                continue;
            }
            for instruction in &instructions {
                assert!(
                    !line.starts_with(instruction.as_str()),
                    "{module} contains the instruction `{line}`"
                );
            }
        }
    }
}

/// Modules quarantined after Codex's review (#2028 comments 6149477404 and
/// 6149949044): each hides a learned dense contraction in scalar loops, which
/// the token and PTX scans above cannot see. They stay out of the build, their
/// source stays in the evidence directory, and none of their kernels returns to a
/// compiled source.
#[test]
fn quarantined_dense_contraction_modules_stay_out_of_the_build() {
    let quarantined = [
        ("native_context", "native_context_quarantined.cu"),
        ("selected_potential", "selected_potential_quarantined.cu"),
    ];
    let build = fs::read_to_string(crate_path("build.rs")).expect("build.rs");
    for (module, evidence) in quarantined {
        assert!(
            !crate_path(&format!("src/{module}.cu")).exists(),
            "src/{module}.cu is back in the crate"
        );
        assert!(
            !build.contains(&format!("{module}.cu")) && !build.contains(&format!("{module}.ptx")),
            "build.rs compiles quarantined {module}"
        );
        let source = fs::read_to_string(crate_path(&format!(
            "../../docs/history/d29-excluded-kernels-2026-10-13/{evidence}"
        )))
        .expect("quarantined evidence file");
        let names: Vec<&str> = source
            .split("__global__ void ")
            .skip(1)
            .filter_map(|rest| rest.split('(').next())
            .map(str::trim)
            .collect();
        assert!(!names.is_empty(), "{evidence} lost its kernels");
        for compiled in COMPILED {
            let text = code(&fs::read_to_string(crate_path(compiled)).expect(compiled));
            for name in &names {
                assert!(
                    !text.contains(&format!("__global__ void {name}(")),
                    "{compiled} defines quarantined `{name}`"
                );
            }
        }
    }
}
