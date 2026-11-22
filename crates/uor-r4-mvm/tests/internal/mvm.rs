use std::collections::BTreeMap;

use crate::mvm::{
    admit_elf64_riscv, admit_guest_socket_domain, admit_spiralcore_header, MvmSandbox,
    SpiralCoreHeader, SPIRALCORE_MAGIC,
};

use crate::mvm_cc::{compile_c_to_rv64_elf, compile_c_units_to_rv64_elf};
use crate::spiralcore_vsock::r4_spiralcore_vsock::{
    EgressVerdict, ElfAdmission, FrameVerdict, SocketAdmission,
};

#[test]
fn mvm_boot_reports_rv64imac_and_zero_nic() {
    let mut vm = MvmSandbox::boot_default().expect("default MVM boots");
    assert_eq!(vm.guest_cid(), 3);
    assert_eq!(vm.architecture(), "riscv64-rv64imac");

    let agent = vm.read_guest_file("/sbin/mvm-agent").expect("agent ELF");
    assert_eq!(&agent[..4], b"\x7fELF");
    assert_eq!(u16::from_le_bytes([agent[18], agent[19]]), 243);
    assert_eq!(admit_elf64_riscv(&agent), ElfAdmission::ElfAdmitted);

    let handshake = vm
        .exec_command("/sbin/mvm-agent --handshake", "/workspace")
        .expect("RV64 handshake executes");
    assert_eq!(handshake.exit_code, 0, "{}", handshake.stderr);
    assert!(handshake.stdout.contains("MVM_AGENT_ONLINE"));
    assert!(!handshake.vsock_frames.is_empty());

    let inet = vm
        .exec_command("/bin/vsock_probe --try-inet", "/workspace")
        .expect("vsock probe executes");
    assert_eq!(inet.exit_code, 0);
    assert!(
        inet.stdout.contains("AF_INET_DENIED=-97"),
        "{}",
        inet.stdout
    );
}

#[test]
fn mvm_exec_mutates_virtual_files() {
    let mut vm = MvmSandbox::boot_default().expect("boot");
    let source = "#include <stdio.h>\nint main(void) { printf(\"FILE=%d\\n\", 7); return 0; }\n";
    vm.write_guest_file("/workspace/source.c", source.as_bytes())
        .expect("write source");
    let outcome = vm
        .exec_command(
            "cc /workspace/source.c -o /workspace/generated",
            "/workspace",
        )
        .expect("guest cc writes an executable");
    assert_eq!(outcome.exit_code, 0, "{}", outcome.stderr);
    assert_eq!(outcome.compiled_elf_machine, Some(243));
    let generated = vm
        .read_guest_file("/workspace/generated")
        .expect("generated ELF");
    assert_eq!(
        generated,
        compile_c_units_to_rv64_elf(
            &[("/workspace/source.c", source)],
            &["/workspace".to_string(), "/usr/include".to_string()],
            &[],
            |hdr| vm
                .read_guest_file(hdr)
                .and_then(|b| String::from_utf8(b).ok()),
        )
        .expect("direct compiler output")
    );

    let mut files = BTreeMap::new();
    files.insert("notes/result.txt".to_string(), "from-workspace".to_string());
    vm.sync_from_workspace_map(&files);
    assert_eq!(
        vm.export_workspace_text_files()
            .get("notes/result.txt")
            .map(String::as_str),
        Some("from-workspace")
    );
}

#[test]
fn mvm_cow_restore_is_deterministic() {
    let mut vm = MvmSandbox::boot_default().expect("boot");
    vm.write_guest_file("/workspace/state.txt", b"baseline-v1")
        .expect("write baseline");
    let baseline = vm.cow_state_sha256();
    let snapshot = vm.snapshot_cow();

    vm.write_guest_file("/workspace/state.txt", b"mutated-v2")
        .expect("write mutation");
    assert!(vm.dirty_cow_sector_count() >= 1);
    assert_ne!(vm.cow_state_sha256(), baseline);
    assert!(vm.export_dirty_cow_sectors().len() <= 256);

    vm.restore_cow(&snapshot);
    assert_eq!(vm.cow_state_sha256(), baseline);
    assert_eq!(
        vm.read_guest_file("/workspace/state.txt").as_deref(),
        Some(&b"baseline-v1"[..])
    );
}

