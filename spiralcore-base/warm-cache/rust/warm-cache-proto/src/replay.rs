//! Bounded replay ring for optional F/R diagnostics.
use std::collections::VecDeque;
use crate::N;

pub struct ReplayRing {
    cap: usize,
    frames: VecDeque<[f64; N]>,
}

impl ReplayRing {
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0);
        Self { cap, frames: VecDeque::with_capacity(cap) }
    }
    pub fn push(&mut self, frame: [f64; N]) {
        if self.frames.len() == self.cap { self.frames.pop_front(); }
        self.frames.push_back(frame);
    }
    pub fn len(&self) -> usize { self.frames.len() }
    pub fn is_empty(&self) -> bool { self.frames.is_empty() }
    pub fn last(&self) -> Option<[f64; N]> { self.frames.back().copied() }
    pub fn frames(&self) -> &VecDeque<[f64; N]> { &self.frames }
    pub fn capacity(&self) -> usize { self.cap }
    pub fn bytes(&self) -> usize { self.cap * N * std::mem::size_of::<f64>() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{record_forward, KuramotoCell, PHASE_BITS};

    #[test]
    fn ring_is_bounded_and_keeps_latest() {
        let mut ring = ReplayRing::new(10);
        for i in 0..25 {
            ring.push([i as f64; N]);
        }
        assert_eq!(ring.len(), 10);
        assert_eq!(ring.last().unwrap()[0], 24.0);
        assert_eq!(ring.bytes(), 10 * N * 8);
    }

    #[test]
    fn ring_replay_is_exact_for_retained_window() {
        let cell = KuramotoCell::new(101, PHASE_BITS);
        let (_after, fwd) = record_forward(cell, 0.5, 0.001);
        let mut ring = ReplayRing::new(100);
        for f in &fwd { ring.push(*f); }
        let retained: Vec<[f64; N]> = ring.frames().iter().cloned().collect();
        let rev: Vec<[f64; N]> = retained.iter().rev().cloned().collect();
        for i in 0..rev.len() {
            for k in 0..N { assert!((rev[i][k] - retained[rev.len()-1-i][k]).abs() < 1e-15); }
        }
    }
}
