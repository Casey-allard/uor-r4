//! Extracted pure LexLean admission state machines from `Casey-allard/lexlean-r4`
//! commit `288e2677` (`lexlean_generated.rs`, MIT OR Apache-2.0).
//! Excludes all model, tokenizer, and AST-eval shortcut modules.

#![allow(dead_code, unused_parens, clippy::all)]

pub mod r4_secops_audit {
    /// Inductive type `R4.SecopsAudit.AuditChainOutcome`.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum AuditChainOutcome {
        /// Variant `chainVerified`.
        ChainVerified(u64, u64, u64),
        /// Variant `invalidSchemaVersion`.
        InvalidSchemaVersion(u64),
        /// Variant `nonMonotonicSequence`.
        NonMonotonicSequence(u64, u64),
        /// Variant `brokenPreviousHashLink`.
        BrokenPreviousHashLink(u64, u64),
        /// Variant `tamperedEventDigest`.
        TamperedEventDigest(u64, u64),
        /// Variant `zeroEventDigest`.
        ZeroEventDigest,
    }

    /// Inductive type `R4.SecopsAudit.McpPolicyOutcome`.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum McpPolicyOutcome {
        /// Variant `allowedSafe`.
        AllowedSafe(u64, u64),
        /// Variant `allowedConfirmed`.
        AllowedConfirmed(u64, u64),
        /// Variant `rejectedByUser`.
        RejectedByUser(u64),
        /// Variant `deniedByPolicy`.
        DeniedByPolicy(u64),
        /// Variant `invalidPolicyMode`.
        InvalidPolicyMode(u64),
    }

    /// Inductive type `R4.SecopsAudit.SandboxAdmissionOutcome`.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum SandboxAdmissionOutcome {
        /// Variant `admittedWorkspaceWrite`.
        AdmittedWorkspaceWrite(u64, u64, u64),
        /// Variant `rejectedEmptyPath`.
        RejectedEmptyPath,
        /// Variant `rejectedPathTraversal`.
        RejectedPathTraversal(u64),
        /// Variant `rejectedFileQuotaExceeded`.
        RejectedFileQuotaExceeded(u64, u64),
        /// Variant `rejectedByteQuotaExceeded`.
        RejectedByteQuotaExceeded(u64, u64),
        /// Variant `rejectedZeroReceipt`.
        RejectedZeroReceipt,
    }

    /// Function `R4.SecopsAudit.verifyAuditLink`.
    pub fn verify_audit_link(
        schema_version: u64,
        prev_seq: u64,
        event_seq: u64,
        expected_prev_hash: u64,
        recorded_prev_hash: u64,
        computed_event_hash: u64,
        recorded_event_hash: u64,
        merkle_root_digest: u64,
    ) -> AuditChainOutcome {
        if (!((schema_version.clone()) == (1u64))) {
            AuditChainOutcome::InvalidSchemaVersion(schema_version.clone())
        } else {
            if (!((event_seq.clone()) == ((prev_seq.clone()).saturating_add(1u64)))) {
                AuditChainOutcome::NonMonotonicSequence(
                    (prev_seq.clone()).saturating_add(1u64),
                    event_seq.clone(),
                )
            } else {
                if (!((recorded_prev_hash.clone()) == (expected_prev_hash.clone()))) {
                    AuditChainOutcome::BrokenPreviousHashLink(
                        expected_prev_hash.clone(),
                        recorded_prev_hash.clone(),
                    )
                } else {
                    if ((computed_event_hash.clone()) == (0u64)) {
                        AuditChainOutcome::ZeroEventDigest
                    } else {
                        if (!((recorded_event_hash.clone()) == (computed_event_hash.clone()))) {
                            AuditChainOutcome::TamperedEventDigest(
                                computed_event_hash.clone(),
                                recorded_event_hash.clone(),
                            )
                        } else {
                            AuditChainOutcome::ChainVerified(
                                event_seq.clone(),
                                recorded_event_hash.clone(),
                                merkle_root_digest.clone(),
                            )
                        }
                    }
                }
            }
        }
    }

    /// Function `R4.SecopsAudit.evaluateMcpToolPolicy`.
    pub fn evaluate_mcp_tool_policy(
        policy_mode: u64,
        tool_id: u64,
        user_confirmed: bool,
        audit_seq: u64,
    ) -> McpPolicyOutcome {
        if ((policy_mode.clone()) == (0u64)) {
            McpPolicyOutcome::AllowedSafe(tool_id.clone(), audit_seq.clone())
        } else {
            if ((policy_mode.clone()) == (1u64)) {
                if user_confirmed.clone() {
                    McpPolicyOutcome::AllowedConfirmed(tool_id.clone(), audit_seq.clone())
                } else {
                    McpPolicyOutcome::RejectedByUser(tool_id.clone())
                }
            } else {
                if ((policy_mode.clone()) == (2u64)) {
                    McpPolicyOutcome::DeniedByPolicy(tool_id.clone())
                } else {
                    McpPolicyOutcome::InvalidPolicyMode(policy_mode.clone())
                }
            }
        }
    }

    /// Function `R4.SecopsAudit.admitSandboxWrite`.
    pub fn admit_sandbox_write(
        path_byte_len: u64,
        has_parent_traversal: bool,
        current_file_count: u64,
        max_file_quota: u64,
        payload_byte_len: u64,
        max_payload_bytes: u64,
        content_sha256_hi: u64,
    ) -> SandboxAdmissionOutcome {
        if ((path_byte_len.clone()) == (0u64)) {
            SandboxAdmissionOutcome::RejectedEmptyPath
        } else {
            if has_parent_traversal.clone() {
                SandboxAdmissionOutcome::RejectedPathTraversal(path_byte_len.clone())
            } else {
                if ((max_file_quota.clone()) <= (current_file_count.clone())) {
                    SandboxAdmissionOutcome::RejectedFileQuotaExceeded(
                        current_file_count.clone(),
                        max_file_quota.clone(),
                    )
                } else {
                    if ((max_payload_bytes.clone()) < (payload_byte_len.clone())) {
                        SandboxAdmissionOutcome::RejectedByteQuotaExceeded(
                            payload_byte_len.clone(),
                            max_payload_bytes.clone(),
                        )
                    } else {
                        if ((content_sha256_hi.clone()) == (0u64)) {
                            SandboxAdmissionOutcome::RejectedZeroReceipt
                        } else {
                            SandboxAdmissionOutcome::AdmittedWorkspaceWrite(
                                (current_file_count.clone()).saturating_add(1u64),
                                payload_byte_len.clone(),
                                content_sha256_hi.clone(),
                            )
                        }
                    }
                }
            }
        }
    }
}

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

