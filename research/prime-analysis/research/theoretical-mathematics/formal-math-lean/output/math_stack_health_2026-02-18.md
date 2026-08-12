# Math Stack Health (2026-02-18)

## Installed / Upgraded

- numpy: 1.26.4
- scipy: 1.15.3
- scikit-learn: 1.7.2
- sympy: 1.14.0
- mpmath: 1.3.0
- matplotlib: 3.10.8

## Validation

- Import check for numpy/scipy/sklearn/sympy/mpmath/matplotlib passed.
- Numeric sanity checks passed:
  - `scipy.linalg.solve([[3,1],[1,2]],[9,8]) = [2,3]`
  - sklearn linear regression fit executed successfully.

## Environment Fix

- Added wrapper: `research/run_with_math_env.sh`
- This sets `MPLCONFIGDIR` to `research/cache/mplconfig` so matplotlib avoids non-writable home-cache warnings.
