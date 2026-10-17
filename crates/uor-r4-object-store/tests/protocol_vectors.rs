//! Protocol v1 conformance vectors. A port can reproduce these exactly.

use uor_r4_object_store::{Limits, Store};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn limits() -> Limits {
    Limits {
        max_object_bytes: 1 << 30,
        max_records: 1_000_000,
        max_record_bytes: 1 << 30,
        max_returned_bytes: 1 << 30,
        max_visits: 100_000_000,
        max_occurrences: 10_000_000,
    }
}

#[test]
fn protocol_v1_vectors_are_stable() {
    let payload: Vec<u8> = (0u8..96)
        .map(|i| i.wrapping_mul(7).wrapping_add(3))
        .collect();
    assert_eq!(
        hex(&payload),
        "030a11181f262d343b424950575e656c737a81888f969da4abb2b9c0c7ced5dce3eaf1f8ff060d141b222930373e454c535a61686f767d848b9299a0a7aeb5bcc3cad1d8dfe6edf4fb020910171e252c333a41484f565d646b727980878e959c"
    );
    let mut store = Store::new(limits());
    let manifest = store.fold(&payload).expect("fold").manifest;
    assert_eq!(manifest.byte_len(), 96);
    assert_eq!(
        hex(&manifest.whole_sha256()),
        "c9f1a5f79d7bea01a54f4edb41673722f627ee2e82dda324946b63cf4b9b16af"
    );
    assert_eq!(
        hex(&manifest.encode()),
        "554f524f424a30316000000000000000c9f1a5f79d7bea01a54f4edb41673722f627ee2e82dda324946b63cf4b9b16af0180ae70cded63205a6ec66dc801"
    );
    assert_eq!(
        hex(&manifest.version()),
        "924abee83e5bafbeed9d712604081105fde9e7bb1f71eb6405c2bdb522e7968e"
    );
    assert_eq!(manifest.roots().len(), 1);
    let (root_id, root_scale) = manifest.roots()[0];
    assert_eq!(root_scale, 1);
    assert_eq!(hex(&root_id), "80ae70cded63205a6ec66dc8");
    let root_record = store.record_bytes(root_id).expect("root record").to_vec();
    assert_eq!(
        hex(&root_record),
        "0180ae70cded63205a6ec66dc8cce0826423d1cf8912a7863de5eef536d8dd6b28bc323ca17dddcb6d6ee0fbd7694edd07"
    );
    let mut id = root_id;
    loop {
        let record = store.record_bytes(id).expect("record").to_vec();
        if record[0] == 0 {
            assert_eq!(hex(&id), "cce0826423d1cf8912a7863d");
            assert_eq!(
                hex(&record),
                "00cce0826423d1cf8912a7863d030a11181f262d343b424950575e656c737a81888f969da4abb2b9c0c7ced5dc"
            );
            break;
        }
        let mut child = [0u8; 12];
        child.copy_from_slice(&record[13..25]);
        id = child;
    }
}