pub mod r4_connected_studio {
    /// Inductive type `R4.ConnectedStudio.TransportKind`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TransportKind {
        /// Variant `WssRelay`.
        WssRelay,
        /// Variant `HttpsDirect`.
        HttpsDirect,
        /// Variant `LoopbackDev`.
        LoopbackDev,
        /// Variant `PlainHttp`.
        PlainHttp,
        /// Variant `PlainWs`.
        PlainWs,
    }

    /// Inductive type `R4.ConnectedStudio.McpPolicyMode`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum McpPolicyMode {
        /// Variant `Safe`.
        Safe,
        /// Variant `Confirm`.
        Confirm,
        /// Variant `Deny`.
        Deny,
    }

    /// Inductive type `R4.ConnectedStudio.BridgeTransportVerdict`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BridgeTransportVerdict {
        /// Variant `TransportAdmitted`.
        TransportAdmitted,
        /// Variant `PolicyDenied`.
        PolicyDenied,
        /// Variant `InsecureTransportRejected`.
        InsecureTransportRejected,
        /// Variant `InvalidVsockCid`.
        InvalidVsockCid,
    }

    /// Inductive type `R4.ConnectedStudio.GitSecretVerdict`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum GitSecretVerdict {
        /// Variant `SecretSubstitutionAdmitted`.
        SecretSubstitutionAdmitted,
        /// Variant `GuestRamSecretContamination`.
        GuestRamSecretContamination,
        /// Variant `HostNotAllowlistedForSecret`.
        HostNotAllowlistedForSecret,
        /// Variant `MissingSecretPlaceholder`.
        MissingSecretPlaceholder,
    }

    /// Inductive type `R4.ConnectedStudio.PackageInstallVerdict`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum PackageInstallVerdict {
        /// Variant `PackageAdmitted`.
        PackageAdmitted,
        /// Variant `DigestMismatch`.
        DigestMismatch,
        /// Variant `InvalidElf64Riscv`.
        InvalidElf64Riscv,
        /// Variant `SizeExceedsQuota`.
        SizeExceedsQuota,
    }

    /// Inductive type `R4.ConnectedStudio.WorkerTurnVerdict`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum WorkerTurnVerdict {
        /// Variant `WorkerTurnAdmitted`.
        WorkerTurnAdmitted,
        /// Variant `MainThreadBlockingRejected`.
        MainThreadBlockingRejected,
        /// Variant `SequenceWindowExceeded`.
        SequenceWindowExceeded,
    }

    /// Function `R4.ConnectedStudio.admitBridgeTransport`.
    pub fn admit_bridge_transport(
        src_cid: u16,
        dst_cid: u16,
        policy: McpPolicyMode,
        transport: TransportKind,
    ) -> BridgeTransportVerdict {
        if src_cid < 3 || dst_cid != 2 {
            BridgeTransportVerdict::InvalidVsockCid
        } else if matches!(policy, McpPolicyMode::Deny) {
            BridgeTransportVerdict::PolicyDenied
        } else if matches!(transport, TransportKind::PlainHttp | TransportKind::PlainWs) {
            BridgeTransportVerdict::InsecureTransportRejected
        } else {
            BridgeTransportVerdict::TransportAdmitted
        }
    }

    /// Function `R4.ConnectedStudio.admitGitSecretSubstitution`.
    pub fn admit_git_secret_substitution(
        secret_in_guest_ram: bool,
        placeholder_present: bool,
        is_allowlisted_github_host: bool,
    ) -> GitSecretVerdict {
        if secret_in_guest_ram {
            GitSecretVerdict::GuestRamSecretContamination
        } else if !placeholder_present {
            GitSecretVerdict::MissingSecretPlaceholder
        } else if !is_allowlisted_github_host {
            GitSecretVerdict::HostNotAllowlistedForSecret
        } else {
            GitSecretVerdict::SecretSubstitutionAdmitted
        }
    }

    /// Function `R4.ConnectedStudio.admitPackageInstall`.
    pub fn admit_package_install(
        sha256_matched: bool,
        elf_magic_ok: bool,
        elf_class_64: bool,
        machine_code: u16,
        byte_len: u32,
        max_bytes: u32,
    ) -> PackageInstallVerdict {
        if !sha256_matched {
            PackageInstallVerdict::DigestMismatch
        } else if !elf_magic_ok || !elf_class_64 || machine_code != 243 || byte_len == 0 {
            PackageInstallVerdict::InvalidElf64Riscv
        } else if byte_len > max_bytes {
            PackageInstallVerdict::SizeExceedsQuota
        } else {
            PackageInstallVerdict::PackageAdmitted
        }
    }

    /// Function `R4.ConnectedStudio.admitWorkerTurn`.
    pub fn admit_worker_turn(
        runs_on_worker_thread: bool,
        prompt_tokens: u32,
        max_new_tokens: u32,
        max_seq_window: u32,
    ) -> WorkerTurnVerdict {
        if !runs_on_worker_thread {
            WorkerTurnVerdict::MainThreadBlockingRejected
        } else if max_seq_window == 0
            || (prompt_tokens as u64) + (max_new_tokens as u64) > (max_seq_window as u64)
        {
            WorkerTurnVerdict::SequenceWindowExceeded
        } else {
            WorkerTurnVerdict::WorkerTurnAdmitted
        }
    }
}

