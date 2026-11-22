use uor_r4_mvm::{guest_identity, run_collatz, MvmSandbox};

#[test]
fn public_api_checks_identity_and_rejects_invalid_limits() {
    assert_eq!(guest_identity().unwrap(), ("riscv64-rv64imac", 3));
    for limit in [0, 513, u16::MAX] {
        assert!(run_collatz(limit).is_err());
    }
}

#[test]
fn every_supported_limit_matches_the_independent_oracle() {
    let mut best = (1, 0);
    for limit in 1..=512 {
        let mut value = u64::from(limit);
        let mut steps = 0;
        while value > 1 {
            value = if value % 2 == 0 {
                value / 2
            } else {
                3 * value + 1
            };
            steps += 1;
        }
        if steps > best.1 {
            best = (limit, steps);
        }
        let expected = format!("limit={limit} start={} steps={}\n", best.0, best.1);
        assert_eq!(run_collatz(limit).unwrap(), expected);
    }
}

#[test]
fn sandbox_shell_pipes_heredoc_jq_and_cwd_persist() {
    let mut vm = MvmSandbox::new().expect("boot MvmSandbox");
    assert_eq!(vm.guest_identity(), ("riscv64-rv64imac", 3));
    assert_eq!(vm.cwd(), "/workspace");

    let uname = vm.exec("uname -a", "");
    assert_eq!(uname.exit_code, 0, "stderr: {}", uname.stderr);
    assert!(uname.stdout.contains("riscv64"), "{}", uname.stdout);

    let heredoc_and_jq = vm.exec(
        "mkdir -p /workspace/sub && cd /workspace/sub && cat <<'EOF' > data.json\n{\"items\":[{\"name\":\"r4\",\"n\":7},{\"name\":\"mvm\",\"n\":42}]}\nEOF\njq '.items[1].n' data.json",
        "",
    );
    assert_eq!(
        heredoc_and_jq.exit_code, 0,
        "stderr: {}",
        heredoc_and_jq.stderr
    );
    assert_eq!(heredoc_and_jq.cwd, "/workspace/sub");
    assert_eq!(vm.cwd(), "/workspace/sub");
    assert_eq!(heredoc_and_jq.stdout.trim(), "42");

    let pipe_out = vm.exec("cat data.json | tr a-z A-Z | wc -c", "");
    assert_eq!(pipe_out.exit_code, 0, "stderr: {}", pipe_out.stderr);
    let len: usize = pipe_out.stdout.trim().parse().expect("wc -c integer");
    assert!(len > 40);
}

