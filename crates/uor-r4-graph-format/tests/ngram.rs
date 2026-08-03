use uor_r4_graph_format::{
    NgramTable, ScoreQ, NGRAM_ENTRY_LEN, NGRAM_HEADER_LEN, NGRAM_MAGIC, NGRAM_ROW_LEN,
    NGRAM_VERSION,
};

fn fixture() -> Vec<u8> {
    let rows = 2usize;
    let entries_start = NGRAM_HEADER_LEN + rows * NGRAM_ROW_LEN;
    let mut bytes = vec![0u8; entries_start];
    bytes[..4].copy_from_slice(&NGRAM_MAGIC);
    bytes[4..6].copy_from_slice(&NGRAM_VERSION.to_le_bytes());
    bytes[8..12].copy_from_slice(&(rows as u32).to_le_bytes());
    bytes[12..14].copy_from_slice(&2u16.to_le_bytes());
    bytes[NGRAM_HEADER_LEN] = 1;
    bytes[NGRAM_HEADER_LEN + 2..NGRAM_HEADER_LEN + 4].copy_from_slice(&1u16.to_le_bytes());
    bytes[NGRAM_HEADER_LEN + 4..NGRAM_HEADER_LEN + 8].copy_from_slice(&7u32.to_le_bytes());
    bytes[NGRAM_HEADER_LEN + 12..NGRAM_HEADER_LEN + 16]
        .copy_from_slice(&(entries_start as u32).to_le_bytes());
    bytes[NGRAM_HEADER_LEN + NGRAM_ROW_LEN] = 2;
    bytes[NGRAM_HEADER_LEN + NGRAM_ROW_LEN + 2..NGRAM_HEADER_LEN + NGRAM_ROW_LEN + 4]
        .copy_from_slice(&2u16.to_le_bytes());
    bytes[NGRAM_HEADER_LEN + NGRAM_ROW_LEN + 4..NGRAM_HEADER_LEN + NGRAM_ROW_LEN + 8]
        .copy_from_slice(&7u32.to_le_bytes());
    bytes[NGRAM_HEADER_LEN + NGRAM_ROW_LEN + 8..NGRAM_HEADER_LEN + NGRAM_ROW_LEN + 12]
        .copy_from_slice(&9u32.to_le_bytes());
    bytes[NGRAM_HEADER_LEN + NGRAM_ROW_LEN + 12..NGRAM_HEADER_LEN + NGRAM_ROW_LEN + 16]
        .copy_from_slice(&((entries_start + NGRAM_ENTRY_LEN) as u32).to_le_bytes());
    bytes.extend_from_slice(&11u32.to_le_bytes());
    bytes.extend_from_slice(&ScoreQ::from_raw(20).raw().to_le_bytes());
    bytes.extend_from_slice(&13u32.to_le_bytes());
    bytes.extend_from_slice(&ScoreQ::from_raw(30).raw().to_le_bytes());
    bytes.extend_from_slice(&15u32.to_le_bytes());
    bytes.extend_from_slice(&ScoreQ::from_raw(40).raw().to_le_bytes());
    bytes
}

#[test]
fn finds_rows_by_specific_context_key() {
    let bytes = fixture();
    let table = NgramTable::parse(&bytes).expect("valid NGRAM");
    assert_eq!(table.row_count(), 2);
    let row = table.find(2, 7, 9).expect("trigram row");
    let entries: Vec<_> = row.entries().collect();
    assert_eq!(entries[0].token, 13);
    assert_eq!(entries[1].score_q.raw(), 40);
    assert!(table.find(1, 8, 0).is_none());
}

#[test]
fn rejects_noncanonical_rows() {
    let mut bytes = fixture();
    bytes[NGRAM_HEADER_LEN] = 2;
    bytes[NGRAM_HEADER_LEN + 8..NGRAM_HEADER_LEN + 12].copy_from_slice(&9u32.to_le_bytes());
    assert!(NgramTable::parse(&bytes).is_err());
}

#[test]
fn rejects_malformed_header_and_entry_ranges() {
    let mut bad_magic = fixture();
    bad_magic[..4].copy_from_slice(b"BAD!");
    assert!(NgramTable::parse(&bad_magic).is_err());

    let mut bad_reserved = fixture();
    bad_reserved[6] = 1;
    assert!(NgramTable::parse(&bad_reserved).is_err());

    let mut bad_entry_count = fixture();
    bad_entry_count[12..14].copy_from_slice(&1u16.to_le_bytes());
    assert!(NgramTable::parse(&bad_entry_count).is_err());

    let mut overlapping = fixture();
    let first_offset = (NGRAM_HEADER_LEN + NGRAM_ROW_LEN * 2) as u32;
    overlapping[NGRAM_HEADER_LEN + NGRAM_ROW_LEN + 12..NGRAM_HEADER_LEN + NGRAM_ROW_LEN + 16]
        .copy_from_slice(&first_offset.to_le_bytes());
    assert!(NgramTable::parse(&overlapping).is_err());

    let mut trailing = fixture();
    trailing.extend_from_slice(&[0; NGRAM_ENTRY_LEN]);
    assert!(NgramTable::parse(&trailing).is_err());
}
