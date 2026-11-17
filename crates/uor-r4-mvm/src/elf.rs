//! Checked loadable segments for the software guest. Admission and loading use the same parser.
use crate::spiralcore_vsock::r4_spiralcore_vsock::{
    admit_rv64_elf, Elf64HeaderSummary, ElfAdmission,
};

pub(super) struct Image<'a> {
    pub entry: u64,
    pub segments: Vec<Segment<'a>>,
}

pub(super) struct Segment<'a> {
    pub address: u64,
    pub end: u64,
    pub data: &'a [u8],
}

const BAD_SEGMENT: ElfAdmission = ElfAdmission::RefusedSegmentOverflow;
const MAX_SEGMENTS: usize = 16;

fn word64(bytes: &[u8]) -> Result<u64, ElfAdmission> {
    Ok(u64::from_le_bytes(
        bytes.try_into().map_err(|_| BAD_SEGMENT)?,
    ))
}

fn table(
    bytes: &[u8],
    offset: u64,
    count: u16,
    stride: u16,
) -> Result<std::slice::ChunksExact<'_, u8>, ElfAdmission> {
    if stride < 56 || offset < 64 {
        return Err(BAD_SEGMENT);
    }
    let start = usize::try_from(offset).map_err(|_| BAD_SEGMENT)?;
    let len = usize::from(count)
        .checked_mul(usize::from(stride))
        .ok_or(BAD_SEGMENT)?;
    let end = start.checked_add(len).ok_or(BAD_SEGMENT)?;
    let bytes = bytes.get(start..end).ok_or(BAD_SEGMENT)?;
    Ok(bytes.chunks_exact(usize::from(stride)))
}

pub(super) fn parse(bytes: &[u8], ram_bytes: usize) -> Result<Image<'_>, ElfAdmission> {
    let header = bytes
        .first_chunk::<64>()
        .ok_or(ElfAdmission::RefusedBadMagic)?;
    let entry = word64(&header[24..32])?;
    let mut summary = Elf64HeaderSummary {
        magic_valid: &header[..4] == b"\x7fELF",
        elf_class64: header[4] == 2,
        little_endian: header[5] == 1,
        machine_code: u64::from(u16::from_le_bytes([header[18], header[19]])),
        entry_aligned: entry >= 0x1000 && entry < ram_bytes as u64 && entry.is_multiple_of(2),
        segment_count: 0,
        max_segments: MAX_SEGMENTS as u64,
    };
    let mut segments = Vec::new();
    if summary.magic_valid
        && summary.elf_class64
        && summary.little_endian
        && summary.machine_code == 243
    {
        let headers = table(
            bytes,
            word64(&header[32..40])?,
            u16::from_le_bytes([header[56], header[57]]),
            u16::from_le_bytes([header[54], header[55]]),
        )?;
        for ph in headers {
            if ph[..4] != 1_u32.to_le_bytes() {
                continue;
            }
            let offset = usize::try_from(word64(&ph[8..16])?).map_err(|_| BAD_SEGMENT)?;
            let address = word64(&ph[16..24])?;
            let file_size = word64(&ph[32..40])?;
            let memory_size = word64(&ph[40..48])?;
            let end = address.checked_add(memory_size).ok_or(BAD_SEGMENT)?;
            if file_size > memory_size || end > ram_bytes as u64 || segments.len() == MAX_SEGMENTS {
                return Err(BAD_SEGMENT);
            }
            let len = usize::try_from(file_size).map_err(|_| BAD_SEGMENT)?;
            let file_end = offset.checked_add(len).ok_or(BAD_SEGMENT)?;
            let data = bytes.get(offset..file_end).ok_or(BAD_SEGMENT)?;
            segments.push(Segment { address, end, data });
        }
        summary.segment_count = segments.len() as u64;
    }
    let verdict = admit_rv64_elf(summary);
    match verdict {
        ElfAdmission::ElfAdmitted => Ok(Image { entry, segments }),
        refusal => Err(refusal),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_requires_every_complete_entry() {
        let bytes = [0_u8; 176];
        assert_eq!(table(&bytes, 64, 2, 56).unwrap().len(), 2);
        assert!(matches!(
            table(&bytes[..175], 64, 2, 56),
            Err(ElfAdmission::RefusedSegmentOverflow)
        ));
        assert!(matches!(
            table(&bytes, u64::MAX, 1, 56),
            Err(ElfAdmission::RefusedSegmentOverflow)
        ));
    }
}
