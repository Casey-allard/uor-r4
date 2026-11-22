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

use crate::lexlean_admission::r4_connected_studio::{
    admit_bridge_transport, admit_git_secret_substitution, admit_package_install,
    BridgeTransportVerdict, GitSecretVerdict, McpPolicyMode, PackageInstallVerdict, TransportKind,
};
use crate::lexlean_admission::r4_secops_audit::{admit_sandbox_write, SandboxAdmissionOutcome};
use crate::lexlean_admission::r4_spiralcore_vsock::{
    admit_guest_socket, admit_host_egress, admit_spiralcore_frame, EgressPolicyCheck,
    EgressVerdict, ElfAdmission, FrameVerdict, SocketAdmission, SpiralCoreFrame,
};
use crate::mvm_cc::{compile_c_units_to_rv64_elf, CCompileError};

/// Four-byte magic prefix for 16-byte `SpiralCore` `virtio-vsock` frames.
pub const SPIRALCORE_MAGIC: [u8; 4] = *b"SPVS";
/// Canonical `SpiralCore` `virtio-vsock` control and bridge port.
pub const SPIRALCORE_VSOCK_PORT: u16 = 5257;
/// Reserved Host bridge Context ID (`CID 2`).
pub const SPIRALCORE_HOST_CID: u16 = 2;
/// Default guest MicroVM Context ID (`CID 3`).
pub const SPIRALCORE_GUEST_CID: u16 = 3;
/// Reserved filesystem superblock sector ID for instant session resumption.
pub const MVM_SUPERBLOCK_SECTOR_ID: u32 = 0xFFFF_F000;
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
/// Linux `EACCES` (`13`) negated.
const LINUX_EACCES: i64 = -13;
/// Linux `EINVAL` (`22`) negated.
const LINUX_EINVAL: i64 = -22;
/// Linux `EFBIG` (`27`) negated.
const LINUX_EFBIG: i64 = -27;
/// Linux `ENOSYS` (`38`) negated.
const LINUX_ENOSYS: i64 = -38;
/// Maximum size of a single virtual file written through Linux syscalls (8 MiB).
const MAX_GUEST_FILE_BYTES: usize = 8 * 1024 * 1024;

/// Typed `SpiralCore` bridge and package-admission errors (`R4-39`, `R5`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeError {
    /// Outbound bridge blocked because MCP policy mode is `Deny`.
    PolicyDenied,
    /// Non-TLS `http://` or `ws://` remote transport rejected.
    InsecureTransportRejected,
    /// Source or destination `virtio-vsock` CID violated `CID >= 3 -> CID 2`.
    InvalidVsockCid,
    /// Raw secret bytes were detected inside `MvmGuest` RAM.
    GuestRamSecretContamination,
    /// Secret placeholder targeted a host outside `["github.com", "api.github.com"]`.
    HostNotAllowlistedForSecret,
    /// Secret substitution invoked without a `{{MVM_SECRET:...}}` placeholder.
    MissingSecretPlaceholder,
    /// Downloaded package bytes failed SHA-256 verification.
    PackageDigestMismatch,
    /// Downloaded package failed `ELF64` little-endian `EM_RISCV = 243` verification.
    PackageInvalidElf64,
    /// Downloaded package exceeded the 4 MB binary quota.
    PackageSizeExceedsQuota,
    /// Bridge resume or response JSON was malformed.
    InvalidBridgeJson(String),
    /// Subresource Integrity (`SHA-512` npm or `SHA-384` Pyodide) verification failed (`R4-58`).
    IntegrityMismatch(String),
    /// Host `esbuildModuleUrl` / `esbuildWasmUrl` adapter is unconfigured or unavailable (`R4-58`).
    EsbuildAdapterUnavailable(String),
    /// WASI Preview 1 module instantiation or execution failed (`R4-58`).
    WasiRuntimeError(String),
    /// In-Worker SQLite execution or binary DB validation failed (`R4-58`).
    SqliteRuntimeError(String),
    /// Host is outside the Prism `capabilities/1` `Sandboxed` allowlist (`R4-59`).
    CapabilityDenied(String),
    /// Escalation to `Full SpiralCore Internet` requires explicit user consent (`R4-59`).
    RelayConsentRequired,
    /// `AF_VSOCK` relay failback is strictly forbidden for credentialed hosts/requests (`R4-59`).
    RelayForbiddenForCredentialedHost,
    /// Private, loopback, or link-local SSRF address was blocked (`R4-59`).
    PrivateAddressBlocked(String),
    /// RFC 8484 DNS-over-HTTPS resolution failed (`R4-59`).
    DoHResolutionFailed(String),
    /// SpiralCore `AF_VSOCK` relay endpoint is unreachable or timed out (`R4-59`).
    RelayUnavailable(String),
}

impl BridgeError {
    /// Canonical machine-readable error code matching `R4-39`, `R4-58`, and `R4-59` failure variants.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::PolicyDenied => "PolicyDenied",
            Self::InsecureTransportRejected => "InsecureTransportRejected",
            Self::InvalidVsockCid => "InvalidVsockCid",
            Self::GuestRamSecretContamination => "GuestRamSecretContamination",
            Self::HostNotAllowlistedForSecret => "HostNotAllowlistedForSecret",
            Self::MissingSecretPlaceholder => "MissingSecretPlaceholder",
            Self::PackageDigestMismatch => "PackageDigestMismatch",
            Self::PackageInvalidElf64 => "PackageInvalidElf64",
            Self::PackageSizeExceedsQuota => "PackageSizeExceedsQuota",
            Self::InvalidBridgeJson(_) => "InvalidBridgeJson",
            Self::IntegrityMismatch(_) => "IntegrityMismatch",
            Self::EsbuildAdapterUnavailable(_) => "EsbuildAdapterUnavailable",
            Self::WasiRuntimeError(_) => "WasiRuntimeError",
            Self::SqliteRuntimeError(_) => "SqliteRuntimeError",
            Self::CapabilityDenied(_) => "CapabilityDenied",
            Self::RelayConsentRequired => "RelayConsentRequired",
            Self::RelayForbiddenForCredentialedHost => "RelayForbiddenForCredentialedHost",
            Self::PrivateAddressBlocked(_) => "PrivateAddressBlocked",
            Self::DoHResolutionFailed(_) => "DoHResolutionFailed",
            Self::RelayUnavailable(_) => "RelayUnavailable",
        }
    }
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PolicyDenied => write!(f, "SpiralCore bridge denied by active MCP policy"),
            Self::InsecureTransportRejected => {
                write!(f, "insecure plain HTTP/WS transport rejected by R4-39")
            }
            Self::InvalidVsockCid => write!(f, "invalid SpiralCore vsock CID pair"),
            Self::GuestRamSecretContamination => {
                write!(f, "secret contamination detected in MVM guest RAM")
            }
            Self::HostNotAllowlistedForSecret => {
                write!(f, "target host is not allowlisted for secret substitution")
            }
            Self::MissingSecretPlaceholder => {
                write!(f, "missing MVM secret placeholder in bridge request")
            }
            Self::PackageDigestMismatch => write!(f, "package SHA-256 digest mismatch"),
            Self::PackageInvalidElf64 => {
                write!(f, "package is not a valid 64-bit EM_RISCV (243) ELF binary")
            }
            Self::PackageSizeExceedsQuota => write!(f, "package exceeds 4 MB quota"),
            Self::InvalidBridgeJson(msg) => write!(f, "invalid bridge JSON: {msg}"),
            Self::IntegrityMismatch(msg) => write!(f, "subresource integrity mismatch: {msg}"),
            Self::EsbuildAdapterUnavailable(msg) => {
                write!(f, "esbuild adapter unavailable: {msg}")
            }
            Self::WasiRuntimeError(msg) => write!(f, "WASI runtime error: {msg}"),
            Self::SqliteRuntimeError(msg) => write!(f, "SQLite runtime error: {msg}"),
            Self::CapabilityDenied(host) => {
                write!(
                    f,
                    "capability denied for host '{host}' under Prism capabilities/1 Sandboxed mode"
                )
            }
            Self::RelayConsentRequired => {
                write!(
                    f,
                    "Full SpiralCore Internet mode requires explicit user consent (consentGranted: true)"
                )
            }
            Self::RelayForbiddenForCredentialedHost => {
                write!(
                    f,
                    "SpiralCore AF_VSOCK relay failback is forbidden for credentialed host or Authorization header"
                )
            }
            Self::PrivateAddressBlocked(target) => {
                write!(f, "private or link-local SSRF target blocked: {target}")
            }
            Self::DoHResolutionFailed(msg) => write!(f, "RFC 8484 DoH resolution failed: {msg}"),
            Self::RelayUnavailable(msg) => {
                write!(f, "SpiralCore AF_VSOCK relay unavailable: {msg}")
            }
        }
    }
}

/// Prism `capabilities/1` outbound network mode (`R4-59`, `R4-63`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkGateMode {
    /// Mode A (`Sandboxed`): restricted to the pinned package registry, GitHub, HuggingFace, and DoH hosts.
    Sandboxed,
    /// Mode B (`Full SpiralCore Internet`): permits general internet via direct fetch/DoH first with consented unauthenticated relay failback.
    FullSpiralcore,
    /// Mode C (`SpiralCoreNet`): Studio default transparent `AF_INET`->`SPVS` encapsulation & unprompted public CORS preemption (`R4-63`).
    SpiralCoreNet,
}

/// Exact Prism `capabilities/1` `Sandboxed` host allowlist (`R4-59`).
pub const SANDBOXED_CAPABILITY_HOSTS: &[&str] = &[
    "registry.npmjs.org",
    "esm.sh",
    "pypi.org",
    "files.pythonhosted.org",
    "cdn.jsdelivr.net",
    "github.com",
    "api.github.com",
    "huggingface.co",
    "cloudflare-dns.com",
    "dns.google",
];

/// Return `true` when `host` matches the Prism `capabilities/1` `Sandboxed` allowlist (`R4-59`).
pub fn is_sandboxed_capability_host(host: &str) -> bool {
    let lower = host.trim().trim_end_matches('.').to_ascii_lowercase();
    SANDBOXED_CAPABILITY_HOSTS.iter().any(|&h| h == lower)
}

/// Return `true` when `host` is a loopback, RFC 1918, CGNAT, link-local, or internal SSRF target (`R4-59`).
pub fn is_private_or_loopback_host(host: &str) -> bool {
    let clean = host
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim_end_matches('.')
        .to_ascii_lowercase();
    if clean.is_empty() {
        return false;
    }
    if clean == "localhost"
        || clean == "ip6-localhost"
        || clean == "ip6-loopback"
        || clean.ends_with(".localhost")
        || clean.ends_with(".local")
        || clean.ends_with(".internal")
    {
        return true;
    }
    if let Ok(ip) = clean.parse::<std::net::IpAddr>() {
        return match ip {
            std::net::IpAddr::V4(v4) => is_private_ipv4_octets(v4.octets()),
            std::net::IpAddr::V6(v6) => {
                if v6.is_loopback() || v6.is_unspecified() {
                    return true;
                }
                if let Some(mapped) = v6.to_ipv4_mapped() {
                    return is_private_ipv4_octets(mapped.octets());
                }
                let seg0 = v6.segments()[0];
                (seg0 & 0xfe00) == 0xfc00 || (seg0 & 0xffc0) == 0xfe80 || (seg0 >> 8) == 0xff
            }
        };
    }
    false
}

fn is_private_ipv4_octets(octets: [u8; 4]) -> bool {
    let [a, b, _, _] = octets;
    a == 0
        || a == 10
        || (a == 100 && (64..=127).contains(&b))
        || a == 127
        || (a == 169 && b == 254)
        || (a == 172 && (16..=31).contains(&b))
        || (a == 192 && b == 168)
        || a >= 224
}

impl std::error::Error for BridgeError {}

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
    /// Canonical protected path (`prism/*.lex.tex`, `app/state`) refused mutation (`R4-57`).
    CanonicalPathProtected(String),
    /// `SpiralCore` `R4-39` connected bridge or package admission refused the operation.
    BridgeRejected(BridgeError),
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
            Self::CanonicalPathProtected(p) => {
                write!(f, "mvm: refused mutation of protected canonical path '{p}'")
            }
            Self::BridgeRejected(e) => write!(f, "{e}"),
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
    /// Formal R4.MvmLinuxContainer.
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

/// Suspended `SpiralCore` (`CID 3 -> CID 2`) operation awaiting host Worker relay completion (`R4-39`).
#[derive(Debug, Clone)]
pub struct MvmBridgePending {
    /// Outbound `SpiralCore` bridge request JSON (`dns_query`, `http_request`, `pkg_fetch`, `git_smart_http`).
    pub bridge_req: serde_json::Value,
    /// Deterministic guest continuation state passed back into `complete_bridge`.
    pub resume_state: serde_json::Value,
    /// Verified 16-byte `SPVS` frames emitted for this request.
    pub vsock_frames: Vec<VsockPacket>,
}

/// Outcome of an interactive MicroVM shell turn (`Completed` locally or `NeedsBridge` over `AF_VSOCK = 40`).
#[derive(Debug, Clone)]
pub enum MvmInteractiveOutcome {
    /// Command chain completed synchronously inside `MVM-Wasm`.
    Completed(MvmExecOutcome),
    /// Command requires an asynchronous `SpiralCore` (`CID 3 -> CID 2`) bridge round-trip.
    NeedsBridge(MvmBridgePending),
}

/// Metadata for an admitted `ELF64` (`EM_RISCV = 243`) userland binary installed into `/usr/local/bin`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InstalledPackageMeta {
    /// Binary name under `/usr/local/bin/<name>`.
    pub name: String,
    /// Verified FIPS 180-4 SHA-256 hex digest.
    pub sha256: String,
    /// Binary size in bytes (`<= 4 MB`).
    pub byte_len: u32,
    /// Source URL or catalog identifier.
    pub source: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct GitCommitRecord {
    oid: String,
    parent_oid: Option<String>,
    message: String,
    tree: BTreeMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct GitRepoState {
    branch: String,
    remote_url: Option<String>,
    head_oid: Option<String>,
    staged: BTreeMap<String, Option<String>>,
    head_tree: BTreeMap<String, String>,
    commits: Vec<GitCommitRecord>,
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

/// Inode classification within guest MicroVM filesystem.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum InodeKind {
    /// Regular file backed by contiguous 64 KB sectors.
    Regular {
        /// Initial sector index in CoW store.
        sector_start: u32,
        /// Total byte length of stored file.
        byte_len: usize,
    },
    /// Symbolic link pointing to target pathname.
    Symlink {
        /// Target path for symlink.
        target: String,
    },
}

/// Metadata record for a guest filesystem inode.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuestInode {
    /// File mode bits (permissions and type flags).
    pub mode: u32,
    /// Storage kind and sector pointer.
    pub kind: InodeKind,
}

