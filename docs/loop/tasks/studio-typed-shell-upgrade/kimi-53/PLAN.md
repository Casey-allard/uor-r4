# Visibility and evidence repair plan

1. Add failing tests for `root.hidden = false` on mount and initialization failure, `root.hidden = true` on dispose, and no mount/mutation without `typed=1`. Extend the built-browser scenario to assert computed root display and typed-only text, not just DOM presence.
2. Implement visibility transitions in `mountTypedStudio` and `src/main.ts`, preserving the existing worker and readiness guard. Rebuild before every browser check so stale `dist` cannot be mistaken for a product failure.
3. Narrow publication statements to the proven boundary: the default source UI remains legacy and no publish ran, while the current allowlist still needs a later `dist` packaging unit. Record the original branch base separately from the reviewed merge base.
4. Update the REPORT with actual final HEAD/tree and executed check results, run focused/full/plain-JS/legacy/harness/full gates, and obtain a fresh exact-head review. Do not publish or start native packaging in this repair.
