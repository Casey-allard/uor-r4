# Layout Surgery Notes

## Main-text assembly changes

- Shortened the title to `Fixed Geometric Routing from Angular Structure`.
- Simplified the author block and removed the email address from page 1.
- Removed the main-text setup table and folded that information back into prose.
- Removed the proxy-control figure from the main paper and moved it to the appendix.
- Removed the main-text null-result table and kept that result in prose, with the compact table moved to the appendix.
- Kept the main paper to the minimum visual budget:
  - Figure 1: method schematic
  - Figure 2: scaling figure
  - Table 1: combined LM result table

## Float and pagination fixes

- Eliminated the crowded mid-paper float stack by leaving only one table in the body.
- Changed the LM main results table to a single-column table with a controlled resize instead of a full-width float.
- Moved all non-core supporting figures and tables into the appendix.
- Inserted `\clearpage` before the appendix and switched the appendix to `\onecolumn`.
- Kept references in the main two-column flow so they no longer compete with appendix floats.

## Appendix assembly changes

- Rebuilt the appendix as a separate one-column supplement section.
- Moved setup, control, null-result, and detail tables into the appendix.
- Kept the appendix figure and tables grouped as intentional support material rather than scattered float spillover from the body.

## Remaining layout caveats

- The custom ML-style shell is stable, but it is still not an official venue package.
- Some appendix tables are dense and still trigger minor TeX box warnings, though the compiled result is visually acceptable.
- The appendix now prioritizes stability and legibility over compactness.
