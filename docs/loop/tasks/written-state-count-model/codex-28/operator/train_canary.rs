//! Independent planted byte oracle for the D28 CUDA fitting front end.
use std::{error::Error, fs, path::Path};

const FIT_MAGIC: u32 = 0x38324644;
const RAW_MAGIC: u32 = 0x31425355;
const CAP: usize = 256;
const NO_RADIUS: u32 = 65535;

#[derive(Clone, Copy)]
struct Example {
    row: u32,
    source: u32,
    fence: u32,
    connector_depth: u32,
    label: u32,
    roots: u32,
}

impl Example {
    fn shape(self) -> u32 {
        self.connector_depth | (1 << 8)
    }

    fn signature(self) -> u32 {
        match self.connector_depth {
            1 => 0b01_01,
            2 => 0b01_01_01,
            3 => 0b01_01_01_01,
            _ => unreachable!(),
        }
    }

    fn input(self) -> [u32; 13] {
        [
            self.row,
            self.source,
            self.source + 1,
            self.fence,
            (self.row - 28000) / 4,
            self.shape(),
            self.signature(),
            self.label,
            self.roots,
            0,
            0,
            0,
            0,
        ]
    }

    fn raw(self, radius: u32) -> [u32; 8] {
        [
            self.shape() | (self.label << 16),
            self.signature(),
            radius,
            self.roots,
            0,
            0,
            0,
            0,
        ]
    }
}

fn write(path: &Path, words: impl IntoIterator<Item = u32>) -> std::io::Result<()> {
    let bytes: Vec<u8> = words.into_iter().flat_map(u32::to_le_bytes).collect();
    fs::write(path, bytes)
}

fn fit_input(path: &Path, examples: &[Example]) -> std::io::Result<()> {
    write(
        path,
        [FIT_MAGIC, examples.len() as u32, 13, 0]
            .into_iter()
            .chain(examples.iter().flat_map(|example| example.input())),
    )
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let [output] = args.as_slice() else {
        return Err("usage: d28-train-canary NEW_DIRECTORY".into());
    };
    let output = Path::new(output);
    fs::create_dir(output)?;

    let examples = [
        Example {
            row: 28000,
            source: 6,
            fence: 11,
            connector_depth: 1,
            label: 0,
            roots: 0,
        },
        Example {
            row: 28001,
            source: 7,
            fence: 12,
            connector_depth: 1,
            label: 0,
            roots: 2,
        },
        Example {
            row: 28004,
            source: 8,
            fence: 13,
            connector_depth: 1,
            label: 0,
            roots: 1,
        },
        Example {
            row: 28005,
            source: 8,
            fence: 13,
            connector_depth: 1,
            label: 1,
            roots: 0x0101,
        },
        Example {
            row: 28008,
            source: 9,
            fence: 14,
            connector_depth: 2,
            label: 0,
            roots: 0,
        },
        Example {
            row: 28009,
            source: 9,
            fence: 14,
            connector_depth: 2,
            label: 0,
            roots: 2,
        },
        Example {
            row: 28012,
            source: 9,
            fence: 14,
            connector_depth: 2,
            label: 1,
            roots: 1,
        },
        Example {
            row: 28013,
            source: 13,
            fence: 15,
            connector_depth: 3,
            label: 1,
            roots: 0,
        },
    ];

    // D11 roots 0, 1 and 2 have first two coordinates (2,2), (-2,-2)
    // and (-2,2). Cross-family minimum L1 distances are 8, 4, 4 for
    // the first signature's three positives, so its radius is 8.
    // The second has positive support from only one family. The third
    // has no positives. Neither gets a finite radius.
    let radius = [8, 8, 8, 8, NO_RADIUS, NO_RADIUS, NO_RADIUS, NO_RADIUS];
    fit_input(&output.join("canary.fit"), &examples)?;
    let mut oracle = vec![RAW_MAGIC, 0, examples.len() as u32, CAP as u32];
    oracle.extend(
        examples
            .iter()
            .zip(radius)
            .flat_map(|(example, radius)| example.raw(radius)),
    );
    oracle.resize(4 + CAP * 8, 0);
    write(&output.join("canary.oracle.raw"), oracle)?;

    let mut collision = examples;
    collision[3].roots = examples[0].roots;
    fit_input(&output.join("inverse-label-collision.fit"), &collision)?;

    let mut over_cap = examples;
    over_cap[7].source = over_cap[7].fence - 8;
    fit_input(&output.join("eight-piece-negative.fit"), &over_cap)?;
    Ok(())
}
