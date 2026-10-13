//! Shared production vector-channel/store bridge, without the retired whole-path graph.
use crate::Result;

/// New compiler/exact-store/emitter writer entry, sharing the stack's actual
/// learned embedding parameter. Old whole-path probability artifacts cannot
/// be silently adopted or converted into this vector channel.
pub fn native_vector_channel(
    model: &crate::geometric_stack::StackModel,
) -> Result<crate::native_vector_writer::NativeVectorChannel> {
    model.native_vector_channel()
}

/// Host exact-store metadata selects a record; all vector lookup stays on CUDA.
/// Record/version and per-record occurrence survive independently of geometry.
pub struct DeviceVectorStoreValue {
    pub record: u64,
    pub commit: u64,
    pub update: crate::stack_store::Update,
    pub conflict: bool,
    pub tokens: Vec<crate::native_vector_writer::CarriedVectorToken>,
}
pub enum DeviceVectorStoreRead {
    Found(DeviceVectorStoreValue),
    Absent,
    NoHistory,
    Evicted,
}
pub fn carry_native_store_read(
    channel: &crate::native_vector_writer::NativeVectorChannel,
    read: &crate::stack_store::StoreRead,
) -> Result<DeviceVectorStoreRead> {
    Ok(match read {
        crate::stack_store::StoreRead::Found(value) => {
            DeviceVectorStoreRead::Found(DeviceVectorStoreValue {
                record: value.record,
                commit: value.commit,
                update: value.update,
                conflict: value.conflict,
                tokens: channel.compile_tokens(&value.tokens, 0, value.commit)?,
            })
        }
        crate::stack_store::StoreRead::Absent => DeviceVectorStoreRead::Absent,
        crate::stack_store::StoreRead::NoHistory => DeviceVectorStoreRead::NoHistory,
        crate::stack_store::StoreRead::Evicted => DeviceVectorStoreRead::Evicted,
    })
}
