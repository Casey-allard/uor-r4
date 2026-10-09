"""One-use repair observer. No production mutation; final council packet required.

Scope: the inspected lab-runner tests may change process groups, but do not
create new sessions. Membership in our new OS session retains reparented children.
Limits are sampled; this is not containment for arbitrary daemonizing payloads.
"""
import datetime
import hashlib
import json
import os
import pathlib
import signal
import subprocess
import sys
import time

root = pathlib.Path(__file__).resolve().parent
packet_bytes = pathlib.Path(sys.argv[1]).read_bytes()
packet = json.loads(packet_bytes)
wt = pathlib.Path(packet['cwd'])
target = pathlib.Path(packet['target_dir'])
name = packet['receipt_directory']
if not isinstance(name, str) or pathlib.Path(name).name != name or name in ('', '.', '..'):
    raise RuntimeError('Receipt directory must be a fresh child name')
out = root / name
out.mkdir()  # Exclusive attempt: never overwrite a prior invocation.
production = pathlib.Path('/Users/casey.allard/.local/share/uor-r4/runner')
start = time.monotonic()
created = datetime.datetime.now(datetime.timezone.utc).isoformat()
results = []
failure = None
hold = boot = baseline = None
peak = 0
stop_state = 'never_started'
child = None
tracked = {}
session_id = None


def atomic_json(path, value):
    temp = path.with_name(path.name + '.tmp')
    with temp.open('x') as stream:
        json.dump(value, stream, indent=2)
        stream.write('\n')
        stream.flush()
        os.fsync(stream.fileno())
    os.replace(temp, path)
    fd = os.open(path.parent, os.O_RDONLY)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def probe(argv, *, cwd=None, timeout=3):
    return subprocess.run(argv, cwd=cwd, text=True, stdout=subprocess.PIPE,
                          stderr=subprocess.PIPE, timeout=timeout, check=True).stdout


def table(pid=None):
    args = ['/bin/ps', '-Aww'] if pid is None else ['/bin/ps', '-ww', '-p', str(pid)]
    args += ['-o', 'pid=,ppid=,rss=,lstart=,stat=,command=']
    try:
        raw = probe(args)
    except subprocess.CalledProcessError as error:
        if pid is not None and error.returncode == 1 and not error.stdout.strip():
            return {}
        raise
    rows = {}
    for line in raw.splitlines():
        f = line.split(None, 9)
        if len(f) != 10:
            raise RuntimeError('Incomplete process observation')
        process_id = int(f[0])
        try:
            sid = os.getsid(process_id)
        except ProcessLookupError:
            continue
        rows[process_id] = {
            'ppid': int(f[1]), 'rss': int(f[2]) * 1024,
            'start': ' '.join(f[3:8]), 'state': f[8], 'command': f[9], 'sid': sid,
        }
    return rows


def pressure():
    return int(probe(['/usr/sbin/sysctl', '-n', 'kern.memorystatus_vm_pressure_level']).strip())


def free():
    s = os.statvfs('/System/Volumes/Data')
    return s.f_bavail * s.f_frsize


def cache_size():
    return int(probe(['/usr/bin/du', '-sk', str(target)], timeout=5).split()[0]) * 1024


def boot_now():
    return probe(['/usr/sbin/sysctl', '-n', 'kern.bootsessionuuid']).strip()


def same(a, b):
    return a['start'] == b['start'] and a['command'] == b['command'] and a['sid'] == b['sid']


def observe():
    """Never silently discard a live former member when its identity changes."""
    if boot_now() != boot:
        raise RuntimeError('Boot identity changed or unavailable')
    current = table()
    for pid, old in list(tracked.items()):
        row = current.get(pid)
        if row is None or row['state'].startswith('Z'):
            continue
        if row['start'] != old['start']:
            raise RuntimeError(f'Observed PID reuse; stop authority is uncertain: {pid}')
        if row['sid'] != session_id:
            raise RuntimeError(f'Observed descendant escaped its session: {pid}')
        # Exec is allowed only with unchanged start and the owned OS session.
        tracked[pid] = row
    for pid, row in current.items():
        if row['sid'] == session_id:
            tracked[pid] = row
    live = {pid: row for pid, row in current.items()
            if pid in tracked and same(tracked[pid], row) and not row['state'].startswith('Z')}
    return current, live


