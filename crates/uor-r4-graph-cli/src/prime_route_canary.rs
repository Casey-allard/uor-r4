//! Bounded release runner and terminal-report publisher for the #958
//! one-worker/four-worker prime-route canary.

use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use uor_r4_graph_certify::prime_route_worker_canary::{
    CANARY_HARD_WALL_MILLIS, CANARY_WATCHDOG_KILL_MILLIS, PrimeRouteCanaryProgress,
    PrimeRouteCanaryProgressState, PrimeRouteCanaryVerdict, PrimeRouteWorkerCanaryReport,
    run_prime_route_worker_canary, validate_prime_route_worker_canary_report,
};
use uor_r4_model_source::SourceUnavailable;

const TERMINAL_SCHEMA: u32 = 2;
const TERMINAL_DOMAIN: &str = "uor-r4.prime-route-worker-canary-terminal/2";
const BUILD_PROFILE: &str = env!("UOR_R4_GRAPH_CLI_BUILD_PROFILE");
const BUILD_OPT_LEVEL: &str = env!("UOR_R4_GRAPH_CLI_OPT_LEVEL");
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(1);
const CHILD_POLL_INTERVAL: Duration = Duration::from_millis(100);
const MAX_CHILD_REPORT_BYTES: usize = 64 * 1024 * 1024;
const HARD_WALL_EXIT_CODE: i32 = 124;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum TerminalVerdict {
    Pass,
    OptimizeBeforeLongRun,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TerminalHost {
    release_build: bool,
    build_profile: String,
    opt_level: String,
    target_arch: String,
    available_parallelism: usize,
    binary_cid: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PrimeRouteCanaryTerminal {
    schema: u32,
    domain: String,
    verdict: TerminalVerdict,
    reason: String,
    hard_wall_millis: u64,
    watchdog_kill_millis: u64,
    wall_elapsed_millis: u64,
    progress_events_emitted: usize,
    host: TerminalHost,
    certifier: Option<PrimeRouteWorkerCanaryReport>,
}

impl PrimeRouteCanaryTerminal {
    fn from_report(
        report: PrimeRouteWorkerCanaryReport,
        wall_elapsed: Duration,
        progress_events_emitted: usize,
    ) -> Self {
        let (verdict, reason) = match report.decision.verdict {
            PrimeRouteCanaryVerdict::Pass => (TerminalVerdict::Pass, "PASS".to_owned()),
            PrimeRouteCanaryVerdict::OptimizeBeforeLongRun => (
                TerminalVerdict::OptimizeBeforeLongRun,
                format!("{:?}", report.decision.failures),
            ),
        };
        Self {
            schema: TERMINAL_SCHEMA,
            domain: TERMINAL_DOMAIN.to_owned(),
            verdict,
            reason,
            hard_wall_millis: CANARY_HARD_WALL_MILLIS,
            watchdog_kill_millis: CANARY_WATCHDOG_KILL_MILLIS,
            wall_elapsed_millis: duration_millis(wall_elapsed),
            progress_events_emitted,
            host: terminal_host(),
            certifier: Some(report),
        }
    }

    fn optimize(reason: impl Into<String>, wall_elapsed: Duration) -> Self {
        Self {
            schema: TERMINAL_SCHEMA,
            domain: TERMINAL_DOMAIN.to_owned(),
            verdict: TerminalVerdict::OptimizeBeforeLongRun,
            reason: reason.into(),
            hard_wall_millis: CANARY_HARD_WALL_MILLIS,
            watchdog_kill_millis: CANARY_WATCHDOG_KILL_MILLIS,
            wall_elapsed_millis: duration_millis(wall_elapsed),
            progress_events_emitted: 0,
            host: terminal_host(),
            certifier: None,
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.schema != TERMINAL_SCHEMA || self.domain != TERMINAL_DOMAIN {
            return Err("child terminal report has an unsupported schema or domain".to_owned());
        }
        if self.hard_wall_millis != CANARY_HARD_WALL_MILLIS
            || self.watchdog_kill_millis != CANARY_WATCHDOG_KILL_MILLIS
        {
            return Err("child terminal report changed the frozen wall-clock contract".to_owned());
        }
        if let Some(report) = &self.certifier {
            validate_prime_route_worker_canary_report(report)
                .map_err(|error| format!("child certifier report is invalid: {error}"))?;
            let expected = match report.decision.verdict {
                PrimeRouteCanaryVerdict::Pass => TerminalVerdict::Pass,
                PrimeRouteCanaryVerdict::OptimizeBeforeLongRun => {
                    TerminalVerdict::OptimizeBeforeLongRun
                }
            };
            if self.verdict != expected {
                return Err("child and certifier verdicts disagree".to_owned());
            }
            if report.contract.hard_wall_millis != CANARY_HARD_WALL_MILLIS
                || report.contract.watchdog_kill_millis != CANARY_WATCHDOG_KILL_MILLIS
            {
                return Err("certifier report changed the frozen wall-clock contract".to_owned());
            }
        } else if self.verdict == TerminalVerdict::Pass {
            return Err("PASS terminal report is missing certifier evidence".to_owned());
        }
        let expected_release =
            release_configuration_is_valid(&self.host.build_profile, &self.host.opt_level);
        if self.host.build_profile != BUILD_PROFILE
            || self.host.opt_level != BUILD_OPT_LEVEL
            || self.host.release_build != expected_release
        {
            return Err("terminal host build metadata disagrees with this binary".to_owned());
        }
        if self.verdict == TerminalVerdict::Pass && !expected_release {
            return Err(
                "PASS terminal report was not produced by an optimized release build".to_owned(),
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Mode {
    Parent { report: PathBuf },
    InternalChild,
}

pub fn run(args: &[String]) -> Result<(), SourceUnavailable> {
    let mode = parse_mode(args).map_err(SourceUnavailable::new)?;
    let result = match mode {
        Mode::Parent { report } => run_parent(&report),
        Mode::InternalChild => run_child(),
    };
    result.map_err(SourceUnavailable::new)
}

fn parse_mode(args: &[String]) -> Result<Mode, String> {
    match args {
        [flag, value] if flag == "--report" => Ok(Mode::Parent {
            report: PathBuf::from(value),
        }),
        [flag] if flag == "--internal-child" => Ok(Mode::InternalChild),
        _ => Err(
            "usage: r4 transformerless prime-route-canary --report <fresh-path.json>".to_owned(),
        ),
    }
}

type SharedChild = Arc<Mutex<Option<Child>>>;

#[derive(Debug, Clone, Copy)]
struct ParentDeadlines {
    watchdog: Instant,
    hard_wall: Instant,
}

impl ParentDeadlines {
    fn from_start(started: Instant) -> Self {
        Self {
            watchdog: started + Duration::from_millis(CANARY_WATCHDOG_KILL_MILLIS),
            hard_wall: started + Duration::from_millis(CANARY_HARD_WALL_MILLIS),
        }
    }
}

struct ActiveHardWall {
    cancel: mpsc::Sender<()>,
    thread: Option<std::thread::JoinHandle<()>>,
    child: SharedChild,
}

impl ActiveHardWall {
    fn start(deadline: Instant, child: SharedChild) -> Self {
        let (cancel, receiver) = mpsc::channel();
        let guarded_child = Arc::clone(&child);
        let thread = std::thread::spawn(move || {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if matches!(
                receiver.recv_timeout(remaining),
                Err(mpsc::RecvTimeoutError::Timeout)
            ) {
                let kill_error = kill_live_child(&guarded_child).err();
                eprintln!(
                    "prime-route canary hard wall reached; terminating parent and child{}",
                    kill_error
                        .map(|error| format!("; child termination error: {error}"))
                        .unwrap_or_default()
                );
                std::process::exit(HARD_WALL_EXIT_CODE);
            }
        });
        Self {
            cancel,
            thread: Some(thread),
            child,
        }
    }

    fn disarm(mut self) {
        let _ = kill_live_child(&self.child);
        let _ = self.cancel.send(());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for ActiveHardWall {
    fn drop(&mut self) {
        let _ = self.cancel.send(());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn run_parent(report_path: &Path) -> Result<(), String> {
    let parent_started = Instant::now();
    let deadlines = ParentDeadlines::from_start(parent_started);
    let child = Arc::new(Mutex::new(None));
    let hard_wall = ActiveHardWall::start(deadlines.hard_wall, Arc::clone(&child));
    let result = run_parent_bounded(report_path, parent_started, deadlines, &child);
    hard_wall.disarm();
    result
}

fn run_parent_bounded(
    report_path: &Path,
    parent_started: Instant,
    deadlines: ParentDeadlines,
    child_slot: &SharedChild,
) -> Result<(), String> {
    ensure_fresh_destination(report_path)?;
    if !current_release_configuration_is_valid() {
        let terminal =
            PrimeRouteCanaryTerminal::optimize("RELEASE_BUILD_REQUIRED", parent_started.elapsed());
        publish_terminal_new(report_path, &terminal)?;
        return Err(format!(
            "prime-route canary requires an optimized release r4 binary; profile={BUILD_PROFILE}, opt-level={BUILD_OPT_LEVEL}"
        ));
    }
    let available = available_parallelism();
    if available < 4 {
        let terminal = PrimeRouteCanaryTerminal::optimize(
            format!("INSUFFICIENT_PARALLELISM: available={available}, required=4"),
            parent_started.elapsed(),
        );
        publish_terminal_new(report_path, &terminal)?;
        return Err(format!(
            "prime-route canary requires four available workers; host reports {available}"
        ));
    }

    let executable = match std::env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            return publish_parent_failure(
                report_path,
                parent_started,
                format!("CURRENT_EXECUTABLE_UNAVAILABLE: {error}"),
            );
        }
    };
    if Instant::now() >= deadlines.watchdog {
        return publish_parent_failure(
            report_path,
            parent_started,
            "WATCHDOG_BUDGET_EXHAUSTED_BEFORE_SPAWN".to_owned(),
        );
    }
    let spawned = match spawn_child(&executable) {
        Ok(spawned) => spawned,
        Err(error) => {
            return publish_parent_failure(
                report_path,
                parent_started,
                format!("CHILD_SPAWN_FAILED: {error}"),
            );
        }
    };
    {
        let mut child = lock_child(child_slot);
        *child = Some(spawned.child);
    }
    let child_status = wait_for_child(child_slot, deadlines.watchdog);

    let terminal = match child_status {
        ChildOutcome::Exited(status) => receive_child_terminal(spawned.output, deadlines.hard_wall)
            .and_then(|bytes| read_and_validate_child_terminal(&bytes, status))
            .unwrap_or_else(|error| {
                PrimeRouteCanaryTerminal::optimize(
                    format!("CHILD_TERMINAL_INVALID: {error}"),
                    parent_started.elapsed(),
                )
            }),
        ChildOutcome::TimedOut(kill_error) => PrimeRouteCanaryTerminal::optimize(
            kill_error
                .map(|error| format!("WATCHDOG_TIMEOUT; child termination error: {error}"))
                .unwrap_or_else(|| "WATCHDOG_TIMEOUT".to_owned()),
            parent_started.elapsed(),
        ),
        ChildOutcome::WaitFailed(error) => PrimeRouteCanaryTerminal::optimize(
            format!("CHILD_WAIT_FAILED: {error}"),
            parent_started.elapsed(),
        ),
    };

    let parent_elapsed = parent_started.elapsed();
    let mut terminal = terminal;
    terminal.wall_elapsed_millis = duration_millis(parent_elapsed);
    if parent_elapsed >= Duration::from_millis(CANARY_HARD_WALL_MILLIS) {
        terminal.verdict = TerminalVerdict::OptimizeBeforeLongRun;
        terminal.reason = "HARD_WALL_EXCEEDED_DURING_FINALIZATION".to_owned();
        terminal.certifier = None;
    }
    terminal.validate()?;
    publish_terminal_new(report_path, &terminal)?;

    println!(
        "prime-route worker canary: verdict={:?} report={} reason={}",
        terminal.verdict,
        report_path.display(),
        terminal.reason
    );
    match terminal.verdict {
        TerminalVerdict::Pass => Ok(()),
        TerminalVerdict::OptimizeBeforeLongRun => Err(format!(
            "prime-route worker canary did not authorize a longer run: {}",
            terminal.reason
        )),
    }
}

fn run_child() -> Result<(), String> {
    let started = Instant::now();
    if !current_release_configuration_is_valid() {
        let terminal =
            PrimeRouteCanaryTerminal::optimize("RELEASE_BUILD_REQUIRED", started.elapsed());
        write_child_terminal(&terminal)?;
        return Err("prime-route canary child requires an optimized release build".to_owned());
    }
    let progress_events = Arc::new(AtomicUsize::new(0));
    let progress = ProgressEmitter::start(Arc::clone(&progress_events));
    let result = run_prime_route_worker_canary(|event| progress.publish(event));
    progress.stop();

    let mut terminal = match result {
        Ok(report) => PrimeRouteCanaryTerminal::from_report(
            report,
            started.elapsed(),
            progress_events.load(Ordering::Acquire),
        ),
        Err(error) => PrimeRouteCanaryTerminal::optimize(
            format!("CERTIFIER_ERROR: {error}"),
            started.elapsed(),
        ),
    };
    terminal.progress_events_emitted = progress_events.load(Ordering::Acquire);
    terminal.validate()?;
    write_child_terminal(&terminal)?;
    match terminal.verdict {
        TerminalVerdict::Pass => Ok(()),
        TerminalVerdict::OptimizeBeforeLongRun => Err(format!(
            "prime-route canary child completed without authorization: {}",
            terminal.reason
        )),
    }
}

fn publish_parent_failure(
    report_path: &Path,
    started: Instant,
    reason: String,
) -> Result<(), String> {
    let terminal = PrimeRouteCanaryTerminal::optimize(reason.clone(), started.elapsed());
    publish_terminal_new(report_path, &terminal)?;
    Err(format!(
        "prime-route worker canary did not authorize a longer run: {reason}"
    ))
}

fn terminal_host() -> TerminalHost {
    TerminalHost {
        release_build: current_release_configuration_is_valid(),
        build_profile: BUILD_PROFILE.to_owned(),
        opt_level: BUILD_OPT_LEVEL.to_owned(),
        target_arch: std::env::consts::ARCH.to_owned(),
        available_parallelism: available_parallelism(),
        binary_cid: std::env::current_exe()
            .ok()
            .and_then(|path| fs::read(path).ok())
            .map(|bytes| format!("blake3:{}", blake3::hash(&bytes).to_hex())),
    }
}

fn current_release_configuration_is_valid() -> bool {
    release_configuration_is_valid(BUILD_PROFILE, BUILD_OPT_LEVEL)
}

fn release_configuration_is_valid(profile: &str, opt_level: &str) -> bool {
    profile == "release" && matches!(opt_level, "1" | "2" | "3" | "s" | "z")
}

fn available_parallelism() -> usize {
    std::thread::available_parallelism()
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(1)
}

struct ProgressEmitter {
    latest: Arc<Mutex<Option<PrimeRouteCanaryProgress>>>,
    emitted: Arc<AtomicUsize>,
    started: Instant,
    stop_sender: mpsc::Sender<()>,
    heartbeat: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl ProgressEmitter {
    fn start(emitted: Arc<AtomicUsize>) -> Self {
        let latest = Arc::new(Mutex::new(None));
        let (stop_sender, stop_receiver) = mpsc::channel();
        let heartbeat_latest = Arc::clone(&latest);
        let heartbeat_emitted = Arc::clone(&emitted);
        let started = Instant::now();
        let heartbeat = std::thread::spawn(move || {
            loop {
                match stop_receiver.recv_timeout(HEARTBEAT_INTERVAL) {
                    Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if let Ok(guard) = heartbeat_latest.lock()
                            && let Some(progress) = guard.as_ref()
                        {
                            emit_progress("HEARTBEAT", progress, started.elapsed());
                            heartbeat_emitted.fetch_add(1, Ordering::AcqRel);
                        }
                    }
                }
            }
        });
        Self {
            latest,
            emitted,
            started,
            stop_sender,
            heartbeat: Mutex::new(Some(heartbeat)),
        }
    }

    fn publish(&self, progress: &PrimeRouteCanaryProgress) {
        if let Ok(mut latest) = self.latest.lock() {
            *latest = Some(progress.clone());
        }
        emit_progress("BOUNDARY", progress, self.started.elapsed());
        self.emitted.fetch_add(1, Ordering::AcqRel);
    }

    fn stop(&self) {
        let _ = self.stop_sender.send(());
        if let Ok(mut heartbeat) = self.heartbeat.lock()
            && let Some(handle) = heartbeat.take()
        {
            let _ = handle.join();
        }
    }
}

#[derive(Serialize)]
struct ProgressLine<'a> {
    domain: &'static str,
    event: &'a str,
    state: PrimeRouteCanaryProgressState,
    phase: u8,
    repetition: usize,
    execution: usize,
    total_executions: usize,
    requested_workers: usize,
    completed_compilations: usize,
    total_compilations: usize,
    completed_transition_work: usize,
    total_transition_work: usize,
    elapsed_millis: u64,
    rate_milli_transitions_per_second: Option<u64>,
    eta_millis: Option<u64>,
}

fn emit_progress(event: &str, progress: &PrimeRouteCanaryProgress, elapsed: Duration) {
    let elapsed_millis = duration_millis(elapsed);
    let completed = progress.completed_transition_work;
    let rate_milli = if completed > 0 && elapsed_millis > 0 {
        u64::try_from(
            (completed as u128)
                .saturating_mul(1_000_000)
                .checked_div(u128::from(elapsed_millis))
                .unwrap_or(0),
        )
        .ok()
    } else {
        None
    };
    let eta_millis = match (completed, elapsed_millis) {
        (completed, elapsed_millis) if completed > 0 && elapsed_millis > 0 => u64::try_from(
            ((progress.total_transition_work.saturating_sub(completed)) as u128)
                .saturating_mul(u128::from(elapsed_millis))
                .checked_div(completed as u128)
                .unwrap_or(0),
        )
        .ok(),
        _ => None,
    };
    let line = ProgressLine {
        domain: "uor-r4.prime-route-worker-canary-progress/1",
        event,
        state: progress.state,
        phase: match progress.phase {
            uor_r4_graph_certify::prime_route_worker_canary::PrimeRouteCanaryPhase::Warmup => 0,
            uor_r4_graph_certify::prime_route_worker_canary::PrimeRouteCanaryPhase::Measured => 1,
        },
        repetition: progress.repetition,
        execution: progress.execution_index + 1,
        total_executions: progress.total_executions,
        requested_workers: progress.requested_workers,
        completed_compilations: progress.completed_compilations,
        total_compilations: progress.total_compilations,
        completed_transition_work: completed,
        total_transition_work: progress.total_transition_work,
        elapsed_millis,
        rate_milli_transitions_per_second: rate_milli,
        eta_millis,
    };
    match serde_json::to_string(&line) {
        Ok(serialized) => eprintln!("{serialized}"),
        Err(error) => eprintln!("prime-route canary progress serialization failed: {error}"),
    }
}

enum ChildOutcome {
    Exited(ExitStatus),
    TimedOut(Option<String>),
    WaitFailed(String),
}

struct SpawnedChild {
    child: Child,
    output: mpsc::Receiver<Result<Vec<u8>, String>>,
}

fn spawn_child(executable: &Path) -> Result<SpawnedChild, String> {
    let args = child_arguments();
    let mut child = Command::new(executable)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("spawn prime-route canary child: {error}"))?;
    let Some(stdout) = child.stdout.take() else {
        let _ = child.kill();
        return Err("prime-route canary child stdout pipe is unavailable".to_owned());
    };
    let (sender, output) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(read_bounded(stdout, MAX_CHILD_REPORT_BYTES));
    });
    Ok(SpawnedChild { child, output })
}

fn child_arguments() -> Vec<OsString> {
    vec![
        OsString::from("transformerless"),
        OsString::from("prime-route-canary"),
        OsString::from("--internal-child"),
    ]
}

fn wait_for_child(child_slot: &SharedChild, watchdog_deadline: Instant) -> ChildOutcome {
    loop {
        let status = {
            let mut slot = lock_child(child_slot);
            let Some(child) = slot.as_mut() else {
                return ChildOutcome::WaitFailed("child handle is unavailable".to_owned());
            };
            child.try_wait()
        };
        match status {
            Ok(Some(status)) => {
                lock_child(child_slot).take();
                return ChildOutcome::Exited(status);
            }
            Ok(None) if Instant::now() < watchdog_deadline => {
                let remaining = watchdog_deadline.saturating_duration_since(Instant::now());
                std::thread::sleep(CHILD_POLL_INTERVAL.min(remaining));
            }
            Ok(None) => {
                return ChildOutcome::TimedOut(kill_live_child(child_slot).err());
            }
            Err(error) => {
                let kill_error = kill_live_child(child_slot).err();
                return ChildOutcome::WaitFailed(
                    kill_error
                        .map(|kill| format!("{error}; child termination error: {kill}"))
                        .unwrap_or_else(|| error.to_string()),
                );
            }
        }
    }
}

fn lock_child(child: &SharedChild) -> std::sync::MutexGuard<'_, Option<Child>> {
    match child.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn kill_live_child(child_slot: &SharedChild) -> Result<(), String> {
    let mut slot = lock_child(child_slot);
    let Some(child) = slot.as_mut() else {
        return Ok(());
    };
    match child.try_wait() {
        Ok(Some(_)) => {
            slot.take();
            Ok(())
        }
        Ok(None) => child
            .kill()
            .map_err(|error| format!("kill prime-route canary child: {error}")),
        Err(wait_error) => child.kill().map_err(|kill_error| {
            format!(
                "inspect prime-route canary child before kill: {wait_error}; kill failed: {kill_error}"
            )
        }),
    }
}

fn receive_child_terminal(
    receiver: mpsc::Receiver<Result<Vec<u8>, String>>,
    hard_wall_deadline: Instant,
) -> Result<Vec<u8>, String> {
    let remaining = hard_wall_deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return Err("hard wall reached before child terminal output was available".to_owned());
    }
    receiver
        .recv_timeout(remaining)
        .map_err(|error| format!("receive child terminal output: {error}"))?
}

fn read_and_validate_child_terminal(
    bytes: &[u8],
    child_status: ExitStatus,
) -> Result<PrimeRouteCanaryTerminal, String> {
    if bytes.len() > MAX_CHILD_REPORT_BYTES {
        return Err("child terminal report exceeds its byte ceiling".to_owned());
    }
    let terminal: PrimeRouteCanaryTerminal = serde_json::from_slice(bytes)
        .map_err(|error| format!("decode child terminal report: {error}"))?;
    terminal.validate()?;
    let status_matches = match terminal.verdict {
        TerminalVerdict::Pass => child_status.success(),
        TerminalVerdict::OptimizeBeforeLongRun => !child_status.success(),
    };
    if !status_matches {
        return Err(format!(
            "child exit status {child_status} disagrees with terminal verdict {:?}",
            terminal.verdict
        ));
    }
    Ok(terminal)
}

fn read_bounded(reader: impl Read, maximum: usize) -> Result<Vec<u8>, String> {
    let limit = maximum
        .checked_add(1)
        .and_then(|value| u64::try_from(value).ok())
        .ok_or_else(|| "child terminal byte ceiling overflowed".to_owned())?;
    let mut bytes = Vec::new();
    reader
        .take(limit)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("read child terminal output: {error}"))?;
    if bytes.len() > maximum {
        return Err(format!("child terminal output exceeds {maximum} bytes"));
    }
    Ok(bytes)
}

fn write_child_terminal(terminal: &PrimeRouteCanaryTerminal) -> Result<(), String> {
    terminal.validate()?;
    let mut bytes = serde_json::to_vec(terminal)
        .map_err(|error| format!("serialize child terminal report: {error}"))?;
    bytes.push(b'\n');
    if bytes.len() > MAX_CHILD_REPORT_BYTES {
        return Err(format!(
            "child terminal output exceeds {} bytes",
            MAX_CHILD_REPORT_BYTES
        ));
    }
    let stdout = std::io::stdout();
    let mut output = stdout.lock();
    output
        .write_all(&bytes)
        .map_err(|error| format!("write child terminal output: {error}"))?;
    output
        .flush()
        .map_err(|error| format!("flush child terminal output: {error}"))
}

fn ensure_fresh_destination(path: &Path) -> Result<(), String> {
    if path.file_name().is_none() {
        return Err("terminal report path must name a file".to_owned());
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let metadata = fs::metadata(parent)
        .map_err(|error| format!("report parent {} is unavailable: {error}", parent.display()))?;
    if !metadata.is_dir() {
        return Err(format!(
            "report parent {} is not a directory",
            parent.display()
        ));
    }
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Ok(metadata) if metadata.file_type().is_symlink() => Err(format!(
            "refusing symlink report destination {}",
            path.display()
        )),
        Ok(_) => Err(format!(
            "refusing to overwrite existing report {}",
            path.display()
        )),
        Err(error) => Err(format!(
            "inspect report destination {}: {error}",
            path.display()
        )),
    }
}

fn publish_terminal_new(path: &Path, terminal: &PrimeRouteCanaryTerminal) -> Result<(), String> {
    terminal.validate()?;
    ensure_fresh_destination(path)?;
    let mut bytes = serde_json::to_vec_pretty(terminal)
        .map_err(|error| format!("serialize terminal report: {error}"))?;
    bytes.push(b'\n');
    publish_bytes_new(path, &bytes)
}

fn publish_bytes_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .ok_or_else(|| "report path has no file name".to_owned())?
        .to_string_lossy();
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::AcqRel);
    let temporary = parent.join(format!(
        ".{name}.prime-route-terminal-{}-{sequence}.tmp",
        std::process::id()
    ));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(|error| format!("create terminal temporary {}: {error}", temporary.display()))?;
    let publish_result = (|| {
        file.write_all(bytes)
            .map_err(|error| format!("write terminal temporary: {error}"))?;
        file.sync_all()
            .map_err(|error| format!("sync terminal temporary: {error}"))?;
        fs::hard_link(&temporary, path).map_err(|error| {
            format!(
                "atomically publish fresh terminal report {}: {error}",
                path.display()
            )
        })?;
        File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|error| format!("sync terminal report directory: {error}"))?;
        Ok(())
    })();
    drop(file);
    let _ = fs::remove_file(&temporary);
    publish_result
}

