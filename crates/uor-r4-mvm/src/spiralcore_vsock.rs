pub mod r4_spiralcore_vsock {
    /// Inductive type `R4.SpiralCoreVsock.SocketDomain`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SocketDomain {
        /// Variant `AfVsock`.
        AfVsock,
        /// Variant `AfInet`.
        AfInet,
        /// Variant `AfInet6`.
        AfInet6,
        /// Variant `AfPacket`.
        AfPacket,
        /// Variant `AfUnknown`.
        AfUnknown,
    }

    /// Inductive type `R4.SpiralCoreVsock.SocketAdmission`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SocketAdmission {
        /// Variant `AdmittedVsock`.
        AdmittedVsock,
        /// Variant `RefusedNicDisabled`.
        RefusedNicDisabled,
        /// Variant `RefusedUnsupportedDomain`.
        RefusedUnsupportedDomain,
    }

    /// Structure `R4.SpiralCoreVsock.SpiralCoreFrame`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SpiralCoreFrame {
        /// Field `srcCid`.
        pub src_cid: u64,
        /// Field `dstCid`.
        pub dst_cid: u64,
        /// Field `vsockPort`.
        pub vsock_port: u64,
        /// Field `cl06Spin`.
        pub cl06_spin: u64,
        /// Field `e8Orbit`.
        pub e8_orbit: u64,
        /// Field `payloadBytes`.
        pub payload_bytes: u64,
        /// Field `maxPayloadBytes`.
        pub max_payload_bytes: u64,
    }

    /// Inductive type `R4.SpiralCoreVsock.FrameVerdict`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FrameVerdict {
        /// Variant `FrameAdmitted`.
        FrameAdmitted,
        /// Variant `RefusedInvalidCid`.
        RefusedInvalidCid,
        /// Variant `RefusedSpinOutOfRange`.
        RefusedSpinOutOfRange,
        /// Variant `RefusedOrbitOutOfRange`.
        RefusedOrbitOutOfRange,
        /// Variant `RefusedPayloadTooLarge`.
        RefusedPayloadTooLarge,
    }

    /// Structure `R4.SpiralCoreVsock.Elf64HeaderSummary`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Elf64HeaderSummary {
        /// Field `magicValid`.
        pub magic_valid: bool,
        /// Field `elfClass64`.
        pub elf_class64: bool,
        /// Field `littleEndian`.
        pub little_endian: bool,
        /// Field `machineCode`.
        pub machine_code: u64,
        /// Field `entryAligned`.
        pub entry_aligned: bool,
        /// Field `segmentCount`.
        pub segment_count: u64,
        /// Field `maxSegments`.
        pub max_segments: u64,
    }

    /// Inductive type `R4.SpiralCoreVsock.ElfAdmission`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ElfAdmission {
        /// Variant `ElfAdmitted`.
        ElfAdmitted,
        /// Variant `RefusedBadMagic`.
        RefusedBadMagic,
        /// Variant `RefusedWrongClass`.
        RefusedWrongClass,
        /// Variant `RefusedWrongMachine`.
        RefusedWrongMachine,
        /// Variant `RefusedSegmentOverflow`.
        RefusedSegmentOverflow,
    }

    /// Structure `R4.SpiralCoreVsock.EgressPolicyCheck`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct EgressPolicyCheck {
        /// Field `hostAllowlisted`.
        pub host_allowlisted: bool,
        /// Field `placeholderMatched`.
        pub placeholder_matched: bool,
        /// Field `secretInGuestRam`.
        pub secret_in_guest_ram: bool,
    }

    /// Inductive type `R4.SpiralCoreVsock.EgressVerdict`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum EgressVerdict {
        /// Variant `ProxyAdmitted`.
        ProxyAdmitted,
        /// Variant `RefusedSecretLeakToGuest`.
        RefusedSecretLeakToGuest,
        /// Variant `RefusedHostNotAllowlisted`.
        RefusedHostNotAllowlisted,
        /// Variant `RefusedMissingPlaceholder`.
        RefusedMissingPlaceholder,
    }

    /// Function `R4.SpiralCoreVsock.classifyDomain`.
    pub fn classify_domain(domain_code: u64) -> SocketDomain {
        if domain_code == 40 {
            SocketDomain::AfVsock
        } else if domain_code == 2 {
            SocketDomain::AfInet
        } else if domain_code == 10 {
            SocketDomain::AfInet6
        } else if domain_code == 17 {
            SocketDomain::AfPacket
        } else {
            SocketDomain::AfUnknown
        }
    }

    /// Function `R4.SpiralCoreVsock.admitGuestSocket`.
    pub fn admit_guest_socket(domain_code: u64) -> SocketAdmission {
        match classify_domain(domain_code) {
            SocketDomain::AfVsock => SocketAdmission::AdmittedVsock,
            SocketDomain::AfInet | SocketDomain::AfInet6 | SocketDomain::AfPacket => {
                SocketAdmission::RefusedNicDisabled
            }
            SocketDomain::AfUnknown => SocketAdmission::RefusedUnsupportedDomain,
        }
    }

    /// Function `R4.SpiralCoreVsock.admitSpiralCoreFrame`.
    pub fn admit_spiralcore_frame(f: SpiralCoreFrame) -> FrameVerdict {
        if f.src_cid == 0 || f.dst_cid == 0 || f.src_cid == f.dst_cid {
            FrameVerdict::RefusedInvalidCid
        } else if 64 <= f.cl06_spin {
            FrameVerdict::RefusedSpinOutOfRange
        } else if 16 <= f.e8_orbit {
            FrameVerdict::RefusedOrbitOutOfRange
        } else if f.payload_bytes > f.max_payload_bytes {
            FrameVerdict::RefusedPayloadTooLarge
        } else {
            FrameVerdict::FrameAdmitted
        }
    }

    /// Function `R4.SpiralCoreVsock.admitRv64Elf`.
    pub fn admit_rv64_elf(h: Elf64HeaderSummary) -> ElfAdmission {
        if !h.magic_valid {
            ElfAdmission::RefusedBadMagic
        } else if !(h.elf_class64 && h.little_endian && h.entry_aligned) {
            ElfAdmission::RefusedWrongClass
        } else if h.machine_code != 243 {
            ElfAdmission::RefusedWrongMachine
        } else if h.segment_count == 0 || h.segment_count > h.max_segments {
            ElfAdmission::RefusedSegmentOverflow
        } else {
            ElfAdmission::ElfAdmitted
        }
    }

    /// Function `R4.SpiralCoreVsock.admitHostEgress`.
    pub fn admit_host_egress(e: EgressPolicyCheck) -> EgressVerdict {
        if e.secret_in_guest_ram {
            EgressVerdict::RefusedSecretLeakToGuest
        } else if !e.host_allowlisted {
            EgressVerdict::RefusedHostNotAllowlisted
        } else if !e.placeholder_matched {
            EgressVerdict::RefusedMissingPlaceholder
        } else {
            EgressVerdict::ProxyAdmitted
        }
    }
}
