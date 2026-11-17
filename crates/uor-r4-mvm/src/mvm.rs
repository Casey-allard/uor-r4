//! `MVM-Wasm` RV64IMAC 64-bit Linux MicroVM and `SpiralCore` `virtio-vsock` zero-NIC switch (`R4-37`).
//!
//! Implements a deterministic, zero-`unsafe` 64-bit RISC-V (`RV64IMAC` + `Zicsr`) processor,
//! multi-process Linux `riscv64` system call executive, 64 KB Copy-on-Write block storage
//! backed by the cross-compiled `mvm_initramfs.bin` (`UORMVM01`), in-sandbox C-to-ELF64
//! compilation (`mvm_cc`), zero-NIC guest socket isolation (`AF_VSOCK = 40` only), and
//! host secret-placeholder substitution that keeps raw credentials out of guest RAM.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::rc::Rc;

use sha2::{Digest, Sha256};

use crate::spiralcore_vsock::r4_spiralcore_vsock::{
    admit_guest_socket, admit_host_egress, admit_spiralcore_frame,
    EgressPolicyCheck, EgressVerdict, ElfAdmission, FrameVerdict,
    SocketAdmission, SpiralCoreFrame,
};
use crate::mvm_cc::{compile_c_to_rv64_elf, CCompileError};

/// Four-byte magic prefix for 16-byte `SpiralCore` `virtio-vsock` frames.
pub const SPIRALCORE_MAGIC: [u8; 4] = *b"SPVS";
/// Maximum payload bytes admitted in a single `SpiralCore` `virtio-vsock` frame.
pub const SPIRALCORE_MAX_PAYLOAD: usize = 65_536;
/// Copy-on-Write virtual block sector size (64 KB).
pub const MVM_COW_SECTOR_BYTES: usize = 65_536;
/// Flat guest physical memory size per process (8 MB).
const GUEST_RAM_BYTES: usize = 8 * 1024 * 1024;
/// Initial top of guest user stack.
const GUEST_STACK_TOP: u64 = 0x007E_0000;
/// Base virtual address for anonymous `mmap` allocations.
const GUEST_MMAP_BASE: u64 = 0x0040_0000;
/// Instruction fuel cap per command invocation.
const MAX_INSTRUCTION_FUEL: u64 = 25_000_000;

/// Embedded `UORMVM01` initramfs archive containing genuine cross-compiled `riscv64` ELF64 userland.
static INITRAMFS_IMAGE: &[u8] = crate::mvm_initramfs::MVM_INITRAMFS;

/// Linux `EAFNOSUPPORT` (`97`) negated for `riscv64` kernel error return when NIC domains are denied.
const LINUX_EAFNOSUPPORT: i64 = -97;
/// Linux `ENOENT` (`2`) negated.
const LINUX_ENOENT: i64 = -2;
/// Linux `EBADF` (`9`) negated.
const LINUX_EBADF: i64 = -9;
/// Linux `EINVAL` (`22`) negated.
const LINUX_EINVAL: i64 = -22;
/// Linux `ENOSYS` (`38`) negated.
const LINUX_ENOSYS: i64 = -38;

/// Typed runtime error returned by `MVM-Wasm` operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MvmError {
    /// Initramfs image header or SHA-256 entry integrity failed.
    CorruptInitramfs(String),
    /// ELF64 header failed `R4.SpiralCoreVsock.admitRv64Elf`.
    ElfRejected(ElfAdmission),
    /// `SpiralCore` 16-byte frame failed `R4.SpiralCoreVsock.admitSpiralCoreFrame`.
    FrameRejected(FrameVerdict),
    /// Guest execution exceeded deterministic instruction fuel or trapped on invalid memory/opcode.
    CpuTrap(String),
    /// In-sandbox C compilation failed.
    CompileError(CCompileError),
    /// Requested guest file or executable was not found.
    NotFound(String),
}

impl std::fmt::Display for MvmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CorruptInitramfs(m) => write!(f, "corrupt initramfs: {m}"),
            Self::ElfRejected(v) => write!(f, "ELF64 rejected by R4-37 kernel: {v:?}"),
            Self::FrameRejected(v) => write!(f, "SpiralCore frame rejected: {v:?}"),
            Self::CpuTrap(m) => write!(f, "RV64IMAC trap: {m}"),
            Self::CompileError(e) => write!(f, "{e}"),
            Self::NotFound(p) => write!(f, "file not found: {p}"),
        }
    }
}

impl std::error::Error for MvmError {}

/// Exact 16-byte `SpiralCore` `virtio-vsock` packet header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpiralCoreHeader {
    /// Four-byte magic (`*b"SPVS"`).
    pub magic: [u8; 4],
    /// Source Context ID (`CID`, `2` = host switch, `>= 3` = MicroVM guest).
    pub src_cid: u16,
    /// Destination Context ID (`CID`).
    pub dst_cid: u16,
    /// `virtio-vsock` destination port.
    pub vsock_port: u16,
    /// Clifford `Cl(0,6)` spinor action (`0..63`).
    pub cl06_spin: u8,
    /// $E_8$ root orbit cell (`0..15`).
    pub e8_orbit: u8,
    /// Byte length of payload immediately following the 16-byte header.
    pub payload_len: u32,
}

impl SpiralCoreHeader {
    /// Encode header to its canonical 16-byte little-endian wire representation.
    pub fn to_bytes(self) -> [u8; 16] {
        let mut out = [0u8; 16];
        out[0..4].copy_from_slice(&self.magic);
        out[4..6].copy_from_slice(&self.src_cid.to_le_bytes());
        out[6..8].copy_from_slice(&self.dst_cid.to_le_bytes());
        out[8..10].copy_from_slice(&self.vsock_port.to_le_bytes());
        out[10] = self.cl06_spin;
        out[11] = self.e8_orbit;
        out[12..16].copy_from_slice(&self.payload_len.to_le_bytes());
        out
    }

    /// Decode a 16-byte `SpiralCore` header slice.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 16 {
            return None;
        }
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&bytes[0..4]);
        Some(Self {
            magic,
            src_cid: u16::from_le_bytes([bytes[4], bytes[5]]),
            dst_cid: u16::from_le_bytes([bytes[6], bytes[7]]),
            vsock_port: u16::from_le_bytes([bytes[8], bytes[9]]),
            cl06_spin: bytes[10],
            e8_orbit: bytes[11],
            payload_len: u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
        })
    }
}

/// Complete `SpiralCore` `virtio-vsock` packet (16-byte header + payload).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VsockPacket {
    /// Verified 16-byte header.
    pub header: SpiralCoreHeader,
    /// Raw payload bytes.
    pub payload: Vec<u8>,
}

/// Result of a host egress secret-placeholder proxy check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EgressProxyOutcome {
    /// Formal `R4.SpiralCoreVsock.EgressVerdict`.
    pub verdict: EgressVerdict,
    /// Substituted outbound payload (present only when `verdict == ProxyAdmitted`).
    pub outbound_payload: Option<String>,
}

/// Copy-on-Write snapshot of the MicroVM filesystem and 64 KB dirty block overlay.
#[derive(Debug, Clone)]
pub struct MvmCowSnapshot {
    /// FIPS 180-4 SHA-256 hex digest of the logical filesystem and CoW blocks.
    pub state_sha256: String,
    inodes: BTreeMap<String, GuestInode>,
    dirs: BTreeSet<String>,
    cow_sectors: BTreeMap<u32, Vec<u8>>,
    dirty_sectors: BTreeSet<u32>,
    next_sector: u32,
}

/// Structured result of executing a command chain inside `MVM-Wasm`.
#[derive(Debug, Clone)]
pub struct MvmExecOutcome {
    /// Final process exit code (`0` = success, `127` = command not found).
    pub exit_code: i32,
    /// Captured standard output UTF-8 text.
    pub stdout: String,
    /// Captured standard error UTF-8 text.
    pub stderr: String,
    /// Total `RV64IMAC` instructions retired across spawned processes.
    pub instructions_retired: u64,
    /// `SpiralCore` `AF_VSOCK` packets emitted during execution.
    pub vsock_frames: Vec<VsockPacket>,
    /// `e_machine` of any ELF64 binary compiled during this invocation (`243` for `EM_RISCV`).
    pub compiled_elf_machine: Option<u16>,
    /// Updated SHA-256 digest of the CoW block state.
    pub cow_state_sha256: String,
}

/// Admit a guest `socket(domain, ...)` domain code via `R4.SpiralCoreVsock.admitGuestSocket`.
pub fn admit_guest_socket_domain(domain_code: u64) -> SocketAdmission {
    admit_guest_socket(domain_code)
}

/// Admit a 16-byte `SpiralCore` header via `R4.SpiralCoreVsock.admitSpiralCoreFrame`.
pub fn admit_spiralcore_header(hdr: &SpiralCoreHeader, max_payload_bytes: usize) -> FrameVerdict {
    if hdr.magic != SPIRALCORE_MAGIC {
        return FrameVerdict::RefusedInvalidCid;
    }
    admit_spiralcore_frame(SpiralCoreFrame {
        src_cid: hdr.src_cid as u64,
        dst_cid: hdr.dst_cid as u64,
        vsock_port: hdr.vsock_port as u64,
        cl06_spin: hdr.cl06_spin as u64,
        e8_orbit: hdr.e8_orbit as u64,
        payload_bytes: hdr.payload_len as u64,
        max_payload_bytes: max_payload_bytes as u64,
    })
}

/// Admit a 64-bit `EM_RISCV` ELF64 image via `R4.SpiralCoreVsock.admitRv64Elf`.
pub fn admit_elf64_riscv(bytes: &[u8]) -> ElfAdmission {
    match crate::elf::parse(bytes, GUEST_RAM_BYTES) {
        Ok(_) => ElfAdmission::ElfAdmitted,
        Err(refusal) => refusal,
    }
}

#[derive(Debug, Clone)]
enum InodeKind {
    Regular { sector_start: u32, byte_len: usize },
    Symlink { target: String },
}

#[derive(Debug, Clone)]
struct GuestInode {
    mode: u32,
    kind: InodeKind,
}

#[derive(Debug, Clone)]
struct HostSecretRule {
    raw_secret: String,
    allowlisted_hosts: BTreeSet<String>,
}

#[derive(Clone)]
enum FdKind {
    StdinBuffer(Rc<RefCell<VecDeque<u8>>>),
    StdoutCapture(Rc<RefCell<Vec<u8>>>),
    StderrCapture(Rc<RefCell<Vec<u8>>>),
    PipeRead(Rc<RefCell<VecDeque<u8>>>),
    PipeWrite(Rc<RefCell<VecDeque<u8>>>),
    RegularFile {
        path: String,
        offset: usize,
        writable: bool,
        append: bool,
    },
    Directory {
        entries: Vec<(u8, String)>,
        index: usize,
    },
    VsockSocket {
        peer_cid: u16,
        peer_port: u16,
        rx_queue: VecDeque<Vec<u8>>,
    },
}

#[derive(Clone)]
struct Rv64Process {
    pid: i64,
    regs: [u64; 32],
    pc: u64,
    ram: Vec<u8>,
    brk_addr: u64,
    mmap_cursor: u64,
    lr_reservation: Option<u64>,
    fds: BTreeMap<i32, FdKind>,
    next_fd: i32,
    cwd: String,
    exited: Option<i32>,
}

impl Rv64Process {
    fn read_u8(&self, addr: u64) -> Result<u8, MvmError> {
        let idx = addr as usize;
        self.ram
            .get(idx)
            .copied()
            .ok_or_else(|| MvmError::CpuTrap(format!("load u8 fault at 0x{addr:x}")))
    }

