//! Private same-executable worker. This module never binds a listener.

use crate::authority::frozen_accepted_binding;
use crate::base64::decode_canonical;
use crate::intake::{load_configuration, read_exact_artifact, ValidatedConfiguration};
use crate::ipc::{read_request, write_response};
use crate::launch::hash_inherited_executable;
use crate::strict_json;
use crate::wire::{
    IpcCommand, IpcReplyKind, IpcRequest, IpcRequestPayload, IpcResponse, IpcResponsePayload,
    NativeError as WireNativeError, NativeTextResult, Progress, ProgressStage, ServiceError,
    ServiceErrorTag, WorkerReady, ARTIFACT_BYTES, IPC_SCHEMA,
};
use crate::{BoxError, SERVICE_CONTRACT_SHA256};
use std::io::{self, IsTerminal};
use std::path::Path;
use uor_r4_api::learned_reference::{
    LoadedResearchReference, NativeError, NativeErrorTag, RawRequest,
};

pub fn run() -> Result<(), BoxError> {
    if io::stdin().is_terminal() || io::stdout().is_terminal() {
        return Err("internal worker requires parent-created private pipes".into());
    }
    let (binary_sha256, _) = hash_inherited_executable()?;
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    let load_request = read_request(&mut input)?;
    let (configuration_path, configuration_sha256) =
        match (load_request.command, &load_request.payload) {
            (IpcCommand::Load, IpcRequestPayload::Load(load)) => {
                (&load.configuration_path, &load.configuration_sha256)
            }
            _ => {
                send_failure(
                    &mut output,
                    &load_request,
                    protocol_error("The first worker command must be load."),
                )?;
                return Err("first worker command was not load".into());
            }
        };

    let binding = frozen_accepted_binding()?;
    let configuration = match load_configuration(
        Path::new(configuration_path),
        configuration_sha256,
        &binary_sha256,
        &binding,
        SERVICE_CONTRACT_SHA256,
    ) {
        Ok(configuration) => configuration,
        Err(error) => {
            send_failure(
                &mut output,
                &load_request,
                service_error(
                    ServiceErrorTag::ArtifactRejected,
                    "Worker configuration did not match the adopted host release.",
                    None,
                ),
            )?;
            return Err(error.into());
        }
    };
    if configuration.host_acceptance.is_none() {
        send_failure(
            &mut output,
            &load_request,
            service_error(
                ServiceErrorTag::UnavailableNativeQualification,
                "Current host has no accepted qualification.",
                None,
            ),
        )?;
        return Err("worker refused absent host acceptance".into());
    }

    send_progress(
        &mut output,
        &load_request,
        Progress {
            stage: ProgressStage::ReadingArtifact,
            completed: Some(0),
            total: Some(ARTIFACT_BYTES),
            unit: Some("bytes".to_owned()),
            fraction: None,
            eta_ms: None,
        },
    )?;
    let artifact = match read_exact_artifact(&configuration) {
        Ok(artifact) => artifact,
        Err(error) => {
            send_failure(
                &mut output,
                &load_request,
                service_error(
                    ServiceErrorTag::UnavailableArtifact,
                    "The configured native artifact is unavailable.",
                    None,
                ),
            )?;
            return Err(error.into());
        }
    };
    send_progress(
        &mut output,
        &load_request,
        Progress {
            stage: ProgressStage::ReadingArtifact,
            completed: Some(ARTIFACT_BYTES),
            total: Some(ARTIFACT_BYTES),
            unit: Some("bytes".to_owned()),
            fraction: None,
            eta_ms: None,
        },
    )?;
    send_progress(
        &mut output,
        &load_request,
        plain_progress(ProgressStage::Validating),
    )?;
    let mut engine = match LoadedResearchReference::load(
        artifact,
        &configuration.configuration.value.expected_binding,
    ) {
        Ok(engine) => engine,
        Err(error) => {
            send_failure(&mut output, &load_request, map_native_error(&error, true))?;
            return Err(error.into());
        }
    };
    send_progress(
        &mut output,
        &load_request,
        plain_progress(ProgressStage::Qualifying),
    )?;
    let adopted = configuration
        .host_acceptance
        .as_ref()
        .ok_or("accepted host disappeared")?;
    if let Err(error) = engine.qualify(
        &adopted.qualification_bytes,
        &adopted.acceptance.value.qualification.sha256,
        adopted.runtime_identity.clone(),
    ) {
        send_failure(&mut output, &load_request, map_native_error(&error, false))?;
        return Err(error.into());
    }
    if engine.artifact_sha256() != configuration.configured_artifact.artifact_sha256
        || engine.owned_artifact_bytes() as u64 != ARTIFACT_BYTES
        || engine.manifest().native_state_sha256
            != configuration.configured_artifact.native_state_sha256
    {
        let error = protocol_error("Loaded worker identity differs from configuration.");
        send_failure(&mut output, &load_request, error)?;
        return Err("loaded worker identity mismatch".into());
    }
    send_reply(
        &mut output,
        &load_request,
        IpcReplyKind::Ready,
        IpcResponsePayload::Ready(WorkerReady {
            host: configuration.host.clone(),
            artifact: configuration.configured_artifact.clone(),
        }),
    )?;

    serve_loaded(
        &mut input,
        &mut output,
        &configuration,
        &engine,
        &load_request,
    )
}

