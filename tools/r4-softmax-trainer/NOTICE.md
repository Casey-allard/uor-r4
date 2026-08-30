# Attribution and scope

The training shape and workflow in this directory are adapted from the MIT
licensed [`karpathy/llama2.c`](https://github.com/karpathy/llama2.c) project,
pinned for issue #1014 at commit
[`350e04fe35433e6d2941dce5a1f53308f87058eb`](https://github.com/karpathy/llama2.c/tree/350e04fe35433e6d2941dce5a1f53308f87058eb).
Copyright (c) 2023 Andrej. No upstream source files or model weights are
vendored here.

The TinyStories corpus is downloaded from the
[`roneneldan/TinyStories`](https://huggingface.co/datasets/roneneldan/TinyStories)
dataset repository at the revision and file digest declared in
`r4_softmax_trainer.constants`. Dataset bytes, derived token stores,
checkpoints, environments, and exports stay under the ignored local
`.uor-models/research/issue-1014/` tree.

The implementation itself remains under the repository's root MIT license.

