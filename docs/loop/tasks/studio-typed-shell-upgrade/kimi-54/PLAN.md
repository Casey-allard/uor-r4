# Entry behavior and evidence repair plan

1. Rewrite `main.test.ts` to reset modules, create the root before evaluation, dynamically import `main.ts` without `typed=1`, and prove no mount or mutation. Add a fake-timer error-path test with incomplete workflow methods, bounded advancement, and assertions for computed visibility, `data-active`, and error text.
2. Assert that the built typed scenario has no runtime exception or console error; preserve existing visibility/model checks and rebuild before running it.
3. Update the notebook to 383 and make the final-head convention explicit: `identity.log` and PR metadata carry exact final HEAD/tree, while the REPORT names repair and reviewed merge bases without a self-referential hash.
4. Run focused/full/plain-JS/legacy/harness/full gates and obtain a fresh exact-head review. No production source change or publication.