fn serve_loaded(
    input: &mut impl io::Read,
    output: &mut impl io::Write,
    configuration: &ValidatedConfiguration,
    engine: &LoadedResearchReference,
    load: &IpcRequest,
) -> Result<(), BoxError> {
    loop {
        let request = read_request(input)?;
        if request.instance_id != load.instance_id
            || request.worker_generation != load.worker_generation
        {
            send_failure(
                output,
                &request,
                protocol_error("Worker correlation did not match its loaded instance."),
            )?;
            return Err("worker correlation mismatch".into());
        }
        match (&request.command, &request.payload) {
            (IpcCommand::Answer, IpcRequestPayload::Answer(raw)) => {
                send_progress(output, &request, plain_progress(ProgressStage::Inference))?;
                let bytes = match decode_canonical(&raw.bytes_b64, 8_192) {
                    Ok(bytes) => bytes,
                    Err(error) => {
                        send_failure(
                            output,
                            &request,
                            protocol_error("Worker received noncanonical raw transport."),
                        )?;
                        return Err(error.into());
                    }
                };
                let result = match engine.answer(RawRequest {
                    schema: &raw.schema,
                    text: &bytes,
                }) {
                    Ok(result) => result,
                    Err(error) => {
                        send_failure(output, &request, map_native_error(&error, false))?;
                        return Err(error.into());
                    }
                };
                let encoded = serde_json::to_vec(&result)?;
                let result: NativeTextResult = strict_json::from_slice(&encoded)?;
                result.validate()?;
                send_reply(
                    output,
                    &request,
                    IpcReplyKind::Result,
                    IpcResponsePayload::Result(result),
                )?;
            }
            (IpcCommand::Unload, IpcRequestPayload::Empty(())) => {
                let _ = configuration;
                send_progress(output, &request, plain_progress(ProgressStage::Terminating))?;
                send_reply(
                    output,
                    &request,
                    IpcReplyKind::Unloaded,
                    IpcResponsePayload::Empty(()),
                )?;
                return Ok(());
            }
            _ => {
                send_failure(
                    output,
                    &request,
                    protocol_error("Worker command is invalid for a loaded reference."),
                )?;
                return Err("invalid loaded worker command".into());
            }
        }
    }
}

fn plain_progress(stage: ProgressStage) -> Progress {
    Progress {
        stage,
        completed: None,
        total: None,
        unit: None,
        fraction: None,
        eta_ms: None,
    }
}

fn send_progress(
    output: &mut impl io::Write,
    request: &IpcRequest,
    progress: Progress,
) -> Result<(), BoxError> {
    send_reply(
        output,
        request,
        IpcReplyKind::Progress,
        IpcResponsePayload::Progress(progress),
    )
}

fn send_failure(
    output: &mut impl io::Write,
    request: &IpcRequest,
    error: ServiceError,
) -> Result<(), BoxError> {
    send_reply(
        output,
        request,
        IpcReplyKind::Failure,
        IpcResponsePayload::Failure(error),
    )
}

fn send_reply(
    output: &mut impl io::Write,
    request: &IpcRequest,
    kind: IpcReplyKind,
    payload: IpcResponsePayload,
) -> Result<(), BoxError> {
    write_response(
        output,
        &IpcResponse {
            schema: IPC_SCHEMA.to_owned(),
            instance_id: request.instance_id.clone(),
            job_id: request.job_id.clone(),
            worker_generation: request.worker_generation,
            kind,
            payload,
        },
    )?;
    Ok(())
}

