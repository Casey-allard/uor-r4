use uor_r4_mvm::{guest_identity, run_collatz};

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
