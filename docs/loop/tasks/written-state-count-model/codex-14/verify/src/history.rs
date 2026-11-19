//! Exact merged claude-433 receipts, with the independent plain KN controls.
use super::{ensure, sha, Failure, Score};
use serde_json::Value;

struct Packet {
    hashes: [&'static str; 2],
    bytes: [&'static [u8]; 2],
}

const PACKETS: [Packet; 4] = [
    Packet {
        hashes: [
            "42aefc3696499fee32d499d12a6c8b5ea807eb0b34df2332b69111cb5425315e",
            "e43ab96be8b63c9001bad20c807e8256bf2f675b4530831124dbe726ccf3d8ea",
        ],
        bytes: [
            include_bytes!("../../../claude-433/run1/c433r/ceiling-p1.json"),
            include_bytes!("../../../claude-433/run1/c433r/plain-p1.json"),
        ],
    },
    Packet {
        hashes: [
            "a1701713be1fe5fd51ad6f7dc5558c215bf7c8ace93e86d5c7f961ce7c798117",
            "125bd607c2413113cad4b394b836d049bd1fb344711266eafd8931b9b7bf1623",
        ],
        bytes: [
            include_bytes!("../../../claude-433/run1/c433r/ceiling-p10.json"),
            include_bytes!("../../../claude-433/run1/c433r/plain-p10.json"),
        ],
    },
    Packet {
        hashes: [
            "3b86d50aaf055f0b7defaa59bf7d902283b4c3ca5e78f5b523a875a415f2a1a2",
            "b6ad0de41e865c6892c7a5f217ee09977c4561d3f2f189dc31eafb9fd99ab4c0",
        ],
        bytes: [
            include_bytes!("../../../claude-433/run1/c433r/ceiling-p100.json"),
            include_bytes!("../../../claude-433/run1/c433r/plain-p100.json"),
        ],
    },
    Packet {
        hashes: [
            "3ff2cc691daa7fcf8dbfddba1648174ec3080d420e34a676acfa6958bc9df124",
            "e30ace0193df0f744a06c9b8d8d0de5f8bb41d36138c24c257dab926a60dab55",
        ],
        bytes: [
            include_bytes!("../../../claude-433/run1/c433p/ceiling-p1000.json"),
            include_bytes!("../../../claude-422/run1/sweep-10.json"),
        ],
    },
];

pub(super) fn read(per_mille: u64) -> Result<Value, Failure> {
    let packet = match per_mille {
        1 => &PACKETS[0],
        10 => &PACKETS[1],
        100 => &PACKETS[2],
        1000 => &PACKETS[3],
        _ => {
            return Err(
                format!("no registered historical receipt for {per_mille} per mille").into(),
            )
        }
    };
    for (bytes, expected) in packet.bytes.iter().zip(packet.hashes) {
        ensure(sha(bytes) == expected, "historical receipt digest")?;
    }
    let ceiling: Value = serde_json::from_slice(packet.bytes[0])?;
    let plain: Value = serde_json::from_slice(packet.bytes[1])?;
    for field in [
        "inputs",
        "masses_sha256",
        "highest_order",
        "budget",
        "all",
        "novel",
    ] {
        ensure(
            ceiling[field] == plain[field],
            format!("historical plain KN {field}"),
        )?;
    }
    for (order, (ceil, plain)) in super::array(&ceiling["orders"])?
        .iter()
        .zip(super::array(&plain["orders"])?)
        .enumerate()
    {
        for key in ["triple_sha256", "context_sha256"] {
            ensure(
                ceil[key] == plain[key],
                format!("historical plain order {order} {key}"),
            )?;
        }
    }
    Ok(ceiling)
}

pub(super) fn score(receipt: &Value) -> Result<Score, Failure> {
    let tally = |class: &str| {
        let row = &receipt[class];
        Ok::<_, Failure>(super::Tally {
            targets: super::number(&row["targets"])?,
            bytes: super::number(&row["bytes"])?,
            bits_q12: super::number(&row["bits_q12"])?,
        })
    };
    Ok(Score {
        all: tally("all")?,
        novel: tally("novel")?,
    })
}
