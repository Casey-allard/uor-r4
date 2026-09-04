//! Bounded framing and reply sequencing for the private worker pipe.
//!
//! This module owns no process and opens no socket.  The effect layer supplies
//! the parent-created stdin/stdout pipes.  A `ReplySequence` represents the
//! contract's single command in flight and prevents a reply for one worker or
//! job from completing another.

use crate::strict_json;
use crate::wire::{
    IpcCommand, IpcReplyKind, IpcRequest, IpcResponse, IpcResponsePayload, Progress, ProgressStage,
    ServiceErrorTag,
};
use serde::Serialize;
use std::fmt;
use std::io::{self, Read, Write};

pub const MAX_FRAME_BYTES: usize = 65_536;

#[derive(Debug)]
pub enum IpcError {
    Io(io::Error),
    FrameTooLarge { declared: u64, maximum: usize },
    InvalidMessage(String),
    CorrelationMismatch,
    ReplyAfterTerminal,
    UnexpectedReply,
    InvalidProgress(&'static str),
}

impl IpcError {
    pub fn service_tag(&self) -> ServiceErrorTag {
        match self {
            Self::Io(_) => ServiceErrorTag::WorkerFailure,
            Self::FrameTooLarge { .. }
            | Self::InvalidMessage(_)
            | Self::CorrelationMismatch
            | Self::ReplyAfterTerminal
            | Self::UnexpectedReply
            | Self::InvalidProgress(_) => ServiceErrorTag::WorkerProtocolFailure,
        }
    }
}

impl fmt::Display for IpcError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "private IPC I/O failed: {error}"),
            Self::FrameTooLarge { declared, maximum } => write!(
                formatter,
                "private IPC frame declares {declared} bytes; maximum is {maximum}"
            ),
            Self::InvalidMessage(message) => {
                write!(formatter, "invalid private IPC message: {message}")
            }
            Self::CorrelationMismatch => {
                formatter.write_str("private IPC correlation does not match the active command")
            }
            Self::ReplyAfterTerminal => {
                formatter.write_str("private IPC reply followed a terminal reply")
            }
            Self::UnexpectedReply => {
                formatter.write_str("private IPC reply is not valid for the active command")
            }
            Self::InvalidProgress(message) => {
                write!(formatter, "invalid private IPC progress: {message}")
            }
        }
    }
}

impl std::error::Error for IpcError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for IpcError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Read one `u32` little-endian length followed by exactly that many bytes.
///
/// The declared length is rejected before allocating the payload buffer.
pub fn read_frame(reader: &mut impl Read) -> Result<Vec<u8>, IpcError> {
    let mut prefix = [0u8; 4];
    reader.read_exact(&mut prefix)?;
    let declared = u32::from_le_bytes(prefix) as usize;
    if declared > MAX_FRAME_BYTES {
        return Err(IpcError::FrameTooLarge {
            declared: declared as u64,
            maximum: MAX_FRAME_BYTES,
        });
    }
    let mut payload = vec![0u8; declared];
    reader.read_exact(&mut payload)?;
    Ok(payload)
}

/// Write one already-encoded frame after enforcing the same wire bound.
pub fn write_frame(writer: &mut impl Write, payload: &[u8]) -> Result<(), IpcError> {
    if payload.len() > MAX_FRAME_BYTES {
        return Err(IpcError::FrameTooLarge {
            declared: payload.len() as u64,
            maximum: MAX_FRAME_BYTES,
        });
    }
    let length = u32::try_from(payload.len()).map_err(|_| IpcError::FrameTooLarge {
        declared: payload.len() as u64,
        maximum: MAX_FRAME_BYTES,
    })?;
    writer.write_all(&length.to_le_bytes())?;
    writer.write_all(payload)?;
    writer.flush()?;
    Ok(())
}

struct BoundedJsonBuffer {
    bytes: Vec<u8>,
}

impl BoundedJsonBuffer {
    fn new() -> Self {
        Self { bytes: Vec::new() }
    }
}