pub mod r4_mvm_linux_container {
    /// Inductive `R4.MvmLinuxContainer.WorkspaceAuthorityMode` (`R4-63`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum WorkspaceAuthorityMode {
        SingleSourceMvmCow,
        SplitBrainDuplicateStore,
    }

    /// Inductive `R4.MvmLinuxContainer.WorkspaceAuthorityVerdict` (`R4-63`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum WorkspaceAuthorityVerdict {
        SingleSourceAdmitted,
        RefusedSplitBrainWorkspace,
        RefusedInconsistentCowState,
    }

    /// Inductive `R4.MvmLinuxContainer.MvmNetworkMode` (`R4-63`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum MvmNetworkMode {
        ZeroNicIsolated,
        SpiralCoreEncapsulated,
    }

    /// Inductive `R4.MvmLinuxContainer.MvmSocketVerdict` (`R4-63`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum MvmSocketVerdict {
        AdmittedDirectVsock,
        AdmittedSpiralCoreInetEncapsulation,
        RefusedZeroNicIsolated,
        RefusedUnsupportedDomain,
    }

    /// Inductive `R4.MvmLinuxContainer.CorsPreemptionVerdict` (`R4-63`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CorsPreemptionVerdict {
        DirectBrowserFetchAdmitted,
        StatelessRelayCorsPreempted,
        RefusedPrivateAddressSsrf,
        RefusedCredentialedCorsProxy,
        RefusedSpiralCoreNetDisabled,
    }

    /// Inductive `R4.MvmLinuxContainer.McpGateMode` (`R4-63`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum McpGateMode {
        Deny,
        Confirm,
        Safe,
    }

    /// Inductive `R4.MvmLinuxContainer.AgentPlanMode` (`R4-63`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AgentPlanMode {
        PlanReadOnly,
        AgentFull,
    }

    /// Inductive `R4.MvmLinuxContainer.ShellCommandKind` (`R4-63`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ShellCommandKind {
        ReadOnlyInspection,
        MutatingOrNetwork,
    }

    /// Inductive `R4.MvmLinuxContainer.InteractiveCommandVerdict` (`R4-63`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum InteractiveCommandVerdict {
        AutoRunAdmitted,
        StagedPendingUserGo,
        RefusedByDenyPolicy,
        RefusedInPlanMode,
    }

    /// Function `R4.MvmLinuxContainer.maxLiveStateHeaderTokens` (`R4-63`).
    pub fn max_live_state_header_tokens() -> u64 {
        120
    }

    /// Function `R4.MvmLinuxContainer.admitLiveStateHeader` (`R4-63`).
    pub fn admit_live_state_header(token_count: u64) -> bool {
        !(max_live_state_header_tokens() < token_count)
    }

    /// Function `R4.MvmLinuxContainer.admitWorkspaceAuthority` (`R4-63`).
    pub fn admit_workspace_authority(
        mode: WorkspaceAuthorityMode,
        cow_consistent: bool,
    ) -> WorkspaceAuthorityVerdict {
        match mode {
            WorkspaceAuthorityMode::SingleSourceMvmCow => {
                if cow_consistent {
                    WorkspaceAuthorityVerdict::SingleSourceAdmitted
                } else {
                    WorkspaceAuthorityVerdict::RefusedInconsistentCowState
                }
            }
            WorkspaceAuthorityMode::SplitBrainDuplicateStore => {
                WorkspaceAuthorityVerdict::RefusedSplitBrainWorkspace
            }
        }
    }

    /// Function `R4.MvmLinuxContainer.admitMvmSocket` (`R4-63`).
    pub fn admit_mvm_socket(
        net_mode: MvmNetworkMode,
        is_inet_domain: bool,
        is_vsock_domain: bool,
    ) -> MvmSocketVerdict {
        if is_vsock_domain {
            MvmSocketVerdict::AdmittedDirectVsock
        } else if is_inet_domain {
            match net_mode {
                MvmNetworkMode::ZeroNicIsolated => MvmSocketVerdict::RefusedZeroNicIsolated,
                MvmNetworkMode::SpiralCoreEncapsulated => {
                    MvmSocketVerdict::AdmittedSpiralCoreInetEncapsulation
                }
            }
        } else {
            MvmSocketVerdict::RefusedUnsupportedDomain
        }
    }

    /// Function `R4.MvmLinuxContainer.admitPublicCorsPreemption` (`R4-63`).
    pub fn admit_public_cors_preemption(
        spiral_core_net_enabled: bool,
        is_private_or_loopback: bool,
        has_credentials: bool,
        direct_fetch_cors_blocked: bool,
    ) -> CorsPreemptionVerdict {
        if is_private_or_loopback {
            CorsPreemptionVerdict::RefusedPrivateAddressSsrf
        } else if !spiral_core_net_enabled {
            CorsPreemptionVerdict::RefusedSpiralCoreNetDisabled
        } else if !direct_fetch_cors_blocked {
            CorsPreemptionVerdict::DirectBrowserFetchAdmitted
        } else if has_credentials {
            CorsPreemptionVerdict::RefusedCredentialedCorsProxy
        } else {
            CorsPreemptionVerdict::StatelessRelayCorsPreempted
        }
    }

    /// Function `R4.MvmLinuxContainer.admitInteractiveCommand` (`R4-63`).
    pub fn admit_interactive_command(
        mcp_mode: McpGateMode,
        plan_mode: AgentPlanMode,
        auto_run: bool,
        cmd_kind: ShellCommandKind,
    ) -> InteractiveCommandVerdict {
        match mcp_mode {
            McpGateMode::Deny => InteractiveCommandVerdict::RefusedByDenyPolicy,
            McpGateMode::Confirm => match plan_mode {
                AgentPlanMode::PlanReadOnly => match cmd_kind {
                    ShellCommandKind::ReadOnlyInspection => {
                        InteractiveCommandVerdict::AutoRunAdmitted
                    }
                    ShellCommandKind::MutatingOrNetwork => {
                        InteractiveCommandVerdict::RefusedInPlanMode
                    }
                },
                AgentPlanMode::AgentFull => match cmd_kind {
                    ShellCommandKind::ReadOnlyInspection => {
                        InteractiveCommandVerdict::AutoRunAdmitted
                    }
                    ShellCommandKind::MutatingOrNetwork => {
                        InteractiveCommandVerdict::StagedPendingUserGo
                    }
                },
            },
            McpGateMode::Safe => match plan_mode {
                AgentPlanMode::PlanReadOnly => match cmd_kind {
                    ShellCommandKind::ReadOnlyInspection => {
                        InteractiveCommandVerdict::AutoRunAdmitted
                    }
                    ShellCommandKind::MutatingOrNetwork => {
                        InteractiveCommandVerdict::RefusedInPlanMode
                    }
                },
                AgentPlanMode::AgentFull => match cmd_kind {
                    ShellCommandKind::ReadOnlyInspection => {
                        InteractiveCommandVerdict::AutoRunAdmitted
                    }
                    ShellCommandKind::MutatingOrNetwork => {
                        if auto_run {
                            InteractiveCommandVerdict::AutoRunAdmitted
                        } else {
                            InteractiveCommandVerdict::StagedPendingUserGo
                        }
                    }
                },
            },
        }
    }

    /// Function `R4.MvmLinuxContainer.verifyMvmLinuxContainerAdmitted` (`R4-63`).
    pub fn verify_mvm_linux_container_admitted() -> bool {
        admit_workspace_authority(WorkspaceAuthorityMode::SingleSourceMvmCow, true)
            == WorkspaceAuthorityVerdict::SingleSourceAdmitted
            && admit_workspace_authority(WorkspaceAuthorityMode::SplitBrainDuplicateStore, true)
                == WorkspaceAuthorityVerdict::RefusedSplitBrainWorkspace
            && admit_workspace_authority(WorkspaceAuthorityMode::SingleSourceMvmCow, false)
                == WorkspaceAuthorityVerdict::RefusedInconsistentCowState
            && admit_mvm_socket(MvmNetworkMode::ZeroNicIsolated, false, true)
                == MvmSocketVerdict::AdmittedDirectVsock
            && admit_mvm_socket(MvmNetworkMode::ZeroNicIsolated, true, false)
                == MvmSocketVerdict::RefusedZeroNicIsolated
            && admit_mvm_socket(MvmNetworkMode::SpiralCoreEncapsulated, true, false)
                == MvmSocketVerdict::AdmittedSpiralCoreInetEncapsulation
            && admit_public_cors_preemption(true, false, false, true)
                == CorsPreemptionVerdict::StatelessRelayCorsPreempted
            && admit_public_cors_preemption(true, false, true, true)
                == CorsPreemptionVerdict::RefusedCredentialedCorsProxy
            && admit_public_cors_preemption(true, true, false, true)
                == CorsPreemptionVerdict::RefusedPrivateAddressSsrf
            && admit_public_cors_preemption(false, false, false, true)
                == CorsPreemptionVerdict::RefusedSpiralCoreNetDisabled
            && admit_interactive_command(
                McpGateMode::Safe,
                AgentPlanMode::AgentFull,
                true,
                ShellCommandKind::MutatingOrNetwork,
            ) == InteractiveCommandVerdict::AutoRunAdmitted
            && admit_interactive_command(
                McpGateMode::Safe,
                AgentPlanMode::AgentFull,
                false,
                ShellCommandKind::MutatingOrNetwork,
            ) == InteractiveCommandVerdict::StagedPendingUserGo
            && admit_interactive_command(
                McpGateMode::Safe,
                AgentPlanMode::PlanReadOnly,
                true,
                ShellCommandKind::MutatingOrNetwork,
            ) == InteractiveCommandVerdict::RefusedInPlanMode
            && admit_interactive_command(
                McpGateMode::Deny,
                AgentPlanMode::AgentFull,
                true,
                ShellCommandKind::ReadOnlyInspection,
            ) == InteractiveCommandVerdict::RefusedByDenyPolicy
            && admit_live_state_header(120)
            && !admit_live_state_header(121)
    }
}

