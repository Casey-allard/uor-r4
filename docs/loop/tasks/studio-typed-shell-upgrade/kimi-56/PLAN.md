# Real readiness failure and preview evidence repair plan

1. Replace the entry's bootstrap mock with real legacy workflow fixtures. Change the bootstrap partial-method test to retain `getSettings` while omitting session methods, and run both focused tests against the old readiness predicate to capture red evidence.
2. Add the minimal initial-render method checks to `waitForWorkflowApis`, complete the bootstrap fixtures, and rerun the focused tests. Verify the real component reference passed to Preact without claiming the render mock produces DOM.
3. Require one `.studio-shell`, retain exact MCP counts, and reject raw TypeScript/default `src/main` request URLs from their recorded network paths. Run the built runtime check with the typed scenario enabled.
4. Correct the notebook link and historical/current evidence wording. Rerun all product and repository checks at one committed head, then obtain a fresh exact-head review before protected delivery.
