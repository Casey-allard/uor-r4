//! Exact addressed objects and observational offers. No emission policy lives here.
use std::fmt;

pub const OCCURRENCES: usize = 256;
pub const SPAN_BYTES: usize = 64;
pub const RESULTS: usize = 8;
pub const SNAPSHOT_MAX: usize = 65_536;
pub type Result<T> = std::result::Result<T, ObjectError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectError {
    InvalidReference,
    InvalidExtent,
    InvalidGeometry,
    InvalidScalar,
    Overflow,
    IllegalAction,
    UnknownOffer,
    OfferConflict,
    KeyRequired,
    NoKeyPending,
    Exhausted,
    InvalidSnapshot,
    SnapshotBinding,
    SnapshotLimit,
}
impl fmt::Display for ObjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "addressed object error: {self:?}")
    }
}
impl std::error::Error for ObjectError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    Input,
    Generated,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    Byte(u8),
    Eos,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Hold,
    AcquireA,
    AcquireB,
    AddAB,
    SubAB,
    Advance,
    Clear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Occurrence {
    pub epoch: u64,
    pub sequence: u64,
    pub turn: u64,
    pub byte: u8,
    pub origin: Origin,
    pub keys: [u16; 2],
    pub roots: [u16; 4],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reference {
    Occurrence {
        epoch: u64,
        turn: u64,
        start: u64,
        end: u64,
    },
    Result {
        epoch: u64,
        id: u64,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lease {
    reference: Reference,
    bytes: [u8; SPAN_BYTES],
    len: u8,
    keys: [u16; 2],
    roots: [u16; 4],
}
impl Lease {
    pub fn reference(&self) -> Reference {
        self.reference
    }
    pub fn payload(&self) -> &[u8] {
        &self.bytes[..usize::from(self.len)]
    }
    pub fn keys(&self) -> [u16; 2] {
        self.keys
    }
    pub fn roots(&self) -> [u16; 4] {
        self.roots
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Derivation {
    pub id: u64,
    pub action: Action,
    pub operands: [Reference; 2],
    pub operand_values: [i64; 2],
    pub value: i64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResultRecord {
    pub lease: Lease,
    pub derivation: Derivation,
    pub published_at: u64,
    pub turn: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveLease {
    lease: Lease,
    cursor: u8,
    acknowledged: bool,
    provisional: Option<Derivation>,
}
impl ActiveLease {
    pub fn lease(&self) -> &Lease {
        &self.lease
    }
    pub fn cursor(&self) -> u8 {
        self.cursor
    }
    pub fn acknowledged(&self) -> bool {
        self.acknowledged
    }
    pub fn byte(&self) -> u8 {
        self.lease.bytes[usize::from(self.cursor)]
    }
    pub fn provisional(&self) -> Option<Derivation> {
        self.provisional
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Offer {
    pub id: u64,
    pub epoch: u64,
    pub frontier: u64,
    pub turn: u64,
    pub symbol: Symbol,
    pub action: Action,
    operands: [Option<Lease>; 2],
    active: Option<ActiveLease>,
}
impl Offer {
    pub fn active(&self) -> Option<&ActiveLease> {
        self.active.as_ref()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AwaitingKey {
    byte: u8,
    ready: Option<Derivation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectSession {
    model: [u8; 32],
    geometry: [u8; 32],
    epoch: u64,
    turn: u64,
    seen: u64,
    next_offer: u64,
    next_result: u64,
    publications: u8,
    occurrences: [Option<Occurrence>; OCCURRENCES],
    operands: [Option<Lease>; 2],
    active: Option<ActiveLease>,
    results: [Option<ResultRecord>; RESULTS],
    pending: Option<Offer>,
    awaiting_key: Option<AwaitingKey>,
}
impl ObjectSession {
    /// `epoch` is a host-assigned non-reused session namespace. Independent
    /// sessions that exchange leases must never share an epoch. After complete
    /// source eviction the owned bytes are authoritative, so no retained source
    /// can distinguish a host-created namespace collision.
    pub fn new(model: [u8; 32], geometry: [u8; 32], epoch: u64) -> Self {
        Self {
            model,
            geometry,
            epoch,
            turn: 0,
            seen: 0,
            next_offer: 1,
            next_result: 1,
            publications: 0,
            occurrences: [None; OCCURRENCES],
            operands: [None; 2],
            active: None,
            results: [None; RESULTS],
            pending: None,
            awaiting_key: None,
        }
    }
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    pub fn turn(&self) -> u64 {
        self.turn
    }
    pub fn frontier(&self) -> u64 {
        self.seen
    }
    pub fn active(&self) -> Option<&ActiveLease> {
        self.active.as_ref()
    }
    pub fn pending(&self) -> Option<&Offer> {
        self.pending.as_ref()
    }
    pub fn key_pending(&self) -> bool {
        self.awaiting_key.is_some()
    }
    pub fn publications_this_turn(&self) -> u8 {
        self.publications
    }
    pub fn occurrence(&self, epoch: u64, sequence: u64) -> Result<&Occurrence> {
        if epoch != self.epoch || sequence >= self.seen {
            return Err(ObjectError::InvalidReference);
        }
        self.occurrences[(sequence & 255) as usize]
            .as_ref()
            .filter(|r| r.epoch == epoch && r.sequence == sequence)
            .ok_or(ObjectError::InvalidReference)
    }
    pub fn result(&self, epoch: u64, id: u64) -> Result<&ResultRecord> {
        if epoch != self.epoch || id == 0 {
            return Err(ObjectError::InvalidReference);
        }
        self.results[(id & 7) as usize]
            .as_ref()
            .filter(|r| r.derivation.id == id)
            .ok_or(ObjectError::InvalidReference)
    }
    pub fn acquire_occurrence(&self, epoch: u64, start: u64, len: u8) -> Result<Lease> {
        if len == 0 || usize::from(len) > SPAN_BYTES {
            return Err(ObjectError::InvalidExtent);
        }
        let end = start
            .checked_add(u64::from(len))
            .ok_or(ObjectError::InvalidExtent)?;
        if end > self.seen {
            return Err(ObjectError::InvalidExtent);
        }
        let first = self.occurrence(epoch, start)?;
        let mut bytes = [0; SPAN_BYTES];
        for (i, output) in bytes[..usize::from(len)].iter_mut().enumerate() {
            let entry = self.occurrence(epoch, start + i as u64)?;
            if entry.turn != first.turn || entry.origin != first.origin {
                return Err(ObjectError::InvalidExtent);
            }
            *output = entry.byte;
        }
        Ok(Lease {
            reference: Reference::Occurrence {
                epoch,
                turn: first.turn,
                start,
                end,
            },
            bytes,
            len,
            keys: first.keys,
            roots: first.roots,
        })
    }
    pub fn acquire_result(&self, epoch: u64, id: u64) -> Result<Lease> {
        Ok(self.result(epoch, id)?.lease)
    }
    fn check_owned(&self, lease: &Lease) -> Result<()> {
        validate_lease(lease, self.epoch, self.seen, self.next_result)?;
        self.validate_retained_lease(lease)?;
        // Acquisition constructors are private-field capabilities. Their payload
        // remains owned even if the originating ring/result slot is overwritten.
        Ok(())
    }
    fn prospective(
        &self,
        action: Action,
        operands: [Option<Lease>; 2],
    ) -> Result<Option<ActiveLease>> {
        for lease in operands.iter().flatten() {
            self.check_owned(lease)?;
        }
        let acquired = |index: usize| -> Result<Option<ActiveLease>> {
            Ok(Some(ActiveLease {
                lease: operands[index].ok_or(ObjectError::IllegalAction)?,
                cursor: 0,
                acknowledged: false,
                provisional: None,
            }))
        };
        match action {
            Action::Hold => Ok(self.active),
            Action::AcquireA => acquired(0),
            Action::AcquireB => acquired(1),
            Action::Clear => Ok(None),
            Action::Advance => {
                let mut active = self.active.ok_or(ObjectError::IllegalAction)?;
                if !active.acknowledged || active.cursor + 1 >= active.lease.len {
                    return Err(ObjectError::IllegalAction);
                }
                active.cursor += 1;
                active.acknowledged = false;
                Ok(Some(active))
            }
            Action::AddAB | Action::SubAB => {
                if self.publications >= 2 || self.next_result == u64::MAX {
                    return Err(ObjectError::IllegalAction);
                }
                let a = operands[0].ok_or(ObjectError::IllegalAction)?;
                let b = operands[1].ok_or(ObjectError::IllegalAction)?;
                let values = [decode_i64(a.payload())?, decode_i64(b.payload())?];
                let value = execute(action, values[0], values[1])?;
                let (bytes, len) = encode_i64(value);
                let derivation = Derivation {
                    id: self.next_result,
                    action,
                    operands: [a.reference, b.reference],
                    operand_values: values,
                    value,
                };
                Ok(Some(ActiveLease {
                    lease: Lease {
                        reference: Reference::Result {
                            epoch: self.epoch,
                            id: self.next_result,
                        },
                        bytes,
                        len,
                        keys: [0; 2],
                        roots: [0; 4],
                    },
                    cursor: 0,
                    acknowledged: false,
                    provisional: Some(derivation),
                }))
            }
        }
    }
    pub fn legal_actions(&self, a: Option<Lease>, b: Option<Lease>) -> [bool; 7] {
        [
            Action::Hold,
            Action::AcquireA,
            Action::AcquireB,
            Action::AddAB,
            Action::SubAB,
            Action::Advance,
            Action::Clear,
        ]
        .map(|action| self.awaiting_key.is_none() && self.prospective(action, [a, b]).is_ok())
    }
    /// Cache one already-learned offer. This never replaces the supplied symbol
    /// with a lease byte; exact content is input to the caller's emission head.
    pub fn cache_offer(
        &mut self,
        symbol: Symbol,
        action: Action,
        a: Option<Lease>,
        b: Option<Lease>,
    ) -> Result<Offer> {
        if self.awaiting_key.is_some() {
            return Err(ObjectError::KeyRequired);
        }
        if let Some(offer) = self.pending {
            return if offer.symbol == symbol && offer.action == action && offer.operands == [a, b] {
                Ok(offer)
            } else {
                Err(ObjectError::OfferConflict)
            };
        }
        if self.next_offer == u64::MAX {
            return Err(ObjectError::Exhausted);
        }
        let active = self.prospective(action, [a, b])?;
        let offer = Offer {
            id: self.next_offer,
            epoch: self.epoch,
            frontier: self.seen,
            turn: self.turn,
            symbol,
            action,
            operands: [a, b],
            active,
        };
        self.pending = Some(offer);
        Ok(offer)
    }
    /// Validate first, then consume exactly this offer. Byte observations require
    /// one subsequent KEY commit; EOS creates no occurrence.
    pub fn acknowledge(&mut self, actual: Symbol, offer_id: u64) -> Result<()> {
        let offer = self
            .pending
            .filter(|p| {
                p.id == offer_id
                    && p.epoch == self.epoch
                    && p.frontier == self.seen
                    && p.turn == self.turn
            })
            .ok_or(ObjectError::UnknownOffer)?;
        if self.awaiting_key.is_some() {
            return Err(ObjectError::KeyRequired);
        }
        if matches!(actual, Symbol::Byte(_)) && self.seen == u64::MAX {
            return Err(ObjectError::Exhausted);
        }
        self.pending = None;
        self.next_offer += 1;
        let Symbol::Byte(byte) = actual else {
            self.active = None;
            self.operands = [None; 2];
            return Ok(());
        };
        let mut ready = None;
        if actual != offer.symbol {
            self.active = None;
            self.operands = [None; 2];
        } else {
            self.operands = offer.operands;
            self.active = offer.active;
            if let Some(active) = &mut self.active {
                if active.provisional.is_some() && active.acknowledged {
                    // Hold does not advance; repeating an already acknowledged
                    // provisional byte is not canonical result emission.
                    self.active = None;
                    self.operands = [None; 2];
                } else if byte == active.byte() {
                    active.acknowledged = true;
                    if active.cursor + 1 == active.lease.len {
                        ready = active.provisional;
                    }
                } else {
                    self.active = None;
                    self.operands = [None; 2];
                }
            }
        }
        self.awaiting_key = Some(AwaitingKey { byte, ready });
        Ok(())
    }
    /// The same observed byte's KEY phase supplies its newly committed state.
    /// A ready arithmetic result receives these keys/roots and publishes once.
    pub fn commit_key(&mut self, keys: [u16; 2], roots: [u16; 4]) -> Result<()> {
        check_geometry(keys, roots)?;
        let pending = self.awaiting_key.ok_or(ObjectError::NoKeyPending)?;
        if self.seen == u64::MAX || (pending.ready.is_some() && self.next_result == u64::MAX) {
            return Err(ObjectError::Exhausted);
        }
        let sequence = self.seen;
        self.append(pending.byte, Origin::Generated, keys, roots);
        if let Some(derivation) = pending.ready {
            // A ready lease is structurally guaranteed by acknowledge/restore.
            if let Some(active) = &mut self.active {
                active.lease.keys = keys;
                active.lease.roots = roots;
                active.provisional = None;
                self.results[(derivation.id & 7) as usize] = Some(ResultRecord {
                    lease: active.lease,
                    derivation,
                    published_at: sequence,
                    turn: self.turn,
                });
                self.next_result += 1;
                self.publications += 1;
            }
        }
        self.awaiting_key = None;
        Ok(())
    }
    fn append(&mut self, byte: u8, origin: Origin, keys: [u16; 2], roots: [u16; 4]) {
        self.occurrences[(self.seen & 255) as usize] = Some(Occurrence {
            epoch: self.epoch,
            sequence: self.seen,
            turn: self.turn,
            byte,
            origin,
            keys,
            roots,
        });
        self.seen += 1;
    }
    pub fn observe_input(&mut self, byte: u8, keys: [u16; 2], roots: [u16; 4]) -> Result<()> {
        check_geometry(keys, roots)?;
        if self.awaiting_key.is_some() {
            return Err(ObjectError::KeyRequired);
        }
        if self.seen == u64::MAX {
            return Err(ObjectError::Exhausted);
        }
        // Invalidate canceled offer IDs, including across the next identical frontier.
        if self.pending.is_some() && self.next_offer == u64::MAX {
            return Err(ObjectError::Exhausted);
        }
        if self.pending.take().is_some() {
            self.next_offer += 1;
        }
        self.active = None;
        self.operands = [None; 2];
        self.append(byte, Origin::Input, keys, roots);
        Ok(())
    }
    pub fn begin_turn(&mut self) -> Result<()> {
        if self.awaiting_key.is_some() {
            return Err(ObjectError::KeyRequired);
        }
        let turn = self.turn.checked_add(1).ok_or(ObjectError::Exhausted)?;
        if self.pending.is_some() && self.next_offer == u64::MAX {
            return Err(ObjectError::Exhausted);
        }
        if self.pending.take().is_some() {
            self.next_offer += 1;
        }
        self.turn = turn;
        self.publications = 0;
        self.active = None;
        self.operands = [None; 2];
        Ok(())
    }
    pub fn reset_session(&mut self) -> Result<()> {
        let epoch = self.epoch.checked_add(1).ok_or(ObjectError::Exhausted)?;
        let next_offer = self
            .next_offer
            .checked_add(1)
            .ok_or(ObjectError::Exhausted)?;
        *self = Self::new(self.model, self.geometry, epoch);
        self.next_offer = next_offer;
        Ok(())
    }
}

fn check_geometry(keys: [u16; 2], roots: [u16; 4]) -> Result<()> {
    if keys.into_iter().chain(roots).any(|x| x >= 120) {
        Err(ObjectError::InvalidGeometry)
    } else {
        Ok(())
    }
}
pub fn execute(action: Action, a: i64, b: i64) -> Result<i64> {
    match action {
        Action::AddAB => a.checked_add(b),
        Action::SubAB => a.checked_sub(b),
        _ => return Err(ObjectError::IllegalAction),
    }
    .ok_or(ObjectError::Overflow)
}
/// Complete canonical decimal, including MIN, using checked shift/add only.
pub fn decode_i64(bytes: &[u8]) -> Result<i64> {
    if bytes.is_empty() || bytes.len() > 20 {
        return Err(ObjectError::InvalidScalar);
    }
    let negative = bytes[0] == b'-';
    let digits = if negative { &bytes[1..] } else { bytes };
    if digits.is_empty() || (digits.len() > 1 && digits[0] == b'0') || (negative && digits == b"0")
    {
        return Err(ObjectError::InvalidScalar);
    }
    let mut value = 0_i64;
    for &byte in digits {
        if !byte.is_ascii_digit() {
            return Err(ObjectError::InvalidScalar);
        }
        // Accumulate negatively so abs(MIN) is never represented in i64.
        let twice = value.checked_add(value).ok_or(ObjectError::Overflow)?;
        let four = twice.checked_add(twice).ok_or(ObjectError::Overflow)?;
        let eight = four.checked_add(four).ok_or(ObjectError::Overflow)?;
        value = eight
            .checked_add(twice)
            .and_then(|v| v.checked_sub(i64::from(byte - b'0')))
            .ok_or(ObjectError::Overflow)?;
    }
    if negative {
        Ok(value)
    } else {
        value.checked_neg().ok_or(ObjectError::Overflow)
    }
}
/// Integer decimal rendering by bounded powers-of-ten subtraction, no division.
pub fn encode_i64(value: i64) -> ([u8; SPAN_BYTES], u8) {
    const POWERS: [u64; 19] = [
        1_000_000_000_000_000_000,
        100_000_000_000_000_000,
        10_000_000_000_000_000,
        1_000_000_000_000_000,
        100_000_000_000_000,
        10_000_000_000_000,
        1_000_000_000_000,
        100_000_000_000,
        10_000_000_000,
        1_000_000_000,
        100_000_000,
        10_000_000,
        1_000_000,
        100_000,
        10_000,
        1_000,
        100,
        10,
        1,
    ];
    let mut bytes = [0; SPAN_BYTES];
    let mut len = 0;
    let mut magnitude = value.unsigned_abs();
    let mut begun = false;
    if value < 0 {
        bytes[0] = b'-';
        len = 1;
    }
    for power in POWERS {
        let mut digit = 0;
        while magnitude >= power {
            magnitude -= power;
            digit += 1;
        }
        if digit != 0 || begun || power == 1 {
            bytes[len] = b'0' + digit;
            len += 1;
            begun = true;
        }
    }
    (bytes, len as u8)
}

// Explicit canonical little-endian encoding, independent of Rust layout. Only
// snapshot/restore allocate; the observation/lease/offer path uses fixed storage.
struct Encoder(Vec<u8>);
impl Encoder {
    fn byte(&mut self, x: u8) {
        self.0.push(x)
    }
    fn u16(&mut self, x: u16) {
        self.0.extend_from_slice(&x.to_le_bytes())
    }
    fn u64(&mut self, x: u64) {
        self.0.extend_from_slice(&x.to_le_bytes())
    }
    fn option<T>(&mut self, x: Option<T>, f: impl FnOnce(&mut Self, T)) {
        self.byte(u8::from(x.is_some()));
        if let Some(v) = x {
            f(self, v)
        }
    }
    fn geometry(&mut self, keys: [u16; 2], roots: [u16; 4]) {
        for x in keys.into_iter().chain(roots) {
            self.u16(x)
        }
    }
    fn reference(&mut self, r: Reference) {
        match r {
            Reference::Occurrence {
                epoch,
                turn,
                start,
                end,
            } => {
                self.byte(0);
                for x in [epoch, turn, start, end] {
                    self.u64(x)
                }
            }
            Reference::Result { epoch, id } => {
                self.byte(1);
                self.u64(epoch);
                self.u64(id)
            }
        }
    }
    fn lease(&mut self, l: Lease) {
        self.reference(l.reference);
        self.byte(l.len);
        self.0.extend_from_slice(l.payload());
        self.geometry(l.keys, l.roots)
    }
    fn derivation(&mut self, d: Derivation) {
        self.u64(d.id);
        self.byte(action_byte(d.action));
        for r in d.operands {
            self.reference(r)
        }
        for x in d.operand_values {
            self.u64(x as u64)
        }
        self.u64(d.value as u64)
    }
    fn active(&mut self, a: ActiveLease) {
        self.lease(a.lease);
        self.byte(a.cursor);
        self.byte(u8::from(a.acknowledged));
        self.option(a.provisional, Self::derivation)
    }
    fn symbol(&mut self, s: Symbol) {
        match s {
            Symbol::Byte(b) => {
                self.byte(0);
                self.byte(b)
            }
            Symbol::Eos => self.byte(1),
        }
    }
    fn offer(&mut self, o: Offer) {
        for x in [o.id, o.epoch, o.frontier, o.turn] {
            self.u64(x)
        }
        self.symbol(o.symbol);
        self.byte(action_byte(o.action));
        for l in o.operands {
            self.option(l, Self::lease)
        }
        self.option(o.active, Self::active)
    }
}
struct Decoder<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Decoder<'a> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N]> {
        let end = self.at.checked_add(N).ok_or(ObjectError::InvalidSnapshot)?;
        let slice = self
            .bytes
            .get(self.at..end)
            .ok_or(ObjectError::InvalidSnapshot)?;
        let mut out = [0; N];
        out.copy_from_slice(slice);
        self.at = end;
        Ok(out)
    }
    fn byte(&mut self) -> Result<u8> {
        Ok(self.take::<1>()?[0])
    }
    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take()?))
    }
    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take()?))
    }
    fn boolean(&mut self) -> Result<bool> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(ObjectError::InvalidSnapshot),
        }
    }
    fn option<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T>) -> Result<Option<T>> {
        if self.boolean()? {
            Ok(Some(f(self)?))
        } else {
            Ok(None)
        }
    }
    fn geometry(&mut self) -> Result<([u16; 2], [u16; 4])> {
        let keys = [self.u16()?, self.u16()?];
        let roots = [self.u16()?, self.u16()?, self.u16()?, self.u16()?];
        check_geometry(keys, roots)?;
        Ok((keys, roots))
    }
    fn reference(&mut self) -> Result<Reference> {
        match self.byte()? {
            0 => Ok(Reference::Occurrence {
                epoch: self.u64()?,
                turn: self.u64()?,
                start: self.u64()?,
                end: self.u64()?,
            }),
            1 => Ok(Reference::Result {
                epoch: self.u64()?,
                id: self.u64()?,
            }),
            _ => Err(ObjectError::InvalidSnapshot),
        }
    }
    fn lease(&mut self) -> Result<Lease> {
        let reference = self.reference()?;
        let len = self.byte()?;
        if len == 0 || usize::from(len) > SPAN_BYTES {
            return Err(ObjectError::InvalidSnapshot);
        }
        let mut bytes = [0; SPAN_BYTES];
        for x in &mut bytes[..usize::from(len)] {
            *x = self.byte()?;
        }
        let (keys, roots) = self.geometry()?;
        Ok(Lease {
            reference,
            bytes,
            len,
            keys,
            roots,
        })
    }
    fn derivation(&mut self) -> Result<Derivation> {
        Ok(Derivation {
            id: self.u64()?,
            action: decode_action(self.byte()?)?,
            operands: [self.reference()?, self.reference()?],
            operand_values: [self.u64()? as i64, self.u64()? as i64],
            value: self.u64()? as i64,
        })
    }
    fn active(&mut self) -> Result<ActiveLease> {
        Ok(ActiveLease {
            lease: self.lease()?,
            cursor: self.byte()?,
            acknowledged: self.boolean()?,
            provisional: self.option(Self::derivation)?,
        })
    }
    fn symbol(&mut self) -> Result<Symbol> {
        match self.byte()? {
            0 => Ok(Symbol::Byte(self.byte()?)),
            1 => Ok(Symbol::Eos),
            _ => Err(ObjectError::InvalidSnapshot),
        }
    }
    fn offer(&mut self) -> Result<Offer> {
        Ok(Offer {
            id: self.u64()?,
            epoch: self.u64()?,
            frontier: self.u64()?,
            turn: self.u64()?,
            symbol: self.symbol()?,
            action: decode_action(self.byte()?)?,
            operands: [self.option(Self::lease)?, self.option(Self::lease)?],
            active: self.option(Self::active)?,
        })
    }
}
fn action_byte(a: Action) -> u8 {
    match a {
        Action::Hold => 0,
        Action::AcquireA => 1,
        Action::AcquireB => 2,
        Action::AddAB => 3,
        Action::SubAB => 4,
        Action::Advance => 5,
        Action::Clear => 6,
    }
}
fn decode_action(x: u8) -> Result<Action> {
    match x {
        0 => Ok(Action::Hold),
        1 => Ok(Action::AcquireA),
        2 => Ok(Action::AcquireB),
        3 => Ok(Action::AddAB),
        4 => Ok(Action::SubAB),
        5 => Ok(Action::Advance),
        6 => Ok(Action::Clear),
        _ => Err(ObjectError::InvalidSnapshot),
    }
}
fn validate_reference(r: Reference, epoch: u64, seen: u64, result_limit: u64) -> Result<()> {
    match r {
        Reference::Occurrence {
            epoch: e,
            start,
            end,
            ..
        } if e == epoch && start < end && end <= seen && end - start <= 64 => Ok(()),
        Reference::Result { epoch: e, id } if e == epoch && id != 0 && id < result_limit => Ok(()),
        _ => Err(ObjectError::InvalidSnapshot),
    }
}
fn validate_lease(l: &Lease, epoch: u64, seen: u64, result_limit: u64) -> Result<()> {
    validate_reference(l.reference, epoch, seen, result_limit)?;
    check_geometry(l.keys, l.roots)?;
    if l.len == 0
        || usize::from(l.len) > 64
        || l.bytes[usize::from(l.len)..].iter().any(|&b| b != 0)
    {
        return Err(ObjectError::InvalidSnapshot);
    }
    if matches!(l.reference,Reference::Occurrence{start,end,..} if end-start!=u64::from(l.len)) {
        return Err(ObjectError::InvalidSnapshot);
    }
    Ok(())
}
fn validate_derivation(d: Derivation, epoch: u64, seen: u64) -> Result<()> {
    if d.id == 0 || execute(d.action, d.operand_values[0], d.operand_values[1])? != d.value {
        return Err(ObjectError::InvalidSnapshot);
    }
    for r in d.operands {
        validate_reference(r, epoch, seen, d.id)?;
    }
    Ok(())
}
impl ObjectSession {
    fn validate_active(&self, a: ActiveLease) -> Result<()> {
        let limit = if a.provisional.is_some() {
            self.next_result
                .checked_add(1)
                .ok_or(ObjectError::InvalidSnapshot)?
        } else {
            self.next_result
        };
        validate_lease(&a.lease, self.epoch, self.seen, limit)?;
        if a.cursor >= a.lease.len {
            return Err(ObjectError::InvalidSnapshot);
        }
        if let Some(d) = a.provisional {
            validate_derivation(d, self.epoch, self.seen)?;
            self.validate_operand_values(d)?;
            if self.publications >= 2
                || d.id != self.next_result
                || a.lease.reference
                    != (Reference::Result {
                        epoch: self.epoch,
                        id: d.id,
                    })
                || encode_i64(d.value) != (a.lease.bytes, a.lease.len)
            {
                return Err(ObjectError::InvalidSnapshot);
            }
        }
        Ok(())
    }
    fn validate(&self) -> Result<()> {
        if self.next_offer == 0 || self.next_result == 0 || self.publications > 2 {
            return Err(ObjectError::InvalidSnapshot);
        }
        let lower = self.seen.saturating_sub(256);
        for (slot, entry) in self.occurrences.iter().enumerate() {
            if let Some(r) = entry {
                if r.epoch != self.epoch
                    || r.sequence < lower
                    || r.sequence >= self.seen
                    || (r.sequence & 255) as usize != slot
                    || r.turn > self.turn
                {
                    return Err(ObjectError::InvalidSnapshot);
                }
                check_geometry(r.keys, r.roots)?;
            }
        }
        for sequence in lower..self.seen {
            self.occurrence(self.epoch, sequence)?;
        }
        for l in self.operands.iter().flatten() {
            self.check_owned(l)?;
            self.validate_retained_lease(l)?;
        }
        if let Some(a) = self.active {
            self.validate_active(a)?;
            self.validate_retained_lease(&a.lease)?;
        }
        for (slot, entry) in self.results.iter().enumerate() {
            if let Some(r) = entry {
                validate_derivation(r.derivation, self.epoch, r.published_at)?;
                validate_lease(&r.lease, self.epoch, self.seen, self.next_result)?;
                self.validate_operand_values(r.derivation)?;
                if (r.derivation.id & 7) as usize != slot
                    || r.derivation.id < self.next_result.saturating_sub(8)
                    || r.published_at >= self.seen
                    || r.turn > self.turn
                    || r.lease.reference
                        != (Reference::Result {
                            epoch: self.epoch,
                            id: r.derivation.id,
                        })
                    || encode_i64(r.derivation.value) != (r.lease.bytes, r.lease.len)
                {
                    return Err(ObjectError::InvalidSnapshot);
                }
            }
        }
        for id in self.next_result.saturating_sub(8).max(1)..self.next_result {
            self.result(self.epoch, id)?;
        }
        if let Some(o) = self.pending {
            if self.awaiting_key.is_some()
                || o.id != self.next_offer
                || o.epoch != self.epoch
                || o.frontier != self.seen
                || o.turn != self.turn
                || self.prospective(o.action, o.operands)? != o.active
            {
                return Err(ObjectError::InvalidSnapshot);
            }
            for l in o.operands.iter().flatten() {
                self.validate_retained_lease(l)?;
            }
            if let Some(a) = o.active {
                self.validate_active(a)?;
                self.validate_retained_lease(&a.lease)?;
            }
        }
        if let Some(key) = self.awaiting_key {
            if let Some(d) = key.ready {
                let a = self.active.ok_or(ObjectError::InvalidSnapshot)?;
                if a.provisional != Some(d)
                    || !a.acknowledged
                    || a.cursor + 1 != a.lease.len
                    || a.byte() != key.byte
                {
                    return Err(ObjectError::InvalidSnapshot);
                }
            } else if self.active.is_some_and(|a| {
                a.provisional.is_some() && a.acknowledged && a.cursor + 1 == a.lease.len
            }) {
                return Err(ObjectError::InvalidSnapshot);
            }
        } else if self.active.is_some_and(|a| {
            a.provisional.is_some() && a.acknowledged && a.cursor + 1 == a.lease.len
        }) {
            return Err(ObjectError::InvalidSnapshot);
        }
        Ok(())
    }
    fn validate_operand_values(&self, d: Derivation) -> Result<()> {
        for (reference, value) in d.operands.into_iter().zip(d.operand_values) {
            let retained = match reference {
                Reference::Occurrence {
                    epoch, start, end, ..
                } => {
                    if start >= self.seen.saturating_sub(256) && end <= self.seen {
                        Some(self.acquire_occurrence(epoch, start, (end - start) as u8)?)
                    } else {
                        None
                    }
                }
                Reference::Result { epoch, id } => self.acquire_result(epoch, id).ok(),
            };
            if let Some(lease) = retained {
                if lease.reference() != reference || decode_i64(lease.payload())? != value {
                    return Err(ObjectError::InvalidSnapshot);
                }
            }
        }
        Ok(())
    }
    fn validate_retained_lease(&self, l: &Lease) -> Result<()> {
        match l.reference {
            Reference::Occurrence {
                epoch,
                turn,
                start,
                end,
            } => {
                if turn > self.turn {
                    return Err(ObjectError::InvalidSnapshot);
                }
                for sequence in start..end {
                    if let Ok(r) = self.occurrence(epoch, sequence) {
                        if r.turn != turn || r.byte != l.bytes[(sequence - start) as usize] {
                            return Err(ObjectError::InvalidSnapshot);
                        }
                        if sequence == start && (r.keys != l.keys || r.roots != l.roots) {
                            return Err(ObjectError::InvalidSnapshot);
                        }
                    }
                }
            }
            Reference::Result { epoch, id } => {
                if let Ok(r) = self.result(epoch, id) {
                    if r.lease != *l {
                        return Err(ObjectError::InvalidSnapshot);
                    }
                }
            }
        }
        Ok(())
    }
    pub fn snapshot(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut e = Encoder(Vec::new());
        e.0.extend_from_slice(b"UORAO001");
        e.0.extend_from_slice(&self.model);
        e.0.extend_from_slice(&self.geometry);
        for x in [
            self.epoch,
            self.turn,
            self.seen,
            self.next_offer,
            self.next_result,
        ] {
            e.u64(x)
        }
        e.byte(self.publications);
        for entry in self.occurrences {
            e.option(entry, |e, r| {
                for x in [r.epoch, r.sequence, r.turn] {
                    e.u64(x)
                }
                e.byte(r.byte);
                e.byte(match r.origin {
                    Origin::Input => 0,
                    Origin::Generated => 1,
                });
                e.geometry(r.keys, r.roots)
            });
        }
        for l in self.operands {
            e.option(l, Encoder::lease)
        }
        e.option(self.active, Encoder::active);
        for r in self.results {
            e.option(r, |e, r| {
                e.lease(r.lease);
                e.derivation(r.derivation);
                e.u64(r.published_at);
                e.u64(r.turn)
            });
        }
        e.option(self.pending, Encoder::offer);
        e.option(self.awaiting_key, |e, k| {
            e.byte(k.byte);
            e.option(k.ready, Encoder::derivation)
        });
        if e.0.len() + 36 > SNAPSHOT_MAX {
            return Err(ObjectError::SnapshotLimit);
        }
        let len = (e.0.len() + 36) as u32;
        e.0.extend_from_slice(&len.to_le_bytes());
        let checksum = *blake3::hash(&e.0).as_bytes();
        e.0.extend_from_slice(&checksum);
        Ok(e.0)
    }
    pub fn restore(bytes: &[u8], model: [u8; 32], geometry: [u8; 32]) -> Result<Self> {
        if bytes.len() > SNAPSHOT_MAX {
            return Err(ObjectError::SnapshotLimit);
        }
        let body_len = bytes
            .len()
            .checked_sub(32)
            .ok_or(ObjectError::InvalidSnapshot)?;
        if blake3::hash(&bytes[..body_len]).as_bytes() != &bytes[body_len..] {
            return Err(ObjectError::InvalidSnapshot);
        }
        let mut d = Decoder {
            bytes: &bytes[..body_len],
            at: 0,
        };
        if d.take::<8>()? != *b"UORAO001" {
            return Err(ObjectError::InvalidSnapshot);
        }
        if d.take::<32>()? != model || d.take::<32>()? != geometry {
            return Err(ObjectError::SnapshotBinding);
        }
        let epoch = d.u64()?;
        let mut s = Self::new(model, geometry, epoch);
        s.turn = d.u64()?;
        s.seen = d.u64()?;
        s.next_offer = d.u64()?;
        s.next_result = d.u64()?;
        s.publications = d.byte()?;
        for slot in &mut s.occurrences {
            *slot = d.option(|d| {
                let epoch = d.u64()?;
                let sequence = d.u64()?;
                let turn = d.u64()?;
                let byte = d.byte()?;
                let origin = match d.byte()? {
                    0 => Origin::Input,
                    1 => Origin::Generated,
                    _ => return Err(ObjectError::InvalidSnapshot),
                };
                let (keys, roots) = d.geometry()?;
                Ok(Occurrence {
                    epoch,
                    sequence,
                    turn,
                    byte,
                    origin,
                    keys,
                    roots,
                })
            })?;
        }
        for slot in &mut s.operands {
            *slot = d.option(Decoder::lease)?;
        }
        s.active = d.option(Decoder::active)?;
        for slot in &mut s.results {
            *slot = d.option(|d| {
                Ok(ResultRecord {
                    lease: d.lease()?,
                    derivation: d.derivation()?,
                    published_at: d.u64()?,
                    turn: d.u64()?,
                })
            })?;
        }
        s.pending = d.option(Decoder::offer)?;
        s.awaiting_key = d.option(|d| {
            Ok(AwaitingKey {
                byte: d.byte()?,
                ready: d.option(Decoder::derivation)?,
            })
        })?;
        let total = u32::from_le_bytes(d.take()?);
        if total as usize != bytes.len() || d.at != body_len {
            return Err(ObjectError::InvalidSnapshot);
        }
        s.validate()?;
        Ok(s)
    }
}

#[cfg(test)]
#[path = "objects_tests.rs"]
mod tests;
