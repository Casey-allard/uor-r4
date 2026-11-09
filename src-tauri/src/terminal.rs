use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use serde::Serialize;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::{DesktopBackend, DesktopClientError};

const MAX_TERMINAL_SESSIONS: usize = 8;
const MAX_TERMINAL_INPUT_BYTES: usize = 64 * 1024;
const MAX_TERMINAL_RETAINED_BYTES: usize = 1024 * 1024;
const MAX_TERMINAL_OUTPUT_CHUNK: usize = 16 * 1024;

#[derive(Debug, Clone, Serialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum TerminalEvent {
    Output {
        session_id: String,
        sequence: u64,
        data_hex: String,
    },
    Exit {
        session_id: String,
        sequence: u64,
        code: Option<u32>,
    },
    Error {
        session_id: String,
        sequence: u64,
        message: String,
    },
}

impl TerminalEvent {
    fn sequence(&self) -> u64 {
        match self {
            Self::Output { sequence, .. }
            | Self::Exit { sequence, .. }
            | Self::Error { sequence, .. } => *sequence,
        }
    }

    fn bytes(&self) -> usize {
        match self {
            Self::Output { data_hex, .. } => data_hex.len() / 2,
            Self::Exit { .. } => 0,
            Self::Error { message, .. } => message.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalSnapshot {
    pub session_id: String,
    pub cursor: u64,
    pub active: bool,
    pub truncated: bool,
    pub cwd: PathBuf,
    pub shell: PathBuf,
    pub pid: Option<u32>,
    pub events: Vec<TerminalEvent>,
}

#[derive(Default)]
struct EventQueue {
    events: VecDeque<TerminalEvent>,
    next_sequence: u64,
    retained_bytes: usize,
    active: bool,
    truncated: bool,
    exit_emitted: bool,
}

impl EventQueue {
    fn push(&mut self, event: TerminalEvent) {
        self.next_sequence += 1;
        let next_sequence = self.next_sequence;
        let mut event = event;
        match &mut event {
            TerminalEvent::Output { sequence, .. }
            | TerminalEvent::Exit { sequence, .. }
            | TerminalEvent::Error { sequence, .. } => *sequence = next_sequence,
        }
        self.retained_bytes += event.bytes();
        self.events.push_back(event);
        while self.retained_bytes > MAX_TERMINAL_RETAINED_BYTES {
            let Some(old) = self.events.pop_front() else {
                break;
            };
            self.retained_bytes = self.retained_bytes.saturating_sub(old.bytes());
            self.truncated = true;
        }
    }

    fn snapshot(&self, after: u64) -> (u64, bool, bool, Vec<TerminalEvent>) {
        (
            self.next_sequence,
            self.active,
            self.truncated,
            self.events
                .iter()
                .filter(|event| event.sequence() > after)
                .cloned()
                .collect(),
        )
    }
}

pub(crate) struct TerminalSession {
    shared: Arc<Mutex<EventQueue>>,
    stopped: Arc<AtomicBool>,
    child: Box<dyn Child + Send + Sync>,
    master: Mutex<Option<Box<dyn MasterPty>>>,
    writer: Mutex<Option<Box<dyn Write + Send>>>,
    reader: Option<JoinHandle<()>>,
    cwd: PathBuf,
    shell: PathBuf,
}

impl TerminalSession {
    fn spawn(
        backend: &mut DesktopBackend,
        cwd: &Path,
        shell: &Path,
        rows: u16,
        cols: u16,
    ) -> Result<(String, Self), DesktopClientError> {
        let pair = native_pty_system()
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| DesktopClientError::CommandFailed(error.to_string()))?;
        let mut command = CommandBuilder::new(shell);
        command.arg("-i");
        command.cwd(cwd);
        command.env_clear();
        command.env("PATH", std::env::var_os("PATH").unwrap_or_default());
        command.env("TERM", "xterm-256color");
        command.env("LANG", "en_US.UTF-8");
        command.env("LC_ALL", "en_US.UTF-8");
        command.env("SHELL", shell);
        let home = backend
            .terminal_home
            .clone()
            .or_else(|| std::env::var_os("HOME").map(PathBuf::from));
        if let Some(home) = home {
            command.env("HOME", &home);
            command.env("ZDOTDIR", home);
        }
        let child = pair
            .slave
            .spawn_command(command)
            .map_err(|error| DesktopClientError::CommandFailed(error.to_string()))?;
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|error| DesktopClientError::CommandFailed(error.to_string()))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|error| DesktopClientError::CommandFailed(error.to_string()))?;
        let shared = Arc::new(Mutex::new(EventQueue {
            active: true,
            ..EventQueue::default()
        }));
        let stopped = Arc::new(AtomicBool::new(false));
        let id = format!("terminal-{}", backend.next_terminal_id);
        backend.next_terminal_id += 1;
        let event_shared = shared.clone();
        let event_stopped = stopped.clone();
        let reader_session = id.clone();
        let reader = std::thread::Builder::new()
            .name(format!("uor-{reader_session}"))
            .spawn(move || {
                let mut buffer = [0u8; MAX_TERMINAL_OUTPUT_CHUNK];
                loop {
                    match reader.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(count) => {
                            if event_stopped.load(Ordering::Relaxed) {
                                break;
                            }
                            let event = TerminalEvent::Output {
                                session_id: reader_session.clone(),
                                sequence: 0,
                                data_hex: hex::encode(&buffer[..count]),
                            };
                            event_shared
                                .lock()
                                .unwrap_or_else(|error| error.into_inner())
                                .push(event);
                        }
                        Err(_) => break,
                    }
                }
                event_shared
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .active = false;
            })
            .map_err(|error| DesktopClientError::CommandFailed(error.to_string()))?;
        Ok((
            id,
            Self {
                shared,
                stopped,
                child,
                master: Mutex::new(Some(pair.master)),
                writer: Mutex::new(Some(writer)),
                reader: Some(reader),
                cwd: cwd.to_path_buf(),
                shell: shell.to_path_buf(),
            },
        ))
    }

    fn reap(&mut self) {
        let status = self
            .child
            .try_wait()
            .ok()
            .flatten()
            .map(|status| status.exit_code());
        if let Some(code) = status {
            let mut queue = self
                .shared
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            queue.active = false;
            if !queue.exit_emitted {
                queue.exit_emitted = true;
                queue.push(TerminalEvent::Exit {
                    session_id: String::new(),
                    sequence: 0,
                    code: Some(code),
                });
            }
        }
    }

    fn snapshot(&mut self, after: u64) -> TerminalSnapshot {
        self.reap();
        let (cursor, active, truncated, events) = self
            .shared
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .snapshot(after);
        TerminalSnapshot {
            session_id: String::new(),
            cursor,
            active,
            truncated,
            cwd: self.cwd.clone(),
            shell: self.shell.clone(),
            pid: self.child.process_id(),
            events,
        }
    }

    pub(crate) fn shutdown(&mut self) -> bool {
        if self.stopped.load(Ordering::Relaxed) {
            return matches!(self.child.try_wait(), Ok(Some(_)));
        }
        self.stopped.store(true, Ordering::Relaxed);
        if let Some(master) = self
            .master
            .get_mut()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
        {
            #[cfg(unix)]
            if let Some(group) = master.process_group_leader() {
                let group = format!("-{group}");
                for signal in ["TERM", "KILL"] {
                    let _ = Command::new("kill")
                        .args([signal, "--", &group])
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status();
                }
            }
        }
        let _ = self.child.kill();
        self.writer
            .get_mut()
            .unwrap_or_else(|error| error.into_inner())
            .take();
        self.master
            .get_mut()
            .unwrap_or_else(|error| error.into_inner())
            .take();
        let deadline = Instant::now() + Duration::from_millis(250);
        let reaped = loop {
            match self.child.try_wait() {
                Ok(Some(_)) => break true,
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(1))
                }
                Ok(None) => break false,
                Err(_) => break true,
            }
        };
        if let Some(reader) = self.reader.take() {
            if reader.is_finished() {
                let _ = reader.join();
            }
        }
        self.shared
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .active = false;
        reaped
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl Drop for DesktopBackend {
    fn drop(&mut self) {
        for (_, session) in self.terminal_sessions.iter_mut() {
            session.shutdown();
        }
    }
}

