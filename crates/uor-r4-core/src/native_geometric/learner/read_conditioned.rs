//! One read-conditioned geometric state update feeding a shared emission readout.
//!
//! The frozen scored/confidence reader changes only the selected payload logit, so it cannot favour a
//! token that is absent from every admitted payload. This module adds the smallest operation that
//! can: a state update
//!
//! ```text
//! q0 = learned query root of the causal prefix
//! r  = directed relation of query and selected source
//! v  = learned value code of the selected actual payload
//! q1 = (q0 * T[r]) * V[v]          exact signed-H4 products
//! z1 = z_local + u(q1) - u(q0)      shared low-bit residual readout
//! ```
//!
//! `NoRead`/`UpdateDisabled` set `q1 = q0`, so the residual is exactly zero and the local baseline is
//! preserved bit for bit. Only `T` (one element per relation) and `V` (one element per observed
//! payload token) are learned; the readout `u` is the frozen shared reader row table.
#![forbid(unsafe_code)]

use super::group_table::{group_table, GROUP_ORDER, ROW_STRIDE};

/// Parameters of the read-conditioned H4 update. Different initial value-code maps remain the
/// same geometric operator; they are not an ordinary categorical-state comparator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadConditionedParams {
    /// `transport[r]` is the group element applied for directed relation `r`.
    pub transport: Vec<u8>,
    /// Payload tokens the value code was learned for; every other token uses the identity element.
    pub value_domain: Vec<u32>,
    /// `value_code[i]` is the group element for `value_domain[i]`.
    pub value_code: Vec<u8>,
}

impl ReadConditionedParams {
    /// Identity transport and identity value code: `q1 = q0` for every read.
    pub fn identity() -> Self {
        ReadConditionedParams {
            transport: vec![group_table().identity; GROUP_ORDER],
            value_domain: Vec::new(),
            value_code: Vec::new(),
        }
    }

    #[inline]
    pub fn value_state(&self, payload: u32) -> usize {
        match self.value_domain.iter().position(|t| *t == payload) {
            Some(i) => self.value_code[i] as usize,
            None => group_table().identity as usize,
        }
    }

    /// The read-conditioned state `q1 = (q0 * T[r]) * V[payload]`. Callers implement NoRead and
    /// UpdateDisabled by not calling this method. Relations are reduced modulo the group order.
    #[inline]
    pub fn update(&self, q0: usize, r: usize, payload: u32) -> usize {
        let t = group_table();
        let q0 = q0.min(GROUP_ORDER - 1);
        let a = self
            .transport
            .get(r % GROUP_ORDER)
            .copied()
            .unwrap_or(t.identity) as usize;
        let v = self.value_state(payload);
        let mid = t.product[q0 * ROW_STRIDE + a.min(GROUP_ORDER - 1)] as usize;
        t.product[mid * ROW_STRIDE + v.min(GROUP_ORDER - 1)] as usize
    }

    /// The **matched cyclic C120 update**: `q1 = (q0 + T[r] + V[payload]) mod 120`. Same state
    /// count, maps, information and readout as the H4 composition; abelian, so it is not an H4
    /// relabeling. This is the declared non-geometric update-algebra comparator.
    #[inline]
    pub fn update_cyclic(&self, q0: usize, r: usize, payload: u32) -> usize {
        let a = self.transport.get(r % GROUP_ORDER).copied().unwrap_or(0) as usize;
        let v = self.value_state(payload);
        (q0 + a + v) % GROUP_ORDER
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(b"RLRC");
        o.extend_from_slice(&1u32.to_le_bytes());
        o.extend_from_slice(&(self.transport.len() as u32).to_le_bytes());
        o.extend_from_slice(&self.transport);
        o.extend_from_slice(&(self.value_domain.len() as u32).to_le_bytes());
        for t in self.value_domain.iter() {
            o.extend_from_slice(&t.to_le_bytes());
        }
        o.extend_from_slice(&(self.value_code.len() as u32).to_le_bytes());
        o.extend_from_slice(&self.value_code);
        o
    }

