# [codex] D37 DEV-tune exact-pair tape

Use the admitted C2 AOT sm_89 selector and its independently verified
synthetic finite pass. The sole input stream is claude-420's original
DEV-tune UORT, pinned at
`75c6d1ac479bf120bac3aa60ac160536e8146f8052aa49c002209500799b7d0a`.
The selector receives BOS-reset exclusive prefix intervals of length at
most 192. It emits the latest exact-pair predecessor or the absent word
for every non-BOS position through EOS. Require all 10,807,417 words,
the header, and matching native probes of the first 64 record frames,
score/capture/Pick seams, fixed stride interior points and the final
two ordinals. Every fired word must be in the causal prefix.

The immutable output is 43,229,684 bytes, plus bounded receipts. Estimate
under five minutes on an sm_89 RTX 4090. Require the shared Wick lock
or the codex/d37-tune-21 Vast GPU lease lock,
no active foreign PID, 4 GiB free device memory and 4 GiB available host
memory. Require a separate FIT and explicit digest-pinned parent GO.
The sealed tape is an input to a later frozen comparison.
