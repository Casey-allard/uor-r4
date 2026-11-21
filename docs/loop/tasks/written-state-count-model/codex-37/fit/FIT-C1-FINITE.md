# FIT for the D37 masked optimizer finite instrument

Reviewer [codex] tune37, 20 Nov 2026. Independent of the fit and device source authors. **TEST FITNESS: FIT** for the invented 128-row masked fixture under its own parent GO and Wick lock.

I checked the Wick control source and stage manifests with strict SHA-256 readback. The staged `control-stage.sha256` is `40c0d4d93f2606dc24ba0dd0ef304539f6c5a58154c8f606aac3234700a13b1f`. The dispatch script is `025324329951fa58ff7bba509d990dbba2189ebf4be1d62806a6e235c2230901` and the control sheet is `90bdc1d820aa7aa6b01c83cebda7365bc77dc58ab5cfab6694555b159f1d972d`. The masked executable is `9dc81179eb9ee34532b5e9bb791896526af5e9e0e72d0e8bdb8d883e9a29840a` and the Rust verifier is `226f7c9a72fa51d0b9d13c2fdefa0be60dbac7529ecd0ff570eed10f06b0e457`. The staged masked SASS scan found zero FADD, FMUL, FFMA, MUFU, HMMA or MMA mnemonics.

The device excludes Pick through `live` at initialization and when reviving live zero weights. The host independently excludes the Pick expert in the Rust EM mask. I read the fixture words at ordinal 10 and verified bucket 527 and a zero Pick mass. The verifier requires the 97 objective words and 717,024 weight words across initialization and 96 updates, plus an exact native versus CUDA terminal bucket and Q48 mass. Comparing every state exposes an implementation that gives even one Q32 unit to a masked but otherwise live expert.

The exact staged invocation uses fresh output, the registered sm_89 device, disabled PTX JIT and the shared nonblocking Wick lock. This review records source and fixture checks. The masked GPU result requires the finite dispatch.
