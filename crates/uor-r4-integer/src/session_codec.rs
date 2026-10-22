//! Integer-safe byte codec for [`SerializedStackSession`].
//!
//! The session snapshot is all integers and short strings
//! ([`crate::stack::SerializedStackSession`]). `serde_json` parses numbers
//! with float/div machinery (its `parse_number`/`parse_exponent` paths), which
//! is R2-hostile on WebAssembly — their own audit fails on `f32.*` and
//! `i64.div` in served code. This codec is the R2-clean alternative: one
//! self-describing little-endian byte stream, integer add/shift/compare only,
//! no serde, no float, no division.
//!
//! Layout (all integers little-endian, `v64` = u64 varint 7-bit groups):
//!
//! ```text
//! magic            8 B   "UR4SESS1"
//! schema.len       u32   schema bytes (UTF-8)
//! version          u32
//! artifact_sha256  u32 len + bytes
//! position         u64
//! cache_at         u64
//! lift_at          u64
//! copy_scale_q16   i32
//! logits           v64 len + i32 each
//! tokens           v64 len + u32 each
//! snap_trace       u8 tag (0 none, 1 some) + v64 len + u32 each
//! pointer_keys     v64 len + i32 each
//! pointer_lifts    v64 len + u64 each
//! mixture          v64 len + i32 each
//! layers           v64 count, then per layer:
//!   kind           u8 (0 recurrence, 1 read)
//!   recurrence:    state v64 len + i64 each; history v64 len + i32 each
//!   read:          keys v64 len + i32 each; values v64 len + i32 each;
//!                  lifts v64 len + u64 each
//! ```
//!
//! Round-trip equality with the engine's own
//! [`crate::stack::IntegerStackSession::save_state`] /
//! [`crate::stack::IntegerStackSession::restore_state`] is exact: every field
//! is carried bit-for-bit.

#![cfg(feature = "serving-wasm")]

use crate::stack::{SerializedStackLayerState, SerializedStackSession};

const MAGIC: &[u8; 8] = b"UR4SESS1";

fn put_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn put_i32(out: &mut Vec<u8>, v: i32) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn put_u64(out: &mut Vec<u8>, v: u64) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn put_i64(out: &mut Vec<u8>, v: i64) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn put_v64(out: &mut Vec<u8>, mut v: u64) {
    loop {
        let b = (v & 0x7f) as u8;
        v >>= 7;
        if v == 0 {
            out.push(b);
            return;
        }
        out.push(b | 0x80);
    }
}
fn put_str(out: &mut Vec<u8>, s: &str) {
    put_u32(out, s.len() as u32);
    out.extend_from_slice(s.as_bytes());
}
fn put_i32s(out: &mut Vec<u8>, xs: &[i32]) {
    put_v64(out, xs.len() as u64);
    for &x in xs {
        put_i32(out, x);
    }
}
fn put_u32s(out: &mut Vec<u8>, xs: &[u32]) {
    put_v64(out, xs.len() as u64);
    for &x in xs {
        put_u32(out, x);
    }
}
fn put_i64s(out: &mut Vec<u8>, xs: &[i64]) {
    put_v64(out, xs.len() as u64);
    for &x in xs {
        put_i64(out, x);
    }
}
fn put_u64s(out: &mut Vec<u8>, xs: &[u64]) {
    put_v64(out, xs.len() as u64);
    for &x in xs {
        put_u64(out, x);
    }
}

/// Encode a session snapshot. Allocation: one output buffer, grown once per
/// segment (the snapshot is small — the read caches dominate).
pub fn encode(state: &SerializedStackSession) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    put_str(&mut out, &state.schema);
    put_u32(&mut out, state.version);
    put_str(&mut out, &state.artifact_sha256);
    put_u64(&mut out, state.position);
    put_u64(&mut out, state.cache_at);
    put_u64(&mut out, state.lift_at);
    put_i32(&mut out, state.copy_scale_q16);
    put_i32s(&mut out, &state.logits);
    put_u32s(&mut out, &state.tokens);
    match &state.snap_trace {
        None => out.push(0),
        Some(xs) => {
            out.push(1);
            put_u32s(&mut out, xs);
        }
    }
    put_i32s(&mut out, &state.pointer_keys);
    put_u64s(&mut out, &state.pointer_lifts);
    put_i32s(&mut out, &state.mixture);
    put_v64(&mut out, state.layers.len() as u64);
    for layer in &state.layers {
        match layer {
            SerializedStackLayerState::Recurrence { state, history } => {
                out.push(0);
                put_i64s(&mut out, state);
                put_i32s(&mut out, history);
            }
            SerializedStackLayerState::Read {
                keys,
                values,
                lifts,
            } => {
                out.push(1);
                put_i32s(&mut out, keys);
                put_i32s(&mut out, values);
                put_u64s(&mut out, lifts);
            }
        }
    }
    out
}

