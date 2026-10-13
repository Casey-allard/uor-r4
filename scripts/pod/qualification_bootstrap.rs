//! Python-free, setup-only finite CUDA bootstrap. std only; no model runs.
//!
//! If Rust/rustup is absent, the admission controller may use this official prologue:
//!   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /workspace/rustup-init.sh
//!   sh /workspace/rustup-init.sh -y --profile minimal --default-toolchain 1.97.1
//! Then, through scripts/pod/uor-pod only, compile this helper on the pod:
//! Copy this file and ../native_dependency_boundary.rs preserving their relative paths, then:
//!   /root/.cargo/bin/rustc --edition=2021 /workspace/scripts/pod/qualification_bootstrap.rs -o /workspace/qualification-bootstrap
//!   /workspace/qualification-bootstrap --sha FULL_SHA --lab codex --session SESSION --root /workspace/EXCLUSIVE_ROOT
//! Existing roots require --resume and the exact same ownership/source marker.
//! No HF client/dataset, Python, old bootstrap, parity suite, or test execution.
#[path = "../native_dependency_boundary.rs"]
mod native_dependency_boundary;

use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Component, Path, PathBuf},
    process::{self, Command, Stdio},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

const RUST_VERSION: &str = "1.97.1";
const REPOSITORY: &str = "https://github.com/UOR-Foundation/uor-r4.git";
type Result<T> = std::result::Result<T, String>;

#[derive(Debug)]
struct Config {
    sha: String,
    lab: String,
    session: String,
    root: PathBuf,
    reports: PathBuf,
    jobs: u32,
    arch: String,
    resume: bool,
    check_only: bool,
}

fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 160
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
        && s != "."
        && s != ".."
}
fn full_sha(s: &str) -> bool {
    s.len() == 40
        && s.bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
fn parse(args: &[String]) -> Result<Config> {
    let mut sha = None;
    let mut lab = None;
    let mut session = None;
    let mut root = None;
    let mut jobs = 4;
    let mut arch = "compute_120".to_string();
    let mut resume = false;
    let mut check_only = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--resume" => { resume = true; i += 1; continue; }
            "--check-args" => { check_only = true; i += 1; continue; }
            "--sha" | "--lab" | "--session" | "--root" | "--jobs" | "--arch" => {}
            _ => return Err("unknown argument; expected --sha --lab --session --root [--jobs N] [--arch compute_N] [--resume] [--check-args]".into()),
        }
        let value = args.get(i + 1).ok_or("option lacks a value")?.clone();
        match args[i].as_str() {
            "--sha" if sha.is_none() => sha = Some(value),
            "--lab" if lab.is_none() => lab = Some(value),
            "--session" if session.is_none() => session = Some(value),
            "--root" if root.is_none() => root = Some(PathBuf::from(value)),
            "--jobs" => jobs = value.parse().map_err(|_| "jobs must be an integer")?,
            "--arch" => arch = value,
            _ => return Err("duplicate identity option".into()),
        }
        i += 2;
    }
    let mut config = Config {
        sha: sha.ok_or("missing --sha")?,
        lab: lab.ok_or("missing --lab")?,
        session: session.ok_or("missing --session")?,
        root: root.ok_or("missing --root")?,
        reports: PathBuf::new(),
        jobs,
        arch,
        resume,
        check_only,
    };
    if !full_sha(&config.sha) {
        return Err("source must be a full lowercase 40-character commit".into());
    }
    if !identifier(&config.lab) || !identifier(&config.session) {
        return Err("lab/session must be bounded plain identifiers".into());
    }
    if !config.root.starts_with("/workspace")
        || config.root == Path::new("/workspace")
        || !config.root.is_absolute()
        || config
            .root
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
        || !config
            .root
            .to_string_lossy()
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"/-_.".contains(&c))
    {
        return Err("root must be a new exclusive plain path strictly inside /workspace, with no parent traversal".into());
    }
    if !(1..=16).contains(&config.jobs) {
        return Err("jobs must be 1..16".into());
    }
    if config.root.starts_with("/workspace/uor-r4/jobs") {
        return Err(
            "build root must be outside jobs; only bounded receipts belong in automatic archival"
                .into(),
        );
    }
    let name = config
        .root
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("missing root name")?;
    config.reports = Path::new("/workspace/uor-r4/jobs").join(format!(
        "{}-{}-{}-{}-setup",
        config.lab,
        config.session,
        name,
        &config.sha[..12]
    ));
    if config.reports.file_name().map(|n| n.len()).unwrap_or(0) > 240 {
        return Err("combined exclusive report name exceeds bounded filesystem name".into());
    }
    let number = config
        .arch
        .strip_prefix("compute_")
        .ok_or("arch must be compute_N")?;
    if number.is_empty() || !number.bytes().all(|c| c.is_ascii_digit()) {
        return Err("arch must be compute_N".into());
    }
    Ok(config)
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|t| t.as_secs())
        .unwrap_or(0)
}
fn record(root: &Path, stage: &str, code: i32, seconds: f64) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join("stages.tsv"))
        .map_err(|e| e.to_string())?;
    writeln!(file, "{}\t{stage}\t{code}\t{seconds:.3}", unix_seconds()).map_err(|e| e.to_string())
}