    fn read_u16(&self, addr: u64) -> Result<u16, MvmError> {
        let idx = addr as usize;
        if idx + 2 > self.ram.len() {
            return Err(MvmError::CpuTrap(format!("load u16 fault at 0x{addr:x}")));
        }
        Ok(u16::from_le_bytes([self.ram[idx], self.ram[idx + 1]]))
    }

    fn read_u32(&self, addr: u64) -> Result<u32, MvmError> {
        let idx = addr as usize;
        if idx + 4 > self.ram.len() {
            return Err(MvmError::CpuTrap(format!("load u32 fault at 0x{addr:x}")));
        }
        Ok(u32::from_le_bytes([
            self.ram[idx],
            self.ram[idx + 1],
            self.ram[idx + 2],
            self.ram[idx + 3],
        ]))
    }

    fn read_u64(&self, addr: u64) -> Result<u64, MvmError> {
        let idx = addr as usize;
        if idx + 8 > self.ram.len() {
            return Err(MvmError::CpuTrap(format!("load u64 fault at 0x{addr:x}")));
        }
        let mut b = [0u8; 8];
        b.copy_from_slice(&self.ram[idx..idx + 8]);
        Ok(u64::from_le_bytes(b))
    }

    fn write_bytes(&mut self, addr: u64, data: &[u8]) -> Result<(), MvmError> {
        let idx = addr as usize;
        if idx.saturating_add(data.len()) > self.ram.len() {
            return Err(MvmError::CpuTrap(format!(
                "store fault at 0x{addr:x} len={}",
                data.len()
            )));
        }
        self.ram[idx..idx + data.len()].copy_from_slice(data);
        Ok(())
    }

    fn read_cstr(&self, addr: u64) -> Result<String, MvmError> {
        let mut idx = addr as usize;
        let mut bytes = Vec::new();
        while idx < self.ram.len() && bytes.len() < 4096 {
            let b = self.ram[idx];
            if b == 0 {
                return Ok(String::from_utf8_lossy(&bytes).into_owned());
            }
            bytes.push(b);
            idx += 1;
        }
        Err(MvmError::CpuTrap(format!(
            "unterminated cstr at 0x{addr:x}"
        )))
    }
}

/// Complete `MVM-Wasm` RV64IMAC Linux MicroVM instance with CoW storage and `SpiralCore` switch.
pub struct MvmSandbox {
    guest_cid: u16,
    inodes: BTreeMap<String, GuestInode>,
    dirs: BTreeSet<String>,
    cow_sectors: BTreeMap<u32, Vec<u8>>,
    dirty_sectors: BTreeSet<u32>,
    next_sector: u32,
    next_pid: i64,
    host_secrets: BTreeMap<String, HostSecretRule>,
    vsock_inbox: BTreeMap<u16, VecDeque<VsockPacket>>,
    last_guest_ram: Vec<u8>,
}

impl MvmSandbox {
    /// Boot a fresh `MVM-Wasm` MicroVM with default guest `CID = 3` from `mvm_initramfs.bin`.
    pub fn boot_default() -> Result<Self, MvmError> {
        Self::boot_with_cid(3)
    }

    /// Boot a fresh `MVM-Wasm` MicroVM assigned to `guest_cid` (`>= 3`).
    pub fn boot_with_cid(guest_cid: u16) -> Result<Self, MvmError> {
        if guest_cid < 3 {
            return Err(MvmError::FrameRejected(FrameVerdict::RefusedInvalidCid));
        }
        let mut dirs = BTreeSet::new();
        for d in [
            "/",
            "/bin",
            "/sbin",
            "/usr",
            "/usr/bin",
            "/workspace",
            "/tmp",
            "/etc",
        ] {
            dirs.insert(d.to_string());
        }
        let mut vm = Self {
            guest_cid,
            inodes: BTreeMap::new(),
            dirs,
            cow_sectors: BTreeMap::new(),
            dirty_sectors: BTreeSet::new(),
            next_sector: 0,
            next_pid: 100,
            host_secrets: BTreeMap::new(),
            vsock_inbox: BTreeMap::new(),
            last_guest_ram: Vec::new(),
        };
        vm.mount_initramfs(INITRAMFS_IMAGE)?;
        vm.dirty_sectors.clear();
        Ok(vm)
    }

    /// Guest `virtio-vsock` Context ID (`CID`).
    pub fn guest_cid(&self) -> u16 {
        self.guest_cid
    }

