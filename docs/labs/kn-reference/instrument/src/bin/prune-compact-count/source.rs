//! Bounded reader for the fixed-width UORKNC02 sections. The ordinary native
//! reader independently validates the emitted artifact during cold replay.

use super::{follower_width, key_width, Fail, Stats, HEADER, N, ORDERS, ORDER_HEADER};
use sha2::{Digest, Sha256};
use std::fs::{File, Metadata};
use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use uor_r4_discrete::ngram_count::NGRAM_VOCABULARY;

const BUFFER: usize = 1 << 20;
const MAX_ROWS: usize = NGRAM_VOCABULARY as usize - 1;

struct Section {
    header: [u8; ORDER_HEADER as usize],
    key_start: u64,
    follower_start: u64,
    contexts: u32,
    followers: u32,
}

pub struct Source {
    pub header: [u8; HEADER as usize],
    pub bytes: u64,
    sections: Vec<Section>,
    metadata: Metadata,
    keys: BufReader<File>,
    rows: BufReader<File>,
}

fn le_u32(bytes: &[u8]) -> Result<u32, Fail> {
    Ok(u32::from_le_bytes(bytes.try_into()?))
}

fn same_version(a: &Metadata, b: &Metadata) -> Result<bool, Fail> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if (a.dev(), a.ino(), a.ctime(), a.ctime_nsec())
            != (b.dev(), b.ino(), b.ctime(), b.ctime_nsec())
        {
            return Ok(false);
        }
    }
    Ok(a.is_file() && b.is_file() && a.len() == b.len() && a.modified()? == b.modified()?)
}

impl Source {
    pub fn open(path: &Path, source_pin: [u8; 32], training_pin: [u8; 32]) -> Result<Self, Fail> {
        let mut keys = BufReader::with_capacity(BUFFER, File::open(path)?);
        let rows = BufReader::with_capacity(BUFFER, File::open(path)?);
        let metadata = keys.get_ref().metadata()?;
        if !same_version(&metadata, &rows.get_ref().metadata()?)? {
            return Err("compact source descriptors require the same regular file".into());
        }
        let mut digest = Sha256::new();
        let bytes = io::copy(&mut keys, &mut digest)?;
        if bytes != metadata.len() || digest.finalize()[..] != source_pin {
            return Err("compact count source digest mismatch".into());
        }
        keys.seek(SeekFrom::Start(0))?;
        let mut header = [0; HEADER as usize];
        keys.read_exact(&mut header)?;
        if &header[..8] != b"UORKNC02"
            || le_u32(&header[8..12])? != N
            || header[12..] != training_pin
        {
            return Err("compact count format, order, or TRAIN digest mismatch".into());
        }
        let mut sections = Vec::with_capacity(ORDERS);
        let mut start = HEADER;
        for order in 0..=N {
            let mut header = [0; ORDER_HEADER as usize];
            keys.seek(SeekFrom::Start(start))?;
            keys.read_exact(&mut header)?;
            let followers = le_u32(&header[..4])?;
            let contexts = le_u32(&header[4..8])?;
            let key_start = start + ORDER_HEADER;
            let follower_start = key_start + u64::from(contexts) * (key_width(order) as u64 + 4);
            start = follower_start + u64::from(followers) * follower_width(order, N);
            if start > bytes || (contexts == 0) != (followers == 0) {
                return Err("invalid compact count section length".into());
            }
            sections.push(Section {
                header,
                key_start,
                follower_start,
                contexts,
                followers,
            });
        }
        if start != bytes {
            return Err("trailing compact count source bytes".into());
        }
        Ok(Self {
            header,
            bytes,
            sections,
            metadata,
            keys,
            rows,
        })
    }

    pub fn check_unchanged(&self) -> Result<(), Fail> {
        for reader in [&self.keys, &self.rows] {
            if !same_version(&self.metadata, &reader.get_ref().metadata()?)? {
                return Err("compact source was modified during materialization".into());
            }
        }
        Ok(())
    }

    pub fn visit_groups(
        &mut self,
        mut visit: impl FnMut(u32, &[u8], Stats, &[u8]) -> Result<(), Fail>,
    ) -> Result<(), Fail> {
        let mut followers = vec![0u8; MAX_ROWS * 10];
        for (order, section) in (0..=N).zip(&self.sections) {
            self.keys.seek(SeekFrom::Start(section.key_start))?;
            self.rows.seek(SeekFrom::Start(section.follower_start))?;
            let width = key_width(order);
            let stride = width + 4;
            let follower_stride = follower_width(order, N) as usize;
            let mut row = [0; 19];
            let mut next_row = [0; 19];
            let mut start = 0;
            let mut expected = Stats::default();
            if section.contexts > 0 {
                self.keys.read_exact(&mut row[..stride])?;
                if le_u32(&row[width..stride])? != 0 {
                    return Err("compact source does not start at follower zero".into());
                }
            }
            for i in 0..section.contexts {
                if order % 2 == 1 && row[0] > 0x0f {
                    return Err("compact source context key has nonzero padding".into());
                }
                let end = if i + 1 == section.contexts {
                    section.followers
                } else {
                    self.keys.read_exact(&mut next_row[..stride])?;
                    if row[..width] >= next_row[..width] {
                        return Err("compact source context keys are not strictly ordered".into());
                    }
                    le_u32(&next_row[width..stride])?
                };
                let count = end
                    .checked_sub(start)
                    .ok_or("compact source follower offsets regress")?
                    as usize;
                if count == 0 || count > MAX_ROWS || end > section.followers {
                    return Err("invalid compact source follower span".into());
                }
                let follower_rows = &mut followers[..count * follower_stride];
                self.rows.read_exact(follower_rows)?;
                let mut stats = Stats {
                    contexts: 1,
                    ..Stats::default()
                };
                let mut previous = 0;
                for follower in follower_rows.chunks_exact(follower_stride) {
                    let next = u16::from_le_bytes([follower[0], follower[1]]);
                    let raw = le_u32(&follower[2..6])?;
                    let adjusted = if order == N {
                        raw
                    } else {
                        le_u32(&follower[6..10])?
                    };
                    if next <= previous
                        || u32::from(next) >= NGRAM_VOCABULARY
                        || raw == 0
                        || adjusted == 0
                    {
                        return Err("invalid or unordered compact source follower".into());
                    }
                    previous = next;
                    stats.observe(raw, adjusted);
                }
                expected.add(stats);
                visit(order, &row[..width], stats, follower_rows)?;
                row = next_row;
                start = end;
            }
            for (slot, word) in expected
                .spectra
                .into_iter()
                .flatten()
                .zip(section.header[8..].chunks_exact(8))
            {
                if slot != u64::from_le_bytes(word.try_into()?) {
                    return Err("compact source spectrum does not match followers".into());
                }
            }
        }
        self.check_unchanged()
    }
}
