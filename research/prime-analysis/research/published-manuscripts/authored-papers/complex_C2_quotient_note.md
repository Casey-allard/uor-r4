# Candidate C^2 quotient of the exact phase/fiber recursion

Built a candidate 4-real-dimensional complex state z=(z1,z2) in C^2 from
low-order torus harmonics on the global W=30030 chart:

- b = j mod 35
- f = floor(j/35) mod 143

Procedure:
1. construct complex torus characters exp(2pi i (kb*b/35 + kf*f/143))
   for small |kb|,|kf| <= 4
2. take the top two complex principal components
3. define z=(z1,z2) as the top two coordinates
4. fit a linear transport map A so that z(j+1) ≈ A z(j)

This is the first direct fit of the proposed 4D complex routing manifold.

Important interpretation:
- low transport error means the quotient is dynamically usable
- large contribution from several torus modes means the quotient is not one simple angle,
  but a coupled multi-mode complex state
