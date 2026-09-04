//! Stable-handle execution and owned-child termination for the macOS target.

use crate::{BoxError, TARGET};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::Child;
#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub const EXECUTABLE_CAP: u64 = 268_435_456;
pub const INHERITED_EXECUTABLE_FD: i32 = 3;

pub struct VerifiedExecutable {
    path: PathBuf,
    file: File,
    bytes: u64,
    sha256: String,
}

impl VerifiedExecutable {
    pub fn open_current() -> Result<Self, BoxError> {
        let path = std::env::current_exe()?;
        Self::open(&path)
    }

    pub fn open(path: &Path) -> Result<Self, BoxError> {
        if !supported_target() {
            return Err(format!("unsupported runtime target; required {TARGET}").into());
        }
        let mut file = open_regular_nofollow(path)?;
        let metadata = file.metadata()?;
        if !metadata.is_file() || metadata.len() == 0 || metadata.len() > EXECUTABLE_CAP {
            return Err("executable length rejected".into());
        }
        let sha256 = hash_open_file(&mut file, EXECUTABLE_CAP)?;
        Ok(Self {
            path: path.to_path_buf(),
            file,
            bytes: metadata.len(),
            sha256,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn bytes(&self) -> u64 {
        self.bytes
    }

    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    /// Execute the exact open vnode through macOS `/dev/fd`, and retain an
    /// inherited descriptor at fd 3 for the child to hash before model work.
    #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
    pub fn spawn_worker(&self) -> Result<Child, BoxError> {
        use std::os::fd::AsRawFd;
        use std::os::unix::process::CommandExt;

        let source_fd = self.file.as_raw_fd();
        // Safety: getdtablesize has no arguments and only reads the current
        // process descriptor-table limit. Capture it before fork so the
        // pre-exec closure needs only async-signal-safe descriptor operations.
        let descriptor_limit = unsafe { libc::getdtablesize() };
        if descriptor_limit <= INHERITED_EXECUTABLE_FD {
            return Err("descriptor-table limit cannot preserve executable fd 3".into());
        }
        let executable = format!("/dev/fd/{source_fd}");
        let mut command = Command::new(executable);
        command
            .arg("--internal-worker")
            .env_clear()
            .env("LANG", "en_US.UTF-8")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // Safety: after fork this closure invokes only async-signal-safe fcntl
        // and dup2. It reports failure through io::Error and touches no shared
        // Rust state. The open source descriptor stays alive through spawn.
        unsafe {
            command.pre_exec(move || {
                if source_fd != INHERITED_EXECUTABLE_FD
                    && libc::dup2(source_fd, INHERITED_EXECUTABLE_FD) < 0
                {
                    return Err(std::io::Error::last_os_error());
                }
                // The child keeps only stdio and the verified executable at
                // fd 3. Mark every other possible descriptor close-on-exec so
                // caller descriptors, listener sockets and credentials cannot
                // cross the exec boundary even if the parent inherited them
                // without CLOEXEC.
                for descriptor in (INHERITED_EXECUTABLE_FD + 1)..descriptor_limit {
                    if libc::fcntl(descriptor, libc::F_SETFD, libc::FD_CLOEXEC) < 0 {
                        let error = std::io::Error::last_os_error();
                        if error.raw_os_error() != Some(libc::EBADF) {
                            return Err(error);
                        }
                    }
                }
                if libc::fcntl(INHERITED_EXECUTABLE_FD, libc::F_SETFD, 0) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        Ok(command.spawn()?)
    }

    #[cfg(not(all(target_arch = "aarch64", target_os = "macos")))]
    pub fn spawn_worker(&self) -> Result<Child, BoxError> {
        let _ = self;
        Err(format!("unsupported runtime target; required {TARGET}").into())
    }
}

pub fn supported_target() -> bool {
    cfg!(all(target_arch = "aarch64", target_os = "macos"))
}

pub fn rust_target() -> &'static str {
    if supported_target() {
        TARGET
    } else {
        "unsupported"
    }
}

#[cfg(unix)]
fn open_regular_nofollow(path: &Path) -> Result<File, BoxError> {
    use std::os::unix::fs::OpenOptionsExt;
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("executable must be a regular non-symlink file".into());
    }
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    if !file.metadata()?.is_file() {
        return Err("executable changed during open".into());
    }
    Ok(file)
}

#[cfg(not(unix))]
fn open_regular_nofollow(_path: &Path) -> Result<File, BoxError> {
    Err(format!("unsupported runtime target; required {TARGET}").into())
}

pub fn hash_open_file(file: &mut File, cap: u64) -> Result<String, BoxError> {
    file.seek(SeekFrom::Start(0))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65_536];
    let mut total = 0u64;
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        total = total.checked_add(read as u64).ok_or("length overflow")?;
        if total > cap {
            return Err("file exceeds hash cap".into());
        }
        hasher.update(&buffer[..read]);
    }
    file.seek(SeekFrom::Start(0))?;
    Ok(format!("{:x}", hasher.finalize()))
}

/// Hash the exact inherited image handle. Private model modes fail closed if
/// the accepted launcher did not provide it.
#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
pub fn hash_inherited_executable() -> Result<(String, u64), BoxError> {
    use std::os::fd::FromRawFd;
    // Safety: fcntl returns a new owned descriptor or -1. The original fd 3
    // remains open and is never wrapped or closed here.
    let duplicated = unsafe { libc::fcntl(INHERITED_EXECUTABLE_FD, libc::F_DUPFD_CLOEXEC, 10) };
    if duplicated < 0 {
        return Err("missing accepted inherited executable handle".into());
    }
    // Safety: duplicated is a new descriptor owned by this File.
    let mut file = unsafe { File::from_raw_fd(duplicated) };
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > EXECUTABLE_CAP {
        return Err("inherited executable rejected".into());
    }
    let sha = hash_open_file(&mut file, EXECUTABLE_CAP)?;
    Ok((sha, metadata.len()))
}

#[cfg(not(all(target_arch = "aarch64", target_os = "macos")))]
pub fn hash_inherited_executable() -> Result<(String, u64), BoxError> {
    Err(format!("unsupported runtime target; required {TARGET}").into())
}

pub fn terminate_owned(child: &mut Child) -> Result<(), BoxError> {
    if child.try_wait()?.is_some() {
        return Ok(());
    }
    #[cfg(unix)]
    {
        // Safety: `id` is taken from the owned Child and no PID search occurs.
        if unsafe { libc::kill(child.id() as i32, libc::SIGTERM) } != 0 {
            let error = std::io::Error::last_os_error();
            if child.try_wait()?.is_none() {
                return Err(error.into());
            }
        }
    }
    #[cfg(not(unix))]
    child.kill()?;

    let graceful_deadline = Instant::now() + Duration::from_millis(2_000);
    while Instant::now() < graceful_deadline {
        if child.try_wait()?.is_some() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(10));
    }
    child.kill()?;
    let forced_deadline = Instant::now() + Duration::from_millis(2_000);
    while Instant::now() < forced_deadline {
        if child.try_wait()?.is_some() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(10));
    }
    Err("owned child termination could not be confirmed".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_builds_never_claim_the_accepted_target() {
        if !cfg!(all(target_arch = "aarch64", target_os = "macos")) {
            assert_eq!(rust_target(), "unsupported");
        }
    }

    #[test]
    fn executable_cap_matches_the_service_contract() {
        assert_eq!(EXECUTABLE_CAP, 268_435_456);
        assert_eq!(INHERITED_EXECUTABLE_FD, 3);
    }
}
