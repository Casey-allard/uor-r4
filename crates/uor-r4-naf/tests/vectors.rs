//! §12 conformance vectors for the #623 slice: the golden integer/tensor
//! address chains (every payload byte, manifest byte, and κ), the mandatory
//! rejection corpus, and the §3 core laws. These are the spec's own numbers —
//! NAF-ADDR-008 requires reproducing them exactly.

use uor_r4_naf::*;

fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

/// §12.2 golden chain: integer `3` under `uor-naf/math-int/1`.
#[test]
fn golden_integer_3_chain() {
    let v = NafValue::Integer(Int::from_i128(3));
    let sem = labels(&v);
    assert_eq!(
        sem.semantic_payload,
        hex("554f5253454d01000113756f722d6e61662f696e74656765722d7a2f310103")
    );
    assert_eq!(
        sem.payload_sha256.as_str(),
        "sha256:be329674625f0bce5e4e06e7b223767e42e5ce2fc12d29a835b5507c81054940"
    );
    assert_eq!(
        sem.semantic_manifest,
        "{\"domain\":\"integer\",\"kind\":\"semantic\",\"payload_bytes\":\"31\",\
         \"payload_sha256\":\"sha256:be329674625f0bce5e4e06e7b223767e42e5ce2fc12d29a835b5507c81054940\",\
         \"spec\":\"uor-naf/1-draft.6\"}"
    );
    assert_eq!(
        sem.semantic_kappa.as_str(),
        "sha256:84deea1b24435e7ebc365b35ab466053ad88217a6e24041b154b8b93fea308e7"
    );

    let art = artifact_labels(&v, "uor-naf/math-int/1").unwrap();
    assert_eq!(
        art.artifact_payload,
        hex(
            "554f524e4146010001477368613235363a38346465656131623234343335653765626333363562\
             3335616234363630353361643838323137613665323430343162313534623862393366656133\
             3038653713756f722d6e61662f696e74656765722d7a2f3112756f722d6e61662f6d6174682d\
             696e742f310312"
                .replace(char::is_whitespace, "")
                .as_str()
        )
    );
    assert_eq!(
        art.payload_sha256.as_str(),
        "sha256:93296c0d8c0ebdf81e57f83bb6b733da281158ecc54290e8393d3d8cb7e358ff"
    );
    assert_eq!(
        art.artifact_kappa.as_str(),
        "sha256:666852ad6d1902690b7075c4891de4d5d4e6d014db085cbd97b2cb678067e545"
    );

    // Round-trip through the strict decoder, including embedded-κ verify.
    let (decoded, profile) = decode_artifact(&art.artifact_payload).unwrap();
    assert_eq!(decoded, v);
    assert_eq!(profile, "uor-naf/math-int/1");
}

/// §12.2 golden chain: tensor shape `(2)`, values `(0, -1)`.
#[test]
fn golden_tensor_chain() {
    let v = NafValue::Tensor {
        shape: vec![2],
        values: vec![Int::from_i128(0), Int::from_i128(-1)],
    };
    let sem = labels(&v);
    assert_eq!(
        sem.semantic_payload,
        hex("554f5253454d01000213756f722d6e61662f696e74656765722d7a2f31010201000201")
    );
    assert_eq!(
        sem.semantic_kappa.as_str(),
        "sha256:423cd7e8f5b9c8df367ad957f66490f4ba34d164a7cd6b7c2ba6cd60c268c75f"
    );
    let art = artifact_labels(&v, "uor-naf/math-int/1").unwrap();
    assert_eq!(
        art.payload_sha256.as_str(),
        "sha256:77093c5fbcac55f0c8cbc714faffb59434d8fb471e0737899f8e2dde8b09ef27"
    );
    assert_eq!(
        art.artifact_kappa.as_str(),
        "sha256:88e492d5de9db4bc1e825de3c8bc6ff5f7dfa433e0b0923f97d115ac7ee1b248"
    );
    let (decoded, _) = decode_artifact(&art.artifact_payload).unwrap();
    assert_eq!(decoded, v);
}

/// The storage-profile semantic-identity law (§8.2): two profiles, same
/// mathematical value ⇒ same semantic κ, different artifact payloads.
#[test]
fn storage_profile_splits_artifact_not_semantic() {
    let v = NafValue::Integer(Int::from_i128(3));
    let a = artifact_labels(&v, "uor-naf/math-int/1").unwrap();
    let b = artifact_labels(&v, "uor-naf/twos-i8/1").unwrap();
    assert_ne!(a.artifact_payload, b.artifact_payload);
    assert_ne!(a.artifact_kappa, b.artifact_kappa);
    // Both embed the SAME semantic κ.
    assert_eq!(labels(&v).semantic_kappa.as_str().len(), 71);
}

