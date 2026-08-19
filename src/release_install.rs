//! #741: the explicit, verified release-bundle fetch — the install half
//! of the release pipeline (`docs/RELEASE_PIPELINE.md`).
//!
//! `r4 install-release --tag vX.Y` downloads the two release assets a
//! published GitHub Release carries — `release-bundle.json` (the #655-D
//! sidecar manifest) and `release-bundle.tar.gz` (the packaged compiled
//! bundle) — and installs the bundle under the model store ONLY after
//! every declared component digest matches the extracted bytes. This is
//! deliberately stricter than serving-time sidecar verification
//! (`release_bundle_loader`, advisory by design): at install time a
//! mismatch is a hard failure and nothing is installed. The fetch is
//! explicit — no serving path, first request, or setup step ever
//! triggers it (#655's own scope: "first request must not silently
//! download").
//!
//! Fail-closed inventory: the archive must contain EXACTLY the
//! attested component files (the five `docs/serving_release_packaging_655_d.md`
//! relative paths, tokenizer present iff the manifest declares it) — an
//! archive smuggling any other file is refused outright, so nothing
//! unattested ever lands on disk.

use std::path::{Path, PathBuf};
use std::process::Command;

use uor_r4_api::ReleaseBundleManifest;

use crate::release_bundle_loader::RELEASE_BUNDLE_SIDECAR_FILE_NAME;

/// Release asset name of the packaged bundle archive.
pub const RELEASE_BUNDLE_ARCHIVE_ASSET: &str = "release-bundle.tar.gz";

/// The component relative paths inside a packaged bundle —
/// `docs/serving_release_packaging_655_d.md`'s field-to-file mapping,
/// mirrored from `release_bundle_packager`.
const GRAPH_RELATIVE_PATH: &str = "graph/score.r4g1";
const SIGNATURE_ARTIFACT_RELATIVE_PATH: &str = "tless_artifacts.bin";
const TOKENIZER_RELATIVE_PATH: &str = "tokenizer.bin";
const SCORE_REPORT_RELATIVE_PATH: &str = "graph/score_report.json";
const COMPILE_REPORT_RELATIVE_PATH: &str = "graph-cover/cover_report.json";

/// One verified-install request.
pub struct InstallReleaseRequest {
    /// GitHub repository the release lives in (`owner/name`).
    pub repo: String,
    /// Release tag (e.g. `v0.1`).
    pub tag: String,
    /// Install name under the model store's `compiled/` inventory;
    /// `None` uses the manifest's own `model_id`.
    pub name: Option<String>,
}

/// A completed verified install.
#[derive(Debug)]
pub struct InstalledRelease {
    /// Where the verified bundle now lives.
    pub destination: PathBuf,
    /// The verified manifest (also written beside the bundle as its
    /// serving-time sidecar).
    pub manifest: ReleaseBundleManifest,
}

/// Fetch one URL to a destination file. The production fetcher shells
/// out to `curl` (the same external-tool convention `download_source`
/// uses for `hf`); tests substitute a local fixture writer.
pub trait AssetFetcher {
    fn fetch(&mut self, url: &str, destination: &Path) -> Result<(), String>;
}

/// `curl -fsSL --retry 3` — fail on HTTP errors, follow the release
/// asset redirect, no progress noise, bounded retries.
pub struct CurlFetcher;

impl AssetFetcher for CurlFetcher {
    fn fetch(&mut self, url: &str, destination: &Path) -> Result<(), String> {
        let status = Command::new("curl")
            .args(["-fsSL", "--retry", "3", "-o"])
            .arg(destination)
            .arg(url)
            .status()
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    "curl is required for install-release but was not found on PATH".to_owned()
                } else {
                    format!("could not run curl: {error}")
                }
            })?;
        if !status.success() {
            return Err(format!(
                "download failed ({url}): curl exited with {status}; the release/tag/asset may not exist"
            ));
        }
        Ok(())
    }
}

