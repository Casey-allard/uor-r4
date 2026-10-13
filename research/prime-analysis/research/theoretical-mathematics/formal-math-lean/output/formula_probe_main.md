# Formula Probe

Generated: 2026-02-16T22:41:12.895188+00:00

## Candidate Meta-Formula

holdout ~= +0.060543 -0.241022*sep -7.236915*entropy_delta -0.396807*phase_jerk_delta +1.417360*delta_h1 +1.343815*delta_h3

R^2=0.996051

## Asymptotic Fits

- separation_ratio(N) ~= +0.379385 -9277.066926*N^(-1.45) (R^2=0.1524)
- entropy_delta(N) ~= -0.103839 +0.024736*N^(-0.20) (R^2=0.5941)
- phase_jerk_delta(N) ~= +1.519343 -54954.561196*N^(-1.45) (R^2=0.7810)
- delta_h1(N) ~= +0.174070 -21205.501867*N^(-1.45) (R^2=0.9029)
- delta_h3(N) ~= +0.181254 +0.217011*N^(-0.20) (R^2=0.8689)
- holdout_accuracy(N) ~= +0.618646 +4724.153153*N^(-1.45) (R^2=0.1031)

## Discriminant Score

S(n) = -276710.641771*x + +448.582324*y + -105212.038974*z + +893.774846*t + -21511453317.788532
Prime-like if S(n) > 0, where (x,y,z,t)=embed_4d(n,moduli,radius_power).
