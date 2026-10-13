#!/usr/bin/env python3
"""Run the T_z' rebuild from spin_H_core_v6 and write the bounded audit CSV.

This runner generates:
    results/prime_transport_recursive_system/prime_transport_Tz_rebuild_v1.csv

Using geometry_native_operator_model_v21 where:
    - T_z': fiber_phase_lift_component_v21 (rebuilt from spin_H_core_v6)
    - T_c: coupled_torus_kick_component_v20 (from spin_H_core_v6; unchanged)
    - All other components: v10/v12 (unchanged from v20)
"""

from __future__ import annotations

import sys
import time
from pathlib import Path

_root = Path(__file__).resolve().parent
if str(_root) not in sys.path:
    sys.path.insert(0, str(_root))

from geometry_native_operator_model_v21 import (
    OUTPUT_PATH_TZ_REBUILD_V1,
    summarize_operator_from_spinH_core_v6_lift,
    write_Tz_rebuild_v1,
)


def main(depth: int = 8) -> None:
    print(f"[v21] Running T_z' rebuild audit at depth={depth} ...")
    t0 = time.perf_counter()
    rows = summarize_operator_from_spinH_core_v6_lift(depth=depth)
    elapsed = time.perf_counter() - t0

    print(f"[v21] Completed in {elapsed:.1f}s")
    print()
    header = f"{'metric':<45}  {'value':>20}  note"
    print(header)
    print("-" * len(header))
    for row in rows:
        print(f"{row['metric']:<45}  {str(row['value']):>20}  {row['note']}")

    write_Tz_rebuild_v1(rows)
    print(f"\n[v21] Wrote {OUTPUT_PATH_TZ_REBUILD_V1}")


if __name__ == "__main__":
    main()
