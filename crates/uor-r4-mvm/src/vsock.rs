//! SpiralCore Zero-NIC virtio-vsock framing and relay wire format.

pub use crate::mvm::{
    SpiralCoreHeader, VsockPacket, SPIRALCORE_GUEST_CID, SPIRALCORE_HOST_CID, SPIRALCORE_MAGIC,
    SPIRALCORE_MAX_PAYLOAD, SPIRALCORE_VSOCK_PORT,
};

/// Encode a SpiralCore frame into its 16-byte header followed by payload bytes.
pub fn encode_vsock_frame(
    src_cid: u16,
    dst_cid: u16,
    vsock_port: u16,
    cl06_spin: u8,
    e8_orbit: u8,
    payload: &[u8],
) -> Vec<u8> {
    let mut out = Vec::with_capacity(16 + payload.len());
    let hdr = SpiralCoreHeader {
        magic: SPIRALCORE_MAGIC,
        src_cid,
        dst_cid,
        vsock_port,
        cl06_spin,
        e8_orbit,
        payload_len: payload.len() as u32,
    };
    out.extend_from_slice(&hdr.to_bytes());
    out.extend_from_slice(payload);
    out
}

/// Decode a wire-encoded SpiralCore frame into header and payload.
pub fn decode_vsock_frame(wire: &[u8]) -> Option<(SpiralCoreHeader, &[u8])> {
    let hdr = SpiralCoreHeader::from_bytes(wire)?;
    let end = 16usize.saturating_add(hdr.payload_len as usize);
    if wire.len() != end {
        return None;
    }
    Some((hdr, &wire[16..end]))
}
