//! Artifact-bound group operations in the classifier's historical signed H4 order.
//!
//! The offline core compiler supplies the bytes; this module contains no group
//! construction or learned scores. Hashing/validation occurs only at admission.
//! Numerical lookup is immutable and uses byte reads, shifts and addition.

use std::fmt;

use sha2::{Digest, Sha256};

use crate::h4_classifier::H4_ROOT_COEFFICIENTS;

pub const SCHEMA: &str = "uor-r4.historical-signed-h4-tables/1";
pub const ROOT_COUNT: usize = 120;
pub const ROW_STRIDE: usize = 128;
pub const PRODUCT_BYTES: usize = ROOT_COUNT * ROW_STRIDE;
pub const INVERSE_OFFSET: usize = PRODUCT_BYTES;
pub const MAPPING_OFFSET: usize = INVERSE_OFFSET + ROOT_COUNT;
pub const IDENTITY_OFFSET: usize = MAPPING_OFFSET + ROOT_COUNT;
pub const PAYLOAD_BYTES: usize = IDENTITY_OFFSET + 1;

/// Pending one-off generation with `export-h4-tables`. `None` admits no table.
/// Pin only the exact compiler's `mathematical_sha256`, never a caller digest.
pub const TRUSTED_MATHEMATICAL_SHA256: Option<&str> = None;

/// Fixed framing followed by 960 coefficient bytes and exactly 15,601 payload
/// bytes. Coefficients are root/coordinate/[a,b] order, signed i8 two's-complement
/// bytes denoting (a+b*phi)/2. Source provenance is outside mathematical identity.
pub const IDENTITY_FRAMING: &[u8] = b"uor-r4.historical-signed-h4-tables/1\0order=learner.embedding.historical/1\0roots=120\0stride=128\0identity=1\0relation=inverse(query)*key\0coefficients=i8-twos-complement-root-coordinate-ab;(a+b*phi)/2\0payload=product[15360]||inverse[120]||historical_to_sorted[120]||identity[1]\0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum H4TableError {
    InvalidCode(u8),
    InvalidPayloadLength(usize),
    InvalidStructure(&'static str),
    TrustAnchorNotPinned,
    CanonicalIdentityMismatch,
}

impl fmt::Display for H4TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCode(code) => write!(f, "historical H4 code {code} is outside 0..120"),
            Self::InvalidPayloadLength(length) => {
                write!(
                    f,
                    "historical H4 payload has {length} bytes, expected {PAYLOAD_BYTES}"
                )
            }
            Self::InvalidStructure(reason) => write!(f, "historical H4 payload: {reason}"),
            Self::TrustAnchorNotPinned => write!(f, "historical H4 canonical table is not pinned"),
            Self::CanonicalIdentityMismatch => {
                write!(f, "historical H4 canonical identity differs")
            }
        }
    }
}

impl std::error::Error for H4TableError {}

/// Signed historical root ID, not a sorted-root ID, palette slot or distance.
/// Private construction prevents padded columns and deserialized arbitrary u8s
/// from entering numerical lookup. Antipodes remain distinct.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct H4Code(u8);

impl TryFrom<u8> for H4Code {
    type Error = H4TableError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if usize::from(value) < ROOT_COUNT {
            Ok(Self(value))
        } else {
            Err(H4TableError::InvalidCode(value))
        }
    }
}

impl H4Code {
    pub const IDENTITY: Self = Self(1);

    pub const fn index(self) -> u8 {
        self.0
    }
}

/// One immutable, independently owned 15,601-byte payload. No steady-state
/// allocation or mutable table access is exposed; metadata/session bytes are
/// outside this payload count.
pub struct HistoricalH4Tables {
    payload: Box<[u8]>,
}

impl HistoricalH4Tables {
    /// Admit only the fixed canonical artifact. The surrounding model/bundle
    /// loader remains responsible for its complete-file seal and provenance.
    /// A self-reported artifact hash is never an admission argument here.
    pub fn from_bytes(payload: &[u8]) -> Result<Self, H4TableError> {
        validate_structure(payload)?;
        let expected = TRUSTED_MATHEMATICAL_SHA256.ok_or(H4TableError::TrustAnchorNotPinned)?;
        if mathematical_sha256(payload)? != expected {
            return Err(H4TableError::CanonicalIdentityMismatch);
        }
        Ok(Self {
            payload: payload.into(),
        })
    }

    pub const fn identity(&self) -> H4Code {
        H4Code::IDENTITY
    }

    pub fn inverse(&self, code: H4Code) -> H4Code {
        H4Code(self.payload[INVERSE_OFFSET + usize::from(code.0)])
    }

    /// Hamilton composition, left operand first, in historical order.
    pub fn compose(&self, left: H4Code, right: H4Code) -> H4Code {
        H4Code(self.payload[(usize::from(left.0) << 7) + usize::from(right.0)])
    }

    /// Directed reader relation `inverse(query) * key`. This is not symmetric.
    pub fn relative(&self, query: H4Code, key: H4Code) -> H4Code {
        self.compose(self.inverse(query), key)
    }
}

/// Digest helper for the offline exporter. Structural validity and a computed
/// digest alone do NOT admit a runtime table or establish its exact group law.
pub fn mathematical_sha256(payload: &[u8]) -> Result<String, H4TableError> {
    validate_structure(payload)?;
    let mut hash = Sha256::new();
    hash.update(IDENTITY_FRAMING);
    for root in H4_ROOT_COEFFICIENTS {
        for coordinate in root {
            hash.update(coordinate.map(|coefficient| coefficient as u8));
        }
    }
    hash.update(payload);
    Ok(hex::encode(hash.finalize()))
}

