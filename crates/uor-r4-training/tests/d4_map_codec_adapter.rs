use std::sync::Arc;
use uor_r4_integer::codec::{Grouped4BitCodec, Grouped4BitRounding};
use uor_r4_training::geometric_stack::{D11Interim, MapCodec};
use uor_r4_training::Result;

/// Lab 3 adapter connecting `Grouped4BitCodec` to the shared `MapCodec` training interface.
#[derive(Clone, Debug, Default)]
pub struct D4Grouped4BitAdapter(pub Grouped4BitCodec);

impl MapCodec for D4Grouped4BitAdapter {
    fn name(&self) -> &str {
        self.0.name()
    }

    fn round_trip(&self, values: &[f32], rows: usize, cols: usize) -> Result<Vec<f32>> {
        self.0.round_trip(values, rows, cols).map_err(Into::into)
    }
}

#[test]
fn native_grouped_4bit_codec_plugs_into_map_codec_and_matches_d11_interim() -> Result<()> {
    let values: Vec<f32> = (0..64 * 32)
        .map(|i| (((i as f32) * 0.17) % 7.0 - 3.5) * 0.1)
        .collect();
    let d11 = D11Interim;
    let native_rtn = D4Grouped4BitAdapter(Grouped4BitCodec::default());

    let rt_d11 = d11.round_trip(&values, 64, 32)?;
    let rt_native = native_rtn.round_trip(&values, 64, 32)?;
    assert_eq!(
        rt_d11, rt_native,
        "Native Grouped4BitCodec via D4Grouped4BitAdapter must match D11Interim round-trip bit for bit"
    );

    let dyn_codec: Arc<dyn MapCodec> = Arc::new(native_rtn);
    assert_eq!(dyn_codec.name(), "native-d11-grouped-4bit-g32-rtn");
    let rt_dyn = dyn_codec.round_trip(&values, 64, 32)?;
    assert_eq!(rt_dyn, rt_d11);

    // Test with MinimumMseScale mode as well
    let native_opt = D4Grouped4BitAdapter(Grouped4BitCodec::new(
        32,
        Grouped4BitRounding::MinimumMseScale,
    ));
    let dyn_opt: Arc<dyn MapCodec> = Arc::new(native_opt);
    assert_eq!(dyn_opt.name(), "native-d11-grouped-4bit-g32-min-mse");
    let rt_opt = dyn_opt.round_trip(&values, 64, 32)?;
    assert_eq!(rt_opt.len(), 64 * 32);

    Ok(())
}