#[test]
fn mvm_cc_emits_and_executes_real_rv64_elf() {
    let source = r#"
#include <stdio.h>
long fib(long n) {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2);
}
long collatz_steps(long x) {
    long steps = 0;
    while (x > 1) {
        if ((x % 2) == 0) x = x / 2;
        else x = 3 * x + 1;
        steps = steps + 1;
    }
    return steps;
}
int main(void) {
    long f12 = fib(12);
    long c27 = collatz_steps(27);
    printf("FIB12=%ld COLLATZ27=%ld SUM=%ld\n", f12, c27, f12 + c27);
    return 0;
}
"#;
    let mut vm = MvmSandbox::boot_default().expect("boot");
    let elf = compile_c_units_to_rv64_elf(
        &[("/workspace/src/bench.c", source)],
        &["/workspace/src".to_string(), "/usr/include".to_string()],
        &[],
        |hdr| {
            vm.read_guest_file(hdr)
                .and_then(|b| String::from_utf8(b).ok())
        },
    )
    .expect("C compiles to RV64 ELF");
    assert_eq!(&elf[..4], b"\x7fELF");
    assert_eq!(u16::from_le_bytes([elf[18], elf[19]]), 243);
    vm.write_guest_file("/workspace/src/bench.c", source.as_bytes())
        .expect("write C source");
    let outcome = vm
        .exec_command(
            "cc /workspace/src/bench.c -o /workspace/bench && /workspace/bench",
            "/workspace",
        )
        .expect("guest cc and executable run");
    assert_eq!(outcome.exit_code, 0, "{}", outcome.stderr);
    assert_eq!(outcome.compiled_elf_machine, Some(243));
    assert_eq!(outcome.stdout, "FIB12=144 COLLATZ27=111 SUM=255\n");
    assert!(outcome.stderr.is_empty());
    assert!(outcome.instructions_retired > 0);

    let second = r#"
#include <stdio.h>
int main(void) {
    long answer = 6 * 7;
    printf("ANSWER=%ld\n", answer);
    return 0;
}
"#;
    vm.write_guest_file("/workspace/answer.c", second.as_bytes())
        .expect("write second C source");
    let second_out = vm
        .exec_command(
            "cc /workspace/answer.c -o /workspace/answer && /workspace/answer",
            "/workspace",
        )
        .expect("second C program executes");
    assert_eq!(second_out.exit_code, 0, "{}", second_out.stderr);
    assert_eq!(second_out.stdout, "ANSWER=42\n");
    assert!(second_out.stderr.is_empty());
    assert_eq!(second_out.compiled_elf_machine, Some(243));
    assert!(second_out.instructions_retired > 0);
}

#[test]
fn mvm_invalid_c_is_refused_without_panic() {
    let error =
        compile_c_to_rv64_elf("int main( { this is not C").expect_err("invalid C is refused");
    assert!(!error.message.is_empty());
}

#[test]
fn mvm_vsock_admits_only_bounded_frames_and_placeholders() {
    let mut vm = MvmSandbox::boot_default().expect("boot");
    let mut receiver = MvmSandbox::boot_with_cid(4).expect("boot receiver");
    let packet = vm
        .build_vsock_packet(4, 4096, 19, 7, b"VERIFY_DIFF_SHA256:cafe1234")
        .expect("valid packet");
    receiver
        .deliver_vsock_packet(&packet)
        .expect("deliver packet");
    let received = receiver.pop_vsock_inbox(4096).expect("inbox packet");
    assert_eq!(received.header.src_cid, 3);
    assert_eq!(received.header.dst_cid, 4);
    assert_eq!(received.payload, b"VERIFY_DIFF_SHA256:cafe1234");

    let synthetic = "synthetic-test-secret-not-a-token";
    vm.register_host_secret("TEST_ONLY", synthetic, &["api.github.com"]);
    let denied = vm.proxy_vsock_egress(
        "evil.invalid",
        "Authorization: Bearer {{MVM_SECRET:TEST_ONLY}}",
    );
    assert_eq!(denied.verdict, EgressVerdict::RefusedHostNotAllowlisted);
    assert!(denied.outbound_payload.is_none());
    let allowed = vm.proxy_vsock_egress(
        "api.github.com",
        "Authorization: Bearer {{MVM_SECRET:TEST_ONLY}}",
    );
    assert_eq!(allowed.verdict, EgressVerdict::ProxyAdmitted);
    assert_eq!(
        allowed.outbound_payload.as_deref(),
        Some("Authorization: Bearer synthetic-test-secret-not-a-token")
    );
    assert!(!vm.guest_ram_contains(synthetic.as_bytes()));
}