def no_peer_cargo(current, own=()):
    for pid, row in current.items():
        if pid in own or row['state'].startswith('Z'):
            continue
        exe = row['command'].split(' ', 1)[0].rsplit('/', 1)[-1]
        if exe in ('cargo', 'rustc'):
            raise RuntimeError(f'Another Cargo/rustc is active: {pid}')


def check_source():
    if probe(['git', 'rev-parse', 'HEAD'], cwd=wt).strip() != packet['source_sha']:
        raise RuntimeError('Source HEAD changed')
    if probe(['git', 'status', '--porcelain', '--untracked-files=no'], cwd=wt):
        raise RuntimeError('Tracked source changed')


def guard(current, live, check_cache=False):
    global peak
    # Leave twenty seconds inside the projection for an observed bounded stop.
    if time.monotonic() - start >= packet['wall_seconds'] - 20:
        raise RuntimeError('Wall stop threshold (20-second stop allowance)')
    sampled = sum(row['rss'] for row in live.values())
    peak = max(peak, sampled)
    if sampled > packet['rss_bytes']:
        raise RuntimeError('Sampled aggregate RSS limit')
    if pressure() != 1:
        raise RuntimeError('Host memory pressure')
    if free() < packet['floor_bytes'] + packet['stop_margin_bytes']:
        raise RuntimeError('Physical free-space floor')
    if (production / 'admissions-held.json').read_bytes() != hold:
        raise RuntimeError('Production hold changed')
    if boot_now() != boot:
        raise RuntimeError('Boot identity changed')
    no_peer_cargo(current, live)
    if check_cache:
        check_source()
        if cache_size() - baseline > packet['new_cache_bytes']:
            raise RuntimeError('Cache growth limit')


def stop_owned():
    """Refresh membership for every signal; uncertain identity returns UNKNOWN."""
    errors = []
    for sig in (signal.SIGTERM, signal.SIGKILL):
        try:
            _, live = observe()
            # Root last, giving children a chance to be reaped by their parents.
            order = sorted(live, key=lambda pid: (pid == child.pid, -pid))
            for pid in order:
                if boot_now() != boot:
                    raise RuntimeError('Boot identity unavailable for signal')
                fresh = table(pid).get(pid)
                if fresh is None or fresh['state'].startswith('Z'):
                    continue
                if not same(tracked[pid], fresh):
                    raise RuntimeError(f'Identity changed before signal: {pid}')
                try:
                    os.kill(pid, sig)
                except ProcessLookupError:
                    pass
        except Exception as error:
            errors.append(type(error).__name__ + ': ' + str(error))
            # Popen owns its unreaped direct child, even if full observation failed.
            try:
                if boot_now() == boot and child.poll() is None:
                    child.send_signal(sig)
            except Exception as direct_error:
                errors.append('Direct-child stop: ' + str(direct_error))
        time.sleep(0.5)
    try:
        child.wait(timeout=3)
        _, live = observe()
        if live:
            raise RuntimeError('Owned live members remain: ' + ','.join(map(str, live)))
        # Recheck after reaping; never infer group exit solely from Cargo's exit.
        time.sleep(0.1)
        _, live = observe()
        if live:
            raise RuntimeError('Owned live members appeared after stop')
    except Exception as error:
        errors.append(type(error).__name__ + ': ' + str(error))
    return ('confirmed_stopped' if not errors else 'UNKNOWN'), errors


def available(fn):
    try:
        return fn()
    except Exception as error:
        return {'status': 'UNAVAILABLE', 'reason': type(error).__name__ + ': ' + str(error)}