fn canonical_shell(shell: &Path) -> Result<PathBuf, DesktopClientError> {
    let canonical = std::fs::canonicalize(shell).map_err(|_| {
        DesktopClientError::InvalidInput(format!("shell is not executable: {}", shell.display()))
    })?;
    if !canonical.is_file()
        || ![
            Path::new("/bin/zsh"),
            Path::new("/bin/sh"),
            Path::new("/bin/bash"),
        ]
        .contains(&canonical.as_path())
    {
        return Err(DesktopClientError::InvalidInput(format!(
            "shell is not allowed: {}",
            shell.display()
        )));
    }
    Ok(canonical)
}

impl DesktopBackend {
    pub fn set_terminal_home(&mut self, home: &Path) -> Result<(), DesktopClientError> {
        self.terminal_home = Some(self.verify_granted_existing_path(home)?);
        Ok(())
    }

    pub fn host_approve_terminal_shell(
        &mut self,
        cwd: &Path,
        shell: &Path,
    ) -> Result<(), DesktopClientError> {
        if !self.terminal_enabled {
            return Err(DesktopClientError::TerminalDisabledByDefault);
        }
        let cwd = self.verify_granted_existing_path(cwd)?;
        let shell = canonical_shell(shell)?;
        self.host_approved_shells.insert((cwd, shell));
        Ok(())
    }