fn run(c: &Config, stage: &str, program: &str, args: &[&str], cwd: &Path) -> Result<String> {
    let receipts = c.reports.clone();
    let serial = unix_seconds();
    // create_new prevents a resume from overwriting a previous attempt.
    let mut suffix = 0;
    let (stdout_path, stderr_path, stdout, stderr) = loop {
        let prefix = format!("{stage}-{serial}-{suffix}");
        let out = receipts.join(format!("{prefix}.stdout.log"));
        let err = receipts.join(format!("{prefix}.stderr.log"));
        match OpenOptions::new().write(true).create_new(true).open(&out) {
            Ok(stdout) => {
                let stderr = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&err)
                    .map_err(|e| e.to_string())?;
                break (out, err, stdout, stderr);
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => suffix += 1,
            Err(e) => return Err(e.to_string()),
        }
    };
    println!(
        "SETUP {stage}: stdout={} stderr={}",
        stdout_path.display(),
        stderr_path.display()
    );
    let start = Instant::now();
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .env("CARGO_TARGET_DIR", c.root.join("target"))
        .env("CARGO_BUILD_JOBS", c.jobs.to_string())
        .env("CARGO_INCREMENTAL", "0")
        .env("UOR_NVCC_ARCH", &c.arch)
        .env("NVCC", "nvcc")
        .env("CUDA_COMPUTE_CAP", c.arch.trim_start_matches("compute_"))
        .env("UOR_LAB", &c.lab)
        .env("GIT_TERMINAL_PROMPT", "0");
    for name in [
        "GIT_TRACE",
        "GIT_TRACE_PACKET",
        "GIT_TRACE_CURL",
        "GIT_CURL_VERBOSE",
        "GIT_ASKPASS",
        "SSH_ASKPASS",
    ] {
        command.env_remove(name);
    }
    let status = match command.status() {
        Ok(status) => status,
        Err(e) => {
            record(&c.reports, stage, -1, start.elapsed().as_secs_f64())?;
            return Err(format!("cannot start {program}: {e}; receipts retained"));
        }
    };
    let exit = status.code().unwrap_or(-1);
    record(&c.reports, stage, exit, start.elapsed().as_secs_f64())?;
    if !status.success() {
        return Err(format!(
            "{stage} exited {exit}; inspect {} and {}; no cleanup performed",
            stdout_path.display(),
            stderr_path.display()
        ));
    }
    let length = fs::metadata(&stdout_path).map_err(|e| e.to_string())?.len();
    if length > 32 * 1024 * 1024 {
        return Err(format!(
            "{stage} stdout exceeds parser bound; complete receipt preserved"
        ));
    }
    fs::read_to_string(stdout_path).map_err(|e| e.to_string())
}