atomic_json(out / 'intent.json', {
    'schema': 'uor-r4.manual-repair-intent/1', 'created_utc': created,
    'packet_sha256': hashlib.sha256(packet_bytes).hexdigest(), 'packet': packet,
    'observer_sha256': hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),
    'state': 'preflight; execution and stop outcomes not established',
})
try:
    if packet['execute_authorized'] is not True:
        raise RuntimeError('Exact council execution authorization required')
    if not target.is_dir() or target.resolve() != target:
        raise RuntimeError('Target path identity changed')
    check_source()
    hold = (production / 'admissions-held.json').read_bytes()
    boot = boot_now()
    if not hold or not boot:
        raise RuntimeError('Hold or boot identity unavailable')
    initial = table()
    no_peer_cargo(initial)
    if pressure() != 1:
        raise RuntimeError('Host memory pressure')
    if free() < packet['floor_bytes'] + packet['new_cache_bytes'] + packet['stop_margin_bytes']:
        raise RuntimeError('Insufficient projected physical space')
    baseline = cache_size()
    env = dict(os.environ, CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS='2')
    for index, argv in enumerate(packet['commands']):
        check_source()
        guard(table(), {}, check_cache=True)
        log = out / f'command-{index}.log'
        command_start = time.monotonic()
        tracked = {}
        abort = None
        result = {'argv': argv, 'log': str(log), 'process_state': 'UNKNOWN'}
        atomic_json(out / f'command-{index}-intent.json', result)
        with log.open('xb') as stream:
            child = subprocess.Popen(argv, cwd=wt, env=env, stdout=stream,
                                     stderr=subprocess.STDOUT, start_new_session=True)
            session_id = child.pid  # Guaranteed by start_new_session=True.
            stop_state = 'UNKNOWN'
            last_storage = 0
            try:
                atomic_json(out / f'command-{index}-launch.json', {
                    'pid': child.pid, 'session_id': session_id, 'boot_uuid': boot,
                    'observer_pid': os.getpid(), 'argv': argv,
                    'created_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
                    'state': 'launched; exit and stopped state not established',
                })
                first, _ = observe()
                atomic_json(out / f'command-{index}-identity.json', {
                    'pid': child.pid, 'session_id': session_id, 'boot_uuid': boot,
                    'observation': first.get(child.pid, {'status': 'UNAVAILABLE', 'reason': 'direct child absent at first observation'}),
                })
                while True:
                    current, live = observe()
                    elapsed = time.monotonic() - start
                    check_cache = elapsed - last_storage >= 10
                    guard(current, live, check_cache=check_cache)
                    if check_cache:
                        last_storage = elapsed
                    status = child.poll()
                    if status is not None and not live:
                        # A second observation prevents treating parent exit as absence.
                        time.sleep(0.1)
                        _, remaining = observe()
                        if not remaining:
                            stop_state = 'confirmed_stopped'
                            break
                    time.sleep(0.5)
            except BaseException as error:
                abort = type(error).__name__ + ': ' + str(error)
                failure = abort
                stop_state, errors = stop_owned()
                if errors:
                    failure += '; stop UNKNOWN: ' + '; '.join(errors)
            stream.flush()
            os.fsync(stream.fileno())
        result.update({
            'exit_code': child.returncode, 'elapsed_seconds': time.monotonic() - command_start,
            'log_sha256': available(lambda: hashlib.sha256(log.read_bytes()).hexdigest()),
            'stop_reason': abort, 'process_state': stop_state,
        })
        results.append(result)
        atomic_json(out / f'command-{index}-receipt.json', result)
        if failure or child.returncode:
            break
    check_source()
    if failure is None:
        guard(table(), {}, check_cache=True)
except BaseException as error:
    failure = (failure + '; ' if failure else '') + type(error).__name__ + ': ' + str(error)
finally:
    receipt = {
        'schema': 'uor-r4.manual-repair-checks/1', 'source_sha': packet.get('source_sha'),
        'packet_sha256': hashlib.sha256(packet_bytes).hexdigest(),
        'started_under_explicit_council_exception': bool(results or child is not None),
        'production_hold_unchanged': available(lambda: (production / 'admissions-held.json').read_bytes() == hold),
        'elapsed_seconds': time.monotonic() - start, 'sampled_peak_aggregate_rss_bytes': peak,
        'cache_growth_bytes': available(lambda: cache_size() - baseline),
        'free_bytes_after': available(free), 'commands': results, 'failure': failure,
        'process_state': stop_state, 'created_utc': created,
        'limitations': 'RSS and cache growth are sampled. Scope assumes the inspected tests do not create new sessions. UNKNOWN never releases the production hold or reservation. No production job reconciliation or resource-ledger debit is performed here; the coordinator records this receipt exactly once.',
    }
    atomic_json(out / 'receipt.json', receipt)
    print(json.dumps(receipt, indent=2))
sys.exit(0 if failure is None and stop_state == 'confirmed_stopped' and len(results) == len(packet['commands']) and all(x['exit_code'] == 0 for x in results) else 1)
