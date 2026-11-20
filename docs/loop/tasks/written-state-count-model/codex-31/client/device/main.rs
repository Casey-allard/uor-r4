//! Independent CUDA greedy replay of the ordinary D31 client panel.
#![forbid(unsafe_code)]

#[cfg(feature = "cuda")]
#[path = "../../../../../../labs/kn-reference/instrument/src/pick_device.rs"]
mod pick_device;

#[cfg(feature = "cuda")]
mod replay;

use std::path::PathBuf;
type Failure = Box<dyn std::error::Error>;
fn fail(message: impl Into<String>) -> Failure {
    message.into().into()
}

fn main() -> Result<(), Failure> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    match args.as_slice() {
        #[cfg(feature = "cuda")]
        [mode, mixer, trunk, out] if mode == std::path::Path::new("finite") => replay::finite(mixer, trunk, out),
        #[cfg(feature = "cuda")]
        [mode, mixer, trunk, counts, manifest, weights, weights_sha, packet, out]
            if mode == std::path::Path::new("replay") => {
                replay::run(mixer, trunk, counts, manifest, weights, weights_sha, packet, out)
            }
        _ => Err(fail("usage: d31-client-device finite MIXER TRUNK NEW-DIR | replay MIXER TRUNK COUNTS FIT-MANIFEST WEIGHTS WEIGHTS-SHA PREFIX-JSON NEW-DIR (CUDA build)")),
    }
}