fn validate_repo(repo: &str) -> Result<(), String> {
    let mut parts = repo.split('/');
    let (Some(owner), Some(name), None) = (parts.next(), parts.next(), parts.next()) else {
        return Err(format!("--repo must be owner/name, got {repo:?}"));
    };
    let ok = |s: &str| {
        !s.is_empty()
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.')
    };
    if ok(owner) && ok(name) {
        Ok(())
    } else {
        Err(format!("--repo must be owner/name, got {repo:?}"))
    }
}

fn validate_tag(tag: &str) -> Result<(), String> {
    if !tag.is_empty()
        && tag
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.')
    {
        Ok(())
    } else {
        Err(format!(
            "--tag must be a plain release tag (alphanumeric/._-), got {tag:?}"
        ))
    }
}

fn asset_url(repo: &str, tag: &str, asset: &str) -> String {
    format!("https://github.com/{repo}/releases/download/{tag}/{asset}")
}

fn digest_matches(path: &Path, declared: &str, label: &str) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|error| {
        format!(
            "{label}: could not read extracted {}: {error}",
            path.display()
        )
    })?;
    let actual = format!("blake3:{}", blake3::hash(&bytes).to_hex());
    if actual == declared {
        Ok(())
    } else {
        Err(format!(
            "{label}: digest mismatch — manifest declares {declared}, archive bytes are {actual}; refusing to install"
        ))
    }
}

/// Every regular file under `root`, as `/`-joined paths relative to it.
fn inventory(root: &Path) -> Result<Vec<String>, String> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir)
            .map_err(|error| format!("could not list {}: {error}", dir.display()))?;
        for entry in entries {
            let entry = entry.map_err(|error| format!("could not list archive entry: {error}"))?;
            let path = entry.path();
            let kind = entry
                .file_type()
                .map_err(|error| format!("could not stat {}: {error}", path.display()))?;
            if kind.is_symlink() {
                return Err(format!(
                    "archive contains a symlink ({}); refusing to install",
                    path.display()
                ));
            }
            if kind.is_dir() {
                stack.push(path);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .map_err(|_| "archive entry escaped the extraction root".to_owned())?;
                let mut joined = String::new();
                for component in relative.components() {
                    if !joined.is_empty() {
                        joined.push('/');
                    }
                    joined.push_str(&component.as_os_str().to_string_lossy());
                }
                found.push(joined);
            }
        }
    }
    found.sort();
    Ok(found)
}

/// Download, verify, and install one published release bundle. See the
/// module docs for the contract; every failure leaves the model store
/// untouched (all work happens in a staging directory that is removed
/// on the way out).
pub fn install_release(
    store_root: &Path,
    request: &InstallReleaseRequest,
    fetcher: &mut dyn AssetFetcher,
) -> Result<InstalledRelease, String> {
    validate_repo(&request.repo)?;
    validate_tag(&request.tag)?;
    if let Some(name) = request.name.as_deref() {
        if name.is_empty()
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.')
        {
            return Err(format!(
                "--name must be a plain directory name (alphanumeric/._-), got {name:?}"
            ));
        }
    }

    let staging = store_root.join("staging").join(format!(
        "install-release-{}-{}",
        request.tag,
        std::process::id()
    ));
    let result = install_into_staging(store_root, &staging, request, fetcher);
    let _ = std::fs::remove_dir_all(&staging);
    result
}

