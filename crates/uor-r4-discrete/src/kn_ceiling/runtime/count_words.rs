/// Bytes per canonical triple: key u128 LE, next u16 LE, count u32 LE.
pub const TRIPLE_BYTES: usize = 22;
/// Bytes per canonical context record: key u128, total u64, N1+, N1, N2, N3+ (u32 each).
pub const CONTEXT_BYTES: usize = 40;
pub const TRIPLE_DOMAIN: &[u8] = b"uor-r4/ngram-triples/v1\0";
pub const CONTEXT_DOMAIN: &[u8] = b"uor-r4/ngram-contexts/v1\0";
pub const ADJUSTED_TRIPLE_DOMAIN: &[u8] = b"uor-r4/ngram-adjusted-triples/v1\0";
pub const ADJUSTED_CONTEXT_DOMAIN: &[u8] = b"uor-r4/ngram-adjusted-contexts/v1\0";

/// One canonical (context key, next token, count) triple.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NgramTriple {
    pub key: u128,
    pub next: u16,
    pub count: u32,
}

/// Canonical 22-byte encoding of a triple.
pub fn triple_bytes(triple: &NgramTriple) -> [u8; TRIPLE_BYTES] {
    let mut bytes = [0u8; TRIPLE_BYTES];
    bytes[..16].copy_from_slice(&triple.key.to_le_bytes());
    bytes[16..18].copy_from_slice(&triple.next.to_le_bytes());
    bytes[18..].copy_from_slice(&triple.count.to_le_bytes());
    bytes
}

/// Canonical 40-byte encoding of a context record. N3+ is implied
/// (followers - n1 - n2) but carried so a reader needs no rule.
pub fn context_bytes(context: &NgramContext) -> [u8; CONTEXT_BYTES] {
    let mut bytes = [0u8; CONTEXT_BYTES];
    bytes[..16].copy_from_slice(&context.key.to_le_bytes());
    bytes[16..24].copy_from_slice(&context.total.to_le_bytes());
    bytes[24..28].copy_from_slice(&context.followers.to_le_bytes());
    bytes[28..32].copy_from_slice(&context.n1.to_le_bytes());
    bytes[32..36].copy_from_slice(&context.n2.to_le_bytes());
    bytes[36..40].copy_from_slice(&context.n3_plus.to_le_bytes());
    bytes
}
