use uor_r4_vision::{decompose_rgba, parse_canonical, Grid, VisionError};

const FIXTURE_RGBA: [u8; 16] = [
    255, 0, 0, 255, // red
    0, 255, 0, 128, // green
    0, 0, 255, 64, // blue
    255, 255, 255, 0, // white
];

fn fixture_canonical(grid: Grid, patches: &[[u8; 40]]) -> Vec<u8> {
    let rgba = FIXTURE_RGBA;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"R4IMG001");
    bytes.extend_from_slice(&2u16.to_be_bytes());
    bytes.extend_from_slice(&2u16.to_be_bytes());
    bytes.extend_from_slice(&grid.columns().to_be_bytes());
    bytes.extend_from_slice(&grid.rows().to_be_bytes());
    bytes.extend_from_slice(&(patches.len() as u32).to_be_bytes());
    bytes.extend_from_slice(blake3::hash(&rgba).as_bytes());
    for patch in patches {
        bytes.extend_from_slice(patch);
    }
    bytes
}

fn encoded_patch(
    bounds: [u16; 4],
    mean_rgb: [u16; 3],
    luma: u16,
    chroma: [i16; 2],
    edges: [i16; 2],
    r4: [[i16; 4]; 2],
) -> [u8; 40] {
    let mut bytes = [0; 40];
    let mut at = 0;
    for value in bounds {
        bytes[at..at + 2].copy_from_slice(&value.to_be_bytes());
        at += 2;
    }
    for value in mean_rgb.into_iter().chain([luma]) {
        bytes[at..at + 2].copy_from_slice(&value.to_be_bytes());
        at += 2;
    }
    for value in chroma.into_iter().chain(edges).chain(r4[0]).chain(r4[1]) {
        bytes[at..at + 2].copy_from_slice(&value.to_be_bytes());
        at += 2;
    }
    assert_eq!(at, bytes.len());
    bytes
}

#[test]
fn pixel_grid_has_hand_computed_q12_features() {
    let out = decompose_rgba(2, 2, &FIXTURE_RGBA, Grid::new(2, 2)).unwrap();
    assert_eq!(out.width(), 2);
    assert_eq!(out.height(), 2);
    assert_eq!(out.patches().len(), 4);

    let expected = [
        (
            [0, 0, 1, 1],
            [4096, 0, 0],
            1225,
            [2871, -1225],
            [[2048, -2048, -2048, -823], [2871, -1225, 0, 0]],
        ),
        (
            [1, 0, 2, 1],
            [0, 4096, 0],
            2404,
            [-2404, -2404],
            [[-2048, 2048, -2048, 356], [-2404, -2404, 0, 0]],
        ),
        (
            [0, 1, 1, 2],
            [0, 0, 4096],
            467,
            [-467, 3629],
            [[-2048, -2048, 2048, -1581], [-467, 3629, 0, 0]],
        ),
        (
            [1, 1, 2, 2],
            [4096, 4096, 4096],
            4096,
            [0, 0],
            [[2048, 2048, 2048, 2048], [0, 0, 0, 0]],
        ),
    ];

    for (patch, (bounds, mean_rgb, luma, chroma, r4)) in out.patches().iter().zip(expected) {
        assert_eq!([patch.x0, patch.y0, patch.x1, patch.y1], bounds);
        assert_eq!(patch.mean_rgb_q12, mean_rgb);
        assert_eq!(patch.luma_q12, luma);
        assert_eq!(patch.chroma_q12, chroma);
        assert_eq!(patch.edges_q12, [0, 0]);
        assert_eq!(patch.r4_q12, r4);
    }
}

#[test]
fn one_patch_has_hand_computed_means_and_signed_edges() {
    let out = decompose_rgba(2, 2, &FIXTURE_RGBA, Grid::new(1, 1)).unwrap();
    let patch = &out.patches()[0];
    assert_eq!([patch.x0, patch.y0, patch.x1, patch.y1], [0, 0, 2, 2]);
    assert_eq!(patch.mean_rgb_q12, [2048, 2048, 2048]);
    assert_eq!(patch.luma_q12, 2048);
    assert_eq!(patch.chroma_q12, [0, 0]);
    assert_eq!(patch.edges_q12, [2401, 474]);
    assert_eq!(patch.r4_q12, [[0, 0, 0, 0], [0, 0, 2401, 474]]);
}

#[test]
fn canonical_bytes_are_pinned_in_grid_order() {
    let grid = Grid::new(2, 2);
    let out = decompose_rgba(2, 2, &FIXTURE_RGBA, grid).unwrap();
    let encoded = [
        encoded_patch(
            [0, 0, 1, 1],
            [4096, 0, 0],
            1225,
            [2871, -1225],
            [0, 0],
            [[2048, -2048, -2048, -823], [2871, -1225, 0, 0]],
        ),
        encoded_patch(
            [1, 0, 2, 1],
            [0, 4096, 0],
            2404,
            [-2404, -2404],
            [0, 0],
            [[-2048, 2048, -2048, 356], [-2404, -2404, 0, 0]],
        ),
        encoded_patch(
            [0, 1, 1, 2],
            [0, 0, 4096],
            467,
            [-467, 3629],
            [0, 0],
            [[-2048, -2048, 2048, -1581], [-467, 3629, 0, 0]],
        ),
        encoded_patch(
            [1, 1, 2, 2],
            [4096, 4096, 4096],
            4096,
            [0, 0],
            [0, 0],
            [[2048, 2048, 2048, 2048], [0, 0, 0, 0]],
        ),
    ];
    assert_eq!(out.canonical_bytes(), fixture_canonical(grid, &encoded));
    assert_eq!(out.canonical_bytes(), out.canonical_bytes());
}