fn disk(c: &Config) -> Result<()> {
    let output = run(
        c,
        "disk",
        "df",
        &["-Pk", c.root.to_str().ok_or("non-UTF8 root")?],
        &c.root,
    )?;
    let row = output
        .lines()
        .last()
        .ok_or("df returned no filesystem row")?;
    let available: u64 = row
        .split_whitespace()
        .nth(3)
        .ok_or("df has no available KiB")?
        .parse()
        .map_err(|_| "invalid df available field")?;
    if available < 30 * 1024 * 1024 {
        return Err("pod filesystem below 30GiB free floor; no build started".into());
    }
    Ok(())
}

fn marker(c: &Config) -> String {
    format!(
        "lab={}\nsession={}\nsource={}\nrust={}\narch={}\nbuild_root={}\n",
        c.lab,
        c.session,
        c.sha,
        RUST_VERSION,
        c.arch,
        c.root.display()
    )
}
fn claim_directory(c: &Config, directory: &Path) -> Result<()> {
    let workspace =
        fs::canonicalize("/workspace").map_err(|e| format!("workspace missing: {e}"))?;
    // Reject every existing symlink component, before creating parent directories.
    let mut cursor = PathBuf::new();
    for component in directory.components() {
        cursor.push(component.as_os_str());
        if let Ok(m) = fs::symlink_metadata(&cursor) {
            if m.file_type().is_symlink() {
                return Err(
                    "root path contains a symlink; preserve it and choose a new root".into(),
                );
            }
        }
    }
    if directory.exists() {
        if !c.resume {
            return Err(
                "root exists; use a new root or explicit --resume with matching marker".into(),
            );
        }
        let current = fs::read_to_string(directory.join("OWNERSHIP.txt"))
            .map_err(|_| "existing root lacks ownership marker")?;
        if current != marker(c) {
            return Err(
                "existing root belongs to a different source/lab/session/toolchain; no mutation"
                    .into(),
            );
        }
    } else {
        if c.resume {
            return Err("cannot resume a root that does not exist".into());
        }
        fs::create_dir_all(directory.parent().ok_or("root has no parent")?)
            .map_err(|e| e.to_string())?;
        fs::create_dir(directory).map_err(|e| e.to_string())?;
        let actual = fs::canonicalize(directory).map_err(|e| e.to_string())?;
        if !actual.starts_with(workspace) {
            return Err("resolved root escaped workspace".into());
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join("OWNERSHIP.txt"))
            .map_err(|e| e.to_string())?;
        file.write_all(marker(c).as_bytes())
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn cuda_release(output: &str) -> Result<(u32, u32)> {
    let version = output
        .split("release ")
        .nth(1)
        .ok_or("nvcc has no release field")?
        .split(',')
        .next()
        .ok_or("nvcc release empty")?;
    let mut parts = version.split('.');
    let major = parts
        .next()
        .ok_or("missing CUDA major")?
        .parse()
        .map_err(|_| "bad CUDA major")?;
    let minor = parts
        .next()
        .ok_or("missing CUDA minor")?
        .parse()
        .map_err(|_| "bad CUDA minor")?;
    Ok((major, minor))
}

fn json_string(input: &str) -> String {
    let mut out = String::from("\"");
    for c in input.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// Read only the requested target's executable in Cargo's compiler-artifact event.
// Integration builds also emit ancillary binaries and build-script events.
// source/root paths admitted above are plain ASCII; unknown escapes fail closed.
fn executable(line: &str, target_name: &str) -> Result<Option<PathBuf>> {
    if !line.contains("\"reason\":\"compiler-artifact\"")
        || line.contains("\"kind\":[\"custom-build\"]")
        || !line.contains(&format!("\"name\":\"{target_name}\""))
    {
        return Ok(None);
    }
    let tail = match line.split_once("\"executable\":") {
        Some((_, tail)) => tail.trim_start(),
        None => return Ok(None),
    };
    if tail.starts_with("null") {
        return Ok(None);
    }
    if !tail.starts_with('"') {
        return Err("invalid Cargo executable field".into());
    }
    let mut chars = tail[1..].chars();
    let mut path = String::new();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Ok(Some(PathBuf::from(path))),
            '\\' => match chars.next() {
                Some('"') => path.push('"'),
                Some('\\') => path.push('\\'),
                Some('/') => path.push('/'),
                _ => return Err("unsupported executable path escape".into()),
            },
            c => path.push(c),
        }
    }
    Err("unclosed Cargo executable path".into())
}

fn hash_file(c: &Config, stage: &str, file: &Path) -> Result<String> {
    let output = run(
        c,
        stage,
        "sha256sum",
        &[file.to_str().ok_or("non-UTF8 artifact path")?],
        &c.root,
    )?;
    let digest = output.split_whitespace().next().ok_or("missing SHA256")?;
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|v| v.is_ascii_digit() || (b'a'..=b'f').contains(&v))
    {
        return Err("invalid sha256sum receipt".into());
    }
    Ok(digest.to_string())
}

