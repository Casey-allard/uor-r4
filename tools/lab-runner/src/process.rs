//! Process identity and bounded signalling. A bare PID is never authority.
use crate::{invalid, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Identity {
    pub pid: u32,
    pub pgid: u32,
    pub started: String,
    pub boot: String,
    /// New identities bind the physical host. Missing legacy host fields need
    /// a separate local-host receipt before even a stopped proof is accepted.
    #[serde(default)]
    pub host: String,
    pub token: String,
    /// The persistent supervisor anchors process-group membership. Legacy
    /// identities without this field fail closed instead of being adopted.
    #[serde(default)]
    pub supervisor_started: String,
}

pub const SUPERVISOR_MARKER: &str = "uor-owned-supervisor-v1";

fn output(program: &str, args: &[&str]) -> Result<String> {
    let out = Command::new(program)
        .args(args)
        .env("LC_ALL", "C")
        .output()?;
    if !out.status.success() {
        return Err(invalid(format!("process identity query failed: {program}")));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

pub fn host_state_dir() -> Result<std::path::PathBuf> {
    let home =
        std::env::var_os("HOME").ok_or_else(|| invalid("HOME unavailable for host state"))?;
    Ok(std::path::PathBuf::from(home).join(".local/share/uor-r4"))
}

/// Stable pseudonymous host identity; unrelated to PID or boot identity.
pub fn host_id() -> Result<String> {
    use sha2::{Digest, Sha256};
    static HOST: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    if let Some(host) = HOST.get() {
        return Ok(host.clone());
    }
    #[cfg(target_os = "macos")]
    let raw = {
        let text = output("/usr/sbin/ioreg", &["-rd1", "-c", "IOPlatformExpertDevice"])?;
        text.lines()
            .find_map(|line| {
                let (key, value) = line.split_once('=')?;
                (key.trim() == "\"IOPlatformUUID\"")
                    .then(|| value.trim().trim_matches('"').to_string())
            })
            .filter(|s| !s.is_empty())
            .ok_or_else(|| invalid("stable host identity unavailable"))?
    };
    #[cfg(not(target_os = "macos"))]
    let raw = std::fs::read_to_string("/etc/machine-id")?
        .trim()
        .to_string();
    if raw.is_empty() {
        return Err(invalid("stable host identity empty"));
    }
    let host = format!("host-{:x}", Sha256::digest(raw.as_bytes()));
    let _ = HOST.set(host.clone());
    Ok(host)
}

const MACOS_BOOT_SCHEME: &str = "macos-kern.bootsessionuuid-v1:";
const LINUX_BOOT_SCHEME: &str = "linux-proc-boot-id-v1:";

/// Only UUIDs in an explicitly named stable platform scheme can establish
/// boot equality or a reboot. Historical kern.boottime strings and untagged
/// IDs remain readable, but never authorize adoption or signalling.
fn uuid(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    if bytes.len() != 36
        || !bytes.iter().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                *b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
        || bytes.iter().all(|b| *b == b'0' || *b == b'-')
    {
        return None;
    }
    Some(value.to_ascii_lowercase())
}

fn boot_token(scheme: &str, value: &str) -> Result<String> {
    if ![MACOS_BOOT_SCHEME, LINUX_BOOT_SCHEME].contains(&scheme) {
        return Err(invalid("unsupported boot identity scheme"));
    }
    Ok(format!(
        "{scheme}{}",
        uuid(value.trim()).ok_or_else(|| invalid("stable boot UUID unavailable or malformed"))?
    ))
}

#[derive(Debug, PartialEq, Eq)]
enum BootRelation {
    Same,
    Different,
    Unverified,
}

fn boot_relation(saved: &str, current: &str) -> BootRelation {
    for scheme in [MACOS_BOOT_SCHEME, LINUX_BOOT_SCHEME] {
        if let (Some(a), Some(b)) = (saved.strip_prefix(scheme), current.strip_prefix(scheme)) {
            if let (Some(a), Some(b)) = (uuid(a), uuid(b)) {
                return if a == b {
                    BootRelation::Same
                } else {
                    BootRelation::Different
                };
            }
        }
    }
    BootRelation::Unverified
}

fn boot() -> Result<String> {
    #[cfg(target_os = "macos")]
    {
        boot_token(
            MACOS_BOOT_SCHEME,
            &output("/usr/sbin/sysctl", &["-n", "kern.bootsessionuuid"])?,
        )
    }
    #[cfg(target_os = "linux")]
    {
        boot_token(
            LINUX_BOOT_SCHEME,
            &std::fs::read_to_string("/proc/sys/kernel/random/boot_id")?,
        )
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Err(invalid("stable boot identity unsupported on this platform"))
    }
}

pub fn capture(pid: u32, token: &str) -> Result<Identity> {
    if pid < 2 || token.len() < 8 {
        return Err(invalid("invalid owned process identity"));
    }
    let pid_text = pid.to_string();
    let started = output("/bin/ps", &["-p", &pid_text, "-o", "lstart="])?;
    if started.is_empty() {
        return Err(invalid("process start identity unavailable"));
    }
    let pgid: u32 = output("/bin/ps", &["-p", &pid_text, "-o", "pgid="])?
        .parse()
        .map_err(|_| invalid("process group unavailable"))?;
    let supervisor_started = output("/bin/ps", &["-p", &pgid.to_string(), "-o", "lstart="])?;
    let identity = Identity {
        pid,
        pgid,
        started,
        boot: boot()?,
        host: host_id()?,
        token: token.to_string(),
        supervisor_started,
    };
    if !matches(&identity) {
        return Err(invalid("process ownership token unavailable"));
    }
    Ok(identity)
}

/// A persistent supervisor retains its nonce in argv because macOS can omit
/// process environments even for owned children. The nonce is an ownership
/// marker, not a secret. An absent supervisor never authorizes a live child.
pub fn matches(identity: &Identity) -> bool {
    let pid = identity.pid.to_string();
    if host_id().ok().as_ref() != Some(&identity.host)
        || !boot()
            .is_ok_and(|current| boot_relation(&identity.boot, &current) == BootRelation::Same)
    {
        return false;
    }
    if output("/bin/ps", &["-p", &pid, "-o", "lstart="])
        .ok()
        .as_ref()
        != Some(&identity.started)
    {
        return false;
    }
    if output("/bin/ps", &["-p", &pid, "-o", "pgid="])
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        != Some(identity.pgid)
    {
        return false;
    }
    let supervisor = identity.pgid.to_string();
    if identity.supervisor_started.is_empty()
        || output("/bin/ps", &["-p", &supervisor, "-o", "lstart="])
            .ok()
            .as_ref()
            != Some(&identity.supervisor_started)
    {
        return false;
    }
    let Ok(arguments) = output("/bin/ps", &["-ww", "-p", &supervisor, "-o", "command="]) else {
        return false;
    };
    let fields: Vec<_> = arguments.split_whitespace().collect();
    fields
        .windows(2)
        .any(|pair| pair == [SUPERVISOR_MARKER, identity.token.as_str()])
}

/// Verified members remain discoverable after their parent exits. Process
/// group membership alone is insufficient: its supervisor must retain the
/// nonce. Surviving children whose supervisor has vanished remain unknown.
pub fn owned_members(identity: &Identity) -> Result<Vec<Identity>> {
    if identity.host != host_id()? {
        return Err(invalid(
            "process identity lacks this host binding; no adoption or signal authority",
        ));
    }
    let relation = boot_relation(&identity.boot, &boot()?);
    members_with_relation(identity, relation)
}

/// A legacy receipt cannot be adopted or signalled. Its immutable exit's
/// same-host binding can instead authorize a fresh, strict absence observation.
/// The old boot string is never upgraded or treated as proof of reboot.
pub fn confirmed_stopped(identity: &Identity, recorded_host: &str) -> Result<bool> {
    if recorded_host != host_id()? {
        return Err(invalid("stopped proof belongs to another or unknown host"));
    }
    if identity.host.is_empty() {
        Ok(members_with_relation(identity, BootRelation::Unverified)?.is_empty())
    } else {
        Ok(owned_members(identity)?.is_empty())
    }
}

fn members_with_relation(identity: &Identity, relation: BootRelation) -> Result<Vec<Identity>> {
    if identity.pid < 2 || identity.pgid < 2 || identity.token.len() < 8 {
        return Err(invalid(
            "malformed process identity cannot establish a stopped group",
        ));
    }
    if relation == BootRelation::Different {
        // A verified reboot on this host cannot retain a prior boot's process.
        // Callers still bind the identity receipt to the same host and attempt.
        return Ok(Vec::new());
    }
    let table = output("/bin/ps", &["-Ao", "pid=,pgid=,stat="])?;
    let rows = process_rows(&table)?;
    let mut members = Vec::new();
    for (pid, pgid, zombie) in rows {
        // Zombies cannot execute or spawn. Reaping the daemon's Child is a
        // separate responsibility and must not be lost when recording UNKNOWN.
        if zombie {
            continue;
        }
        if relation == BootRelation::Unverified && (pgid == identity.pgid || pid == identity.pid) {
            return Err(invalid("legacy or malformed boot identity has a live process; no adoption or signal authority"));
        }
        if pgid == identity.pgid {
            match capture(pid, &identity.token) {
                Ok(member)
                    if member.pgid == identity.pgid
                        && member.supervisor_started == identity.supervisor_started
                        && boot_relation(&identity.boot, &member.boot) == BootRelation::Same =>
                {
                    members.push(member)
                }
                _ => {
                    // Exiting between enumeration and capture is normal, but
                    // inaccessible/changed ownership is not proof of absence.
                    let state = Command::new("/bin/ps")
                        .args(["-p", &pid.to_string(), "-o", "pgid=,stat="])
                        .output()?;
                    let text = String::from_utf8_lossy(&state.stdout);
                    let fields: Vec<_> = text.split_whitespace().collect();
                    let gone = state.status.code() == Some(1)
                        && fields.is_empty()
                        && state.stderr.is_empty();
                    let zombie =
                        state.status.success() && fields.get(1).is_some_and(|s| s.starts_with('Z'));
                    if !gone && !zombie {
                        return Err(invalid("live process-group member ownership is unavailable; stop state is unknown"));
                    }
                }
            }
        }
    }
    Ok(members)
}

fn process_rows(table: &str) -> Result<Vec<(u32, u32, bool)>> {
    let mut rows = Vec::new();
    for line in table.lines().filter(|line| !line.trim().is_empty()) {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() != 3 {
            return Err(invalid("incomplete process table; stop state is unknown"));
        }
        let pid = fields[0]
            .parse::<u32>()
            .map_err(|_| invalid("invalid process table PID"))?;
        let pgid = fields[1]
            .parse::<u32>()
            .map_err(|_| invalid("invalid process table group"))?;
        rows.push((pid, pgid, fields[2].starts_with('Z')));
    }
    if rows.is_empty() {
        return Err(invalid("empty process table; stop state is unknown"));
    }
    Ok(rows)
}

/// Recover the actual persistent supervisor, never a bare PID, when a crash
/// occurred between spawn and durable process registration. No args are logged.
pub fn find_supervisor(token: &str) -> Result<Option<Identity>> {
    if token.len() < 8 {
        return Err(invalid("invalid process recovery nonce"));
    }
    let table = output("/bin/ps", &["-Aww", "-o", "pid=,pgid=,command="])?;
    let mut found = None;
    for line in table.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() < 4 || fields[0] != fields[1] {
            continue;
        }
        if fields[2..]
            .windows(2)
            .any(|pair| pair == [SUPERVISOR_MARKER, token])
        {
            let pid = fields[0]
                .parse()
                .map_err(|_| invalid("invalid supervisor PID"))?;
            let identity = capture(pid, token)?;
            if found.replace(identity).is_some() {
                return Err(invalid("multiple supervisors have the recovery nonce"));
            }
        }
    }
    Ok(found)
}

/// Revalidate each PID/start/boot/group/nonce before every signal. Re-enumerate
/// after TERM so a last-moment child is covered, without signalling a bare PGID.
pub fn terminate(identity: &Identity, grace: Duration) -> Result<()> {
    let identities = owned_members(identity)?;
    if identities.is_empty() {
        return Err(invalid("process identity mismatch; no signal sent"));
    }
    // Retain the nonce-bearing anchor throughout descendant termination.
    // Otherwise a TERM-ignoring child would become unowned before KILL.
    for item in identities.iter().filter(|item| item.pid != identity.pgid) {
        if matches(item) {
            let _ = Command::new("/bin/kill")
                .args(["-TERM", &item.pid.to_string()])
                .status();
        }
    }
    std::thread::sleep(grace);
    let remaining = owned_members(identity)?;
    for item in remaining.iter().filter(|item| item.pid != identity.pgid) {
        if matches(item) {
            let _ = Command::new("/bin/kill")
                .args(["-KILL", &item.pid.to_string()])
                .status();
        }
    }
    for _ in 0..20 {
        let members = owned_members(identity)?;
        if members.iter().all(|item| item.pid == identity.pgid) {
            if let Some(anchor) = members.first() {
                if !matches(anchor) {
                    return Err(invalid("supervisor identity changed before stop"));
                }
                let _ = Command::new("/bin/kill")
                    .args(["-TERM", &anchor.pid.to_string()])
                    .status();
            }
            for _ in 0..20 {
                if owned_members(identity)?.is_empty() {
                    return Ok(());
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            return Err(invalid("supervisor remained after bounded termination"));
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    Err(invalid(
        "owned process still present after bounded termination",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn term_handlers_get_bounded_grace_without_claiming_checkpoint_success() {
        use std::os::unix::process::CommandExt;
        for writes_checkpoint in [false, true] {
            let root = std::env::temp_dir().join(format!(
                "uor-stop-proof-{}-{}",
                std::process::id(),
                crate::jobs::now_ms()
            ));
            std::fs::create_dir(&root).unwrap();
            let fifo = root.join("hold.fifo");
            let ready = root.join("ready");
            let checkpoint = root.join("checkpoint");
            assert!(Command::new("/usr/bin/mkfifo")
                .arg(&fifo)
                .status()
                .unwrap()
                .success());
            let token = format!("stop-proof-{}-{writes_checkpoint}", std::process::id());
            let payload = if writes_checkpoint {
                "trap 'printf saved > \"$2\"; exit 0' TERM; printf ready > \"$1\"; /bin/sleep 10"
            } else {
                "trap '' TERM; printf ready > \"$1\"; /bin/sleep 10"
            };
            let mut child = Command::new("/bin/sh").args(["-c",
            "shift; hold=$1; shift; \"$@\" & child=$!; wait \"$child\"; exec 3<> \"$hold\"; IFS= read -r ack <&3",
            SUPERVISOR_MARKER, &token]).arg(&fifo)
            .args(["/bin/sh", "-c", payload, "payload"])
            .arg(&ready).arg(&checkpoint).process_group(0).spawn().unwrap();
            let deadline = std::time::Instant::now() + Duration::from_secs(2);
            while !ready.exists() && std::time::Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(10));
            }
            let identity = capture(child.id(), &token).unwrap();
            let result = terminate(&identity, Duration::from_millis(500));
            if result.is_err() {
                let _ = child.kill();
            }
            let _ = child.wait();
            result.unwrap();
            assert!(owned_members(&identity).unwrap().is_empty());
            assert_eq!(checkpoint.exists(), writes_checkpoint);
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn verified_prior_boot_has_no_surviving_process_members() {
        let current = boot().unwrap();
        let mut prior = current.clone();
        let last = prior.pop().unwrap();
        prior.push(if last == '0' { '1' } else { '0' });
        assert_eq!(boot_relation(&prior, &current), BootRelation::Different);
        let identity = Identity {
            pid: 2,
            pgid: 2,
            started: "unused".into(),
            boot: prior,
            host: host_id().unwrap(),
            token: "prior-boot-fixture".into(),
            supervisor_started: "unused".into(),
        };
        assert!(owned_members(&identity).unwrap().is_empty());
        assert!(!matches(&identity));
    }

    #[test]
    fn stable_boot_schemes_validate_uuid_and_never_promote_legacy_time_drift() {
        let id = "763E5ACF-75D5-44F0-9384-F80B31312894";
        for scheme in [MACOS_BOOT_SCHEME, LINUX_BOOT_SCHEME] {
            let stable = boot_token(scheme, id).unwrap();
            assert_eq!(
                boot_relation(&stable, &stable.to_ascii_lowercase()),
                BootRelation::Same
            );
            for legacy in [
                "{ sec = 1790723628, usec = 114456 } Tue Sep 29 19:13:48 2026",
                "{ sec = 1790723628, usec = 41370 } Tue Sep 29 19:13:48 2026",
                id,
                "prior-boot-fixture",
            ] {
                assert_eq!(boot_relation(legacy, &stable), BootRelation::Unverified);
            }
            assert!(boot_token(scheme, "00000000-0000-0000-0000-000000000000").is_err());
            assert!(boot_token(scheme, "763E5ACF-75D5-44F0-9384-F80B3131289Z").is_err());
            assert_eq!(
                boot_relation(&format!("{stable}-different"), &stable),
                BootRelation::Unverified
            );
        }
        assert_eq!(
            boot_relation(
                &boot_token(MACOS_BOOT_SCHEME, id).unwrap(),
                &boot_token(LINUX_BOOT_SCHEME, id).unwrap()
            ),
            BootRelation::Unverified
        );
        assert!(process_rows("1 1 Ss\ninvalid row").is_err());
        assert!(process_rows("").is_err());
        assert_eq!(
            process_rows("123 123 Z\n1 1 Ss").unwrap(),
            vec![(123, 123, true), (1, 1, false)]
        );
    }

    #[test]
    fn legacy_live_identity_cannot_signal_but_same_host_absence_can_be_reconciled() {
        use std::os::unix::process::CommandExt;
        let token = format!("legacy-identity-test-{}", std::process::id());
        let mut child = Command::new("/bin/sh")
            .args([
                "-c",
                "shift; \"$@\" & child=$!; wait \"$child\"",
                SUPERVISOR_MARKER,
                &token,
                "/bin/sleep",
                "10",
            ])
            .process_group(0)
            .spawn()
            .unwrap();
        let original = capture(child.id(), &token).unwrap();
        let mut legacy = original.clone();
        legacy.host.clear();
        legacy.boot = "{ sec = 1790723628, usec = 114456 }".into();
        assert!(!matches(&legacy));
        assert!(owned_members(&legacy).is_err());
        assert!(terminate(&legacy, Duration::ZERO).is_err());
        assert!(confirmed_stopped(&legacy, &host_id().unwrap()).is_err());
        assert!(child.try_wait().unwrap().is_none());
        let mut foreign = original.clone();
        foreign.host = "another-host".into();
        assert!(owned_members(&foreign).is_err());
        let mut reused = original.clone();
        reused.supervisor_started = "different process at the same PID".into();
        assert!(!matches(&reused));
        assert!(owned_members(&reused).is_err());
        assert!(terminate(&reused, Duration::ZERO).is_err());
        assert!(child.try_wait().unwrap().is_none());
        terminate(&original, Duration::from_millis(50)).unwrap();
        child.wait().unwrap();
        assert!(confirmed_stopped(&legacy, &host_id().unwrap()).unwrap());
        assert!(confirmed_stopped(&legacy, "another-host").is_err());
    }

    #[test]
    fn nonce_mismatch_never_signals_the_process() {
        let token = format!("identity-test-{}", std::process::id());
        use std::os::unix::process::CommandExt;
        let mut child = Command::new("/bin/sh")
            .args([
                "-c",
                "shift; \"$@\" & child=$!; wait \"$child\"; status=$?; exit \"$status\"",
                SUPERVISOR_MARKER,
                &token,
                "/bin/sleep",
                "10",
            ])
            .process_group(0)
            .spawn()
            .unwrap();
        let mut identity = capture(child.id(), &token).unwrap();
        identity.token.push_str("-wrong");
        assert!(!matches(&identity));
        assert!(terminate(&identity, Duration::ZERO).is_err());
        assert!(child.try_wait().unwrap().is_none());
        identity.token = token;
        terminate(&identity, Duration::from_millis(50)).unwrap();
        child.wait().unwrap();
    }
}