fn install_into_staging(
    store_root: &Path,
    staging: &Path,
    request: &InstallReleaseRequest,
    fetcher: &mut dyn AssetFetcher,
) -> Result<InstalledRelease, String> {
    std::fs::create_dir_all(staging)
        .map_err(|error| format!("could not create staging {}: {error}", staging.display()))?;

    // 1. The sidecar manifest first: small, and it names what the
    //    archive must contain before the archive is even fetched.
    let sidecar_path = staging.join(RELEASE_BUNDLE_SIDECAR_FILE_NAME);
    fetcher.fetch(
        &asset_url(
            &request.repo,
            &request.tag,
            RELEASE_BUNDLE_SIDECAR_FILE_NAME,
        ),
        &sidecar_path,
    )?;
    let sidecar_bytes = std::fs::read(&sidecar_path)
        .map_err(|error| format!("could not read downloaded sidecar: {error}"))?;
    let manifest: ReleaseBundleManifest = serde_json::from_slice(&sidecar_bytes)
        .map_err(|error| format!("release-bundle.json does not parse: {error}"))?;
    if let Some(reason) = manifest.validate() {
        return Err(format!("release-bundle.json failed validation: {reason}"));
    }

    // 2. The archive, extracted into an empty staging subdirectory.
    let archive_path = staging.join(RELEASE_BUNDLE_ARCHIVE_ASSET);
    fetcher.fetch(
        &asset_url(&request.repo, &request.tag, RELEASE_BUNDLE_ARCHIVE_ASSET),
        &archive_path,
    )?;
    let bundle_dir = staging.join("bundle");
    std::fs::create_dir_all(&bundle_dir)
        .map_err(|error| format!("could not create {}: {error}", bundle_dir.display()))?;
    let status = Command::new("tar")
        .arg("-xzf")
        .arg(&archive_path)
        .arg("-C")
        .arg(&bundle_dir)
        .status()
        .map_err(|error| format!("could not run tar: {error}"))?;
    if !status.success() {
        return Err(format!(
            "archive extraction failed: tar exited with {status}"
        ));
    }

    // 3. Fail-closed inventory: exactly the attested files, nothing else.
    let mut expected: Vec<String> = vec![
        GRAPH_RELATIVE_PATH.to_owned(),
        SIGNATURE_ARTIFACT_RELATIVE_PATH.to_owned(),
        SCORE_REPORT_RELATIVE_PATH.to_owned(),
        COMPILE_REPORT_RELATIVE_PATH.to_owned(),
    ];
    if manifest.components.tokenizer.is_some() {
        expected.push(TOKENIZER_RELATIVE_PATH.to_owned());
    }
    expected.sort();
    let found = inventory(&bundle_dir)?;
    if found != expected {
        return Err(format!(
            "archive contents do not match the manifest's attested component set; expected exactly {expected:?}, found {found:?}; refusing to install"
        ));
    }

    // 4. Every declared digest must match the extracted bytes.
    digest_matches(
        &bundle_dir.join(GRAPH_RELATIVE_PATH),
        &manifest.components.graph,
        "components.graph",
    )?;
    digest_matches(
        &bundle_dir.join(SIGNATURE_ARTIFACT_RELATIVE_PATH),
        &manifest.components.signature_artifact,
        "components.signature_artifact",
    )?;
    digest_matches(
        &bundle_dir.join(SCORE_REPORT_RELATIVE_PATH),
        &manifest.components.score_report,
        "components.score_report",
    )?;
    digest_matches(
        &bundle_dir.join(COMPILE_REPORT_RELATIVE_PATH),
        &manifest.components.compile_report,
        "components.compile_report",
    )?;
    if let Some(declared) = manifest.components.tokenizer.as_deref() {
        digest_matches(
            &bundle_dir.join(TOKENIZER_RELATIVE_PATH),
            declared,
            "components.tokenizer",
        )?;
    }

    // 5. Keep the verified sidecar beside the bundle so serving-time
    //    advisory verification (#655-C1c) finds it.
    std::fs::write(
        bundle_dir.join(RELEASE_BUNDLE_SIDECAR_FILE_NAME),
        &sidecar_bytes,
    )
    .map_err(|error| format!("could not write bundle sidecar: {error}"))?;

    // 6. Atomic move into the store; never overwrite an existing install.
    let install_name = request
        .name
        .clone()
        .unwrap_or_else(|| manifest.model_id.clone());
    let destination = store_root.join("compiled").join(&install_name);
    if destination.exists() {
        return Err(format!(
            "{} already exists; remove it first if you intend to replace it (install-release never overwrites)",
            destination.display()
        ));
    }
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("could not create {}: {error}", parent.display()))?;
    }
    std::fs::rename(&bundle_dir, &destination).map_err(|error| {
        format!(
            "could not move verified bundle into {}: {error}",
            destination.display()
        )
    })?;

    Ok(InstalledRelease {
        destination,
        manifest,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::release_bundle_packager::{package_release_bundle, PackageInputs};
    use uor_r4_api::{BundleCapability, UorMatmulProvenance};
    use uor_r4_core::transformerless::hf_bpe::TokenizerAdapter;

    /// Serves the two release assets from local fixture files, recording
    /// requested URLs — the network never runs in tests.
    struct FixtureFetcher {
        sidecar: Vec<u8>,
        archive: Vec<u8>,
        urls: Vec<String>,
    }

    impl AssetFetcher for FixtureFetcher {
        fn fetch(&mut self, url: &str, destination: &Path) -> Result<(), String> {
            self.urls.push(url.to_owned());
            let bytes = if url.ends_with(RELEASE_BUNDLE_SIDECAR_FILE_NAME) {
                &self.sidecar
            } else if url.ends_with(RELEASE_BUNDLE_ARCHIVE_ASSET) {
                &self.archive
            } else {
                return Err(format!("unexpected fixture URL {url}"));
            };
            std::fs::write(destination, bytes).map_err(|error| error.to_string())
        }
    }

    fn scratch_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "uor-r4-release-install-{label}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch directory");
        dir
    }

    fn write(dir: &Path, relative: &str, bytes: &[u8]) {
        let path = dir.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent dir");
        }
        std::fs::write(&path, bytes).expect("write fixture file");
    }

    /// A complete packaged bundle directory + its honestly-derived
    /// manifest, mirroring the packager's own layout.
    fn packaged_fixture(root: &Path) -> (PathBuf, ReleaseBundleManifest) {
        let bundle = root.join("packaged");
        write(&bundle, "graph/score.r4g1", b"graph bytes");
        write(&bundle, "tless_artifacts.bin", b"signature bytes");
        write(&bundle, "tokenizer.bin", b"tokenizer bytes");
        write(&bundle, "graph/score_report.json", b"{\"score\":true}");
        write(
            &bundle,
            "graph-cover/cover_report.json",
            b"{\"cover\":true}",
        );
        let manifest = package_release_bundle(
            &bundle,
            PackageInputs {
                model_id: "r4".to_owned(),
                capability: BundleCapability::InstructionChat,
                uor_matmul: UorMatmulProvenance {
                    rev: "b13c98449948174f590e337c4dc25dfc394a07d0".to_owned(),
                    operation_profile: "exact-gemm-float".to_owned(),
                    license: "MIT".to_owned(),
                    source_digest: None,
                },
                tokenizer_adapter: TokenizerAdapter {
                    family: "hf-byte-bpe".to_owned(),
                    ..Default::default()
                },
                provenance_note: None,
            },
        )
        .expect("fixture packages");
        (bundle, manifest)
    }

    fn tar_gz(dir: &Path, out: &Path) -> Vec<u8> {
        let status = Command::new("tar")
            .arg("-czf")
            .arg(out)
            .arg("-C")
            .arg(dir)
            .arg(".")
            .status()
            .expect("tar runs");
        assert!(status.success(), "fixture archive builds");
        std::fs::read(out).expect("read fixture archive")
    }

    fn fetcher_for(root: &Path) -> (FixtureFetcher, ReleaseBundleManifest) {
        let (bundle, manifest) = packaged_fixture(root);
        let archive = tar_gz(&bundle, &root.join("release-bundle.tar.gz"));
        (
            FixtureFetcher {
                sidecar: serde_json::to_vec(&manifest).expect("serialize manifest"),
                archive,
                urls: Vec::new(),
            },
            manifest,
        )
    }

    fn request() -> InstallReleaseRequest {
        InstallReleaseRequest {
            repo: "UOR-Foundation/uor-r4".to_owned(),
            tag: "v0.1".to_owned(),
            name: None,
        }
    }

    #[test]
    fn verified_release_installs_with_its_sidecar_and_exact_urls() {
        let root = scratch_dir("happy");
        let store = root.join("store");
        let (mut fetcher, manifest) = fetcher_for(&root);
        let installed =
            install_release(&store, &request(), &mut fetcher).expect("verified install");
        assert_eq!(installed.destination, store.join("compiled").join("r4"));
        assert_eq!(installed.manifest, manifest);
        // The attested components and the sidecar are in place; the
        // serving loader's advisory verification would now find them.
        for file in [
            "graph/score.r4g1",
            "tless_artifacts.bin",
            "tokenizer.bin",
            "graph/score_report.json",
            "graph-cover/cover_report.json",
            RELEASE_BUNDLE_SIDECAR_FILE_NAME,
        ] {
            assert!(
                installed.destination.join(file).is_file(),
                "{file} installed"
            );
        }
        assert_eq!(
            fetcher.urls,
            vec![
                "https://github.com/UOR-Foundation/uor-r4/releases/download/v0.1/release-bundle.json".to_owned(),
                "https://github.com/UOR-Foundation/uor-r4/releases/download/v0.1/release-bundle.tar.gz".to_owned(),
            ],
            "explicit fetch hits exactly the two declared release assets"
        );
        // Staging is cleaned up on success.
        assert!(
            !store.join("staging").exists()
                || std::fs::read_dir(store.join("staging"))
                    .map(|mut d| d.next().is_none())
                    .unwrap_or(true)
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn tampered_component_bytes_refuse_to_install() {
        let root = scratch_dir("tampered");
        let store = root.join("store");
        let (bundle, manifest) = packaged_fixture(&root);
        // Tamper AFTER the manifest was derived.
        write(&bundle, "graph/score.r4g1", b"TAMPERED graph bytes");
        let archive = tar_gz(&bundle, &root.join("release-bundle.tar.gz"));
        let mut fetcher = FixtureFetcher {
            sidecar: serde_json::to_vec(&manifest).expect("serialize manifest"),
            archive,
            urls: Vec::new(),
        };
        let error = install_release(&store, &request(), &mut fetcher)
            .expect_err("tampered bytes must refuse");
        assert!(error.contains("components.graph"), "{error}");
        assert!(error.contains("digest mismatch"), "{error}");
        assert!(
            !store.join("compiled").join("r4").exists(),
            "nothing installed"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn smuggled_extra_file_refuses_to_install() {
        let root = scratch_dir("smuggled");
        let store = root.join("store");
        let (bundle, manifest) = packaged_fixture(&root);
        write(&bundle, "extra/unattested.bin", b"not in the manifest");
        let archive = tar_gz(&bundle, &root.join("release-bundle.tar.gz"));
        let mut fetcher = FixtureFetcher {
            sidecar: serde_json::to_vec(&manifest).expect("serialize manifest"),
            archive,
            urls: Vec::new(),
        };
        let error = install_release(&store, &request(), &mut fetcher)
            .expect_err("unattested file must refuse");
        assert!(error.contains("attested component set"), "{error}");
        assert!(
            !store.join("compiled").join("r4").exists(),
            "nothing installed"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn existing_install_is_never_overwritten() {
        let root = scratch_dir("existing");
        let store = root.join("store");
        std::fs::create_dir_all(store.join("compiled").join("r4")).expect("pre-existing install");
        let (mut fetcher, _) = fetcher_for(&root);
        let error = install_release(&store, &request(), &mut fetcher)
            .expect_err("existing destination must refuse");
        assert!(error.contains("never overwrites"), "{error}");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn invalid_repo_tag_and_name_shapes_are_rejected_before_any_fetch() {
        let root = scratch_dir("shapes");
        let store = root.join("store");
        let (mut fetcher, _) = fetcher_for(&root);
        for (repo, tag, name) in [
            ("not-a-repo", "v0.1", None),
            ("owner/name/extra", "v0.1", None),
            ("UOR-Foundation/uor-r4", "v0.1/../evil", None),
            ("UOR-Foundation/uor-r4", "", None),
            ("UOR-Foundation/uor-r4", "v0.1", Some("../evil")),
        ] {
            let bad = InstallReleaseRequest {
                repo: repo.to_owned(),
                tag: tag.to_owned(),
                name: name.map(str::to_owned),
            };
            assert!(
                install_release(&store, &bad, &mut fetcher).is_err(),
                "shape {repo:?} {tag:?} {name:?} must be rejected"
            );
        }
        assert!(fetcher.urls.is_empty(), "no fetch before validation");
        let _ = std::fs::remove_dir_all(&root);
    }
}