pub mod r4_mvm_polyglot_toolchain {
    /// Inductive `R4.MvmPolyglotToolchain.PolyglotToolchainVerdict` (`R4-67`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum PolyglotToolchainVerdict {
        CargoAstToolchainAdmitted,
        NpmSubcommandAdmitted,
        C99StdlibHeapAdmitted,
        RefusedUnsupportedToolchainSubcommand,
        RefusedProtectedPathToolchainMutation,
        RefusedUnalignedHeapBreak,
    }

    /// Function `R4.MvmPolyglotToolchain.admitCargoTerminalSubcommand` (`R4-67`).
    pub fn admit_cargo_terminal_subcommand(
        known_subcommand: bool,
        touches_protected_canonical: bool,
    ) -> PolyglotToolchainVerdict {
        if touches_protected_canonical {
            PolyglotToolchainVerdict::RefusedProtectedPathToolchainMutation
        } else if !known_subcommand {
            PolyglotToolchainVerdict::RefusedUnsupportedToolchainSubcommand
        } else {
            PolyglotToolchainVerdict::CargoAstToolchainAdmitted
        }
    }

    /// Function `R4.MvmPolyglotToolchain.admitNpmTerminalSubcommand` (`R4-67`).
    pub fn admit_npm_terminal_subcommand(
        known_subcommand: bool,
        touches_protected_canonical: bool,
    ) -> PolyglotToolchainVerdict {
        if touches_protected_canonical {
            PolyglotToolchainVerdict::RefusedProtectedPathToolchainMutation
        } else if !known_subcommand {
            PolyglotToolchainVerdict::RefusedUnsupportedToolchainSubcommand
        } else {
            PolyglotToolchainVerdict::NpmSubcommandAdmitted
        }
    }

    /// Function `R4.MvmPolyglotToolchain.admitC99StdlibBrkAlign` (`R4-67`).
    pub fn admit_c99_stdlib_brk_align(
        aligned_mod8: u64,
        touches_protected_canonical: bool,
    ) -> PolyglotToolchainVerdict {
        if touches_protected_canonical {
            PolyglotToolchainVerdict::RefusedProtectedPathToolchainMutation
        } else if aligned_mod8 != 0 {
            PolyglotToolchainVerdict::RefusedUnalignedHeapBreak
        } else {
            PolyglotToolchainVerdict::C99StdlibHeapAdmitted
        }
    }

    /// Function `R4.MvmPolyglotToolchain.verifyMvmPolyglotToolchainAdmitted` (`R4-67`).
    pub fn verify_mvm_polyglot_toolchain_admitted() -> bool {
        admit_cargo_terminal_subcommand(true, false)
            == PolyglotToolchainVerdict::CargoAstToolchainAdmitted
            && admit_npm_terminal_subcommand(true, false)
                == PolyglotToolchainVerdict::NpmSubcommandAdmitted
            && admit_c99_stdlib_brk_align(0, false)
                == PolyglotToolchainVerdict::C99StdlibHeapAdmitted
            && admit_cargo_terminal_subcommand(false, false)
                == PolyglotToolchainVerdict::RefusedUnsupportedToolchainSubcommand
            && admit_npm_terminal_subcommand(true, true)
                == PolyglotToolchainVerdict::RefusedProtectedPathToolchainMutation
            && admit_c99_stdlib_brk_align(4, false)
                == PolyglotToolchainVerdict::RefusedUnalignedHeapBreak
    }
}
