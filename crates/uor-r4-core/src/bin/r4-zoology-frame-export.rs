//! Export the existing native HELM frame policy for #1059; no model/data reads.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::Serialize;
use uor_r4_core::h4_spin_frame_sidecar::H4SpinFrameSidecarV1;
use uor_r4_core::helm_d_r4_attention::{R4SpinFrameAtlas, HELM_D_R4_GAUGE_SOFTMAX_POLICY};

const USAGE: &str = "usage: r4-zoology-frame-export <NEW_DIRECTORY>";
const MAXIMUM_TOKEN_ID: u32 = 8191;

#[derive(Serialize)]
struct PrefixWitness {
    tokens: Vec<u32>,
    frame_indices: Vec<u16>,
}

#[derive(Serialize)]
struct TokenFrames {
    schema: &'static str,
    policy_identity: &'static str,
    maximum_token_id: u32,
    identity_index: u16,
    frame_artifact_cid: String,
    frame_file_cid: String,
    token_leaf_indices: Vec<u16>,
    prefix_witnesses: Vec<PrefixWitness>,
    direct_leaf_count: usize,
    witness_frame_count: usize,
    artifact_cid: String,
}

fn cid(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn build() -> Result<(Vec<u8>, Vec<u8>), String> {
    let frames = H4SpinFrameSidecarV1::build().map_err(|error| error.to_string())?;
    let frame_bytes = frames
        .canonical_bytes()
        .map_err(|error| error.to_string())?;
    H4SpinFrameSidecarV1::from_canonical_bytes(&frame_bytes).map_err(|error| error.to_string())?;

    // Use the public native atlas rather than restating its token mapping.
    let mut atlas =
        R4SpinFrameAtlas::new(MAXIMUM_TOKEN_ID, 8).map_err(|error| error.to_string())?;
    let mut token_leaf_indices = Vec::with_capacity(MAXIMUM_TOKEN_ID as usize + 1);
    for token in 0..=MAXIMUM_TOKEN_ID {
        atlas.reset();
        atlas
            .begin_position(token, 0)
            .map_err(|error| error.to_string())?;
        token_leaf_indices.push(
            atlas
                .frame_table_offset(0)
                .map_err(|error| error.to_string())?,
        );
    }

    let mut prefix_witnesses = Vec::new();
    for tokens in [
        vec![0, 1, 2, 3, 8191],
        vec![4095, 4096, 17, 0, 8190, 8191],
        vec![8191, 8191, 2, 1, 8191, 0, 3],
    ] {
        atlas.reset();
        let mut frame_indices = Vec::with_capacity(tokens.len());
        for (position, &token) in tokens.iter().enumerate() {
            atlas
                .begin_position(token, position)
                .map_err(|error| error.to_string())?;
            frame_indices.push(
                atlas
                    .frame_table_offset(position)
                    .map_err(|error| error.to_string())?,
            );
        }
        prefix_witnesses.push(PrefixWitness {
            tokens,
            frame_indices,
        });
    }
    let direct_leaf_count = token_leaf_indices
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .len();
    let witness_frame_count = prefix_witnesses
        .iter()
        .flat_map(|witness| witness.frame_indices.iter().copied())
        .collect::<BTreeSet<_>>()
        .len();
    let mut map = TokenFrames {
        schema: "uor-r4.zoology-r4-token-frames/1",
        policy_identity: HELM_D_R4_GAUGE_SOFTMAX_POLICY,
        maximum_token_id: MAXIMUM_TOKEN_ID,
        identity_index: frames.identity_index,
        frame_artifact_cid: frames.artifact_cid,
        frame_file_cid: cid(&frame_bytes),
        token_leaf_indices,
        prefix_witnesses,
        direct_leaf_count,
        witness_frame_count,
        artifact_cid: String::new(),
    };
    map.artifact_cid = cid(&serde_json::to_vec(&map).map_err(|error| error.to_string())?);
    let map_bytes = serde_json::to_vec(&map).map_err(|error| error.to_string())?;
    Ok((frame_bytes, map_bytes))
}

fn write_new(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

fn run(arguments: impl Iterator<Item = OsString>) -> Result<(), String> {
    let arguments = arguments.collect::<Vec<_>>();
    if arguments.len() == 1 && (arguments[0] == "--help" || arguments[0] == "-h") {
        println!("{USAGE}");
        return Ok(());
    }
    if arguments.len() != 1 || arguments[0].is_empty() || arguments[0] == "-" {
        return Err(USAGE.to_owned());
    }
    let directory = PathBuf::from(&arguments[0]);
    let (frame_bytes, map_bytes) = build()?;
    // An existing directory is never reused or overwritten. The map is the
    // completion marker and is published after the bound canonical frames.
    std::fs::create_dir(&directory).map_err(|error| error.to_string())?;
    let publication = (|| {
        write_new(&directory.join("h4-frames.json"), &frame_bytes)?;
        write_new(&directory.join("token-frames.json"), &map_bytes)?;
        std::fs::File::open(&directory)?.sync_all()
    })();
    publication.map_err(|error: std::io::Error| {
        format!("incomplete export at {}: {error}", directory.display())
    })?;
    println!(
        "exported 8192 native token frames to {} ({})",
        directory.display(),
        cid(&map_bytes)
    );
    Ok(())
}

fn main() -> ExitCode {
    match run(std::env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("r4-zoology-frame-export: {error}");
            ExitCode::FAILURE
        }
    }
}
