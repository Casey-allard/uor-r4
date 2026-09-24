//! Precompiled sparse readout with caller-owned scratch and output storage.
use super::*;
#[derive(Clone, Debug)]
pub struct TlReadPlan {
    cols: usize,
    rows: usize,
    offsets: Vec<usize>,
    entries: Vec<u16>,
    shifts: Vec<u32>,
    bias: Vec<i32>,
    h: usize,
}
impl TlReadPlan {
    pub fn compile(m: &TlModel) -> Result<Self, String> {
        if m.wo
            .rows
            .checked_mul(m.wo.cols)
            .ok_or("read plan size overflow")?
            > 16_777_216
        {
            return Err("read plan exceeds declared coefficient bound".into());
        }
        let checked = TlModel::from_bytes(&m.to_bytes())?;
        if checked != *m {
            return Err("invalid source model".into());
        }
        let w = &m.wo;
        if w.cols >= 32768 || w.cols != 2 * m.h_dim + TL_F_DIM + TL_EVENTS || w.rows != m.vocab + 2
        {
            return Err("readout dimensions outside plan envelope".into());
        }
        let mut offsets = Vec::with_capacity(w.rows + 1);
        let mut entries = Vec::new();
        offsets.push(0);
        for r in 0..w.rows {
            for c in 0..w.cols {
                let v = unpack_ternary(&w.packed, r, c, w.cols);
                if v != 0 {
                    entries.push(c as u16 | if v < 0 { 0x8000 } else { 0 });
                }
            }
            offsets.push(entries.len());
        }
        Ok(Self {
            cols: w.cols,
            rows: w.rows,
            offsets,
            entries,
            shifts: w.shift.clone(),
            bias: m.bo.clone(),
            h: m.h_dim,
        })
    }
    pub fn workspace_len(&self) -> usize {
        self.cols
    }
    pub fn output_len(&self) -> usize {
        self.rows
    }
    pub fn nonzeros(&self) -> usize {
        self.entries.len()
    }
    pub fn bytes(&self) -> usize {
        self.entries.len() * 2 + self.offsets.len() * std::mem::size_of::<usize>() + self.rows * 8
    }
    pub fn score_into(
        &self,
        h: &[i32],
        m: &[i32],
        f: &[i32],
        event: usize,
        scratch: &mut [i32],
        out: &mut [i32],
    ) -> Result<(), String> {
        if h.len() != self.h
            || m.len() != self.h
            || f.len() != TL_F_DIM
            || event >= TL_EVENTS
            || scratch.len() != self.cols
            || out.len() != self.rows
        {
            return Err("read plan input dimensions/event mismatch".into());
        }
        scratch[..self.h].copy_from_slice(h);
        scratch[self.h..2 * self.h].copy_from_slice(m);
        scratch[2 * self.h..2 * self.h + TL_F_DIM].copy_from_slice(f);
        scratch[2 * self.h + TL_F_DIM..].fill(0);
        scratch[2 * self.h + TL_F_DIM + event] = 1;
        for r in 0..self.rows {
            let mut acc = 0i64;
            for &entry in &self.entries[self.offsets[r]..self.offsets[r + 1]] {
                let x = i64::from(scratch[(entry & 0x7fff) as usize]);
                if entry & 0x8000 != 0 {
                    acc -= x;
                } else {
                    acc += x;
                }
            }
            let z = (acc << self.shifts[r]) + i64::from(self.bias[r]);
            out[r] = i32::try_from(z).map_err(|_| "read plan score overflow")?;
        }
        Ok(())
    }
}
#[cfg(test)]
mod continuation_tests {
    use super::*;
    #[test]
    fn compiled_readout_matches_every_row_and_reuses_buffers() {
        let mut c = TlConfig::new(32);
        c.h_dim = 8;
        let mut m = TlTrainer::new(c, TlTrainConfig::default())
            .unwrap()
            .model()
            .unwrap();
        m.wo.packed.fill(0x19);
        let p = TlReadPlan::compile(&m).unwrap();
        let mut scratch = vec![0; p.workspace_len()];
        let mut out = vec![0; p.output_len()];
        let addr = (scratch.as_ptr(), out.as_ptr());
        let f = m.typed_block(&[], &[], SlFacts::default());
        for i in 0..24 {
            let h = vec![i - 12; c.h_dim];
            let evidence = vec![i % 5; c.h_dim];
            p.score_into(&h, &evidence, &f, i as usize % 4, &mut scratch, &mut out)
                .unwrap();
            assert_eq!(out, m.readout(&h, &evidence, &f, i as usize % 4));
            assert_eq!((scratch.as_ptr(), out.as_ptr()), addr);
        }
    }
    #[test]
    fn invalid_read_plan_dimensions_return_error() {
        let m = TlTrainer::new(TlConfig::new(8), TlTrainConfig::default())
            .unwrap()
            .model()
            .unwrap();
        let p = TlReadPlan::compile(&m).unwrap();
        assert!(p.score_into(&[], &[], &[], 4, &mut [], &mut []).is_err());
    }
}