fn map_native_error(error: &NativeError, loader: bool) -> ServiceError {
    let wire_native = serde_json::to_vec(error)
        .ok()
        .and_then(|bytes| strict_json::from_slice::<WireNativeError>(&bytes).ok());
    let tag = match &error.tag {
        NativeErrorTag::UnavailableNativeQualification => {
            ServiceErrorTag::UnavailableNativeQualification
        }
        NativeErrorTag::UnsupportedProfile => ServiceErrorTag::UnsupportedRuntime,
        NativeErrorTag::UnavailableArtifact => ServiceErrorTag::UnavailableArtifact,
        NativeErrorTag::Busy | NativeErrorTag::NumericalFailure => ServiceErrorTag::NativeFailure,
        _ if loader => ServiceErrorTag::ArtifactRejected,
        _ => ServiceErrorTag::NativeFailure,
    };
    let message = match tag {
        ServiceErrorTag::UnavailableNativeQualification => {
            "Native qualification is unavailable for this exact worker."
        }
        ServiceErrorTag::UnsupportedRuntime => {
            "The executing runtime does not match the accepted operator profile."
        }
        ServiceErrorTag::UnavailableArtifact => "The configured native artifact is unavailable.",
        ServiceErrorTag::ArtifactRejected => {
            "Native artifact validation rejected the configured bytes."
        }
        _ => "The native research reference failed without a publishable result.",
    };
    service_error(tag, message, wire_native)
}

fn protocol_error(message: &str) -> ServiceError {
    service_error(ServiceErrorTag::WorkerProtocolFailure, message, None)
}

fn service_error(
    tag: ServiceErrorTag,
    message: &str,
    native: Option<WireNativeError>,
) -> ServiceError {
    let mut message_bytes = 0;
    let message: String = message
        .chars()
        .filter(|character| !character.is_control())
        .take_while(|character| {
            let next = message_bytes + character.len_utf8();
            if next > 512 {
                false
            } else {
                message_bytes = next;
                true
            }
        })
        .collect();
    ServiceError {
        tag,
        message,
        native,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_native_tags_map_to_a_frozen_public_tag() {
        let tags = [
            NativeErrorTag::ContainerLimit,
            NativeErrorTag::InvalidContainer,
            NativeErrorTag::ArtifactIdentityMismatch,
            NativeErrorTag::UnsupportedManifest,
            NativeErrorTag::UnsupportedProfile,
            NativeErrorTag::SourceBindingMismatch,
            NativeErrorTag::InvalidComponent,
            NativeErrorTag::InvalidTensor,
            NativeErrorTag::InvalidCodecPolicy,
            NativeErrorTag::InvalidFrameTable,
            NativeErrorTag::StateIdentityMismatch,
            NativeErrorTag::UnavailableArtifact,
            NativeErrorTag::UnavailableNativeQualification,
            NativeErrorTag::Busy,
            NativeErrorTag::NumericalFailure,
        ];
        for tag in tags {
            let value = serde_json::json!({"tag":tag,"component":null,"offset":null});
            let error: NativeError = serde_json::from_value(value).unwrap();
            map_native_error(&error, true).validate().unwrap();
            map_native_error(&error, false).validate().unwrap();
        }
    }

    #[test]
    fn worker_commands_contain_no_private_comparison_variant() {
        let commands = [IpcCommand::Load, IpcCommand::Answer, IpcCommand::Unload];
        assert_eq!(commands.len(), 3);
    }

    #[test]
    fn configured_artifact_size_is_fixed_before_worker_read() {
        assert_eq!(ARTIFACT_BYTES, 2_172_252);
        let _ = crate::wire::NativeErrorTag::UnavailableNativeQualification;
    }

    #[test]
    fn service_error_message_is_control_free_and_bounded_in_utf8_bytes() {
        let input = format!("\0{}tail", "é".repeat(300));
        let error = service_error(ServiceErrorTag::WorkerFailure, &input, None);
        assert_eq!(error.message.len(), 512);
        assert_eq!(error.message.chars().count(), 256);
        assert!(!error.message.chars().any(char::is_control));
        error.validate().expect("bounded service error");
    }
}
