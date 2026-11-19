# D24 integer role learner

Build the standalone CUDA executable ahead of time on the GPU host. The same binary and invocation are used for Identity and Positional. Each invocation receives the feature rows for exactly one projector arm and seed.

```sh
nvcc -std=c++17 -O2 device/role.cu -o d24-role
./d24-role finite finite.bank
./d24-role fit train.features.tsv train.labels.txt role.bank
./d24-role predict held.features.tsv role.bank held.predictions.tsv
```

The fit and predict commands are separate processes. Predict starts by loading the saved bank into a fresh process and uploads it to CUDA. Each output requires a fresh path. Seal the feature, training label, executable, fitted bank and prediction files with the run manifest. Keep held labels in the scorer's input set.

Every feature line contains 32 whitespace-separated signed 32-bit coordinates in projector order. Every training label line contains one integer matching the corresponding feature line. The roles are 0 unspecified, 1 requested and 2 excluded. Each split contains at least one row and at most 1,000,000. Fit requires all three roles.

The saved bank is 800 bytes. Its eight-byte ASCII header is `D24ROLE1`, followed by three records in role order. Each record is a signed little-endian 64-bit count followed by 32 signed little-endian 64-bit coordinate sums. The CUDA fit kernel directly writes these integer counts and sums. Serialization preserves all 99 words. Both controls have the same learned state capacity and serialized footprint.

For query vector `x`, role count `c` and role coordinate sums `s_i`, CUDA computes `n = sum_i abs(c*x_i - s_i)`. The L1 distance to the exact rational class prototype is `n/c`. The exact comparison uses unsigned 128-bit cross products. The row cap bounds each distance numerator by 137,438,953,440,000,000 and each cross product below 2^77. The selected role is the unique minimum, with `-1` for a tied minimum. This analytic prototype fit uses supervised integer accumulation and has no gradient or optimizer state.

Prediction TSV has no header. It preserves input row order and writes the selected role followed by the three exact distance numerators in role order. Recover each distance by pairing its numerator with the corresponding count in the serialized bank. Candidate identity, clause metadata and held labels stay outside the device interface.

The finite command uses only a synthetic unequal-count fixture. It checks all raw bank words against explicit counts and sums before and after reload. It checks the raw result words for unique requested, excluded and unspecified predictions, abstentions and a case where the smallest raw numerator would pick the wrong class. The same typed classifier is compiled for CUDA and for the finite host comparison. The host comparison executes only inside the finite command.

## Projector and CLI refusal checks

```sh
nvcc -std=c++17 -O2 device/project.cu -o d24-project
./d24-project identity tables.bin train.candidates.txt train.device.identity.txt
./d24-project positional tables.bin train.candidates.txt train.device.positional.txt
device/invalid-inputs.sh ./d24-role ./d24-project invalid
```

Use the seed's frozen D15 table packet. It contains 4096 packed-root and class pairs as little-endian u32 words, then 16 × 8 signed root bytes and 64 × 8 unsigned signed-axis codes. Each candidate line contains four half-open token offsets followed by the original tokenizer IDs, beginning with BOS. The offsets are mention start, mention end, clause start and clause end. Candidate tapes carry at most 192 tokens. Project preserves row order across 1024-row CUDA batches and writes the 32 signed coordinates separated by single spaces. A bytewise comparison with the panel's host feature output checks the entire projector boundary. The same packet and tape apply to both projector arms. The dispatch manifest binds each fitted bank to the arm, seed and train input identities before scoring.

The finite fit also permutes requested and excluded TRAIN classes and checks the resulting bank, distances and calls for that permutation. The CLI refusal script independently checks malformed features, label files, banks and projector inputs. Its checks fail before CUDA initializes.
