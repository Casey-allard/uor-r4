use std::process::{Command, Output};

fn invoke(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_uor-mvm"))
        .args(args)
        .output()
        .expect("execute the native workbench")
}

#[test]
fn status_boots_the_pinned_guest() {
    let output = invoke(&["status"]);
    assert!(output.status.success());
    assert_eq!(
        output.stdout,
        b"architecture=riscv64-rv64imac guest_cid=3\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn guest_output_matches_reference_stopping_times() {
    for limit in [1_u64, 2, 19, 27, 100, 512] {
        let mut best = (1, 0);
        for start in 1..=limit {
            let mut n = start;
            let mut steps = 0;
            while n != 1 {
                n = if n % 2 == 0 { n / 2 } else { 3 * n + 1 };
                steps += 1;
            }
            if steps > best.1 {
                best = (start, steps);
            }
        }
        let result = invoke(&["collatz", &limit.to_string()]);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(
            result.stdout,
            format!("limit={limit} start={} steps={}\n", best.0, best.1).as_bytes()
        );
        assert!(result.stderr.is_empty());
    }
}

#[test]
fn rejects_input_outside_the_workload_contract() {
    for args in [
        &[][..],
        &["status", "extra"],
        &["collatz"],
        &["collatz", "0"],
        &["collatz", "513"],
        &["collatz", "x"],
        &["cc", "user.c"],
    ] {
        let output = invoke(args);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}