fn preserve_ptx(c: &Config) -> Result<()> {
    let build = c.root.join("target/debug/build");
    let destination = c.reports.join(format!("ptx-{}", unix_seconds()));
    fs::create_dir(&destination).map_err(|e| e.to_string())?;
    let mut count = 0;
    for entry in fs::read_dir(&build).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        let out = entry.path().join("out");
        if !out.is_dir() {
            continue;
        }
        for item in fs::read_dir(&out).map_err(|e| e.to_string())? {
            let item = item.map_err(|e| e.to_string())?;
            let path = item.path();
            if path.extension().and_then(|v| v.to_str()) != Some("ptx") {
                continue;
            }
            if ["native_context.ptx", "selected_potential.ptx"]
                .contains(&item.file_name().to_string_lossy().as_ref())
            {
                return Err("quarantined project PTX found; A2 blocker".into());
            }
            if !item.file_type().map_err(|e| e.to_string())?.is_file() {
                return Err("PTX entry is not a regular file".into());
            }
            let length = item.metadata().map_err(|e| e.to_string())?.len();
            if length > 32 * 1024 * 1024 || count >= 32 {
                return Err("PTX preservation exceeds bounded 32 modules /32MiB each; retain original and inspect".into());
            }
            let parent = destination.join(entry.file_name());
            fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
            let copy = parent.join(item.file_name());
            fs::copy(&path, &copy).map_err(|e| e.to_string())?;
            hash_file(c, &format!("ptx-{count}-sha256"), &copy)?;
            count += 1;
        }
    }
    if count < 12 {
        return Err(format!("only {count} PTX files observed; expected nine vendor plus three project modules; inspect source/build"));
    }
    println!(
        "SETUP preserved {count} PTX modules at {}; module compliance NOT_PROVEN by copying",
        destination.display()
    );
    Ok(())
}

