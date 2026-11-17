# REPORT for the past-window scorer input refusals

Lab: codex. Item: [integer-long-context](../../../../roadmap/items/integer-long-context.md). [ORDER](ORDER.md). References #3714.

## What did not work

At the contributor head a 4096-token checkpoint and a short carry panel passed the agreement check. The scorer installed the maximum of W and C, while the output file recorded the panel's C of 2048. The issue's fifth item, full default scorer output equivalence, still lacks a host-level regression fixture.

The first adoption head selected C for panels but also used it for the optional TRAIN query-cache pass. A TRAIN window wider than C would exceed the model context after rows were written. An empty offset band also has no loss mean, which the first documentation change overlooked.

## What changed

Preserved @Rajaykumar12's commit from [#4039](https://github.com/UOR-Foundation/uor-r4/pull/4039). It rejects missing run objectives, loss vectors whose length differs from the band count, carry panels with different contexts and zero-width document windows. The adoption resolves the optional model override from the panels themselves. Every carry panel shares the reported C and supplies it to the model. Other paths keep the loaded context. Context selection happens before model loading.

The cache forward uses W while reading TRAIN and restores C for subsequent decodes and memory modes, including on a cache-write failure. Written bands with at least one target have a loss mean.

## Measurements

Eighteen past-window host tests passed, including the saved-objective, band length, cutter and TRAIN cache controls. The dedicated mixed-context and zero-window tests also passed. A host control builds an actual 4096-token TRAIN window. Its width check refuses the window at C=2048 and accepts it at W=4096. Success and an injected write error both restore C=2048.

## Checks run at the head

Rust 1.97.1, default features, repaired source `f4d2eb833`. Used the float-train library test binary with filters `past_window`, `carry_refuses_panels_at_different_contexts` and `document_windowers_refuse_a_zero_window`. The whitespace check passed.

## Skills used

Source review and host refusal tests under the ORDER.

## Still open

#3714 item 5 covers complete output from the default scorer. This run exercised host data and configuration checks. There was no GPU scorer execution or model evaluation.

## Plain summary

The scorer now refuses all four malformed inputs the contributor took on and uses the same carry context it writes into its output.
