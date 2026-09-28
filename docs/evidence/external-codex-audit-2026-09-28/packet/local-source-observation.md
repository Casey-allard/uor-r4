# Local source observation: radial information in the addressing contest

Observed read-only on Casey's Mac through Desktop Commander, 28 September 2026, 05:25–05:26 UTC. The source was not edited or executed. No origin reference contained the observed commit according to the cached local reference test at 05:25; this is not a fresh GitHub publication query.

Repository: `/Users/casey.allard/uor-r4`.
Commit: `924ffb2d77f0248158e647a59006cc1c2885b2d5`.
Path: `crates/uor-r4-training/src/addressing_arms.rs`.
Git blob: `656e39b40bc78bc11182d76ee173e867c3b4977d`.
SHA-256: `a4dc5e08e84007ab3d3d0b80a92cda0c028912c262f610c3c921a98194e64cf9`.

Relevant source excerpts:

```rust
/// One quantized key: one code index per block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Codes {
    /// Code index for each block, in block order.
    pub blocks: Vec<u8>,
}
```

`fit_kmeans` copies raw `f32` key coordinates into `f64` block points, calls `fit_block_centroids`, and retains the fitted centroids. No unit normalization occurs in that inspected construction.

```rust
pub fn encode(&self, key: &[f32]) -> Result<Codes> {
    let rotated = self.rotate(key)?;
    let mut codes = Vec::with_capacity(self.blocks);
    for block in 0..self.blocks {
        let q = &rotated[block * self.block_dim..(block + 1) * self.block_dim];
        codes.push(self.assign_block(block, q)? as u8);
    }
    Ok(Codes { blocks: codes })
}
```

`query_lut` takes the unquantized query, applies the configured rotation, and computes block inner products against all code coordinates. `score` sums the entries selected by `codes.blocks`. There is no per-key gain field or multiplicative gain in that score.

```rust
match self.family {
    CodebookFamily::H4 => {
        let root = canonical_h4_roots()
            .get(code)
            .ok_or(AddressingError::NotFitted("H4 roots"))?;
        Ok(root.to_array().to_vec())
    }
    CodebookFamily::E8 => {
        let root = e8_roots()
            .get(code)
            .ok_or(AddressingError::NotFitted("E8 roots"))?;
        Ok(root.to_vec())
    }
    CodebookFamily::KMeans => self
        .centroid(block, code)
        .map(|slice| slice.to_vec())
        .ok_or(AddressingError::NotFitted("k-means centroids")),
    // Sign-family branch omitted from this excerpt.
}
```

The local frozen plan already specifies asymmetric scoring and an equal-cost complete-quantizer contest. Its scope is one parent, T=256 and declared bit rates. This review does not alter that study. The implication is narrower: fixed-root and raw learned-centroid arms have different representational treatment of magnitude, so their outcome does not isolate geometry alone. A future gain-controlled comparison must charge the added storage and computation for all arms.

Retrieval commands included:

```sh
git -C /Users/casey.allard/uor-r4 show 924ffb2d77f0248158e647a59006cc1c2885b2d5:crates/uor-r4-training/src/addressing_arms.rs | sed -n '255,306p;565,650p;410,475p'
git -C /Users/casey.allard/uor-r4 show 924ffb2d77f0248158e647a59006cc1c2885b2d5:crates/uor-r4-training/src/addressing_arms.rs | sed -n '650,760p'
git -C /Users/casey.allard/uor-r4 rev-parse 924ffb2d77f0248158e647a59006cc1c2885b2d5:crates/uor-r4-training/src/addressing_arms.rs
git -C /Users/casey.allard/uor-r4 show 924ffb2d77f0248158e647a59006cc1c2885b2d5:crates/uor-r4-training/src/addressing_arms.rs | shasum -a 256
```
