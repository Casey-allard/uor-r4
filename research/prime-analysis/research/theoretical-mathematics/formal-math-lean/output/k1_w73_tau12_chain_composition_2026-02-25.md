# K1 W73 Tau12 Chain Composition (2026-02-25)

## Objective
Compose source-locked tau12 C2 closure (W72) with rounding-preservation and W70 tail contract into one explicit q<a1 chain packet.

## Component Status
- C2 source-locked closure certified: `true`
- rho interval: `[1.487262003864974425, 1.487262003909051833]`, excludes `3/2`: `true`
- tau source rows: `4`
- rounding threshold (c0=1/2, a1=0.98): `X_round <= 35.777750435449`
- W70 tail constants: `beta=0.62`, `eta=0.01`, `C_total=124.268080597763`, `x1=1.0e+21`

## Composed Chain Statement
For any `A1>0`, `A2>=0`, and `0<q<a1`, under the W72 source-locked C2 assumptions and W70 tail envelope,
- define `X_tail = max(x1, (C_total/(q*A1))^(1/eta))`,
- define `X_joint = max(X_round, X_tail)`,
- then `c = A1*(a1-q) > 0` and for every `X` there exists `x>=X` with `Y(x) >= c`.

This is exactly the theorem-shape consumed by
`normalized_lower_envelope_of_buffered_c2_rounding_tail_and_q_lt_a1`.

## Scenario Thresholds
- A1_min_sample (A1=0.029241453574):
  - X_round: `35.777750435449` (log10 `1.553613`)
  - q=0.490000000000 (fraction `0.5`), a1-q=0.490000000000, log10(X_joint)=393.816455, c=A1*(a1-q)=0.014328312251
  - q=0.686000000000 (fraction `0.7`), a1-q=0.294000000000, log10(X_joint)=379.203652, c=A1*(a1-q)=0.008596987351
  - q=0.882000000000 (fraction `0.9`), a1-q=0.098000000000, log10(X_joint)=368.289205, c=A1*(a1-q)=0.002865662450
- A1_median_sample (A1=0.030014504040):
  - X_round: `35.777750435449` (log10 `1.553613`)
  - q=0.490000000000 (fraction `0.5`), a1-q=0.490000000000, log10(X_joint)=392.683234, c=A1*(a1-q)=0.014707106980
  - q=0.686000000000 (fraction `0.7`), a1-q=0.294000000000, log10(X_joint)=378.070430, c=A1*(a1-q)=0.008824264188
  - q=0.882000000000 (fraction `0.9`), a1-q=0.098000000000, log10(X_joint)=367.155983, c=A1*(a1-q)=0.002941421396
- A1_max_sample (A1=0.031722201975):
  - X_round: `35.777750435449` (log10 `1.553613`)
  - q=0.490000000000 (fraction `0.5`), a1-q=0.490000000000, log10(X_joint)=390.280018, c=A1*(a1-q)=0.015543878968
  - q=0.686000000000 (fraction `0.7`), a1-q=0.294000000000, log10(X_joint)=375.667215, c=A1*(a1-q)=0.009326327381
  - q=0.882000000000 (fraction `0.9`), a1-q=0.098000000000, log10(X_joint)=364.752768, c=A1*(a1-q)=0.003108775794

## Remaining Blocker
- Construct one concrete non-circular instance of K1SourceNonCircularProvider.theorem_term without RH in hypotheses or intermediate derivation.