#[test]
fn mvm_invalid_elf_cid_domain_and_secret_are_refused() {
    assert!(MvmSandbox::boot_with_cid(2).is_err());
    assert_eq!(
        admit_guest_socket_domain(40),
        SocketAdmission::AdmittedVsock
    );
    assert_eq!(
        admit_guest_socket_domain(2),
        SocketAdmission::RefusedNicDisabled
    );
    assert_eq!(
        admit_guest_socket_domain(999),
        SocketAdmission::RefusedUnsupportedDomain
    );
    assert_eq!(admit_elf64_riscv(b"NOPE"), ElfAdmission::RefusedBadMagic);

    let vm = MvmSandbox::boot_default().expect("boot");
    let mut wrong_machine = vm.read_guest_file("/sbin/mvm-agent").expect("agent");
    wrong_machine[18] = 62;
    wrong_machine[19] = 0;
    assert_eq!(
        admit_elf64_riscv(&wrong_machine),
        ElfAdmission::RefusedWrongMachine
    );

    let self_loop = SpiralCoreHeader {
        magic: SPIRALCORE_MAGIC,
        src_cid: 3,
        dst_cid: 3,
        vsock_port: 1024,
        cl06_spin: 0,
        e8_orbit: 0,
        payload_len: 16,
    };
    assert_eq!(
        admit_spiralcore_header(&self_loop, 65_536),
        FrameVerdict::RefusedInvalidCid
    );
    assert!(vm
        .build_vsock_packet(4, 4096, 0, 0, &vec![0; 65_537])
        .is_err());
}

#[test]
fn cow_restore_replays_sector_allocation_and_preserves_files() {
    let mut original = MvmSandbox::boot_default().expect("boot");
    original
        .write_guest_file("/workspace/first", b"first")
        .unwrap();
    let snapshot = original.snapshot_cow();
    original
        .write_guest_file("/workspace/second", b"second")
        .unwrap();
    let expected = original.export_dirty_cow_sectors();
    original
        .write_guest_file("/workspace/third", b"third")
        .unwrap();

    let mut fresh = MvmSandbox::boot_default().expect("fresh boot");
    for vm in [&mut original, &mut fresh] {
        vm.restore_cow(&snapshot);
        vm.write_guest_file("/workspace/second", b"second").unwrap();
        assert_eq!(
            vm.read_guest_file("/workspace/first").as_deref(),
            Some(b"first".as_slice())
        );
        assert_eq!(
            vm.read_guest_file("/workspace/second").as_deref(),
            Some(b"second".as_slice())
        );
        assert!(vm.read_guest_file("/workspace/third").is_none());
        assert_eq!(vm.export_dirty_cow_sectors(), expected);
    }
}

#[test]
fn vsock_delivers_exact_limit_and_refuses_padding() {
    use crate::mvm::{MvmError, SPIRALCORE_MAX_PAYLOAD};
    let sender = MvmSandbox::boot_default().expect("boot");
    let mut receiver = MvmSandbox::boot_with_cid(4).expect("boot receiver");
    let payload = vec![b'x'; SPIRALCORE_MAX_PAYLOAD];
    let packet = sender.build_vsock_packet(4, 4096, 0, 0, &payload).unwrap();
    receiver
        .deliver_vsock_packet(&packet)
        .expect("deliver exact limit");
    assert_eq!(receiver.pop_vsock_inbox(4096).unwrap().payload, payload);
    let mut padded = sender.build_vsock_packet(4, 4096, 0, 0, b"note").unwrap();
    padded.push(0);
    assert!(matches!(
        receiver.deliver_vsock_packet(&padded),
        Err(MvmError::FrameRejected(
            FrameVerdict::RefusedPayloadTooLarge
        ))
    ));
    assert!(receiver.pop_vsock_inbox(4096).is_none());
}