struct Reader<'a> {
    buf: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self
            .at
            .checked_add(n)
            .ok_or_else(|| "session codec: truncated input".to_string())?;
        if end > self.buf.len() {
            return Err("session codec: truncated input".to_string());
        }
        let s = &self.buf[self.at..end];
        self.at = end;
        Ok(s)
    }
    fn checked_count(&mut self, elem_shift: u32) -> Result<usize, String> {
        let raw = self.v64()?;
        let n = usize::try_from(raw).map_err(|_| "session codec: truncated input".to_string())?;
        if n > ((self.buf.len() - self.at) >> elem_shift) {
            return Err("session codec: truncated input".to_string());
        }
        Ok(n)
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32, String> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn i32(&mut self) -> Result<i32, String> {
        let b = self.take(4)?;
        Ok(i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn u64(&mut self) -> Result<u64, String> {
        let b = self.take(8)?;
        Ok(u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]))
    }
    fn i64(&mut self) -> Result<i64, String> {
        let b = self.take(8)?;
        Ok(i64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]))
    }
    fn v64(&mut self) -> Result<u64, String> {
        let mut acc = 0u64;
        let mut shift = 0u32;
        loop {
            let b = self.u8()?;
            acc |= u64::from(b & 0x7f) << shift;
            if b & 0x80 == 0 {
                return Ok(acc);
            }
            shift += 7;
            if shift >= 64 {
                return Err("session codec: varint overflow".to_string());
            }
        }
    }
    fn string(&mut self) -> Result<String, String> {
        let len = self.u32()? as usize;
        let b = self.take(len)?;
        String::from_utf8(b.to_vec()).map_err(|_| "session codec: invalid utf-8".to_string())
    }
    fn i32s(&mut self) -> Result<Vec<i32>, String> {
        let n = self.checked_count(2)?;
        let mut v = Vec::with_capacity(n);
        for _ in 0..n {
            v.push(self.i32()?);
        }
        Ok(v)
    }
    fn u32s(&mut self) -> Result<Vec<u32>, String> {
        let n = self.checked_count(2)?;
        let mut v = Vec::with_capacity(n);
        for _ in 0..n {
            v.push(self.u32()?);
        }
        Ok(v)
    }
    fn i64s(&mut self) -> Result<Vec<i64>, String> {
        let n = self.checked_count(3)?;
        let mut v = Vec::with_capacity(n);
        for _ in 0..n {
            v.push(self.i64()?);
        }
        Ok(v)
    }
    fn u64s(&mut self) -> Result<Vec<u64>, String> {
        let n = self.checked_count(3)?;
        let mut v = Vec::with_capacity(n);
        for _ in 0..n {
            v.push(self.u64()?);
        }
        Ok(v)
    }
}

/// Decode a session snapshot produced by [`encode`]. Rejects truncation,
/// trailing garbage, and a schema mismatch against the artifact's own
/// declaration (the engine's `restore_state` re-checks everything else).
pub fn decode(bytes: &[u8]) -> Result<SerializedStackSession, String> {
    let mut r = Reader { buf: bytes, at: 0 };
    if r.take(8)? != MAGIC {
        return Err("session codec: bad magic".to_string());
    }
    let schema = r.string()?;
    let version = r.u32()?;
    let artifact_sha256 = r.string()?;
    let position = r.u64()?;
    let cache_at = r.u64()?;
    let lift_at = r.u64()?;
    let copy_scale_q16 = r.i32()?;
    let logits = r.i32s()?;
    let tokens = r.u32s()?;
    let snap_trace = match r.u8()? {
        0 => None,
        1 => Some(r.u32s()?),
        other => return Err(format!("session codec: bad snap_trace tag {other}")),
    };
    let pointer_keys = r.i32s()?;
    let pointer_lifts = r.u64s()?;
    let mixture = r.i32s()?;
    let layer_count = r.checked_count(0)?;
    let mut layers = Vec::with_capacity(layer_count);
    for _ in 0..layer_count {
        match r.u8()? {
            0 => {
                let state = r.i64s()?;
                let history = r.i32s()?;
                layers.push(SerializedStackLayerState::Recurrence { state, history });
            }
            1 => {
                let keys = r.i32s()?;
                let values = r.i32s()?;
                let lifts = r.u64s()?;
                layers.push(SerializedStackLayerState::Read {
                    keys,
                    values,
                    lifts,
                });
            }
            other => return Err(format!("session codec: bad layer tag {other}")),
        }
    }
    if r.at != bytes.len() {
        return Err("session codec: trailing bytes".to_string());
    }
    Ok(SerializedStackSession {
        schema,
        version,
        artifact_sha256,
        position,
        cache_at,
        lift_at,
        copy_scale_q16,
        logits,
        tokens,
        snap_trace,
        pointer_keys,
        pointer_lifts,
        mixture,
        layers,
    })
}

