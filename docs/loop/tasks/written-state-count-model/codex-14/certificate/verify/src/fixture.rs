//! Independent scoring and directional arithmetic for bounded synthetic tape only.
use super::*;
use std::io::Read;

#[allow(dead_code)]
#[path = "../../../../../../../../crates/uor-r4-integer/src/lawful_likelihood.rs"]
mod likelihood;

pub(super) use likelihood::bits_q12;

const MAX_ROWS: u64 = 65_536;
const ROW_BYTES: usize = 109;
type Lanes = [u64; LANES];

struct Row {
    bucket: usize,
    masses: Lanes,
}

pub(super) struct Tape(Vec<Row>);

impl Tape {
    pub(super) fn read(directory: &Path, manifest: &Manifest) -> Result<Self> {
        require(
            manifest.synthetic_fixture && manifest.target_count <= MAX_ROWS,
            "finite tape requires a sealed synthetic manifest and at most 65,536 targets",
        )?;
        let path = directory.join("frame.bin");
        let length = manifest.target_count * ROW_BYTES as u64;
        require(
            fs::metadata(&path)?.len() == length,
            "finite tape exact length",
        )?;
        let mut bytes = Vec::with_capacity(length as usize);
        File::open(path)?.take(length + 1).read_to_end(&mut bytes)?;
        require(
            bytes.len() as u64 == length && digest(&bytes) == manifest.frame_sha256,
            "finite tape digest or extent",
        )?;

        let mut actual_kn = Sha256::new();
        let mut counts = [0u64; BUCKETS];
        let mut tape = Vec::with_capacity(manifest.target_count as usize);
        for packed in bytes.chunks_exact(ROW_BYTES) {
            let bucket = usize::from(u16::from_le_bytes(packed[..2].try_into()?));
            require(
                bucket < BUCKETS && packed[2] <= 1 && packed[4] <= 1,
                "finite tape bucket or annotation domain",
            )?;
            counts[bucket] += 1;
            let mut masses = [0; LANES];
            for (lane, word) in packed[5..].chunks_exact(8).enumerate() {
                let mass = u64::from_le_bytes(word.try_into()?);
                require(
                    u128::from(mass) <= M && (mask(bucket) & (1 << lane) != 0 || mass == 0),
                    "finite tape mass or absent lane",
                )?;
                masses[lane] = mass;
            }
            require(
                masses[0] > 0 && u128::from(masses[LANES - 1]) == M / 4096,
                "finite tape KN or uniform mass",
            )?;
            actual_kn.update(masses[0].to_le_bytes());
            tape.push(Row { bucket, masses });
        }
        require(
            counts.as_slice() == manifest.bucket_targets
                && format!("{:x}", actual_kn.finalize()) == manifest.kn_mass_sha256,
            "finite tape bucket census or KN mass digest",
        )?;
        Ok(Self(tape))
    }

    pub(super) fn rowmax(&self) -> Result<u64> {
        self.0.iter().try_fold(0, |total, row| {
            let mass = row.masses.iter().copied().max().unwrap_or(1).max(1);
            Ok(total + bits_q12(mass)?)
        })
    }

    pub(super) fn verify_anchor(
        &self,
        point: Point,
        fit: &Value,
        buckets: &[Bucket],
    ) -> Result<()> {
        let weights = if point.mode == "kn_only" {
            let mut pure = [0; LANES];
            pure[0] = B as u64;
            vec![pure; BUCKETS]
        } else {
            let fit_buckets = fit[point.mode]["buckets"]
                .as_array()
                .ok_or("missing finite fit buckets")?;
            let field = if point.pass == 0 {
                "start_q32"
            } else {
                "weights_q32"
            };
            fit_buckets
                .iter()
                .map(|bucket| Ok(serde_json::from_value::<Lanes>(bucket[field].clone())?))
                .collect::<Result<Vec<_>>>()?
        };
        require(
            weights.len() == BUCKETS,
            "finite anchor bucket weight extent",
        )?;
        let mut direction = vec![[0u128; LANES]; BUCKETS];
        let mut score = 0;
        for row in &self.0 {
            let mut weighted_mass = 0;
            for (&mass, &weight) in row.masses.iter().zip(&weights[row.bucket]) {
                weighted_mass += u128::from(mass) * u128::from(weight);
            }
            let mass = (weighted_mass / B).max(1);
            let active = mask(row.bucket);
            for (lane, &expert) in row.masses.iter().enumerate() {
                if active & (1 << lane) != 0 {
                    // Independent quotient and remainder instead of the CUDA long-division path.
                    let numerator = B * (u128::from(expert) + 1);
                    let ceiling = numerator / mass + u128::from(numerator % mass != 0);
                    direction[row.bucket][lane] += ceiling;
                }
            }
            score += bits_q12(u64::try_from(mass)?)?;
        }
        require(
            score == point.score_q12,
            format!("finite {} anchor score mismatch", point.mode),
        )?;
        for (bucket, expected) in buckets.iter().zip(direction) {
            for (lane, reported) in bucket.sums_q32.iter().enumerate() {
                if let Some(sum) = reported {
                    require(
                        decimal(sum)? == expected[lane],
                        format!(
                            "finite {} bucket {} lane {lane} direction mismatch",
                            point.mode, bucket.bucket
                        ),
                    )?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sealed_fixture_checks_digest_and_direction_independently() {
        let path = std::env::temp_dir().join(format!(
            "uor-r4-cert-verify-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        let mut packed = [0u8; ROW_BYTES];
        let masses = [3u64, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1 << 36];
        for (bytes, &mass) in packed[5..].chunks_exact_mut(8).zip(&masses) {
            bytes.copy_from_slice(&mass.to_le_bytes());
        }
        let mut target_counts = vec![0; BUCKETS];
        target_counts[0] = 1;
        let manifest = Manifest {
            schema: String::new(),
            frame_sha256: digest(&packed),
            kn_mass_sha256: digest(&3u64.to_le_bytes()),
            source_sha256: String::new(),
            binary_sha256: String::new(),
            target_count: 1,
            per_mille: 1,
            bucket_targets: target_counts,
            bucket_masks: (0..BUCKETS).map(mask).collect(),
            inputs: Value::Null,
            synthetic_fixture: true,
        };
        let frame = path.join("frame.bin");
        fs::write(&frame, packed).unwrap();
        let tape = Tape::read(&path, &manifest).unwrap();
        let mut buckets = vec![Bucket {
            bucket: 0,
            targets: 1,
            mask: mask(0),
            max_lane: LANES - 1,
            sums_q32: std::array::from_fn(|lane| match lane {
                0 => Some("5726623062".into()),
                1 => Some("2863311531".into()),
                12 => Some("98382635061215931051".into()),
                _ => None,
            }),
        }];
        let point = Point {
            mode: "kn_only",
            pass: 0,
            score_q12: bits_q12(3).unwrap(),
        };
        assert_eq!(tape.rowmax().unwrap(), 12 * 4096);
        tape.verify_anchor(point, &Value::Null, &buckets).unwrap();
        buckets[0].sums_q32[1] = Some("2863311530".into());
        assert!(tape.verify_anchor(point, &Value::Null, &buckets).is_err());
        buckets[0].sums_q32[1] = Some("2863311531".into());
        buckets[0].sums_q32.swap(0, 1);
        assert!(tape.verify_anchor(point, &Value::Null, &buckets).is_err());
        packed[13] = 2;
        fs::write(frame, packed).unwrap();
        assert!(Tape::read(&path, &manifest).is_err());
        fs::remove_dir_all(path).unwrap();
    }
}
