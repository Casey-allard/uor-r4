//! Small application-level multiplexer over a byte stream.
use std::collections::VecDeque;

pub const MUX_MAGIC: &[u8; 4] = b"WMX1";
pub const MUX_VERSION: u8 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MuxKind { Control = 0, Warm = 1, Repair = 2, Cold = 3 }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MuxFrame {
    pub stream_id: u32,
    pub kind: MuxKind,
    pub priority: u8,
    pub payload: Vec<u8>,
}

impl MuxFrame {
    pub fn encode(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(MUX_MAGIC);
        v.push(MUX_VERSION);
        v.extend_from_slice(&self.stream_id.to_le_bytes());
        v.push(self.kind as u8);
        v.push(self.priority);
        v.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        v.extend_from_slice(&self.payload);
        v
    }
    pub fn wire_len(&self) -> u64 { self.encode().len() as u64 }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 4 + 1 + 4 + 1 + 1 + 4 { return Err("short".into()); }
        if &bytes[..4] != MUX_MAGIC { return Err("magic".into()); }
        if bytes[4] != MUX_VERSION { return Err("version".into()); }
        let stream_id = u32::from_le_bytes(bytes[5..9].try_into().unwrap());
        let kind = match bytes[9] { 0 => MuxKind::Control, 1 => MuxKind::Warm, 2 => MuxKind::Repair, 3 => MuxKind::Cold, _ => return Err("kind".into()) };
        let priority = bytes[10];
        let plen = u32::from_le_bytes(bytes[11..15].try_into().unwrap()) as usize;
        if bytes.len() != 15 + plen { return Err("payload".into()); }
        Ok(Self { stream_id, kind, priority, payload: bytes[15..].to_vec() })
    }
}

#[derive(Clone, Debug, Default)]
pub struct MuxQueue {
    control: VecDeque<MuxFrame>,
    warm: VecDeque<MuxFrame>,
    repair: VecDeque<MuxFrame>,
    cold: VecDeque<MuxFrame>,
}

impl MuxQueue {
    pub fn new() -> Self { Self::default() }
    pub fn push(&mut self, f: MuxFrame) {
        match f.kind {
            MuxKind::Control => self.control.push_back(f),
            MuxKind::Warm => self.warm.push_back(f),
            MuxKind::Repair => self.repair.push_back(f),
            MuxKind::Cold => self.cold.push_back(f),
        }
    }
    pub fn pop(&mut self) -> Option<MuxFrame> {
        self.control.pop_front()
            .or_else(|| self.warm.pop_front())
            .or_else(|| self.repair.pop_front())
            .or_else(|| self.cold.pop_front())
    }
    pub fn len(&self) -> usize { self.control.len() + self.warm.len() + self.repair.len() + self.cold.len() }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_roundtrip_and_wire_len() {
        let f = MuxFrame { stream_id: 42, kind: MuxKind::Warm, priority: 1, payload: b"hello".to_vec() };
        let enc = f.encode();
        assert_eq!(MuxFrame::decode(&enc).unwrap(), f);
        assert_eq!(f.wire_len(), 20);
    }

    #[test]
    fn control_preempts_bulk_and_fifo_within_class() {
        let mut q = MuxQueue::new();
        q.push(MuxFrame { stream_id: 1, kind: MuxKind::Cold, priority: 3, payload: vec![1] });
        q.push(MuxFrame { stream_id: 2, kind: MuxKind::Warm, priority: 1, payload: vec![2] });
        q.push(MuxFrame { stream_id: 3, kind: MuxKind::Control, priority: 0, payload: vec![3] });
        q.push(MuxFrame { stream_id: 4, kind: MuxKind::Warm, priority: 1, payload: vec![4] });
        let order: Vec<u32> = std::iter::from_fn(|| q.pop()).map(|f| f.stream_id).collect();
        assert_eq!(order, vec![3, 2, 4, 1]);
    }
}