    pub fn open_terminal(
        &mut self,
        cwd: &Path,
        shell: Option<&Path>,
        rows: u16,
        cols: u16,
    ) -> Result<TerminalSnapshot, DesktopClientError> {
        if !self.terminal_enabled {
            return Err(DesktopClientError::TerminalDisabledByDefault);
        }
        if self.terminal_sessions.len() >= MAX_TERMINAL_SESSIONS {
            return Err(DesktopClientError::SessionLimitReached);
        }
        if !(2..=200).contains(&rows) || !(2..=500).contains(&cols) {
            return Err(DesktopClientError::InvalidInput(
                "terminal dimensions are out of bounds".to_string(),
            ));
        }
        let cwd = self.verify_granted_existing_path(cwd)?;
        let shell = canonical_shell(shell.unwrap_or_else(|| Path::new("/bin/zsh")))?;
        if !self
            .host_approved_shells
            .remove(&(cwd.clone(), shell.clone()))
        {
            return Err(DesktopClientError::TerminalConsentRequired);
        }
        let (id, session) = TerminalSession::spawn(self, &cwd, &shell, rows, cols)?;
        self.terminal_sessions.insert(id.clone(), session);
        self.read_terminal(&id, 0)
    }

    pub fn read_terminal(
        &mut self,
        session_id: &str,
        after: u64,
    ) -> Result<TerminalSnapshot, DesktopClientError> {
        let session = self
            .terminal_sessions
            .get_mut(session_id)
            .ok_or_else(|| DesktopClientError::SessionNotFound(session_id.to_string()))?;
        let mut snapshot = session.snapshot(after);
        snapshot.session_id = session_id.to_string();
        for event in &mut snapshot.events {
            match event {
                TerminalEvent::Output { session_id: id, .. }
                | TerminalEvent::Exit { session_id: id, .. }
                | TerminalEvent::Error { session_id: id, .. } => *id = session_id.to_string(),
            }
        }
        Ok(snapshot)
    }

    pub fn write_terminal(
        &mut self,
        session_id: &str,
        input: &[u8],
    ) -> Result<(), DesktopClientError> {
        if input.len() > MAX_TERMINAL_INPUT_BYTES {
            return Err(DesktopClientError::InvalidInput(
                "terminal input exceeds 64 KiB".to_string(),
            ));
        }
        let session = self
            .terminal_sessions
            .get_mut(session_id)
            .ok_or_else(|| DesktopClientError::SessionNotFound(session_id.to_string()))?;
        if session.stopped.load(Ordering::Relaxed) {
            return Err(DesktopClientError::SessionNotFound(session_id.to_string()));
        }
        let mut guard = session
            .writer
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let writer = guard
            .as_mut()
            .ok_or_else(|| DesktopClientError::SessionNotFound(session_id.to_string()))?;
        writer
            .write_all(input)
            .map_err(|error| DesktopClientError::CommandFailed(error.to_string()))?;
        writer
            .flush()
            .map_err(|error| DesktopClientError::CommandFailed(error.to_string()))
    }

    pub fn resize_terminal(
        &mut self,
        session_id: &str,
        rows: u16,
        cols: u16,
    ) -> Result<(), DesktopClientError> {
        if !(2..=200).contains(&rows) || !(2..=500).contains(&cols) {
            return Err(DesktopClientError::InvalidInput(
                "terminal dimensions are out of bounds".to_string(),
            ));
        }
        let session = self
            .terminal_sessions
            .get_mut(session_id)
            .ok_or_else(|| DesktopClientError::SessionNotFound(session_id.to_string()))?;
        if session.stopped.load(Ordering::Relaxed) {
            return Err(DesktopClientError::SessionNotFound(session_id.to_string()));
        }
        session
            .master
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
            .ok_or_else(|| DesktopClientError::SessionNotFound(session_id.to_string()))?
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| DesktopClientError::CommandFailed(error.to_string()))
    }

    pub fn close_terminal(
        &mut self,
        session_id: &str,
    ) -> Result<TerminalSnapshot, DesktopClientError> {
        let mut session = self
            .terminal_sessions
            .remove(session_id)
            .ok_or_else(|| DesktopClientError::SessionNotFound(session_id.to_string()))?;
        let mut snapshot = session.snapshot(u64::MAX);
        if !session.shutdown() {
            self.terminal_sessions
                .insert(session_id.to_string(), session);
            return Err(DesktopClientError::CommandFailed(format!(
                "PTY did not exit cleanly: {session_id}"
            )));
        }
        snapshot.session_id = session_id.to_string();
        snapshot.active = false;
        snapshot.events.clear();
        Ok(snapshot)
    }
}