    /// Canonical guest ISA identifier.
    pub fn architecture(&self) -> &'static str {
        "riscv64-rv64imac"
    }

    /// Count of 64 KB Copy-on-Write sectors dirtied since boot or last snapshot restore.
    pub fn dirty_cow_sector_count(&self) -> usize {
        self.dirty_sectors.len()
    }

    /// Export dirty 64 KB CoW sectors `(sector_id, bytes)` for Worker OPFS persistence.
    pub fn export_dirty_cow_sectors(&self) -> Vec<(u32, Vec<u8>)> {
        self.dirty_sectors
            .iter()
            .filter_map(|id| self.cow_sectors.get(id).map(|b| (*id, b.clone())))
            .collect()
    }

    /// Hydrate 64 KB CoW sectors loaded from OPFS `FileSystemSyncAccessHandle`.
    pub fn import_cow_sectors(&mut self, sectors: &[(u32, Vec<u8>)]) {
        for (id, data) in sectors {
            if data.len() == MVM_COW_SECTOR_BYTES {
                self.cow_sectors.insert(*id, data.clone());
                self.dirty_sectors.insert(*id);
                if *id >= self.next_sector {
                    self.next_sector = *id + 1;
                }
            }
        }
    }

    /// Compute the FIPS 180-4 SHA-256 hex digest of the logical filesystem and CoW state.
    pub fn cow_state_sha256(&self) -> String {
        let mut h = Sha256::new();
        h.update(b"UOR_MVM_COW_V1:");
        for d in &self.dirs {
            h.update(b"D:");
            h.update(d.as_bytes());
            h.update([0]);
        }
        for (path, inode) in &self.inodes {
            h.update(b"F:");
            h.update(path.as_bytes());
            h.update(inode.mode.to_le_bytes());
            match &inode.kind {
                InodeKind::Symlink { target } => {
                    h.update(b"L:");
                    h.update(target.as_bytes());
                }
                InodeKind::Regular { .. } => {
                    if let Some(bytes) = self.read_guest_file(path) {
                        h.update((bytes.len() as u64).to_le_bytes());
                        h.update(&bytes);
                    }
                }
            }
        }
        let digest = h.finalize();
        let mut out = String::with_capacity(64);
        for b in digest {
            use std::fmt::Write;
            let _ = write!(&mut out, "{b:02x}");
        }
        out
    }

    /// Capture an O(1)-metadata Copy-on-Write snapshot of the MicroVM storage layer.
    pub fn snapshot_cow(&self) -> MvmCowSnapshot {
        MvmCowSnapshot {
            state_sha256: self.cow_state_sha256(),
            inodes: self.inodes.clone(),
            dirs: self.dirs.clone(),
            cow_sectors: self.cow_sectors.clone(),
            dirty_sectors: self.dirty_sectors.clone(),
            next_sector: self.next_sector,
        }
    }

    /// Roll back the MicroVM storage layer to a previously captured CoW snapshot.
    pub fn restore_cow(&mut self, snap: &MvmCowSnapshot) {
        self.inodes = snap.inodes.clone();
        self.dirs = snap.dirs.clone();
        self.cow_sectors = snap.cow_sectors.clone();
        self.dirty_sectors = snap.dirty_sectors.clone();
        self.next_sector = snap.next_sector;
    }

    fn mount_initramfs(&mut self, img: &[u8]) -> Result<(), MvmError> {
        if img.len() < 12 || &img[0..8] != b"UORMVM01" {
            return Err(MvmError::CorruptInitramfs("bad magic".to_string()));
        }
        let count = u32::from_le_bytes([img[8], img[9], img[10], img[11]]) as usize;
        let mut cur = 12usize;
        for _ in 0..count {
            if cur + 42 > img.len() {
                return Err(MvmError::CorruptInitramfs(
                    "truncated entry header".to_string(),
                ));
            }
            let path_len = u16::from_le_bytes([img[cur], img[cur + 1]]) as usize;
            let mode = u32::from_le_bytes([img[cur + 2], img[cur + 3], img[cur + 4], img[cur + 5]]);
            let data_len =
                u32::from_le_bytes([img[cur + 6], img[cur + 7], img[cur + 8], img[cur + 9]])
                    as usize;
            let expected_sha = &img[cur + 10..cur + 42];
            cur += 42;
            if cur + path_len + data_len > img.len() {
                return Err(MvmError::CorruptInitramfs(
                    "truncated entry payload".to_string(),
                ));
            }
            let path = std::str::from_utf8(&img[cur..cur + path_len])
                .map_err(|_| MvmError::CorruptInitramfs("invalid utf8 path".to_string()))?
                .to_string();
            cur += path_len;
            let data = &img[cur..cur + data_len];
            cur += data_len;

            let mut h = Sha256::new();
            h.update(data);
            if h.finalize().as_slice() != expected_sha {
                return Err(MvmError::CorruptInitramfs(format!(
                    "SHA-256 mismatch for {path}"
                )));
            }

            if (mode & 0o170000) == 0o120000 {
                let target = String::from_utf8_lossy(data).into_owned();
                self.inodes.insert(
                    path,
                    GuestInode {
                        mode,
                        kind: InodeKind::Symlink { target },
                    },
                );
            } else {
                self.write_inode_with_mode(&path, data, mode)?;
            }
        }
        Ok(())
    }

    fn normalize_guest_path(cwd: &str, raw: &str) -> String {
        let joined = if raw.starts_with('/') {
            raw.to_string()
        } else if cwd == "/" {
            format!("/{raw}")
        } else {
            format!("{cwd}/{raw}")
        };
        let mut stack: Vec<&str> = Vec::new();
        for seg in joined.split('/') {
            if seg.is_empty() || seg == "." {
                continue;
            }
            if seg == ".." {
                stack.pop();
            } else {
                stack.push(seg);
            }
        }
        if stack.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", stack.join("/"))
        }
    }

    fn ensure_parent_dirs(&mut self, norm_path: &str) {
        let mut prefix = String::new();
        let parts: Vec<&str> = norm_path.split('/').filter(|s| !s.is_empty()).collect();
        if parts.len() <= 1 {
            return;
        }
        for seg in &parts[..parts.len() - 1] {
            prefix.push('/');
            prefix.push_str(seg);
            self.dirs.insert(prefix.clone());
        }
    }

    fn write_inode_with_mode(
        &mut self,
        path: &str,
        data: &[u8],
        mode: u32,
    ) -> Result<(), MvmError> {
        let norm = Self::normalize_guest_path("/", path);
        self.ensure_parent_dirs(&norm);
        let needed_sectors = data.len().div_ceil(MVM_COW_SECTOR_BYTES).max(1);
        let start = self.next_sector;
        for i in 0..needed_sectors {
            let sec_id = start + (i as u32);
            let mut sector = vec![0u8; MVM_COW_SECTOR_BYTES];
            let chunk_off = i * MVM_COW_SECTOR_BYTES;
            if chunk_off < data.len() {
                let end = (chunk_off + MVM_COW_SECTOR_BYTES).min(data.len());
                sector[0..(end - chunk_off)].copy_from_slice(&data[chunk_off..end]);
            }
            self.cow_sectors.insert(sec_id, sector);
            self.dirty_sectors.insert(sec_id);
        }
        self.next_sector = start + (needed_sectors as u32);
        self.inodes.insert(
            norm,
            GuestInode {
                mode,
                kind: InodeKind::Regular {
                    sector_start: start,
                    byte_len: data.len(),
                },
            },
        );
        Ok(())
    }

    /// Write a regular file into the guest CoW filesystem (`0o100644` default permissions).
    pub fn write_guest_file(&mut self, path: &str, data: &[u8]) -> Result<(), MvmError> {
        let mode = if data.starts_with(b"\x7fELF") {
            0o100755
        } else {
            0o100644
        };
        self.write_inode_with_mode(path, data, mode)
    }

    /// Read a file (resolving up to 8 symlink hops) from the guest CoW filesystem.
    pub fn read_guest_file(&self, path: &str) -> Option<Vec<u8>> {
        let mut cur = Self::normalize_guest_path("/", path);
        for _ in 0..8 {
            let inode = self.inodes.get(&cur)?;
            match &inode.kind {
                InodeKind::Symlink { target } => {
                    cur = Self::normalize_guest_path("/", target);
                }
                InodeKind::Regular {
                    sector_start,
                    byte_len,
                } => {
                    let mut out = Vec::with_capacity(*byte_len);
                    let mut rem = *byte_len;
                    let mut sec = *sector_start;
                    while rem > 0 {
                        let sector = self.cow_sectors.get(&sec)?;
                        let take = rem.min(MVM_COW_SECTOR_BYTES);
                        out.extend_from_slice(&sector[0..take]);
                        rem -= take;
                        sec += 1;
                    }
                    return Some(out);
                }
            }
        }
        None
    }

    /// Synchronize host Studio `/workspace` text files into the guest `/workspace` directory.
    pub fn sync_from_workspace_map(&mut self, workspace_files: &BTreeMap<String, String>) {
        for (rel, content) in workspace_files {
            let clean = rel.trim_start_matches('/');
            let guest_path = format!("/workspace/{clean}");
            let _ = self.write_guest_file(&guest_path, content.as_bytes());
        }
    }

    /// Export UTF-8 files under guest `/workspace` back to Studio's workspace map.
    pub fn export_workspace_text_files(&self) -> BTreeMap<String, String> {
        let mut out = BTreeMap::new();
        for path in self.inodes.keys() {
            if let Some(rel) = path.strip_prefix("/workspace/") {
                if let Some(bytes) = self.read_guest_file(path) {
                    if !bytes.starts_with(b"\x7fELF") {
                        if let Ok(text) = String::from_utf8(bytes) {
                            out.insert(rel.to_string(), text);
                        }
                    }
                }
            }
        }
        out
    }

    /// Register a host-only credential placeholder (`{{MVM_SECRET:<name>}}`) restricted to `allowlisted_hosts`.
    pub fn register_host_secret(
        &mut self,
        name: &str,
        raw_secret: &str,
        allowlisted_hosts: &[&str],
    ) {
        let hosts = allowlisted_hosts.iter().map(|s| (*s).to_string()).collect();
        self.host_secrets.insert(
            name.to_string(),
            HostSecretRule {
                raw_secret: raw_secret.to_string(),
                allowlisted_hosts: hosts,
            },
        );
    }

    /// Check whether `needle` bytes appear anywhere in the most recently executed guest RAM image.
    pub fn guest_ram_contains(&self, needle: &[u8]) -> bool {
        if needle.is_empty() {
            return false;
        }
        self.last_guest_ram
            .windows(needle.len())
            .any(|w| w == needle)
    }

    /// Evaluate and perform host-side `SpiralCore` egress secret-placeholder substitution (`R4-37`).
    pub fn proxy_vsock_egress(
        &self,
        target_host: &str,
        request_template: &str,
    ) -> EgressProxyOutcome {
        let mut secret_in_guest = false;
        for rule in self.host_secrets.values() {
            if !rule.raw_secret.is_empty() && self.guest_ram_contains(rule.raw_secret.as_bytes()) {
                secret_in_guest = true;
                break;
            }
        }

        let mut matched_rule: Option<(&String, &HostSecretRule)> = None;
        for (name, rule) in &self.host_secrets {
            let marker = format!("{{{{MVM_SECRET:{name}}}}}");
            if request_template.contains(&marker) {
                matched_rule = Some((name, rule));
                break;
            }
        }

        let placeholder_matched = matched_rule.is_some();
        let host_allowlisted = matched_rule
            .map(|(_, r)| r.allowlisted_hosts.contains(target_host))
            .unwrap_or(false);

        let verdict = admit_host_egress(EgressPolicyCheck {
            host_allowlisted,
            placeholder_matched,
            secret_in_guest_ram: secret_in_guest,
        });

        if verdict == EgressVerdict::ProxyAdmitted {
            if let Some((name, rule)) = matched_rule {
                let marker = format!("{{{{MVM_SECRET:{name}}}}}");
                return EgressProxyOutcome {
                    verdict,
                    outbound_payload: Some(request_template.replace(&marker, &rule.raw_secret)),
                };
            }
        }
        EgressProxyOutcome {
            verdict,
            outbound_payload: None,
        }
    }

    /// Construct and validate a wire-encoded 16-byte `SpiralCore` `virtio-vsock` packet from this MicroVM.
    pub fn build_vsock_packet(
        &self,
        dst_cid: u16,
        vsock_port: u16,
        cl06_spin: u8,
        e8_orbit: u8,
        payload: &[u8],
    ) -> Result<Vec<u8>, MvmError> {
        if payload.len() > SPIRALCORE_MAX_PAYLOAD {
            return Err(MvmError::FrameRejected(FrameVerdict::RefusedPayloadTooLarge));
        }
        let hdr = SpiralCoreHeader {
            magic: SPIRALCORE_MAGIC,
            src_cid: self.guest_cid,
            dst_cid,
            vsock_port,
            cl06_spin,
            e8_orbit,
            payload_len: payload.len() as u32,
        };
        let verdict = admit_spiralcore_header(&hdr, SPIRALCORE_MAX_PAYLOAD);
        if verdict != FrameVerdict::FrameAdmitted {
            return Err(MvmError::FrameRejected(verdict));
        }
        let mut wire = Vec::with_capacity(16 + payload.len());
        wire.extend_from_slice(&hdr.to_bytes());
        wire.extend_from_slice(payload);
        Ok(wire)
    }

    /// Deliver a wire-encoded 16-byte `SpiralCore` `virtio-vsock` packet into this MicroVM's port inbox.
    pub fn deliver_vsock_packet(&mut self, wire: &[u8]) -> Result<(), MvmError> {
        let Some(hdr) = SpiralCoreHeader::from_bytes(wire) else {
            return Err(MvmError::FrameRejected(FrameVerdict::RefusedInvalidCid));
        };
        let verdict = admit_spiralcore_header(&hdr, SPIRALCORE_MAX_PAYLOAD);
        if verdict != FrameVerdict::FrameAdmitted {
            return Err(MvmError::FrameRejected(verdict));
        }
        if hdr.dst_cid != self.guest_cid {
            return Err(MvmError::FrameRejected(FrameVerdict::RefusedInvalidCid));
        }
        let end = 16usize.saturating_add(hdr.payload_len as usize);
        if wire.len() != end {
            return Err(MvmError::FrameRejected(
                FrameVerdict::RefusedPayloadTooLarge,
            ));
        }
        let pkt = VsockPacket {
            header: hdr,
            payload: wire[16..end].to_vec(),
        };
        self.vsock_inbox
            .entry(hdr.vsock_port)
            .or_default()
            .push_back(pkt);
        Ok(())
    }

    /// Pop the next queued `SpiralCore` packet on `vsock_port`.
    pub fn pop_vsock_inbox(&mut self, vsock_port: u16) -> Option<VsockPacket> {
        self.vsock_inbox.get_mut(&vsock_port)?.pop_front()
    }

    /// Execute a command or `&&`/`||`/`;` chain inside the `MVM-Wasm` RV64IMAC Linux MicroVM.
    pub fn exec_command(&mut self, cmd_line: &str, cwd: &str) -> Result<MvmExecOutcome, MvmError> {
        let norm_cwd = Self::normalize_guest_path("/workspace", cwd);
        let segments = split_command_chain(cmd_line);
        let mut total_stdout = String::new();
        let mut total_stderr = String::new();
        let mut last_exit = 0i32;
        let mut total_insts = 0u64;
        let mut all_frames = Vec::new();
        let mut compiled_machine = None;

        for (op, seg) in segments {
            if op == ChainOp::And && last_exit != 0 {
                continue;
            }
            if op == ChainOp::Or && last_exit == 0 {
                continue;
            }
            let single = self.exec_single_segment(&seg, &norm_cwd)?;
            total_stdout.push_str(&single.stdout);
            total_stderr.push_str(&single.stderr);
            last_exit = single.exit_code;
            total_insts = total_insts.saturating_add(single.instructions_retired);
            all_frames.extend(single.vsock_frames);
            if single.compiled_elf_machine.is_some() {
                compiled_machine = single.compiled_elf_machine;
            }
        }

        Ok(MvmExecOutcome {
            exit_code: last_exit,
            stdout: total_stdout,
            stderr: total_stderr,
            instructions_retired: total_insts,
            vsock_frames: all_frames,
            compiled_elf_machine: compiled_machine,
            cow_state_sha256: self.cow_state_sha256(),
        })
    }

    fn exec_single_segment(&mut self, seg: &str, cwd: &str) -> Result<MvmExecOutcome, MvmError> {
        let trimmed = seg.trim();
        if trimmed.is_empty() {
            return Ok(MvmExecOutcome {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
                instructions_retired: 0,
                vsock_frames: Vec::new(),
                compiled_elf_machine: None,
                cow_state_sha256: self.cow_state_sha256(),
            });
        }

        // If pipeline `|`, route directly through `/bin/mvmbox sh -c "<seg>"`
        if contains_unquoted_pipe(trimmed) {
            let argv = vec!["/bin/sh".to_string(), "-c".to_string(), trimmed.to_string()];
            return self.run_elf_process("/bin/sh", &argv, cwd);
        }

        let words = split_shell_words(trimmed);
        if words.is_empty() {
            return Ok(MvmExecOutcome {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
                instructions_retired: 0,
                vsock_frames: Vec::new(),
                compiled_elf_machine: None,
                cow_state_sha256: self.cow_state_sha256(),
            });
        }

        // Built-in C-to-RV64IM ELF64 compiler (`cc` / `gcc`)
        if words[0] == "cc" || words[0] == "gcc" || words[0] == "/usr/bin/cc" {
            let mut src_arg: Option<String> = None;
            let mut out_arg = "a.out".to_string();
            let mut i = 1usize;
            while i < words.len() {
                if words[i] == "-o" && i + 1 < words.len() {
                    out_arg = words[i + 1].clone();
                    i += 2;
                    continue;
                }
                if !words[i].starts_with('-') && src_arg.is_none() {
                    src_arg = Some(words[i].clone());
                }
                i += 1;
            }
            let Some(raw_src) = src_arg else {
                return Ok(MvmExecOutcome {
                    exit_code: 1,
                    stdout: String::new(),
                    stderr: "cc: missing input C source file\n".to_string(),
                    instructions_retired: 0,
                    vsock_frames: Vec::new(),
                    compiled_elf_machine: None,
                    cow_state_sha256: self.cow_state_sha256(),
                });
            };
            let src_path = Self::normalize_guest_path(cwd, &raw_src);
            let out_path = Self::normalize_guest_path(cwd, &out_arg);
            let Some(src_bytes) = self.read_guest_file(&src_path) else {
                return Ok(MvmExecOutcome {
                    exit_code: 1,
                    stdout: String::new(),
                    stderr: format!("cc: file not found: {src_path}\n"),
                    instructions_retired: 0,
                    vsock_frames: Vec::new(),
                    compiled_elf_machine: None,
                    cow_state_sha256: self.cow_state_sha256(),
                });
            };
            let src_text = String::from_utf8_lossy(&src_bytes);
            match compile_c_to_rv64_elf(&src_text) {
                Ok(elf_bytes) => {
                    let adm = admit_elf64_riscv(&elf_bytes);
                    if adm != ElfAdmission::ElfAdmitted {
                        return Err(MvmError::ElfRejected(adm));
                    }
                    let machine = u16::from_le_bytes([elf_bytes[18], elf_bytes[19]]);
                    self.write_inode_with_mode(&out_path, &elf_bytes, 0o100755)?;
                    return Ok(MvmExecOutcome {
                        exit_code: 0,
                        stdout: String::new(),
                        stderr: String::new(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: Some(machine),
                        cow_state_sha256: self.cow_state_sha256(),
                    });
                }
                Err(diag) => {
                    return Ok(MvmExecOutcome {
                        exit_code: 1,
                        stdout: String::new(),
                        stderr: format!("{diag}\n"),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    });
                }
            }
        }

        // Resolve executable path in guest filesystem
        let prog = &words[0];
        let resolved = if prog.starts_with('/') || prog.starts_with("./") || prog.starts_with("../")
        {
            let p = Self::normalize_guest_path(cwd, prog);
            if self.inodes.contains_key(&p) {
                Some(p)
            } else {
                None
            }
        } else {
            [
                format!("/bin/{prog}"),
                format!("/sbin/{prog}"),
                format!("/workspace/{prog}"),
            ]
            .into_iter()
            .find(|cand| self.inodes.contains_key(cand))
        };

        let Some(exec_path) = resolved else {
            return Ok(MvmExecOutcome {
                exit_code: 127,
                stdout: String::new(),
                stderr: format!("mvm-sh: command not found: {prog}\n"),
                instructions_retired: 0,
                vsock_frames: Vec::new(),
                compiled_elf_machine: None,
                cow_state_sha256: self.cow_state_sha256(),
            });
        };

        self.run_elf_process(&exec_path, &words, cwd)
    }

    fn load_elf_into_process(
        &mut self,
        proc_state: &mut Rv64Process,
        exec_path: &str,
        argv: &[String],
        envp: &[String],
    ) -> Result<(), MvmError> {
        let Some(elf) = self.read_guest_file(exec_path) else {
            return Err(MvmError::NotFound(exec_path.to_string()));
        };
        let image = crate::elf::parse(&elf, proc_state.ram.len()).map_err(MvmError::ElfRejected)?;
        proc_state.ram.fill(0);
        proc_state.regs = [0u64; 32];
        proc_state.lr_reservation = None;
        let entry = image.entry;
        let mut max_end = 0x20000u64;
        for segment in image.segments {
            proc_state.write_bytes(segment.address, segment.data)?;
            max_end = max_end.max(segment.end);
        }

        proc_state.brk_addr = (max_end + 4095) & !4095;
        proc_state.mmap_cursor = GUEST_MMAP_BASE;

        // Build SysV RISC-V initial user stack: [argc, argv..., 0, envp..., 0, auxv_null(0,0)]
        let mut sp = GUEST_STACK_TOP;
        let mut env_ptrs = Vec::with_capacity(envp.len());
        for e in envp {
            let bytes = e.as_bytes();
            sp -= (bytes.len() as u64) + 1;
            proc_state.write_bytes(sp, bytes)?;
            proc_state.write_bytes(sp + (bytes.len() as u64), &[0])?;
            env_ptrs.push(sp);
        }
        let mut arg_ptrs = Vec::with_capacity(argv.len());
        for a in argv {
            let bytes = a.as_bytes();
            sp -= (bytes.len() as u64) + 1;
            proc_state.write_bytes(sp, bytes)?;
            proc_state.write_bytes(sp + (bytes.len() as u64), &[0])?;
            arg_ptrs.push(sp);
        }
        sp &= !15;
        let table_words = 1 + arg_ptrs.len() + 1 + env_ptrs.len() + 1 + 2;
        if !table_words.is_multiple_of(2) {
            sp -= 8;
        }
        // AT_NULL (0, 0)
        sp -= 16;
        proc_state.write_bytes(sp, &0u64.to_le_bytes())?;
        proc_state.write_bytes(sp + 8, &0u64.to_le_bytes())?;
        // envp NULL terminator + pointers
        sp -= 8;
        proc_state.write_bytes(sp, &0u64.to_le_bytes())?;
        for &ep in env_ptrs.iter().rev() {
            sp -= 8;
            proc_state.write_bytes(sp, &ep.to_le_bytes())?;
        }
        // argv NULL terminator + pointers
        sp -= 8;
        proc_state.write_bytes(sp, &0u64.to_le_bytes())?;
        for &ap in arg_ptrs.iter().rev() {
            sp -= 8;
            proc_state.write_bytes(sp, &ap.to_le_bytes())?;
        }
        // argc
        sp -= 8;
        proc_state.write_bytes(sp, &(argv.len() as u64).to_le_bytes())?;

        proc_state.regs[2] = sp; // sp
        proc_state.pc = entry;
        Ok(())
    }

    fn run_elf_process(
        &mut self,
        exec_path: &str,
        argv: &[String],
        cwd: &str,
    ) -> Result<MvmExecOutcome, MvmError> {
        let stdout_buf = Rc::new(RefCell::new(Vec::new()));
        let stderr_buf = Rc::new(RefCell::new(Vec::new()));
        let stdin_buf = Rc::new(RefCell::new(VecDeque::new()));

        let pid = self.next_pid;
        self.next_pid += 1;

        let mut fds = BTreeMap::new();
        fds.insert(0, FdKind::StdinBuffer(stdin_buf));
        fds.insert(1, FdKind::StdoutCapture(stdout_buf.clone()));
        fds.insert(2, FdKind::StderrCapture(stderr_buf.clone()));

        let mut root_proc = Rv64Process {
            pid,
            regs: [0u64; 32],
            pc: 0,
            ram: vec![0u8; GUEST_RAM_BYTES],
            brk_addr: 0x20000,
            mmap_cursor: GUEST_MMAP_BASE,
            lr_reservation: None,
            fds,
            next_fd: 3,
            cwd: cwd.to_string(),
            exited: None,
        };

        let envp = vec![
            "PATH=/bin:/sbin:/usr/bin:/workspace".to_string(),
            "USER=lexlean".to_string(),
            "HOME=/workspace".to_string(),
        ];
        self.load_elf_into_process(&mut root_proc, exec_path, argv, &envp)?;

        let mut procs: Vec<Rv64Process> = vec![root_proc];
        let mut active_idx = 0usize;
        let mut fuel_used = 0u64;
        let mut vsock_frames = Vec::new();

        while fuel_used < MAX_INSTRUCTION_FUEL {
            if procs[active_idx].exited.is_some() {
                if let Some(next_runnable) = procs.iter().position(|p| p.exited.is_none()) {
                    active_idx = next_runnable;
                } else {
                    break;
                }
            }

            let trap_or_sys = self.step_process(&mut procs[active_idx], &mut fuel_used)?;
            if let Some(sys_event) = trap_or_sys {
                match sys_event {
                    StepEvent::CloneChild => {
                        let mut child = procs[active_idx].clone();
                        let child_pid = self.next_pid;
                        self.next_pid += 1;
                        child.pid = child_pid;
                        child.regs[10] = 0; // child sees 0
                        procs[active_idx].regs[10] = child_pid as u64;
                        let child_idx = procs.len();
                        procs.push(child);
                        // Run child first so pipeline writers/execve populate pipes before parent wait4
                        active_idx = child_idx;
                    }
                    StepEvent::Wait4 {
                        target_pid,
                        status_ptr,
                    } => {
                        let found_idx = procs.iter().position(|p| {
                            p.pid != procs[active_idx].pid
                                && (target_pid <= 0 || p.pid == target_pid)
                        });
                        if let Some(cidx) = found_idx {
                            if let Some(code) = procs[cidx].exited {
                                let cpid = procs[cidx].pid;
                                if status_ptr != 0 {
                                    let encoded = ((code & 0xff) << 8) as u32;
                                    procs[active_idx]
                                        .write_bytes(status_ptr, &encoded.to_le_bytes())?;
                                }
                                procs.remove(cidx);
                                if active_idx > cidx {
                                    active_idx -= 1;
                                }
                                procs[active_idx].regs[10] = cpid as u64;
                            } else {
                                // Suspend parent at wait4 ecall and switch to runnable child
                                procs[active_idx].pc -= 4;
                                active_idx = cidx;
                            }
                        } else {
                            procs[active_idx].regs[10] = (-10i64) as u64; // -ECHILD
                        }
                    }
                    StepEvent::VsockPacketEmitted(pkt) => {
                        vsock_frames.push(pkt);
                    }
                }
            }
        }

        if fuel_used >= MAX_INSTRUCTION_FUEL {
            return Err(MvmError::CpuTrap(
                "instruction fuel budget exceeded".to_string(),
            ));
        }

        self.last_guest_ram = procs[0].ram.clone();
        let exit_code = procs[0].exited.unwrap_or(0);
        let stdout = String::from_utf8_lossy(&stdout_buf.borrow()).into_owned();
        let stderr = String::from_utf8_lossy(&stderr_buf.borrow()).into_owned();

        Ok(MvmExecOutcome {
            exit_code,
            stdout,
            stderr,
            instructions_retired: fuel_used,
            vsock_frames,
            compiled_elf_machine: None,
            cow_state_sha256: self.cow_state_sha256(),
        })
    }

    fn step_process(
        &mut self,
        p: &mut Rv64Process,
        fuel_used: &mut u64,
    ) -> Result<Option<StepEvent>, MvmError> {
        *fuel_used += 1;
        p.regs[0] = 0;

        let half = p.read_u16(p.pc)?;
        let (inst, step_bytes) = if (half & 0b11) != 0b11 {
            (expand_rv64c(half)?, 2u64)
        } else {
            (p.read_u32(p.pc)?, 4u64)
        };

        let pc = p.pc;
        let mut next_pc = pc.wrapping_add(step_bytes);

        let opcode = inst & 0x7f;
        let rd = ((inst >> 7) & 0x1f) as usize;
        let funct3 = (inst >> 12) & 0x7;
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let rs2 = ((inst >> 20) & 0x1f) as usize;
        let funct7 = (inst >> 25) & 0x7f;

        match opcode {
            0x37 => {
                // LUI
                let imm = ((inst & 0xffff_f000) as i32) as i64 as u64;
                if rd != 0 {
                    p.regs[rd] = imm;
                }
            }
            0x17 => {
                // AUIPC
                let imm = ((inst & 0xffff_f000) as i32) as i64 as u64;
                if rd != 0 {
                    p.regs[rd] = pc.wrapping_add(imm);
                }
            }
            0x6f => {
                // JAL
                let b20 = ((inst >> 31) & 1) << 20;
                let b10_1 = ((inst >> 21) & 0x3ff) << 1;
                let b11 = ((inst >> 20) & 1) << 11;
                let b19_12 = ((inst >> 12) & 0xff) << 12;
                let raw = (b20 | b19_12 | b11 | b10_1) as i32;
                let off = ((raw << 11) >> 11) as i64 as u64;
                if rd != 0 {
                    p.regs[rd] = next_pc;
                }
                next_pc = pc.wrapping_add(off);
            }
            0x67 => {
                // JALR
                let imm = ((inst as i32) >> 20) as i64 as u64;
                let target = p.regs[rs1].wrapping_add(imm) & !1;
                if rd != 0 {
                    p.regs[rd] = next_pc;
                }
                next_pc = target;
            }
            0x63 => {
                // BRANCH
                let b12 = ((inst >> 31) & 1) << 12;
                let b11 = ((inst >> 7) & 1) << 11;
                let b10_5 = ((inst >> 25) & 0x3f) << 5;
                let b4_1 = ((inst >> 8) & 0xf) << 1;
                let raw = (b12 | b11 | b10_5 | b4_1) as i32;
                let off = ((raw << 19) >> 19) as i64 as u64;
                let a = p.regs[rs1];
                let b = p.regs[rs2];
                let take = match funct3 {
                    0 => a == b,
                    1 => a != b,
                    4 => (a as i64) < (b as i64),
                    5 => (a as i64) >= (b as i64),
                    6 => a < b,
                    7 => a >= b,
                    _ => return Err(MvmError::CpuTrap(format!("bad branch funct3={funct3}"))),
                };
                if take {
                    next_pc = pc.wrapping_add(off);
                }
            }
            0x03 => {
                // LOAD
                let imm = ((inst as i32) >> 20) as i64 as u64;
                let addr = p.regs[rs1].wrapping_add(imm);
                let val = match funct3 {
                    0 => (p.read_u8(addr)? as i8) as i64 as u64,
                    1 => (p.read_u16(addr)? as i16) as i64 as u64,
                    2 => (p.read_u32(addr)? as i32) as i64 as u64,
                    3 => p.read_u64(addr)?,
                    4 => p.read_u8(addr)? as u64,
                    5 => p.read_u16(addr)? as u64,
                    6 => p.read_u32(addr)? as u64,
                    _ => return Err(MvmError::CpuTrap(format!("bad load funct3={funct3}"))),
                };
                if rd != 0 {
                    p.regs[rd] = val;
                }
            }
            0x23 => {
                // STORE
                let imm_lo = (inst >> 7) & 0x1f;
                let imm_hi = (inst >> 25) & 0x7f;
                let raw = ((imm_hi << 5) | imm_lo) as i32;
                let imm = ((raw << 20) >> 20) as i64 as u64;
                let addr = p.regs[rs1].wrapping_add(imm);
                let val = p.regs[rs2];
                match funct3 {
                    0 => p.write_bytes(addr, &[val as u8])?,
                    1 => p.write_bytes(addr, &(val as u16).to_le_bytes())?,
                    2 => p.write_bytes(addr, &(val as u32).to_le_bytes())?,
                    3 => p.write_bytes(addr, &val.to_le_bytes())?,
                    _ => return Err(MvmError::CpuTrap(format!("bad store funct3={funct3}"))),
                }
            }
            0x13 => {
                // OP-IMM (64-bit)
                let imm = ((inst as i32) >> 20) as i64;
                let src = p.regs[rs1];
                let shamt = (inst >> 20) & 0x3f;
                let res = match funct3 {
                    0 => src.wrapping_add(imm as u64),
                    2 => u64::from((src as i64) < imm),
                    3 => u64::from(src < (imm as u64)),
                    4 => src ^ (imm as u64),
                    6 => src | (imm as u64),
                    7 => src & (imm as u64),
                    1 => src << shamt,
                    5 => {
                        if ((inst >> 30) & 1) == 1 {
                            ((src as i64) >> shamt) as u64
                        } else {
                            src >> shamt
                        }
                    }
                    _ => return Err(MvmError::CpuTrap(format!("bad op-imm funct3={funct3}"))),
                };
                if rd != 0 {
                    p.regs[rd] = res;
                }
            }
            0x1b => {
                // OP-IMM-32
                let imm = (inst as i32) >> 20;
                let src32 = p.regs[rs1] as u32;
                let shamt = (inst >> 20) & 0x1f;
                let res32: i32 = match funct3 {
                    0 => (src32 as i32).wrapping_add(imm),
                    1 => (src32 << shamt) as i32,
                    5 => {
                        if ((inst >> 30) & 1) == 1 {
                            (src32 as i32) >> shamt
                        } else {
                            (src32 >> shamt) as i32
                        }
                    }
                    _ => return Err(MvmError::CpuTrap(format!("bad op-imm-32 funct3={funct3}"))),
                };
                if rd != 0 {
                    p.regs[rd] = (res32 as i64) as u64;
                }
            }
            0x33 => {
                // OP (64-bit + RV64M)
                let a = p.regs[rs1];
                let b = p.regs[rs2];
                let res = if funct7 == 0x01 {
                    match funct3 {
                        0 => a.wrapping_mul(b),
                        1 => (((a as i64 as i128) * (b as i64 as i128)) >> 64) as u64,
                        2 => (((a as i64 as i128) * (b as u128 as i128)) >> 64) as u64,
                        3 => (((a as u128) * (b as u128)) >> 64) as u64,
                        4 => {
                            if b == 0 {
                                u64::MAX
                            } else if (a as i64) == i64::MIN && (b as i64) == -1 {
                                i64::MIN as u64
                            } else {
                                ((a as i64) / (b as i64)) as u64
                            }
                        }
                        5 => a.checked_div(b).unwrap_or(u64::MAX),
                        6 => {
                            if b == 0 {
                                a
                            } else if (a as i64) == i64::MIN && (b as i64) == -1 {
                                0
                            } else {
                                ((a as i64) % (b as i64)) as u64
                            }
                        }
                        7 => {
                            if b == 0 {
                                a
                            } else {
                                a % b
                            }
                        }
                        _ => unreachable!(),
                    }
                } else {
                    match funct3 {
                        0 => {
                            if funct7 == 0x20 {
                                a.wrapping_sub(b)
                            } else {
                                a.wrapping_add(b)
                            }
                        }
                        1 => a << (b & 0x3f),
                        2 => u64::from((a as i64) < (b as i64)),
                        3 => u64::from(a < b),
                        4 => a ^ b,
                        5 => {
                            if funct7 == 0x20 {
                                ((a as i64) >> (b & 0x3f)) as u64
                            } else {
                                a >> (b & 0x3f)
                            }
                        }
                        6 => a | b,
                        7 => a & b,
                        _ => unreachable!(),
                    }
                };
                if rd != 0 {
                    p.regs[rd] = res;
                }
            }
            0x3b => {
                // OP-32 + RV64M-32
                let a = p.regs[rs1] as u32;
                let b = p.regs[rs2] as u32;
                let res32: i32 = if funct7 == 0x01 {
                    match funct3 {
                        0 => (a as i32).wrapping_mul(b as i32),
                        4 => {
                            if b == 0 {
                                -1
                            } else if (a as i32) == i32::MIN && (b as i32) == -1 {
                                i32::MIN
                            } else {
                                (a as i32) / (b as i32)
                            }
                        }
                        5 => a.checked_div(b).map_or(-1, |q| q as i32),
                        6 => {
                            if b == 0 {
                                a as i32
                            } else if (a as i32) == i32::MIN && (b as i32) == -1 {
                                0
                            } else {
                                (a as i32) % (b as i32)
                            }
                        }
                        7 => {
                            if b == 0 {
                                a as i32
                            } else {
                                (a % b) as i32
                            }
                        }
                        _ => return Err(MvmError::CpuTrap(format!("bad op-32M funct3={funct3}"))),
                    }
                } else {
                    match funct3 {
                        0 => {
                            if funct7 == 0x20 {
                                (a as i32).wrapping_sub(b as i32)
                            } else {
                                (a as i32).wrapping_add(b as i32)
                            }
                        }
                        1 => (a << (b & 0x1f)) as i32,
                        5 => {
                            if funct7 == 0x20 {
                                (a as i32) >> (b & 0x1f)
                            } else {
                                (a >> (b & 0x1f)) as i32
                            }
                        }
                        _ => return Err(MvmError::CpuTrap(format!("bad op-32 funct3={funct3}"))),
                    }
                };
                if rd != 0 {
                    p.regs[rd] = (res32 as i64) as u64;
                }
            }
            0x2f => {
                // RV64A Atomics
                let funct5 = funct7 >> 2;
                let addr = p.regs[rs1];
                if funct3 == 2 {
                    let old = p.read_u32(addr)? as i32;
                    let src = p.regs[rs2] as i32;
                    match funct5 {
                        0x02 => {
                            p.lr_reservation = Some(addr);
                            if rd != 0 {
                                p.regs[rd] = (old as i64) as u64;
                            }
                        }
                        0x03 => {
                            if p.lr_reservation == Some(addr) {
                                p.write_bytes(addr, &(src as u32).to_le_bytes())?;
                                if rd != 0 {
                                    p.regs[rd] = 0;
                                }
                            } else if rd != 0 {
                                p.regs[rd] = 1;
                            }
                            p.lr_reservation = None;
                        }
                        _ => {
                            let new_v = match funct5 {
                                0x01 => src,
                                0x00 => old.wrapping_add(src),
                                0x04 => old ^ src,
                                0x0c => old & src,
                                0x08 => old | src,
                                0x10 => old.min(src),
                                0x14 => old.max(src),
                                0x18 => ((old as u32).min(src as u32)) as i32,
                                0x1c => ((old as u32).max(src as u32)) as i32,
                                _ => return Err(MvmError::CpuTrap("bad amo.w".to_string())),
                            };
                            p.write_bytes(addr, &(new_v as u32).to_le_bytes())?;
                            if rd != 0 {
                                p.regs[rd] = (old as i64) as u64;
                            }
                        }
                    }
                } else if funct3 == 3 {
                    let old = p.read_u64(addr)?;
                    let src = p.regs[rs2];
                    match funct5 {
                        0x02 => {
                            p.lr_reservation = Some(addr);
                            if rd != 0 {
                                p.regs[rd] = old;
                            }
                        }
                        0x03 => {
                            if p.lr_reservation == Some(addr) {
                                p.write_bytes(addr, &src.to_le_bytes())?;
                                if rd != 0 {
                                    p.regs[rd] = 0;
                                }
                            } else if rd != 0 {
                                p.regs[rd] = 1;
                            }
                            p.lr_reservation = None;
                        }
                        _ => {
                            let new_v = match funct5 {
                                0x01 => src,
                                0x00 => old.wrapping_add(src),
                                0x04 => old ^ src,
                                0x0c => old & src,
                                0x08 => old | src,
                                0x10 => ((old as i64).min(src as i64)) as u64,
                                0x14 => ((old as i64).max(src as i64)) as u64,
                                0x18 => old.min(src),
                                0x1c => old.max(src),
                                _ => return Err(MvmError::CpuTrap("bad amo.d".to_string())),
                            };
                            p.write_bytes(addr, &new_v.to_le_bytes())?;
                            if rd != 0 {
                                p.regs[rd] = old;
                            }
                        }
                    }
                }
            }
            0x0f => {
                // FENCE / FENCE.I (deterministic single-hart memory is sequentially consistent)
            }
            0x73 => {
                // SYSTEM / ECALL / EBREAK / Zicsr
                if funct3 == 0 {
                    let sys_imm = inst >> 20;
                    if sys_imm == 0 {
                        p.pc = next_pc;
                        return self.handle_syscall(p, *fuel_used);
                    } else if sys_imm == 1 {
                        p.exited = Some(0);
                    }
                } else if rd != 0 {
                    // Read-only deterministic cycle/time/instret CSRs
                    p.regs[rd] = *fuel_used;
                }
            }
            _ => {
                return Err(MvmError::CpuTrap(format!(
                    "unsupported RV64 opcode 0x{opcode:02x} at pc=0x{pc:x} (inst=0x{inst:08x})"
                )));
            }
        }

        p.regs[0] = 0;
        p.pc = next_pc;
        Ok(None)
    }

    fn handle_syscall(
        &mut self,
        p: &mut Rv64Process,
        fuel_used: u64,
    ) -> Result<Option<StepEvent>, MvmError> {
        let nr = p.regs[17];
        let a0 = p.regs[10];
        let a1 = p.regs[11];
        let a2 = p.regs[12];

        match nr {
            17 => {
                // SYS_getcwd(buf, size)
                let bytes = p.cwd.as_bytes().to_vec();
                if (a1 as usize) <= bytes.len() {
                    p.regs[10] = (-34i64) as u64; // -ERANGE
                } else {
                    p.write_bytes(a0, &bytes)?;
                    p.write_bytes(a0 + (bytes.len() as u64), &[0])?;
                    p.regs[10] = a0;
                }
            }
            24 => {
                // SYS_dup3(oldfd, newfd, flags)
                let oldfd = a0 as i32;
                let newfd = a1 as i32;
                if let Some(kind) = p.fds.get(&oldfd).cloned() {
                    p.fds.insert(newfd, kind);
                    p.regs[10] = newfd as u64;
                } else {
                    p.regs[10] = LINUX_EBADF as u64;
                }
            }
            34 => {
                // SYS_mkdirat(dirfd, path, mode)
                let raw = p.read_cstr(a1)?;
                let norm = Self::normalize_guest_path(&p.cwd, &raw);
                self.ensure_parent_dirs(&norm);
                self.dirs.insert(norm);
                p.regs[10] = 0;
            }
            35 => {
                // SYS_unlinkat(dirfd, path, flags)
                let raw = p.read_cstr(a1)?;
                let norm = Self::normalize_guest_path(&p.cwd, &raw);
                if self.inodes.remove(&norm).is_some() || self.dirs.remove(&norm) {
                    p.regs[10] = 0;
                } else {
                    p.regs[10] = LINUX_ENOENT as u64;
                }
            }
            49 => {
                // SYS_chdir(path)
                let raw = p.read_cstr(a0)?;
                let norm = Self::normalize_guest_path(&p.cwd, &raw);
                if self.dirs.contains(&norm) {
                    p.cwd = norm;
                    p.regs[10] = 0;
                } else {
                    p.regs[10] = LINUX_ENOENT as u64;
                }
            }
            56 => {
                // SYS_openat(dirfd, pathname, flags, mode)
                let raw = p.read_cstr(a1)?;
                let norm = Self::normalize_guest_path(&p.cwd, &raw);
                let flags = a2 as u32;
                let is_dir_flag = (flags & 0o200000) != 0;
                let o_creat = (flags & 0o100) != 0;
                let o_trunc = (flags & 0o1000) != 0;
                let o_append = (flags & 0o2000) != 0;
                let writable = (flags & 0b11) != 0;

                if is_dir_flag || self.dirs.contains(&norm) {
                    if !self.dirs.contains(&norm) {
                        p.regs[10] = LINUX_ENOENT as u64;
                        return Ok(None);
                    }
                    let entries = self.list_dir_entries(&norm);
                    let fd = p.next_fd;
                    p.next_fd += 1;
                    p.fds.insert(fd, FdKind::Directory { entries, index: 0 });
                    p.regs[10] = fd as u64;
                } else {
                    if !self.inodes.contains_key(&norm) {
                        if o_creat {
                            self.write_guest_file(&norm, &[])?;
                        } else {
                            p.regs[10] = LINUX_ENOENT as u64;
                            return Ok(None);
                        }
                    } else if o_trunc && writable {
                        self.write_guest_file(&norm, &[])?;
                    }
                    let fd = p.next_fd;
                    p.next_fd += 1;
                    p.fds.insert(
                        fd,
                        FdKind::RegularFile {
                            path: norm,
                            offset: 0,
                            writable,
                            append: o_append,
                        },
                    );
                    p.regs[10] = fd as u64;
                }
            }
            57 => {
                // SYS_close(fd)
                let fd = a0 as i32;
                if p.fds.remove(&fd).is_some() {
                    p.regs[10] = 0;
                } else {
                    p.regs[10] = LINUX_EBADF as u64;
                }
            }
            59 => {
                // SYS_pipe2(pipefd[2], flags)
                let shared = Rc::new(RefCell::new(VecDeque::new()));
                let rfd = p.next_fd;
                let wfd = p.next_fd + 1;
                p.next_fd += 2;
                p.fds.insert(rfd, FdKind::PipeRead(shared.clone()));
                p.fds.insert(wfd, FdKind::PipeWrite(shared));
                p.write_bytes(a0, &rfd.to_le_bytes())?;
                p.write_bytes(a0 + 4, &wfd.to_le_bytes())?;
                p.regs[10] = 0;
            }
            61 => {
                // SYS_getdents64(fd, dirp, count)
                let fd = a0 as i32;
                let max_bytes = a2 as usize;
                let mut out_buf: Vec<u8> = Vec::new();
                {
                    let Some(FdKind::Directory { entries, index }) = p.fds.get_mut(&fd) else {
                        p.regs[10] = LINUX_EBADF as u64;
                        return Ok(None);
                    };
                    while *index < entries.len() {
                        let (dtype, ref name) = entries[*index];
                        let name_bytes = name.as_bytes();
                        let reclen = (19 + name_bytes.len() + 1 + 7) & !7;
                        if out_buf.len() + reclen > max_bytes {
                            break;
                        }
                        let mut rec = vec![0u8; reclen];
                        let ino = (*index as u64) + 1000;
                        let next_off = (*index + 1) as i64;
                        rec[0..8].copy_from_slice(&ino.to_le_bytes());
                        rec[8..16].copy_from_slice(&next_off.to_le_bytes());
                        rec[16..18].copy_from_slice(&(reclen as u16).to_le_bytes());
                        rec[18] = dtype;
                        rec[19..19 + name_bytes.len()].copy_from_slice(name_bytes);
                        out_buf.extend_from_slice(&rec);
                        *index += 1;
                    }
                }
                if !out_buf.is_empty() {
                    p.write_bytes(a1, &out_buf)?;
                }
                p.regs[10] = out_buf.len() as u64;
            }
            62 => {
                // SYS_lseek(fd, offset, whence)
                let fd = a0 as i32;
                let off = a1 as i64;
                let whence = a2 as u32;
                let file_len = match p.fds.get(&fd) {
                    Some(FdKind::RegularFile { path, .. }) => {
                        self.read_guest_file(path).map(|b| b.len()).unwrap_or(0)
                    }
                    _ => {
                        p.regs[10] = LINUX_EBADF as u64;
                        return Ok(None);
                    }
                };
                if let Some(FdKind::RegularFile { offset, .. }) = p.fds.get_mut(&fd) {
                    let base = match whence {
                        0 => 0i64,
                        1 => *offset as i64,
                        2 => file_len as i64,
                        _ => {
                            p.regs[10] = LINUX_EINVAL as u64;
                            return Ok(None);
                        }
                    };
                    let next = (base + off).max(0) as usize;
                    *offset = next;
                    p.regs[10] = next as u64;
                }
            }
            63 => {
                // SYS_read(fd, buf, count)
                let fd = a0 as i32;
                let count = a2 as usize;
                let kind_clone = p.fds.get(&fd).cloned();
                match kind_clone {
                    Some(FdKind::StdinBuffer(q)) | Some(FdKind::PipeRead(q)) => {
                        let mut guard = q.borrow_mut();
                        let take = count.min(guard.len());
                        let mut tmp = Vec::with_capacity(take);
                        for _ in 0..take {
                            if let Some(b) = guard.pop_front() {
                                tmp.push(b);
                            }
                        }
                        drop(guard);
                        p.write_bytes(a1, &tmp)?;
                        p.regs[10] = tmp.len() as u64;
                    }
                    Some(FdKind::RegularFile { path, offset, .. }) => {
                        let Some(data) = self.read_guest_file(&path) else {
                            p.regs[10] = LINUX_ENOENT as u64;
                            return Ok(None);
                        };
                        let avail = data.len().saturating_sub(offset);
                        let take = count.min(avail);
                        if take > 0 {
                            p.write_bytes(a1, &data[offset..offset + take])?;
                            if let Some(FdKind::RegularFile {
                                offset: off_mut, ..
                            }) = p.fds.get_mut(&fd)
                            {
                                *off_mut += take;
                            }
                        }
                        p.regs[10] = take as u64;
                    }
                    _ => p.regs[10] = LINUX_EBADF as u64,
                }
            }
            64 => {
                // SYS_write(fd, buf, count)
                let fd = a0 as i32;
                let count = a2 as usize;
                let idx = a1 as usize;
                if idx.saturating_add(count) > p.ram.len() {
                    return Err(MvmError::CpuTrap("SYS_write buffer fault".to_string()));
                }
                let data = p.ram[idx..idx + count].to_vec();
                let kind_clone = p.fds.get(&fd).cloned();
                match kind_clone {
                    Some(FdKind::StdoutCapture(out)) => {
                        out.borrow_mut().extend_from_slice(&data);
                        p.regs[10] = count as u64;
                    }
                    Some(FdKind::StderrCapture(err)) => {
                        err.borrow_mut().extend_from_slice(&data);
                        p.regs[10] = count as u64;
                    }
                    Some(FdKind::PipeWrite(q)) => {
                        q.borrow_mut().extend(data.iter().copied());
                        p.regs[10] = count as u64;
                    }
                    Some(FdKind::RegularFile {
                        path,
                        offset,
                        writable,
                        append,
                    }) => {
                        if !writable {
                            p.regs[10] = LINUX_EBADF as u64;
                            return Ok(None);
                        }
                        let mut cur = self.read_guest_file(&path).unwrap_or_default();
                        let pos = if append { cur.len() } else { offset };
                        if pos + data.len() > cur.len() {
                            cur.resize(pos + data.len(), 0);
                        }
                        cur[pos..pos + data.len()].copy_from_slice(&data);
                        self.write_guest_file(&path, &cur)?;
                        if let Some(FdKind::RegularFile {
                            offset: off_mut, ..
                        }) = p.fds.get_mut(&fd)
                        {
                            *off_mut = pos + data.len();
                        }
                        p.regs[10] = count as u64;
                    }
                    _ => p.regs[10] = LINUX_EBADF as u64,
                }
            }
            79 | 80 => {
                // SYS_fstatat / SYS_fstat (minimal 128-byte zeroed stat struct with st_size)
                let stat_ptr = if nr == 80 { a1 } else { a2 };
                let zeros = [0u8; 128];
                p.write_bytes(stat_ptr, &zeros)?;
                p.regs[10] = 0;
            }
            93 | 94 => {
                // SYS_exit / SYS_exit_group
                p.exited = Some((a0 as i32) & 0xff);
            }
            113 => {
                // SYS_clock_gettime(clk_id, tp)
                let secs = 1_794_332_000u64 + (fuel_used / 1_000_000);
                let nsecs = (fuel_used % 1_000_000) * 1_000;
                if a1 != 0 {
                    p.write_bytes(a1, &secs.to_le_bytes())?;
                    p.write_bytes(a1 + 8, &nsecs.to_le_bytes())?;
                }
                p.regs[10] = 0;
            }
            160 => {
                // SYS_uname(buf) — 6 * 65-byte fields
                let mut u = [0u8; 390];
                let fields = [
                    "Linux",
                    "mvm-wasm",
                    "6.6.0-uor-rv64imac",
                    "#1 SMP PREEMPT_DYNAMIC",
                    "riscv64",
                    "uor.local",
                ];
                for (i, f) in fields.iter().enumerate() {
                    let b = f.as_bytes();
                    let take = b.len().min(64);
                    u[i * 65..i * 65 + take].copy_from_slice(&b[..take]);
                }
                p.write_bytes(a0, &u)?;
                p.regs[10] = 0;
            }
            172 => p.regs[10] = p.pid as u64, // SYS_getpid
            174 | 175 => p.regs[10] = 1000,   // SYS_getuid / SYS_geteuid
            198 => {
                // SYS_socket(domain, type, protocol) — Zero-NIC enforcement (`R4-37`)
                match admit_guest_socket_domain(a0) {
                    SocketAdmission::AdmittedVsock => {
                        let fd = p.next_fd;
                        p.next_fd += 1;
                        p.fds.insert(
                            fd,
                            FdKind::VsockSocket {
                                peer_cid: 0,
                                peer_port: 0,
                                rx_queue: VecDeque::new(),
                            },
                        );
                        p.regs[10] = fd as u64;
                    }
                    SocketAdmission::RefusedNicDisabled
                    | SocketAdmission::RefusedUnsupportedDomain => {
                        p.regs[10] = LINUX_EAFNOSUPPORT as u64;
                    }
                }
            }
            200 | 201 | 242 => {
                // SYS_bind / SYS_listen / SYS_accept4
                p.regs[10] = 0;
            }
            203 => {
                // SYS_connect(sockfd, sockaddr_vm*, addrlen)
                let fd = a0 as i32;
                if a2 < 16 {
                    p.regs[10] = LINUX_EINVAL as u64;
                    return Ok(None);
                }
                let family = p.read_u16(a1)?;
                let port = p.read_u32(a1 + 4)? as u16;
                let cid = p.read_u32(a1 + 8)? as u16;
                if admit_guest_socket_domain(family as u64) != SocketAdmission::AdmittedVsock {
                    p.regs[10] = LINUX_EAFNOSUPPORT as u64;
                    return Ok(None);
                }
                if let Some(FdKind::VsockSocket {
                    peer_cid,
                    peer_port,
                    ..
                }) = p.fds.get_mut(&fd)
                {
                    *peer_cid = cid;
                    *peer_port = port;
                    p.regs[10] = 0;
                } else {
                    p.regs[10] = LINUX_EBADF as u64;
                }
            }
            206 => {
                // SYS_sendto(sockfd, buf, len, flags, dest_addr, addrlen)
                let fd = a0 as i32;
                let len = a2 as usize;
                let idx = a1 as usize;
                if idx.saturating_add(len) > p.ram.len() {
                    return Err(MvmError::CpuTrap("SYS_sendto fault".to_string()));
                }
                let payload = p.ram[idx..idx + len].to_vec();
                let Some(FdKind::VsockSocket {
                    peer_cid,
                    peer_port,
                    rx_queue,
                }) = p.fds.get_mut(&fd)
                else {
                    p.regs[10] = LINUX_EBADF as u64;
                    return Ok(None);
                };
                let dst_cid = *peer_cid;
                let port = *peer_port;

                // Derive deterministic Cl(0,6) spin (0..63) and E8 orbit (0..15) from payload SHA-256
                let mut h = Sha256::new();
                h.update(&payload);
                let dig = h.finalize();
                let hdr = SpiralCoreHeader {
                    magic: SPIRALCORE_MAGIC,
                    src_cid: self.guest_cid,
                    dst_cid,
                    vsock_port: port,
                    cl06_spin: dig[0] & 63,
                    e8_orbit: dig[1] & 15,
                    payload_len: payload.len() as u32,
                };
                let verdict = admit_spiralcore_header(&hdr, SPIRALCORE_MAX_PAYLOAD);
                if verdict != FrameVerdict::FrameAdmitted {
                    p.regs[10] = LINUX_EINVAL as u64;
                    return Ok(None);
                }

                // Host CID 2 control plane immediate reply
                if dst_cid == 2 {
                    rx_queue.push_back(b"SPIRALCORE_HOST_ACK".to_vec());
                }
                p.regs[10] = len as u64;
                return Ok(Some(StepEvent::VsockPacketEmitted(VsockPacket {
                    header: hdr,
                    payload,
                })));
            }
            207 => {
                // SYS_recvfrom(sockfd, buf, len, ...)
                let fd = a0 as i32;
                let max_len = a2 as usize;
                let msg_opt = match p.fds.get_mut(&fd) {
                    Some(FdKind::VsockSocket { rx_queue, .. }) => rx_queue.pop_front(),
                    _ => {
                        p.regs[10] = LINUX_EBADF as u64;
                        return Ok(None);
                    }
                };
                if let Some(msg) = msg_opt {
                    let take = max_len.min(msg.len());
                    p.write_bytes(a1, &msg[..take])?;
                    p.regs[10] = take as u64;
                } else {
                    p.regs[10] = 0;
                }
            }
            214 => {
                // SYS_brk(addr)
                if a0 >= p.brk_addr && (a0 as usize) < GUEST_MMAP_BASE as usize {
                    p.brk_addr = a0;
                }
                p.regs[10] = p.brk_addr;
            }
            215 => p.regs[10] = 0, // SYS_munmap
            220 => {
                // SYS_clone
                return Ok(Some(StepEvent::CloneChild));
            }
            221 => {
                // SYS_execve(filename, argv, envp)
                let raw_path = p.read_cstr(a0)?;
                let norm = Self::normalize_guest_path(&p.cwd, &raw_path);
                let mut argv_vec = Vec::new();
                if a1 != 0 {
                    let mut cur = a1;
                    for _ in 0..64 {
                        let ptr = p.read_u64(cur)?;
                        if ptr == 0 {
                            break;
                        }
                        argv_vec.push(p.read_cstr(ptr)?);
                        cur += 8;
                    }
                }
                let mut envp_vec = Vec::new();
                if a2 != 0 {
                    let mut cur = a2;
                    for _ in 0..64 {
                        let ptr = p.read_u64(cur)?;
                        if ptr == 0 {
                            break;
                        }
                        envp_vec.push(p.read_cstr(ptr)?);
                        cur += 8;
                    }
                }
                if self
                    .load_elf_into_process(p, &norm, &argv_vec, &envp_vec)
                    .is_err()
                {
                    p.regs[10] = LINUX_ENOENT as u64;
                }
            }
            222 => {
                // SYS_mmap(addr, length, ...)
                let aligned = (a1.max(4096) + 4095) & !4095;
                let out = p.mmap_cursor;
                if (out.saturating_add(aligned)) >= GUEST_STACK_TOP - 65536 {
                    p.regs[10] = (-12i64) as u64; // -ENOMEM
                } else {
                    p.mmap_cursor += aligned;
                    p.regs[10] = out;
                }
            }
            260 => {
                // SYS_wait4(pid, wstatus, options, rusage)
                return Ok(Some(StepEvent::Wait4 {
                    target_pid: a0 as i64,
                    status_ptr: a1,
                }));
            }
            _ => {
                p.regs[10] = LINUX_ENOSYS as u64;
            }
        }
        Ok(None)
    }

    fn list_dir_entries(&self, norm_dir: &str) -> Vec<(u8, String)> {
        let mut seen = BTreeMap::new();
        seen.insert(".".to_string(), 4u8);
        seen.insert("..".to_string(), 4u8);
        let prefix = if norm_dir == "/" {
            "/".to_string()
        } else {
            format!("{norm_dir}/")
        };
        for d in &self.dirs {
            if let Some(rest) = d.strip_prefix(&prefix) {
                if !rest.is_empty() && !rest.contains('/') {
                    seen.insert(rest.to_string(), 4u8);
                }
            }
        }
        for (f, inode) in &self.inodes {
            if let Some(rest) = f.strip_prefix(&prefix) {
                if !rest.is_empty() && !rest.contains('/') {
                    let dtype = match inode.kind {
                        InodeKind::Symlink { .. } => 10u8,
                        InodeKind::Regular { .. } => 8u8,
                    };
                    seen.insert(rest.to_string(), dtype);
                }
            }
        }
        seen.into_iter().map(|(k, v)| (v, k)).collect()
    }
}

