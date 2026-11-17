use std::fs;
use std::path::{Path, PathBuf};

use crate::mvm_initramfs::MVM_INITRAMFS;
use sha2::{Digest, Sha256};

const UPSTREAM_COMMIT: &str = "01956d5c35f268fc7d1b646d97d3dff21b9e945f";
const MVM_SHA: &str = "2789ec57daa7f92dbd094e5d2c38a8ae343b4f38bd09cd2ed6bc377fa7d67206";
const MVM_CC_SHA: &str = "19390475442bbede4332bf38776706d42aa09e2daa9df4e014d140c6af5ce12f";
const GENERATED_SHA: &str = "acc9a89d65bb43bdb61a720fd90570dacf46a43adb9340fcbc427a9245e247e8";
const VSOCK_MODULE_SHA: &str = "102c1c859413c4d4a1470d8fecb4dcc3dfa8fa0ea50c402468f22718ad0b478f";
const INITRAMFS_SHA: &str = "071e9c8d7749c3b86947529db532db0b82ac3707ad1b738d3ba681dc8161c65c";

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

#[test]
fn readme_pins_upstream_commit_and_hashes() {
    let readme = fs::read_to_string(manifest().join("README.md")).expect("crate README");
    for pinned in [
        UPSTREAM_COMMIT,
        MVM_SHA,
        MVM_CC_SHA,
        GENERATED_SHA,
        VSOCK_MODULE_SHA,
        INITRAMFS_SHA,
    ] {
        assert!(readme.contains(pinned), "README missing {pinned}");
    }
    assert!(readme.contains("45,751-45,938"));
    assert!(readme.contains("188-line"));
}

#[test]
fn reversing_declared_mvm_patch_recovers_pinned_upstream_bytes() {
    let adapted = String::from_utf8(source("mvm.rs")).expect("mvm.rs UTF-8");
    let lines: Vec<_> = adapted.split_inclusive('\n').collect();
    let mut patch = include_str!("../../provenance/mvm-uor.patch")
        .split_inclusive('\n')
        .peekable();
    assert_eq!(patch.next(), Some("--- a/mvm.rs\n"));
    assert_eq!(patch.next(), Some("+++ b/mvm.rs\n"));
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
        // A zero-line range starts immediately after this one-based line number.
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
    assert_eq!(upstream.len(), 98_525);
    assert_eq!(sha(upstream.as_bytes()), MVM_SHA);
}

#[test]
fn mvm_cc_is_byte_identical_to_upstream() {
    assert_eq!(sha(&source("mvm_cc.rs")), MVM_CC_SHA);
}

#[test]
fn vsock_module_is_exact_188_line_upstream_tail() {
    let module = source("spiralcore_vsock.rs");
    assert_eq!(module.iter().filter(|&&b| b == b'\n').count(), 188);
    assert_eq!(sha(&module), VSOCK_MODULE_SHA);
}

#[test]
fn initramfs_source_bytes_match_upstream_and_no_binary_is_committed() {
    assert_eq!(MVM_INITRAMFS.len(), 12_164);
    assert_eq!(sha(MVM_INITRAMFS), INITRAMFS_SHA);
    assert!(!Path::new(&manifest().join("src/mvm_initramfs.bin")).exists());
}

#[test]
fn excluded_generated_model_and_ast_cargo_are_absent() {
    for name in [
        "lib.rs",
        "mvm.rs",
        "mvm_cc.rs",
        "spiralcore_vsock.rs",
        "mvm_initramfs.rs",
    ] {
        let text = String::from_utf8(source(name)).expect("source UTF-8");
        for excluded in [
            "r4_compositional_gllm",
            "wasm_ast_eval",
            "execute_cargo_json",
        ] {
            assert!(
                !text.contains(excluded),
                "{name} contains excluded {excluded}"
            );
        }
    }
}
