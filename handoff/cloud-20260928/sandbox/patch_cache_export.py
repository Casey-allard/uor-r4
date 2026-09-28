import re
p='crates/uor-r4-training/src/lut_export.rs'
s=open(p).read()
def rep(old,new):
    global s
    assert old in s, old[:90]
    s=s.replace(old,new,1)
rep('''use uor_r4_lut::format::{ArtifactBuilder, Fixed, Numerics, Shape, TableValues};''','''use uor_r4_lut::format::{ArtifactBuilder, CacheSpec, Fixed, Numerics, Shape, TableValues};''')
rep('''/// Export `checkpoint` for integer serving with sessions of up to
/// `max_positions` tokens, with GPTQ when `calibration` gives input moments
/// (and its damping). Returns the artifact bytes and a quantization report.
pub fn export_llama(
    checkpoint: &Checkpoint,
    max_positions: usize,
    source: Value,
    calibration: Option<(&Calibration, f64)>,
) -> Result<(Vec<u8>, Value)> {''', open('/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/cache_weights.rs').read())
rep('''    add(
        &mut builder,
        "head",
        &head,
        s.vocab,
        s.width,
        Some(Site::Head),
    )?;
''', open('/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/cache_add.rs').read())
open(p,'w').write(s)
print('ok')