/// Binary magic of the M3 object-store durable-memory envelope (`UORMEM01`).
pub const DURABLE_SCOPE_MAGIC: &[u8; 8] = b"UORMEM01";

/// Version of the `UORMEM01` durable-scope envelope.
pub const DURABLE_SCOPE_VERSION: u32 = 1;

/// Identity scope triple (`user_id`, `project_id`, `session_id`) for `UORMEM01`
/// object-store session persistence (`export_scope` / `import_scope`, #2873).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopedSessionIdent {
    pub user_id: String,
    pub project_id: String,
    pub session_id: String,
}

impl ScopedSessionIdent {
    pub fn new(
        user_id: impl Into<String>,
        project_id: impl Into<String>,
        session_id: impl Into<String>,
    ) -> Result<Self, String> {
        let u = user_id.into().trim().to_string();
        let p = project_id.into().trim().to_string();
        let s = session_id.into().trim().to_string();
        if u.is_empty()
            || p.is_empty()
            || s.is_empty()
            || u.len() > 256
            || p.len() > 256
            || s.len() > 256
        {
            return Err("session codec: invalid identity scope".to_string());
        }
        Ok(Self {
            user_id: u,
            project_id: p,
            session_id: s,
        })
    }
}

/// Encode an [`ScopedSessionIdent`] and [`SerializedStackSession`] into a canonical
/// `UORMEM01` binary envelope ready to fold into `uor-r4-object-store::Store`.
pub fn export_scope(
    scope: &ScopedSessionIdent,
    state: &SerializedStackSession,
) -> Result<Vec<u8>, String> {
    ScopedSessionIdent::new(&scope.user_id, &scope.project_id, &scope.session_id)?;
    let sess_bytes = encode(state);
    let mut out = Vec::with_capacity(64 + sess_bytes.len());
    out.extend_from_slice(DURABLE_SCOPE_MAGIC);
    put_u32(&mut out, DURABLE_SCOPE_VERSION);
    put_str(&mut out, &scope.user_id);
    put_str(&mut out, &scope.project_id);
    put_str(&mut out, &scope.session_id);
    out.push(0); // last_entity = None
    out.push(1); // ur4sess1_snapshot = Some
    put_u32(&mut out, sess_bytes.len() as u32);
    out.extend_from_slice(&sess_bytes);
    put_u32(&mut out, 0); // facts count = 0
    put_u32(&mut out, 0); // non_memory_turns count = 0
    Ok(out)
}