#[test]
fn sandbox_virtual_files_and_protected_paths_enforced() {
    let mut vm = MvmSandbox::new().expect("boot MvmSandbox");
    vm.write_file("/workspace/hello.txt", b"uor-r4-mvm\n")
        .expect("write regular file");
    assert_eq!(
        vm.read_file("/workspace/hello.txt").as_deref(),
        Some(b"uor-r4-mvm\n".as_slice())
    );
    let listed = vm.list_files("/workspace");
    assert!(listed.iter().any(|p| p == "/workspace/hello.txt"));

    vm.remove_file("/workspace/hello.txt")
        .expect("remove regular file");
    assert!(vm.read_file("/workspace/hello.txt").is_none());

    assert!(vm
        .write_file("/workspace/prism/app.lex.tex", b"tamper")
        .is_err());
    assert!(vm.write_file("/workspace/app/state", b"tamper").is_err());
    assert!(vm.write_file("/sbin/mvm-agent", b"tamper").is_err());
    assert!(vm.write_file("/bin/mvmbox", b"tamper").is_err());
    assert!(vm.write_file("/usr/include/stdio.h", b"tamper").is_err());
    assert!(vm.remove_file("/sbin/mvm-agent").is_err());
    assert!(vm.remove_file("/bin/mvmbox").is_err());
    assert!(vm.remove_file("/usr/include/stdio.h").is_err());

    assert_ne!(vm.exec("rm /sbin/mvm-agent", "").exit_code, 0);
    assert_ne!(vm.exec("rm -rf /usr", "").exit_code, 0);
    vm.write_file("/workspace/src.txt", b"x").unwrap();
    assert_ne!(
        vm.exec("mv /workspace/src.txt /sbin/mvm-agent", "")
            .exit_code,
        0
    );
    assert_ne!(vm.exec("echo evil > /bin/mvmbox", "").exit_code, 0);
    assert_ne!(vm.exec("ln -sf /tmp/x /bin/mvmbox", "").exit_code, 0);
    assert_ne!(vm.exec("chmod 000 /sbin/mvm-agent", "").exit_code, 0);
    assert_ne!(vm.exec("touch /etc/evil", "").exit_code, 0);
    assert_ne!(vm.exec("mkdir /bin/evil", "").exit_code, 0);
    assert_ne!(vm.exec("rmdir /usr/bin", "").exit_code, 0);
    vm.write_file("/workspace/sub/f.txt", b"y").unwrap();
    assert_ne!(vm.exec("cp -r /workspace/sub /bin/sub", "").exit_code, 0);
    assert_ne!(vm.exec("ln -s /workspace/bin /", "").exit_code, 0);
    assert_ne!(vm.exec("ln -s / /workspace", "").exit_code, 0);
    assert_ne!(vm.exec("ln -s /workspace/. /workspace", "").exit_code, 0);
    assert_ne!(vm.exec("ln -s x/.. /workspace", "").exit_code, 0);
    assert_eq!(vm.exec("mkdir -p /workspace/bin", "").exit_code, 0);
    assert_ne!(vm.exec("mv /workspace/bin /", "").exit_code, 0);
    assert_ne!(vm.exec("cp -r /workspace/bin /", "").exit_code, 0);
    assert!(vm.read_file("/sbin/mvm-agent").is_some());
    assert!(vm.read_file("/bin/mvmbox").is_some());
    assert!(vm.read_file("/etc/evil").is_none());
}

#[test]
fn sandbox_cow_snapshot_roundtrips_across_fresh_instance_and_rejects_tamper() {
    let mut vm = MvmSandbox::new().expect("boot MvmSandbox");
    vm.write_file("/workspace/persist.txt", b"before-snapshot\n")
        .expect("write persist.txt");
    let cd_res = vm.exec(
        "mkdir -p /workspace/dir && cd /workspace/dir && touch marker.txt",
        "",
    );
    assert_eq!(cd_res.exit_code, 0, "{}", cd_res.stderr);

    let mut snap = vm.export_cow_snapshot().expect("export UORCOW01 snapshot");
    assert_eq!(&snap[..8], b"UORCOW01");

    let mut fresh = MvmSandbox::new().expect("boot fresh MvmSandbox");
    fresh
        .import_cow_snapshot(&snap)
        .expect("restore UORCOW01 snapshot");
    assert_eq!(fresh.cwd(), "/workspace/dir");
    assert_eq!(
        fresh.read_file("/workspace/persist.txt").as_deref(),
        Some(b"before-snapshot\n".as_slice())
    );
    assert!(fresh.read_file("/workspace/dir/marker.txt").is_some());

    // Tampering anywhere inside wire payload OR metadata (e.g. current_cwd byte) must fail verification.
    let meta_pos = snap
        .windows(b"/workspace/dir".len())
        .position(|w| w == b"/workspace/dir")
        .expect("metadata contains cwd");
    let mut cwd_tampered = snap.clone();
    cwd_tampered[meta_pos + b"/workspace/".len()] = b'x';
    let mut untouched_cwd = MvmSandbox::new().expect("boot untouched");
    assert!(untouched_cwd.import_cow_snapshot(&cwd_tampered).is_err());

    let last = snap.len() - 1;
    snap[last] ^= 0xff;
    let mut untouched = MvmSandbox::new().expect("boot untouched");
    assert!(untouched.import_cow_snapshot(&snap).is_err());
}

