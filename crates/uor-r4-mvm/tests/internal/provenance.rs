use std::fs;
use std::path::{Path, PathBuf};

use crate::mvm_initramfs::MVM_INITRAMFS;
use sha2::{Digest, Sha256};

const UPSTREAM_COMMIT: &str = "288e2677";
const MVM_SHA: &str = "85aa69b442c70fc8de646c62bba4c37842b5fbc502627970b438873aecbe12ee";
const MVM_SHELL_SHA: &str = "db9a015b21ab77e8ad6e2901a7ff2a8ce68fa2646212f975b2f3b25266220db2";
const MVM_CC_SHA: &str = "66d497f62868dce78ba119972a41dd21e1c572328ab1317e466e2e147e70491d";
const VSOCK_SHA: &str = "1314bc8f02e4ed542f116b57769d2d6b23a5433732252fed811f6a2d2992df09";
const INITRAMFS_SHA: &str = "071e9c8d7749c3b86947529db532db0b82ac3707ad1b738d3ba681dc8161c65c";

const SECOPS_AUDIT_SHA: &str = "a262c7082608879ba3feb4dfe977ed0fca9b6a1fed7bc64fa4437cec091838e6";
const SPIRALCORE_VSOCK_SHA: &str =
    "102c1c859413c4d4a1470d8fecb4dcc3dfa8fa0ea50c402468f22718ad0b478f";
const CONNECTED_STUDIO_SHA: &str =
    "26706cf61cdabccba78fe89320cf706e1d23e9e8901d7916f293c15d863528be";
const MVM_LINUX_CONTAINER_SHA: &str =
    "a7a9a9b78ff47edcbf5bf6486ee6e6994552f33ef30602051a59ebeb5dd07955";