#[test]
fn one_pixel_and_alpha_changes_are_source_bound() {
    let original = decompose_rgba(2, 2, &FIXTURE_RGBA, Grid::new(2, 2)).unwrap();

    let mut changed = FIXTURE_RGBA;
    changed[0] = 254;
    let changed_out = decompose_rgba(2, 2, &changed, Grid::new(2, 2)).unwrap();
    assert_ne!(changed_out.source_hash(), original.source_hash());
    assert_ne!(changed_out.canonical_bytes(), original.canonical_bytes());
    assert_ne!(changed_out.patches()[0].mean_rgb_q12, [4096, 0, 0]);
    assert_eq!(changed_out.patches()[1], original.patches()[1]);

    let mut alpha_only = FIXTURE_RGBA;
    alpha_only[3] ^= 0xff;
    let alpha_out = decompose_rgba(2, 2, &alpha_only, Grid::new(2, 2)).unwrap();
    assert_ne!(alpha_out.source_hash(), original.source_hash());
    assert_ne!(alpha_out.canonical_bytes(), original.canonical_bytes());
    assert_eq!(alpha_out.patches(), original.patches());
}

#[test]
fn validation_reports_each_failure_without_reading_out_of_bounds() {
    assert!(matches!(
        decompose_rgba(0, 1, &[], Grid::new(1, 1)),
        Err(VisionError::DimensionOutOfRange { .. })
    ));
    assert!(matches!(
        decompose_rgba(257, 1, &[], Grid::new(1, 1)),
        Err(VisionError::DimensionOutOfRange { .. })
    ));
    assert!(matches!(
        decompose_rgba(2, 2, &FIXTURE_RGBA[..15], Grid::new(1, 1)),
        Err(VisionError::PixelBufferLength {
            expected: 16,
            actual: 15
        })
    ));
    assert!(matches!(
        decompose_rgba(2, 2, &FIXTURE_RGBA, Grid::new(0, 1)),
        Err(VisionError::EmptyGrid)
    ));
    assert!(matches!(
        decompose_rgba(2, 2, &FIXTURE_RGBA, Grid::new(33, 1)),
        Err(VisionError::GridOutOfRange { .. })
    ));
    assert!(matches!(
        decompose_rgba(2, 2, &FIXTURE_RGBA, Grid::new(3, 2)),
        Err(VisionError::GridLargerThanImage { .. })
    ));
}

#[test]
fn every_pixel_belongs_to_exactly_one_positioned_patch() {
    let rgba: Vec<u8> = (0..7 * 5 * 4).map(|value| value as u8).collect();
    let untouched = rgba.clone();
    let out = decompose_rgba(7, 5, &rgba, Grid::new(3, 2)).unwrap();
    let mut covered = [[0u8; 7]; 5];
    for patch in out.patches() {
        assert!(patch.x0 < patch.x1 && patch.y0 < patch.y1);
        for y in patch.y0..patch.y1 {
            for x in patch.x0..patch.x1 {
                covered[usize::from(y)][usize::from(x)] += 1;
            }
        }
    }
    assert!(covered.iter().flatten().all(|&count| count == 1));
    assert_eq!(rgba, untouched);
    assert_eq!(out.source_hash(), *blake3::hash(&rgba).as_bytes());
    assert_eq!(
        out.source_hash_hex(),
        blake3::hash(&rgba).to_hex().to_string()
    );
}

#[test]
fn canonical_bytes_parse_back_to_the_same_decomposition() {
    let out = decompose_rgba(2, 2, &FIXTURE_RGBA, Grid::new(2, 2)).unwrap();
    let parsed = parse_canonical(&out.canonical_bytes()).unwrap();
    assert_eq!(parsed, out);
    assert_eq!(parsed.canonical_bytes(), out.canonical_bytes());
}

#[test]
fn canonical_parser_rejects_noncanonical_or_inconsistent_bytes() {
    let out = decompose_rgba(2, 2, &FIXTURE_RGBA, Grid::new(2, 2)).unwrap();
    let canonical = out.canonical_bytes();

    let mut bad = canonical.clone();
    bad[0] ^= 0xff;
    assert!(matches!(
        parse_canonical(&bad),
        Err(VisionError::InvalidCanonical(_))
    ));

    let mut bad = canonical.clone();
    bad.truncate(bad.len() - 1);
    assert!(matches!(
        parse_canonical(&bad),
        Err(VisionError::InvalidCanonical(_))
    ));

    let mut bad = canonical.clone();
    bad[16..20].copy_from_slice(&3u32.to_be_bytes());
    assert!(matches!(
        parse_canonical(&bad),
        Err(VisionError::InvalidCanonical(_))
    ));

    let mut bad = canonical.clone();
    bad[52..54].copy_from_slice(&1u16.to_be_bytes());
    assert!(matches!(
        parse_canonical(&bad),
        Err(VisionError::InvalidCanonical(_))
    ));

    let mut bad = canonical.clone();
    bad[52 + 8..52 + 10].copy_from_slice(&4097u16.to_be_bytes());
    assert!(matches!(
        parse_canonical(&bad),
        Err(VisionError::InvalidCanonical(_))
    ));

    let mut bad = canonical;
    let redundant_r4 = 52 + 26;
    bad[redundant_r4..redundant_r4 + 2].copy_from_slice(&0i16.to_be_bytes());
    assert!(matches!(
        parse_canonical(&bad),
        Err(VisionError::InvalidCanonical(_))
    ));
}