fn duration_millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_directory(label: &str) -> PathBuf {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::AcqRel);
        std::env::temp_dir().join(format!(
            "uor-r4-prime-route-canary-{label}-{}-{sequence}",
            std::process::id()
        ))
    }

    #[test]
    fn parser_accepts_only_public_or_hidden_exact_forms() {
        assert_eq!(
            parse_mode(&["--report".to_owned(), "result.json".to_owned()]),
            Ok(Mode::Parent {
                report: PathBuf::from("result.json")
            })
        );
        assert_eq!(
            parse_mode(&["--internal-child".to_owned()]),
            Ok(Mode::InternalChild)
        );
        assert!(parse_mode(&[]).is_err());
        assert!(parse_mode(&["--report".to_owned()]).is_err());
        assert!(parse_mode(&["--internal-child".to_owned(), "scratch.json".to_owned()]).is_err());
        assert!(parse_mode(&["--report".to_owned(), "a".to_owned(), "extra".to_owned()]).is_err());
    }

    #[test]
    fn child_command_is_exact_and_does_not_reenter_parent_mode() {
        let args = child_arguments();
        assert_eq!(
            args,
            vec![
                OsString::from("transformerless"),
                OsString::from("prime-route-canary"),
                OsString::from("--internal-child"),
            ]
        );
    }

    #[test]
    fn watchdog_and_hard_wall_share_the_parent_start() {
        let started = Instant::now();
        let deadlines = ParentDeadlines::from_start(started);
        assert_eq!(
            deadlines.watchdog.duration_since(started),
            Duration::from_millis(CANARY_WATCHDOG_KILL_MILLIS)
        );
        assert_eq!(
            deadlines.hard_wall.duration_since(started),
            Duration::from_millis(CANARY_HARD_WALL_MILLIS)
        );
        assert_eq!(
            deadlines.hard_wall.duration_since(deadlines.watchdog),
            Duration::from_millis(CANARY_HARD_WALL_MILLIS - CANARY_WATCHDOG_KILL_MILLIS)
        );
    }

    #[test]
    fn release_gate_requires_release_profile_and_optimization() {
        for optimized in ["1", "2", "3", "s", "z"] {
            assert!(release_configuration_is_valid("release", optimized));
        }
        assert!(!release_configuration_is_valid("release", "0"));
        assert!(!release_configuration_is_valid("debug", "3"));
        assert!(!release_configuration_is_valid("custom", "3"));
        assert!(!release_configuration_is_valid("release", "unknown"));
    }

    #[test]
    fn bounded_child_pipe_rejects_excess_without_filesystem_scratch() {
        assert_eq!(
            read_bounded(&b"1234"[..], 4).expect("bounded read"),
            b"1234"
        );
        assert!(read_bounded(&b"12345"[..], 4).is_err());
    }

    #[test]
    fn atomic_publication_is_fresh_and_never_overwrites() {
        let directory = test_directory("publish");
        fs::create_dir(&directory).expect("create test directory");
        let report = directory.join("terminal.json");
        let terminal = PrimeRouteCanaryTerminal::optimize("TEST", Duration::ZERO);
        publish_terminal_new(&report, &terminal).expect("first publication");
        let decoded: PrimeRouteCanaryTerminal =
            serde_json::from_slice(&fs::read(&report).expect("read terminal"))
                .expect("decode terminal");
        assert_eq!(decoded, terminal);
        assert!(publish_terminal_new(&report, &terminal).is_err());
        fs::remove_file(&report).expect("remove report");
        fs::remove_dir(&directory).expect("remove test directory");
    }

    #[cfg(unix)]
    #[test]
    fn symlink_destination_is_refused_without_touching_target() {
        use std::os::unix::fs::symlink;

        let directory = test_directory("symlink");
        fs::create_dir(&directory).expect("create test directory");
        let target = directory.join("target.json");
        fs::write(&target, b"untouched").expect("seed target");
        let link = directory.join("terminal.json");
        symlink(&target, &link).expect("create symlink");
        let terminal = PrimeRouteCanaryTerminal::optimize("TEST", Duration::ZERO);
        assert!(publish_terminal_new(&link, &terminal).is_err());
        assert_eq!(fs::read(&target).expect("read target"), b"untouched");
        fs::remove_file(&link).expect("remove link");
        fs::remove_file(&target).expect("remove target");
        fs::remove_dir(&directory).expect("remove test directory");
    }

    #[test]
    fn terminal_validation_rejects_pass_without_certifier_evidence() {
        let mut terminal = PrimeRouteCanaryTerminal::optimize("TEST", Duration::ZERO);
        terminal.verdict = TerminalVerdict::Pass;
        assert!(terminal.validate().is_err());
    }
}
