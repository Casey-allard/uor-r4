//! Lossless training-state checkpoint, distinct from a served TLX model.
use super::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainingCursor {
    pub data_sha256: [u8; 32],
    pub tokenizer_sha256: [u8; 32],
    pub rng_state: u64,
    pub next_batch: u64,
    pub schedule_total: u64,
    pub lr_start_bits: u64,
    pub lr_end_bits: u64,
}
use sha2::{Digest, Sha256};
const MAX_CHECKPOINT: usize = 64 * 1024 * 1024;
#[derive(Serialize, Deserialize)]
struct Header {
    source_sha256: [u8; 32],
    cfg: [u64; 6],
    opt: [u64; 6],
    step: u64,
    cursor: TrainingCursor,
}
impl TlTrainer {
    pub fn checkpoint(&self, cursor: &TrainingCursor) -> Result<Vec<u8>, String> {
        self.cfg.validate()?;
        validate_cursor(cursor)?;
        validate_optimizer(self.tcfg)?;
        let h = Header {
            source_sha256: Sha256::digest(include_bytes!("transferable_lexical.rs")).into(),
            cfg: [
                self.cfg.vocab as u64,
                self.cfg.h_dim as u64,
                self.cfg.h_clamp as u64,
                self.cfg.m_clamp as u64,
                self.cfg.recurrent_shift as u64,
                self.cfg.score_shift as u64,
            ],
            opt: [
                self.tcfg.lr.to_bits(),
                self.tcfg.beta1.to_bits(),
                self.tcfg.beta2.to_bits(),
                self.tcfg.weight_decay.to_bits(),
                self.tcfg.grad_clip.to_bits(),
                self.tcfg.seed,
            ],
            step: self.step,
            cursor: cursor.clone(),
        };
        let meta = serde_json::to_vec(&h).map_err(|e| e.to_string())?;
        if meta.len() > 4096 {
            return Err("checkpoint header too large".into());
        }
        let groups = [
            &self.e, &self.wi, &self.wh, &self.wf, &self.wo, &self.bh, &self.bo, &self.me,
            &self.mi, &self.mh, &self.mf, &self.mo, &self.mbh, &self.mbo, &self.ve, &self.vi,
            &self.vh, &self.vf, &self.vo, &self.vbh, &self.vbo,
        ];
        let size = groups
            .iter()
            .try_fold(meta.len() + 40, |n, g| {
                n.checked_add(g.len().checked_mul(4)?)
            })
            .ok_or("checkpoint size overflow")?;
        if size > MAX_CHECKPOINT {
            return Err("checkpoint exceeds 64 MiB bound".into());
        }
        let mut out = Vec::with_capacity(size);
        out.extend_from_slice(b"TLK1");
        out.extend_from_slice(&(meta.len() as u32).to_le_bytes());
        out.extend(meta);
        for (i, g) in groups.iter().enumerate() {
            for &x in g.iter() {
                if !x.is_finite() || (i >= 14 && x < 0.) {
                    return Err("invalid latent or moment value".into());
                }
                out.extend_from_slice(&x.to_bits().to_le_bytes());
            }
        }
        let digest = Sha256::digest(&out);
        out.extend_from_slice(&digest);
        Ok(out)
    }
    pub fn from_checkpoint(
        bytes: &[u8],
        data: [u8; 32],
        tokenizer: [u8; 32],
    ) -> Result<(Self, TrainingCursor), String> {
        if bytes.len() < 40 || bytes.len() > MAX_CHECKPOINT || &bytes[..4] != b"TLK1" {
            return Err("invalid checkpoint envelope".into());
        }
        let end = bytes.len() - 32;
        if Sha256::digest(&bytes[..end]).as_slice() != &bytes[end..] {
            return Err("checkpoint digest mismatch".into());
        }
        let n = u32::from_le_bytes(bytes[4..8].try_into().map_err(|_| "header size")?) as usize;
        if n > 4096 || 8 + n > end {
            return Err("invalid checkpoint header length".into());
        }
        let h: Header = serde_json::from_slice(&bytes[8..8 + n]).map_err(|e| e.to_string())?;
        if h.cursor.data_sha256 != data || h.cursor.tokenizer_sha256 != tokenizer {
            return Err("foreign checkpoint data/tokenizer".into());
        }
        validate_cursor(&h.cursor)?;
        let source: [u8; 32] = Sha256::digest(include_bytes!("transferable_lexical.rs")).into();
        if h.source_sha256 != source || h.step == u64::MAX {
            return Err("checkpoint source/step mismatch".into());
        }
        let cfg = TlConfig {
            vocab: usize::try_from(h.cfg[0]).map_err(|_| "vocab range")?,
            h_dim: usize::try_from(h.cfg[1]).map_err(|_| "hidden range")?,
            h_clamp: i32::try_from(h.cfg[2]).map_err(|_| "state clamp range")?,
            m_clamp: i32::try_from(h.cfg[3]).map_err(|_| "evidence clamp range")?,
            recurrent_shift: u32::try_from(h.cfg[4]).map_err(|_| "recurrent shift")?,
            score_shift: u32::try_from(h.cfg[5]).map_err(|_| "score shift")?,
        };
        cfg.validate()?;
        let tcfg = TlTrainConfig {
            lr: f64::from_bits(h.opt[0]),
            beta1: f64::from_bits(h.opt[1]),
            beta2: f64::from_bits(h.opt[2]),
            weight_decay: f64::from_bits(h.opt[3]),
            grad_clip: f64::from_bits(h.opt[4]),
            seed: h.opt[5],
        };
        validate_optimizer(tcfg)?;
        let v = cfg.vocab;
        let d = cfg.h_dim;
        let lengths = [
            (v + 1) * d,
            d * (d + TL_F_DIM),
            d * d,
            d * (d + TL_F_DIM + TL_EVENTS),
            (v + 2) * (2 * d + TL_F_DIM + TL_EVENTS),
            d,
            v + 2,
        ];
        let expected = lengths
            .iter()
            .try_fold(0usize, |n, x| n.checked_add(*x))
            .and_then(|x| x.checked_mul(12))
            .ok_or("checkpoint dimensions overflow")?;
        if expected != end - 8 - n {
            return Err("checkpoint parameter lengths mismatch".into());
        }
        let mut tr = Self::new(cfg, tcfg)?;
        let mut pos = 8 + n;
        let groups = [
            &mut tr.e,
            &mut tr.wi,
            &mut tr.wh,
            &mut tr.wf,
            &mut tr.wo,
            &mut tr.bh,
            &mut tr.bo,
            &mut tr.me,
            &mut tr.mi,
            &mut tr.mh,
            &mut tr.mf,
            &mut tr.mo,
            &mut tr.mbh,
            &mut tr.mbo,
            &mut tr.ve,
            &mut tr.vi,
            &mut tr.vh,
            &mut tr.vf,
            &mut tr.vo,
            &mut tr.vbh,
            &mut tr.vbo,
        ];
        for (i, g) in groups.into_iter().enumerate() {
            for x in g.iter_mut() {
                let raw = u32::from_le_bytes(
                    bytes[pos..pos + 4]
                        .try_into()
                        .map_err(|_| "truncated latent")?,
                );
                pos += 4;
                *x = f32::from_bits(raw);
                if !x.is_finite() || (i >= 14 && *x < 0.) {
                    return Err("invalid latent or moment".into());
                }
            }
        }
        if pos != end {
            return Err("trailing checkpoint data".into());
        }
        tr.step = h.step;
        Ok((tr, h.cursor))
    }
}
fn validate_optimizer(c: TlTrainConfig) -> Result<(), String> {
    if !c.lr.is_finite()
        || c.lr <= 0.
        || !c.beta1.is_finite()
        || !(0.0..1.0).contains(&c.beta1)
        || !c.beta2.is_finite()
        || !(0.0..1.0).contains(&c.beta2)
        || !c.weight_decay.is_finite()
        || c.weight_decay < 0.
        || !c.grad_clip.is_finite()
        || c.grad_clip < 0.
    {
        return Err("invalid checkpoint optimizer".into());
    }
    Ok(())
}
fn validate_cursor(c: &TrainingCursor) -> Result<(), String> {
    let a = f64::from_bits(c.lr_start_bits);
    let b = f64::from_bits(c.lr_end_bits);
    if c.schedule_total == 0
        || c.next_batch > c.schedule_total
        || !a.is_finite()
        || !b.is_finite()
        || a <= 0.
        || b <= 0.
    {
        return Err("invalid checkpoint schedule".into());
    }
    Ok(())
}
#[cfg(test)]
mod continuation_tests {
    use super::*;
    fn sample() -> TlExample {
        TlExample {
            sel: vec![],
            res: vec![],
            facts: SlFacts::default(),
            observed: vec![1, 2],
            actions: vec![TlAction::Generate(3), TlAction::Stop],
            weight: 1.0,
            doc: 0,
            grounded: false,
            terminal_stop: true,
        }
    }
    fn cursor() -> TrainingCursor {
        TrainingCursor {
            data_sha256: [11; 32],
            tokenizer_sha256: [22; 32],
            rng_state: 37,
            next_batch: 0,
            schedule_total: 8,
            lr_start_bits: 0.02f64.to_bits(),
            lr_end_bits: 0.002f64.to_bits(),
        }
    }
    #[test]
    fn exact_checkpoint_resumes_latents_moments_and_updates() {
        let mut cfg = TlConfig::new(8);
        cfg.h_dim = 8;
        let mut a = TlTrainer::new(cfg, TlTrainConfig::default()).unwrap();
        let mut c = cursor();
        for _ in 0..3 {
            a.train_batch(&[sample()]);
            c.next_batch += 1;
        }
        let b = a.checkpoint(&c).unwrap();
        let (mut r, rc) = TlTrainer::from_checkpoint(&b, [11; 32], [22; 32]).unwrap();
        assert_eq!(c, rc);
        assert_eq!(b, r.checkpoint(&rc).unwrap());
        for _ in 0..4 {
            a.train_batch(&[sample()]);
            r.train_batch(&[sample()]);
        }
        assert_eq!(a.checkpoint(&c).unwrap(), r.checkpoint(&c).unwrap());
        assert_eq!(a.model().unwrap(), r.model().unwrap());
    }
    #[test]
    fn rejects_foreign_corrupt_and_truncated_checkpoint() {
        let mut cfg = TlConfig::new(8);
        cfg.h_dim = 8;
        let a = TlTrainer::new(cfg, TlTrainConfig::default()).unwrap();
        let b = a.checkpoint(&cursor()).unwrap();
        assert!(TlTrainer::from_checkpoint(&b, [9; 32], [22; 32]).is_err());
        assert!(TlTrainer::from_checkpoint(&b[..b.len() - 1], [11; 32], [22; 32]).is_err());
        let mut corrupt = b.clone();
        let n = corrupt.len() / 2;
        corrupt[n] ^= 1;
        assert!(TlTrainer::from_checkpoint(&corrupt, [11; 32], [22; 32]).is_err());
    }
}
