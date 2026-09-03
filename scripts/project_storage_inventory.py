#!/usr/bin/env python3
"""Read-only storage inventory; never deletes, chmods, or opens model contents."""

import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import shutil
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, default=Path.cwd())
    parser.add_argument('--path', action='append', default=[], metavar='LABEL=PATH')
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
    reserve = max(20 * 1024**3, int(disk.total * 0.15))
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
    result = {'schema':'uor-storage-inventory-v1', 'collected_at':datetime.now(timezone.utc).isoformat(),
              'disk':{'total_bytes':disk.total,'used_bytes':disk.used,'free_bytes':disk.free},
              'proposed_reserve_bytes':reserve, 'below_proposed_reserve':disk.free < reserve,
              'paths':rows, 'deletions':0,
              'notes':['Rows can overlap; do not sum them as exclusive volume usage.',
                       'du allocated-byte estimates may share APFS clone data.',
                       'Permission errors remain errors; no sealed paths are opened or unlocked.',
                       'Size alone does not make a path disposable. Review changes, evidence and references first.']}
    rendered = json.dumps(result,indent=2) + '\n'
    if args.output:
        args.output.parent.mkdir(parents=True,exist_ok=True)
        args.output.write_text(rendered)
    print(rendered)


if __name__ == '__main__':
    main()