impl Write for BoundedJsonBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let resulting_length = self
            .bytes
            .len()
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::other("private IPC JSON length overflow"))?;
        if resulting_length > MAX_FRAME_BYTES {
            return Err(io::Error::other("private IPC JSON exceeds frame limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn write_json<T: Serialize>(writer: &mut impl Write, message: &T) -> Result<(), IpcError> {
    let mut buffer = BoundedJsonBuffer::new();
    serde_json::to_writer(&mut buffer, message)
        .map_err(|error| IpcError::InvalidMessage(error.to_string()))?;
    write_frame(writer, &buffer.bytes)
}

pub fn read_request(reader: &mut impl Read) -> Result<IpcRequest, IpcError> {
    let bytes = read_frame(reader)?;
    let request: IpcRequest = strict_json::from_slice(&bytes)
        .map_err(|error| IpcError::InvalidMessage(error.to_string()))?;
    request
        .validate()
        .map_err(|error| IpcError::InvalidMessage(error.to_string()))?;
    Ok(request)
}

pub fn write_request(writer: &mut impl Write, request: &IpcRequest) -> Result<(), IpcError> {
    request
        .validate()
        .map_err(|error| IpcError::InvalidMessage(error.to_string()))?;
    write_json(writer, request)
}

pub fn read_response(reader: &mut impl Read) -> Result<IpcResponse, IpcError> {
    let bytes = read_frame(reader)?;
    let response: IpcResponse = strict_json::from_slice(&bytes)
        .map_err(|error| IpcError::InvalidMessage(error.to_string()))?;
    response
        .validate()
        .map_err(|error| IpcError::InvalidMessage(error.to_string()))?;
    Ok(response)
}

/// Read a response while distinguishing a clean frame-boundary EOF from a
/// truncated frame. The parent uses clean EOF as process-lifecycle evidence;
/// any partial prefix or payload remains a protocol failure.
pub fn read_response_or_eof(reader: &mut impl Read) -> Result<Option<IpcResponse>, IpcError> {
    let mut prefix = [0u8; 4];
    let first = reader.read(&mut prefix[..1])?;
    if first == 0 {
        return Ok(None);
    }
    reader.read_exact(&mut prefix[1..]).map_err(|error| {
        if error.kind() == io::ErrorKind::UnexpectedEof {
            IpcError::InvalidMessage("truncated private IPC frame length".to_owned())
        } else {
            IpcError::Io(error)
        }
    })?;
    let declared = u32::from_le_bytes(prefix) as usize;
    if declared > MAX_FRAME_BYTES {
        return Err(IpcError::FrameTooLarge {
            declared: declared as u64,
            maximum: MAX_FRAME_BYTES,
        });
    }
    let mut bytes = vec![0u8; declared];
    reader.read_exact(&mut bytes).map_err(|error| {
        if error.kind() == io::ErrorKind::UnexpectedEof {
            IpcError::InvalidMessage("truncated private IPC frame payload".to_owned())
        } else {
            IpcError::Io(error)
        }
    })?;
    let response: IpcResponse = strict_json::from_slice(&bytes)
        .map_err(|error| IpcError::InvalidMessage(error.to_string()))?;
    response
        .validate()
        .map_err(|error| IpcError::InvalidMessage(error.to_string()))?;
    Ok(Some(response))
}

pub fn write_response(writer: &mut impl Write, response: &IpcResponse) -> Result<(), IpcError> {
    response
        .validate()
        .map_err(|error| IpcError::InvalidMessage(error.to_string()))?;
    write_json(writer, response)
}

/// A semantically accepted reply for the one active exchange.
#[derive(Debug, Clone, PartialEq)]
pub enum AcceptedReply {
    Progress(Progress),
    Ready(crate::wire::WorkerReady),
    Result(crate::wire::NativeTextResult),
    Failure(crate::wire::ServiceError),
    Unloaded,
    /// The parent already committed a stop winner.  Parsed output is drained
    /// but must not replace that winner or publish a result.
    DiscardedAfterStop,
}

pub struct ReplySequence {
    instance_id: String,
    job_id: String,
    worker_generation: u64,
    command: IpcCommand,
    artifact_bytes: u64,
    last_progress: Option<Progress>,
    terminal_seen: bool,
    parent_stop_won: bool,
}

impl ReplySequence {
    pub fn new(request: &IpcRequest, artifact_bytes: u64) -> Result<Self, IpcError> {
        request
            .validate()
            .map_err(|error| IpcError::InvalidMessage(error.to_string()))?;
        Ok(Self {
            instance_id: request.instance_id.clone(),
            job_id: request.job_id.clone(),
            worker_generation: request.worker_generation,
            command: request.command,
            artifact_bytes,
            last_progress: None,
            terminal_seen: false,
            parent_stop_won: false,
        })
    }

    pub fn terminal_seen(&self) -> bool {
        self.terminal_seen
    }

    /// Freeze publication after cancellation, deadline, failure, or shutdown
    /// wins in the parent.  Subsequent structurally valid replies are drained.
    pub fn mark_parent_stop(&mut self) -> bool {
        if self.terminal_seen {
            return false;
        }
        self.parent_stop_won = true;
        true
    }

    pub fn accept(&mut self, response: IpcResponse) -> Result<AcceptedReply, IpcError> {
        response
            .validate()
            .map_err(|error| IpcError::InvalidMessage(error.to_string()))?;
        if response.instance_id != self.instance_id
            || response.job_id != self.job_id
            || response.worker_generation != self.worker_generation
        {
            return Err(IpcError::CorrelationMismatch);
        }
        if self.parent_stop_won {
            return Ok(AcceptedReply::DiscardedAfterStop);
        }
        if self.terminal_seen {
            return Err(IpcError::ReplyAfterTerminal);
        }

        let accepted = match (response.kind, response.payload) {
            (IpcReplyKind::Progress, IpcResponsePayload::Progress(progress)) => {
                validate_progress(
                    &self.command,
                    self.last_progress.as_ref(),
                    &progress,
                    self.artifact_bytes,
                )?;
                self.last_progress = Some(progress.clone());
                AcceptedReply::Progress(progress)
            }
            (IpcReplyKind::Ready, IpcResponsePayload::Ready(ready))
                if self.command == IpcCommand::Load =>
            {
                self.terminal_seen = true;
                AcceptedReply::Ready(ready)
            }
            (IpcReplyKind::Result, IpcResponsePayload::Result(result))
                if self.command == IpcCommand::Answer =>
            {
                self.terminal_seen = true;
                AcceptedReply::Result(result)
            }
            (IpcReplyKind::Unloaded, IpcResponsePayload::Empty(()))
                if self.command == IpcCommand::Unload =>
            {
                self.terminal_seen = true;
                AcceptedReply::Unloaded
            }
            (IpcReplyKind::Failure, IpcResponsePayload::Failure(error)) => {
                self.terminal_seen = true;
                AcceptedReply::Failure(error)
            }
            _ => return Err(IpcError::UnexpectedReply),
        };
        Ok(accepted)
    }
}

fn validate_progress(
    command: &IpcCommand,
    previous: Option<&Progress>,
    current: &Progress,
    artifact_bytes: u64,
) -> Result<(), IpcError> {
    if current.fraction.is_some() || current.eta_ms.is_some() {
        return Err(IpcError::InvalidProgress(
            "fraction and ETA must remain null",
        ));
    }
    if current.stage == ProgressStage::ReadingArtifact {
        let completed = current.completed.ok_or(IpcError::InvalidProgress(
            "artifact progress requires a completed byte count",
        ))?;
        if current.unit.as_deref() != Some("bytes")
            || current.total != Some(artifact_bytes)
            || completed > artifact_bytes
        {
            return Err(IpcError::InvalidProgress(
                "artifact progress must use bounded bytes with the exact total",
            ));
        }
    } else if current.completed.is_some() || current.total.is_some() || current.unit.is_some() {
        return Err(IpcError::InvalidProgress(
            "only artifact reading may carry completed, total, or unit",
        ));
    }

    let rank = progress_rank(command, &current.stage).ok_or(IpcError::InvalidProgress(
        "stage is not valid for this command",
    ))?;
    if let Some(previous) = previous {
        let previous_rank = progress_rank(command, &previous.stage)
            .ok_or(IpcError::InvalidProgress("prior stage was invalid"))?;
        if rank < previous_rank {
            return Err(IpcError::InvalidProgress("stage moved backwards"));
        }
        if current.stage == ProgressStage::ReadingArtifact
            && previous.stage == ProgressStage::ReadingArtifact
            && current.completed.unwrap_or(0) < previous.completed.unwrap_or(0)
        {
            return Err(IpcError::InvalidProgress(
                "artifact byte count moved backwards",
            ));
        }
    }
    Ok(())
}

fn progress_rank(command: &IpcCommand, stage: &ProgressStage) -> Option<u8> {
    match (command, stage) {
        (_, ProgressStage::Idle) => Some(0),
        (IpcCommand::Load, ProgressStage::ReadingArtifact) => Some(1),
        (IpcCommand::Load, ProgressStage::Validating) => Some(2),
        (IpcCommand::Load, ProgressStage::Qualifying) => Some(3),
        (IpcCommand::Answer, ProgressStage::Inference) => Some(1),
        (IpcCommand::Unload, ProgressStage::Terminating) => Some(1),
        (IpcCommand::Unload, ProgressStage::Reaping) => Some(2),
        (_, ProgressStage::Complete) => Some(4),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::{IpcLoad, IpcRequestPayload, IpcResponsePayload};
    use std::io::Cursor;

    fn unload_request() -> IpcRequest {
        IpcRequest {
            schema: "uor-r4.workbench-ipc/1".to_owned(),
            instance_id: "0".repeat(32),
            job_id: "1".to_owned(),
            worker_generation: 7,
            command: IpcCommand::Unload,
            payload: IpcRequestPayload::Empty(()),
        }
    }

    fn response(kind: IpcReplyKind, payload: IpcResponsePayload) -> IpcResponse {
        IpcResponse {
            schema: "uor-r4.workbench-ipc/1".to_owned(),
            instance_id: "0".repeat(32),
            job_id: "1".to_owned(),
            worker_generation: 7,
            kind,
            payload,
        }
    }

    fn load_request() -> IpcRequest {
        IpcRequest {
            schema: "uor-r4.workbench-ipc/1".to_owned(),
            instance_id: "0".repeat(32),
            job_id: "1".to_owned(),
            worker_generation: 7,
            command: IpcCommand::Load,
            payload: IpcRequestPayload::Load(IpcLoad {
                configuration_path: "/operator/config.json".to_owned(),
                configuration_sha256: "a".repeat(64),
            }),
        }
    }

    fn progress(stage: ProgressStage) -> Progress {
        Progress {
            stage,
            completed: None,
            total: None,
            unit: None,
            fraction: None,
            eta_ms: None,
        }
    }

    #[test]
    fn frame_prefix_is_little_endian_and_round_trips() {
        let mut bytes = Vec::new();
        write_frame(&mut bytes, b"{}\n").expect("bounded frame");
        assert_eq!(&bytes[..4], &[3, 0, 0, 0]);
        assert_eq!(
            read_frame(&mut Cursor::new(bytes)).expect("round trip"),
            b"{}\n"
        );
    }

    #[test]
    fn oversized_declared_frame_is_rejected_before_payload_read() {
        let prefix = ((MAX_FRAME_BYTES as u32) + 1).to_le_bytes();
        let error = read_frame(&mut Cursor::new(prefix)).expect_err("oversized frame");
        assert!(matches!(error, IpcError::FrameTooLarge { .. }));
    }

    #[test]
    fn response_reader_distinguishes_clean_eof_from_truncated_frames() {
        assert!(read_response_or_eof(&mut Cursor::new(Vec::<u8>::new()))
            .expect("clean boundary EOF")
            .is_none());
        assert!(matches!(
            read_response_or_eof(&mut Cursor::new(vec![1_u8])),
            Err(IpcError::InvalidMessage(_))
        ));
        let truncated_payload = [2_u32.to_le_bytes().as_slice(), b"{"].concat();
        assert!(matches!(
            read_response_or_eof(&mut Cursor::new(truncated_payload)),
            Err(IpcError::InvalidMessage(_))
        ));
    }

    #[test]
    fn strict_reader_rejects_duplicate_correlation_fields() {
        let json = br#"{"schema":"uor-r4.workbench-ipc/1","instance_id":"00000000000000000000000000000000","instance_id":"11111111111111111111111111111111","job_id":"1","worker_generation":7,"command":"unload","payload":null}"#;
        let mut framed = Vec::new();
        write_frame(&mut framed, json).expect("synthetic frame");
        assert!(matches!(
            read_request(&mut Cursor::new(framed)),
            Err(IpcError::InvalidMessage(_))
        ));
    }

    #[test]
    fn exchange_enforces_correlation_and_one_terminal_reply() {
        let request = unload_request();
        let mut sequence = ReplySequence::new(&request, 2_172_252).expect("sequence");
        let mut mismatched = response(IpcReplyKind::Unloaded, IpcResponsePayload::Empty(()));
        mismatched.worker_generation = 8;
        assert!(matches!(
            sequence.accept(mismatched),
            Err(IpcError::CorrelationMismatch)
        ));

        let terminal = response(IpcReplyKind::Unloaded, IpcResponsePayload::Empty(()));
        assert_eq!(
            sequence.accept(terminal.clone()).expect("terminal"),
            AcceptedReply::Unloaded
        );
        assert!(
            !sequence.mark_parent_stop(),
            "terminal reply won arbitration"
        );
        assert!(matches!(
            sequence.accept(terminal),
            Err(IpcError::ReplyAfterTerminal)
        ));
    }

    #[test]
    fn progress_is_a_monotonic_command_specific_subsequence() {
        let request = unload_request();
        let mut sequence = ReplySequence::new(&request, 2_172_252).expect("sequence");
        sequence
            .accept(response(
                IpcReplyKind::Progress,
                IpcResponsePayload::Progress(progress(ProgressStage::Terminating)),
            ))
            .expect("terminating may be first observed stage");
        sequence
            .accept(response(
                IpcReplyKind::Progress,
                IpcResponsePayload::Progress(progress(ProgressStage::Reaping)),
            ))
            .expect("reaping follows terminating");
        assert!(matches!(
            sequence.accept(response(
                IpcReplyKind::Progress,
                IpcResponsePayload::Progress(progress(ProgressStage::Terminating)),
            )),
            Err(IpcError::InvalidProgress(_))
        ));
    }

    #[test]
    fn artifact_progress_requires_exact_total_and_nondecreasing_bytes() {
        let request = load_request();
        let mut sequence = ReplySequence::new(&request, 2_172_252).expect("sequence");
        let artifact_progress = |completed| Progress {
            stage: ProgressStage::ReadingArtifact,
            completed: Some(completed),
            total: Some(2_172_252),
            unit: Some("bytes".to_owned()),
            fraction: None,
            eta_ms: None,
        };
        sequence
            .accept(response(
                IpcReplyKind::Progress,
                IpcResponsePayload::Progress(artifact_progress(10)),
            ))
            .expect("first observed bytes");
        assert!(matches!(
            sequence.accept(response(
                IpcReplyKind::Progress,
                IpcResponsePayload::Progress(artifact_progress(9)),
            )),
            Err(IpcError::InvalidProgress(_))
        ));
    }

    #[test]
    fn a_parent_stop_discards_late_output_without_changing_the_winner() {
        let request = unload_request();
        let mut sequence = ReplySequence::new(&request, 2_172_252).expect("sequence");
        assert!(sequence.mark_parent_stop());

        let mut wrong_instance = response(IpcReplyKind::Unloaded, IpcResponsePayload::Empty(()));
        wrong_instance.instance_id = "1".repeat(32);
        assert!(matches!(
            sequence.accept(wrong_instance),
            Err(IpcError::CorrelationMismatch)
        ));

        let mut wrong_job = response(IpcReplyKind::Unloaded, IpcResponsePayload::Empty(()));
        wrong_job.job_id = "99".to_owned();
        assert!(matches!(
            sequence.accept(wrong_job),
            Err(IpcError::CorrelationMismatch)
        ));

        let mut wrong_generation = response(IpcReplyKind::Unloaded, IpcResponsePayload::Empty(()));
        wrong_generation.worker_generation = 8;
        assert!(matches!(
            sequence.accept(wrong_generation),
            Err(IpcError::CorrelationMismatch)
        ));

        let current = response(IpcReplyKind::Unloaded, IpcResponsePayload::Empty(()));
        assert_eq!(
            sequence
                .accept(current)
                .expect("correlated late output is drained"),
            AcceptedReply::DiscardedAfterStop
        );
        assert!(!sequence.terminal_seen());
    }
}