/// Filesystem superblock metadata sector payload for session resumption.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MvmSuperblockMeta {
    /// Magic identifier string `MVMSUPER`.
    pub magic: String,
    /// Set of directory paths present in guest filesystem.
    pub dirs: BTreeSet<String>,
    /// Mapping of absolute file paths to guest inode records.
    pub inodes: BTreeMap<String, GuestInode>,
    /// Metadata of packages installed in the guest environment.
    pub installed_packages: BTreeMap<String, InstalledPackageMeta>,
    /// Active working directory at state capture.
    pub current_cwd: String,
    /// Workspace paths synchronized from host.
    pub synced_workspace_paths: BTreeSet<String>,
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
    InetEncapsulatedSocket {
        family: u16,
        peer_ip: String,
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
    fn guest_range(&self, addr: u64, len: usize) -> Result<std::ops::Range<usize>, MvmError> {
        let start = usize::try_from(addr)
            .map_err(|_| MvmError::CpuTrap(format!("guest address out of range: 0x{addr:x}")))?;
        let end = start
            .checked_add(len)
            .ok_or_else(|| MvmError::CpuTrap(format!("guest address overflow: 0x{addr:x}")))?;
        if end > self.ram.len() {
            return Err(MvmError::CpuTrap(format!(
                "memory fault at 0x{addr:x} len={len}"
            )));
        }
        Ok(start..end)
    }

    /// Formal R4.MvmLinuxContainer.
    fn read_u8(&self, addr: u64) -> Result<u8, MvmError> {
        let range = self.guest_range(addr, 1)?;
        Ok(self.ram[range.start])
    }

    fn read_u16(&self, addr: u64) -> Result<u16, MvmError> {
        let range = self.guest_range(addr, 2)?;
        Ok(u16::from_le_bytes([
            self.ram[range.start],
            self.ram[range.start + 1],
        ]))
    }

    fn read_u32(&self, addr: u64) -> Result<u32, MvmError> {
        let range = self.guest_range(addr, 4)?;
        Ok(u32::from_le_bytes([
            self.ram[range.start],
            self.ram[range.start + 1],
            self.ram[range.start + 2],
            self.ram[range.start + 3],
        ]))
    }

    fn read_u64(&self, addr: u64) -> Result<u64, MvmError> {
        let range = self.guest_range(addr, 8)?;
        let mut b = [0u8; 8];
        b.copy_from_slice(&self.ram[range]);
        Ok(u64::from_le_bytes(b))
    }

    /// Formal R4.MvmLinuxContainer.
    fn write_bytes(&mut self, addr: u64, data: &[u8]) -> Result<(), MvmError> {
        let range = self.guest_range(addr, data.len())?;
        self.ram[range].copy_from_slice(data);
        Ok(())
    }

    /// Formal R4.MvmLinuxContainer.
    fn read_cstr(&self, addr: u64) -> Result<String, MvmError> {
        let start = usize::try_from(addr)
            .map_err(|_| MvmError::CpuTrap(format!("unterminated cstr at 0x{addr:x}")))?;
        let mut idx = start;
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
#[derive(Clone)]
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
    current_cwd: String,
    policy_mode: McpPolicyMode,
    synced_workspace_paths: BTreeSet<String>,
    installed_packages: BTreeMap<String, InstalledPackageMeta>,
    next_bridge_seq: u64,
    env_vars: BTreeMap<String, String>,
    last_exit_code: i32,
    workspace_sync_epoch: u64,
    workspace_diff_baseline: BTreeMap<String, String>,
    /// Whether Prism `capabilities/1` outbound host gating is active (`R4-59`).
    pub network_gate_enforced: bool,
    /// Active Prism `capabilities/1` network mode (`Sandboxed` vs `FullSpiralcore`, `R4-59`).
    pub network_mode: NetworkGateMode,
    /// Whether explicit `Full SpiralCore Internet` consent was granted (`R4-59`).
    pub relay_consent_granted: bool,
    /// Configured SpiralCore `AF_VSOCK` WebSocket relay URL (`R4-59`).
    pub relay_ws_url: String,
}

impl MvmSandbox {
    /// List normalized guest file paths matching `prefix`.
    pub fn list_guest_paths(&self, prefix: &str) -> Vec<String> {
        let norm_prefix = if prefix.is_empty() || prefix == "/" {
            "/".to_string()
        } else {
            Self::normalize_guest_path("/", prefix)
        };
        self.inodes
            .keys()
            .filter(|k| {
                norm_prefix == "/"
                    || k.as_str() == norm_prefix.as_str()
                    || k.starts_with(&format!("{}/", norm_prefix.trim_end_matches('/')))
            })
            .cloned()
            .collect()
    }

    /// Remove a regular guest file or symlink, refusing protected canonical and system paths.
    pub fn remove_guest_file(&mut self, path: &str) -> Result<(), MvmError> {
        let norm = Self::normalize_guest_path("/", path);
        if Self::is_protected_guest_path(&norm) {
            return Err(MvmError::CanonicalPathProtected(norm));
        }
        if let Some(rel) = norm.strip_prefix("/workspace/") {
            self.synced_workspace_paths.remove(rel);
            self.workspace_diff_baseline.remove(rel);
        }
        if !self.remove_inode_reclaiming(&norm) {
            return Err(MvmError::NotFound(norm));
        }
        Ok(())
    }

    /// Reclaim CoW sectors owned solely by `removed_inode`.
    pub(super) fn free_inode_sectors(&mut self, removed_inode: &GuestInode) {
        let InodeKind::Regular {
            sector_start,
            byte_len,
        } = &removed_inode.kind
        else {
            return;
        };
        let count = (*byte_len).div_ceil(MVM_COW_SECTOR_BYTES).max(1) as u32;
        for sec_id in *sector_start..sector_start.saturating_add(count) {
            let still_referenced = self.inodes.values().any(|ino| {
                if let InodeKind::Regular {
                    sector_start: os,
                    byte_len: ol,
                } = &ino.kind
                {
                    let oc = (*ol).div_ceil(MVM_COW_SECTOR_BYTES).max(1) as u32;
                    sec_id >= *os && sec_id < os.saturating_add(oc)
                } else {
                    false
                }
            });
            if !still_referenced {
                self.cow_sectors.remove(&sec_id);
                self.dirty_sectors.remove(&sec_id);
            }
        }
    }

    /// Remove `norm_path` from `self.inodes` and reclaim unreferenced CoW sectors.
    pub(super) fn remove_inode_reclaiming(&mut self, norm_path: &str) -> bool {
        if let Some(old) = self.inodes.remove(norm_path) {
            self.free_inode_sectors(&old);
            true
        } else {
            false
        }
    }

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
            "/usr/local",
            "/usr/local/bin",
            "/workspace",
            "/tmp",
            "/etc",
            "/etc/pkg",
        ] {
            dirs.insert(d.to_string());
        }
        let mut env_vars = BTreeMap::new();
        env_vars.insert(
            "PATH".to_string(),
            "/usr/local/bin:/bin:/sbin:/usr/bin:/workspace".to_string(),
        );
        env_vars.insert("HOME".to_string(), "/workspace".to_string());
        env_vars.insert("USER".to_string(), "lexlean".to_string());
        env_vars.insert("SHELL".to_string(), "/bin/sh".to_string());
        env_vars.insert("PWD".to_string(), "/workspace".to_string());
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
            current_cwd: "/workspace".to_string(),
            policy_mode: McpPolicyMode::Safe,
            synced_workspace_paths: BTreeSet::new(),
            installed_packages: BTreeMap::new(),
            next_bridge_seq: 1,
            env_vars,
            last_exit_code: 0,
            workspace_sync_epoch: 0,
            workspace_diff_baseline: BTreeMap::new(),
            network_gate_enforced: true,
            network_mode: NetworkGateMode::Sandboxed,
            relay_consent_granted: false,
            relay_ws_url: String::new(),
        };
        vm.mount_initramfs(INITRAMFS_IMAGE)?;
        vm.seed_r4_63_headers_and_self_host()?;
        vm.dirty_sectors.clear();
        Ok(vm)
    }

    /// Formal R4.MvmLinuxContainer.
    /// Active interactive shell working directory inside the guest filesystem.
    pub fn current_cwd(&self) -> &str {
        &self.current_cwd
    }

    /// Formal R4.MvmLinuxContainer return the last shell exit status code inside MVM (`R4-63`).
    pub fn last_exit_code(&self) -> i32 {
        self.last_exit_code
    }

    /// Formal R4.MvmLinuxContainer verify guest CoW VFS sector consistency for `SingleSourceMvmCow` (`R4-63`).
    pub fn is_cow_vfs_consistent(&self) -> bool {
        if !self.dirs.contains("/workspace") {
            return false;
        }
        self.inodes.values().all(|inode| match &inode.kind {
            InodeKind::Symlink { .. } => true,
            InodeKind::Regular {
                sector_start,
                byte_len,
            } => {
                let needed = byte_len.div_ceil(MVM_COW_SECTOR_BYTES).max(1) as u32;
                (0..needed).all(|off| self.cow_sectors.contains_key(&(sector_start + off)))
            }
        })
    }

    /// Formal R4.MvmLinuxContainer.
    /// Active `R4-39` MCP bridge policy mode (`Safe`, `Confirm`, or `Deny`).
    pub fn policy_mode(&self) -> McpPolicyMode {
        self.policy_mode
    }

    /// Formal R4.MvmLinuxContainer.
    /// Update the active `R4-39` MCP bridge policy mode.
    pub fn set_policy_mode(&mut self, mode: McpPolicyMode) {
        self.policy_mode = mode;
    }

    /// Formal R4.MvmLinuxContainer.
    /// Admitted `ELF64` (`EM_RISCV = 243`) packages installed under `/usr/local/bin`.
    pub fn installed_packages(&self) -> &BTreeMap<String, InstalledPackageMeta> {
        &self.installed_packages
    }

    /// Guest `virtio-vsock` Context ID (`CID`).
    pub fn guest_cid(&self) -> u16 {
        self.guest_cid
    }

    /// Canonical guest ISA identifier.
    pub fn architecture(&self) -> &'static str {
        "riscv64-rv64imac"
    }

    /// Formal R4.MvmLinuxContainer.
    /// Count of 64 KB Copy-on-Write sectors dirtied since boot or last snapshot restore.
    pub fn dirty_cow_sector_count(&self) -> usize {
        self.dirty_sectors.len()
    }

    /// Export dirty 64 KB CoW sectors `(sector_id, bytes)` for Worker OPFS persistence.
    pub fn export_dirty_cow_sectors(&self) -> Vec<(u32, Vec<u8>)> {
        let mut list: Vec<(u32, Vec<u8>)> = self
            .dirty_sectors
            .iter()
            .filter(|id| **id != MVM_SUPERBLOCK_SECTOR_ID)
            .filter_map(|id| self.cow_sectors.get(id).map(|b| (*id, b.clone())))
            .collect();

        if !list.is_empty() || !self.inodes.is_empty() {
            let meta = MvmSuperblockMeta {
                magic: "MVMSUPER".to_string(),
                dirs: self.dirs.clone(),
                inodes: self.inodes.clone(),
                installed_packages: self.installed_packages.clone(),
                current_cwd: self.current_cwd.clone(),
                synced_workspace_paths: self.synced_workspace_paths.clone(),
            };
            if let Ok(json_str) = serde_json::to_string(&meta) {
                let json_bytes = json_str.as_bytes();
                let mut sector = vec![0u8; MVM_COW_SECTOR_BYTES];
                let copy_len = json_bytes.len().min(MVM_COW_SECTOR_BYTES);
                sector[..copy_len].copy_from_slice(&json_bytes[..copy_len]);
                list.push((MVM_SUPERBLOCK_SECTOR_ID, sector));
            }
        }
        list
    }

    /// Hydrate 64 KB CoW sectors loaded from OPFS `FileSystemSyncAccessHandle`.
    pub fn import_cow_sectors(&mut self, sectors: &[(u32, Vec<u8>)]) -> usize {
        let mut count = 0;
        for (id, data) in sectors {
            if data.len() == MVM_COW_SECTOR_BYTES {
                if *id == MVM_SUPERBLOCK_SECTOR_ID {
                    let zero_idx = data.iter().position(|&b| b == 0).unwrap_or(data.len());
                    if let Ok(meta) = serde_json::from_slice::<MvmSuperblockMeta>(&data[..zero_idx])
                    {
                        if meta.magic == "MVMSUPER" {
                            let has_workspace_snapshot =
                                meta.inodes.keys().any(|k| k.starts_with("/workspace/"));
                            if has_workspace_snapshot {
                                self.inodes.retain(|k, _| {
                                    !k.starts_with("/workspace/")
                                        || meta.inodes.contains_key(k)
                                        || Self::is_protected_canonical_guest_path(k)
                                });
                                self.synced_workspace_paths = meta.synced_workspace_paths;
                            } else {
                                self.synced_workspace_paths
                                    .extend(meta.synced_workspace_paths);
                            }
                            self.dirs.extend(meta.dirs);
                            self.inodes.extend(meta.inodes);
                            self.installed_packages.extend(meta.installed_packages);
                            if !meta.current_cwd.is_empty() {
                                self.current_cwd = meta.current_cwd;
                            }
                        }
                    }
                } else {
                    self.cow_sectors.insert(*id, data.clone());
                    self.dirty_sectors.insert(*id);
                    if *id >= self.next_sector {
                        self.next_sector = *id + 1;
                    }
                    count += 1;
                }
            }
        }
        count
    }

    /// Formal R4.StudioFrame remove a non-canonical workspace file inode during OPFS boot sync.
    pub(crate) fn remove_workspace_file_inode(&mut self, rel_path: &str) -> bool {
        let clean = rel_path.trim().trim_start_matches('/');
        if clean.is_empty() {
            return false;
        }
        let guest = format!("/workspace/{clean}");
        if Self::is_protected_guest_path(&guest) {
            return false;
        }
        self.synced_workspace_paths.remove(clean);
        self.remove_inode_reclaiming(&guest)
    }

    /// Formal R4.StudioFrame list persisted `/workspace/` relative paths currently present in guest inodes.
    pub(crate) fn workspace_rel_paths(&self) -> Vec<String> {
        self.inodes
            .keys()
            .filter_map(|p| p.strip_prefix("/workspace/"))
            .filter(|rel| {
                !rel.is_empty()
                    && !is_git_internal_rel_path(rel)
                    && !is_canonical_protected_rel(rel)
            })
            .map(str::to_string)
            .collect()
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

    /// Formal R4.MvmLinuxContainer.
    /// Roll back the MicroVM storage layer to a previously captured CoW snapshot.
    pub fn restore_cow(&mut self, snap: &MvmCowSnapshot) {
        self.inodes = snap.inodes.clone();
        self.dirs = snap.dirs.clone();
        self.cow_sectors = snap.cow_sectors.clone();
        self.dirty_sectors = snap.dirty_sectors.clone();
        self.next_sector = snap.next_sector;
    }

    /// Encode a complete, self-verifying Copy-on-Write snapshot for OPFS persistence.
    pub fn export_cow_snapshot_bytes(&self) -> Result<Vec<u8>, String> {
        #[derive(serde::Serialize)]
        struct CowWireMeta<'a> {
            state_sha256: &'a str,
            dirs: &'a BTreeSet<String>,
            inodes: &'a BTreeMap<String, GuestInode>,
            installed_packages: &'a BTreeMap<String, InstalledPackageMeta>,
            current_cwd: &'a str,
            synced_workspace_paths: &'a BTreeSet<String>,
            dirty_sectors: &'a BTreeSet<u32>,
            next_sector: u32,
        }

        let state_sha256 = self.cow_state_sha256();
        let meta = CowWireMeta {
            state_sha256: &state_sha256,
            dirs: &self.dirs,
            inodes: &self.inodes,
            installed_packages: &self.installed_packages,
            current_cwd: &self.current_cwd,
            synced_workspace_paths: &self.synced_workspace_paths,
            dirty_sectors: &self.dirty_sectors,
            next_sector: self.next_sector,
        };
        let meta_json = serde_json::to_vec(&meta).map_err(|e| format!("cow serialize: {e}"))?;
        let mut body = Vec::with_capacity(8 + meta_json.len() + self.cow_sectors.len() * 1024);
        body.extend_from_slice(&(meta_json.len() as u32).to_le_bytes());
        body.extend_from_slice(&meta_json);
        body.extend_from_slice(&(self.cow_sectors.len() as u32).to_le_bytes());
        for (&sector_id, sector) in &self.cow_sectors {
            let trimmed_len = sector
                .iter()
                .rposition(|&b| b != 0)
                .map_or(0, |idx| idx + 1);
            body.extend_from_slice(&sector_id.to_le_bytes());
            body.extend_from_slice(&(trimmed_len as u32).to_le_bytes());
            body.extend_from_slice(&sector[..trimmed_len]);
        }
        let checksum = Sha256::digest(&body);
        let mut out = Vec::with_capacity(8 + 32 + body.len());
        out.extend_from_slice(b"UORCOW01");
        out.extend_from_slice(&checksum);
        out.extend_from_slice(&body);
        Ok(out)
    }

    /// Restore a binary Copy-on-Write snapshot, validating header bounds, full SHA-256 integrity,
    /// VFS consistency, and cwd normalization.
    pub fn import_cow_snapshot_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        #[derive(serde::Deserialize)]
        struct CowWireMetaOwned {
            state_sha256: String,
            dirs: BTreeSet<String>,
            inodes: BTreeMap<String, GuestInode>,
            installed_packages: BTreeMap<String, InstalledPackageMeta>,
            current_cwd: String,
            synced_workspace_paths: BTreeSet<String>,
            dirty_sectors: BTreeSet<u32>,
            next_sector: u32,
        }

        if bytes.len() < 48 || &bytes[..8] != b"UORCOW01" {
            return Err("invalid UORCOW01 snapshot header".to_string());
        }
        let body = &bytes[40..];
        let actual_checksum = Sha256::digest(body);
        if actual_checksum.as_slice() != &bytes[8..40] {
            return Err("UORCOW01 wire checksum verification failed".to_string());
        }

        let meta_len = u32::from_le_bytes([body[0], body[1], body[2], body[3]]) as usize;
        let mut cur = 4usize;
        let meta_end = cur
            .checked_add(meta_len)
            .ok_or_else(|| "snapshot metadata length overflow".to_string())?;
        let meta_slice = body
            .get(cur..meta_end)
            .ok_or_else(|| "truncated snapshot metadata".to_string())?;
        let meta: CowWireMetaOwned =
            serde_json::from_slice(meta_slice).map_err(|e| format!("bad snapshot metadata: {e}"))?;
        cur = meta_end;

        let count_bytes = body
            .get(cur..cur + 4)
            .ok_or_else(|| "missing snapshot sector count".to_string())?;
        let sector_count =
            u32::from_le_bytes([count_bytes[0], count_bytes[1], count_bytes[2], count_bytes[3]])
                as usize;
        cur += 4;

        let mut cow_sectors = BTreeMap::new();
        for _ in 0..sector_count {
            let hdr = body
                .get(cur..cur + 8)
                .ok_or_else(|| "truncated snapshot sector header".to_string())?;
            cur += 8;
            let sector_id = u32::from_le_bytes([hdr[0], hdr[1], hdr[2], hdr[3]]);
            let trimmed_len = u32::from_le_bytes([hdr[4], hdr[5], hdr[6], hdr[7]]) as usize;
            if trimmed_len > MVM_COW_SECTOR_BYTES {
                return Err(format!("snapshot sector {sector_id} exceeds 64 KB"));
            }
            let payload_end = cur
                .checked_add(trimmed_len)
                .ok_or_else(|| "snapshot sector length overflow".to_string())?;
            let payload = body
                .get(cur..payload_end)
                .ok_or_else(|| "truncated snapshot sector payload".to_string())?;
            cur = payload_end;
            let mut full = vec![0u8; MVM_COW_SECTOR_BYTES];
            full[..trimmed_len].copy_from_slice(payload);
            cow_sectors.insert(sector_id, full);
        }
        if cur != body.len() {
            return Err("trailing bytes in UORCOW01 snapshot".to_string());
        }

        for (path, inode) in &meta.inodes {
            if !path.starts_with('/') {
                return Err(format!("invalid inode path in snapshot: {path}"));
            }
            if let InodeKind::Regular {
                sector_start,
                byte_len,
            } = &inode.kind
            {
                let needed = u32::try_from((*byte_len).div_ceil(MVM_COW_SECTOR_BYTES).max(1))
                    .map_err(|_| "snapshot sector count overflow".to_string())?;
                for off in 0..needed {
                    let sid = sector_start
                        .checked_add(off)
                        .ok_or_else(|| "snapshot sector index overflow".to_string())?;
                    if !cow_sectors.contains_key(&sid) {
                        return Err(format!("missing sector {sid} for inode {path}"));
                    }
                }
            }
        }

        let norm_cwd = if meta.current_cwd.trim().is_empty() {
            "/workspace".to_string()
        } else {
            Self::normalize_guest_path("/workspace", &meta.current_cwd)
        };
        let target_cwd = if meta.dirs.contains(&norm_cwd) {
            norm_cwd
        } else {
            "/workspace".to_string()
        };

        let backup = self.clone();
        self.dirs = meta.dirs;
        self.inodes = meta.inodes;
        self.installed_packages = meta.installed_packages;
        self.current_cwd = target_cwd.clone();
        self.env_vars.insert("PWD".to_string(), target_cwd);
        self.synced_workspace_paths = meta.synced_workspace_paths;
        self.dirty_sectors = meta.dirty_sectors;
        self.next_sector = meta.next_sector;
        self.cow_sectors = cow_sectors;

        if !self.is_cow_vfs_consistent() || self.cow_state_sha256() != meta.state_sha256 {
            *self = backup;
            return Err("UORCOW01 snapshot VFS/SHA-256 verification failed".to_string());
        }
        Ok(())
    }

    /// Formal R4.MvmLinuxContainer mount and SHA-256 verify either raw `UORMVM01` or gzip-compressed `UORMVMZ1` initramfs images (`R4-63`).
    pub fn mount_initramfs_image(&mut self, img: &[u8]) -> Result<(), MvmError> {
        self.mount_initramfs(img)
    }

    fn mount_initramfs(&mut self, img: &[u8]) -> Result<(), MvmError> {
        let decompressed;
        let raw_img = if img.len() >= 8 && &img[0..8] == b"UORMVMZ1" {
            decompressed = gzip_decompress_bytes(&img[8..]).map_err(|e| {
                MvmError::CorruptInitramfs(format!("corrupt UORMVMZ1 payload: {e}"))
            })?;
            decompressed.as_slice()
        } else {
            img
        };
        if raw_img.len() < 12 || &raw_img[0..8] != b"UORMVM01" {
            return Err(MvmError::CorruptInitramfs("bad magic".to_string()));
        }
        let count = u32::from_le_bytes([raw_img[8], raw_img[9], raw_img[10], raw_img[11]]) as usize;
        let mut cur = 12usize;
        for _ in 0..count {
            if cur + 42 > raw_img.len() {
                return Err(MvmError::CorruptInitramfs(
                    "truncated entry header".to_string(),
                ));
            }
            let path_len = u16::from_le_bytes([raw_img[cur], raw_img[cur + 1]]) as usize;
            let mode = u32::from_le_bytes([
                raw_img[cur + 2],
                raw_img[cur + 3],
                raw_img[cur + 4],
                raw_img[cur + 5],
            ]);
            let data_len = u32::from_le_bytes([
                raw_img[cur + 6],
                raw_img[cur + 7],
                raw_img[cur + 8],
                raw_img[cur + 9],
            ]) as usize;
            let expected_sha = &raw_img[cur + 10..cur + 42];
            cur += 42;
            if path_len
                .checked_add(data_len)
                .and_then(|sum| cur.checked_add(sum))
                .is_none_or(|end| end > raw_img.len())
            {
                return Err(MvmError::CorruptInitramfs(
                    "truncated entry payload".to_string(),
                ));
            }
            let path = std::str::from_utf8(&raw_img[cur..cur + path_len])
                .map_err(|_| MvmError::CorruptInitramfs("invalid utf8 path".to_string()))?
                .to_string();
            cur += path_len;
            let data = &raw_img[cur..cur + data_len];
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

    /// Formal R4.MvmLinuxContainer seed freestanding C headers (`/usr/include/*`), self-hosting MVM sources (`/usr/src/mvm/*`), and Linux developer tool symlinks (`R4-63`).
    fn seed_r4_63_headers_and_self_host(&mut self) -> Result<(), MvmError> {
        for d in [
            "/usr/include",
            "/usr/include/sys",
            "/usr/include/netinet",
            "/usr/include/arpa",
            "/usr/src",
            "/usr/src/mvm",
        ] {
            self.dirs.insert(d.to_string());
        }

        const SEED_FILES: &[(&str, &str)] = &[
            (
                "/usr/include/stddef.h",
                "#pragma once\n#define NULL 0\ntypedef unsigned long size_t;\n",
            ),
            (
                "/usr/include/stdint.h",
                "#pragma once\ntypedef long int64_t;\ntypedef unsigned long uint64_t;\n#define INT32_MAX 2147483647\n#define UINT32_MAX 4294967295\n",
            ),
            (
                "/usr/include/stdbool.h",
                "#pragma once\n#define true 1\n#define false 0\n",
            ),
            (
                "/usr/include/errno.h",
                "#pragma once\n#define ENOENT 2\n#define EBADF 9\n#define EACCES 13\n#define EINVAL 22\n#define ENOSYS 38\n#define EAFNOSUPPORT 97\n",
            ),
            (
                "/usr/include/stdio.h",
                "#pragma once\n#include <stddef.h>\nint puts(const char *s);\nint putchar(int c);\nint printf(const char *fmt, ...);\n",
            ),
            (
                "/usr/include/stdlib.h",
                "#pragma once\n#include <stddef.h>\nvoid exit(int status);\nvoid *malloc(size_t size);\nvoid *calloc(size_t nmemb, size_t size);\nvoid free(void *ptr);\nint abs(int x);\nint atoi(const char *s);\n",
            ),
            (
                "/usr/include/string.h",
                "#pragma once\n#include <stddef.h>\nsize_t strlen(const char *s);\nint strcmp(const char *a, const char *b);\nint strncmp(const char *a, const char *b, size_t n);\nchar *strcpy(char *dst, const char *src);\nvoid *memcpy(void *dst, const void *src, size_t n);\nvoid *memset(void *dst, int c, size_t n);\n",
            ),
            (
                "/usr/include/sys/types.h",
                "#pragma once\n#include <stddef.h>\n#include <stdint.h>\ntypedef long ssize_t;\ntypedef uint32_t socklen_t;\ntypedef uint16_t sa_family_t;\n",
            ),
            (
                "/usr/include/unistd.h",
                "#pragma once\n#include <stddef.h>\n#include <sys/types.h>\nssize_t read(int fd, void *buf, size_t count);\nssize_t write(int fd, const void *buf, size_t count);\nint close(int fd);\nint unlink(const char *pathname);\n",
            ),
            (
                "/usr/include/fcntl.h",
                "#pragma once\n#define O_RDONLY 0\n#define O_WRONLY 1\n#define O_RDWR 2\n#define O_CREAT 64\n#define O_TRUNC 512\n#define O_APPEND 1024\nint open(const char *pathname, int flags, ...);\nint openat(int dirfd, const char *pathname, int flags, ...);\n",
            ),
            (
                "/usr/include/sys/socket.h",
                "#pragma once\n#include <sys/types.h>\n#define AF_INET 2\n#define AF_INET6 10\n#define AF_VSOCK 40\n#define SOCK_STREAM 1\n#define SOCK_DGRAM 2\nstruct sockaddr {\n    uint16_t sa_family;\n    uint16_t sa_pad;\n};\nstruct sockaddr_vm {\n    uint16_t svm_family;\n    uint16_t svm_reserved1;\n    uint32_t svm_port;\n    uint32_t svm_cid;\n    uint32_t svm_zero;\n};\nint socket(int domain, int type, int protocol);\nint connect(int sockfd, const struct sockaddr *addr, socklen_t addrlen);\nssize_t send(int sockfd, const void *buf, size_t len, int flags);\nssize_t sendto(int sockfd, const void *buf, size_t len, int flags, const struct sockaddr *dest_addr, socklen_t addrlen);\nssize_t recv(int sockfd, void *buf, size_t len, int flags);\nssize_t recvfrom(int sockfd, void *buf, size_t len, int flags, struct sockaddr *src_addr, socklen_t *addrlen);\n",
            ),
            (
                "/usr/include/netinet/in.h",
                "#pragma once\n#include <stdint.h>\n#include <sys/socket.h>\nstruct in_addr {\n    uint32_t s_addr;\n};\nstruct sockaddr_in {\n    uint16_t sin_family;\n    uint16_t sin_port;\n    struct in_addr sin_addr;\n    uint64_t sin_zero;\n};\nuint16_t htons(uint16_t hostshort);\nuint16_t ntohs(uint16_t netshort);\nuint32_t htonl(uint32_t hostlong);\nuint32_t ntohl(uint32_t netlong);\n",
            ),
            (
                "/usr/include/arpa/inet.h",
                "#pragma once\n#include <netinet/in.h>\n",
            ),
            (
                "/usr/include/netdb.h",
                "#pragma once\n#include <sys/socket.h>\nstruct addrinfo {\n    int ai_flags;\n    int ai_family;\n    int ai_socktype;\n    int ai_protocol;\n};\n",
            ),
            (
                "/usr/src/mvm/README.md",
                "# MVM Self-Hosting Guest Source Pack\nBuild and run `/usr/src/mvm/mvm_hello` directly inside MVM:\n  make -C /usr/src/mvm && /usr/src/mvm/mvm_hello\n",
            ),
            (
                "/usr/src/mvm/mvm_util.h",
                "#pragma once\nstruct MvmVec2 {\n    int x;\n    int y;\n};\nint mvm_dot2(struct MvmVec2 *a, struct MvmVec2 *b);\n",
            ),
            (
                "/usr/src/mvm/mvm_util.c",
                "#include \"mvm_util.h\"\nint mvm_dot2(struct MvmVec2 *a, struct MvmVec2 *b) {\n    return a->x * b->x + a->y * b->y;\n}\n",
            ),
            (
                "/usr/src/mvm/mvm_hello.c",
                "#include <stdio.h>\n#include \"mvm_util.h\"\nint main() {\n    struct MvmVec2 a;\n    struct MvmVec2 b;\n    a.x = 3;\n    a.y = 4;\n    b.x = 2;\n    b.y = 5;\n    int dot = mvm_dot2(&a, &b);\n    printf(\"MVM_SELF_HOST_OK dot=%d\\n\", dot);\n    return 0;\n}\n",
            ),
            (
                "/usr/src/mvm/vsock_client.c",
                "#include <stdio.h>\n#include <string.h>\n#include <sys/socket.h>\nint main() {\n    int fd = socket(AF_VSOCK, SOCK_STREAM, 0);\n    if (fd < 0) return 1;\n    struct sockaddr_vm addr;\n    addr.svm_family = AF_VSOCK;\n    addr.svm_reserved1 = 0;\n    addr.svm_port = 5257;\n    addr.svm_cid = 2;\n    addr.svm_zero = 0;\n    if (connect(fd, &addr, sizeof(addr)) < 0) return 2;\n    char msg[8];\n    strcpy(msg, \"PING\");\n    send(fd, msg, 4, 0);\n    return 0;\n}\n",
            ),
            (
                "/usr/src/mvm/Makefile",
                "CC = cc\nCFLAGS = -I/usr/src/mvm -Wall\n\nall: /usr/src/mvm/mvm_hello\n\n/usr/src/mvm/mvm_hello: /usr/src/mvm/mvm_util.c /usr/src/mvm/mvm_hello.c /usr/src/mvm/mvm_util.h\n\t$(CC) $(CFLAGS) /usr/src/mvm/mvm_util.c /usr/src/mvm/mvm_hello.c -o /usr/src/mvm/mvm_hello\n\nclean:\n\trm -f /usr/src/mvm/mvm_hello\n",
            ),
        ];

        for &(path, content) in SEED_FILES {
            self.write_inode_with_mode(path, content.as_bytes(), 0o100644)?;
        }

        for tool in [
            "rg", "xargs", "patch", "make", "df", "free", "xxd", "cargo", "rustc",
        ] {
            let link_path = format!("/bin/{tool}");
            self.inodes.entry(link_path).or_insert_with(|| GuestInode {
                mode: 0o120777,
                kind: InodeKind::Symlink {
                    target: "/bin/mvmbox".to_string(),
                },
            });
        }
        Ok(())
    }

    /// Formal R4.MvmLinuxContainer.
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

    /// Formal R4.MvmLinuxContainer.
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

    /// Formal R4.MvmLinuxContainer.
    fn write_inode_with_mode(
        &mut self,
        path: &str,
        data: &[u8],
        mode: u32,
    ) -> Result<(), MvmError> {
        let norm = Self::normalize_guest_path("/", path);
        self.ensure_parent_dirs(&norm);
        if let Some(old) = self.inodes.remove(&norm) {
            self.free_inode_sectors(&old);
        }
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

    pub(super) fn write_symlink_inode(&mut self, path: &str, target: &str) -> Result<(), String> {
        let norm = Self::normalize_guest_path("/", path);
        if Self::is_protected_guest_path(&norm) {
            return Err(format!("mvm: refused write to protected path '{norm}'"));
        }
        self.ensure_parent_dirs(&norm);
        if let Some(old) = self.inodes.remove(&norm) {
            self.free_inode_sectors(&old);
        }
        self.inodes.insert(
            norm,
            GuestInode {
                mode: 0o120777,
                kind: InodeKind::Symlink {
                    target: target.to_string(),
                },
            },
        );
        Ok(())
    }

    fn resolve_symlink_step(link_path: &str, target: &str) -> String {
        if target.starts_with('/') {
            Self::normalize_guest_path("/", target)
        } else {
            let parent = match link_path.rsplit_once('/') {
                Some((p, _)) if !p.is_empty() => p,
                _ => "/",
            };
            Self::normalize_guest_path(parent, target)
        }
    }

    /// Formal R4.MvmLinuxContainer.
    /// Write a regular file into the guest CoW filesystem (`0o100644` default permissions),
    /// refusing protected canonical/system paths (`prism/*.lex.tex`, `app/state`, `/bin`, `/sbin`, `/usr`, `/etc`) and symlinks to them.
    pub fn write_guest_file(&mut self, path: &str, data: &[u8]) -> Result<(), MvmError> {
        let norm = Self::normalize_guest_path("/", path);
        let mut cur = norm.clone();
        for _ in 0..8 {
            if Self::is_protected_guest_path(&cur) {
                return Err(MvmError::CanonicalPathProtected(norm));
            }
            match self.inodes.get(&cur) {
                Some(ino) => match &ino.kind {
                    InodeKind::Symlink { target } => {
                        cur = Self::resolve_symlink_step(&cur, target);
                    }
                    InodeKind::Regular { .. } => break,
                },
                None => break,
            }
        }
        let mode = if data.starts_with(b"\x7fELF") {
            0o100755
        } else {
            0o100644
        };
        self.write_inode_with_mode(&norm, data, mode)
    }

    /// Formal R4.MvmLinuxContainer.
    /// Mount an immutable canonical file (`prism/*.lex.tex`, `app/state`) write-once at setup (`R4-57`).
    pub fn seed_canonical_guest_file(&mut self, path: &str, data: &[u8]) -> Result<(), MvmError> {
        let norm = Self::normalize_guest_path("/", path);
        if !Self::is_protected_canonical_guest_path(&norm) || self.inodes.contains_key(&norm) {
            return Err(MvmError::CanonicalPathProtected(norm));
        }
        self.write_inode_with_mode(&norm, data, 0o100444)
    }

    /// Formal R4.MvmLinuxContainer.
    /// Read a file (resolving up to 8 symlink hops) from the guest CoW filesystem.
    pub fn read_guest_file(&self, path: &str) -> Option<Vec<u8>> {
        let mut cur = Self::normalize_guest_path("/", path);
        for _ in 0..8 {
            let inode = self.inodes.get(&cur)?;
            match &inode.kind {
                InodeKind::Symlink { target } => {
                    cur = Self::resolve_symlink_step(&cur, target);
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

    /// Formal R4.MvmLinuxContainer.
    /// Synchronize host Studio `/workspace` text files into the guest `/workspace` directory,
    /// propagating Studio deletions without disturbing `.git` metadata or compiled ELF binaries.
    pub fn sync_from_workspace_map(&mut self, workspace_files: &BTreeMap<String, String>) {
        let mut clean_map: BTreeMap<String, &String> = BTreeMap::new();
        for (rel, content) in workspace_files {
            let clean = rel.trim_start_matches('/').to_string();
            if clean.is_empty() || is_excluded_workspace_rel_path(&clean) {
                continue;
            }
            clean_map.insert(clean, content);
        }

        let removed: Vec<String> = self
            .synced_workspace_paths
            .iter()
            .filter(|old| !clean_map.contains_key(*old))
            .cloned()
            .collect();
        for old_rel in removed {
            let guest_path = format!("/workspace/{old_rel}");
            self.remove_inode_reclaiming(&guest_path);
        }

        for (clean, content) in &clean_map {
            let guest_path = format!("/workspace/{clean}");
            let unchanged = self
                .read_guest_file(&guest_path)
                .map(|cur| cur == content.as_bytes())
                .unwrap_or(false);
            if !unchanged {
                let _ = self.write_guest_file(&guest_path, content.as_bytes());
            }
        }

        self.synced_workspace_paths = clean_map.keys().cloned().collect();
    }

    /// Record pre-exec baseline SHA-256 digests from host Studio workspace files (`R4-57`).
    pub fn record_workspace_baseline_from_map(
        &mut self,
        workspace_files: &BTreeMap<String, String>,
    ) {
        let mut baseline = BTreeMap::new();
        for (rel, content) in workspace_files {
            let clean = rel.trim_start_matches('/');
            if !clean.is_empty()
                && !is_excluded_workspace_rel_path(clean)
                && !is_binary_workspace_bytes(content.as_bytes())
            {
                baseline.insert(clean.to_string(), sha256_hex_bytes(content.as_bytes()));
            }
        }
        self.workspace_diff_baseline = baseline;
    }

    /// Synchronize incoming base64 workspace snapshot JSON into MVM `/workspace` (`R4-57`).
    pub fn mvm_sync_workspace_in_json(&mut self, snapshot_json: &str) -> String {
        #[derive(serde::Deserialize)]
        struct SyncInReq {
            #[serde(default)]
            epoch: u64,
            files: Vec<SyncInEntry>,
        }
        #[derive(serde::Deserialize)]
        struct SyncInEntry {
            path: String,
            #[serde(default)]
            content_base64: String,
            #[serde(default)]
            deleted: bool,
        }

        let req: SyncInReq = match serde_json::from_str(snapshot_json.trim()) {
            Ok(v) => v,
            Err(e) => {
                return serde_json::json!({
                    "status": "error",
                    "error_code": "InvalidPayload",
                    "message": format!("invalid mvm_sync_workspace_in JSON: {e}"),
                })
                .to_string();
            }
        };

        let mut ignored_canonical_paths = 0usize;
        let mut decoded_ops: Vec<(String, bool, Vec<u8>)> = Vec::with_capacity(req.files.len());
        for entry in &req.files {
            let trimmed_raw = entry.path.trim();
            let stripped = trimmed_raw
                .strip_prefix("/workspace/")
                .unwrap_or_else(|| trimmed_raw.trim_start_matches('/'));
            let stripped = stripped.strip_prefix("./").unwrap_or(stripped);
            let mut parts: Vec<&str> = Vec::new();
            for seg in stripped.split('/') {
                if seg.is_empty() || seg == "." {
                    continue;
                }
                if seg == ".." {
                    return serde_json::json!({
                        "status": "error",
                        "error_code": "RefusedParentTraversal",
                        "message": format!("parent path traversal refused in workspace path `{}`", entry.path),
                    })
                    .to_string();
                }
                parts.push(seg);
            }
            if parts.is_empty() {
                return serde_json::json!({
                    "status": "error",
                    "error_code": "EmptyPath",
                    "message": "workspace path must not be empty",
                })
                .to_string();
            }
            let rel = parts.join("/");
            if is_canonical_protected_rel(&rel) {
                ignored_canonical_paths += 1;
                continue;
            }
            if entry.deleted {
                decoded_ops.push((rel, true, Vec::new()));
            } else {
                let bytes = match decode_base64_bytes(entry.content_base64.trim()) {
                    Ok(b) => b,
                    Err(e) => {
                        return serde_json::json!({
                            "status": "error",
                            "error_code": "InvalidBase64",
                            "message": format!("invalid base64 for `{rel}`: {e}"),
                        })
                        .to_string();
                    }
                };
                let sha = sha256_hex_bytes(&bytes);
                let sha_hi = u64::from_str_radix(&sha[..16], 16).unwrap_or(1).max(1);
                match admit_sandbox_write(
                    rel.len() as u64,
                    rel.contains(".."),
                    self.workspace_diff_baseline.len() as u64,
                    65_536,
                    bytes.len() as u64,
                    16 * 1024 * 1024,
                    sha_hi,
                ) {
                    SandboxAdmissionOutcome::AdmittedWorkspaceWrite(_, _, _) => {}
                    refused => {
                        return serde_json::json!({
                            "status": "error",
                            "error_code": format!("{refused:?}"),
                            "message": format!("R4.SecopsAudit rejected workspace sync `{rel}`: {refused:?}"),
                        })
                        .to_string();
                    }
                }
                decoded_ops.push((rel, false, bytes));
            }
        }

        if req.epoch != 0 && req.epoch < self.workspace_sync_epoch {
            return serde_json::json!({
                "status": "error",
                "error_code": "StaleSyncEpoch",
                "epoch": self.workspace_sync_epoch,
                "message": format!(
                    "stale sync epoch {} < active epoch {}",
                    req.epoch, self.workspace_sync_epoch
                ),
            })
            .to_string();
        }
        self.workspace_sync_epoch = self.workspace_sync_epoch.max(req.epoch);
        let mut applied_writes = 0usize;
        let mut applied_deletes = 0usize;
        for (rel, deleted, bytes) in decoded_ops {
            let guest_path = format!("/workspace/{rel}");
            if Self::is_protected_canonical_guest_path(&guest_path) {
                ignored_canonical_paths += 1;
                continue;
            }
            if deleted {
                if self.remove_inode_reclaiming(&guest_path) {
                    applied_deletes += 1;
                }
                self.workspace_diff_baseline.remove(&rel);
                self.synced_workspace_paths.remove(&rel);
            } else {
                let unchanged = self
                    .read_guest_file(&guest_path)
                    .map(|cur| cur == bytes)
                    .unwrap_or(false);
                let write_ok = if unchanged {
                    true
                } else if self.write_guest_file(&guest_path, &bytes).is_ok() {
                    applied_writes += 1;
                    true
                } else {
                    false
                };
                if write_ok
                    && !is_excluded_workspace_rel_path(&rel)
                    && !is_binary_workspace_bytes(&bytes)
                {
                    self.workspace_diff_baseline
                        .insert(rel.clone(), sha256_hex_bytes(&bytes));
                    self.synced_workspace_paths.insert(rel);
                }
            }
        }

        serde_json::json!({
            "status": "ok",
            "epoch": self.workspace_sync_epoch,
            "applied_writes": applied_writes,
            "applied_deletes": applied_deletes,
            "ignored_canonical_paths": ignored_canonical_paths,
            "cow_state_sha256": self.cow_state_sha256(),
        })
        .to_string()
    }

    /// Export minimal filtered UTF-8 `/workspace` writes + deletes since last sync,
    /// excluding `node_modules/**`, `.venv/**`, `**/__pycache__/**`, `.git/objects/**`,
    /// non-UTF-8 binaries (`*.db`, `*.wasm`, >1 MiB), and canonical rule files (`R4-57`).
    pub fn mvm_export_workspace_diff_json(&mut self) -> String {
        let paths: Vec<String> = self.inodes.keys().cloned().collect();
        let mut ignored_heavy_paths = 0usize;
        let mut current_eligible: BTreeMap<String, (Vec<u8>, String)> = BTreeMap::new();

        for path in paths {
            let Some(rel) = path.strip_prefix("/workspace/") else {
                continue;
            };
            if rel.is_empty() {
                continue;
            }
            let Some(bytes) = self.read_guest_file(&path) else {
                continue;
            };
            if is_excluded_workspace_rel_path(rel) || is_binary_workspace_bytes(&bytes) {
                ignored_heavy_paths += 1;
                continue;
            }
            let sha = sha256_hex_bytes(&bytes);
            current_eligible.insert(rel.to_string(), (bytes, sha));
        }

        let mut writes = Vec::new();
        let mut next_baseline = BTreeMap::new();
        for (rel, (bytes, sha)) in current_eligible {
            if self.workspace_diff_baseline.get(&rel) != Some(&sha) {
                writes.push(serde_json::json!({
                    "path": rel,
                    "content_base64": encode_base64_bytes(&bytes),
                }));
            }
            next_baseline.insert(rel, sha);
        }

        let mut deletes = Vec::new();
        for prev_rel in self.workspace_diff_baseline.keys() {
            if !next_baseline.contains_key(prev_rel) {
                deletes.push(prev_rel.clone());
            }
        }

        self.workspace_diff_baseline = next_baseline;
        if !writes.is_empty() || !deletes.is_empty() {
            self.workspace_sync_epoch = self.workspace_sync_epoch.saturating_add(1);
        }

        serde_json::json!({
            "status": "ok",
            "epoch": self.workspace_sync_epoch,
            "writes": writes,
            "deletes": deletes,
            "ignored_heavy_paths": ignored_heavy_paths,
            "cow_state_sha256": self.cow_state_sha256(),
        })
        .to_string()
    }

    /// Formal R4.MvmLinuxContainer snapshot UTF-8 files under guest `/workspace` directly without mutating VM state (`R4-63`).
    pub fn snapshot_workspace_text_files(&self) -> BTreeMap<String, String> {
        let mut out = BTreeMap::new();
        for path in self.inodes.keys() {
            if let Some(rel) = path.strip_prefix("/workspace/") {
                if rel.is_empty() || is_excluded_workspace_rel_path(rel) {
                    continue;
                }
                if let Some(bytes) = self.read_guest_file(path) {
                    if !is_binary_workspace_bytes(&bytes) {
                        if let Ok(text) = String::from_utf8(bytes) {
                            out.insert(rel.to_string(), text);
                        }
                    }
                }
            }
        }
        out
    }

    /// Formal R4.MvmLinuxContainer.
    /// Export UTF-8 files under guest `/workspace` (excluding heavy/binary/canonical paths)
    /// back to Studio's workspace map and update deletion tracking.
    pub fn export_workspace_text_files(&mut self) -> BTreeMap<String, String> {
        let out = self.snapshot_workspace_text_files();
        self.synced_workspace_paths = out.keys().cloned().collect();
        out
    }

    /// Formal R4.MvmLinuxContainer delete a guest `/workspace` file or directory prefix safely (`R4-63`).
    pub fn delete_workspace_file(&mut self, rel_path: &str) -> bool {
        let target = rel_path
            .trim()
            .trim_start_matches('/')
            .trim_start_matches("./");
        if target.is_empty() || target.contains("..") {
            return false;
        }
        let prefix = format!("/workspace/{target}");
        if target.ends_with('/') {
            let matches: Vec<String> = self
                .inodes
                .keys()
                .filter(|k| k.starts_with(&prefix) && !Self::is_protected_canonical_guest_path(k))
                .cloned()
                .collect();
            if matches.is_empty() {
                return false;
            }
            for k in matches {
                if let Some(rel) = k.strip_prefix("/workspace/") {
                    self.synced_workspace_paths.remove(rel);
                    self.workspace_diff_baseline.remove(rel);
                }
                self.remove_inode_reclaiming(&k);
            }
            return true;
        }
        if Self::is_protected_canonical_guest_path(&prefix) {
            return false;
        }
        self.synced_workspace_paths.remove(target);
        self.workspace_diff_baseline.remove(target);
        self.remove_inode_reclaiming(&prefix)
    }

    /// Formal R4.MvmLinuxContainer rename a guest `/workspace` file or symlink preserving ELF/binary mode (`R4-63`).
    pub fn rename_workspace_file(
        &mut self,
        from_rel: &str,
        to_rel: &str,
    ) -> Result<bool, MvmError> {
        let from_clean = from_rel
            .trim()
            .trim_start_matches('/')
            .trim_start_matches("./");
        let to_clean = to_rel
            .trim()
            .trim_start_matches('/')
            .trim_start_matches("./");
        if from_clean.is_empty()
            || to_clean.is_empty()
            || from_clean.contains("..")
            || to_clean.contains("..")
        {
            return Err(MvmError::CpuTrap(
                "RefusedParentTraversal: invalid workspace rename path".to_string(),
            ));
        }
        let from_guest = Self::normalize_guest_path("/workspace", from_clean);
        let to_guest = Self::normalize_guest_path("/workspace", to_clean);
        if Self::is_protected_guest_path(&from_guest) || Self::is_protected_guest_path(&to_guest)
        {
            return Err(MvmError::CanonicalPathProtected(from_guest));
        }
        let Some(inode) = self.inodes.get(&from_guest).cloned() else {
            return Ok(false);
        };
        if from_guest == to_guest {
            return Ok(true);
        }
        match inode.kind {
            InodeKind::Symlink { target } => {
                self.write_symlink_inode(&to_guest, &target)
                    .map_err(MvmError::CanonicalPathProtected)?;
                if let Some(ino) = self.inodes.get_mut(&to_guest) {
                    ino.mode = inode.mode;
                }
            }
            InodeKind::Regular { .. } => {
                let bytes = self.read_guest_file(&from_guest).unwrap_or_default();
                self.write_inode_with_mode(&to_guest, &bytes, inode.mode)?;
            }
        }
        self.remove_inode_reclaiming(&from_guest);
        self.synced_workspace_paths.remove(from_clean);
        self.synced_workspace_paths.insert(to_clean.to_string());
        Ok(true)
    }

    /// Formal R4.MvmLinuxContainer snapshot SHA-256 digests of current `/workspace` text files (`R4-63`).
    pub fn record_workspace_baseline_current(&mut self) {
        let current = self.snapshot_workspace_text_files();
        self.record_workspace_baseline_from_map(&current);
    }

    fn is_protected_canonical_guest_path(norm_path: &str) -> bool {
        norm_path
            .strip_prefix("/workspace/")
            .is_some_and(is_canonical_protected_rel)
    }

    /// Unified check enforcing protected canonical paths and read-only system directories/binaries.
    pub(super) fn is_protected_guest_path(norm_path: &str) -> bool {
        norm_path == "/"
            || norm_path == "/bin"
            || norm_path.starts_with("/bin/")
            || norm_path == "/sbin"
            || norm_path.starts_with("/sbin/")
            || norm_path == "/usr"
            || norm_path == "/usr/bin"
            || norm_path.starts_with("/usr/bin/")
            || norm_path == "/usr/include"
            || norm_path.starts_with("/usr/include/")
            || norm_path == "/usr/src"
            || norm_path.starts_with("/usr/src/")
            || norm_path == "/usr/local"
            || norm_path == "/etc"
            || norm_path.starts_with("/etc/")
            || norm_path == "/workspace"
            || Self::is_protected_canonical_guest_path(norm_path)
    }

    /// Formal R4.MvmLinuxContainer.
    fn remove_guest_path(&mut self, norm_path: &str, recursive: bool) -> Result<usize, String> {
        if Self::is_protected_guest_path(norm_path) {
            return Err(format!("rm: refusing to remove protected path {norm_path}"));
        }
        let prefix = format!("{norm_path}/");
        if recursive
            && self
                .inodes
                .keys()
                .any(|k| k.starts_with(&prefix) && Self::is_protected_guest_path(k))
        {
            return Err(format!(
                "rm: refusing to remove '{norm_path}' containing protected files"
            ));
        }
        let mut removed = 0usize;
        if self.remove_inode_reclaiming(norm_path) {
            removed += 1;
        }
        if recursive {
            let child_files: Vec<String> = self
                .inodes
                .keys()
                .filter(|k| k.starts_with(&prefix) && !Self::is_protected_guest_path(k))
                .cloned()
                .collect();
            for k in child_files {
                if self.remove_inode_reclaiming(&k) {
                    removed += 1;
                }
            }
            let child_dirs: Vec<String> = self
                .dirs
                .iter()
                .filter(|d| *d == norm_path || d.starts_with(&prefix))
                .cloned()
                .collect();
            for d in child_dirs {
                let d_prefix = format!("{d}/");
                let has_protected_below = self
                    .inodes
                    .keys()
                    .any(|k| k.starts_with(&d_prefix) && Self::is_protected_guest_path(k));
                if !Self::is_protected_guest_path(&d) && !has_protected_below {
                    self.dirs.remove(&d);
                    removed += 1;
                }
            }
        } else if removed == 0 && self.dirs.contains(norm_path) {
            return Err(format!("rm: cannot remove '{norm_path}': Is a directory"));
        }
        Ok(removed)
    }

    /// Formal R4.MvmLinuxContainer.
    /// Register a host-only credential placeholder (`{{MVM_SECRET:<name>}}`) restricted to `allowlisted_hosts`.
    /// Passing an empty `raw_secret` removes any previously registered credential entry so cleared secrets
    /// neither linger in `host_secrets` nor trigger `ensure_guest_ram_uncontaminated` on empty strings.
    pub fn register_host_secret(
        &mut self,
        name: &str,
        raw_secret: &str,
        allowlisted_hosts: &[&str],
    ) {
        if raw_secret.trim().is_empty() {
            self.host_secrets.remove(name);
            return;
        }
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

    /// Formal R4.MvmLinuxContainer.
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

    /// Formal R4.MvmLinuxContainer.
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

    /// Formal R4.MvmLinuxContainer.
    /// Pop the next queued `SpiralCore` packet on `vsock_port`.
    pub fn pop_vsock_inbox(&mut self, vsock_port: u16) -> Option<VsockPacket> {
        self.vsock_inbox.get_mut(&vsock_port)?.pop_front()
    }

    /// Execute an interactive command or `&&`/`||`/`;` chain, persisting `cwd` across `cd`
    /// and suspending cleanly into `MvmInteractiveOutcome::NeedsBridge` when an outbound
    /// `SpiralCore` (`CID 3 -> CID 2`) `nslookup`/`dig`, `curl`/`wget`, `pkg install`, or
    /// `git clone`/`pull`/`push` operation is invoked (`R4-39`).
    pub fn exec_interactive(
        &mut self,
        cmd_line: &str,
        explicit_cwd: Option<&str>,
    ) -> Result<MvmInteractiveOutcome, MvmError> {
        let mut working_cwd = match explicit_cwd {
            Some(c) if !c.trim().is_empty() => Self::normalize_guest_path(&self.current_cwd, c),
            _ => self.current_cwd.clone(),
        };
        self.env_vars.insert("PWD".to_string(), working_cwd.clone());
        let segments = parse_shell_segments(cmd_line);
        self.run_interactive_segments(
            &segments,
            0,
            &mut working_cwd,
            String::new(),
            String::new(),
            self.last_exit_code,
            0,
            Vec::new(),
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn run_interactive_segments(
        &mut self,
        segments: &[ShellCommandSegment],
        start_idx: usize,
        working_cwd: &mut String,
        mut total_stdout: String,
        mut total_stderr: String,
        mut last_exit: i32,
        mut total_insts: u64,
        mut all_frames: Vec<VsockPacket>,
        mut compiled_machine: Option<u16>,
    ) -> Result<MvmInteractiveOutcome, MvmError> {
        for idx in start_idx..segments.len() {
            let seg = &segments[idx];
            if seg.op == ChainOp::And && last_exit != 0 {
                continue;
            }
            if seg.op == ChainOp::Or && last_exit == 0 {
                continue;
            }
            match self.exec_interactive_segment(seg, working_cwd)? {
                MvmInteractiveOutcome::Completed(single) => {
                    total_stdout.push_str(&single.stdout);
                    total_stderr.push_str(&single.stderr);
                    last_exit = single.exit_code;
                    self.last_exit_code = last_exit;
                    total_insts = total_insts.saturating_add(single.instructions_retired);
                    all_frames.extend(single.vsock_frames);
                    if single.compiled_elf_machine.is_some() {
                        compiled_machine = single.compiled_elf_machine;
                    }
                }
                MvmInteractiveOutcome::NeedsBridge(mut pending) => {
                    self.current_cwd = working_cwd.clone();
                    self.env_vars.insert("PWD".to_string(), working_cwd.clone());
                    let remaining: Vec<serde_json::Value> = segments[idx + 1..]
                        .iter()
                        .map(|cseg| {
                            let op_str = match cseg.op {
                                ChainOp::First => "first",
                                ChainOp::Semi => "semi",
                                ChainOp::And => "and",
                                ChainOp::Or => "or",
                            };
                            serde_json::json!({ "op": op_str, "seg": cseg.raw })
                        })
                        .collect();
                    if let Some(obj) = pending.resume_state.as_object_mut() {
                        obj.insert(
                            "working_cwd".to_string(),
                            serde_json::Value::String(working_cwd.clone()),
                        );
                        obj.insert(
                            "stdout_prefix".to_string(),
                            serde_json::Value::String(total_stdout),
                        );
                        obj.insert(
                            "stderr_prefix".to_string(),
                            serde_json::Value::String(total_stderr),
                        );
                        obj.insert(
                            "instructions_prefix".to_string(),
                            serde_json::Value::Number(total_insts.into()),
                        );
                        obj.insert(
                            "remaining_segments".to_string(),
                            serde_json::Value::Array(remaining),
                        );
                    }
                    all_frames.extend(pending.vsock_frames.clone());
                    pending.vsock_frames = all_frames;
                    return Ok(MvmInteractiveOutcome::NeedsBridge(pending));
                }
            }
        }
        self.current_cwd = working_cwd.clone();
        self.env_vars.insert("PWD".to_string(), working_cwd.clone());
        self.last_exit_code = last_exit;
        Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
            exit_code: last_exit,
            stdout: total_stdout,
            stderr: total_stderr,
            instructions_retired: total_insts,
            vsock_frames: all_frames,
            compiled_elf_machine: compiled_machine,
            cow_state_sha256: self.cow_state_sha256(),
        }))
    }

    /// Resume a suspended `NeedsBridge` command once the host Worker delivers `bridge_resp` (`R4-39`).
    pub fn complete_bridge(
        &mut self,
        resume_state: &serde_json::Value,
        bridge_resp: &serde_json::Value,
    ) -> Result<MvmInteractiveOutcome, MvmError> {
        self.ensure_guest_ram_uncontaminated()?;

        let bridge_kind = resume_state
            .get("bridge_kind")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                MvmError::BridgeRejected(BridgeError::InvalidBridgeJson(
                    "missing resume_state.bridge_kind".to_string(),
                ))
            })?;
        let mut working_cwd = resume_state
            .get("working_cwd")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| self.current_cwd.clone());
        let mut total_stdout = resume_state
            .get("stdout_prefix")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let mut total_stderr = resume_state
            .get("stderr_prefix")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let total_insts = resume_state
            .get("instructions_prefix")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let redirect = resume_state
            .get("redirect_path")
            .and_then(|v| v.as_str())
            .map(|p| {
                let append = resume_state
                    .get("redirect_append")
                    .and_then(|a| a.as_bool())
                    .unwrap_or(false);
                (p.to_string(), append)
            });

        if bridge_resp.get("ok") == Some(&serde_json::Value::Bool(false)) {
            let code = bridge_resp
                .get("error_code")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let err_detail = bridge_resp
                .get("error_message")
                .or_else(|| bridge_resp.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("SpiralCore bridge error")
                .to_string();
            match code {
                "PolicyDenied" => return Err(MvmError::BridgeRejected(BridgeError::PolicyDenied)),
                "InsecureTransportRejected" => {
                    return Err(MvmError::BridgeRejected(
                        BridgeError::InsecureTransportRejected,
                    ))
                }
                "InvalidVsockCid" => {
                    return Err(MvmError::BridgeRejected(BridgeError::InvalidVsockCid))
                }
                "GuestRamSecretContamination" => {
                    return Err(MvmError::BridgeRejected(
                        BridgeError::GuestRamSecretContamination,
                    ))
                }
                "HostNotAllowlistedForSecret" => {
                    return Err(MvmError::BridgeRejected(
                        BridgeError::HostNotAllowlistedForSecret,
                    ))
                }
                "MissingSecretPlaceholder" => {
                    return Err(MvmError::BridgeRejected(
                        BridgeError::MissingSecretPlaceholder,
                    ))
                }
                "PackageDigestMismatch" | "DigestMismatch" => {
                    return Err(MvmError::BridgeRejected(BridgeError::PackageDigestMismatch))
                }
                "PackageInvalidElf64" | "InvalidElf64Riscv" => {
                    return Err(MvmError::BridgeRejected(BridgeError::PackageInvalidElf64))
                }
                "PackageSizeExceedsQuota" | "SizeExceedsQuota" => {
                    return Err(MvmError::BridgeRejected(
                        BridgeError::PackageSizeExceedsQuota,
                    ))
                }
                "IntegrityMismatch" => {
                    return Err(MvmError::BridgeRejected(BridgeError::IntegrityMismatch(
                        err_detail,
                    )))
                }
                "EsbuildAdapterUnavailable" => {
                    return Err(MvmError::BridgeRejected(
                        BridgeError::EsbuildAdapterUnavailable(err_detail),
                    ))
                }
                "WasiRuntimeError" => {
                    return Err(MvmError::BridgeRejected(BridgeError::WasiRuntimeError(
                        err_detail,
                    )))
                }
                "SqliteRuntimeError" | "CorruptSqliteDatabase" => {
                    return Err(MvmError::BridgeRejected(BridgeError::SqliteRuntimeError(
                        err_detail,
                    )))
                }
                "CapabilityDenied" => {
                    return Err(MvmError::BridgeRejected(BridgeError::CapabilityDenied(
                        bridge_resp
                            .get("host")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&err_detail)
                            .to_string(),
                    )))
                }
                "RelayConsentRequired" => {
                    return Err(MvmError::BridgeRejected(BridgeError::RelayConsentRequired))
                }
                "RelayForbiddenForCredentialedHost" => {
                    return Err(MvmError::BridgeRejected(
                        BridgeError::RelayForbiddenForCredentialedHost,
                    ))
                }
                "PrivateAddressBlocked" => {
                    return Err(MvmError::BridgeRejected(
                        BridgeError::PrivateAddressBlocked(
                            bridge_resp
                                .get("host")
                                .or_else(|| bridge_resp.get("target"))
                                .and_then(|v| v.as_str())
                                .unwrap_or(&err_detail)
                                .to_string(),
                        ),
                    ))
                }
                "DoHResolutionFailed" => {
                    return Err(MvmError::BridgeRejected(BridgeError::DoHResolutionFailed(
                        err_detail,
                    )))
                }
                "RelayUnavailable" => {
                    return Err(MvmError::BridgeRejected(BridgeError::RelayUnavailable(
                        err_detail,
                    )))
                }
                "CanonicalPathProtected" => {
                    return Err(MvmError::CanonicalPathProtected(err_detail))
                }
                _ => {
                    total_stderr.push_str(&err_detail);
                    if !total_stderr.ends_with('\n') {
                        total_stderr.push('\n');
                    }
                    self.current_cwd = working_cwd;
                    return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                        exit_code: 1,
                        stdout: total_stdout,
                        stderr: total_stderr,
                        instructions_retired: total_insts,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    }));
                }
            }
        }

        let data = bridge_resp
            .get("data")
            .filter(|d| d.is_object())
            .unwrap_or(bridge_resp);

        let (seg_exit, seg_stdout, seg_stderr) = match bridge_kind {
            "dns_query" => {
                let qname = resume_state
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let mut out = format!("Server:\t\t1.1.1.1 (SpiralCore DoH)\nName:\t{qname}\n");
                if let Some(answers) = data.get("answers").and_then(|v| v.as_array()) {
                    for ans in answers {
                        let rdata = ans
                            .get("data")
                            .and_then(|v| v.as_str())
                            .unwrap_or("0.0.0.0");
                        let rtype = ans.get("type_name").and_then(|v| v.as_str()).unwrap_or("A");
                        out.push_str(&format!("Address ({rtype}): {rdata}\n"));
                    }
                } else if let Some(ip) = data.get("address").and_then(|v| v.as_str()) {
                    out.push_str(&format!("Address: {ip}\n"));
                }
                (0, out, String::new())
            }
            "http_request" => {
                let raw_bytes = extract_bridge_payload_bytes(data, bridge_resp);
                if let Some(out_file) = resume_state.get("output_file").and_then(|v| v.as_str()) {
                    let norm_out = Self::normalize_guest_path(&working_cwd, out_file);
                    self.write_guest_file(&norm_out, &raw_bytes)?;
                    (
                        0,
                        format!("Saved {} bytes to {norm_out}\n", raw_bytes.len()),
                        String::new(),
                    )
                } else {
                    let mut body_str = String::from_utf8_lossy(&raw_bytes).into_owned();
                    if !body_str.is_empty() && !body_str.ends_with('\n') {
                        body_str.push('\n');
                    }
                    (0, body_str, String::new())
                }
            }
            "pkg_fetch"
                if resume_state.get("ecosystem").and_then(|v| v.as_str()) == Some("npm") =>
            {
                self.complete_polyglot_bridge_outcome(data, bridge_resp)?
            }
            "node_exec" | "python_exec" | "wasi_exec" | "sqlite_exec" => {
                self.complete_polyglot_bridge_outcome(data, bridge_resp)?
            }
            "pkg_fetch" => {
                let pkg_name = resume_state
                    .get("pkg_name")
                    .and_then(|v| v.as_str())
                    .or_else(|| data.get("pkg_name").and_then(|v| v.as_str()))
                    .or_else(|| data.get("name").and_then(|v| v.as_str()))
                    .or_else(|| bridge_resp.get("name").and_then(|v| v.as_str()))
                    .unwrap_or("pkg-bin");
                let expected_sha = resume_state
                    .get("expected_sha256")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .or_else(|| data.get("expected_sha256").and_then(|v| v.as_str()))
                    .or_else(|| bridge_resp.get("expected_sha256").and_then(|v| v.as_str()))
                    .or_else(|| data.get("sha256").and_then(|v| v.as_str()))
                    .or_else(|| bridge_resp.get("sha256").and_then(|v| v.as_str()))
                    .unwrap_or("");
                let elf_bytes = extract_bridge_elf_bytes(data, bridge_resp)?;
                let actual_sha = sha256_hex_bytes(&elf_bytes);
                let sha_matched =
                    !expected_sha.is_empty() && actual_sha.eq_ignore_ascii_case(expected_sha);
                let elf_magic_ok = elf_bytes.len() >= 64 && &elf_bytes[0..4] == b"\x7fELF";
                let elf_class_64 = elf_bytes.len() >= 64 && elf_bytes[4] == 2 && elf_bytes[5] == 1;
                let machine_code = if elf_bytes.len() >= 20 {
                    u16::from_le_bytes([elf_bytes[18], elf_bytes[19]])
                } else {
                    0
                };
                let byte_len = elf_bytes.len().min(u32::MAX as usize) as u32;
                let verdict = admit_package_install(
                    sha_matched,
                    elf_magic_ok,
                    elf_class_64,
                    machine_code,
                    byte_len,
                    4 * 1024 * 1024,
                );
                match verdict {
                    PackageInstallVerdict::DigestMismatch => {
                        return Err(MvmError::BridgeRejected(BridgeError::PackageDigestMismatch))
                    }
                    PackageInstallVerdict::InvalidElf64Riscv => {
                        return Err(MvmError::BridgeRejected(BridgeError::PackageInvalidElf64))
                    }
                    PackageInstallVerdict::SizeExceedsQuota => {
                        return Err(MvmError::BridgeRejected(
                            BridgeError::PackageSizeExceedsQuota,
                        ))
                    }
                    PackageInstallVerdict::PackageAdmitted => {}
                }
                if admit_elf64_riscv(&elf_bytes) != ElfAdmission::ElfAdmitted {
                    return Err(MvmError::BridgeRejected(BridgeError::PackageInvalidElf64));
                }
                let bin_path = format!("/usr/local/bin/{pkg_name}");
                self.write_inode_with_mode(&bin_path, &elf_bytes, 0o100755)?;
                let source = resume_state
                    .get("source_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("spiralcore-pkg")
                    .to_string();
                self.installed_packages.insert(
                    pkg_name.to_string(),
                    InstalledPackageMeta {
                        name: pkg_name.to_string(),
                        sha256: actual_sha.clone(),
                        byte_len,
                        source,
                    },
                );
                if let Ok(cat_bytes) = serde_json::to_vec(&self.installed_packages) {
                    let _ = self.write_inode_with_mode("/etc/pkg/installed.json", &cat_bytes, 0o100644);
                }
                (
                    0,
                    format!(
                        "Installed {pkg_name} ({byte_len} bytes, sha256:{}) -> {bin_path}\n",
                        &actual_sha[..12]
                    ),
                    String::new(),
                )
            }
            "git_smart_http" => {
                let git_action = resume_state
                    .get("git_action")
                    .and_then(|v| v.as_str())
                    .unwrap_or("clone");
                let repo_url = resume_state
                    .get("repo_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let target_dir = resume_state
                    .get("target_dir")
                    .and_then(|v| v.as_str())
                    .unwrap_or("/workspace");
                self.complete_git_bridge(git_action, repo_url, target_dir, data, bridge_resp)?
            }
            other => {
                return Err(MvmError::BridgeRejected(BridgeError::InvalidBridgeJson(
                    format!("unsupported bridge_kind `{other}`"),
                )))
            }
        };

        self.ensure_guest_ram_uncontaminated()?;

        let mut stage_out_bytes = seg_stdout.into_bytes();
        let mut stage_err_str = seg_stderr;
        let mut stage_exit = seg_exit;
        if resume_state
            .get("stderr_to_stdout")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            stage_out_bytes.extend_from_slice(stage_err_str.as_bytes());
            stage_err_str.clear();
        }
        if resume_state
            .get("stdout_to_stderr")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            stage_err_str.push_str(&String::from_utf8_lossy(&stage_out_bytes));
            stage_out_bytes.clear();
        }
        if let Some(err_path) = resume_state
            .get("stderr_redirect_path")
            .and_then(|v| v.as_str())
        {
            let err_append = resume_state
                .get("stderr_redirect_append")
                .and_then(|a| a.as_bool())
                .unwrap_or(false);
            let err_bytes = stage_err_str.as_bytes().to_vec();
            stage_err_str.clear();
            if let Err(msg) =
                self.write_redirected_bytes(&working_cwd, err_path, &err_bytes, err_append, None)
            {
                stage_err_str.push_str(&msg);
                stage_err_str.push('\n');
                stage_exit = 1;
            }
        }
        if let Some((redir_path, append)) = redirect {
            self.write_redirected_bytes(&working_cwd, &redir_path, &stage_out_bytes, append, None)
                .map_err(MvmError::CpuTrap)?;
            stage_out_bytes.clear();
        }
        if let Some(pipe_err) = resume_state
            .get("pipeline_stderr_prefix")
            .and_then(|v| v.as_str())
        {
            total_stderr.push_str(pipe_err);
        }
        total_stderr.push_str(&stage_err_str);

        let mut remaining_segments: Vec<ShellCommandSegment> = Vec::new();
        if let Some(arr) = resume_state
            .get("remaining_segments")
            .and_then(|v| v.as_array())
        {
            for item in arr {
                let op_s = item.get("op").and_then(|v| v.as_str()).unwrap_or("semi");
                let seg_s = item.get("seg").and_then(|v| v.as_str()).unwrap_or("");
                let cop = match op_s {
                    "and" => ChainOp::And,
                    "or" => ChainOp::Or,
                    "first" => ChainOp::First,
                    _ => ChainOp::Semi,
                };
                for mut parsed_seg in parse_shell_segments(seg_s) {
                    parsed_seg.op = cop;
                    remaining_segments.push(parsed_seg);
                }
            }
        }

        let rem_stages: Vec<String> = resume_state
            .get("remaining_pipeline_stages")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let pipeline_insts_prefix = resume_state
            .get("pipeline_insts_prefix")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        if !rem_stages.is_empty() {
            match self.exec_pipeline_stages_with_mode(
                &rem_stages,
                &[],
                &mut working_cwd,
                stage_out_bytes,
                String::new(),
                stage_exit,
                pipeline_insts_prefix,
                Vec::new(),
                None,
                true,
            )? {
                MvmInteractiveOutcome::Completed(pipe_done) => {
                    total_stdout.push_str(&pipe_done.stdout);
                    total_stderr.push_str(&pipe_done.stderr);
                    let next_insts = total_insts.saturating_add(pipe_done.instructions_retired);
                    return self.run_interactive_segments(
                        &remaining_segments,
                        0,
                        &mut working_cwd,
                        total_stdout,
                        total_stderr,
                        pipe_done.exit_code,
                        next_insts,
                        pipe_done.vsock_frames,
                        pipe_done.compiled_elf_machine,
                    );
                }
                MvmInteractiveOutcome::NeedsBridge(mut next_pending) => {
                    if let Some(obj) = next_pending.resume_state.as_object_mut() {
                        obj.insert(
                            "working_cwd".to_string(),
                            serde_json::Value::String(working_cwd),
                        );
                        obj.insert(
                            "stdout_prefix".to_string(),
                            serde_json::Value::String(total_stdout),
                        );
                        obj.insert(
                            "stderr_prefix".to_string(),
                            serde_json::Value::String(total_stderr),
                        );
                        obj.insert(
                            "instructions_prefix".to_string(),
                            serde_json::Value::Number(total_insts.into()),
                        );
                        if let Some(rem) = resume_state.get("remaining_segments").cloned() {
                            obj.insert("remaining_segments".to_string(), rem);
                        }
                    }
                    return Ok(MvmInteractiveOutcome::NeedsBridge(next_pending));
                }
            }
        }

        total_stdout.push_str(&String::from_utf8_lossy(&stage_out_bytes));
        self.run_interactive_segments(
            &remaining_segments,
            0,
            &mut working_cwd,
            total_stdout,
            total_stderr,
            stage_exit,
            total_insts.saturating_add(pipeline_insts_prefix),
            Vec::new(),
            None,
        )
    }

    /// Execute a command or `&&`/`||`/`;` chain inside the `MVM-Wasm` RV64IMAC Linux MicroVM.
    pub fn exec_command(&mut self, cmd_line: &str, cwd: &str) -> Result<MvmExecOutcome, MvmError> {
        let mut working_cwd = Self::normalize_guest_path("/workspace", cwd);
        self.env_vars.insert("PWD".to_string(), working_cwd.clone());
        let segments = parse_shell_segments(cmd_line);
        let mut total_stdout = String::new();
        let mut total_stderr = String::new();
        let mut last_exit = self.last_exit_code;
        let mut total_insts = 0u64;
        let mut all_frames = Vec::new();
        let mut compiled_machine = None;

        for seg in &segments {
            if seg.op == ChainOp::And && last_exit != 0 {
                continue;
            }
            if seg.op == ChainOp::Or && last_exit == 0 {
                continue;
            }
            let single = self.exec_single_segment_ast(seg, &mut working_cwd)?;
            total_stdout.push_str(&single.stdout);
            total_stderr.push_str(&single.stderr);
            last_exit = single.exit_code;
            self.last_exit_code = last_exit;
            total_insts = total_insts.saturating_add(single.instructions_retired);
            all_frames.extend(single.vsock_frames);
            if single.compiled_elf_machine.is_some() {
                compiled_machine = single.compiled_elf_machine;
            }
        }

        self.current_cwd = working_cwd.clone();
        self.env_vars.insert("PWD".to_string(), working_cwd);

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

    fn exec_interactive_segment(
        &mut self,
        seg: &ShellCommandSegment,
        working_cwd: &mut String,
    ) -> Result<MvmInteractiveOutcome, MvmError> {
        let trimmed = seg.raw.trim();
        if trimmed.is_empty() {
            return Ok(MvmInteractiveOutcome::Completed(self.empty_outcome()));
        }
        let stages = &seg.stages;
        if stages.is_empty() {
            return Ok(MvmInteractiveOutcome::Completed(self.empty_outcome()));
        }
        if seg.heredocs.is_empty()
            && stages.len() == 2
            && !trimmed.contains('$')
            && !trimmed.contains('>')
            && !trimmed.contains('<')
            && !trimmed.contains('=')
            && self.inodes.contains_key("/bin/sh")
        {
            let p0 = parse_stage_words_and_redirects(
                &stages[0],
                &self.env_vars,
                self.last_exit_code,
                self.next_pid,
            );
            let p1 = parse_stage_words_and_redirects(
                &stages[1],
                &self.env_vars,
                self.last_exit_code,
                self.next_pid,
            );
            let is_mvmbox_simple = |words: &[String]| -> bool {
                let Some(prog) = words.first().map(String::as_str) else {
                    return false;
                };
                if self
                    .resolve_guest_exec_path(working_cwd, prog)
                    .is_some_and(|p| p.starts_with("/usr/local/bin/"))
                {
                    return false;
                }
                matches!(
                    prog,
                    "echo" | "sha256sum" | "uname" | "pwd" | "id" | "whoami" | "true" | "false"
                )
            };
            if is_mvmbox_simple(&p0.words) && is_mvmbox_simple(&p1.words) {
                let outcome = self.exec_single_segment_ast(seg, working_cwd)?;
                return Ok(MvmInteractiveOutcome::Completed(outcome));
            }
        }
        let out = self.exec_pipeline_stages_with_mode(
            stages,
            &seg.heredocs,
            working_cwd,
            Vec::new(),
            String::new(),
            0,
            0,
            Vec::new(),
            None,
            true,
        )?;
        if let MvmInteractiveOutcome::Completed(ref done) = out {
            self.last_exit_code = done.exit_code;
        }
        Ok(out)
    }

    fn prepare_polyglot_bridge(
        &mut self,
        head: &str,
        words: &[String],
        working_cwd: &str,
        stdin: &[u8],
        eff_env: &BTreeMap<String, String>,
        redirects: &StageRedirects,
    ) -> Result<MvmBridgePending, MvmError> {
        self.ensure_guest_ram_uncontaminated()?;

        let (op, kind, bridge_kind, ecosystem): (&str, &str, &str, Option<&str>) = match head {
            "node" | "npx" => ("node_exec", "node_exec", "node_exec", None),
            "npm" => ("pkg_fetch", "pkg_fetch", "pkg_fetch", Some("npm")),
            "python" | "python3" | "pip" | "pip3" => {
                ("python_exec", "python_exec", "python_exec", None)
            }
            "wasm" => ("wasi_exec", "wasi_exec", "wasi_exec", None),
            "sqlite3" => ("sqlite_exec", "sqlite_exec", "sqlite_exec", None),
            _ => {
                return Err(MvmError::CpuTrap(format!(
                    "unsupported polyglot tool `{head}`"
                )))
            }
        };

        let req_id = self.next_bridge_req_id(&format!("mvm-{head}"));
        let args: Vec<String> = words.get(1..).unwrap_or(&[]).to_vec();
        let mut env_map = serde_json::Map::new();
        for (k, v) in eff_env {
            env_map.insert(k.clone(), serde_json::Value::String(v.clone()));
        }
        let workspace_entries = self.export_polyglot_workspace_entries();
        let stdin_utf8 = String::from_utf8_lossy(stdin).into_owned();
        let stdin_base64 = encode_base64_bytes(stdin);

        let bridge_req = serde_json::json!({
            "req_id": req_id,
            "spvs_magic": "SPVS",
            "src_cid": self.guest_cid,
            "dst_cid": 2,
            "policy": self.policy_str(),
            "op": op,
            "kind": kind,
            "tool": head,
            "ecosystem": ecosystem,
            "args": args,
            "cwd": working_cwd,
            "env": env_map,
            "stdin_utf8": stdin_utf8,
            "stdin_base64": stdin_base64,
            "workspace": workspace_entries,
        });
        let frame = self.emit_bridge_vsock_frame(bridge_req.to_string().as_bytes())?;
        let resume_state = serde_json::json!({
            "bridge_kind": bridge_kind,
            "ecosystem": ecosystem,
            "tool": head,
            "working_cwd": working_cwd,
            "redirect_path": redirects.stdout_file.as_ref().map(|(p, _)| p.clone()),
            "redirect_append": redirects.stdout_file.as_ref().map(|(_, a)| *a).unwrap_or(false),
            "stderr_redirect_path": redirects.stderr_file.as_ref().map(|(p, _)| p.clone()),
            "stderr_redirect_append": redirects.stderr_file.as_ref().map(|(_, a)| *a).unwrap_or(false),
            "stderr_to_stdout": redirects.stderr_to_stdout,
            "stdout_to_stderr": redirects.stdout_to_stderr,
        });
        Ok(MvmBridgePending {
            bridge_req,
            resume_state,
            vsock_frames: vec![frame],
        })
    }

    fn export_polyglot_workspace_entries(&self) -> Vec<serde_json::Value> {
        let mut entries = Vec::new();
        for (path, ino) in &self.inodes {
            let Some(rel) = path.strip_prefix("/workspace/") else {
                continue;
            };
            if rel.is_empty() || is_git_internal_rel_path(rel) {
                continue;
            }
            let readonly = is_canonical_protected_rel(rel);
            match &ino.kind {
                InodeKind::Regular { .. } => {
                    let bytes = self.read_guest_file(path).unwrap_or_default();
                    entries.push(serde_json::json!({
                        "path": rel,
                        "mode": ino.mode,
                        "readonly": readonly,
                        "content_base64": encode_base64_bytes(&bytes),
                    }));
                }
                InodeKind::Symlink { target } => {
                    let resolved_bytes = self.read_guest_file(path).unwrap_or_default();
                    entries.push(serde_json::json!({
                        "path": rel,
                        "mode": ino.mode,
                        "readonly": readonly,
                        "symlink_target": target,
                        "content_base64": encode_base64_bytes(&resolved_bytes),
                    }));
                }
            }
        }
        entries
    }

    fn apply_polyglot_workspace_mutations(
        &mut self,
        data: &serde_json::Value,
        bridge_resp: &serde_json::Value,
    ) -> Result<(), MvmError> {
        enum StagedPolyglotWrite {
            Symlink(String),
            Regular { bytes: Vec<u8>, mode: u32 },
        }

        let writes = data
            .get("writes")
            .or_else(|| bridge_resp.get("writes"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let deletes = data
            .get("deletes")
            .or_else(|| bridge_resp.get("deletes"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut staged_writes: Vec<(String, StagedPolyglotWrite)> =
            Vec::with_capacity(writes.len());
        let mut staged_write_map: BTreeMap<String, usize> = BTreeMap::new();
        let mut staged_deletes: Vec<String> = Vec::with_capacity(deletes.len());
        let mut delete_set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

        for w in writes {
            let Some(raw_path) = w.get("path").and_then(|v| v.as_str()) else {
                continue;
            };
            let norm = if raw_path.starts_with('/') {
                Self::normalize_guest_path("/", raw_path)
            } else {
                Self::normalize_guest_path("/workspace", raw_path)
            };
            if !norm.starts_with("/workspace/") && !norm.starts_with("/tmp/") {
                return Err(MvmError::CanonicalPathProtected(norm));
            }
            if Self::is_protected_canonical_guest_path(&norm) {
                return Err(MvmError::CanonicalPathProtected(norm));
            }
            if let Some(sym_target) = w.get("symlink_target").and_then(|v| v.as_str()) {
                let idx = staged_writes.len();
                staged_write_map.insert(norm.clone(), idx);
                staged_writes.push((norm, StagedPolyglotWrite::Symlink(sym_target.to_string())));
            } else {
                let bytes = if let Some(b64) = w.get("content_base64").and_then(|v| v.as_str()) {
                    decode_base64_bytes(b64).map_err(MvmError::BridgeRejected)?
                } else if let Some(txt) = w.get("content_utf8").and_then(|v| v.as_str()) {
                    txt.as_bytes().to_vec()
                } else {
                    Vec::new()
                };
                let mode = w
                    .get("mode")
                    .and_then(|v| v.as_u64())
                    .map(|m| m as u32)
                    .unwrap_or_else(|| {
                        if bytes.starts_with(b"\x7fELF") {
                            0o100755
                        } else {
                            0o100644
                        }
                    });
                let idx = staged_writes.len();
                staged_write_map.insert(norm.clone(), idx);
                staged_writes.push((norm, StagedPolyglotWrite::Regular { bytes, mode }));
            }
        }

        for d in deletes {
            let Some(raw_path) = d.as_str() else {
                continue;
            };
            let norm = if raw_path.starts_with('/') {
                Self::normalize_guest_path("/", raw_path)
            } else {
                Self::normalize_guest_path("/workspace", raw_path)
            };
            if !norm.starts_with("/workspace/") && !norm.starts_with("/tmp/") {
                return Err(MvmError::CanonicalPathProtected(norm));
            }
            if Self::is_protected_canonical_guest_path(&norm) {
                return Err(MvmError::CanonicalPathProtected(norm));
            }
            let prefix = format!("{norm}/");
            if self
                .inodes
                .keys()
                .any(|k| k.starts_with(&prefix) && Self::is_protected_canonical_guest_path(k))
            {
                return Err(MvmError::CanonicalPathProtected(norm));
            }
            delete_set.insert(norm.clone());
            staged_deletes.push(norm);
        }

        let virtual_symlink_target = |path: &str| -> Option<String> {
            if let Some(&idx) = staged_write_map.get(path) {
                return match &staged_writes[idx].1 {
                    StagedPolyglotWrite::Symlink(t) => Some(t.clone()),
                    StagedPolyglotWrite::Regular { .. } => None,
                };
            }
            if delete_set.contains(path) {
                return None;
            }
            match self.inodes.get(path) {
                Some(GuestInode {
                    kind: InodeKind::Symlink { target },
                    ..
                }) => Some(target.clone()),
                _ => None,
            }
        };

        let check_chain = |origin: &str| -> Result<(), MvmError> {
            let in_ws = origin.starts_with("/workspace/");
            let in_tmp = origin.starts_with("/tmp/");
            let within_scope = |p: &str| -> bool {
                if in_ws {
                    p == "/workspace" || p.starts_with("/workspace/")
                } else if in_tmp {
                    p == "/tmp" || p.starts_with("/tmp/")
                } else {
                    false
                }
            };

            // Check ancestor directory symlinks along `origin`
            let mut slash_pos = 1usize;
            while let Some(next_slash) = origin[slash_pos..].find('/') {
                let prefix_end = slash_pos + next_slash;
                let prefix = &origin[..prefix_end];
                let suffix = &origin[prefix_end..];
                let mut cur_prefix = prefix.to_string();
                for _ in 0..8 {
                    if let Some(target) = virtual_symlink_target(&cur_prefix) {
                        let resolved_prefix = Self::resolve_symlink_step(&cur_prefix, &target);
                        if !within_scope(&resolved_prefix) {
                            return Err(MvmError::CanonicalPathProtected(origin.to_string()));
                        }
                        let combined = format!("{resolved_prefix}{suffix}");
                        if Self::is_protected_canonical_guest_path(&combined) {
                            return Err(MvmError::CanonicalPathProtected(origin.to_string()));
                        }
                        cur_prefix = resolved_prefix;
                    } else {
                        break;
                    }
                }
                slash_pos = prefix_end + 1;
            }

            // Chase symlink hops at `origin` itself up to 8 hops
            let mut cur = origin.to_string();
            for hop in 0..8 {
                if Self::is_protected_canonical_guest_path(&cur) {
                    return Err(MvmError::CanonicalPathProtected(origin.to_string()));
                }
                if !within_scope(&cur) {
                    return Err(MvmError::CanonicalPathProtected(origin.to_string()));
                }
                match virtual_symlink_target(&cur) {
                    Some(target) => {
                        let next = Self::resolve_symlink_step(&cur, &target);
                        if !within_scope(&next) || Self::is_protected_canonical_guest_path(&next) {
                            return Err(MvmError::CanonicalPathProtected(origin.to_string()));
                        }
                        cur = next;
                        if hop == 7 {
                            return Err(MvmError::CanonicalPathProtected(origin.to_string()));
                        }
                    }
                    None => break,
                }
            }
            Ok(())
        };

        for (norm, mut_kind) in &staged_writes {
            check_chain(norm)?;
            if matches!(mut_kind, StagedPolyglotWrite::Regular { .. }) && !delete_set.contains(norm)
            {
                let mut existing_cur = norm.clone();
                for _ in 0..8 {
                    if Self::is_protected_canonical_guest_path(&existing_cur) {
                        return Err(MvmError::CanonicalPathProtected(norm.clone()));
                    }
                    match self.inodes.get(&existing_cur) {
                        Some(GuestInode {
                            kind: InodeKind::Symlink { target },
                            ..
                        }) => {
                            existing_cur = Self::resolve_symlink_step(&existing_cur, target);
                        }
                        _ => break,
                    }
                }
            }
        }

        // All validations succeeded: apply staged mutations atomically.
        for norm in staged_deletes {
            let _ = self
                .remove_guest_path(&norm, false)
                .map_err(MvmError::CanonicalPathProtected)?;
        }
        for (norm, mut_kind) in staged_writes {
            match mut_kind {
                StagedPolyglotWrite::Symlink(sym_target) => {
                    self.write_symlink_inode(&norm, &sym_target)
                        .map_err(MvmError::CanonicalPathProtected)?;
                }
                StagedPolyglotWrite::Regular { bytes, mode } => {
                    self.write_inode_with_mode(&norm, &bytes, mode)?;
                }
            }
        }
        Ok(())
    }

    fn complete_polyglot_bridge_outcome(
        &mut self,
        data: &serde_json::Value,
        bridge_resp: &serde_json::Value,
    ) -> Result<(i32, String, String), MvmError> {
        self.apply_polyglot_workspace_mutations(data, bridge_resp)?;
        let exit_code = data
            .get("exit_code")
            .or_else(|| bridge_resp.get("exit_code"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        let stdout = data
            .get("stdout")
            .or_else(|| data.get("body_text"))
            .or_else(|| bridge_resp.get("stdout"))
            .or_else(|| bridge_resp.get("body_text"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let stderr = data
            .get("stderr")
            .or_else(|| bridge_resp.get("stderr"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        Ok((exit_code, stdout, stderr))
    }

    #[allow(clippy::too_many_arguments)]
    fn exec_pipeline_stages_with_mode(
        &mut self,
        stages: &[String],
        heredocs: &[HeredocBody],
        working_cwd: &mut String,
        mut piped_stdin: Vec<u8>,
        mut total_stderr: String,
        mut last_exit: i32,
        mut total_insts: u64,
        mut all_frames: Vec<VsockPacket>,
        mut compiled_machine: Option<u16>,
        allow_bridge: bool,
    ) -> Result<MvmInteractiveOutcome, MvmError> {
        let unwrap_env_and_polyglot = |sandbox: &MvmSandbox,
                                       words: &[String],
                                       base_env: &BTreeMap<String, String>,
                                       cwd_str: &str|
         -> (Vec<String>, BTreeMap<String, String>) {
            let mut env_out = base_env.clone();
            let mut idx = 0usize;
            while idx < words.len() {
                let head = words[idx].as_str();
                if head == "env" || head == "/usr/bin/env" || head == "/bin/env" {
                    idx += 1;
                    while idx < words.len() {
                        let a = &words[idx];
                        if a == "-i" || a == "--ignore-environment" {
                            env_out.clear();
                            idx += 1;
                        } else if a == "-u" && idx + 1 < words.len() {
                            env_out.remove(&words[idx + 1]);
                            idx += 2;
                        } else if let Some((k, v)) = a.split_once('=') {
                            env_out.insert(k.to_string(), v.to_string());
                            idx += 1;
                        } else {
                            break;
                        }
                    }
                    continue;
                }
                break;
            }
            let mut eff_words: Vec<String> = words[idx..].to_vec();
            if let Some(first) = eff_words.first().cloned() {
                if let Some(stripped) = first
                    .strip_prefix("/usr/bin/")
                    .or_else(|| first.strip_prefix("/bin/"))
                    .or_else(|| first.strip_prefix("/usr/local/bin/"))
                {
                    let norm = Self::normalize_guest_path(cwd_str, &first);
                    let is_elf = sandbox
                        .read_guest_file(&norm)
                        .is_some_and(|b| b.starts_with(b"\x7fELF"));
                    if !is_elf && !stripped.is_empty() {
                        eff_words[0] = stripped.to_string();
                    }
                } else {
                    let cand_path = if first.starts_with('/')
                        || first.starts_with("./")
                        || first.starts_with("../")
                    {
                        Self::normalize_guest_path(cwd_str, &first)
                    } else {
                        format!("/workspace/node_modules/.bin/{first}")
                    };
                    if let Some(bytes) = sandbox.read_guest_file(&cand_path) {
                        if !bytes.starts_with(b"\x7fELF") {
                            let first_line = bytes.split(|&b| b == b'\n').next().unwrap_or(&[]);
                            let header = String::from_utf8_lossy(first_line);
                            if header.starts_with("#!") && header.contains("python") {
                                let mut next = vec!["python3".to_string(), cand_path];
                                next.extend(eff_words.into_iter().skip(1));
                                eff_words = next;
                            } else if (header.starts_with("#!") && header.contains("node"))
                                || cand_path.ends_with(".js")
                                || cand_path.ends_with(".mjs")
                                || cand_path.ends_with(".cjs")
                                || cand_path.ends_with(".ts")
                            {
                                let mut next = vec!["node".to_string(), cand_path];
                                next.extend(eff_words.into_iter().skip(1));
                                eff_words = next;
                            }
                        }
                    }
                }
            }
            (eff_words, env_out)
        };

        let expand_globs = |sandbox: &MvmSandbox,
                            raw_words: &[String],
                            quoted_flags: &[bool],
                            cwd_str: &str|
         -> Vec<String> {
            let cmd_head = raw_words.first().map(String::as_str).unwrap_or("");
            let allow_glob = matches!(
                cmd_head,
                "ls" | "cat"
                    | "head"
                    | "tail"
                    | "wc"
                    | "cp"
                    | "mv"
                    | "rm"
                    | "touch"
                    | "chmod"
                    | "tar"
                    | "gzip"
                    | "gunzip"
                    | "diff"
                    | "base64"
                    | "sha256sum"
                    | "sort"
                    | "uniq"
            );
            if !allow_glob {
                return raw_words.to_vec();
            }
            let mut g_out = Vec::with_capacity(raw_words.len());
            g_out.push(raw_words[0].clone());
            for (idx, tok) in raw_words.iter().enumerate().skip(1) {
                let quoted = quoted_flags.get(idx).copied().unwrap_or(false);
                if !quoted && (tok.contains('*') || tok.contains('?')) {
                    let norm_pat = Self::normalize_guest_path(cwd_str, tok);
                    let mut matches = Vec::new();
                    for path in sandbox.inodes.keys() {
                        if glob_match(&norm_pat, path) {
                            matches.push(path.clone());
                        }
                    }
                    if !matches.is_empty() {
                        g_out.extend(matches);
                        continue;
                    }
                }
                g_out.push(tok.clone());
            }
            g_out
        };

        for (stage_idx, stage_raw) in stages.iter().enumerate() {
            let parsed = parse_stage_words_and_redirects(
                stage_raw,
                &self.env_vars,
                self.last_exit_code,
                self.next_pid,
            );
            let mut stage_env = self.env_vars.clone();
            for (k, v) in &parsed.assignments {
                stage_env.insert(k.clone(), v.clone());
            }

            if parsed.words.is_empty() && !parsed.assignments.is_empty() && stages.len() == 1 {
                for (k, v) in parsed.assignments {
                    self.env_vars.insert(k, v);
                }
                last_exit = 0;
                piped_stdin.clear();
                continue;
            }

            let stage_stdin = if let Some(h_idx) = parsed.redirects.stdin_heredoc {
                if let Some(h) = heredocs.get(h_idx) {
                    if h.expand_vars {
                        expand_shell_vars(&h.body, &stage_env, self.last_exit_code, self.next_pid)
                            .into_bytes()
                    } else {
                        h.body.as_bytes().to_vec()
                    }
                } else {
                    Vec::new()
                }
            } else if let Some(ref in_file) = parsed.redirects.stdin_file {
                if in_file == "/dev/null" {
                    Vec::new()
                } else {
                    let norm_in = Self::normalize_guest_path(working_cwd, in_file);
                    match self.read_guest_file(&norm_in) {
                        Some(b) => b,
                        None => {
                            total_stderr.push_str(&format!(
                                "mvm-sh: {in_file}: No such file or directory\n"
                            ));
                            last_exit = 1;
                            piped_stdin.clear();
                            continue;
                        }
                    }
                }
            } else {
                std::mem::take(&mut piped_stdin)
            };

            if parsed.words.is_empty() {
                if let Some((ref target, append)) = parsed.redirects.stdout_file {
                    if let Err(msg) =
                        self.write_redirected_bytes(working_cwd, target, &[], append, None)
                    {
                        total_stderr.push_str(&msg);
                        total_stderr.push('\n');
                        last_exit = 1;
                    }
                }
                continue;
            }

            if allow_bridge {
                let (eff_words, eff_env) =
                    unwrap_env_and_polyglot(self, &parsed.words, &stage_env, working_cwd);
                let head = eff_words.first().map(String::as_str).unwrap_or("");
                let redirect = parsed.redirects.stdout_file.clone();

                let maybe_bridge: Option<MvmInteractiveOutcome> = match head {
                    "nslookup" | "dig" | "host" => Some(MvmInteractiveOutcome::NeedsBridge(
                        self.prepare_dns_bridge(&eff_words, working_cwd, redirect)?,
                    )),
                    "curl" | "wget" => Some(MvmInteractiveOutcome::NeedsBridge(
                        self.prepare_http_bridge(&eff_words, working_cwd, redirect)?,
                    )),
                    "pkg" => Some(self.exec_pkg_command(&eff_words, working_cwd, redirect)?),
                    "git" => Some(self.exec_git_command(&eff_words, working_cwd, redirect)?),
                    "node" | "npm" | "npx" | "python" | "python3" | "pip" | "pip3" | "wasm"
                    | "sqlite3" => Some(MvmInteractiveOutcome::NeedsBridge(
                        self.prepare_polyglot_bridge(
                            head,
                            &eff_words,
                            working_cwd,
                            &stage_stdin,
                            &eff_env,
                            &parsed.redirects,
                        )?,
                    )),
                    _ => None,
                };

                if let Some(outcome) = maybe_bridge {
                    match outcome {
                        MvmInteractiveOutcome::NeedsBridge(mut pending) => {
                            let rem: Vec<serde_json::Value> = stages[stage_idx + 1..]
                                .iter()
                                .map(|s| serde_json::Value::String(s.clone()))
                                .collect();
                            if let Some(obj) = pending.resume_state.as_object_mut() {
                                obj.insert(
                                    "pipeline_stderr_prefix".to_string(),
                                    serde_json::Value::String(total_stderr),
                                );
                                obj.insert(
                                    "pipeline_insts_prefix".to_string(),
                                    serde_json::Value::Number(total_insts.into()),
                                );
                                obj.insert(
                                    "remaining_pipeline_stages".to_string(),
                                    serde_json::Value::Array(rem),
                                );
                            }
                            all_frames.extend(pending.vsock_frames);
                            pending.vsock_frames = all_frames;
                            return Ok(MvmInteractiveOutcome::NeedsBridge(pending));
                        }
                        MvmInteractiveOutcome::Completed(done) => {
                            total_stderr.push_str(&done.stderr);
                            total_insts = total_insts.saturating_add(done.instructions_retired);
                            all_frames.extend(done.vsock_frames);
                            last_exit = done.exit_code;
                            if done.compiled_elf_machine.is_some() {
                                compiled_machine = done.compiled_elf_machine;
                            }
                            piped_stdin = done.stdout.into_bytes();
                            continue;
                        }
                    }
                }
            }

            let words = expand_globs(self, &parsed.words, &parsed.quoted_flags, working_cwd);
            let mut stage_res =
                self.exec_single_command_words(&words, working_cwd, &stage_stdin, &stage_env)?;
            if parsed.redirects.stderr_to_stdout {
                stage_res
                    .stdout
                    .extend_from_slice(stage_res.stderr.as_bytes());
                stage_res.stderr.clear();
            }
            if parsed.redirects.stdout_to_stderr {
                stage_res
                    .stderr
                    .push_str(&String::from_utf8_lossy(&stage_res.stdout));
                stage_res.stdout.clear();
            }
            if let Some((ref err_target, err_append)) = parsed.redirects.stderr_file {
                let err_bytes = stage_res.stderr.as_bytes().to_vec();
                stage_res.stderr.clear();
                if let Err(msg) = self.write_redirected_bytes(
                    working_cwd,
                    err_target,
                    &err_bytes,
                    err_append,
                    None,
                ) {
                    stage_res.stderr.push_str(&msg);
                    stage_res.stderr.push('\n');
                    stage_res.exit_code = 1;
                }
            }
            if let Some((ref out_target, out_append)) = parsed.redirects.stdout_file {
                if let Err(msg) = self.write_redirected_bytes(
                    working_cwd,
                    out_target,
                    &stage_res.stdout,
                    out_append,
                    None,
                ) {
                    stage_res.stderr.push_str(&msg);
                    stage_res.stderr.push('\n');
                    stage_res.exit_code = 1;
                }
                stage_res.stdout.clear();
            }

            total_stderr.push_str(&stage_res.stderr);
            total_insts = total_insts.saturating_add(stage_res.instructions_retired);
            all_frames.extend(stage_res.vsock_frames);
            last_exit = stage_res.exit_code;
            if stage_res.compiled_elf_machine.is_some() {
                compiled_machine = stage_res.compiled_elf_machine;
            }
            piped_stdin = stage_res.stdout;
        }

        self.last_exit_code = last_exit;
        Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
            exit_code: last_exit,
            stdout: String::from_utf8_lossy(&piped_stdin).into_owned(),
            stderr: total_stderr,
            instructions_retired: total_insts,
            vsock_frames: all_frames,
            compiled_elf_machine: compiled_machine,
            cow_state_sha256: self.cow_state_sha256(),
        }))
    }

    fn empty_outcome(&self) -> MvmExecOutcome {
        MvmExecOutcome {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            instructions_retired: 0,
            vsock_frames: Vec::new(),
            compiled_elf_machine: None,
            cow_state_sha256: self.cow_state_sha256(),
        }
    }

    /// Formal R4.MvmLinuxContainer.
    fn write_redirected_bytes(
        &mut self,
        cwd: &str,
        raw_target: &str,
        data: &[u8],
        append: bool,
        mode: Option<u32>,
    ) -> Result<(), String> {
        if raw_target == "/dev/null" {
            return Ok(());
        }
        let target_path = Self::normalize_guest_path(cwd, raw_target);
        if Self::is_protected_guest_path(&target_path) {
            return Err(format!(
                "mvm: refused write to protected canonical path '{target_path}'"
            ));
        }
        if append {
            let mut buf = self.read_guest_file(&target_path).unwrap_or_default();
            buf.extend_from_slice(data);
            self.write_guest_file(&target_path, &buf)
                .map_err(|e| format!("{e}"))?;
        } else {
            self.write_guest_file(&target_path, data)
                .map_err(|e| format!("{e}"))?;
        }
        if let Some(m) = mode {
            if let Some(ino) = self.inodes.get_mut(&target_path) {
                ino.mode = m;
            }
        }
        Ok(())
    }

    fn exec_single_segment_ast(
        &mut self,
        seg: &ShellCommandSegment,
        working_cwd: &mut String,
    ) -> Result<MvmExecOutcome, MvmError> {
        let trimmed = seg.raw.trim();
        if trimmed.is_empty() {
            return Ok(self.empty_outcome());
        }

        let stages = &seg.stages;
        if stages.is_empty() {
            return Ok(self.empty_outcome());
        }

        // Preserve genuine RV64 clone+pipe2+dup3+execve+wait4 execution for simple /bin/mvmbox 2-stage pipelines
        if seg.heredocs.is_empty()
            && stages.len() == 2
            && !trimmed.contains('$')
            && !trimmed.contains('>')
            && !trimmed.contains('<')
            && !trimmed.contains('=')
            && self.inodes.contains_key("/bin/sh")
        {
            let p0 = parse_stage_words_and_redirects(
                &stages[0],
                &self.env_vars,
                self.last_exit_code,
                self.next_pid,
            );
            let p1 = parse_stage_words_and_redirects(
                &stages[1],
                &self.env_vars,
                self.last_exit_code,
                self.next_pid,
            );
            let is_mvmbox_simple = |words: &[String]| -> bool {
                let Some(prog) = words.first().map(String::as_str) else {
                    return false;
                };
                if self
                    .resolve_guest_exec_path(working_cwd, prog)
                    .is_some_and(|p| p.starts_with("/usr/local/bin/"))
                {
                    return false;
                }
                matches!(
                    prog,
                    "echo" | "sha256sum" | "uname" | "pwd" | "id" | "whoami" | "true" | "false"
                )
            };
            if is_mvmbox_simple(&p0.words) && is_mvmbox_simple(&p1.words) {
                let argv = vec!["/bin/sh".to_string(), "-c".to_string(), trimmed.to_string()];
                return self.run_elf_process("/bin/sh", &argv, working_cwd);
            }
        }

        match self.exec_pipeline_stages_with_mode(
            stages,
            &seg.heredocs,
            working_cwd,
            Vec::new(),
            String::new(),
            0,
            0,
            Vec::new(),
            None,
            false,
        )? {
            MvmInteractiveOutcome::Completed(done) => Ok(done),
            MvmInteractiveOutcome::NeedsBridge(_) => Err(MvmError::BridgeRejected(
                BridgeError::InvalidBridgeJson("interactive bridge unexpected".to_string()),
            )),
        }
    }

    fn exec_single_command_words(
        &mut self,
        words: &[String],
        working_cwd: &mut String,
        stdin_bytes: &[u8],
        stage_env: &BTreeMap<String, String>,
    ) -> Result<StageExecBytesOutcome, MvmError> {
        if words.is_empty() {
            return Ok(StageExecBytesOutcome::ok_empty());
        }
        let prog = words[0].as_str();

        // Shell built-in `cd`
        if prog == "cd" {
            let target = words.get(1).map(String::as_str).unwrap_or("/workspace");
            let next_dir = Self::normalize_guest_path(working_cwd, target);
            let dir_prefix = if next_dir == "/" {
                "/".to_string()
            } else {
                format!("{next_dir}/")
            };
            let exists = self.dirs.contains(&next_dir)
                || self.inodes.keys().any(|k| k.starts_with(&dir_prefix));
            if exists {
                *working_cwd = next_dir.clone();
                self.env_vars.insert("PWD".to_string(), next_dir);
                return Ok(StageExecBytesOutcome::ok_empty());
            }
            return Ok(StageExecBytesOutcome::err(
                1,
                format!("cd: no such file or directory: {target}\n"),
            ));
        }

        // Built-in `export` / `unset`
        if prog == "export" {
            let args = &words[1..];
            if args.is_empty() {
                let mut out = String::new();
                for (k, v) in stage_env {
                    out.push_str(&format!("export {k}='{}'\n", v.replace('\'', "'\\''")));
                }
                return Ok(StageExecBytesOutcome::ok_text(out));
            }
            let mut remove_mode = false;
            let mut saw_var = false;
            for arg in args {
                if arg == "-p" {
                    continue;
                }
                if arg == "-n" {
                    remove_mode = true;
                    continue;
                }
                saw_var = true;
                if remove_mode {
                    self.env_vars.remove(arg);
                } else if let Some((k, v)) = arg.split_once('=') {
                    if !is_valid_env_name(k) {
                        return Ok(StageExecBytesOutcome::err(
                            1,
                            format!("export: `{arg}': not a valid identifier\n"),
                        ));
                    }
                    self.env_vars.insert(k.to_string(), v.to_string());
                } else {
                    if !is_valid_env_name(arg) {
                        return Ok(StageExecBytesOutcome::err(
                            1,
                            format!("export: `{arg}': not a valid identifier\n"),
                        ));
                    }
                    self.env_vars.entry(arg.clone()).or_default();
                }
            }
            if !saw_var {
                let mut out = String::new();
                for (k, v) in &self.env_vars {
                    out.push_str(&format!("export {k}='{}'\n", v.replace('\'', "'\\''")));
                }
                return Ok(StageExecBytesOutcome::ok_text(out));
            }
            return Ok(StageExecBytesOutcome::ok_empty());
        }
        if prog == "unset" {
            for k in &words[1..] {
                self.env_vars.remove(k);
            }
            return Ok(StageExecBytesOutcome::ok_empty());
        }

        // Built-in `rm` supporting `-r`, `-f`, `-rf` for files and directories
        if prog == "rm" {
            let mut recursive = false;
            let mut force = false;
            let mut targets = Vec::new();
            for arg in &words[1..] {
                if arg.starts_with('-') && arg.len() > 1 {
                    for ch in arg[1..].chars() {
                        match ch {
                            'r' | 'R' => recursive = true,
                            'f' => force = true,
                            _ => {}
                        }
                    }
                } else {
                    targets.push(arg.as_str());
                }
            }
            if targets.is_empty() {
                return Ok(StageExecBytesOutcome::err(
                    if force { 0 } else { 1 },
                    if force {
                        String::new()
                    } else {
                        "rm: missing operand\n".to_string()
                    },
                ));
            }
            let mut err_out = String::new();
            let mut code = 0i32;
            for raw_t in targets {
                let norm = Self::normalize_guest_path(working_cwd, raw_t);
                match self.remove_guest_path(&norm, recursive) {
                    Ok(0) if !force => {
                        code = 1;
                        err_out.push_str(&format!(
                            "rm: cannot remove '{raw_t}': No such file or directory\n"
                        ));
                    }
                    Ok(_) => {}
                    Err(msg) => {
                        code = 1;
                        err_out.push_str(&msg);
                        err_out.push('\n');
                    }
                }
            }
            return Ok(StageExecBytesOutcome::err(code, err_out));
        }

        // Built-in C-to-RV64IM ELF64 compiler (`cc` / `gcc`) with multi-TU, `-I`, and `-D` support (`R4-63`)
        if prog == "cc" || prog == "gcc" || prog == "/usr/bin/cc" || prog == "/usr/bin/gcc" {
            let mut src_args: Vec<String> = Vec::new();
            let mut out_arg = "a.out".to_string();
            let mut inc_dirs: Vec<String> = vec![working_cwd.clone(), "/usr/include".to_string()];
            let mut defines: Vec<(String, String)> = Vec::new();
            let mut i = 1usize;
            while i < words.len() {
                let w = &words[i];
                if w == "-o" && i + 1 < words.len() {
                    out_arg = words[i + 1].clone();
                    i += 2;
                    continue;
                }
                if let Some(rest) = w.strip_prefix("-o") {
                    if !rest.is_empty() {
                        out_arg = rest.to_string();
                        i += 1;
                        continue;
                    }
                }
                if w == "-I" && i + 1 < words.len() {
                    inc_dirs.push(Self::normalize_guest_path(working_cwd, &words[i + 1]));
                    i += 2;
                    continue;
                }
                if let Some(dir) = w.strip_prefix("-I") {
                    if !dir.is_empty() {
                        inc_dirs.push(Self::normalize_guest_path(working_cwd, dir));
                    }
                    i += 1;
                    continue;
                }
                if w == "-D" && i + 1 < words.len() {
                    let def = &words[i + 1];
                    let (k, v) = def.split_once('=').unwrap_or((def.as_str(), "1"));
                    defines.push((k.to_string(), v.to_string()));
                    i += 2;
                    continue;
                }
                if let Some(def) = w.strip_prefix("-D") {
                    if !def.is_empty() {
                        let (k, v) = def.split_once('=').unwrap_or((def, "1"));
                        defines.push((k.to_string(), v.to_string()));
                    }
                    i += 1;
                    continue;
                }
                if w.starts_with('-') {
                    i += 1;
                    continue;
                }
                src_args.push(w.clone());
                i += 1;
            }
            if src_args.is_empty() {
                return Ok(StageExecBytesOutcome::err(
                    1,
                    "cc: missing input C source file\n".to_string(),
                ));
            }
            let out_path = Self::normalize_guest_path(working_cwd, &out_arg);
            let mut unit_storage: Vec<(String, String)> = Vec::with_capacity(src_args.len());
            for raw_src in &src_args {
                let src_path = Self::normalize_guest_path(working_cwd, raw_src);
                let Some(src_bytes) = self.read_guest_file(&src_path) else {
                    return Ok(StageExecBytesOutcome::err(
                        1,
                        format!("cc: file not found: {src_path}\n"),
                    ));
                };
                unit_storage.push((src_path, String::from_utf8_lossy(&src_bytes).into_owned()));
            }
            let unit_refs: Vec<(&str, &str)> = unit_storage
                .iter()
                .map(|(p, s)| (p.as_str(), s.as_str()))
                .collect();
            return match compile_c_units_to_rv64_elf(&unit_refs, &inc_dirs, &defines, |hdr| {
                self.read_guest_file(hdr)
                    .and_then(|b| String::from_utf8(b).ok())
            }) {
                Ok(elf_bytes) => {
                    let adm = admit_elf64_riscv(&elf_bytes);
                    if adm != ElfAdmission::ElfAdmitted {
                        return Err(MvmError::ElfRejected(adm));
                    }
                    let machine = u16::from_le_bytes([elf_bytes[18], elf_bytes[19]]);
                    if let Err(msg) = self.write_redirected_bytes(
                        "/",
                        &out_path,
                        &elf_bytes,
                        false,
                        Some(0o100755),
                    ) {
                        return Ok(StageExecBytesOutcome::err(1, format!("{msg}\n")));
                    }
                    Ok(StageExecBytesOutcome {
                        exit_code: 0,
                        stdout: Vec::new(),
                        stderr: String::new(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: Some(machine),
                    })
                }
                Err(diag) => Ok(StageExecBytesOutcome::err(1, format!("{diag}\n"))),
            };
        }

        // POSIX `sh`, `xargs`, and `make` developer runners (`R4-63`)
        if matches!(prog, "sh" | "/bin/sh" | "/usr/bin/sh") {
            return self.exec_sh_script_command(&words[1..], working_cwd, stdin_bytes);
        }
        if matches!(prog, "xargs" | "/bin/xargs" | "/usr/bin/xargs") {
            return self.exec_xargs_command(&words[1..], working_cwd, stdin_bytes, stage_env);
        }
        if matches!(prog, "make" | "/bin/make" | "/usr/bin/make") {
            return self.exec_make_command(&words[1..], working_cwd);
        }
        if matches!(
            prog,
            "cargo" | "/bin/cargo" | "/usr/bin/cargo" | "rustc" | "/bin/rustc" | "/usr/bin/rustc"
        ) {
            return self.exec_cargo_or_rustc_command(prog, &words[1..], working_cwd);
        }

        // Local synchronous `pkg` / `git` invocations
        if prog == "pkg" || prog == "git" {
            let res = if prog == "pkg" {
                self.exec_pkg_command(words, working_cwd, None)?
            } else {
                self.exec_git_command(words, working_cwd, None)?
            };
            if let MvmInteractiveOutcome::Completed(done) = res {
                return Ok(StageExecBytesOutcome::from_exec_outcome(done));
            }
            return Ok(StageExecBytesOutcome::err(
                1,
                format!("{prog}: remote operation requires interactive SpiralCore bridge\n"),
            ));
        }

        // Explicit path execution (`/bin/...`, `/sbin/...`, `/usr/bin/...`, `/workspace/...`, `./...`)
        if prog.starts_with('/') || prog.starts_with("./") || prog.starts_with("../") {
            let p = Self::normalize_guest_path(working_cwd, prog);
            if let Some(applet) = p
                .strip_prefix("/usr/bin/")
                .or_else(|| p.strip_prefix("/bin/"))
                .or_else(|| p.strip_prefix("/sbin/"))
            {
                if matches!(applet, "rg" | "patch" | "df" | "free" | "xxd") {
                    if let Some(res) = self.try_exec_native_coreutil(
                        applet,
                        &words[1..],
                        working_cwd,
                        stdin_bytes,
                        stage_env,
                    ) {
                        return Ok(res);
                    }
                }
            }
            if self.inodes.contains_key(&p) {
                let out = self.run_elf_process_with_stdin(&p, words, working_cwd, stdin_bytes)?;
                return Ok(StageExecBytesOutcome::from_exec_outcome(out));
            }
            if let Some(applet) = p
                .strip_prefix("/usr/bin/")
                .or_else(|| p.strip_prefix("/bin/"))
                .or_else(|| p.strip_prefix("/sbin/"))
            {
                if let Some(res) = self.try_exec_native_coreutil(
                    applet,
                    &words[1..],
                    working_cwd,
                    stdin_bytes,
                    stage_env,
                ) {
                    return Ok(res);
                }
                if let Some(bin_cand) = ["/bin/", "/sbin/", "/usr/local/bin/"]
                    .iter()
                    .map(|pref| format!("{pref}{applet}"))
                    .find(|c| self.inodes.contains_key(c))
                {
                    let out = self.run_elf_process_with_stdin(
                        &bin_cand,
                        words,
                        working_cwd,
                        stdin_bytes,
                    )?;
                    return Ok(StageExecBytesOutcome::from_exec_outcome(out));
                }
            }
            return Ok(StageExecBytesOutcome::err(
                127,
                format!("mvm-sh: command not found: {prog}\n"),
            ));
        }

        // User-installed packages (`/usr/local/bin/<prog>`) or compiled workspace binaries run as RV64 ELF
        for prefix in ["/usr/local/bin/", "/workspace/"] {
            let cand = format!("{prefix}{prog}");
            if self.inodes.contains_key(&cand) {
                let out =
                    self.run_elf_process_with_stdin(&cand, words, working_cwd, stdin_bytes)?;
                return Ok(StageExecBytesOutcome::from_exec_outcome(out));
            }
        }

        // Keep standard `uname` (`-a`, `-m`, `-r`, bare) and `sha256sum` executing through RV64 `/bin` ELF
        let use_rv64_uname = prog == "uname"
            && (words.len() == 1
                || (words.len() == 2 && matches!(words[1].as_str(), "-a" | "-m" | "-r")));
        let use_rv64_sha256sum = prog == "sha256sum"
            && words.iter().skip(1).all(|a| !a.starts_with('-'))
            && self.inodes.contains_key("/bin/sha256sum");
        if use_rv64_uname || use_rv64_sha256sum {
            let bin = format!("/bin/{prog}");
            if self.inodes.contains_key(&bin) {
                let out = self.run_elf_process_with_stdin(&bin, words, working_cwd, stdin_bytes)?;
                return Ok(StageExecBytesOutcome::from_exec_outcome(out));
            }
        }

        // Native Rust coreutils applets operating directly on MvmCowSnapshot inodes
        if let Some(res) =
            self.try_exec_native_coreutil(prog, &words[1..], working_cwd, stdin_bytes, stage_env)
        {
            return Ok(res);
        }

        // Fallback `/bin` / `/sbin` ELF (e.g. `id`, `whoami`, `mvmbox`, `vsock_probe`, `mvm-agent`)
        for prefix in ["/bin/", "/sbin/", "/usr/bin/"] {
            let cand = format!("{prefix}{prog}");
            if self.inodes.contains_key(&cand) {
                let out =
                    self.run_elf_process_with_stdin(&cand, words, working_cwd, stdin_bytes)?;
                return Ok(StageExecBytesOutcome::from_exec_outcome(out));
            }
        }

        Ok(StageExecBytesOutcome::err(
            127,
            format!("mvm-sh: command not found: {prog}\n"),
        ))
    }

    /// Formal R4.MvmLinuxContainer execute POSIX shell scripts or `-c` commands inside guest MVM CoW VFS (`R4-63`).
    fn exec_sh_script_command(
        &mut self,
        args: &[String],
        working_cwd: &str,
        stdin_bytes: &[u8],
    ) -> Result<StageExecBytesOutcome, MvmError> {
        let mut errexit = false;
        let mut inline_cmd = false;
        let mut idx = 0usize;
        while idx < args.len() {
            let a = &args[idx];
            if a == "--" {
                idx += 1;
                break;
            }
            if !a.starts_with('-') || a == "-" {
                break;
            }
            for ch in a[1..].chars() {
                match ch {
                    'e' => errexit = true,
                    'c' => inline_cmd = true,
                    'x' => {}
                    _ => {}
                }
            }
            idx += 1;
        }

        let (script_name, script_text, pos_args): (String, String, Vec<String>) = if inline_cmd {
            let Some(cmd_str) = args.get(idx) else {
                return Ok(StageExecBytesOutcome::err(
                    2,
                    "sh: -c requires an argument\n".to_string(),
                ));
            };
            let sname = args
                .get(idx + 1)
                .cloned()
                .unwrap_or_else(|| "sh".to_string());
            let rest = if args.len() > idx + 2 {
                args[idx + 2..].to_vec()
            } else {
                Vec::new()
            };
            (sname, cmd_str.clone(), rest)
        } else if let Some(path_arg) = args.get(idx) {
            let norm = Self::normalize_guest_path(working_cwd, path_arg);
            let Some(bytes) = self.read_guest_file(&norm) else {
                return Ok(StageExecBytesOutcome::err(
                    127,
                    format!("sh: can't open '{path_arg}': No such file or directory\n"),
                ));
            };
            let rest = args[idx + 1..].to_vec();
            (
                path_arg.clone(),
                String::from_utf8_lossy(&bytes).into_owned(),
                rest,
            )
        } else {
            (
                "sh".to_string(),
                String::from_utf8_lossy(stdin_bytes).into_owned(),
                Vec::new(),
            )
        };

        let mut logical_lines = Vec::new();
        let mut cur_line = String::new();
        for raw in script_text.lines() {
            let no_cr = raw.trim_end_matches('\r');
            let trimmed = no_cr.trim_start();
            if cur_line.is_empty() && (trimmed.is_empty() || trimmed.starts_with('#')) {
                continue;
            }
            if let Some(cont) = no_cr.strip_suffix('\\') {
                cur_line.push_str(cont);
                cur_line.push(' ');
            } else {
                cur_line.push_str(no_cr);
                if !cur_line.trim().is_empty() {
                    logical_lines.push(cur_line.clone());
                }
                cur_line.clear();
            }
        }
        if !cur_line.trim().is_empty() {
            logical_lines.push(cur_line);
        }

        let outer_cwd = self.current_cwd.clone();
        let mut sub_cwd = working_cwd.to_string();
        let mut out_bytes = Vec::new();
        let mut err_str = String::new();
        let mut total_insts = 0u64;
        let mut vsock_frames = Vec::new();
        let mut compiled_elf_machine = None;
        let mut last_exit = 0i32;

        for raw_line in &logical_lines {
            let mut expanded = Vec::with_capacity(raw_line.len());
            let bytes = raw_line.as_bytes();
            let mut i = 0usize;
            while i < bytes.len() {
                if bytes[i] == b'$' && i + 1 < bytes.len() {
                    let next = bytes[i + 1];
                    if next == b'#' {
                        expanded.extend_from_slice(pos_args.len().to_string().as_bytes());
                        i += 2;
                        continue;
                    }
                    if next == b'@' || next == b'*' {
                        expanded.extend_from_slice(pos_args.join(" ").as_bytes());
                        i += 2;
                        continue;
                    }
                    if next.is_ascii_digit() {
                        let d = (next - b'0') as usize;
                        if d == 0 {
                            expanded.extend_from_slice(script_name.as_bytes());
                        } else if let Some(val) = pos_args.get(d - 1) {
                            expanded.extend_from_slice(val.as_bytes());
                        }
                        i += 2;
                        continue;
                    }
                    if next == b'{'
                        && i + 3 < bytes.len()
                        && bytes[i + 2].is_ascii_digit()
                        && bytes[i + 3] == b'}'
                    {
                        let d = (bytes[i + 2] - b'0') as usize;
                        if d == 0 {
                            expanded.extend_from_slice(script_name.as_bytes());
                        } else if let Some(val) = pos_args.get(d - 1) {
                            expanded.extend_from_slice(val.as_bytes());
                        }
                        i += 4;
                        continue;
                    }
                }
                expanded.push(bytes[i]);
                i += 1;
            }
            let expanded_str = String::from_utf8_lossy(&expanded).into_owned();
            let step = self.exec_command(&expanded_str, &sub_cwd)?;
            sub_cwd = self.current_cwd.clone();
            out_bytes.extend_from_slice(step.stdout.as_bytes());
            err_str.push_str(&step.stderr);
            last_exit = step.exit_code;
            total_insts = total_insts.saturating_add(step.instructions_retired);
            vsock_frames.extend(step.vsock_frames);
            if step.compiled_elf_machine.is_some() {
                compiled_elf_machine = step.compiled_elf_machine;
            }
            if errexit && last_exit != 0 {
                break;
            }
        }

        self.current_cwd = outer_cwd.clone();
        self.env_vars.insert("PWD".to_string(), outer_cwd);

        Ok(StageExecBytesOutcome {
            exit_code: last_exit,
            stdout: out_bytes,
            stderr: err_str,
            instructions_retired: total_insts,
            vsock_frames,
            compiled_elf_machine,
        })
    }

    /// Formal R4.MvmLinuxContainer execute piped argument lists via MVM command dispatch (`R4-63`).
    fn exec_xargs_command(
        &mut self,
        args: &[String],
        working_cwd: &mut String,
        stdin_bytes: &[u8],
        stage_env: &BTreeMap<String, String>,
    ) -> Result<StageExecBytesOutcome, MvmError> {
        let mut nul_sep = false;
        let mut no_run_empty = false;
        let mut max_args: Option<usize> = None;
        let mut replace_tok: Option<String> = None;
        let mut idx = 0usize;
        while idx < args.len() {
            let a = &args[idx];
            if a == "--" {
                idx += 1;
                break;
            }
            if !a.starts_with('-') || a == "-" {
                break;
            }
            if a == "-0" || a == "--null" {
                nul_sep = true;
            } else if a == "-r" || a == "--no-run-if-empty" {
                no_run_empty = true;
            } else if a == "-n" && idx + 1 < args.len() {
                max_args = args[idx + 1].parse::<usize>().ok().filter(|&v| v > 0);
                idx += 2;
                continue;
            } else if a == "-I" && idx + 1 < args.len() {
                replace_tok = Some(args[idx + 1].clone());
                idx += 2;
                continue;
            }
            idx += 1;
        }

        let cmd_template: Vec<String> = if idx < args.len() {
            args[idx..].to_vec()
        } else {
            vec!["echo".to_string()]
        };

        let raw_text = String::from_utf8_lossy(stdin_bytes);
        let items: Vec<String> = if nul_sep {
            raw_text
                .split('\0')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        } else if replace_tok.is_some() {
            raw_text
                .lines()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        } else {
            raw_text.split_whitespace().map(|s| s.to_string()).collect()
        };

        if items.is_empty() && no_run_empty {
            return Ok(StageExecBytesOutcome::ok_empty());
        }

        let mut stdout = Vec::new();
        let mut stderr = String::new();
        let mut exit_code = 0i32;
        let mut total_insts = 0u64;
        let mut vsock_frames = Vec::new();
        let mut compiled_elf_machine = None;

        if let Some(ref ph) = replace_tok {
            for item in &items {
                let words: Vec<String> = cmd_template
                    .iter()
                    .map(|w| w.replace(ph.as_str(), item))
                    .collect();
                let res = self.exec_single_command_words(&words, working_cwd, &[], stage_env)?;
                stdout.extend_from_slice(&res.stdout);
                stderr.push_str(&res.stderr);
                if res.exit_code != 0 {
                    exit_code = res.exit_code;
                }
                total_insts = total_insts.saturating_add(res.instructions_retired);
                vsock_frames.extend(res.vsock_frames);
                if res.compiled_elf_machine.is_some() {
                    compiled_elf_machine = res.compiled_elf_machine;
                }
            }
        } else if items.is_empty() {
            let res = self.exec_single_command_words(&cmd_template, working_cwd, &[], stage_env)?;
            return Ok(res);
        } else {
            let chunk_len = max_args.unwrap_or(items.len());
            for chunk in items.chunks(chunk_len) {
                let mut words = cmd_template.clone();
                words.extend(chunk.iter().cloned());
                let res = self.exec_single_command_words(&words, working_cwd, &[], stage_env)?;
                stdout.extend_from_slice(&res.stdout);
                stderr.push_str(&res.stderr);
                if res.exit_code != 0 {
                    exit_code = res.exit_code;
                }
                total_insts = total_insts.saturating_add(res.instructions_retired);
                vsock_frames.extend(res.vsock_frames);
                if res.compiled_elf_machine.is_some() {
                    compiled_elf_machine = res.compiled_elf_machine;
                }
            }
        }

        Ok(StageExecBytesOutcome {
            exit_code,
            stdout,
            stderr,
            instructions_retired: total_insts,
            vsock_frames,
            compiled_elf_machine,
        })
    }

    /// Formal R4.MvmLinuxContainer execute `Makefile` targets against guest `/workspace` and `/usr/src/mvm` (`R4-63`).
    fn exec_make_command(
        &mut self,
        args: &[String],
        working_cwd: &str,
    ) -> Result<StageExecBytesOutcome, MvmError> {
        let mut make_dir = working_cwd.to_string();
        let mut makefile_name: Option<String> = None;
        let mut dry_run = false;
        let mut cli_vars: BTreeMap<String, String> = BTreeMap::new();
        let mut requested_targets: Vec<String> = Vec::new();

        let mut idx = 0usize;
        while idx < args.len() {
            let a = &args[idx];
            if (a == "-C" || a == "--directory") && idx + 1 < args.len() {
                make_dir = Self::normalize_guest_path(&make_dir, &args[idx + 1]);
                idx += 2;
                continue;
            }
            if (a == "-f" || a == "--file" || a == "--makefile") && idx + 1 < args.len() {
                makefile_name = Some(args[idx + 1].clone());
                idx += 2;
                continue;
            }
            if a == "-n" || a == "--just-print" || a == "--dry-run" {
                dry_run = true;
                idx += 1;
                continue;
            }
            if a.starts_with('-') {
                idx += 1;
                continue;
            }
            if let Some((k, v)) = a.split_once('=') {
                cli_vars.insert(k.trim().to_string(), v.trim().to_string());
            } else {
                requested_targets.push(a.clone());
            }
            idx += 1;
        }

        let mf_path = if let Some(ref f) = makefile_name {
            Self::normalize_guest_path(&make_dir, f)
        } else {
            let c1 = Self::normalize_guest_path(&make_dir, "Makefile");
            let c2 = Self::normalize_guest_path(&make_dir, "makefile");
            if self.inodes.contains_key(&c1) {
                c1
            } else {
                c2
            }
        };
        let Some(mf_bytes) = self.read_guest_file(&mf_path) else {
            return Ok(StageExecBytesOutcome::err(
                2,
                format!(
                    "make: *** No targets specified and no makefile found in {make_dir}. Stop.\n"
                ),
            ));
        };
        let mf_text = String::from_utf8_lossy(&mf_bytes);

        #[derive(Clone)]
        struct MakeRule {
            name: String,
            deps: Vec<String>,
            recipes: Vec<String>,
        }

        let mut vars: BTreeMap<String, String> = BTreeMap::new();
        for (k, v) in &self.env_vars {
            vars.insert(k.clone(), v.clone());
        }
        let mut rules: BTreeMap<String, MakeRule> = BTreeMap::new();
        let mut default_target: Option<String> = None;
        let mut cur_target: Option<String> = None;

        for raw in mf_text.lines() {
            let line = raw.trim_end_matches('\r');
            if let Some(recipe_raw) = line.strip_prefix('\t') {
                if let Some(ref t) = cur_target {
                    if let Some(rule) = rules.get_mut(t) {
                        let r = recipe_raw.trim();
                        if !r.is_empty() {
                            rule.recipes.push(r.to_string());
                        }
                    }
                }
                continue;
            }
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(".PHONY") {
                cur_target = None;
                continue;
            }
            if let Some((lhs, rhs)) = trimmed.split_once(":=") {
                vars.insert(lhs.trim().to_string(), rhs.trim().to_string());
                cur_target = None;
                continue;
            }
            if let Some((lhs, rhs)) = trimmed.split_once('=') {
                if !lhs.contains(':') {
                    vars.insert(lhs.trim().to_string(), rhs.trim().to_string());
                    cur_target = None;
                    continue;
                }
            }
            if let Some((tpart, dpart)) = trimmed.split_once(':') {
                let tname = tpart.trim().to_string();
                if default_target.is_none() && !tname.starts_with('.') {
                    default_target = Some(tname.clone());
                }
                let deps: Vec<String> = dpart.split_whitespace().map(|s| s.to_string()).collect();
                rules.entry(tname.clone()).or_insert_with(|| MakeRule {
                    name: tname.clone(),
                    deps,
                    recipes: Vec::new(),
                });
                cur_target = Some(tname);
            }
        }
        for (k, v) in cli_vars {
            vars.insert(k, v);
        }

        fn expand_make_vars(
            input: &str,
            vars: &BTreeMap<String, String>,
            target: &str,
            deps: &[String],
        ) -> String {
            let mut cur = input.to_string();
            for _ in 0..8 {
                let mut next = Vec::with_capacity(cur.len());
                let b = cur.as_bytes();
                let mut i = 0usize;
                let mut changed = false;
                while i < b.len() {
                    if b[i] == b'$' && i + 1 < b.len() {
                        if b[i + 1] == b'$' {
                            next.push(b'$');
                            i += 2;
                            continue;
                        }
                        if b[i + 1] == b'@' {
                            next.extend_from_slice(target.as_bytes());
                            i += 2;
                            changed = true;
                            continue;
                        }
                        if b[i + 1] == b'<' {
                            if let Some(first_dep) = deps.first() {
                                next.extend_from_slice(first_dep.as_bytes());
                            }
                            i += 2;
                            changed = true;
                            continue;
                        }
                        if b[i + 1] == b'^' {
                            next.extend_from_slice(deps.join(" ").as_bytes());
                            i += 2;
                            changed = true;
                            continue;
                        }
                        if b[i + 1] == b'(' || b[i + 1] == b'{' {
                            let close = if b[i + 1] == b'(' { b')' } else { b'}' };
                            if let Some(rel_end) = b[i + 2..].iter().position(|&c| c == close) {
                                let key = String::from_utf8_lossy(&b[i + 2..i + 2 + rel_end]);
                                if let Some(val) = vars.get(key.as_ref()) {
                                    next.extend_from_slice(val.as_bytes());
                                }
                                i += 3 + rel_end;
                                changed = true;
                                continue;
                            }
                        }
                    }
                    next.push(b[i]);
                    i += 1;
                }
                cur = String::from_utf8_lossy(&next).into_owned();
                if !changed {
                    break;
                }
            }
            cur
        }

        if requested_targets.is_empty() {
            if let Some(dt) = default_target {
                requested_targets.push(dt);
            } else {
                return Ok(StageExecBytesOutcome::err(
                    2,
                    "make: *** No targets. Stop.\n".to_string(),
                ));
            }
        }

        fn collect_rule_order(
            tname: &str,
            rules: &BTreeMap<String, MakeRule>,
            vars: &BTreeMap<String, String>,
            visited: &mut BTreeSet<String>,
            order: &mut Vec<String>,
            depth: usize,
        ) -> Result<(), String> {
            if depth > 16 {
                return Err(format!(
                    "make: dependency recursion limit reached on `{tname}`"
                ));
            }
            let exp = expand_make_vars(tname, vars, tname, &[]);
            if visited.contains(&exp) {
                return Ok(());
            }
            if let Some(rule) = rules.get(&exp) {
                for dep in &rule.deps {
                    let dep_exp = expand_make_vars(dep, vars, &rule.name, &rule.deps);
                    if rules.contains_key(&dep_exp) {
                        collect_rule_order(&dep_exp, rules, vars, visited, order, depth + 1)?;
                    }
                }
                visited.insert(exp.clone());
                order.push(exp);
            }
            Ok(())
        }

        let mut visited = BTreeSet::new();
        let mut order = Vec::new();
        for t in &requested_targets {
            let exp_t = expand_make_vars(t, &vars, t, &[]);
            if !rules.contains_key(&exp_t) {
                return Ok(StageExecBytesOutcome::err(
                    2,
                    format!("make: *** No rule to make target '{exp_t}'. Stop.\n"),
                ));
            }
            if let Err(msg) = collect_rule_order(&exp_t, &rules, &vars, &mut visited, &mut order, 0)
            {
                return Ok(StageExecBytesOutcome::err(2, format!("{msg}\n")));
            }
        }

        let outer_cwd = self.current_cwd.clone();
        let mut stdout = Vec::new();
        let mut stderr = String::new();
        let mut total_insts = 0u64;
        let mut vsock_frames = Vec::new();
        let mut compiled_elf_machine = None;

        for tname in &order {
            if let Some(rule) = rules.get(tname) {
                for recipe in &rule.recipes {
                    let silent = recipe.starts_with('@');
                    let clean_recipe = recipe.strip_prefix('@').unwrap_or(recipe.as_str()).trim();
                    let cmd = expand_make_vars(clean_recipe, &vars, &rule.name, &rule.deps);
                    if !silent {
                        stdout.extend_from_slice(cmd.as_bytes());
                        stdout.push(b'\n');
                    }
                    if dry_run {
                        continue;
                    }
                    let step = self.exec_command(&cmd, &make_dir)?;
                    stdout.extend_from_slice(step.stdout.as_bytes());
                    stderr.push_str(&step.stderr);
                    total_insts = total_insts.saturating_add(step.instructions_retired);
                    vsock_frames.extend(step.vsock_frames);
                    if step.compiled_elf_machine.is_some() {
                        compiled_elf_machine = step.compiled_elf_machine;
                    }
                    if step.exit_code != 0 {
                        stderr.push_str(&format!(
                            "make: *** [{mf_path}: {tname}] Error {}\n",
                            step.exit_code
                        ));
                        self.current_cwd = outer_cwd.clone();
                        self.env_vars.insert("PWD".to_string(), outer_cwd);
                        return Ok(StageExecBytesOutcome {
                            exit_code: 2,
                            stdout,
                            stderr,
                            instructions_retired: total_insts,
                            vsock_frames,
                            compiled_elf_machine,
                        });
                    }
                }
            }
        }

        self.current_cwd = outer_cwd.clone();
        self.env_vars.insert("PWD".to_string(), outer_cwd);
        Ok(StageExecBytesOutcome {
            exit_code: 0,
            stdout,
            stderr,
            instructions_retired: total_insts,
            vsock_frames,
            compiled_elf_machine,
        })
    }

    /// Refuse in-guest `cargo` / `rustc` honestly (no fake AST evaluator or guest Rust toolchain).
    fn exec_cargo_or_rustc_command(
        &self,
        prog: &str,
        _args: &[String],
        _working_cwd: &str,
    ) -> Result<StageExecBytesOutcome, MvmError> {
        let tool = if prog == "rustc" || prog.ends_with("/rustc") {
            "rustc"
        } else {
            "cargo"
        };
        Ok(StageExecBytesOutcome::err(
            127,
            format!(
                "{tool}: unavailable inside RV64IMAC MVM sandbox (no AST evaluator or guest Rust toolchain installed; use cc for in-sandbox RV64 C compilation)\n"
            ),
        ))
    }

    /// Formal R4.MvmLinuxContainer.
    fn resolve_guest_exec_path(&self, working_cwd: &str, prog: &str) -> Option<String> {
        if prog.starts_with('/') || prog.starts_with("./") || prog.starts_with("../") {
            let p = Self::normalize_guest_path(working_cwd, prog);
            self.inodes.contains_key(&p).then_some(p)
        } else {
            [
                format!("/usr/local/bin/{prog}"),
                format!("/bin/{prog}"),
                format!("/sbin/{prog}"),
                format!("/usr/bin/{prog}"),
                format!("/workspace/{prog}"),
            ]
            .into_iter()
            .find(|cand| self.inodes.contains_key(cand))
        }
    }

    /// Formal R4.MvmLinuxContainer.
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
        self.run_elf_process_with_stdin(exec_path, argv, cwd, &[])
    }

    fn run_elf_process_with_stdin(
        &mut self,
        exec_path: &str,
        argv: &[String],
        cwd: &str,
        stdin_bytes: &[u8],
    ) -> Result<MvmExecOutcome, MvmError> {
        let stdout_buf = Rc::new(RefCell::new(Vec::new()));
        let stderr_buf = Rc::new(RefCell::new(Vec::new()));
        let stdin_buf = Rc::new(RefCell::new(VecDeque::from(stdin_bytes.to_vec())));

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
            "PATH=/usr/local/bin:/bin:/sbin:/usr/bin:/workspace".to_string(),
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
                if (a1 as u128) <= (bytes.len() as u128) {
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
                if Self::is_protected_guest_path(&norm) {
                    p.regs[10] = LINUX_EACCES as u64;
                } else {
                    self.ensure_parent_dirs(&norm);
                    self.dirs.insert(norm);
                    p.regs[10] = 0;
                }
            }
            35 => {
                // SYS_unlinkat(dirfd, path, flags)
                let raw = p.read_cstr(a1)?;
                let norm = Self::normalize_guest_path(&p.cwd, &raw);
                let dir_prefix = format!("{norm}/");
                let protects_descendant = self.dirs.contains(&norm)
                    && self
                        .inodes
                        .keys()
                        .any(|k| k.starts_with(&dir_prefix) && Self::is_protected_guest_path(k));
                if Self::is_protected_guest_path(&norm) || protects_descendant {
                    p.regs[10] = LINUX_EACCES as u64;
                } else if self.remove_inode_reclaiming(&norm) || self.dirs.remove(&norm) {
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
                    let mut target_norm = norm.clone();
                    for _ in 0..8 {
                        if let Some(ino) = self.inodes.get(&target_norm) {
                            if let InodeKind::Symlink { target } = &ino.kind {
                                target_norm = Self::resolve_symlink_step(&target_norm, target);
                                continue;
                            }
                        }
                        break;
                    }
                    if (o_creat || o_trunc || writable)
                        && (Self::is_protected_guest_path(&norm)
                            || Self::is_protected_guest_path(&target_norm))
                    {
                        p.regs[10] = LINUX_EACCES as u64;
                        return Ok(None);
                    }
                    if !self.inodes.contains_key(&norm) {
                        if o_creat {
                            if let Err(MvmError::CanonicalPathProtected(_)) =
                                self.write_guest_file(&norm, &[])
                            {
                                p.regs[10] = LINUX_EACCES as u64;
                                return Ok(None);
                            }
                        } else {
                            p.regs[10] = LINUX_ENOENT as u64;
                            return Ok(None);
                        }
                    } else if o_trunc && writable {
                        if let Err(MvmError::CanonicalPathProtected(_)) =
                            self.write_guest_file(&norm, &[])
                        {
                            p.regs[10] = LINUX_EACCES as u64;
                            return Ok(None);
                        }
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
                let max_bytes = usize::try_from(a2).unwrap_or(usize::MAX).min(1024 * 1024);
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
                    let Some(signed_next) = base.checked_add(off) else {
                        p.regs[10] = LINUX_EINVAL as u64;
                        return Ok(None);
                    };
                    let next = if signed_next <= 0 {
                        0usize
                    } else {
                        let Ok(pos) = usize::try_from(signed_next) else {
                            p.regs[10] = LINUX_EINVAL as u64;
                            return Ok(None);
                        };
                        pos
                    };
                    *offset = next;
                    p.regs[10] = next as u64;
                }
            }
            63 => {
                // SYS_read(fd, buf, count)
                let fd = a0 as i32;
                let count = usize::try_from(a2)
                    .unwrap_or(usize::MAX)
                    .min(MAX_GUEST_FILE_BYTES);
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
                    Some(FdKind::VsockSocket { .. })
                    | Some(FdKind::InetEncapsulatedSocket { .. }) => {
                        let msg_opt = match p.fds.get_mut(&fd) {
                            Some(FdKind::VsockSocket { rx_queue, .. })
                            | Some(FdKind::InetEncapsulatedSocket { rx_queue, .. }) => {
                                rx_queue.pop_front()
                            }
                            _ => None,
                        };
                        if let Some(msg) = msg_opt {
                            let take = count.min(msg.len());
                            if take > 0 {
                                p.write_bytes(a1, &msg[..take])?;
                            }
                            if take < msg.len() {
                                if let Some(
                                    FdKind::VsockSocket { rx_queue, .. }
                                    | FdKind::InetEncapsulatedSocket { rx_queue, .. },
                                ) = p.fds.get_mut(&fd)
                                {
                                    rx_queue.push_front(msg[take..].to_vec());
                                }
                            }
                            p.regs[10] = take as u64;
                        } else {
                            p.regs[10] = 0;
                        }
                    }
                    _ => p.regs[10] = LINUX_EBADF as u64,
                }
            }
            64 => {
                // SYS_write(fd, buf, count)
                let fd = a0 as i32;
                let count = usize::try_from(a2)
                    .map_err(|_| MvmError::CpuTrap("SYS_write count overflow".to_string()))?;
                let range = p.guest_range(a1, count).map_err(|_| {
                    MvmError::CpuTrap("SYS_write buffer fault".to_string())
                })?;
                let data = p.ram[range].to_vec();
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
                        if Self::is_protected_guest_path(&path) {
                            p.regs[10] = LINUX_EACCES as u64;
                            return Ok(None);
                        }
                        let mut cur = self.read_guest_file(&path).unwrap_or_default();
                        let pos = if append { cur.len() } else { offset };
                        let new_end = match pos.checked_add(data.len()) {
                            Some(end) if end <= MAX_GUEST_FILE_BYTES => end,
                            _ => {
                                p.regs[10] = LINUX_EFBIG as u64;
                                return Ok(None);
                            }
                        };
                        if new_end > cur.len() {
                            cur.resize(new_end, 0);
                        }
                        cur[pos..new_end].copy_from_slice(&data);
                        if let Err(MvmError::CanonicalPathProtected(_)) =
                            self.write_guest_file(&path, &cur)
                        {
                            p.regs[10] = LINUX_EACCES as u64;
                            return Ok(None);
                        }
                        if let Some(FdKind::RegularFile {
                            offset: off_mut, ..
                        }) = p.fds.get_mut(&fd)
                        {
                            *off_mut = new_end;
                        }
                        p.regs[10] = count as u64;
                    }
                    Some(FdKind::VsockSocket { .. })
                    | Some(FdKind::InetEncapsulatedSocket { .. }) => {
                        if let Some(evt) = self.emit_socket_spvs_packet(p, fd, &data) {
                            return Ok(Some(evt));
                        }
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
                // SYS_socket(domain, type, protocol) — Zero-NIC enforcement (`R4-37`) + SpiralCoreNet `AF_INET` encapsulation (`R4-63`)
                use crate::lexlean_admission::r4_mvm_linux_container::MvmSocketVerdict;
                match self.admit_mvm_guest_socket(a0) {
                    MvmSocketVerdict::AdmittedDirectVsock => {
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
                    MvmSocketVerdict::AdmittedSpiralCoreInetEncapsulation => {
                        let fd = p.next_fd;
                        p.next_fd += 1;
                        p.fds.insert(
                            fd,
                            FdKind::InetEncapsulatedSocket {
                                family: a0 as u16,
                                peer_ip: String::new(),
                                peer_port: 0,
                                rx_queue: VecDeque::new(),
                            },
                        );
                        p.regs[10] = fd as u64;
                    }
                    MvmSocketVerdict::RefusedZeroNicIsolated
                    | MvmSocketVerdict::RefusedUnsupportedDomain => {
                        p.regs[10] = LINUX_EAFNOSUPPORT as u64;
                    }
                }
            }
            200 | 201 | 242 => {
                // SYS_bind / SYS_listen / SYS_accept4
                p.regs[10] = 0;
            }
            203 => {
                // SYS_connect(sockfd, sockaddr*, addrlen)
                use crate::lexlean_admission::r4_mvm_linux_container::MvmSocketVerdict;
                let fd = a0 as i32;
                if a2 < 16 {
                    p.regs[10] = LINUX_EINVAL as u64;
                    return Ok(None);
                }
                let family = p.read_u16(a1)?;
                match self.admit_mvm_guest_socket(family as u64) {
                    MvmSocketVerdict::AdmittedDirectVsock => {
                        let port_addr = a1.checked_add(4).ok_or_else(|| {
                            MvmError::CpuTrap("SYS_connect sockaddr_vm port fault".to_string())
                        })?;
                        let cid_addr = a1.checked_add(8).ok_or_else(|| {
                            MvmError::CpuTrap("SYS_connect sockaddr_vm cid fault".to_string())
                        })?;
                        let port = p.read_u32(port_addr)? as u16;
                        let cid = p.read_u32(cid_addr)? as u16;
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
                    MvmSocketVerdict::AdmittedSpiralCoreInetEncapsulation => {
                        let range4 = p.guest_range(a1, 4).ok();
                        let port = range4.map_or(0, |r| {
                            u16::from_be_bytes([p.ram[r.start + 2], p.ram[r.start + 3]])
                        });
                        let peer_ip = if family == 2 {
                            let r8 = p.guest_range(a1, 8).map_err(|_| {
                                MvmError::CpuTrap("SYS_connect sockaddr_in fault".to_string())
                            })?;
                            format!(
                                "{}.{}.{}.{}",
                                p.ram[r8.start + 4],
                                p.ram[r8.start + 5],
                                p.ram[r8.start + 6],
                                p.ram[r8.start + 7]
                            )
                        } else {
                            let Ok(r24) = p.guest_range(a1, 24) else {
                                p.regs[10] = LINUX_EINVAL as u64;
                                return Ok(None);
                            };
                            if a2 < 24 {
                                p.regs[10] = LINUX_EINVAL as u64;
                                return Ok(None);
                            }
                            let mut hextets = [0u16; 8];
                            for (hi, slot) in hextets.iter_mut().enumerate() {
                                *slot = u16::from_be_bytes([
                                    p.ram[r24.start + 8 + hi * 2],
                                    p.ram[r24.start + 8 + hi * 2 + 1],
                                ]);
                            }
                            hextets
                                .iter()
                                .map(|h| format!("{h:x}"))
                                .collect::<Vec<_>>()
                                .join(":")
                        };
                        if is_private_or_loopback_host(&peer_ip) {
                            p.regs[10] = LINUX_EACCES as u64;
                            return Ok(None);
                        }
                        if let Some(FdKind::InetEncapsulatedSocket {
                            family: f_mut,
                            peer_ip: ip_mut,
                            peer_port: port_mut,
                            ..
                        }) = p.fds.get_mut(&fd)
                        {
                            *f_mut = family;
                            *ip_mut = peer_ip;
                            *port_mut = port;
                            p.regs[10] = 0;
                        } else {
                            p.regs[10] = LINUX_EBADF as u64;
                        }
                    }
                    MvmSocketVerdict::RefusedZeroNicIsolated
                    | MvmSocketVerdict::RefusedUnsupportedDomain => {
                        p.regs[10] = LINUX_EAFNOSUPPORT as u64;
                    }
                }
            }
            206 => {
                // SYS_sendto(sockfd, buf, len, flags, dest_addr, addrlen)
                let fd = a0 as i32;
                let len = usize::try_from(a2)
                    .map_err(|_| MvmError::CpuTrap("SYS_sendto length overflow".to_string()))?;
                let range = p
                    .guest_range(a1, len)
                    .map_err(|_| MvmError::CpuTrap("SYS_sendto fault".to_string()))?;
                let payload = p.ram[range].to_vec();
                if let Some(evt) = self.emit_socket_spvs_packet(p, fd, &payload) {
                    return Ok(Some(evt));
                }
            }
            207 => {
                // SYS_recvfrom(sockfd, buf, len, ...)
                let fd = a0 as i32;
                let max_len = usize::try_from(a2)
                    .unwrap_or(usize::MAX)
                    .min(MAX_GUEST_FILE_BYTES);
                let msg_opt = match p.fds.get_mut(&fd) {
                    Some(FdKind::VsockSocket { rx_queue, .. })
                    | Some(FdKind::InetEncapsulatedSocket { rx_queue, .. }) => rx_queue.pop_front(),
                    _ => {
                        p.regs[10] = LINUX_EBADF as u64;
                        return Ok(None);
                    }
                };
                if let Some(msg) = msg_opt {
                    let take = max_len.min(msg.len());
                    p.write_bytes(a1, &msg[..take])?;
                    if take < msg.len() {
                        if let Some(
                            FdKind::VsockSocket { rx_queue, .. }
                            | FdKind::InetEncapsulatedSocket { rx_queue, .. },
                        ) = p.fds.get_mut(&fd)
                        {
                            rx_queue.push_front(msg[take..].to_vec());
                        }
                    }
                    p.regs[10] = take as u64;
                } else {
                    p.regs[10] = 0;
                }
            }
            214 => {
                // SYS_brk(addr)
                if a0 >= p.brk_addr && a0 < GUEST_MMAP_BASE {
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
                let exec_target = if self.inodes.contains_key(&norm) {
                    norm
                } else if let Some(base) = norm.rsplit('/').next() {
                    let alt = format!("/usr/local/bin/{base}");
                    if self.inodes.contains_key(&alt) {
                        alt
                    } else {
                        norm
                    }
                } else {
                    norm
                };
                if self
                    .load_elf_into_process(p, &exec_target, &argv_vec, &envp_vec)
                    .is_err()
                {
                    p.regs[10] = LINUX_ENOENT as u64;
                }
            }
            222 => {
                // SYS_mmap(addr, length, ...)
                let Some(aligned) = a1.max(4096).checked_add(4095).map(|v| v & !4095) else {
                    p.regs[10] = (-12i64) as u64; // -ENOMEM
                    return Ok(None);
                };
                let out = p.mmap_cursor;
                if out
                    .checked_add(aligned)
                    .is_none_or(|end| end >= GUEST_STACK_TOP - 65536)
                {
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

    /// Formal R4.MvmLinuxContainer.
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

    /// Formal R4.MvmLinuxContainer.
    fn ensure_guest_ram_uncontaminated(&self) -> Result<(), MvmError> {
        for rule in self.host_secrets.values() {
            if !rule.raw_secret.is_empty() && self.guest_ram_contains(rule.raw_secret.as_bytes()) {
                return Err(MvmError::BridgeRejected(
                    BridgeError::GuestRamSecretContamination,
                ));
            }
        }
        Ok(())
    }

    fn admit_outbound_transport(&self, transport: TransportKind) -> Result<(), MvmError> {
        match admit_bridge_transport(self.guest_cid, 2, self.policy_mode, transport) {
            BridgeTransportVerdict::TransportAdmitted => Ok(()),
            BridgeTransportVerdict::PolicyDenied => {
                Err(MvmError::BridgeRejected(BridgeError::PolicyDenied))
            }
            BridgeTransportVerdict::InsecureTransportRejected => Err(MvmError::BridgeRejected(
                BridgeError::InsecureTransportRejected,
            )),
            BridgeTransportVerdict::InvalidVsockCid => {
                Err(MvmError::BridgeRejected(BridgeError::InvalidVsockCid))
            }
        }
    }

    fn admit_network_target(&self, host: &str) -> Result<(), MvmError> {
        let clean_host = host
            .trim()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .trim_end_matches('.')
            .to_ascii_lowercase();
        if clean_host.is_empty() || is_private_or_loopback_host(&clean_host) {
            return Err(MvmError::BridgeRejected(
                BridgeError::PrivateAddressBlocked(host.to_string()),
            ));
        }
        if !self.network_gate_enforced {
            return Ok(());
        }
        match self.network_mode {
            NetworkGateMode::Sandboxed => {
                if !is_sandboxed_capability_host(&clean_host) {
                    return Err(MvmError::BridgeRejected(BridgeError::CapabilityDenied(
                        host.to_string(),
                    )));
                }
            }
            NetworkGateMode::FullSpiralcore => {
                if !self.relay_consent_granted {
                    return Err(MvmError::BridgeRejected(BridgeError::RelayConsentRequired));
                }
            }
            NetworkGateMode::SpiralCoreNet => {}
        }
        Ok(())
    }

    /// Formal R4.MvmLinuxContainer evaluate guest socket domain admission against the formal `R4.MvmLinuxContainer.admitMvmSocket` policy (`R4-63`).
    fn admit_mvm_guest_socket(
        &self,
        domain: u64,
    ) -> crate::lexlean_admission::r4_mvm_linux_container::MvmSocketVerdict {
        use crate::lexlean_admission::r4_mvm_linux_container::{admit_mvm_socket, MvmNetworkMode};
        let net_mode = match self.network_mode {
            NetworkGateMode::SpiralCoreNet => MvmNetworkMode::SpiralCoreEncapsulated,
            NetworkGateMode::FullSpiralcore if self.relay_consent_granted => {
                MvmNetworkMode::SpiralCoreEncapsulated
            }
            _ => MvmNetworkMode::ZeroNicIsolated,
        };
        admit_mvm_socket(net_mode, domain == 2 || domain == 10, domain == 40)
    }

    /// Formal R4.MvmLinuxContainer encapsulate guest `AF_VSOCK` or `AF_INET` socket egress into a signed 16-byte `SPVS` frame (`R4-63`).
    fn emit_socket_spvs_packet(
        &self,
        p: &mut Rv64Process,
        fd: i32,
        guest_payload: &[u8],
    ) -> Option<StepEvent> {
        let (dst_cid, port, wire_payload, reply_bytes) = match p.fds.get(&fd) {
            Some(FdKind::VsockSocket {
                peer_cid,
                peer_port,
                ..
            }) => {
                let ack = if *peer_cid == 2 {
                    Some(b"SPIRALCORE_HOST_ACK".to_vec())
                } else {
                    None
                };
                (*peer_cid, *peer_port, guest_payload.to_vec(), ack)
            }
            Some(FdKind::InetEncapsulatedSocket {
                peer_ip, peer_port, ..
            }) => {
                if peer_ip.is_empty() || is_private_or_loopback_host(peer_ip) {
                    p.regs[10] = LINUX_EACCES as u64;
                    return None;
                }
                let env_json = serde_json::json!({
                    "op": "inet_encap",
                    "dst_ip": peer_ip,
                    "dst_port": peer_port,
                    "payload_len": guest_payload.len(),
                })
                .to_string()
                .into_bytes();
                let is_http = guest_payload.starts_with(b"GET ")
                    || guest_payload.starts_with(b"POST ")
                    || guest_payload.starts_with(b"HEAD ")
                    || guest_payload.starts_with(b"PUT ")
                    || guest_payload.starts_with(b"DELETE ");
                let ack = if is_http {
                    b"HTTP/1.1 200 OK\r\nX-SpiralCore-Encap: SPVS\r\nContent-Length: 19\r\n\r\nSPIRALCORE_INET_ACK".to_vec()
                } else {
                    b"SPIRALCORE_INET_ACK".to_vec()
                };
                (
                    SPIRALCORE_HOST_CID,
                    SPIRALCORE_VSOCK_PORT,
                    env_json,
                    Some(ack),
                )
            }
            _ => {
                p.regs[10] = LINUX_EBADF as u64;
                return None;
            }
        };

        let mut h = Sha256::new();
        h.update(&wire_payload);
        let dig = h.finalize();
        let hdr = SpiralCoreHeader {
            magic: SPIRALCORE_MAGIC,
            src_cid: self.guest_cid,
            dst_cid,
            vsock_port: port,
            cl06_spin: dig[0] & 63,
            e8_orbit: dig[1] & 15,
            payload_len: wire_payload.len() as u32,
        };
        let verdict = admit_spiralcore_header(&hdr, SPIRALCORE_MAX_PAYLOAD);
        if verdict != FrameVerdict::FrameAdmitted {
            p.regs[10] = LINUX_EINVAL as u64;
            return None;
        }
        if let Some(ack) = reply_bytes {
            match p.fds.get_mut(&fd) {
                Some(FdKind::VsockSocket { rx_queue, .. })
                | Some(FdKind::InetEncapsulatedSocket { rx_queue, .. }) => {
                    rx_queue.push_back(ack);
                }
                _ => {}
            }
        }
        p.regs[10] = guest_payload.len() as u64;
        Some(StepEvent::VsockPacketEmitted(VsockPacket {
            header: hdr,
            payload: wire_payload,
        }))
    }

    /// Formal R4.MvmLinuxContainer.
    fn verify_secret_placeholders_for_host(
        &self,
        target_host: &str,
        text_to_scan: &str,
    ) -> Result<bool, MvmError> {
        let mut secret_in_guest = false;
        for rule in self.host_secrets.values() {
            if !rule.raw_secret.is_empty() && self.guest_ram_contains(rule.raw_secret.as_bytes()) {
                secret_in_guest = true;
                break;
            }
        }
        if secret_in_guest {
            return Err(MvmError::BridgeRejected(
                BridgeError::GuestRamSecretContamination,
            ));
        }

        let github_domain_ok = matches!(
            target_host.to_ascii_lowercase().as_str(),
            "github.com" | "api.github.com"
        );
        let mut cursor = text_to_scan;
        let mut any_verified = false;
        while let Some(start) = cursor.find("{{MVM_SECRET:") {
            let after = &cursor[start + "{{MVM_SECRET:".len()..];
            let Some(end) = after.find("}}") else {
                return Err(MvmError::BridgeRejected(
                    BridgeError::MissingSecretPlaceholder,
                ));
            };
            let secret_name = &after[..end];
            let placeholder_present = !secret_name.is_empty();
            let rule_host_ok = self
                .host_secrets
                .get(secret_name)
                .map(|r| r.allowlisted_hosts.contains(target_host))
                .unwrap_or(true);
            let is_allowlisted = github_domain_ok && rule_host_ok;

            match admit_git_secret_substitution(
                secret_in_guest,
                placeholder_present,
                is_allowlisted,
            ) {
                GitSecretVerdict::SecretSubstitutionAdmitted => {
                    any_verified = true;
                }
                GitSecretVerdict::GuestRamSecretContamination => {
                    return Err(MvmError::BridgeRejected(
                        BridgeError::GuestRamSecretContamination,
                    ));
                }
                GitSecretVerdict::MissingSecretPlaceholder => {
                    return Err(MvmError::BridgeRejected(
                        BridgeError::MissingSecretPlaceholder,
                    ));
                }
                GitSecretVerdict::HostNotAllowlistedForSecret => {
                    return Err(MvmError::BridgeRejected(
                        BridgeError::HostNotAllowlistedForSecret,
                    ));
                }
            }
            cursor = &after[end + 2..];
        }
        Ok(any_verified)
    }

    /// Formal R4.MvmLinuxContainer.
    fn next_bridge_req_id(&mut self, prefix: &str) -> String {
        let seq = self.next_bridge_seq;
        self.next_bridge_seq = self.next_bridge_seq.saturating_add(1);
        format!("{prefix}-{seq}")
    }

    fn policy_str(&self) -> &'static str {
        match self.policy_mode {
            McpPolicyMode::Safe => "safe",
            McpPolicyMode::Confirm => "confirm",
            McpPolicyMode::Deny => "deny",
        }
    }

    fn emit_bridge_vsock_frame(&self, op_json: &[u8]) -> Result<VsockPacket, MvmError> {
        let mut h = Sha256::new();
        h.update(op_json);
        let dig = h.finalize();
        let clamped = if op_json.len() > SPIRALCORE_MAX_PAYLOAD {
            &op_json[..SPIRALCORE_MAX_PAYLOAD]
        } else {
            op_json
        };
        let wire = self.build_vsock_packet(
            SPIRALCORE_HOST_CID,
            SPIRALCORE_VSOCK_PORT,
            dig[0] & 63,
            dig[1] & 15,
            clamped,
        )?;
        let hdr = SpiralCoreHeader::from_bytes(&wire)
            .ok_or(MvmError::FrameRejected(FrameVerdict::RefusedInvalidCid))?;
        Ok(VsockPacket {
            header: hdr,
            payload: wire[16..].to_vec(),
        })
    }

    fn prepare_dns_bridge(
        &mut self,
        words: &[String],
        working_cwd: &str,
        redirect: Option<(String, bool)>,
    ) -> Result<MvmBridgePending, MvmError> {
        self.admit_outbound_transport(TransportKind::WssRelay)?;
        self.ensure_guest_ram_uncontaminated()?;

        let mut domain = String::new();
        let mut rtype = "A".to_string();
        for arg in &words[1..] {
            if arg.starts_with('-') || arg.starts_with('@') || arg.starts_with('+') {
                continue;
            }
            let up = arg.to_ascii_uppercase();
            if matches!(up.as_str(), "A" | "AAAA" | "TXT" | "MX" | "CNAME" | "NS")
                && !domain.is_empty()
            {
                rtype = up;
            } else if domain.is_empty() {
                domain = arg.clone();
            }
        }
        if domain.is_empty() {
            domain = "github.com".to_string();
        }
        if is_private_or_loopback_host(&domain) {
            return Err(MvmError::BridgeRejected(
                BridgeError::PrivateAddressBlocked(domain),
            ));
        }
        self.admit_network_target("cloudflare-dns.com")?;

        let req_id = self.next_bridge_req_id("mvm-dns");
        let bridge_req = serde_json::json!({
            "req_id": req_id,
            "spvs_magic": "SPVS",
            "src_cid": self.guest_cid,
            "dst_cid": 2,
            "policy": self.policy_str(),
            "op": "dns_query",
            "name": domain,
            "rr_type": rtype,
            "type": rtype,
            "params": {
                "name": domain,
                "type": rtype,
            }
        });
        let frame = self.emit_bridge_vsock_frame(bridge_req.to_string().as_bytes())?;
        let resume_state = serde_json::json!({
            "bridge_kind": "dns_query",
            "name": domain,
            "rtype": rtype,
            "working_cwd": working_cwd,
            "redirect_path": redirect.as_ref().map(|(p, _)| p.clone()),
            "redirect_append": redirect.as_ref().map(|(_, a)| *a).unwrap_or(false),
        });
        Ok(MvmBridgePending {
            bridge_req,
            resume_state,
            vsock_frames: vec![frame],
        })
    }

    fn prepare_http_bridge(
        &mut self,
        words: &[String],
        working_cwd: &str,
        redirect: Option<(String, bool)>,
    ) -> Result<MvmBridgePending, MvmError> {
        let mut method = "GET".to_string();
        let mut url = String::new();
        let mut headers = serde_json::Map::new();
        let mut body: Option<String> = None;
        let mut output_file: Option<String> = None;

        let mut i = 1usize;
        while i < words.len() {
            match words[i].as_str() {
                "-X" | "--request" if i + 1 < words.len() => {
                    method = words[i + 1].to_ascii_uppercase();
                    i += 2;
                }
                "-H" | "--header" if i + 1 < words.len() => {
                    if let Some((k, v)) = words[i + 1].split_once(':') {
                        headers.insert(
                            k.trim().to_string(),
                            serde_json::Value::String(v.trim().to_string()),
                        );
                    }
                    i += 2;
                }
                "-d" | "--data" | "--data-raw" if i + 1 < words.len() => {
                    body = Some(words[i + 1].clone());
                    if method == "GET" {
                        method = "POST".to_string();
                    }
                    i += 2;
                }
                "-o" | "-O" | "--output" if i + 1 < words.len() => {
                    output_file = Some(words[i + 1].clone());
                    i += 2;
                }
                "-I" | "--head" => {
                    method = "HEAD".to_string();
                    i += 1;
                }
                arg if !arg.starts_with('-') && url.is_empty() => {
                    url = arg.to_string();
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }

        let (scheme, host) = parse_url_scheme_and_host(&url);
        let transport = match scheme.as_str() {
            "https" => TransportKind::HttpsDirect,
            "wss" => TransportKind::WssRelay,
            "ws" => TransportKind::PlainWs,
            _ => TransportKind::PlainHttp,
        };
        self.admit_outbound_transport(transport)?;

        let combined_scan = format!(
            "{} {} {}",
            url,
            serde_json::Value::Object(headers.clone()),
            body.as_deref().unwrap_or("")
        );
        let secret_verified = self.verify_secret_placeholders_for_host(&host, &combined_scan)?;
        self.admit_network_target(&host)?;

        let req_id = self.next_bridge_req_id("mvm-http");
        let body_b64 = body.as_ref().map(|b| encode_base64_bytes(b.as_bytes()));
        let bridge_req = serde_json::json!({
            "req_id": req_id,
            "spvs_magic": "SPVS",
            "src_cid": self.guest_cid,
            "dst_cid": 2,
            "policy": self.policy_str(),
            "op": "http_request",
            "method": method,
            "url": url,
            "headers": headers,
            "body_base64": body_b64,
            "secret_placeholder_verified": secret_verified,
            "params": {
                "method": method,
                "url": url,
                "headers": headers,
                "body_b64": body_b64,
            }
        });
        let frame = self.emit_bridge_vsock_frame(bridge_req.to_string().as_bytes())?;
        let resume_state = serde_json::json!({
            "bridge_kind": "http_request",
            "url": url,
            "output_file": output_file,
            "working_cwd": working_cwd,
            "redirect_path": redirect.as_ref().map(|(p, _)| p.clone()),
            "redirect_append": redirect.as_ref().map(|(_, a)| *a).unwrap_or(false),
        });
        Ok(MvmBridgePending {
            bridge_req,
            resume_state,
            vsock_frames: vec![frame],
        })
    }

    /// Admit and build a `SpiralCore` (`CID 3 -> CID 2`) `web_search` bridge request (`R4-39`).
    pub(crate) fn prepare_web_search_bridge(
        &mut self,
        query: &str,
    ) -> Result<serde_json::Value, MvmError> {
        self.admit_outbound_transport(TransportKind::HttpsDirect)?;
        let secret_verified =
            self.verify_secret_placeholders_for_host("search.uor.foundation", query)?;
        self.admit_network_target("search.uor.foundation")?;
        let req_id = self.next_bridge_req_id("chat-web-search");
        let bridge_req = serde_json::json!({
            "req_id": req_id,
            "spvs_magic": "SPVS",
            "src_cid": self.guest_cid,
            "dst_cid": 2,
            "policy": self.policy_str(),
            "op": "web_search",
            "query": query,
            "limit": 5,
            "secret_placeholder_verified": secret_verified,
            "params": { "query": query, "limit": 5 }
        });
        let _ = self.emit_bridge_vsock_frame(bridge_req.to_string().as_bytes())?;
        Ok(bridge_req)
    }

    /// Admit and build a `SpiralCore` (`CID 3 -> CID 2`) `web_fetch` (`http_request`) bridge request (`R4-39`).
    pub(crate) fn prepare_web_fetch_bridge(
        &mut self,
        url: &str,
    ) -> Result<serde_json::Value, MvmError> {
        let (scheme, host) = parse_url_scheme_and_host(url);
        let transport = match scheme.as_str() {
            "https" => TransportKind::HttpsDirect,
            "wss" => TransportKind::WssRelay,
            "ws" => TransportKind::PlainWs,
            _ => TransportKind::PlainHttp,
        };
        self.admit_outbound_transport(transport)?;
        let secret_verified = self.verify_secret_placeholders_for_host(&host, url)?;
        self.admit_network_target(&host)?;
        let req_id = self.next_bridge_req_id("chat-web-fetch");
        let bridge_req = serde_json::json!({
            "req_id": req_id,
            "spvs_magic": "SPVS",
            "src_cid": self.guest_cid,
            "dst_cid": 2,
            "policy": self.policy_str(),
            "op": "http_request",
            "method": "GET",
            "url": url,
            "secret_placeholder_verified": secret_verified,
            "params": { "method": "GET", "url": url }
        });
        let _ = self.emit_bridge_vsock_frame(bridge_req.to_string().as_bytes())?;
        Ok(bridge_req)
    }

    fn exec_pkg_command(
        &mut self,
        words: &[String],
        working_cwd: &str,
        redirect: Option<(String, bool)>,
    ) -> Result<MvmInteractiveOutcome, MvmError> {
        let sub = words.get(1).map(String::as_str).unwrap_or("list");
        match sub {
            "list" => {
                let mut out = String::from("Installed RV64 ELF64 packages (/usr/local/bin):\n");
                if self.installed_packages.is_empty() {
                    out.push_str("  (none installed; base busybox applets active in /bin)\n");
                } else {
                    for meta in self.installed_packages.values() {
                        out.push_str(&format!(
                            "  {} ({} bytes, sha256:{})\n",
                            meta.name,
                            meta.byte_len,
                            &meta.sha256[..12.min(meta.sha256.len())]
                        ));
                    }
                }
                if let Some((redir, append)) = redirect {
                    self.write_redirected_bytes(working_cwd, &redir, out.as_bytes(), append, None)
                        .map_err(MvmError::CpuTrap)?;
                    out.clear();
                }
                Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                    exit_code: 0,
                    stdout: out,
                    stderr: String::new(),
                    instructions_retired: 0,
                    vsock_frames: Vec::new(),
                    compiled_elf_machine: None,
                    cow_state_sha256: self.cow_state_sha256(),
                }))
            }
            "search" | "update" => {
                let query = words
                    .get(2)
                    .map(|s| s.to_ascii_lowercase())
                    .unwrap_or_default();
                let catalog = [
                    (
                        "jq",
                        "JSON parser, pretty-printer, and path filter (static ELF64 riscv64)",
                    ),
                    (
                        "rg",
                        "Recursive substring file search across directories (static ELF64 riscv64)",
                    ),
                    (
                        "tree",
                        "Recursive ASCII directory hierarchy viewer (static ELF64 riscv64)",
                    ),
                    (
                        "xxd",
                        "Canonical 16-byte hexadecimal and ASCII dump (static ELF64 riscv64)",
                    ),
                    (
                        "sha256sum",
                        "FIPS 180-4 SHA-256 digest utility (static ELF64 riscv64)",
                    ),
                    (
                        "bc",
                        "64-bit integer arithmetic and variable calculator (static ELF64 riscv64)",
                    ),
                    ("wc", "Line, word, and byte counter (static ELF64 riscv64)"),
                    (
                        "head",
                        "Output leading lines or bytes of files (static ELF64 riscv64)",
                    ),
                    (
                        "tail",
                        "Output trailing lines or bytes of files (static ELF64 riscv64)",
                    ),
                    (
                        "http",
                        "Zero-NIC AF_VSOCK SpiralCore HTTP egress client (static ELF64 riscv64)",
                    ),
                ];
                let mut out = String::new();
                for (name, desc) in catalog {
                    if query.is_empty()
                        || name.contains(&query)
                        || desc.to_ascii_lowercase().contains(&query)
                    {
                        out.push_str(&format!("{name}\t-\t{desc}\n"));
                    }
                }
                if let Some((redir, append)) = redirect {
                    self.write_redirected_bytes(working_cwd, &redir, out.as_bytes(), append, None)
                        .map_err(MvmError::CpuTrap)?;
                    out.clear();
                }
                Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                    exit_code: 0,
                    stdout: out,
                    stderr: String::new(),
                    instructions_retired: 0,
                    vsock_frames: Vec::new(),
                    compiled_elf_machine: None,
                    cow_state_sha256: self.cow_state_sha256(),
                }))
            }
            "install" => {
                let target = words.get(2).map(String::as_str).unwrap_or("");
                if target.is_empty() {
                    return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                        exit_code: 1,
                        stdout: String::new(),
                        stderr: "pkg install: missing package name or HTTPS URL\n".to_string(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    }));
                }
                let mut expected_sha = String::new();
                let mut custom_name: Option<String> = None;
                let mut idx = 3usize;
                while idx < words.len() {
                    if words[idx] == "--sha256" && idx + 1 < words.len() {
                        expected_sha = words[idx + 1].clone();
                        idx += 2;
                    } else if words[idx] == "--name" && idx + 1 < words.len() {
                        custom_name = Some(words[idx + 1].clone());
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }

                let (source_url, pkg_name, transport) = if target.contains("://") {
                    let (scheme, _) = parse_url_scheme_and_host(target);
                    let tr = if scheme == "https" {
                        TransportKind::HttpsDirect
                    } else {
                        TransportKind::PlainHttp
                    };
                    let inferred = custom_name.unwrap_or_else(|| {
                        target
                            .rsplit('/')
                            .next()
                            .unwrap_or("pkg-bin")
                            .split('?')
                            .next()
                            .unwrap_or("pkg-bin")
                            .to_string()
                    });
                    (target.to_string(), inferred, tr)
                } else {
                    let name = custom_name.unwrap_or_else(|| target.to_string());
                    (
                        format!("https://pkg.uor.foundation/riscv64/{name}.elf"),
                        name,
                        TransportKind::HttpsDirect,
                    )
                };

                self.admit_outbound_transport(transport)?;
                if target.contains("://") {
                    let (_, pkg_host) = parse_url_scheme_and_host(target);
                    self.admit_network_target(&pkg_host)?;
                }
                self.ensure_guest_ram_uncontaminated()?;

                let req_id = self.next_bridge_req_id("mvm-pkg");
                let bridge_req = serde_json::json!({
                    "req_id": req_id,
                    "spvs_magic": "SPVS",
                    "src_cid": self.guest_cid,
                    "dst_cid": 2,
                    "policy": self.policy_str(),
                    "op": "pkg_fetch",
                    "subcommand": "install",
                    "name": pkg_name,
                    "pkg_name": pkg_name,
                    "expected_sha256": expected_sha,
                    "params": {
                        "pkg_name": pkg_name,
                        "url": source_url,
                        "expected_sha256": expected_sha,
                        "max_bytes": 4 * 1024 * 1024,
                    }
                });
                let frame = self.emit_bridge_vsock_frame(bridge_req.to_string().as_bytes())?;
                let resume_state = serde_json::json!({
                    "bridge_kind": "pkg_fetch",
                    "pkg_name": pkg_name,
                    "source_url": source_url,
                    "expected_sha256": expected_sha,
                    "working_cwd": working_cwd,
                    "redirect_path": redirect.as_ref().map(|(p, _)| p.clone()),
                    "redirect_append": redirect.as_ref().map(|(_, a)| *a).unwrap_or(false),
                });
                Ok(MvmInteractiveOutcome::NeedsBridge(MvmBridgePending {
                    bridge_req,
                    resume_state,
                    vsock_frames: vec![frame],
                }))
            }
            other => Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                exit_code: 1,
                stdout: String::new(),
                stderr: format!("pkg: unknown subcommand '{other}'\n"),
                instructions_retired: 0,
                vsock_frames: Vec::new(),
                compiled_elf_machine: None,
                cow_state_sha256: self.cow_state_sha256(),
            })),
        }
    }

    /// Formal R4.MvmLinuxContainer.
    fn find_git_root(&self, cwd: &str) -> Option<String> {
        let mut cur = cwd.to_string();
        loop {
            let state_path = if cur == "/" {
                "/.git/uor_state.json".to_string()
            } else {
                format!("{cur}/.git/uor_state.json")
            };
            if self.inodes.contains_key(&state_path) {
                return Some(cur);
            }
            if cur == "/" || cur.is_empty() {
                return None;
            }
            let pos = cur.rfind('/')?;
            cur = if pos == 0 {
                "/".to_string()
            } else {
                cur[..pos].to_string()
            };
        }
    }

    fn load_git_state(&self, repo_root: &str) -> Option<GitRepoState> {
        let p = format!("{repo_root}/.git/uor_state.json");
        let bytes = self.read_guest_file(&p)?;
        serde_json::from_slice(&bytes).ok()
    }

    fn save_git_state(&mut self, repo_root: &str, state: &GitRepoState) -> Result<(), MvmError> {
        let p = format!("{repo_root}/.git/uor_state.json");
        let bytes = serde_json::to_vec(state).unwrap_or_default();
        self.write_guest_file(&p, &bytes)?;
        let head_ref = format!("ref: refs/heads/{}\n", state.branch);
        self.write_guest_file(&format!("{repo_root}/.git/HEAD"), head_ref.as_bytes())?;
        if let Some(ref oid) = state.head_oid {
            self.write_guest_file(
                &format!("{repo_root}/.git/refs/heads/{}", state.branch),
                format!("{oid}\n").as_bytes(),
            )?;
        }
        Ok(())
    }

    /// Formal R4.MvmLinuxContainer.
    fn collect_git_working_tree(&self, repo_root: &str) -> BTreeMap<String, String> {
        let mut tree = BTreeMap::new();
        let prefix = if repo_root == "/" {
            "/".to_string()
        } else {
            format!("{repo_root}/")
        };
        for path in self.inodes.keys() {
            if let Some(rel) = path.strip_prefix(&prefix) {
                if rel.is_empty() || is_git_internal_rel_path(rel) {
                    continue;
                }
                if let Some(bytes) = self.read_guest_file(path) {
                    if !bytes.starts_with(b"\x7fELF") {
                        if let Ok(s) = String::from_utf8(bytes) {
                            tree.insert(rel.to_string(), s);
                        }
                    }
                }
            }
        }
        tree
    }

    fn admit_git_remote_request(&self, repo_url: &str) -> Result<(Option<String>, bool), MvmError> {
        let (scheme, host) = parse_url_scheme_and_host(repo_url);
        let transport = if scheme == "https" {
            TransportKind::HttpsDirect
        } else {
            TransportKind::PlainHttp
        };
        self.admit_outbound_transport(transport)?;
        let has_github_secret = self.host_secrets.contains_key("GITHUB_TOKEN");
        let auth_placeholder = if has_github_secret {
            Some("{{MVM_SECRET:GITHUB_TOKEN}}".to_string())
        } else if repo_url.contains("{{MVM_SECRET:") {
            Some(repo_url.to_string())
        } else {
            None
        };
        let secret_verified = if let Some(ref ph) = auth_placeholder {
            self.verify_secret_placeholders_for_host(&host, ph)?
        } else {
            self.ensure_guest_ram_uncontaminated()?;
            false
        };
        self.admit_network_target(&host)?;
        Ok((auth_placeholder, secret_verified))
    }

    fn exec_git_command(
        &mut self,
        words: &[String],
        working_cwd: &mut String,
        redirect: Option<(String, bool)>,
    ) -> Result<MvmInteractiveOutcome, MvmError> {
        let sub = words.get(1).map(String::as_str).unwrap_or("status");
        match sub {
            "init" => {
                let target_dir = words
                    .get(2)
                    .filter(|s| !s.starts_with('-'))
                    .map(|d| Self::normalize_guest_path(working_cwd, d))
                    .unwrap_or_else(|| working_cwd.clone());
                let state = GitRepoState {
                    branch: "main".to_string(),
                    remote_url: None,
                    head_oid: None,
                    staged: BTreeMap::new(),
                    head_tree: BTreeMap::new(),
                    commits: Vec::new(),
                };
                self.save_git_state(&target_dir, &state)?;
                let mut out = format!("Initialized empty Git repository in {target_dir}/.git/\n");
                if let Some((redir, append)) = redirect {
                    self.write_redirected_bytes(working_cwd, &redir, out.as_bytes(), append, None)
                        .map_err(MvmError::CpuTrap)?;
                    out.clear();
                }
                Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                    exit_code: 0,
                    stdout: out,
                    stderr: String::new(),
                    instructions_retired: 0,
                    vsock_frames: Vec::new(),
                    compiled_elf_machine: None,
                    cow_state_sha256: self.cow_state_sha256(),
                }))
            }
            "status" => {
                let Some(repo_root) = self.find_git_root(working_cwd) else {
                    return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                        exit_code: 128,
                        stdout: String::new(),
                        stderr:
                            "fatal: not a git repository (or any of the parent directories): .git\n"
                                .to_string(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    }));
                };
                let Some(state) = self.load_git_state(&repo_root) else {
                    return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                        exit_code: 128,
                        stdout: String::new(),
                        stderr: "fatal: corrupt or missing .git/uor_state.json\n".to_string(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    }));
                };
                let wt = self.collect_git_working_tree(&repo_root);
                let mut out = format!("On branch {}\n", state.branch);
                if !state.staged.is_empty() {
                    out.push_str("Changes to be committed:\n");
                    for (k, v) in &state.staged {
                        let status_lbl = match (state.head_tree.contains_key(k), v.is_some()) {
                            (false, true) => "new file",
                            (true, true) => "modified",
                            (_, false) => "deleted",
                        };
                        out.push_str(&format!("\t{status_lbl}:   {k}\n"));
                    }
                }

                let mut effective_index = state.head_tree.clone();
                for (k, v) in &state.staged {
                    match v {
                        Some(c) => {
                            effective_index.insert(k.clone(), c.clone());
                        }
                        None => {
                            effective_index.remove(k);
                        }
                    }
                }

                let mut unstaged = Vec::new();
                for (k, idx_content) in &effective_index {
                    match wt.get(k) {
                        Some(cur) if cur != idx_content => {
                            unstaged.push(format!("\tmodified:   {k}\n"))
                        }
                        None => unstaged.push(format!("\tdeleted:    {k}\n")),
                        _ => {}
                    }
                }
                if !unstaged.is_empty() {
                    out.push_str("Changes not staged for commit:\n");
                    for line in &unstaged {
                        out.push_str(line);
                    }
                }

                let mut untracked = Vec::new();
                for k in wt.keys() {
                    if !effective_index.contains_key(k) {
                        untracked.push(format!("\t{k}\n"));
                    }
                }
                if !untracked.is_empty() {
                    out.push_str("Untracked files:\n");
                    for line in &untracked {
                        out.push_str(line);
                    }
                }
                if state.staged.is_empty() && unstaged.is_empty() && untracked.is_empty() {
                    out.push_str("nothing to commit, working tree clean\n");
                }
                if let Some((redir, append)) = redirect {
                    self.write_redirected_bytes(working_cwd, &redir, out.as_bytes(), append, None)
                        .map_err(MvmError::CpuTrap)?;
                    out.clear();
                }
                Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                    exit_code: 0,
                    stdout: out,
                    stderr: String::new(),
                    instructions_retired: 0,
                    vsock_frames: Vec::new(),
                    compiled_elf_machine: None,
                    cow_state_sha256: self.cow_state_sha256(),
                }))
            }
            "add" => {
                let Some(repo_root) = self.find_git_root(working_cwd) else {
                    return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                        exit_code: 128,
                        stdout: String::new(),
                        stderr: "fatal: not a git repository\n".to_string(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    }));
                };
                let Some(mut state) = self.load_git_state(&repo_root) else {
                    return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                        exit_code: 128,
                        stdout: String::new(),
                        stderr: "fatal: corrupt or missing .git/uor_state.json\n".to_string(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    }));
                };
                let wt = self.collect_git_working_tree(&repo_root);
                let specs = &words[2..];
                let stage_all = specs
                    .iter()
                    .any(|s| s == "." || s == "-A" || s == "--all" || s == "-a");
                if stage_all {
                    for (k, v) in &wt {
                        if state.head_tree.get(k) != Some(v) {
                            state.staged.insert(k.clone(), Some(v.clone()));
                        } else {
                            state.staged.remove(k);
                        }
                    }
                    for k in state.head_tree.keys() {
                        if !wt.contains_key(k) {
                            state.staged.insert(k.clone(), None);
                        }
                    }
                } else {
                    let prefix = format!("{repo_root}/");
                    for raw_spec in specs {
                        if raw_spec.starts_with('-') {
                            continue;
                        }
                        let norm = Self::normalize_guest_path(working_cwd, raw_spec);
                        if let Some(rel) = norm.strip_prefix(&prefix) {
                            if let Some(v) = wt.get(rel) {
                                state.staged.insert(rel.to_string(), Some(v.clone()));
                            } else if state.head_tree.contains_key(rel) {
                                state.staged.insert(rel.to_string(), None);
                            }
                        }
                    }
                }
                self.save_git_state(&repo_root, &state)?;
                Ok(MvmInteractiveOutcome::Completed(self.empty_outcome()))
            }
            "commit" => {
                let Some(repo_root) = self.find_git_root(working_cwd) else {
                    return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                        exit_code: 128,
                        stdout: String::new(),
                        stderr: "fatal: not a git repository\n".to_string(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    }));
                };
                let Some(mut state) = self.load_git_state(&repo_root) else {
                    return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                        exit_code: 128,
                        stdout: String::new(),
                        stderr: "fatal: corrupt or missing .git/uor_state.json\n".to_string(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    }));
                };
                let mut msg = String::new();
                let mut auto_add = false;
                let mut idx = 2usize;
                while idx < words.len() {
                    if (words[idx] == "-m" || words[idx] == "-am") && idx + 1 < words.len() {
                        if words[idx] == "-am" {
                            auto_add = true;
                        }
                        msg = words[idx + 1].clone();
                        idx += 2;
                    } else if words[idx] == "-a" {
                        auto_add = true;
                        idx += 1;
                    } else {
                        idx += 1;
                    }
                }
                if auto_add {
                    let wt = self.collect_git_working_tree(&repo_root);
                    for (k, c) in &wt {
                        if state.head_tree.contains_key(k) && state.head_tree.get(k) != Some(c) {
                            state.staged.insert(k.clone(), Some(c.clone()));
                        }
                    }
                }
                if state.staged.is_empty() {
                    return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                        exit_code: 1,
                        stdout: "nothing to commit, working tree clean\n".to_string(),
                        stderr: String::new(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    }));
                }
                let changed_count = state.staged.len();
                for (k, v) in std::mem::take(&mut state.staged) {
                    match v {
                        Some(c) => {
                            state.head_tree.insert(k, c);
                        }
                        None => {
                            state.head_tree.remove(&k);
                        }
                    }
                }
                let commit_seed = format!(
                    "commit|parent={:?}|msg={}|tree={:?}|n={}",
                    state.head_oid,
                    msg,
                    state.head_tree,
                    state.commits.len()
                );
                let full_sha = sha256_hex_bytes(commit_seed.as_bytes());
                let oid = full_sha[..40].to_string();
                let rec = GitCommitRecord {
                    oid: oid.clone(),
                    parent_oid: state.head_oid.clone(),
                    message: msg.clone(),
                    tree: state.head_tree.clone(),
                };
                state.head_oid = Some(oid.clone());
                state.commits.push(rec);
                self.save_git_state(&repo_root, &state)?;
                let mut out = format!(
                    "[{} {}] {}\n {} file(s) changed\n",
                    state.branch,
                    &oid[..7],
                    msg,
                    changed_count
                );
                if let Some((redir, append)) = redirect {
                    self.write_redirected_bytes(working_cwd, &redir, out.as_bytes(), append, None)
                        .map_err(MvmError::CpuTrap)?;
                    out.clear();
                }
                Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                    exit_code: 0,
                    stdout: out,
                    stderr: String::new(),
                    instructions_retired: 0,
                    vsock_frames: Vec::new(),
                    compiled_elf_machine: None,
                    cow_state_sha256: self.cow_state_sha256(),
                }))
            }
            "log" => {
                let Some(repo_root) = self.find_git_root(working_cwd) else {
                    return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                        exit_code: 128,
                        stdout: String::new(),
                        stderr: "fatal: not a git repository\n".to_string(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    }));
                };
                let Some(state) = self.load_git_state(&repo_root) else {
                    return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                        exit_code: 128,
                        stdout: String::new(),
                        stderr: "fatal: corrupt or missing .git/uor_state.json\n".to_string(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    }));
                };
                let mut out = String::new();
                for rec in state.commits.iter().rev() {
                    out.push_str(&format!("commit {}\n\n    {}\n\n", rec.oid, rec.message));
                }
                if let Some((redir, append)) = redirect {
                    self.write_redirected_bytes(working_cwd, &redir, out.as_bytes(), append, None)
                        .map_err(MvmError::CpuTrap)?;
                    out.clear();
                }
                Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                    exit_code: 0,
                    stdout: out,
                    stderr: String::new(),
                    instructions_retired: 0,
                    vsock_frames: Vec::new(),
                    compiled_elf_machine: None,
                    cow_state_sha256: self.cow_state_sha256(),
                }))
            }
            "diff" => {
                let Some(repo_root) = self.find_git_root(working_cwd) else {
                    return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                        exit_code: 128,
                        stdout: String::new(),
                        stderr: "fatal: not a git repository\n".to_string(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    }));
                };
                let Some(state) = self.load_git_state(&repo_root) else {
                    return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                        exit_code: 128,
                        stdout: String::new(),
                        stderr: "fatal: corrupt or missing .git/uor_state.json\n".to_string(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    }));
                };
                let wt = self.collect_git_working_tree(&repo_root);
                let mut base = state.head_tree.clone();
                for (k, v) in &state.staged {
                    match v {
                        Some(c) => {
                            base.insert(k.clone(), c.clone());
                        }
                        None => {
                            base.remove(k);
                        }
                    }
                }
                let mut out = String::new();
                for (k, old_c) in &base {
                    let Some(new_c_opt) = wt
                        .get(k)
                        .map(String::as_str)
                        .or(Some(""))
                        .filter(|&n| n != old_c)
                    else {
                        continue;
                    };
                    let old_lines: Vec<&str> = old_c.lines().collect();
                    let new_lines: Vec<&str> = new_c_opt.lines().collect();
                    out.push_str(&format!(
                        "diff --git a/{k} b/{k}\n--- a/{k}\n+++ b/{k}\n@@ -1,{} +1,{} @@\n",
                        old_lines.len(),
                        new_lines.len()
                    ));
                    for idx in 0..old_lines.len().max(new_lines.len()) {
                        match (old_lines.get(idx), new_lines.get(idx)) {
                            (Some(a), Some(b)) if a == b => out.push_str(&format!(" {a}\n")),
                            (Some(a), Some(b)) => out.push_str(&format!("-{a}\n+{b}\n")),
                            (Some(a), None) => out.push_str(&format!("-{a}\n")),
                            (None, Some(b)) => out.push_str(&format!("+{b}\n")),
                            (None, None) => {}
                        }
                    }
                }
                if let Some((redir, append)) = redirect {
                    self.write_redirected_bytes(working_cwd, &redir, out.as_bytes(), append, None)
                        .map_err(MvmError::CpuTrap)?;
                    out.clear();
                }
                Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                    exit_code: 0,
                    stdout: out,
                    stderr: String::new(),
                    instructions_retired: 0,
                    vsock_frames: Vec::new(),
                    compiled_elf_machine: None,
                    cow_state_sha256: self.cow_state_sha256(),
                }))
            }
            "remote" => {
                let Some(repo_root) = self.find_git_root(working_cwd) else {
                    return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                        exit_code: 128,
                        stdout: String::new(),
                        stderr: "fatal: not a git repository\n".to_string(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    }));
                };
                let Some(mut state) = self.load_git_state(&repo_root) else {
                    return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                        exit_code: 128,
                        stdout: String::new(),
                        stderr: "fatal: corrupt or missing .git/uor_state.json\n".to_string(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    }));
                };
                if words.get(2).map(String::as_str) == Some("add") && words.len() >= 5 {
                    state.remote_url = Some(words[4].clone());
                    self.save_git_state(&repo_root, &state)?;
                    return Ok(MvmInteractiveOutcome::Completed(self.empty_outcome()));
                }
                let out = match state.remote_url {
                    Some(u) => format!("origin\t{u} (fetch)\norigin\t{u} (push)\n"),
                    None => String::new(),
                };
                Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                    exit_code: 0,
                    stdout: out,
                    stderr: String::new(),
                    instructions_retired: 0,
                    vsock_frames: Vec::new(),
                    compiled_elf_machine: None,
                    cow_state_sha256: self.cow_state_sha256(),
                }))
            }
            "clone" | "pull" | "push" => {
                let (repo_url, target_dir) = if sub == "clone" {
                    let url = words
                        .iter()
                        .skip(2)
                        .find(|a| !a.starts_with('-'))
                        .cloned()
                        .unwrap_or_default();
                    let explicit_dir = words
                        .iter()
                        .skip(2)
                        .filter(|a| !a.starts_with('-'))
                        .nth(1)
                        .cloned();
                    let dir_name = explicit_dir.unwrap_or_else(|| {
                        url.trim_end_matches('/')
                            .rsplit('/')
                            .next()
                            .unwrap_or("repo")
                            .trim_end_matches(".git")
                            .to_string()
                    });
                    let norm_dir = Self::normalize_guest_path(working_cwd, &dir_name);
                    (url, norm_dir)
                } else {
                    let Some(repo_root) = self.find_git_root(working_cwd) else {
                        return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                            exit_code: 128,
                            stdout: String::new(),
                            stderr: "fatal: not a git repository\n".to_string(),
                            instructions_retired: 0,
                            vsock_frames: Vec::new(),
                            compiled_elf_machine: None,
                            cow_state_sha256: self.cow_state_sha256(),
                        }));
                    };
                    let Some(state) = self.load_git_state(&repo_root) else {
                    return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                        exit_code: 128,
                        stdout: String::new(),
                        stderr: "fatal: corrupt or missing .git/uor_state.json\n".to_string(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    }));
                };
                    let explicit_url = words.iter().skip(2).find(|a| a.contains("://")).cloned();
                    let url = explicit_url.or(state.remote_url).unwrap_or_else(|| {
                        "https://github.com/uor-foundation/repo.git".to_string()
                    });
                    (url, repo_root)
                };

                let (auth_placeholder, secret_verified) =
                    self.admit_git_remote_request(&repo_url)?;

                let (git_branch, push_files, push_msg) = if sub == "push" {
                    if let Some(st) = self.load_git_state(&target_dir) {
                        let tree = self.collect_git_working_tree(&target_dir);
                        let files_arr: Vec<serde_json::Value> = tree
                            .iter()
                            .map(|(p, c)| serde_json::json!({ "path": p, "content": c }))
                            .collect();
                        let last_msg = st
                            .commits
                            .last()
                            .map(|c| c.message.clone())
                            .unwrap_or_else(|| "Update from UOR-R4 Connected Studio".to_string());
                        (st.branch, Some(files_arr), Some(last_msg))
                    } else {
                        ("main".to_string(), None, None)
                    }
                } else {
                    let b = self
                        .load_git_state(&target_dir)
                        .map(|s| s.branch)
                        .unwrap_or_else(|| "main".to_string());
                    (b, None, None)
                };
                let service_q = if sub == "push" {
                    "info/refs?service=git-receive-pack"
                } else {
                    "info/refs?service=git-upload-pack"
                };
                let req_id = self.next_bridge_req_id("mvm-git");
                let bridge_req = serde_json::json!({
                    "req_id": req_id,
                    "spvs_magic": "SPVS",
                    "src_cid": self.guest_cid,
                    "dst_cid": 2,
                    "policy": self.policy_str(),
                    "op": "git_smart_http",
                    "subcommand": sub,
                    "git_action": sub,
                    "repo_url": repo_url,
                    "branch": git_branch,
                    "files": push_files,
                    "message": push_msg,
                    "target_dir": target_dir,
                    "secret_placeholder_verified": secret_verified,
                    "params": {
                        "repo_url": repo_url,
                        "path_and_query": service_q,
                        "method": "GET",
                        "auth_bearer": auth_placeholder,
                    }
                });
                let frame = self.emit_bridge_vsock_frame(bridge_req.to_string().as_bytes())?;
                let resume_state = serde_json::json!({
                    "bridge_kind": "git_smart_http",
                    "git_action": sub,
                    "repo_url": repo_url,
                    "target_dir": target_dir,
                    "working_cwd": working_cwd,
                    "redirect_path": redirect.as_ref().map(|(p, _)| p.clone()),
                    "redirect_append": redirect.as_ref().map(|(_, a)| *a).unwrap_or(false),
                });
                Ok(MvmInteractiveOutcome::NeedsBridge(MvmBridgePending {
                    bridge_req,
                    resume_state,
                    vsock_frames: vec![frame],
                }))
            }
            "branch" => {
                let Some(repo_root) = self.find_git_root(working_cwd) else {
                    return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                        exit_code: 128,
                        stdout: String::new(),
                        stderr: "fatal: not a git repository\n".to_string(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    }));
                };
                let Some(mut state) = self.load_git_state(&repo_root) else {
                    return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                        exit_code: 128,
                        stdout: String::new(),
                        stderr: "fatal: corrupt or missing .git/uor_state.json\n".to_string(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    }));
                };
                let mut wants_list = false;
                let mut positional = Vec::new();
                for arg in words.iter().skip(2) {
                    if arg.starts_with('-') {
                        if matches!(
                            arg.as_str(),
                            "-a" | "--all" | "-l" | "--list" | "-v" | "-vv" | "-r" | "--remotes"
                        ) {
                            wants_list = true;
                        } else {
                            return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                                exit_code: 1,
                                stdout: String::new(),
                                stderr: format!(
                                    "git branch: unsupported flag '{arg}'; use `git branch` to list or `git branch <new> [base]` to create\n"
                                ),
                                instructions_retired: 0,
                                vsock_frames: Vec::new(),
                                compiled_elf_machine: None,
                                cow_state_sha256: self.cow_state_sha256(),
                            }));
                        }
                    } else {
                        positional.push(arg.clone());
                    }
                }
                if wants_list && !positional.is_empty() {
                    return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                        exit_code: 1,
                        stdout: String::new(),
                        stderr: "git branch: list flags cannot be combined with branch name\n"
                            .to_string(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    }));
                }
                if wants_list || positional.is_empty() {
                    let mut out = format!("* {}\n", state.branch);
                    if let Some((redir, append)) = redirect {
                        self.write_redirected_bytes(
                            working_cwd,
                            &redir,
                            out.as_bytes(),
                            append,
                            None,
                        )
                        .map_err(MvmError::CpuTrap)?;
                        out.clear();
                    }
                    return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                        exit_code: 0,
                        stdout: out,
                        stderr: String::new(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    }));
                }
                let branch_name = positional[0].clone();
                let requested_base = positional.get(1).cloned();
                if let Some(ref repo_url) = state.remote_url {
                    let (auth_placeholder, secret_verified) =
                        self.admit_git_remote_request(repo_url)?;
                    let base_branch = requested_base.unwrap_or_else(|| state.branch.clone());
                    let req_id = self.next_bridge_req_id("mvm-git-branch");
                    let bridge_req = serde_json::json!({
                        "req_id": req_id,
                        "spvs_magic": "SPVS",
                        "src_cid": self.guest_cid,
                        "dst_cid": 2,
                        "policy": self.policy_str(),
                        "op": "git_smart_http",
                        "subcommand": "create_branch",
                        "git_action": "create_branch",
                        "repo_url": repo_url,
                        "branch": branch_name,
                        "base_branch": base_branch,
                        "target_dir": repo_root,
                        "secret_placeholder_verified": secret_verified,
                        "params": {
                            "repo_url": repo_url,
                            "branch": branch_name,
                            "base_branch": base_branch,
                            "auth_bearer": auth_placeholder,
                        }
                    });
                    let frame = self.emit_bridge_vsock_frame(bridge_req.to_string().as_bytes())?;
                    let resume_state = serde_json::json!({
                        "bridge_kind": "git_smart_http",
                        "git_action": "create_branch",
                        "repo_url": repo_url,
                        "target_dir": repo_root,
                        "working_cwd": working_cwd,
                        "redirect_path": redirect.as_ref().map(|(p, _)| p.clone()),
                        "redirect_append": redirect.as_ref().map(|(_, a)| *a).unwrap_or(false),
                    });
                    return Ok(MvmInteractiveOutcome::NeedsBridge(MvmBridgePending {
                        bridge_req,
                        resume_state,
                        vsock_frames: vec![frame],
                    }));
                }
                state.branch = branch_name.clone();
                self.save_git_state(&repo_root, &state)?;
                let out = format!("Switched to branch '{branch_name}'\n");
                Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                    exit_code: 0,
                    stdout: out,
                    stderr: String::new(),
                    instructions_retired: 0,
                    vsock_frames: Vec::new(),
                    compiled_elf_machine: None,
                    cow_state_sha256: self.cow_state_sha256(),
                }))
            }
            "pr" => {
                let Some(repo_root) = self.find_git_root(working_cwd) else {
                    return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                        exit_code: 128,
                        stdout: String::new(),
                        stderr: "fatal: not a git repository\n".to_string(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    }));
                };
                let Some(state) = self.load_git_state(&repo_root) else {
                    return Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                        exit_code: 128,
                        stdout: String::new(),
                        stderr: "fatal: corrupt or missing .git/uor_state.json\n".to_string(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                        cow_state_sha256: self.cow_state_sha256(),
                    }));
                };
                let mut title = String::new();
                let mut head_branch = state.branch.clone();
                let mut base_branch = "main".to_string();
                let mut pr_body = String::new();
                let mut explicit_repo = None::<String>;
                let mut idx = 2usize;
                while idx < words.len() {
                    match words[idx].as_str() {
                        "-t" | "--title" if idx + 1 < words.len() => {
                            title = words[idx + 1].clone();
                            idx += 2;
                        }
                        "-H" | "--head" if idx + 1 < words.len() => {
                            head_branch = words[idx + 1].clone();
                            idx += 2;
                        }
                        "-b" | "--base" if idx + 1 < words.len() => {
                            base_branch = words[idx + 1].clone();
                            idx += 2;
                        }
                        "-m" | "--body" if idx + 1 < words.len() => {
                            pr_body = words[idx + 1].clone();
                            idx += 2;
                        }
                        "-R" | "--repo" if idx + 1 < words.len() => {
                            explicit_repo = Some(words[idx + 1].clone());
                            idx += 2;
                        }
                        "create" => {
                            idx += 1;
                        }
                        arg if !arg.starts_with('-') && title.is_empty() => {
                            title = arg.to_string();
                            idx += 1;
                        }
                        _ => {
                            idx += 1;
                        }
                    }
                }
                if title.is_empty() {
                    title = format!("Update {head_branch}");
                }
                let repo_url = explicit_repo
                    .or(state.remote_url)
                    .unwrap_or_else(|| "https://github.com/uor-foundation/repo.git".to_string());
                let (auth_placeholder, secret_verified) =
                    self.admit_git_remote_request(&repo_url)?;
                let req_id = self.next_bridge_req_id("mvm-git-pr");
                let bridge_req = serde_json::json!({
                    "req_id": req_id,
                    "spvs_magic": "SPVS",
                    "src_cid": self.guest_cid,
                    "dst_cid": 2,
                    "policy": self.policy_str(),
                    "op": "git_smart_http",
                    "subcommand": "create_pr",
                    "git_action": "create_pr",
                    "repo_url": repo_url,
                    "branch": head_branch,
                    "head": head_branch,
                    "base": base_branch,
                    "title": title,
                    "body": pr_body,
                    "target_dir": repo_root,
                    "secret_placeholder_verified": secret_verified,
                    "params": {
                        "repo_url": repo_url,
                        "head": head_branch,
                        "base": base_branch,
                        "title": title,
                        "body": pr_body,
                        "auth_bearer": auth_placeholder,
                    }
                });
                let frame = self.emit_bridge_vsock_frame(bridge_req.to_string().as_bytes())?;
                let resume_state = serde_json::json!({
                    "bridge_kind": "git_smart_http",
                    "git_action": "create_pr",
                    "repo_url": repo_url,
                    "target_dir": repo_root,
                    "working_cwd": working_cwd,
                    "redirect_path": redirect.as_ref().map(|(p, _)| p.clone()),
                    "redirect_append": redirect.as_ref().map(|(_, a)| *a).unwrap_or(false),
                });
                Ok(MvmInteractiveOutcome::NeedsBridge(MvmBridgePending {
                    bridge_req,
                    resume_state,
                    vsock_frames: vec![frame],
                }))
            }
            other => Ok(MvmInteractiveOutcome::Completed(MvmExecOutcome {
                exit_code: 1,
                stdout: String::new(),
                stderr: format!("git: '{other}' is not a git command.\n"),
                instructions_retired: 0,
                vsock_frames: Vec::new(),
                compiled_elf_machine: None,
                cow_state_sha256: self.cow_state_sha256(),
            })),
        }
    }

    fn complete_git_bridge(
        &mut self,
        git_action: &str,
        repo_url: &str,
        target_dir: &str,
        data: &serde_json::Value,
        bridge_resp: &serde_json::Value,
    ) -> Result<(i32, String, String), MvmError> {
        let mut branch = data
            .get("branch")
            .or_else(|| bridge_resp.get("branch"))
            .and_then(|v| v.as_str())
            .unwrap_or("main")
            .to_string();
        let mut head_oid = data
            .get("head_oid")
            .or_else(|| data.get("head_sha"))
            .or_else(|| data.get("commit_sha"))
            .or_else(|| bridge_resp.get("head_oid"))
            .or_else(|| bridge_resp.get("head_sha"))
            .or_else(|| bridge_resp.get("commit_sha"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(str::to_string);

        // Parse Git smart-HTTP pkt-line advertisement when `body_b64` is returned directly by relay
        if let Some(b64) = data
            .get("body_b64")
            .or_else(|| bridge_resp.get("body_b64"))
            .and_then(|v| v.as_str())
        {
            if let Ok(raw) = decode_base64_bytes(b64) {
                let adv = String::from_utf8_lossy(&raw);
                let _ =
                    self.write_guest_file(&format!("{target_dir}/.git/FETCH_HEAD"), adv.as_bytes());
                for line in adv.lines() {
                    let trimmed = line.trim();
                    let body = match trimmed.get(..4) {
                        Some(head)
                            if trimmed.len() > 4 && head.bytes().all(|b| b.is_ascii_hexdigit()) =>
                        {
                            trimmed.get(4..).unwrap_or(trimmed)
                        }
                        _ => trimmed,
                    };
                    if body.len() >= 41 {
                        if let (Some(oid_cand), Some(rest_raw)) = (body.get(..40), body.get(40..)) {
                            if oid_cand.bytes().all(|b| b.is_ascii_hexdigit()) {
                                let rest = rest_raw.split('\0').next().unwrap_or("").trim();
                                if rest == "HEAD" && head_oid.is_none() {
                                    head_oid = Some(oid_cand.to_string());
                                } else if let Some(bname) = rest.strip_prefix("refs/heads/") {
                                    if head_oid.as_deref() == Some(oid_cand) || head_oid.is_none() {
                                        branch = bname.to_string();
                                        if head_oid.is_none() {
                                            head_oid = Some(oid_cand.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let resolved_oid = head_oid.unwrap_or_else(|| {
            let h = sha256_hex_bytes(format!("git|{repo_url}|{branch}").as_bytes());
            h[..40].to_string()
        });

        let mut state = self
            .load_git_state(target_dir)
            .unwrap_or_else(|| GitRepoState {
                branch: branch.clone(),
                remote_url: Some(repo_url.to_string()),
                head_oid: Some(resolved_oid.clone()),
                staged: BTreeMap::new(),
                head_tree: BTreeMap::new(),
                commits: Vec::new(),
            });
        state.branch = branch.clone();
        if !repo_url.is_empty() {
            state.remote_url = Some(repo_url.to_string());
        }
        state.head_oid = Some(resolved_oid.clone());

        let mut written_files = 0usize;
        let files_val = data.get("files").or_else(|| bridge_resp.get("files"));
        if let Some(files_obj) = files_val.and_then(|v| v.as_object()) {
            for (rel, val) in files_obj {
                let clean = rel.trim_start_matches('/');
                if clean.is_empty() || is_git_internal_rel_path(clean) {
                    continue;
                }
                if let Some(content) = val.as_str() {
                    let guest_path = format!("{target_dir}/{clean}");
                    self.write_guest_file(&guest_path, content.as_bytes())?;
                    state
                        .head_tree
                        .insert(clean.to_string(), content.to_string());
                    written_files += 1;
                }
            }
        } else if let Some(files_arr) = files_val.and_then(|v| v.as_array()) {
            for entry in files_arr {
                let Some(rel) = entry.get("path").and_then(|v| v.as_str()) else {
                    continue;
                };
                let clean = rel.trim_start_matches('/');
                if clean.is_empty() || is_git_internal_rel_path(clean) {
                    continue;
                }
                if let Some(content) = entry.get("content").and_then(|v| v.as_str()) {
                    let guest_path = format!("{target_dir}/{clean}");
                    self.write_guest_file(&guest_path, content.as_bytes())?;
                    state
                        .head_tree
                        .insert(clean.to_string(), content.to_string());
                    written_files += 1;
                }
            }
        }

        if state.commits.is_empty() {
            state.commits.push(GitCommitRecord {
                oid: resolved_oid.clone(),
                parent_oid: None,
                message: format!("Imported upstream {branch}"),
                tree: state.head_tree.clone(),
            });
        }
        self.save_git_state(target_dir, &state)?;

        let short_oid: String = resolved_oid.chars().take(7).collect();
        let msg = match git_action {
            "push" => format!("To {repo_url}\n   {short_oid}..{short_oid}  {branch} -> {branch}\n"),
            "pull" => format!("From {repo_url}\n * branch {branch} -> FETCH_HEAD ({short_oid}, {written_files} file(s) updated)\n"),
            "create_branch" => format!("Switched to branch '{branch}' (upstream {repo_url} at {short_oid})\n"),
            "create_pr" => {
                let pr_num = data
                    .get("pr_number")
                    .or_else(|| bridge_resp.get("pr_number"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1);
                let pr_url = data
                    .get("html_url")
                    .or_else(|| data.get("url"))
                    .or_else(|| bridge_resp.get("html_url"))
                    .or_else(|| bridge_resp.get("url"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(repo_url);
                format!("Created pull request #{pr_num}: {pr_url}\n")
            }
            _ => format!("Cloning into '{target_dir}'...\nChecked out '{branch}' at {short_oid} ({written_files} file(s) materialized).\n"),
        };
        Ok((0, msg, String::new()))
    }
}

fn is_git_internal_rel_path(rel: &str) -> bool {
    rel == ".git" || rel.starts_with(".git/") || rel.contains("/.git/") || rel.ends_with("/.git")
}

fn is_canonical_protected_rel(rel: &str) -> bool {
    (rel.starts_with("prism/") && rel.ends_with(".lex.tex")) || rel == "app/state"
}

fn is_excluded_workspace_rel_path(rel: &str) -> bool {
    if rel.is_empty()
        || rel == "node_modules"
        || rel.starts_with("node_modules/")
        || rel.contains("/node_modules/")
        || rel == ".venv"
        || rel.starts_with(".venv/")
        || rel.contains("/.venv/")
        || rel == "__pycache__"
        || rel.starts_with("__pycache__/")
        || rel.contains("/__pycache__/")
        || is_git_internal_rel_path(rel)
        || is_canonical_protected_rel(rel)
    {
        return true;
    }
    let lower = rel.to_ascii_lowercase();
    lower.ends_with(".db") || lower.ends_with(".wasm")
}

fn is_binary_workspace_bytes(bytes: &[u8]) -> bool {
    match std::str::from_utf8(bytes) {
        Ok(_) => bytes.len() > 1_048_576 || bytes.starts_with(b"\x7fELF") || bytes.contains(&0),
        Err(_) => true,
    }
}

fn parse_url_scheme_and_host(url: &str) -> (String, String) {
    let trimmed = url.trim();
    let Some((scheme_raw, after_scheme)) = trimmed.split_once("://") else {
        return ("http".to_string(), trimmed.to_string());
    };
    let authority = after_scheme.split(['/', '?', '#']).next().unwrap_or("");
    let host_port = authority.rsplit('@').next().unwrap_or(authority);
    let host = if let Some(rest) = host_port.strip_prefix('[') {
        rest.split(']').next().unwrap_or(rest)
    } else {
        host_port.split(':').next().unwrap_or(host_port)
    };
    (scheme_raw.to_ascii_lowercase(), host.to_ascii_lowercase())
}

fn sha256_hex_bytes(bytes: &[u8]) -> String {
    let dig = Sha256::digest(bytes);
    let mut s = String::with_capacity(64);
    for b in dig {
        use std::fmt::Write;
        let _ = write!(&mut s, "{b:02x}");
    }
    s
}

fn decode_hex_bytes(hex_str: &str) -> Result<Vec<u8>, BridgeError> {
    let clean: Vec<u8> = hex_str
        .bytes()
        .filter(|b| !b.is_ascii_whitespace())
        .collect();
    if !clean.len().is_multiple_of(2) {
        return Err(BridgeError::InvalidBridgeJson(
            "odd-length hex payload".to_string(),
        ));
    }
    let mut out = Vec::with_capacity(clean.len() / 2);
    for pair in clean.chunks_exact(2) {
        let hi = match pair[0] {
            b'0'..=b'9' => pair[0] - b'0',
            b'a'..=b'f' => pair[0] - b'a' + 10,
            b'A'..=b'F' => pair[0] - b'A' + 10,
            _ => {
                return Err(BridgeError::InvalidBridgeJson(
                    "invalid hex digit".to_string(),
                ))
            }
        };
        let lo = match pair[1] {
            b'0'..=b'9' => pair[1] - b'0',
            b'a'..=b'f' => pair[1] - b'a' + 10,
            b'A'..=b'F' => pair[1] - b'A' + 10,
            _ => {
                return Err(BridgeError::InvalidBridgeJson(
                    "invalid hex digit".to_string(),
                ))
            }
        };
        out.push((hi << 4) | lo);
    }
    Ok(out)
}

fn encode_base64_bytes(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };
        out.push(TABLE[(b0 >> 2) as usize] as char);
        out.push(TABLE[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(b2 & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

fn decode_base64_bytes(b64: &str) -> Result<Vec<u8>, BridgeError> {
    let mut vals = Vec::with_capacity(b64.len());
    for ch in b64.bytes() {
        if ch.is_ascii_whitespace() || ch == b'=' {
            continue;
        }
        let v = match ch {
            b'A'..=b'Z' => ch - b'A',
            b'a'..=b'z' => ch - b'a' + 26,
            b'0'..=b'9' => ch - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            _ => {
                return Err(BridgeError::InvalidBridgeJson(
                    "invalid base64 byte".to_string(),
                ))
            }
        };
        vals.push(v);
    }
    if vals.len() % 4 == 1 {
        return Err(BridgeError::InvalidBridgeJson(
            "truncated base64 payload".to_string(),
        ));
    }
    let mut out = Vec::with_capacity((vals.len() * 3) / 4);
    let mut i = 0usize;
    while i + 1 < vals.len() {
        out.push((vals[i] << 2) | (vals[i + 1] >> 4));
        if i + 2 < vals.len() {
            out.push((vals[i + 1] << 4) | (vals[i + 2] >> 2));
            if i + 3 < vals.len() {
                out.push((vals[i + 2] << 6) | vals[i + 3]);
            }
        }
        i += 4;
    }
    Ok(out)
}

fn extract_bridge_payload_bytes(
    data: &serde_json::Value,
    bridge_resp: &serde_json::Value,
) -> Vec<u8> {
    for src in [data, bridge_resp] {
        if let Some(s) = src
            .get("body_utf8")
            .or_else(|| src.get("body_text"))
            .and_then(|v| v.as_str())
        {
            return s.as_bytes().to_vec();
        }
        if let Some(b64) = src
            .get("body_b64")
            .or_else(|| src.get("body_base64"))
            .and_then(|v| v.as_str())
        {
            if let Ok(raw) = decode_base64_bytes(b64) {
                return raw;
            }
        }
        if let Some(ex) = src
            .get("text_excerpt")
            .or_else(|| src.get("excerpt"))
            .and_then(|v| v.as_str())
        {
            return ex.as_bytes().to_vec();
        }
    }
    Vec::new()
}

fn extract_bridge_elf_bytes(
    data: &serde_json::Value,
    bridge_resp: &serde_json::Value,
) -> Result<Vec<u8>, MvmError> {
    for src in [data, bridge_resp] {
        if let Some(hex_s) = src.get("elf_hex").and_then(|v| v.as_str()) {
            return decode_hex_bytes(hex_s).map_err(MvmError::BridgeRejected);
        }
        if let Some(b64) = src
            .get("elf_b64")
            .or_else(|| src.get("elf_base64"))
            .or_else(|| src.get("body_b64"))
            .or_else(|| src.get("body_base64"))
            .and_then(|v| v.as_str())
        {
            return decode_base64_bytes(b64).map_err(MvmError::BridgeRejected);
        }
    }
    Err(MvmError::BridgeRejected(BridgeError::InvalidBridgeJson(
        "missing elf_hex, elf_base64, or body_b64 in pkg_fetch bridge response".to_string(),
    )))
}

#[path = "mvm_shell.rs"]
mod mvm_shell;
pub use mvm_shell::pack_compressed_initramfs_uormvmz1;
#[allow(unused_imports)]
use mvm_shell::*;

#[cfg(test)]
mod hardening_tests {
    use super::*;

    #[test]
    fn syscalls_and_git_bridge_handle_extreme_inputs_safely() {
        let mut vm = MvmSandbox::boot_default().expect("boot");
        let mut proc = Rv64Process {
            pid: 10,
            regs: [0u64; 32],
            pc: 0,
            ram: vec![0u8; 4096],
            brk_addr: 0x1000,
            mmap_cursor: GUEST_MMAP_BASE,
            lr_reservation: None,
            fds: BTreeMap::new(),
            next_fd: 3,
            cwd: "/workspace".to_string(),
            exited: None,
        };
        proc.write_bytes(0x100, b"/bin/evil\0").unwrap();
        proc.regs[17] = 34; // SYS_mkdirat
        proc.regs[11] = 0x100;
        vm.handle_syscall(&mut proc, 0).unwrap();
        assert_eq!(proc.regs[10] as i64, LINUX_EACCES);

        proc.regs[17] = 214; // SYS_brk >= 4 GiB must never truncate to 0 on wasm32
        proc.regs[10] = 0x1_0000_0000;
        vm.handle_syscall(&mut proc, 0).unwrap();
        assert_eq!(proc.regs[10], 0x1000);

        proc.fds.insert(
            5,
            FdKind::RegularFile {
                path: "/workspace/t.txt".to_string(),
                offset: 100,
                writable: false,
                append: false,
            },
        );
        proc.regs[17] = 62; // SYS_lseek overflow
        proc.regs[10] = 5;
        proc.regs[11] = i64::MAX as u64;
        proc.regs[12] = 1; // SEEK_CUR
        vm.handle_syscall(&mut proc, 0).unwrap();
        assert_eq!(proc.regs[10] as i64, LINUX_EINVAL);

        proc.regs[17] = 222; // SYS_mmap overflow
        proc.regs[11] = u64::MAX;
        vm.handle_syscall(&mut proc, 0).unwrap();
        assert_eq!(proc.regs[10] as i64, -12);

        // Multi-byte UTF-8 spanning byte index 4 and 40 must not panic in complete_git_bridge.
        let mut utf8_adv = Vec::from("éééé".as_bytes()); // 8 bytes, 'é' spans byte 4..5
        utf8_adv.extend_from_slice(&[0xc3u8, 0xa9].repeat(25));
        let b64 = encode_base64_bytes(&utf8_adv);
        let data = serde_json::json!({ "body_b64": b64, "repo_url": "https://example.invalid/r", "target_dir": "/workspace/g" });
        let bridge_resp = serde_json::json!({});
        assert!(vm
            .complete_git_bridge(
                "clone",
                "https://example.invalid/r",
                "/workspace/g",
                &data,
                &bridge_resp,
            )
            .is_ok());

        vm.write_guest_file("/workspace/a.txt", b"ok").unwrap();
        assert!(vm.rename_workspace_file("a.txt", ".").is_err());
    }
}