enum StepEvent {
    CloneChild,
    Wait4 { target_pid: i64, status_ptr: u64 },
    VsockPacketEmitted(VsockPacket),
}

#[derive(PartialEq, Eq)]
enum ChainOp {
    First,
    Semi,
    And,
    Or,
}

fn split_command_chain(line: &str) -> Vec<(ChainOp, String)> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut op = ChainOp::First;
    let bytes = line.as_bytes();
    let mut i = 0usize;
    let mut quote = 0u8;
    while i < bytes.len() {
        let c = bytes[i];
        if (c == b'\'' || c == b'"') && (i == 0 || bytes[i - 1] != b'\\') {
            if quote == c {
                quote = 0;
            } else if quote == 0 {
                quote = c;
            }
            cur.push(c as char);
            i += 1;
            continue;
        }
        if quote == 0 {
            if c == b'&' && i + 1 < bytes.len() && bytes[i + 1] == b'&' {
                out.push((op, cur.trim().to_string()));
                cur.clear();
                op = ChainOp::And;
                i += 2;
                continue;
            }
            if c == b'|' && i + 1 < bytes.len() && bytes[i + 1] == b'|' {
                out.push((op, cur.trim().to_string()));
                cur.clear();
                op = ChainOp::Or;
                i += 2;
                continue;
            }
            if c == b';' {
                out.push((op, cur.trim().to_string()));
                cur.clear();
                op = ChainOp::Semi;
                i += 1;
                continue;
            }
        }
        cur.push(c as char);
        i += 1;
    }
    if !cur.trim().is_empty() {
        out.push((op, cur.trim().to_string()));
    }
    out
}

