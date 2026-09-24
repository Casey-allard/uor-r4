//! Compiled complete native execution. Construction allocates; successful request/token calls do not.
use super::state_lexical::SlFacts;
use super::transferable_lexical::*;
#[derive(Clone, Debug)]
struct Map {
    offsets: Vec<usize>,
    entries: Vec<u16>,
    shifts: Vec<u32>,
    cols: usize,
}
impl Map {
    fn compile(m: &TlLinear) -> Self {
        let mut offsets = vec![0];
        let mut entries = Vec::new();
        for r in 0..m.rows {
            for c in 0..m.cols {
                let flat = r * m.cols + c;
                let code = (m.packed[flat >> 2] >> ((flat & 3) << 1)) & 3;
                if code == 1 || code == 2 {
                    entries.push(c as u16 | if code == 2 { 0x8000 } else { 0 });
                }
            }
            offsets.push(entries.len());
        }
        Self {
            offsets,
            entries,
            shifts: m.shift.clone(),
            cols: m.cols,
        }
    }
    fn apply(&self, input: &[i32], out: &mut [i32]) -> Result<(), String> {
        if input.len() != self.cols || out.len() + 1 != self.offsets.len() {
            return Err("compiled map dimensions".into());
        }
        for (r, slot) in out.iter_mut().enumerate() {
            let mut a = 0i64;
            for &e in &self.entries[self.offsets[r]..self.offsets[r + 1]] {
                let x = i64::from(input[(e & 0x7fff) as usize]);
                if e & 0x8000 == 0 {
                    a += x
                } else {
                    a -= x
                }
            }
            *slot = i32::try_from(a << self.shifts[r]).map_err(|_| "compiled map overflow")?;
        }
        Ok(())
    }
    fn bytes(&self) -> usize {
        self.offsets.len() * std::mem::size_of::<usize>()
            + self.entries.len() * 2
            + self.shifts.len() * 4
    }
}
#[derive(Clone, Debug)]
pub struct Execution {
    model: TlModel,
    wi: Map,
    wh: Map,
    wf: Map,
    wo: Map,
    rotations: [usize; TL_CONTENT_POSITIONS],
    embedding_bases: Vec<usize>,
}
pub struct Workspace {
    pub state: Vec<i32>,
    pub meaning: Vec<i32>,
    pub facts: [i32; TL_F_DIM],
    pub logits: Vec<i32>,
    next: Vec<i32>,
    offsets: [Vec<i32>; TL_EVENTS],
    scratch: Vec<i32>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Run {
    pub tokens: usize,
    pub actions: usize,
    pub copied: usize,
    pub stopped: bool,
}
impl Execution {
    pub fn compile(model: &TlModel) -> Result<Self, String> {
        model.validate()?;
        if model.vocab > 65536 {
            return Err("compiled vocabulary bound".into());
        }
        let half = model.h_dim / 2;
        let band = (half / TL_CONTENT_POSITIONS).max(1);
        Ok(Self {
            model: model.clone(),
            wi: Map::compile(&model.wi),
            wh: Map::compile(&model.wh),
            wf: Map::compile(&model.wf),
            wo: Map::compile(&model.wo),
            rotations: std::array::from_fn(|p| (p * band) % half),
            embedding_bases: (0..model.e.rows).map(|r| r * model.h_dim).collect(),
        })
    }
    pub fn workspace(&self) -> Workspace {
        let h = self.model.h_dim;
        Workspace {
            state: vec![0; h],
            meaning: vec![0; h],
            facts: [0; TL_F_DIM],
            logits: vec![0; self.model.vocab + 2],
            next: vec![0; h],
            offsets: std::array::from_fn(|_| vec![0; h]),
            scratch: vec![0; 2 * h + TL_F_DIM + TL_EVENTS],
        }
    }
    pub fn logical_plan_bytes(&self) -> usize {
        self.wi.bytes()
            + self.wh.bytes()
            + self.wf.bytes()
            + self.wo.bytes()
            + self.embedding_bases.len() * std::mem::size_of::<usize>()
    }
    fn embedding(&self, row: usize, col: usize) -> i32 {
        i32::from(self.model.e.codes[self.embedding_bases[row] + col]) << self.model.e.shift[row]
    }
    pub fn source(&self) -> &TlModel {
        &self.model
    }
    fn check(&self, w: &Workspace) -> Result<(), String> {
        let h = self.model.h_dim;
        if w.state.len() != h
            || w.meaning.len() != h
            || w.logits.len() != self.model.vocab + 2
            || w.next.len() != h
            || w.offsets.iter().any(|r| r.len() != h)
            || w.scratch.len() != 2 * h + TL_F_DIM + TL_EVENTS
        {
            return Err("foreign execution workspace".into());
        }
        Ok(())
    }
    pub fn start(
        &self,
        sel: &[u32],
        res: &[u32],
        facts: SlFacts,
        observed: &[u32],
        w: &mut Workspace,
    ) -> Result<(), String> {
        self.start_control(sel, res, facts, observed, false, w)
    }
    pub fn start_control(
        &self,
        sel: &[u32],
        res: &[u32],
        facts: SlFacts,
        observed: &[u32],
        blind: bool,
        w: &mut Workspace,
    ) -> Result<(), String> {
        self.check(w)?;
        if sel.len() > 65536 || res.len() > 65536 || observed.len() > 1_048_576 {
            return Err("request exceeds declared bound".into());
        }
        let m = &self.model;
        let h = m.h_dim;
        let half = h / 2;
        w.meaning.fill(0);
        for (toks, off) in [(sel, 0), (res, half)] {
            for p in
                0..toks.len().min(TL_CONTENT_TOKENS) + usize::from(toks.len() > TL_CONTENT_TOKENS)
            {
                let token = if p == TL_CONTENT_TOKENS {
                    toks[toks.len() - 1]
                } else {
                    toks[p]
                };
                let row = if blind { m.vocab } else { m.token_row(token) };
                let k = self.rotations[p];
                for c in 0..half {
                    let j = c + k;
                    w.meaning[off + if j >= half { j - half } else { j }] += self.embedding(row, c);
                }
            }
        }
        for x in &mut w.meaning {
            *x = (*x).clamp(-m.m_clamp, m.m_clamp);
        }
        let f = &mut w.facts;
        f.fill(0);
        f[(facts.history as usize).min(3)] = 1;
        f[4] = i32::from(facts.derived);
        f[5] = i32::from(facts.prior_differs);
        f[6] = i32::from(facts.committed);
        f[7] = i32::from(facts.key_changed);
        f[8] = i32::from(!sel.is_empty() && sel == res);
        f[9] = sel
            .iter()
            .zip(res)
            .take_while(|(a, b)| a == b)
            .take(4)
            .count() as i32;
        f[10] = (0..sel.len().max(res.len()))
            .filter(|i| sel.get(*i) != res.get(*i))
            .take(4)
            .count() as i32;
        f[11] = sel.len().min(4) as i32;
        f[12] = res.len().min(4) as i32;
        f[13] = sel.iter().filter(|&&t| !m.known_token(t)).take(4).count() as i32;
        f[14] = res.iter().filter(|&&t| !m.known_token(t)).take(4).count() as i32;
        w.scratch[..h].copy_from_slice(&w.meaning);
        w.scratch[h..h + TL_F_DIM].copy_from_slice(f);
        self.wi.apply(&w.scratch[..h + TL_F_DIM], &mut w.state)?;
        for (x, &b) in w.state.iter_mut().zip(&m.bh) {
            *x = x
                .checked_add(b)
                .ok_or("initial state overflow")?
                .clamp(-m.h_clamp, m.h_clamp);
        }
        for event in 0..TL_EVENTS {
            w.scratch[h + TL_F_DIM..h + TL_F_DIM + TL_EVENTS].fill(0);
            w.scratch[h + TL_F_DIM + event] = 1;
            let dst = &mut w.offsets[event];
            self.wf.apply(&w.scratch[..h + TL_F_DIM + TL_EVENTS], dst)?;
            for (x, &b) in dst.iter_mut().zip(&m.bh) {
                *x = x.checked_add(b).ok_or("event offset overflow")?;
            }
        }
        for &token in observed {
            self.advance(TL_EV_OBSERVE, Some(token), w)?;
        }
        Ok(())
    }
    pub fn read(&self, event: usize, w: &mut Workspace) -> Result<(), String> {
        self.check(w)?;
        if event >= TL_EVENTS {
            return Err("invalid read event".into());
        }
        let h = self.model.h_dim;
        w.scratch[..h].copy_from_slice(&w.state);
        w.scratch[h..2 * h].copy_from_slice(&w.meaning);
        w.scratch[2 * h..2 * h + TL_F_DIM].copy_from_slice(&w.facts);
        w.scratch[2 * h + TL_F_DIM..].fill(0);
        w.scratch[2 * h + TL_F_DIM + event] = 1;
        self.wo.apply(&w.scratch, &mut w.logits)?;
        for (x, &b) in w.logits.iter_mut().zip(&self.model.bo) {
            *x = x.checked_add(b).ok_or("readout bias overflow")?;
        }
        Ok(())
    }
    pub fn advance(
        &self,
        event: usize,
        token: Option<u32>,
        w: &mut Workspace,
    ) -> Result<(), String> {
        self.check(w)?;
        if event >= TL_EVENTS {
            return Err("invalid update event".into());
        }
        let m = &self.model;
        let row = token.map(|t| m.token_row(t)).unwrap_or(m.vocab);
        self.wh.apply(&w.state, &mut w.next)?;
        for (r, x) in w.next.iter_mut().enumerate() {
            let v = i64::from(self.embedding(row, r))
                + i64::from(*x >> m.recurrent_shift)
                + i64::from(w.offsets[event][r]);
            *x = i32::try_from(v)
                .map_err(|_| "state addition overflow")?
                .clamp(-m.h_clamp, m.h_clamp);
        }
        std::mem::swap(&mut w.state, &mut w.next);
        Ok(())
    }
    pub fn choose(&self, copy_legal: bool, w: &Workspace) -> Result<TlAction, String> {
        self.check(w)?;
        let v = self.model.vocab;
        let mut best = 0;
        for i in 1..v {
            if w.logits[i] > w.logits[best] {
                best = i;
            }
        }
        if w.logits[v + 1] > w.logits[best] {
            best = v + 1;
        }
        if copy_legal && w.logits[v] > w.logits[best] {
            best = v;
        }
        Ok(if best < v {
            TlAction::Generate(best as u32)
        } else if best == v {
            TlAction::Copy
        } else {
            TlAction::Stop
        })
    }
    #[allow(clippy::too_many_arguments)]
    pub fn run(
        &self,
        sel: &[u32],
        res: &[u32],
        facts: SlFacts,
        observed: &[u32],
        owned: &[u32],
        blind: bool,
        source_disabled: bool,
        w: &mut Workspace,
        tokens: &mut [u32],
        actions: &mut [TlAction],
    ) -> Result<Run, String> {
        if actions.len() > 65536 || tokens.len() < actions.len() || owned.len() > 65536 {
            return Err("invalid generation buffers".into());
        }
        self.start_control(sel, res, facts, observed, blind, w)?;
        let owned = if source_disabled { &[][..] } else { owned };
        let mut out = Run {
            tokens: 0,
            actions: 0,
            copied: 0,
            stopped: false,
        };
        let mut event = TL_EV_OBSERVE;
        for slot in actions {
            self.read(event, w)?;
            let action = self.choose(out.copied < owned.len(), w)?;
            *slot = action;
            out.actions += 1;
            let token = match action {
                TlAction::Generate(t) => t,
                TlAction::Copy => {
                    let t = owned[out.copied];
                    out.copied += 1;
                    t
                }
                TlAction::Stop => {
                    out.stopped = true;
                    break;
                }
            };
            tokens[out.tokens] = token;
            out.tokens += 1;
            event = action.event();
            self.advance(event, if blind { None } else { Some(token) }, w)?;
        }
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn model() -> TlModel {
        let mut c = TlConfig::new(16);
        c.h_dim = 8;
        let mut m = TlTrainer::new(c, TlTrainConfig::default())
            .unwrap()
            .model()
            .unwrap();
        m.wo.packed.fill(0x19);
        m
    }
    #[test]
    fn compiled_prompt_reproduces_grounded_native_state() {
        let m = model();
        let p = Execution::compile(&m).unwrap();
        let mut w = p.workspace();
        let sel = [1, 2, 3, 4, 5, 6];
        let res = [1, 2, 7];
        let f = SlFacts {
            history: 3,
            derived: true,
            prior_differs: true,
            committed: true,
            key_changed: false,
        };
        let meaning = m.content_feature(&sel, &res);
        let facts = m.typed_block(&sel, &res, f);
        let mut h = m.init_state(&meaning, &facts);
        for t in [3, 4, 5] {
            h = m.transition(&h, TL_EV_OBSERVE, Some(t), &meaning, &facts);
        }
        p.start(&sel, &res, f, &[3, 4, 5], &mut w).unwrap();
        assert_eq!(w.state, h);
        assert_eq!(w.meaning, meaning);
        assert_eq!(w.facts.to_vec(), facts);
        for e in 0..4 {
            p.read(e, &mut w).unwrap();
            assert_eq!(w.logits, m.readout(&h, &meaning, &facts, e));
        }
    }
    #[test]
    fn complete_rollout_matches_all_actions_tokens_and_final_state() {
        let m = model();
        let p = Execution::compile(&m).unwrap();
        let mut w = p.workspace();
        for len in [0, 1, 3, 5, 17] {
            let sel = (0..len).map(|x| x as u32 * 7).collect::<Vec<_>>();
            for blind in [false, true] {
                for disabled in [false, true] {
                    let a = m.rollout(
                        &sel,
                        &[1, 2],
                        SlFacts::default(),
                        &[5, 6, 7],
                        &sel,
                        64,
                        blind,
                        disabled,
                    );
                    let mut ts = [0; 64];
                    let mut acts = [TlAction::Stop; 64];
                    let b = p
                        .run(
                            &sel,
                            &[1, 2],
                            SlFacts::default(),
                            &[5, 6, 7],
                            &sel,
                            blind,
                            disabled,
                            &mut w,
                            &mut ts,
                            &mut acts,
                        )
                        .unwrap();
                    assert_eq!(a.tokens, ts[..b.tokens]);
                    assert_eq!(a.actions, acts[..b.actions]);
                    assert_eq!(a.stopped, b.stopped);
                    assert_eq!(a.state_digest, state_digest(&w.state));
                }
            }
        }
    }
    #[test]
    fn copy_stop_tie_order_and_stop_state_are_native() {
        let mut m = model();
        m.wo.packed.fill(0);
        m.bo.fill(-1);
        m.bo[m.vocab] = 1;
        m.bo[m.vocab + 1] = 1;
        let p = Execution::compile(&m).unwrap();
        let mut w = p.workspace();
        p.start(&[1], &[], SlFacts::default(), &[], &mut w).unwrap();
        p.read(0, &mut w).unwrap();
        assert_eq!(p.choose(true, &w).unwrap(), TlAction::Stop);
        let h = w.state.clone();
        let b = p
            .run(
                &[1],
                &[],
                SlFacts::default(),
                &[],
                &[1],
                false,
                false,
                &mut w,
                &mut [0; 4],
                &mut [TlAction::Stop; 4],
            )
            .unwrap();
        assert!(b.stopped);
        assert_eq!(b.tokens, 0);
        assert_eq!(w.state, h);
    }
    #[test]
    fn malformed_inputs_do_not_panic() {
        let p = Execution::compile(&model()).unwrap();
        let mut w = p.workspace();
        assert!(p.advance(4, None, &mut w).is_err());
        assert!(p.read(4, &mut w).is_err());
        assert!(p
            .run(
                &[],
                &[],
                SlFacts::default(),
                &[],
                &[],
                false,
                false,
                &mut w,
                &mut [],
                &mut [TlAction::Stop; 1]
            )
            .is_err());
        w.state.pop();
        assert!(p.start(&[], &[], SlFacts::default(), &[], &mut w).is_err());
    }
}