const MVM_POLYGLOT_SHA: &str = "5efb59ef69c16747a437630a195533f26b524bf1707e664ee61e34aa4d6f0bf9";

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn source(name: &str) -> Vec<u8> {
    fs::read(manifest().join("src").join(name)).expect("source file")
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn reverse_declared_patch(
    src_name: &str,
    patch_text: &str,
    expected_len: usize,
    expected_sha: &str,
) {
    let adapted = String::from_utf8(source(src_name)).expect("source UTF-8");
    let lines: Vec<_> = adapted.split_inclusive('\n').collect();
    let mut patch = patch_text.split_inclusive('\n').peekable();
    let expected_from = format!("--- a/{src_name}\n");
    let expected_to = format!("+++ b/{src_name}\n");
    assert_eq!(patch.next(), Some(expected_from.as_str()));
    assert_eq!(patch.next(), Some(expected_to.as_str()));
    let mut upstream = String::new();
    let mut cursor = 0;
    while let Some(header) = patch.next() {
        assert!(header.starts_with("@@ "));
        let target = header
            .split_whitespace()
            .nth(2)
            .unwrap()
            .strip_prefix('+')
            .unwrap();
        let (start, count) = target.split_once(',').unwrap_or((target, "1"));
        let count = count.parse::<usize>().unwrap();
        let start = start.parse::<usize>().unwrap() - usize::from(count > 0);
        upstream.extend(lines[cursor..start].iter().copied());
        cursor = start;
        while let Some(line) = patch.next_if(|line| !line.starts_with("@@ ")) {
            if let Some(original) = line.strip_prefix('-') {
                upstream.push_str(original);
            } else {
                assert_eq!(line.strip_prefix('+'), Some(lines[cursor]));
                cursor += 1;
            }
        }
        assert_eq!(cursor - start, count);
    }
    upstream.extend(lines[cursor..].iter().copied());
    assert_eq!(upstream.len(), expected_len);
    assert_eq!(sha(upstream.as_bytes()), expected_sha);
}

#[test]
fn readme_pins_upstream_commit_and_hashes() {
    let readme = fs::read_to_string(manifest().join("README.md")).expect("crate README");
    for pinned in [
        UPSTREAM_COMMIT,
        MVM_SHA,
        MVM_SHELL_SHA,
        MVM_CC_SHA,
        VSOCK_SHA,
        INITRAMFS_SHA,
        SECOPS_AUDIT_SHA,
        SPIRALCORE_VSOCK_SHA,
        CONNECTED_STUDIO_SHA,
        MVM_LINUX_CONTAINER_SHA,
        MVM_POLYGLOT_SHA,
    ] {
        assert!(readme.contains(pinned), "README missing {pinned}");
    }
}

#[test]
fn reversing_declared_patches_recovers_pinned_upstream_bytes() {
    reverse_declared_patch(
        "mvm.rs",
        include_str!("../../provenance/mvm-uor.patch"),
        331_436,
        MVM_SHA,
    );
    reverse_declared_patch(
        "mvm_cc.rs",
        include_str!("../../provenance/mvm_cc-uor.patch"),
        127_667,
        MVM_CC_SHA,
    );
    reverse_declared_patch(
        "mvm_shell.rs",
        include_str!("../../provenance/mvm_shell-uor.patch"),
        392_781,
        MVM_SHELL_SHA,
    );
}

#[test]
fn vsock_matches_upstream_with_exact_frame_length_check() {
    let vsock_text = String::from_utf8(source("vsock.rs")).expect("vsock UTF-8");
    assert!(vsock_text.contains("if wire.len() != end {"));
    let restored = vsock_text.replace("if wire.len() != end {", "if wire.len() < end {");
    assert_eq!(sha(restored.as_bytes()), VSOCK_SHA);
}

#[test]
fn lexlean_admission_matches_exact_extracted_modules() {
    let text = String::from_utf8(source("lexlean_admission.rs")).expect("admission UTF-8");
    let expected = [
        ("pub mod r4_secops_audit {", SECOPS_AUDIT_SHA),
        ("pub mod r4_spiralcore_vsock {", SPIRALCORE_VSOCK_SHA),
        ("pub mod r4_connected_studio {", CONNECTED_STUDIO_SHA),
        ("pub mod r4_mvm_linux_container {", MVM_LINUX_CONTAINER_SHA),
        ("pub mod r4_mvm_polyglot_toolchain {", MVM_POLYGLOT_SHA),
    ];
    let mut starts: Vec<(usize, &str)> = expected
        .iter()
        .map(|(marker, hash)| {
            let pos = text
                .find(marker)
                .unwrap_or_else(|| panic!("missing module marker {marker}"));
            (pos, *hash)
        })
        .collect();
    starts.sort_by_key(|(pos, _)| *pos);
    for idx in 0..starts.len() {
        let start = starts[idx].0;
        let end = if idx + 1 < starts.len() {
            starts[idx + 1].0 - 1
        } else {
            text.len()
        };
        let slice = &text[start..end];
        assert_eq!(sha(slice.as_bytes()), starts[idx].1);
    }
}

#[test]
fn initramfs_source_bytes_match_upstream_and_no_binary_is_committed() {
    assert_eq!(MVM_INITRAMFS.len(), 12_164);
    assert_eq!(sha(MVM_INITRAMFS), INITRAMFS_SHA);
    assert!(!Path::new(&manifest().join("src/mvm_initramfs.bin")).exists());
}

#[test]
fn excluded_model_and_ast_shortcut_modules_are_absent() {
    for name in [
        "lib.rs",
        "sandbox.rs",
        "wasm.rs",
        "lexlean_admission.rs",
        "mvm.rs",
        "mvm_shell.rs",
        "mvm_cc.rs",
        "vsock.rs",
        "spiralcore_vsock.rs",
        "mvm_initramfs.rs",
    ] {
        let text = String::from_utf8(source(name)).expect("source UTF-8");
        let mut banned_tokens: Vec<String> = [
            "wasm_ast_eval",
            "execute_cargo_json",
            "r4_compositional_gllm",
            "holo_geometric_runtime",
            "sparse_basin_v15",
            "hopf_zeta",
            "gllm_v9",
            "native_q31_lane",
        ]
        .into_iter()
        .map(str::to_string)
        .collect();
        banned_tokens.push(concat!("soft", "max").to_string());
        for excluded in &banned_tokens {
            assert!(
                !text.contains(excluded.as_str()),
                "{name} contains excluded token {excluded}"
            );
        }
        if name != "lib.rs" && name != "mvm.rs" {
            for kw in ["unsafe {", "unsafe fn", "unsafe impl", "unsafe trait"] {
                assert!(
                    !text.contains(kw),
                    "{name} contains disallowed unsafe keyword `{kw}`"
                );
            }
        }
    }
}