fn contains_unquoted_pipe(seg: &str) -> bool {
    let b = seg.as_bytes();
    let mut q = 0u8;
    for i in 0..b.len() {
        if b[i] == b'\'' || b[i] == b'"' {
            if q == b[i] {
                q = 0;
            } else if q == 0 {
                q = b[i];
            }
        } else if q == 0
            && b[i] == b'|'
            && (i == 0 || b[i - 1] != b'|')
            && (i + 1 >= b.len() || b[i + 1] != b'|')
        {
            return true;
        }
    }
    false
}

fn split_shell_words(seg: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut cur = String::new();
    let mut q = 0u8;
    for &b in seg.as_bytes() {
        if q != 0 {
            if b == q {
                q = 0;
            } else {
                cur.push(b as char);
            }
        } else if b == b'\'' || b == b'"' {
            q = b;
        } else if b.is_ascii_whitespace() {
            if !cur.is_empty() {
                words.push(std::mem::take(&mut cur));
            }
        } else {
            cur.push(b as char);
        }
    }
    if !cur.is_empty() {
        words.push(cur);
    }
    words
}

/// Expand a 16-bit RV64C compressed instruction into its exact 32-bit RV64I equivalent.
fn expand_rv64c(c: u16) -> Result<u32, MvmError> {
    let c32 = c as u32;
    let op = c32 & 0b11;
    let funct3 = (c32 >> 13) & 0x7;
    let r_prime = |bits: u32| 8 + (bits & 0x7);

    let sign_ext = |val: u32, bits: u32| -> i32 {
        let shift = 32 - bits;
        ((val << shift) as i32) >> shift
    };
    let enc_i = |imm: i32, rs1: u32, f3: u32, rd: u32, opc: u32| -> u32 {
        (((imm as u32) & 0xfff) << 20) | (rs1 << 15) | (f3 << 12) | (rd << 7) | opc
    };
    let enc_s = |imm: i32, rs2: u32, rs1: u32, f3: u32, opc: u32| -> u32 {
        let u = (imm as u32) & 0xfff;
        ((u >> 5) << 25) | (rs2 << 20) | (rs1 << 15) | (f3 << 12) | ((u & 0x1f) << 7) | opc
    };
    let enc_r = |f7: u32, rs2: u32, rs1: u32, f3: u32, rd: u32, opc: u32| -> u32 {
        (f7 << 25) | (rs2 << 20) | (rs1 << 15) | (f3 << 12) | (rd << 7) | opc
    };
    let enc_b = |off: i32, rs2: u32, rs1: u32, f3: u32| -> u32 {
        let u = off as u32;
        (((u >> 12) & 1) << 31)
            | (((u >> 5) & 0x3f) << 25)
            | (rs2 << 20)
            | (rs1 << 15)
            | (f3 << 12)
            | (((u >> 1) & 0xf) << 8)
            | (((u >> 11) & 1) << 7)
            | 0x63
    };
    let enc_j = |off: i32, rd: u32| -> u32 {
        let u = off as u32;
        (((u >> 20) & 1) << 31)
            | (((u >> 1) & 0x3ff) << 21)
            | (((u >> 11) & 1) << 20)
            | (((u >> 12) & 0xff) << 12)
            | (rd << 7)
            | 0x6f
    };

    match (op, funct3) {
        (0b00, 0b000) => {
            // C.ADDI4SPN
            let rd = r_prime(c32 >> 2);
            let imm = (((c32 >> 5) & 1) << 3)
                | (((c32 >> 6) & 1) << 2)
                | (((c32 >> 7) & 0xf) << 6)
                | (((c32 >> 11) & 0x3) << 4);
            if imm == 0 {
                return Err(MvmError::CpuTrap(
                    "illegal C instruction 0x0000".to_string(),
                ));
            }
            Ok(enc_i(imm as i32, 2, 0, rd, 0x13))
        }
        (0b00, 0b010) => {
            // C.LW
            let rd = r_prime(c32 >> 2);
            let rs1 = r_prime(c32 >> 7);
            let imm =
                (((c32 >> 5) & 1) << 6) | (((c32 >> 6) & 1) << 2) | (((c32 >> 10) & 0x7) << 3);
            Ok(enc_i(imm as i32, rs1, 2, rd, 0x03))
        }
        (0b00, 0b011) => {
            // C.LD
            let rd = r_prime(c32 >> 2);
            let rs1 = r_prime(c32 >> 7);
            let imm = (((c32 >> 5) & 0x3) << 6) | (((c32 >> 10) & 0x7) << 3);
            Ok(enc_i(imm as i32, rs1, 3, rd, 0x03))
        }
        (0b00, 0b110) => {
            // C.SW
            let rs2 = r_prime(c32 >> 2);
            let rs1 = r_prime(c32 >> 7);
            let imm =
                (((c32 >> 5) & 1) << 6) | (((c32 >> 6) & 1) << 2) | (((c32 >> 10) & 0x7) << 3);
            Ok(enc_s(imm as i32, rs2, rs1, 2, 0x23))
        }
        (0b00, 0b111) => {
            // C.SD
            let rs2 = r_prime(c32 >> 2);
            let rs1 = r_prime(c32 >> 7);
            let imm = (((c32 >> 5) & 0x3) << 6) | (((c32 >> 10) & 0x7) << 3);
            Ok(enc_s(imm as i32, rs2, rs1, 3, 0x23))
        }
        (0b01, 0b000) => {
            // C.NOP / C.ADDI
            let rd = (c32 >> 7) & 0x1f;
            let raw = (((c32 >> 12) & 1) << 5) | ((c32 >> 2) & 0x1f);
            let imm = sign_ext(raw, 6);
            Ok(enc_i(imm, rd, 0, rd, 0x13))
        }
        (0b01, 0b001) => {
            // C.ADDIW
            let rd = (c32 >> 7) & 0x1f;
            let raw = (((c32 >> 12) & 1) << 5) | ((c32 >> 2) & 0x1f);
            let imm = sign_ext(raw, 6);
            Ok(enc_i(imm, rd, 0, rd, 0x1b))
        }
        (0b01, 0b010) => {
            // C.LI
            let rd = (c32 >> 7) & 0x1f;
            let raw = (((c32 >> 12) & 1) << 5) | ((c32 >> 2) & 0x1f);
            let imm = sign_ext(raw, 6);
            Ok(enc_i(imm, 0, 0, rd, 0x13))
        }
        (0b01, 0b011) => {
            // C.ADDI16SP / C.LUI
            let rd = (c32 >> 7) & 0x1f;
            if rd == 2 {
                let raw = (((c32 >> 12) & 1) << 9)
                    | (((c32 >> 3) & 0x3) << 7)
                    | (((c32 >> 5) & 1) << 6)
                    | (((c32 >> 2) & 1) << 5)
                    | (((c32 >> 6) & 1) << 4);
                let imm = sign_ext(raw, 10);
                Ok(enc_i(imm, 2, 0, 2, 0x13))
            } else {
                let raw = (((c32 >> 12) & 1) << 5) | ((c32 >> 2) & 0x1f);
                let imm = sign_ext(raw, 6);
                Ok((((imm as u32) & 0xfffff) << 12) | (rd << 7) | 0x37)
            }
        }
        (0b01, 0b100) => {
            let rd = r_prime(c32 >> 7);
            let sub_op = (c32 >> 10) & 0x3;
            let shamt = (((c32 >> 12) & 1) << 5) | ((c32 >> 2) & 0x1f);
            match sub_op {
                0 => Ok(enc_i(shamt as i32, rd, 5, rd, 0x13)), // C.SRLI
                1 => Ok(enc_i((shamt | (1 << 10)) as i32, rd, 5, rd, 0x13)), // C.SRAI
                2 => {
                    let imm = sign_ext(shamt, 6);
                    Ok(enc_i(imm, rd, 7, rd, 0x13)) // C.ANDI
                }
                3 => {
                    let rs2 = r_prime(c32 >> 2);
                    let f = (((c32 >> 12) & 1) << 2) | ((c32 >> 5) & 0x3);
                    match f {
                        0b000 => Ok(enc_r(0x20, rs2, rd, 0, rd, 0x33)), // C.SUB
                        0b001 => Ok(enc_r(0x00, rs2, rd, 4, rd, 0x33)), // C.XOR
                        0b010 => Ok(enc_r(0x00, rs2, rd, 6, rd, 0x33)), // C.OR
                        0b011 => Ok(enc_r(0x00, rs2, rd, 7, rd, 0x33)), // C.AND
                        0b100 => Ok(enc_r(0x20, rs2, rd, 0, rd, 0x3b)), // C.SUBW
                        0b101 => Ok(enc_r(0x00, rs2, rd, 0, rd, 0x3b)), // C.ADDW
                        _ => Err(MvmError::CpuTrap("bad C.ALU".to_string())),
                    }
                }
                _ => unreachable!(),
            }
        }
        (0b01, 0b101) => {
            // C.J
            let raw = (((c32 >> 12) & 1) << 11)
                | (((c32 >> 8) & 1) << 10)
                | (((c32 >> 9) & 0x3) << 8)
                | (((c32 >> 6) & 1) << 7)
                | (((c32 >> 7) & 1) << 6)
                | (((c32 >> 2) & 1) << 5)
                | (((c32 >> 11) & 1) << 4)
                | (((c32 >> 3) & 0x7) << 1);
            Ok(enc_j(sign_ext(raw, 12), 0))
        }
        (0b01, 0b110) | (0b01, 0b111) => {
            // C.BEQZ / C.BNEZ
            let rs1 = r_prime(c32 >> 7);
            let raw = (((c32 >> 12) & 1) << 8)
                | (((c32 >> 5) & 0x3) << 6)
                | (((c32 >> 2) & 1) << 5)
                | (((c32 >> 10) & 0x3) << 3)
                | (((c32 >> 3) & 0x3) << 1);
            let f3 = if funct3 == 0b110 { 0 } else { 1 };
            Ok(enc_b(sign_ext(raw, 9), 0, rs1, f3))
        }
        (0b10, 0b000) => {
            // C.SLLI
            let rd = (c32 >> 7) & 0x1f;
            let shamt = (((c32 >> 12) & 1) << 5) | ((c32 >> 2) & 0x1f);
            Ok(enc_i(shamt as i32, rd, 1, rd, 0x13))
        }
        (0b10, 0b010) => {
            // C.LWSP
            let rd = (c32 >> 7) & 0x1f;
            let imm =
                (((c32 >> 2) & 0x3) << 6) | (((c32 >> 12) & 1) << 5) | (((c32 >> 4) & 0x7) << 2);
            Ok(enc_i(imm as i32, 2, 2, rd, 0x03))
        }
        (0b10, 0b011) => {
            // C.LDSP
            let rd = (c32 >> 7) & 0x1f;
            let imm =
                (((c32 >> 2) & 0x7) << 6) | (((c32 >> 12) & 1) << 5) | (((c32 >> 5) & 0x3) << 3);
            Ok(enc_i(imm as i32, 2, 3, rd, 0x03))
        }
        (0b10, 0b100) => {
            let bit12 = (c32 >> 12) & 1;
            let rd = (c32 >> 7) & 0x1f;
            let rs2 = (c32 >> 2) & 0x1f;
            match (bit12, rs2) {
                (0, 0) => Ok(enc_i(0, rd, 0, 0, 0x67)),      // C.JR
                (0, _) => Ok(enc_r(0, rs2, 0, 0, rd, 0x33)), // C.MV
                (1, 0) => {
                    if rd == 0 {
                        Ok(0x0010_0073) // C.EBREAK
                    } else {
                        Ok(enc_i(0, rd, 0, 1, 0x67)) // C.JALR
                    }
                }
                (1, _) => Ok(enc_r(0, rs2, rd, 0, rd, 0x33)), // C.ADD
                _ => unreachable!(),
            }
        }
        (0b10, 0b110) => {
            // C.SWSP
            let rs2 = (c32 >> 2) & 0x1f;
            let imm = (((c32 >> 7) & 0x3) << 6) | (((c32 >> 9) & 0xf) << 2);
            Ok(enc_s(imm as i32, rs2, 2, 2, 0x23))
        }
        (0b10, 0b111) => {
            // C.SDSP
            let rs2 = (c32 >> 2) & 0x1f;
            let imm = (((c32 >> 7) & 0x7) << 6) | (((c32 >> 10) & 0x7) << 3);
            Ok(enc_s(imm as i32, rs2, 2, 3, 0x23))
        }
        _ => Err(MvmError::CpuTrap(format!(
            "unsupported RV64C instruction 0x{c:04x}"
        ))),
    }
}