#[test]
fn sandbox_compiles_two_c_programs_and_fixes_compiler_prerequisites() {
    let mut vm = MvmSandbox::new().expect("boot MvmSandbox");
    let prog_a =
        "#include <stdio.h>\nint main(void) { printf(\"SUM=%d\\n\", 19 + 23); return 0; }\n";
    let prog_b = "#include <stdio.h>\nint main(void) { long s = 0; for (long i = 1; i <= 10; i++) s += i; printf(\"TRI=%ld\\n\", s); return 0; }\n";

    vm.write_file("/workspace/a.c", prog_a.as_bytes()).unwrap();
    vm.write_file("/workspace/b.c", prog_b.as_bytes()).unwrap();

    let out_a = vm.exec(
        "cc /workspace/a.c -o /workspace/a && /workspace/a",
        "/workspace",
    );
    assert_eq!(out_a.exit_code, 0, "{}", out_a.stderr);
    assert_eq!(out_a.stdout, "SUM=42\n");
    assert_eq!(out_a.compiled_elf_machine, Some(243));
    assert!(out_a.instructions_retired > 0);

    let out_b = vm.exec(
        "cc /workspace/b.c -o /workspace/b && /workspace/b",
        "/workspace",
    );
    assert_eq!(out_b.exit_code, 0, "{}", out_b.stderr);
    assert_eq!(out_b.stdout, "TRI=55\n");
    assert_eq!(out_b.compiled_elf_machine, Some(243));
    assert!(out_b.instructions_retired > 0);

    // Covers the 3 C compiler prerequisites: global arrays, nested lexical scopes, and printf count.
    let prereq_c = r#"
#include <stdio.h>
long g_arr[4];
int main(void) {
    g_arr[0] = 10;
    g_arr[1] = 20;
    g_arr[2] = g_arr[0] + g_arr[1];
    long outer = 100;
    {
        long outer = 7;
        g_arr[3] = outer;
    }
    int n = printf("G=%ld,%ld,%ld OUTER=%ld INNER=%ld\n", g_arr[0], g_arr[1], g_arr[2], outer, g_arr[3]);
    printf("N=%d\n", n);
    return 0;
}
"#;
    vm.write_file("/workspace/prereq.c", prereq_c.as_bytes())
        .unwrap();
    let prereq_out = vm.exec(
        "cc /workspace/prereq.c -o /workspace/prereq && /workspace/prereq",
        "/workspace",
    );
    assert_eq!(prereq_out.exit_code, 0, "{}", prereq_out.stderr);
    let expected_first = "G=10,20,30 OUTER=100 INNER=7\n";
    assert_eq!(
        prereq_out.stdout,
        format!("{expected_first}N={}\n", expected_first.len())
    );

    let cargo_res = vm.exec("cargo check", "/workspace");
    assert_eq!(cargo_res.exit_code, 127);
    assert!(
        cargo_res.stderr.contains("unavailable"),
        "{}",
        cargo_res.stderr
    );

    let rustc_res = vm.exec("rustc /workspace/a.c", "/workspace");
    assert_eq!(rustc_res.exit_code, 127);
    assert!(
        rustc_res.stderr.contains("unavailable"),
        "{}",
        rustc_res.stderr
    );
}

