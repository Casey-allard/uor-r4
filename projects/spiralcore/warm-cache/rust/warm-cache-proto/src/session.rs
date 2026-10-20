//! Library API: warm session over a seed cache and a time header.
use crate::wire::WarmWire;
use crate::{make_warm, receive_warm, Plan, Seed, TimeHeader, WarmResult};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WarmStats {
    pub frames_sent: u64,
    pub frames_received: u64,
    pub warm_bytes: u64,
    pub payload_bytes: u64,
    pub repairs: u64,
    pub committed: u64,
    pub clock_mismatch: u64,
    pub content_mismatch: u64,
    pub seed_mismatch: u64,
    pub witness_mismatch: u64,
}

#[derive(Clone, Debug)]
pub struct WarmSession {
    pub seed: Seed,
    pub header: TimeHeader,
    pub stats: WarmStats,
}

impl WarmSession {
    pub fn new(seed: Seed, header: TimeHeader) -> Self {
        Self { seed, header, stats: WarmStats::default() }
    }

    pub fn seed_id(&self) -> [u8; 32] { self.seed.id }

    pub fn set_header(&mut self, header: TimeHeader) { self.header = header; }

    pub fn encode(&mut self, plan: Plan) -> WarmWire {
        let frame = make_warm(&self.seed, plan, self.header.clone());
        let wire = WarmWire::from_frame(&frame);
        self.stats.frames_sent += 1;
        self.stats.warm_bytes += wire.wire_len();
        self.stats.payload_bytes += wire.payload.len() as u64;
        if wire.plan_kind == 2 { self.stats.repairs += 1; }
        wire
    }

    pub fn decode(&mut self, bytes: &[u8]) -> Result<WarmResult, String> {
        let wire = WarmWire::decode(bytes)?;
        let frame = wire.to_frame()?;
        self.stats.frames_received += 1;
        self.stats.warm_bytes += bytes.len() as u64;
        self.stats.payload_bytes += wire.payload.len() as u64;
        let res = receive_warm(&self.seed, &self.header, &frame);
        match res {
            WarmResult::Committed => self.stats.committed += 1,
            WarmResult::ClockMismatch => self.stats.clock_mismatch += 1,
            WarmResult::ContentMismatch => self.stats.content_mismatch += 1,
            WarmResult::SeedMismatch => self.stats.seed_mismatch += 1,
            WarmResult::WitnessMismatch => self.stats.witness_mismatch += 1,
        }
        Ok(res)
    }

    pub fn encode_bytes(&mut self, plan: Plan) -> Vec<u8> { self.encode(plan).encode() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{KuramotoCell, PAGE, PHASE_BITS};

    fn header() -> TimeHeader {
        let mut cell = KuramotoCell::new(83, PHASE_BITS);
        for _ in 0..2000 { cell.step(0.001); }
        TimeHeader::from_snapshot(0, 0, 0, &cell.snapshot())
    }

    #[test]
    fn session_sends_and_receives_warm() {
        let seed = Seed::new(vec![6u8; 1 << 16], 1);
        let mut sender = WarmSession::new(seed.clone(), header());
        let mut receiver = WarmSession::new(seed, header());
        let bytes = sender.encode_bytes(Plan::Identity);
        assert_eq!(receiver.decode(&bytes).unwrap(), WarmResult::Committed);
        assert_eq!(sender.stats.frames_sent, 1);
        assert_eq!(receiver.stats.committed, 1);
        assert_eq!(sender.stats.payload_bytes, 0);
    }

    #[test]
    fn session_page_repair() {
        let data: Vec<u8> = (0..(1 << 16)).map(|i| (i * 7) as u8).collect();
        let seed = Seed::new(data.clone(), 1);
        let mut page = data[0..PAGE].to_vec(); page[0] ^= 0xAA;
        let mut sender = WarmSession::new(seed.clone(), header());
        let mut receiver = WarmSession::new(seed, header());
        let bytes = sender.encode_bytes(Plan::PageRepair(vec![(0, page)]));
        assert_eq!(receiver.decode(&bytes).unwrap(), WarmResult::Committed);
        assert_eq!(sender.stats.repairs, 1);
        assert!(sender.stats.warm_bytes > 0);
    }
}