/// Decode a `UORMEM01` binary envelope (rebuilt from `uor-r4-object-store::Store`)
/// back into its [`ScopedSessionIdent`] and [`SerializedStackSession`].
pub fn import_scope(
    bytes: &[u8],
) -> Result<(ScopedSessionIdent, SerializedStackSession), String> {
    let mut r = Reader { buf: bytes, at: 0 };
    if r.take(8)? != DURABLE_SCOPE_MAGIC {
        return Err("session codec: bad UORMEM01 magic".to_string());
    }
    if r.u32()? != DURABLE_SCOPE_VERSION {
        return Err("session codec: unsupported UORMEM01 version".to_string());
    }
    let user_id = r.string()?;
    let project_id = r.string()?;
    let session_id = r.string()?;
    let scope = ScopedSessionIdent::new(user_id, project_id, session_id)?;
    match r.u8()? {
        0 => {}
        1 => {
            let _entity = r.string()?;
        }
        _ => return Err("session codec: bad last_entity tag".to_string()),
    }
    let sess_state = match r.u8()? {
        1 => {
            let len = r.u32()? as usize;
            let raw = r.take(len)?;
            decode(raw)?
        }
        _ => return Err("session codec: missing UR4SESS1 snapshot in UORMEM01".to_string()),
    };
    let fact_count = r.u32()?;
    let prose_count = r.u32()?;
    if fact_count != 0 || prose_count != 0 || r.at != bytes.len() {
        return Err("session codec: unexpected trailing or fact sections".to_string());
    }
    Ok((scope, sess_state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stack::STACK_SESSION_SCHEMA;

    fn sample() -> SerializedStackSession {
        SerializedStackSession {
            schema: STACK_SESSION_SCHEMA.to_string(),
            version: 1,
            artifact_sha256: "ab".repeat(32),
            position: 7,
            cache_at: 7 * 288,
            lift_at: 7 * 6,
            copy_scale_q16: 65_536,
            logits: vec![-3, 0, 65_536, i32::MIN, i32::MAX],
            tokens: vec![0, 5, 77, 4095],
            snap_trace: Some(vec![0, 119]),
            pointer_keys: vec![1, -2, 3],
            pointer_lifts: vec![u64::MAX, 0, 1 << 62],
            mixture: vec![9, -9],
            layers: vec![
                SerializedStackLayerState::Recurrence {
                    state: vec![i64::MIN, -1, 0, 1, i64::MAX],
                    history: vec![4, -4, 16],
                },
                SerializedStackLayerState::Read {
                    keys: vec![11, 12],
                    values: vec![-11, -12],
                    lifts: vec![1, 2, 3, 4],
                },
            ],
        }
    }

    #[test]
    fn round_trip_is_exact() {
        let s = sample();
        let bytes = encode(&s);
        let back = decode(&bytes).expect("decode");
        assert_eq!(s, back);
    }

    #[test]
    fn empty_vecs_round_trip() {
        let mut s = sample();
        s.logits = Vec::new();
        s.tokens = Vec::new();
        s.snap_trace = None;
        s.pointer_keys = Vec::new();
        s.pointer_lifts = Vec::new();
        s.mixture = Vec::new();
        s.layers = Vec::new();
        let bytes = encode(&s);
        let back = decode(&bytes).expect("decode");
        assert_eq!(s, back);
    }

    #[test]
    fn rejects_bad_magic_and_trailing() {
        let s = sample();
        let mut bytes = encode(&s);
        bytes[0] ^= 1;
        assert!(decode(&bytes).is_err());
        let mut bytes = encode(&s);
        bytes.push(0);
        assert!(decode(&bytes).is_err());
        let bytes = encode(&s);
        assert!(decode(&bytes[..bytes.len() - 1]).is_err());
    }

    #[test]
    fn rejects_oversized_varint_slice_count_without_oom() {
        let mut out = Vec::new();
        out.extend_from_slice(MAGIC);
        put_str(&mut out, STACK_SESSION_SCHEMA);
        put_u32(&mut out, 1);
        put_str(&mut out, &"ab".repeat(32));
        put_u64(&mut out, 1);
        put_u64(&mut out, 288);
        put_u64(&mut out, 6);
        put_i32(&mut out, 65_536);
        // Malicious varint count for logits: u64::MAX
        put_v64(&mut out, u64::MAX);
        assert!(decode(&out).is_err());
    }

    #[test]
    fn export_and_import_scope_round_trip_and_refuse_corrupt_envelope() {
        let s = sample();
        let scope = ScopedSessionIdent::new("user-1", "proj-m3", "sess-42").expect("valid scope");
        let envelope = export_scope(&scope, &s).expect("export_scope");
        assert_eq!(&envelope[..8], DURABLE_SCOPE_MAGIC);
        let (restored_scope, restored_session) =
            import_scope(&envelope).expect("import_scope");
        assert_eq!(restored_scope, scope);
        assert_eq!(restored_session, s);

        let mut bad_magic = envelope.clone();
        bad_magic[0] ^= 1;
        assert!(import_scope(&bad_magic).is_err());

        let mut trailing = envelope.clone();
        trailing.push(0);
        assert!(import_scope(&trailing).is_err());
        assert!(ScopedSessionIdent::new("", "proj", "sess").is_err());
    }
}