    /// Independent load. Rejects a wrong magic/version, an out-of-range group element and a domain
    /// whose code length disagrees.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let mut c = 0usize;
        let take = |c: &mut usize, n: usize| -> Result<&[u8], String> {
            let end = c.checked_add(n).ok_or("size overflow")?;
            if end > bytes.len() {
                return Err("truncated read-conditioned artifact".into());
            }
            let s = &bytes[*c..end];
            *c = end;
            Ok(s)
        };
        if take(&mut c, 4)? != b"RLRC" {
            return Err("bad read-conditioned artifact magic".into());
        }
        let ver = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
        if ver != 1 {
            return Err("unsupported read-conditioned artifact version".into());
        }
        let n_t = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
        if n_t != GROUP_ORDER {
            return Err("transport map has the wrong relation count".into());
        }
        let transport = take(&mut c, n_t)?.to_vec();
        let n_d = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
        // Validate the advertised allocation against the remaining serialized bytes first. Each
        // domain entry needs four bytes and a value code, plus the following code-length field.
        let minimum = n_d
            .checked_mul(5)
            .and_then(|n| n.checked_add(4))
            .ok_or("value domain size overflow")?;
        if minimum > bytes.len().saturating_sub(c) {
            return Err("truncated value domain or value codes".into());
        }
        let mut value_domain = Vec::with_capacity(n_d);
        for _ in 0..n_d {
            value_domain.push(u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()));
        }
        if value_domain.windows(2).any(|w| w[0] >= w[1]) {
            return Err("value domain must be strictly increasing and unique".into());
        }
        let n_c = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
        if n_c != n_d {
            return Err("value code length disagrees with its domain".into());
        }
        let value_code = take(&mut c, n_c)?.to_vec();
        if c != bytes.len() {
            return Err("trailing bytes in the read-conditioned artifact".into());
        }
        if transport
            .iter()
            .chain(value_code.iter())
            .any(|v| *v as usize >= GROUP_ORDER)
        {
            return Err("group element outside the declared domain".into());
        }
        Ok(ReadConditionedParams {
            transport,
            value_domain,
            value_code,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_maps_leave_the_state_unchanged_and_round_trip() {
        let p = ReadConditionedParams::identity();
        for q0 in [0usize, 7, 63, 119] {
            assert_eq!(
                p.update(q0, 3, 42),
                q0,
                "identity transport/value is a no-op"
            );
        }
        let back = ReadConditionedParams::from_bytes(&p.to_bytes()).unwrap();
        assert_eq!(back, p);
        // A learned map changes the state and survives the round trip.
        let mut q = p.clone();
        q.transport[5] = 13;
        q.value_domain = vec![100, 101];
        q.value_code = vec![7, 9];
        assert_ne!(q.update(10, 5, 100), 10);
        // An out-of-domain payload applies the identity value code, so only the transport acts.
        let t = group_table();
        let mid = t.product[10 * ROW_STRIDE + 13] as usize;
        let expected = t.product[mid * ROW_STRIDE + t.identity as usize] as usize;
        assert_eq!(q.update(10, 5, 999), expected);
        let back = ReadConditionedParams::from_bytes(&q.to_bytes()).unwrap();
        assert_eq!(back, q);
        // Malformed payloads are rejected rather than silently loaded.
        let mut bad = q.to_bytes();
        bad[4] = 9;
        assert!(ReadConditionedParams::from_bytes(&bad).is_err());
        let mut bad = q.to_bytes();
        bad.truncate(bad.len() - 1);
        assert!(ReadConditionedParams::from_bytes(&bad).is_err());
    }

    #[test]
    fn loader_rejects_unbounded_or_ambiguous_domains() {
        let mut bytes = ReadConditionedParams::identity().to_bytes();
        let domain_count_offset = 12 + GROUP_ORDER;
        bytes[domain_count_offset..domain_count_offset + 4]
            .copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(ReadConditionedParams::from_bytes(&bytes).is_err());

        let mut p = ReadConditionedParams::identity();
        p.value_domain = vec![7, 7];
        p.value_code = vec![0, 1];
        assert!(ReadConditionedParams::from_bytes(&p.to_bytes()).is_err());
        p.value_domain = vec![8, 7];
        assert!(ReadConditionedParams::from_bytes(&p.to_bytes()).is_err());
        p.value_domain = vec![7, 8];
        assert_eq!(ReadConditionedParams::from_bytes(&p.to_bytes()).unwrap(), p);
    }
}