/// SHA256 of the classifier's 960 exact signed coefficient bytes in the same
/// order used by `mathematical_sha256`; loading does not construct new roots.
pub fn coefficients_sha256() -> String {
    let mut hash = Sha256::new();
    for root in H4_ROOT_COEFFICIENTS {
        for coordinate in root {
            hash.update(coordinate.map(|coefficient| coefficient as u8));
        }
    }
    hex::encode(hash.finalize())
}

fn validate_structure(payload: &[u8]) -> Result<(), H4TableError> {
    if payload.len() != PAYLOAD_BYTES {
        return Err(H4TableError::InvalidPayloadLength(payload.len()));
    }
    if payload[IDENTITY_OFFSET] != H4Code::IDENTITY.0 {
        return Err(H4TableError::InvalidStructure(
            "identity must be historical index 1",
        ));
    }
    let mut mapped = [false; ROOT_COUNT];
    for code in 0..ROOT_COUNT {
        let row = &payload[(code << 7)..((code + 1) << 7)];
        if row[..ROOT_COUNT]
            .iter()
            .any(|&value| usize::from(value) >= ROOT_COUNT)
        {
            return Err(H4TableError::InvalidStructure(
                "product code outside 0..120",
            ));
        }
        if row[ROOT_COUNT..].iter().any(|&value| value != 0) {
            return Err(H4TableError::InvalidStructure(
                "nonzero padded product column",
            ));
        }
        let inverse = usize::from(payload[INVERSE_OFFSET + code]);
        let mapped_code = usize::from(payload[MAPPING_OFFSET + code]);
        if inverse >= ROOT_COUNT || mapped_code >= ROOT_COUNT || mapped[mapped_code] {
            return Err(H4TableError::InvalidStructure(
                "inverse range or mapping bijection differs",
            ));
        }
        mapped[mapped_code] = true;
        if row[1] != code as u8 || payload[ROW_STRIDE + code] != code as u8 {
            return Err(H4TableError::InvalidStructure("two-sided identity differs"));
        }
        if row[inverse] != 1 || payload[(inverse << 7) + code] != 1 {
            return Err(H4TableError::InvalidStructure("two-sided inverse differs"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Read at test execution so the first offline compiler build needs no
    // invented fixture. Missing bytes fail the test; no conditional skip.
    fn fixture() -> std::io::Result<Vec<u8>> {
        std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/historical-h4-tables-v1.bin"
        ))
    }

    #[test]
    fn h4_tables_loaded_fixture_covers_every_historical_pair(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let fixture = fixture()?;
        let table = HistoricalH4Tables::from_bytes(&fixture)?;
        assert_eq!(table.identity().index(), 1);
        for query in 0..120u8 {
            let q = H4Code::try_from(query)?;
            assert_eq!(
                table.inverse(q).index(),
                fixture[INVERSE_OFFSET + usize::from(query)]
            );
            assert_eq!(table.relative(q, q), table.identity());
            for key in 0..120u8 {
                let k = H4Code::try_from(key)?;
                assert_eq!(
                    table.compose(q, k).index(),
                    fixture[usize::from(query) * 128 + usize::from(key)]
                );
                let inverse = usize::from(fixture[INVERSE_OFFSET + usize::from(query)]);
                assert_eq!(
                    table.relative(q, k).index(),
                    fixture[inverse * 128 + usize::from(key)]
                );
            }
        }
        Ok(())
    }

    #[test]
    fn h4_tables_direction_uses_signed_hamilton_axes() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = fixture()?;
        let table = HistoricalH4Tables::from_bytes(&fixture)?;
        // Historical axes: +i=3, +j=5, +k=7, -k=6. Independently, ij=k,
        // ji=-k, inverse(i)j=-k and inverse(j)i=k.
        let i = H4Code::try_from(3)?;
        let j = H4Code::try_from(5)?;
        assert_eq!(table.compose(i, j).index(), 7);
        assert_eq!(table.compose(j, i).index(), 6);
        assert_eq!(table.relative(i, j).index(), 6);
        assert_eq!(table.relative(j, i).index(), 7);
        assert_ne!(H4Code::try_from(0)?, table.identity());
        assert!(H4Code::try_from(119).is_ok());
        assert_eq!(H4Code::try_from(120), Err(H4TableError::InvalidCode(120)));
        assert_eq!(H4Code::try_from(255), Err(H4TableError::InvalidCode(255)));
        Ok(())
    }

    #[test]
    fn h4_tables_reject_malformed_and_self_rehashed_order() -> Result<(), Box<dyn std::error::Error>>
    {
        let fixture = fixture()?;
        HistoricalH4Tables::from_bytes(&fixture)?;
        assert!(matches!(
            HistoricalH4Tables::from_bytes(&fixture[..PAYLOAD_BYTES - 1]),
            Err(H4TableError::InvalidPayloadLength(_))
        ));
        for (offset, value) in [
            (0, 120),
            (120, 1),
            (INVERSE_OFFSET, 120),
            (IDENTITY_OFFSET, 0),
        ] {
            let mut changed = fixture.clone();
            changed[offset] = value;
            assert!(matches!(
                HistoricalH4Tables::from_bytes(&changed),
                Err(H4TableError::InvalidStructure(_))
            ));
        }
        let mut changed = fixture.clone();
        changed.swap(MAPPING_OFFSET + 3, MAPPING_OFFSET + 5);
        // A different but structurally bijective mapping can compute its own
        // digest. It cannot supply that digest to authorize canonical admission.
        assert_ne!(
            mathematical_sha256(&changed)?,
            mathematical_sha256(&fixture)?
        );
        assert!(matches!(
            HistoricalH4Tables::from_bytes(&changed),
            Err(H4TableError::CanonicalIdentityMismatch)
        ));
        Ok(())
    }
}