#[test]
fn sandbox_utf8_paths_tar_slicing_sector_reclamation_and_malformed_git_are_safe() {
    let mut vm = MvmSandbox::new().expect("boot MvmSandbox");

    let utf8_res = vm.exec(
        "mkdir -p /workspace/café && echo héllo > /workspace/café/naïve.txt && cat /workspace/café/naïve.txt",
        "/workspace",
    );
    assert_eq!(utf8_res.exit_code, 0, "{}", utf8_res.stderr);
    assert_eq!(utf8_res.stdout, "héllo\n");

    let printf_utf8 = vm.exec("printf '%é' 1", "/workspace");
    assert_eq!(printf_utf8.exit_code, 0, "{}", printf_utf8.stderr);
    let tr_utf8 = vm.exec("echo abc | tr é x", "/workspace");
    assert_eq!(tr_utf8.exit_code, 0, "{}", tr_utf8.stderr);
    let jq_utf8 = vm.exec("echo '{\"café\":7}' | jq '.café'", "/workspace");
    assert_eq!(jq_utf8.exit_code, 3);
    let jq_utf8_key = vm.exec("echo '{\"café\":7}' | jq '.[\"café\"]'", "/workspace");
    assert_eq!(jq_utf8_key.exit_code, 0, "{}", jq_utf8_key.stderr);
    assert_eq!(jq_utf8_key.stdout.trim(), "7");

    // Multi-byte UTF-8 filename longer than 100 bytes spanning byte 100 must not panic in tar.
    let long_utf8 = format!("{}.txt", "café_naïve_".repeat(12));
    assert!(long_utf8.len() > 100);
    vm.write_file(&format!("/workspace/{long_utf8}"), b"tar-utf8\n")
        .unwrap();
    let tar_res = vm.exec(
        &format!("tar -cf /workspace/archive.tar /workspace/{long_utf8} && tar -tf /workspace/archive.tar"),
        "/workspace",
    );
    assert_eq!(tar_res.exit_code, 0, "{}", tar_res.stderr);

    // Overwriting and gzip-cycling a file 50 times must reclaim superseded sectors instead of growing unboundedly.
    for i in 0..50u32 {
        vm.write_file("/workspace/churn.txt", format!("iter-{i}\n").as_bytes())
            .unwrap();
        let gz = vm.exec(
            "gzip /workspace/churn.txt && gunzip /workspace/churn.txt.gz",
            "/workspace",
        );
        assert_eq!(gz.exit_code, 0, "{}", gz.stderr);
    }
    vm.remove_file("/workspace/churn.txt").unwrap();
    let snap = vm.export_cow_snapshot().unwrap();
    assert!(
        snap.len() < 128 * 1024,
        "snapshot retained superseded sectors: {} bytes",
        snap.len()
    );

    // Malformed .git directory without uor_state.json must return exit 128 without panicking.
    let git_res = vm.exec("mkdir -p /workspace/.git && git status", "/workspace");
    assert_eq!(git_res.exit_code, 128, "{}", git_res.stderr);

    // Guest C program attempting protected open(O_WRONLY) or open(O_CREAT) must fail with -EACCES (-13).
    let sys_c = r#"
#include <stdio.h>
int main(void) {
    long fd1 = open("/sbin/mvm-agent", 1, 0);
    long fd2 = open("/bin/evil_c", 65, 420);
    printf("FD1=%ld FD2=%ld\n", fd1, fd2);
    return 0;
}
"#;
    vm.write_file("/workspace/sysguard.c", sys_c.as_bytes())
        .unwrap();
    let sys_out = vm.exec(
        "cc /workspace/sysguard.c -o /workspace/sysguard && /workspace/sysguard",
        "/workspace",
    );
    assert_eq!(sys_out.exit_code, 0, "{}", sys_out.stderr);
    assert_eq!(sys_out.stdout, "FD1=-13 FD2=-13\n");
    assert!(vm.read_file("/bin/evil_c").is_none());

    let mem_c = r#"
#include <stdio.h>
int main(void) {
    char *p = (char *)0x100000000UL;
    printf("%c\n", *p);
    return 0;
}
"#;
    vm.write_file("/workspace/fault.c", mem_c.as_bytes())
        .unwrap();
    let fault_out = vm.exec(
        "cc /workspace/fault.c -o /workspace/fault && /workspace/fault",
        "/workspace",
    );
    assert_ne!(fault_out.exit_code, 0);
}