fn setup(c: &Config) -> Result<()> {
    claim_directory(c, &c.root)?;
    claim_directory(c, &c.reports)?;
    let os = fs::read_to_string("/etc/os-release").map_err(|e| e.to_string())?;
    fs::write(
        c.reports.join(format!("os-release-{}.txt", unix_seconds())),
        &os,
    )
    .map_err(|e| e.to_string())?;
    if !os.lines().any(|l| l == "ID=ubuntu" || l == "ID=\"ubuntu\"")
        || !os.contains("VERSION_ID=\"24.")
    {
        return Err(
            "qualification expects Ubuntu24 image; do not install a different OS/toolkit".into(),
        );
    }
    run(c, "uname", "uname", &["-a"], &c.root)?;
    let cuda = run(c, "nvcc", "nvcc", &["--version"], &c.root)?;
    if cuda_release(&cuda)? < (12, 8) {
        return Err("CUDA12.8+ devel toolkit required; no runtime/toolkit fallback".into());
    }
    run(
        c,
        "gpu-identity",
        "timeout",
        &[
            "--kill-after=2s",
            "20s",
            "nvidia-smi",
            "--query-gpu=name,driver_version,pci.bus_id,compute_cap",
            "--format=csv",
        ],
        &c.root,
    )?;
    disk(c)?;
    run(c, "rustup-version", "rustup", &["--version"], &c.root)?;
    run(
        c,
        "rust-toolchain",
        "rustup",
        &["toolchain", "install", RUST_VERSION, "--profile", "minimal"],
        &c.root,
    )?;
    run(
        c,
        "rustc-identity",
        "rustup",
        &["run", RUST_VERSION, "rustc", "-vV"],
        &c.root,
    )?;
    run(
        c,
        "cargo-identity",
        "rustup",
        &["run", RUST_VERSION, "cargo", "-V"],
        &c.root,
    )?;
    for tool in ["rustc", "cargo"] {
        let path = run(
            c,
            &format!("{tool}-path"),
            "rustup",
            &["which", "--toolchain", RUST_VERSION, tool],
            &c.root,
        )?;
        hash_file(c, &format!("{tool}-binary-sha256"), Path::new(path.trim()))?;
    }
    let own = env::current_exe().map_err(|e| e.to_string())?;
    hash_file(c, "setup-tool-sha256", &own)?;
    let source = c.root.join("source");
    if !source.exists() {
        fs::create_dir(&source).map_err(|e| e.to_string())?;
        run(c, "git-init", "git", &["init"], &source)?;
        run(
            c,
            "git-origin",
            "git",
            &["remote", "add", "origin", REPOSITORY],
            &source,
        )?;
    }
    if !source.join(".git").is_dir() {
        return Err(
            "owned source directory is not a git checkout; preserve and use new root".into(),
        );
    }
    // Resume never resets dirty research. FETCH_HEAD is only checked out for
    // a new checkout; an existing commit must already match the pinned source.
    let head = Command::new("git")
        .args(["rev-parse", "--verify", "HEAD"])
        .current_dir(&source)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| e.to_string())?;
    if !head.status.success() {
        run(
            c,
            "git-fetch",
            "git",
            &[
                "-c",
                "credential.helper=",
                "-c",
                "core.askpass=",
                "fetch",
                "--depth=1",
                "origin",
                &c.sha,
            ],
            &source,
        )?;
        run(
            c,
            "git-checkout",
            "git",
            &["checkout", "--detach", "FETCH_HEAD"],
            &source,
        )?;
    }
    let actual = run(c, "source-head", "git", &["rev-parse", "HEAD"], &source)?;
    if actual.trim() != c.sha {
        return Err("source checkout differs from pin; no reset or build".into());
    }
    let dirty = run(
        c,
        "source-status",
        "git",
        &["status", "--porcelain"],
        &source,
    )?;
    if !dirty.trim().is_empty() {
        return Err("source checkout dirty; preserve before repair".into());
    }
    run(
        c,
        "source-tree",
        "git",
        &["rev-parse", "HEAD^{tree}"],
        &source,
    )?;
    hash_file(c, "cargo-lock-sha256", &source.join("Cargo.lock"))?;
    let source_paths = run(
        c,
        "source-file-inventory",
        "git",
        &[
            "ls-files",
            "--",
            "Cargo.toml",
            "Cargo.lock",
            "crates/uor-r4-training",
            "crates/uor-r4-core",
            "crates/uor-r4-integer",
            "crates/uor-r4-source-interface",
            "crates/uor-r4-tokenizer",
            "third_party",
            "scripts/native_dependency_boundary.rs",
            "scripts/check_native_cuda_vendor.rs",
            "scripts/pod/qualification_bootstrap.rs",
            "scripts/pod/qualify_geometric.rs",
            "docs/harness/prep/m1-d29-vendor-finite-2026-10-13.md",
        ],
        &source,
    )?;
    // This is an identity inventory of retained source, not a claim that every
    // listed historical file is compiled. The dependency/PTX gates define that.
    let files: Vec<&str> = source_paths.lines().collect();
    if files.is_empty() || files.iter().any(|p| p.starts_with('-') || p.contains('\\')) {
        return Err("invalid source identity inventory".into());
    }
    run(c, "source-files-sha256", "sha256sum", &files, &source)?;
    // Fail before building any native executable if a historical teacher or
    // known dense backend is reintroduced transitively. Preserve the full
    // tree receipt, including on refusal, so the dependency edge is visible.
    let features = run(
        c,
        "resolved-cuda-features",
        "rustup",
        &[
            "run",
            RUST_VERSION,
            "cargo",
            "tree",
            "--locked",
            "-p",
            "uor-r4-training",
            "--features",
            "cuda",
            "--edges",
            "features",
        ],
        &source,
    )?;
    native_dependency_boundary::verify(&features)?;
    let builds: [(&str, &[&str]); 4] = [
        (
            "build-module-tests",
            &[
                "run",
                RUST_VERSION,
                "cargo",
                "test",
                "--locked",
                "-p",
                "uor-r4-training",
                "--features",
                "cuda",
                "--test",
                "d29_compiled_modules",
                "--no-run",
                "--message-format=json",
            ],
        ),
        (
            "build-library-tests",
            &[
                "run",
                RUST_VERSION,
                "cargo",
                "test",
                "--locked",
                "-p",
                "uor-r4-training",
                "--features",
                "cuda",
                "--lib",
                "--no-run",
                "--message-format=json",
            ],
        ),
        (
            "build-cli-tests",
            &[
                "run",
                RUST_VERSION,
                "cargo",
                "test",
                "--locked",
                "-p",
                "uor-r4-training",
                "--features",
                "cuda",
                "--example",
                "geometric-stack",
                "--no-run",
                "--message-format=json",
            ],
        ),
        (
            "build-normal-cli",
            &[
                "run",
                RUST_VERSION,
                "cargo",
                "build",
                "--locked",
                "-p",
                "uor-r4-training",
                "--features",
                "cuda",
                "--example",
                "geometric-stack",
                "--message-format=json",
            ],
        ),
    ];
    let mut artifacts = Vec::new();
    for (stage, args) in builds {
        disk(c)?;
        let output = run(c, stage, "rustup", args, &source)?;
        let mut found = Vec::new();
        let target_name = match stage {
            "build-module-tests" => "d29_compiled_modules",
            "build-library-tests" => "uor_r4_training",
            _ => "geometric-stack",
        };
        for line in output.lines() {
            if let Some(path) = executable(line, target_name)? {
                found.push(path);
            }
        }
        if found.len() != 1 {
            return Err(format!(
                "{stage}: expected exactly one executable event, got {}; inspect complete receipt",
                found.len()
            ));
        }
        let file = fs::canonicalize(&found[0]).map_err(|e| e.to_string())?;
        let target = fs::canonicalize(c.root.join("target")).map_err(|e| e.to_string())?;
        if !file.starts_with(target) || !file.is_file() {
            return Err("artifact is outside owned target".into());
        }
        let digest = hash_file(c, &format!("{stage}-sha256"), &file)?;
        let bytes = fs::metadata(&file).map_err(|e| e.to_string())?.len();
        run(
            c,
            &format!("{stage}-dynamic-imports"),
            "readelf",
            &["-dW", file.to_str().ok_or("non-UTF8 binary")?],
            &c.root,
        )?;
        run(
            c,
            &format!("{stage}-dynamic-symbols"),
            "readelf",
            &[
                "--dyn-syms",
                "--wide",
                file.to_str().ok_or("non-UTF8 binary")?,
            ],
            &c.root,
        )?;
        artifacts.push(format!(
            "{{\"role\":{},\"path\":{},\"sha256\":{},\"bytes\":{bytes}}}",
            json_string(stage),
            json_string(&file.to_string_lossy()),
            json_string(&digest)
        ));
    }
    preserve_ptx(c)?;
    let after = run(
        c,
        "source-status-after-build",
        "git",
        &["status", "--porcelain"],
        &source,
    )?;
    if !after.trim().is_empty() {
        return Err(
            "build changed pinned source despite --locked; preserve receipts and investigate"
                .into(),
        );
    }
    let manifest = format!("{{\"schema\":1,\"source\":{},\"lab\":{},\"session\":{},\"rust\":{},\"arch\":{},\"jobs\":{},\"status\":\"SETUP_ONLY\",\"tests\":\"NOT_RUN\",\"optimizer\":\"NOT_RUN\",\"training\":\"HELD\",\"cpu_model\":\"NOT_RUN\",\"artifacts\":[{}]}}\n", json_string(&c.sha), json_string(&c.lab), json_string(&c.session), json_string(RUST_VERSION), json_string(&c.arch), c.jobs, artifacts.join(","));
    fs::write(
        c.reports
            .join(format!("setup-manifest-{}.json", unix_seconds())),
        manifest,
    )
    .map_err(|e| e.to_string())?;
    println!("SETUP_ONLY complete: exact source/build receipts retained at {}; tests NOT_RUN, optimizer NOT_RUN, training HELD. Controller must freeze/audit PTX and execute only approved named controls separately.", c.reports.display());
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let config = match parse(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("argument refusal: {e}");
            process::exit(2);
        }
    };
    if config.check_only {
        println!("arguments valid; no filesystem/network/build/model action");
        return;
    }
    if let Err(e) = setup(&config) {
        eprintln!("SETUP BLOCKER: {e}; existing artifacts preserved; no test/model execution");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args() -> Vec<String> {
        [
            "--sha",
            "0123456789abcdef0123456789abcdef01234567",
            "--lab",
            "codex",
            "--session",
            "finite-a",
            "--root",
            "/workspace/codex-finite-a",
        ]
        .into_iter()
        .map(str::to_string)
        .collect()
    }
    #[test]
    fn arguments_are_metadata_only() {
        let c = parse(&args()).expect("valid");
        assert_eq!(c.jobs, 4);
        assert!(!c.resume);
    }
    #[test]
    fn bad_paths_and_source_are_refused() {
        let mut a = args();
        a[1] = "main".into();
        assert!(parse(&a).is_err());
        a = args();
        a[7] = "/workspace/../root".into();
        assert!(parse(&a).is_err());
        a[7] = "/workspace".into();
        assert!(parse(&a).is_err());
    }
    #[test]
    fn parser_never_treats_null_or_build_script_as_a_test_artifact() {
        assert!(
            executable("{\"reason\":\"compiler-artifact\",\"target\":{\"name\":\"selected\"},\"executable\":null}", "selected")
                .expect("null")
                .is_none()
        );
        let a = executable(
            "{\"reason\":\"compiler-artifact\",\"target\":{\"name\":\"selected\"},\"executable\":\"/workspace/x/target/debug/t\"}",
            "selected",
        )
        .expect("valid")
        .expect("path");
        assert_eq!(a, PathBuf::from("/workspace/x/target/debug/t"));
        assert!(executable("{\"reason\":\"compiler-artifact\",\"target\":{\"name\":\"ancillary\"},\"executable\":\"/workspace/x/target/debug/other\"}", "selected").expect("other target").is_none());
        assert!(executable("{\"reason\":\"compiler-artifact\",\"target\":{\"name\":\"selected\",\"kind\":[\"custom-build\"]},\"executable\":\"/workspace/x/target/debug/build-script\"}", "selected").expect("build script").is_none());
    }
    #[test]
    fn cuda_version_check_is_metadata_only() {
        assert_eq!(
            cuda_release("Cuda compilation tools, release 12.8, V12.8.93").expect("release"),
            (12, 8)
        );
        assert!(cuda_release("toolkit unavailable").is_err());
    }
}
