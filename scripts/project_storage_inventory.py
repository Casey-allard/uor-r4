#!/usr/bin/env python3
"""Read-only storage inventory; never deletes, chmods, or opens model contents."""

import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import shutil
import subprocess

GIB = 1024**3
WATERMARKS = {'internal': (60, 40, 25), 'inner': (120, 60, 30),
              'outer': (240, 120, 60)}


def watermarks(role, free_bytes):
    target, warning, stop = (value * GIB for value in WATERMARKS[role])
    state = ('STOP' if free_bytes < stop else 'WARNING' if free_bytes < warning
             else 'BELOW_TARGET' if free_bytes < target else 'TARGET_MET')
    return {'target_bytes': target, 'warning_bytes': warning, 'stop_bytes': stop,
            'state': state, 'absolute_stop_margin_bytes': 128 * 1024**2,
            'track_b_trace_reservation_bytes': 30 * GIB}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, default=Path.cwd())
    parser.add_argument('--path', action='append', default=[], metavar='LABEL=PATH')
    parser.add_argument('--volume', action='append', default=[], metavar='ROLE=PATH',
                        help='Observe internal, inner or outer physical capacity; no mount identity is implied')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    repo = args.repo.resolve()
    user = Path.home()
    paths = {'model_assets':repo/'.uor-models', 'local_target':repo/'target',
             'codex_worktrees':user/'.codex/worktrees',
             'knowledge':user/'.local/share/uor-r4/knowledge',
             'tooling':user/'.local/share/uor-r4/tooling',
             'uv_tools':user/'.local/share/uv/tools',
             'uv_cache':user/'.cache/uv', 'cargo_registry':user/'.cargo/registry',
             'cargo_git':user/'.cargo/git'}
    for spec in args.path:
        label, separator, value = spec.partition('=')
        if not separator or not label or not value:
            parser.error('--path requires LABEL=PATH')
        paths[label] = Path(value).expanduser().resolve()
    disk = shutil.disk_usage(repo)
    volumes = {'internal': user}
    for spec in args.volume:
        role, separator, value = spec.partition('=')
        if not separator or role not in WATERMARKS or not value:
            parser.error('--volume requires internal|inner|outer=PATH')
        volumes[role] = Path(value).expanduser()
    volume_rows = []
    for role in WATERMARKS:
        path = volumes.get(role)
        if path is None:
            volume_rows.append({'role': role, 'status': 'UNCONFIGURED',
                                'watermarks': watermarks(role, 0),
                                'reason': 'Pass --volume with the verified live mount path.'})
            volume_rows[-1]['watermarks']['state'] = 'UNKNOWN'
            continue
        try:
            observed = shutil.disk_usage(path)
            volume_rows.append({'role': role, 'path': str(path),
                                'resolved_path': str(path.resolve()),
                                'status': 'MEASURED_IDENTITY_UNVERIFIED',
                                'total_bytes': observed.total, 'free_bytes': observed.free,
                                'watermarks': watermarks(role, observed.free)})
        except OSError as error:
            volume_rows.append({'role': role, 'path': str(path), 'status': 'UNAVAILABLE',
                                'reason': type(error).__name__})
    rows = []
    for label, path in paths.items():
        row = {'label':label, 'path':str(path)}
        if not path.exists():
            row.update(status='ABSENT', allocated_bytes=None)
        else:
            try:
                result = subprocess.run(['du','-sk',str(path)], capture_output=True, text=True, timeout=45)
                first = result.stdout.split(maxsplit=1)
                size = int(first[0]) * 1024 if first else None
                row.update(status='MEASURED' if result.returncode == 0 else 'LOWER_BOUND_OR_UNAVAILABLE',
                           allocated_bytes=size, diagnostic_lines=len(result.stderr.splitlines()))
            except (subprocess.TimeoutExpired, ValueError, FileNotFoundError) as error:
                row.update(status='UNAVAILABLE', allocated_bytes=None, reason=type(error).__name__)
        rows.append(row)
    result = {'schema':'uor-storage-inventory-v2', 'collected_at':datetime.now(timezone.utc).isoformat(),
              'disk':{'total_bytes':disk.total,'used_bytes':disk.used,'free_bytes':disk.free},
              'volume_observations': volume_rows,
              'paths':rows, 'deletions':0,
              'notes':['Rows can overlap; do not sum them as exclusive volume usage.',
                       'du allocated-byte estimates may share APFS clone data.',
                       'Permission errors remain errors; no sealed paths are opened or unlocked.',
                       'Watermarks follow docs/labs/operations.md; unconfigured backing volumes remain unknown.',
                       'Inventory alone does not verify UUID/sentinel identity or authorize admission or deletion.',
                       'Track B trace reservation and stop margin must be included in job projections, not double-counted as reclaimed space.',
                       'Size alone does not make a path disposable. Review changes, evidence and references first.']}
    rendered = json.dumps(result,indent=2) + '\n'
    if args.output:
        args.output.parent.mkdir(parents=True,exist_ok=True)
        args.output.write_text(rendered)
    print(rendered)


if __name__ == '__main__':
    main()