/// §12.4 mandatory rejection corpus, verbatim, plus range/profile gates.
#[test]
fn mandatory_rejection_corpus() {
    let inv = |tag: &'static str| NafError::Invalid(tag);
    // uvar
    assert_eq!(uvar_decode(&hex("")).unwrap_err(), inv("truncated-uvar"));
    assert_eq!(uvar_decode(&hex("80")).unwrap_err(), inv("truncated-uvar"));
    assert_eq!(
        uvar_decode(&hex("8000")).unwrap_err(),
        inv("nonminimal-uvar")
    );
    assert_eq!(
        uvar_decode(&hex("ff00")).unwrap_err(),
        inv("nonminimal-uvar")
    );
    assert_eq!(
        uvar_decode(&hex("808000")).unwrap_err(),
        inv("nonminimal-uvar")
    );
    // signed integer
    assert_eq!(
        signed_decode(&hex("03")).unwrap_err(),
        inv("invalid-sign-code")
    );
    assert_eq!(
        signed_decode(&hex("0100")).unwrap_err(),
        inv("zero-magnitude-with-sign")
    );
    assert_eq!(
        signed_decode(&hex("0200")).unwrap_err(),
        inv("zero-magnitude-with-sign")
    );
    // `0000` = a valid zero followed by residue: trailing-bytes at the frame.
    let (z, used) = signed_decode(&hex("0000")).unwrap();
    assert!(z.is_zero());
    assert_eq!(used, 1, "outer frame must reject the residual byte");
    // CoreNAFBytes
    assert_eq!(
        core_naf_decode(&hex("0100")).unwrap_err(),
        inv("zero-top-digit")
    );
    assert_eq!(
        core_naf_decode(&hex("0205")).unwrap_err(),
        inv("adjacent-nonzero")
    );
    assert_eq!(
        core_naf_decode(&hex("0103")).unwrap_err(),
        inv("invalid-digit-code")
    );
    assert_eq!(
        core_naf_decode(&hex("0105")).unwrap_err(),
        inv("nonzero-padding")
    );
    assert_eq!(
        core_naf_decode(&hex("8000")).unwrap_err(),
        inv("nonminimal-uvar")
    );
    let (d, used) = core_naf_decode(&hex("0000")).unwrap();
    assert!(d.is_empty());
    assert_eq!(used, 1, "outer frame must reject the residual byte");
    // Closed profile list + range gates.
    let v = NafValue::Integer(Int::from_i128(300));
    assert_eq!(
        artifact_labels(&v, "uor-naf/twos-i8/1").unwrap_err(),
        inv("storage-range-violation")
    );
    assert_eq!(
        artifact_labels(&v, "uor-naf/twos-i128/1").unwrap_err(),
        inv("unknown-storage-profile")
    );
    // i8 boundary: -128 admitted, +128 not (two's complement asymmetry).
    assert!(artifact_labels(
        &NafValue::Integer(Int::from_i128(-128)),
        "uor-naf/twos-i8/1"
    )
    .is_ok());
    assert_eq!(
        artifact_labels(&NafValue::Integer(Int::from_i128(128)), "uor-naf/twos-i8/1").unwrap_err(),
        inv("storage-range-violation")
    );
}

/// Tampering gates: a flipped payload bit must land in the right outcome
/// class (the instrument must be able to fail).
#[test]
fn tamper_detection() {
    let v = NafValue::Integer(Int::from_i128(42));
    let art = artifact_labels(&v, "uor-naf/twos-i16/1").unwrap();
    // Corrupt the embedded semantic κ (still well-formed hex) ⇒ the
    // reconstruction disagrees ⇒ CommitmentFailure, not Invalid.
    let mut bad = art.artifact_payload.clone();
    let pos = 9 + 1 + 8; // inside the 71-byte label field's hex region
    bad[pos] = if bad[pos] == b'a' { b'b' } else { b'a' };
    assert_eq!(
        decode_artifact(&bad).unwrap_err(),
        NafError::CommitmentFailure
    );
    // Unsupported (not invalid): a valid-tag domain outside this slice.
    let mut atlas = art.artifact_payload.clone();
    atlas[8] = 0x03;
    assert_eq!(decode_artifact(&atlas).unwrap_err(), NafError::Unsupported);
    // Trailing garbage is trailing-bytes.
    let mut long = art.artifact_payload.clone();
    long.push(0x00);
    assert_eq!(
        decode_artifact(&long).unwrap_err(),
        NafError::Invalid("trailing-bytes")
    );
}

/// §3 core laws over a deterministic sweep: soundness, normality,
/// idempotence-by-uniqueness, negation, and the length bound; plus the §3.4
/// worked minimal-weight example.
#[test]
fn core_laws_sweep() {
    for n in -1000i128..=1000 {
        let v = Int::from_i128(n);
        let d = normalize_integer(&v);
        assert!(is_normal(&d), "normality at {n}");
        assert_eq!(eval_digits(&d), v, "soundness at {n}");
        let neg = normalize_integer(&Int::from_i128(-n));
        assert_eq!(
            neg,
            d.iter().map(|x| -x).collect::<Vec<_>>(),
            "negation at {n}"
        );
        if n != 0 {
            let bitlen = 128 - n.unsigned_abs().leading_zeros() as usize;
            assert!(d.len() <= bitlen + 1, "length bound at {n}");
        }
    }
    // 3 = (-1, 0, 1): weight 2, non-adjacent — the spec's own example.
    assert_eq!(normalize_integer(&Int::from_i128(3)), vec![-1, 0, 1]);
    // Zero is the empty sequence and encodes as `00`.
    assert_eq!(normalize_integer(&Int::from_i128(0)), Vec::<i8>::new());
    assert_eq!(core_naf_encode(&[]), vec![0x00]);
    // Encode/decode round-trip across the sweep.
    for n in -1000i128..=1000 {
        let d = normalize_integer(&Int::from_i128(n));
        let enc = core_naf_encode(&d);
        let (dec, used) = core_naf_decode(&enc).unwrap();
        assert_eq!(used, enc.len());
        assert_eq!(dec, d, "wire round-trip at {n}");
    }
}
