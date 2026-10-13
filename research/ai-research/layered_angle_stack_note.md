# Layered Angle-Stack Test on One Coarse Route

Coarse route:
- residue r mod 30030 = 1

Refinement layers:
- x17 in Z/17Z
- x19 in Z/19Z

This treats the route as a layered angle stack:
- base angle fixed by the coarse route
- one angle at each lift layer

The test asks whether these layered local angles are informative about
prime vs large-semiprime labels *inside one exact coarse route*.

Outputs:
- neighborhood purity in (x17, x19)
- information gain from x17 only, x19 only, and their joint partition
- purity of groups under each partition
