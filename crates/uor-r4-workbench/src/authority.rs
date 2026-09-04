//! Compile-time authority inputs shared by the parent and private worker.

use crate::intake::sha256;
use crate::strict_json;
use crate::{BoxError, ORIGINAL_BINDING_SOURCE_SHA256, PRIVATE_RELEASE_CONTRACT_SHA256};
use serde_json::Value;

const ORIGINAL_BINDING_SOURCE: &[u8] =
    include_bytes!("../../../docs/r4_native_bridge_1102_evidence/qualification-handoff.json");
const PRIVATE_RELEASE_CONTRACT: &[u8] =
    include_bytes!("../../../docs/r4_workbench_private_release_1107.json");

pub fn frozen_accepted_binding() -> Result<Value, BoxError> {
    if sha256(ORIGINAL_BINDING_SOURCE) != ORIGINAL_BINDING_SOURCE_SHA256 {
        return Err("compiled original binding source identity mismatch".into());
    }
    let source: Value = strict_json::from_slice(ORIGINAL_BINDING_SOURCE)?;
    source
        .get("trusted_binding")
        .and_then(|value| value.get("accepted_binding"))
        .cloned()
        .ok_or_else(|| "compiled original binding is missing".into())
}

pub fn validate_private_release_contract() -> Result<(), BoxError> {
    if sha256(PRIVATE_RELEASE_CONTRACT) != PRIVATE_RELEASE_CONTRACT_SHA256 {
        return Err("compiled private release contract identity mismatch".into());
    }
    let contract: Value = strict_json::from_slice(PRIVATE_RELEASE_CONTRACT)?;
    if contract["schema"] != "uor-r4.workbench-private-release-contract/1"
        || contract["issue"] != 1107
    {
        return Err("compiled private release contract schema mismatch".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiled_authority_has_the_frozen_binding_shape() {
        let binding = frozen_accepted_binding().unwrap();
        assert_eq!(
            binding["policy_sha256"],
            "91cce30a0b78c48130595369d3ea2a47c4de89cab5db1d4219d1874198cf52d0"
        );
        assert!(binding["assets"]["reader"]["cid"]
            .as_str()
            .unwrap()
            .starts_with("blake3:"));
        validate_private_release_contract().unwrap();
    }
}
