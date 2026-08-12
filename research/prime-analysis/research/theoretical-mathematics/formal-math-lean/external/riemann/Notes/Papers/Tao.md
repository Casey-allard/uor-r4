The Riemann hypothesis in various settings
Terence Tao
[Note: the content of this post is standard number theoretic material that can be found in many textbooks (I am relying principally here on Iwaniec and Kowalski); I am not claiming any new progress on any version of the Riemann hypothesis here, but am simply arranging existing facts together.]

The Riemann hypothesis is arguably the most important and famous unsolved problem in number theory. It is usually phrased in terms of the Riemann zeta function {\zeta}, defined by

\displaystyle  \zeta(s) = \sum_{n=1}^\infty \frac{1}{n^s}

for {\hbox{Re}(s)>1} and extended meromorphically to other values of {s}, and asserts that the only zeroes of {\zeta} in the critical strip {\{ s: 0 \leq \hbox{Re}(s) \leq 1 \}} lie on the critical line {\{ s: \hbox{Re}(s)=\frac{1}{2} \}}.

One of the main reasons that the Riemann hypothesis is so important to number theory is that the zeroes of the zeta function in the critical strip control the distribution of the primes. To see the connection, let us perform the following formal manipulations (ignoring for now the important analytic issues of convergence of series, interchanging sums, branches of the logarithm, etc., in order to focus on the intuition). The starting point is the fundamental theorem of arithmetic, which asserts that every natural number {n} has a unique factorisation {n = p_1^{a_1} \ldots p_k^{a_k}} into primes. Taking logarithms, we obtain the identity

 \displaystyle  \log n = \sum_{d|n} \Lambda(d) \ \ \ \ \ (1)

for any natural number {n}, where {\Lambda} is the von Mangoldt function, thus {\Lambda(n) = \log p} when {n} is a power of a prime {p} and zero otherwise. If we then perform a “Dirichlet-Fourier transform” by viewing both sides of (1) as coefficients of a Dirichlet series, we conclude that

\displaystyle  \sum_{n=1}^\infty \frac{\log n}{n^s} = \sum_{n=1}^\infty \sum_{d|n} \frac{\Lambda(d)}{n^s},

formally at least. Writing {n=dm}, the right-hand side factors as

\displaystyle (\sum_{d=1}^\infty \frac{\Lambda(d)}{d^s}) (\sum_{m=1}^\infty \frac{1}{m^s}) = \zeta(s) \sum_{n=1}^\infty \frac{\Lambda(n)}{n^s}

whereas the left-hand side is (formally, at least) equal to {-\zeta'(s)}. We conclude the identity

\displaystyle  \sum_{n=1}^\infty \frac{\Lambda(n)}{n^s} = -\frac{\zeta'(s)}{\zeta(s)},

(formally, at least). If we integrate this, we are formally led to the identity

\displaystyle  \sum_{n=1}^\infty \frac{\Lambda(n)}{\log n} n^{-s} = \log \zeta(s)

or equivalently to the exponential identity

 \displaystyle  \zeta(s) = \exp( \sum_{n=1}^\infty \frac{\Lambda(n)}{\log n} n^{-s} ) \ \ \ \ \ (2)

which allows one to reconstruct the Riemann zeta function from the von Mangoldt function. (It is instructive exercise in enumerative combinatorics to try to prove this identity directly, at the level of formal Dirichlet series, using the fundamental theorem of arithmetic of course.) Now, as {\zeta} has a simple pole at {s=1} and zeroes at various places {s=\rho} on the critical strip, we expect a Weierstrass factorisation which formally (ignoring normalisation issues) takes the form

\displaystyle  \zeta(s) = \frac{1}{s-1} \times \prod_\rho (s-\rho) \times \ldots

(where we will be intentionally vague about what is hiding in the {\ldots} terms) and so we expect an expansion of the form

 \displaystyle  \sum_{n=1}^\infty \frac{\Lambda(n)}{\log n} n^{-s} = - \log(s-1) + \sum_\rho \log(s-\rho) + \ldots. \ \ \ \ \ (3)

Note that

\displaystyle  \frac{1}{s-\rho} = \int_1^\infty t^{\rho-s} \frac{dt}{t}

and hence on integrating in {s} we formally have

\displaystyle  \log(s-\rho) = -\int_1^\infty t^{\rho-s-1} \frac{dt}{\log t}

and thus we have the heuristic approximation

\displaystyle  \log(s-\rho) \approx - \sum_{n=1}^\infty \frac{n^{\rho-s-1}}{\log n}.

Comparing this with (3), we are led to a heuristic form of the explicit formula

 \displaystyle  \Lambda(n) \approx 1 - \sum_\rho n^{\rho-1}. \ \ \ \ \ (4)

When trying to make this heuristic rigorous, it turns out (due to the rough nature of both sides of (4)) that one has to interpret the explicit formula in some suitably weak sense, for instance by testing (4) against the indicator function {1_{[0,x]}(n)} to obtain the formula

 \displaystyle  \sum_{n \leq x} \Lambda(n) \approx x - \sum_\rho \frac{x^{\rho}}{\rho} \ \ \ \ \ (5)

which can in fact be made into a rigorous statement after some truncation (the von Mangoldt explicit formula). From this formula we now see how helpful the Riemann hypothesis will be to control the distribution of the primes; indeed, if the Riemann hypothesis holds, so that {\hbox{Re}(\rho) = 1/2} for all zeroes {\rho}, it is not difficult to use (a suitably rigorous version of) the explicit formula to conclude that

 \displaystyle  \sum_{n \leq x} \Lambda(n) = x + O( x^{1/2} \log^2 x ) \ \ \ \ \ (6)

as {x \rightarrow \infty}, giving a near-optimal “square root cancellation” for the sum {\sum_{n \leq x} \Lambda(n)-1}. Conversely, if one can somehow establish a bound of the form

\displaystyle  \sum_{n \leq x} \Lambda(n) = x + O( x^{1/2+\epsilon} )

for any fixed {\epsilon}, then the explicit formula can be used to then deduce that all zeroes {\rho} of {\zeta} have real part at most {1/2+\epsilon}, which leads to the following remarkable amplification phenomenon (analogous, as we will see later, to the tensor power trick): any bound of the form

\displaystyle  \sum_{n \leq x} \Lambda(n) = x + O( x^{1/2+o(1)} )

can be automatically amplified to the stronger bound

\displaystyle  \sum_{n \leq x} \Lambda(n) = x + O( x^{1/2} \log^2 x )

with both bounds being equivalent to the Riemann hypothesis. Of course, the Riemann hypothesis for the Riemann zeta function remains open; but partial progress on this hypothesis (in the form of zero-free regions for the zeta function) leads to partial versions of the asymptotic (6). For instance, it is known that there are no zeroes of the zeta function on the line {\hbox{Re}(s)=1}, and this can be shown by some analysis (either complex analysis or Fourier analysis) to be equivalent to the prime number theorem

\displaystyle  \sum_{n \leq x} \Lambda(n) =x + o(x);

see e.g. this previous blog post for more discussion.

The main engine powering the above observations was the fundamental theorem of arithmetic, and so one can expect to establish similar assertions in other contexts where some version of the fundamental theorem of arithmetic is available. One of the simplest such variants is to continue working on the natural numbers, but “twist” them by a Dirichlet character {\chi: {\bf Z} \rightarrow {\bf R}}. The analogue of the Riemann zeta function is then the https://en.wikipedia.org/wiki/Multiplicative_function, the equation (1), which encoded the fundamental theorem of arithmetic, can be twisted by {\chi} to obtain

 \displaystyle  \chi(n) \log n = \sum_{d|n} \chi(d) \Lambda(d) \chi(\frac{n}{d}) \ \ \ \ \ (7)

and essentially the same manipulations as before eventually lead to the exponential identity

 \displaystyle  L(s,\chi) = \exp( \sum_{n=1}^\infty \frac{\chi(n) \Lambda(n)}{\log n} n^{-s} ). \ \ \ \ \ (8)

which is a twisted version of (2), as well as twisted explicit formula, which heuristically takes the form

 \displaystyle  \chi(n) \Lambda(n) \approx - \sum_\rho n^{\rho-1} \ \ \ \ \ (9)

for non-principal {\chi}, where {\rho} now ranges over the zeroes of {L(s,\chi)} in the critical strip, rather than the zeroes of {\zeta(s)}; a more accurate formulation, following (5), would be

 \displaystyle  \sum_{n \leq x} \chi(n) \Lambda(n) \approx - \sum_\rho \frac{x^{\rho}}{\rho}. \ \ \ \ \ (10)

(See e.g. Davenport’s book for a more rigorous discussion which emphasises the analogy between the Riemann zeta function and the Dirichlet {L}-function.) If we assume the generalised Riemann hypothesis, which asserts that all zeroes of {L(s,\chi)} in the critical strip also lie on the critical line, then we obtain the bound

\displaystyle  \sum_{n \leq x} \chi(n) \Lambda(n) = O( x^{1/2} \log(x) \log(xq) )

for any non-principal Dirichlet character {\chi}, again demonstrating a near-optimal square root cancellation for this sum. Again, we have the amplification property that the above bound is implied by the apparently weaker bound

\displaystyle  \sum_{n \leq x} \chi(n) \Lambda(n) = O( x^{1/2+o(1)} )

(where {o(1)} denotes a quantity that goes to zero as {x \rightarrow \infty} for any fixed {q}). Next, one can consider other number systems than the natural numbers {{\bf N}} and integers {{\bf Z}}. For instance, one can replace the integers {{\bf Z}} with rings {{\mathcal O}_K} of integers in other number fields {K} (i.e. finite extensions of {{\bf Q}}), such as the quadratic extensions {K = {\bf Q}[\sqrt{D}]} of the rationals for various square-free integers {D}, in which case the ring of integers would be the ring of quadratic integers {{\mathcal O}_K = {\bf Z}[\omega]} for a suitable generator {\omega} (it turns out that one can take {\omega = \sqrt{D}} if {D=2,3\hbox{ mod } 4}, and {\omega = \frac{1+\sqrt{D}}{2}} if {D=1 \hbox{ mod } 4}). Here, it is not immediately obvious what the analogue of the natural numbers {{\bf N}} is in this setting, since rings such as {{\bf Z}[\omega]} do not come with a natural ordering. However, we can adopt an algebraic viewpoint to see the correct generalisation, observing that every natural number {n} generates a principal ideal {(n) = \{ an: a \in {\bf Z} \}} in the integers, and conversely every non-trivial ideal {{\mathfrak n}} in the integers is associated to precisely one natural number {n} in this fashion, namely the norm {N({\mathfrak n}) := |{\bf Z} / {\mathfrak n}|} of that ideal. So one can identify the natural numbers with the ideals of {{\bf Z}}. Furthermore, with this identification, the prime numbers correspond to the prime ideals, since if {p} is prime, and {a,b} are integers, then {ab \in (p)} if and only if one of {a \in (p)} or {b \in (p)} is true. Finally, even in number systems (such as {{\bf Z}[\sqrt{-5}]}) in which the classical version of the fundamental theorem of arithmetic fail (e.g. {6 = 2 \times 3 = (1-\sqrt{-5})(1+\sqrt{-5})}), we have the fundamental theorem of arithmetic for ideals: every ideal {\mathfrak{n}} in a Dedekind domain (which includes the ring {{\mathcal O}_K} of integers in a number field as a key example) is uniquely representable (up to permutation) as the product of a finite number of prime ideals {\mathfrak{p}} (although these ideals might not necessarily be principal). For instance, in {{\bf Z}[\sqrt{-5}]}, the principal ideal {(6)} factors as the product of four prime (but non-principal) ideals {(2, 1+\sqrt{-5})}, {(2, 1-\sqrt{-5})}, {(3, 1+\sqrt{-5})}, {(3, 1-\sqrt{-5})}. (Note that the first two ideals {(2,1+\sqrt{5}), (2,1-\sqrt{5})} are actually equal to each other.) Because we still have the fundamental theorem of arithmetic, we can develop analogues of the previous observations relating the Riemann hypothesis to the distribution of primes. The analogue of the Riemann hypothesis is now the Dedekind zeta function

\displaystyle  \zeta_K(s) := \sum_{{\mathfrak n}} \frac{1}{N({\mathfrak n})^s}

where the summation is over all non-trivial ideals in {{\mathcal O}_K}. One can also define a von Mangoldt function {\Lambda_K({\mathfrak n})}, defined as {\log N( {\mathfrak p})} when {{\mathfrak n}} is a power of a prime ideal {{\mathfrak p}}, and zero otherwise; then the fundamental theorem of arithmetic for ideals can be encoded in an analogue of (1) (or (7)),

 \displaystyle  \log N({\mathfrak n}) = \sum_{{\mathfrak d}|{\mathfrak n}} \Lambda_K({\mathfrak d}) \ \ \ \ \ (11)

which leads as before to an exponential identity

 \displaystyle  \zeta_K(s) = \exp( \sum_{{\mathfrak n}} \frac{\Lambda_K({\mathfrak n})}{\log N({\mathfrak n})} N({\mathfrak n})^{-s} ) \ \ \ \ \ (12)

and an explicit formula of the heuristic form

\displaystyle  \Lambda({\mathfrak n}) \approx 1 - \sum_\rho N({\mathfrak n})^{\rho-1}

or, a little more accurately,

 \displaystyle  \sum_{N({\mathfrak n}) \leq x} \Lambda({\mathfrak n}) \approx x - \sum_\rho \frac{x^{\rho}}{\rho} \ \ \ \ \ (13)

in analogy with (5) or (10). Again, a suitable Riemann hypothesis for the Dedekind zeta function leads to good asymptotics for the distribution of prime ideals, giving a bound of the form

\displaystyle  \sum_{N({\mathfrak n}) \leq x} \Lambda({\mathfrak n}) = x + O( \sqrt{x} \log(x) (d+\log(Dx)) )

where {D} is the conductor of {K} (which, in the case of number fields, is the absolute value of the discriminant of {K}) and {d = \hbox{dim}_{\bf Q}(K)} is the degree of the extension of {K} over {{\bf Q}}. As before, we have the amplification phenomenon that the above near-optimal square root cancellation bound is implied by the weaker bound

\displaystyle  \sum_{N({\mathfrak n}) \leq x} \Lambda({\mathfrak n}) = x + O( x^{1/2+o(1)} )

where {o(1)} denotes a quantity that goes to zero as {x \rightarrow \infty} (holding {K} fixed). See e.g. Chapter 5 of Iwaniec-Kowalski for details.

As was the case with the Dirichlet {L}-functions, one can twist the Dedekind zeta function example by characters, in this case the Hecke characters; we will not do this here, but see e.g. Section 3 of Iwaniec-Kowalski for details.

Very analogous considerations hold if we move from number fields to function fields. The simplest case is the function field associated to the affine line {{\mathbb A}^1} and a finite field {{\mathbb F} = {\mathbb F}_q} of some order {q}. The polynomial functions on the affine line {{\mathbb A}^1/{\mathbb F}} are just the usual polynomial ring {{\mathbb F}[t]}, which then play the role of the integers {{\bf Z}} (or {{\mathcal O}_K}) in previous examples. This ring happens to be a unique factorisation domain, so the situation is closely analogous to the classical setting of the Riemann zeta function. The analogue of the natural numbers are the monic polynomials (since every non-trivial principal ideal is generated by precisely one monic polynomial), and the analogue of the prime numbers are the irreducible monic polynomials. The norm {N(f)} of a polynomial is the order of {{\mathbb F}[t] / (f)}, which can be computed explicitly as

\displaystyle  N(f) = q^{\hbox{deg}(f)}.

Because of this, we will normalise things slightly differently here and use {\hbox{deg}(f)} in place of {\log N(f)} in what follows. The (local) zeta function {\zeta_{{\mathbb A}^1/{\mathbb F}}(s)} is then defined as

\displaystyle  \zeta_{{\mathbb A}^1/{\mathbb F}}(s) = \sum_f \frac{1}{N(f)^s}

where {f} ranges over monic polynomials, and the von Mangoldt function {\Lambda_{{\mathbb A}^1/{\mathbb F}}(f)} is defined to equal {\hbox{deg}(g)} when {f} is a power of a monic irreducible polynomial {g}, and zero otherwise. Note that because {N(f)} is always a power of {q}, the zeta function here is in fact periodic with period {2\pi i / \log q}. Because of this, it is customary to make a change of variables {T := q^{-s}}, so that

\displaystyle  \zeta_{{\mathbb A}^1/{\mathbb F}}(s) = Z( {\mathbb A}^1/{\mathbb F}, T )

and {Z} is the renormalised zeta function

 \displaystyle  Z( {\mathbb A}^1/{\mathbb F}, T ) = \sum_f T^{\hbox{deg}(f)}. \ \ \ \ \ (14)

We have the analogue of (1) (or (7) or (11)):

 \displaystyle  \hbox{deg}(f) = \sum_{g|f} \Lambda_{{\mathbb A}^1/{\mathbb F}}(g), \ \ \ \ \ (15)

which leads as before to an exponential identity

 \displaystyle  Z( {\mathbb A}^1/{\mathbb F}, T ) = \exp( \sum_f \frac{\Lambda_{{\mathbb A}^1/{\mathbb F}}(f)}{\hbox{deg}(f)} T^{\hbox{deg}(f)} ) \ \ \ \ \ (16)

analogous to (2), (8), or (12). It also leads to the explicit formula

\displaystyle  \Lambda_{{\mathbb A}^1/{\mathbb F}}(f) \approx 1 - \sum_\rho N(f)^{\rho-1}

where {\rho} are the zeroes of the original zeta function {\zeta_{{\mathbb A}^1/{\mathbb F}}(s)} (counting each residue class of the period {2\pi i/\log q} just once), or equivalently

\displaystyle  \Lambda_{{\mathbb A}^1/{\mathbb F}}(f) \approx 1 - q^{-\hbox{deg}(f)} \sum_\alpha \alpha^{\hbox{deg}(f)},

where {\alpha} are the reciprocals of the roots of the normalised zeta function {Z( {\mathbb A}^1/{\mathbb F}, T )} (or to put it another way, {1-\alpha T} are the factors of this zeta function). Again, to make proper sense of this heuristic we need to sum, obtaining

\displaystyle  \sum_{\hbox{deg}(f) = n} \Lambda_{{\mathbb A}^1/{\mathbb F}}(f) \approx q^n - \sum_\alpha \alpha^n.

As it turns out, in the function field setting, the zeta functions are always rational (this is part of the Weil conjectures), and the above heuristic formula is basically exact up to a constant factor, thus

 \displaystyle  \sum_{\hbox{deg}(f) = n} \Lambda_{{\mathbb A}^1/{\mathbb F}}(f) = q^n - \sum_\alpha \alpha^n + c \ \ \ \ \ (17)

for an explicit integer {c} (independent of {n}) arising from any potential pole of {Z} at {T=1}. In the case of the affine line {{\mathbb A}^1}, the situation is particularly simple, because the zeta function {Z( {\mathbb A}^1/{\mathbb F}, T)} is easy to compute. Indeed, since there are exactly {q^n} monic polynomials of a given degree {n}, we see from (14) that

\displaystyle  Z( {\mathbb A}^1/{\mathbb F}, T ) = \sum_{n=0}^\infty q^n T^n = \frac{1}{1-qT}

so in fact there are no zeroes whatsoever, and no pole at {T=1} either, so we have an exact prime number theorem for this function field:

 \displaystyle  \sum_{\hbox{deg}(f) = n} \Lambda_{{\mathbb A}^1/{\mathbb F}}(f) = q^n \ \ \ \ \ (18)

Among other things, this tells us that the number of irreducible monic polynomials of degree {n} is {q^n/n + O(q^{n/2})}.

We can transition from an algebraic perspective to a geometric one, by viewing a given monic polynomial {f \in {\mathbb F}[t]} through its roots, which are a finite set of points in the algebraic closure {\overline{{\mathbb F}}} of the finite field {{\mathbb F}} (or more suggestively, as points on the affine line {{\mathbb A}^1( \overline{{\mathbb F}} )}). The number of such points (counting multiplicity) is the degree of {f}, and from the factor theorem, the set of points determines the monic polynomial {f} (or, if one removes the monic hypothesis, it determines the polynomial {f} projectively). These points have an action of the Galois group {\hbox{Gal}( \overline{{\mathbb F}} / {\mathbb F} )}. It is a classical fact that this Galois group is in fact a cyclic group generated by a single element, the (geometric) Frobenius map {\hbox{Frob}: x \mapsto x^q}, which fixes the elements of the original finite field {{\mathbb F}} but permutes the other elements of {\overline{{\mathbb F}}}. Thus the roots of a given polynomial {f} split into orbits of the Frobenius map. One can check that the roots consist of a single such orbit (counting multiplicity) if and only if {f} is irreducible; thus the fundamental theorem of arithmetic can be viewed geometrically as as the orbit decomposition of any Frobenius-invariant finite set of points in the affine line.

Now consider the degree {n} finite field extension {{\mathbb F}_n} of {{\mathbb F}} (it is a classical fact that there is exactly one such extension up to isomorphism for each {n}); this is a subfield of {\overline{{\mathbb F}}} of order {q^n}. (Here we are performing a standard abuse of notation by overloading the subscripts in the {{\mathbb F}} notation; thus {{\mathbb F}_q} denotes the field of order {q}, while {{\mathbb F}_n} denotes the extension of {{\mathbb F} = {\mathbb F}_q} of order {n}, so that we in fact have {{\mathbb F}_n = {\mathbb F}_{q^n}} if we use one subscript convention on the left-hand side and the other subscript convention on the right-hand side. We hope this overloading will not cause confusion.) Each point {x} in this extension (or, more suggestively, the affine line {{\mathbb A}^1({\mathbb F}_n)} over this extension) has a minimal polynomial – an irreducible monic polynomial whose roots consist the Frobenius orbit of {x}. Since the Frobenius action is periodic of period {n} on {{\mathbb F}_n}, the degree of this minimal polynomial must divide {n}. Conversely, every monic irreducible polynomial of degree {d} dividing {n} produces {d} distinct zeroes that lie in {{\mathbb F}_d} (here we use the classical fact that finite fields are perfect) and hence in {{\mathbb F}_n}. We have thus partitioned {{\mathbb A}^1({\mathbb F}_n)} into Frobenius orbits (also known as closed points), with each monic irreducible polynomial {f} of degree {d} dividing {n} contributing an orbit of size {d = \hbox{deg}(f) = \Lambda(f^{n/d})}. From this we conclude a geometric interpretation of the left-hand side of (18):

 \displaystyle  \sum_{\hbox{deg}(f) = n} \Lambda_{{\mathbb A}^1/{\mathbb F}}(f) = \sum_{x \in {\mathbb A}^1({\mathbb F}_n)} 1. \ \ \ \ \ (19)

The identity (18) thus is equivalent to the thoroughly boring fact that the number of {{\mathbb F}_n}-points on the affine line {{\mathbb A}^1} is equal to {q^n}. However, things become much more interesting if one then replaces the affine line {{\mathbb A}^1} by a more general (geometrically) irreducible curve {C} defined over {{\mathbb F}}; for instance one could take {C} to be an ellpitic curve

 \displaystyle  E = \{ (x,y): y^2 = x^3 + ax + b \} \ \ \ \ \ (20)

for some suitable {a,b \in {\mathbb F}}, although the discussion here applies to more general curves as well (though to avoid some minor technicalities, we will assume that the curve is projective with a finite number of {{\mathbb F}}-rational points removed). The analogue of {{\mathbb F}[t]} is then the coordinate ring of {C} (for instance, in the case of the elliptic curve (20) it would be {{\mathbb F}[x,y] / (y^2-x^3-ax-b)}), with polynomials in this ring producing a set of roots in the curve {C( \overline{\mathbb F})} that is again invariant with respect to the Frobenius action (acting on the {x} and {y} coordinates separately). In general, we do not expect unique factorisation in this coordinate ring (this is basically because Bezout’s theorem suggests that the zero set of a polynomial on {C} will almost never consist of a single (closed) point). Of course, we can use the algebraic formalism of ideals to get around this, setting up a zeta function

\displaystyle  \zeta_{C/{\mathbb F}}(s) = \sum_{{\mathfrak f}} \frac{1}{N({\mathfrak f})^s}

and a von Mangoldt function {\Lambda_{C/{\mathbb F}}({\mathfrak f})} as before, where {{\mathfrak f}} would now run over the non-trivial ideals of the coordinate ring. However, it is more instructive to use the geometric viewpoint, using the ideal-variety dictionary from algebraic geometry to convert algebraic objects involving ideals into geometric objects involving varieties. In this dictionary, a non-trivial ideal would correspond to a proper subvariety (or more precisely, a subscheme, but let us ignore the distinction between varieties and schemes here) of the curve {C}; as the curve is irreducible and one-dimensional, this subvariety must be zero-dimensional and is thus a (multi-)set of points {\{x_1,\ldots,x_k\}} in {C}, or equivalently an effective divisor {[x_1] + \ldots + [x_k]} of {C}; this generalises the concept of the set of roots of a polynomial (which corresponds to the case of a principal ideal). Furthermore, this divisor has to be rational in the sense that it is Frobenius-invariant. The prime ideals correspond to those divisors (or sets of points) which are irreducible, that is to say the individual Frobenius orbits, also known as closed points of {C}. With this dictionary, the zeta function becomes

\displaystyle  \zeta_{C/{\mathbb F}}(s) = \sum_{D \geq 0} \frac{1}{q^{\hbox{deg}(D)}}

where the sum is over effective rational divisors {D} of {C} (with {k} being the degree of an effective divisor {[x_1] + \ldots + [x_k]}), or equivalently

\displaystyle  Z( C/{\mathbb F}, T ) = \sum_{D \geq 0} T^{\hbox{deg}(D)}.

The analogue of (19), which gives a geometric interpretation to sums of the von Mangoldt function, becomes

\displaystyle  \sum_{N({\mathfrak f}) = q^n} \Lambda_{C/{\mathbb F}}({\mathfrak f}) = \sum_{x \in C({\mathbb F}_n)} 1

\displaystyle  = |C({\mathbb F}_n)|,

thus this sum is simply counting the number of {{\mathbb F}_n}-points of {C}. The analogue of the exponential identity (16) (or (2), (8), or (12)) is then

 \displaystyle  Z( C/{\mathbb F}, T ) = \exp( \sum_{n \geq 1} \frac{|C({\mathbb F}_n)|}{n} T^n ) \ \ \ \ \ (21)

and the analogue of the explicit formula (17) (or (5), (10) or (13)) is

 \displaystyle  |C({\mathbb F}_n)| = q^n - \sum_\alpha \alpha^n + c \ \ \ \ \ (22)

where {\alpha} runs over the (reciprocal) zeroes of {Z( C/{\mathbb F}, T )} (counting multiplicity), and {c} is an integer independent of {n}. (As it turns out, {c} equals {1} when {C} is a projective curve, and more generally equals {1-k} when {C} is a projective curve with {k} rational points deleted.)

To evaluate {Z(C/{\mathbb F},T)}, one needs to count the number of effective divisors of a given degree on the curve {C}. Fortunately, there is a tool that is particularly well-designed for this task, namely the Riemann-Roch theorem. By using this theorem, one can show (when {C} is projective) that {Z(C/{\mathbb F},T)} is in fact a rational function, with a finite number of zeroes, and a simple pole at both {1} and {1/q}, with similar results when one deletes some rational points from {C}; see e.g. Chapter 11 of Iwaniec-Kowalski for details. Thus the sum in (22) is finite. For instance, for the affine elliptic curve (20) (which is a projective curve with one point removed), it turns out that we have

\displaystyle  |E({\mathbb F}_n)| = q^n - \alpha^n - \beta^n

for two complex numbers {\alpha,\beta} depending on {E} and {q}.

The Riemann hypothesis for (untwisted) curves – which is the deepest and most difficult aspect of the Weil conjectures for these curves – asserts that the zeroes of {\zeta_{C/{\mathbb F}}} lie on the critical line, or equivalently that all the roots {\alpha} in (22) have modulus {\sqrt{q}}, so that (22) then gives the asymptotic

 \displaystyle  |C({\mathbb F}_n)| = q^n + O( q^{n/2} ) \ \ \ \ \ (23)

where the implied constant depends only on the genus of {C} (and on the number of points removed from {C}). For instance, for elliptic curves we have the Hasse bound

\displaystyle  |E({\mathbb F}_n) - q^n| \leq 2 \sqrt{q}.

As before, we have an important amplification phenomenon: if we can establish a weaker estimate, e.g.

 \displaystyle  |C({\mathbb F}_n)| = q^n + O( q^{n/2 + O(1)} ), \ \ \ \ \ (24)

then we can automatically deduce the stronger bound (23). This amplification is not a mere curiosity; most of the proofs of the Riemann hypothesis for curves proceed via this fact. For instance, by using the elementary method of Stepanov to bound points in curves (discussed for instance in this previous post), one can establish the preliminary bound (24) for large {n}, which then amplifies to the optimal bound (23) for all {n} (and in particular for {n=1}). Again, see Chapter 11 of Iwaniec-Kowalski for details. The ability to convert a bound with {q}-dependent losses over the optimal bound (such as (24)) into an essentially optimal bound with no {q}-dependent losses (such as (23)) is important in analytic number theory, since in many applications (e.g. in those arising from sieve theory) one wishes to sum over large ranges of {q}.

Much as the Riemann zeta function can be twisted by a Dirichlet character to form a Dirichlet {L}-function, one can twist the zeta function on curves by various additive and multiplicative characters. For instance, suppose one has an affine plane curve {C \subset {\mathbb A}^2} and an additive character {\psi: {\mathbb F}^2 \rightarrow {\bf C}^\times}, thus {\psi(x+y) = \psi(x) \psi(y)} for all {x,y \in {\mathbb F}^2}. Given a rational effective divisor {D = [x_1] + \ldots + [x_k]}, the sum {x_1+\ldots+x_k} is Frobenius-invariant and thus lies in {{\mathbb F}^2}. By abuse of notation, we may thus define {\psi} on such divisors by

\displaystyle  \psi( [x_1] + \ldots + [x_k] ) := \psi( x_1 + \ldots + x_k )

and observe that {\psi} is multiplicative in the sense that {\psi(D_1 + D_2) = \psi(D_1) \psi(D_2)} for rational effective divisors {D_1,D_2}. One can then define {\psi({\mathfrak f})} for any non-trivial ideal {{\mathfrak f}} by replacing that ideal with the associated rational effective divisor; for instance, if {f} is a polynomial in the coefficient ring of {C}, with zeroes at {x_1,\ldots,x_k \in C}, then {\psi((f))} is {\psi( x_1+\ldots+x_k )}. Again, we have the multiplicativity property {\psi({\mathfrak f} {\mathfrak g}) = \psi({\mathfrak f}) \psi({\mathfrak g})}. If we then form the twisted normalised zeta function

\displaystyle  Z( C/{\mathbb F}, \psi, T ) = \sum_{D \geq 0} \psi(D) T^{\hbox{deg}(D)}

then by twisting the previous analysis, we eventually arrive at the exponential identity

 \displaystyle  Z( C/{\mathbb F}, \psi, T ) = \exp( \sum_{n \geq 1} \frac{S_n(C/{\mathbb F}, \psi)}{n} T^n ) \ \ \ \ \ (25)

in analogy with (21) (or (2), (8), (12), or (16)), where the companion sums {S_n(C/{\mathbb F}, \psi)} are defined by

\displaystyle  S_n(C/{\mathbb F},\psi) = \sum_{x \in C({\mathbb F}^n)} \psi( \hbox{Tr}_{{\mathbb F}_n/{\mathbb F}}(x) )

where the trace {\hbox{Tr}_{{\mathbb F}_n/{\mathbb F}}(x)} of an element {x} in the plane {{\mathbb A}^2( {\mathbb F}_n )} is defined by the formula

\displaystyle  \hbox{Tr}_{{\mathbb F}_n/{\mathbb F}}(x) = x + \hbox{Frob}(x) + \ldots + \hbox{Frob}^{n-1}(x).

In particular, {S_1} is the exponential sum

\displaystyle  S_1(C/{\mathbb F},\psi) = \sum_{x \in C({\mathbb F})} \psi(x)

which is an important type of sum in analytic number theory, containing for instance the Kloosterman sum

\displaystyle  K(a,b;p) := \sum_{x \in {\mathbb F}_p^\times} e_p( ax + \frac{b}{x})

as a special case, where {a,b \in {\mathbb F}_p^\times}. (NOTE: the sign conventions for the companion sum {S_n} are not consistent across the literature, sometimes it is {-S_n} which is referred to as the companion sum.)

If {\psi} is non-principal (and {C} is non-linear), one can show (by a suitably twisted version of the Riemann-Roch theorem) that {Z} is a rational function of {T}, with no pole at {T=q^{-1}}, and one then gets an explicit formula of the form

 \displaystyle  S_n(C/{\mathbb F},\psi) = -\sum_\alpha \alpha^n + c \ \ \ \ \ (26)

for the companion sums, where {\alpha} are the reciprocals of the zeroes of {S}, in analogy to (22) (or (5), (10), (13), or (17)). For instance, in the case of Kloosterman sums, there is an identity of the form

 \displaystyle  \sum_{x \in {\mathbb F}_{p^n}^\times} e_p( a \hbox{Tr}(x) + \frac{b}{\hbox{Tr}(x)}) = -\alpha^n - \beta^n \ \ \ \ \ (27)

for all {n} and some complex numbers {\alpha,\beta} depending on {a,b,p}, where we have abbreviated {\hbox{Tr}_{{\mathbb F}_{p^n}/{\mathbb F}_p}} as {\hbox{Tr}}. As before, the Riemann hypothesis for {Z} then gives a square root cancellation bound of the form

 \displaystyle  S_n(C/{\mathbb F},\psi) = O( q^{n/2} ) \ \ \ \ \ (28)

for the companion sums (and in particular gives the very explicit Weil bound {|K(a,b;p)| \leq 2\sqrt{p}} for the Kloosterman sum), but again there is the amplification phenomenon that this sort of bound can be deduced from the apparently weaker bound

\displaystyle  S_n(C/{\mathbb F},\psi) = O( q^{n/2 + O(1)} ).

As before, most of the known proofs of the Riemann hypothesis for these twisted zeta functions proceed by first establishing this weaker bound (e.g. one could again use Stepanov’s method here for this goal) and then amplifying to the full bound (28); see Chapter 11 of Iwaniec-Kowalski for further details.

One can also twist the zeta function on a curve by a multiplicative character {\chi: {\mathbb F}^\times \rightarrow {\bf C}^\times} by similar arguments, except that instead of forming the sum {x_1+\ldots+x_k} of all the components of an effective divisor {[x_1]+\ldots+[x_k]}, one takes the product {x_1 \ldots x_k} instead, and similarly one replaces the trace

\displaystyle  \hbox{Tr}_{{\mathbb F}_n/{\mathbb F}}(x) = x + \hbox{Frob}(x) + \ldots + \hbox{Frob}^{n-1}(x)

by the norm

\displaystyle  \hbox{Norm}_{{\mathbb F}_n/{\mathbb F}}(x) = x \cdot \hbox{Frob}(x) \cdot \ldots \cdot \hbox{Frob}^{n-1}(x).

Again, see Chapter 11 of Iwaniec-Kowalski for details.

Deligne famously extended the above theory to higher-dimensional varieties than curves, and also to the closely related context of {\ell}-adic sheaves on curves, giving rise to two separate proofs of the Weil conjectures in full generality. (Very roughly speaking, the former context can be obtained from the latter context by a sort of Fubini theorem type argument that expresses sums on higher-dimensional varieties as iterated sums on curves of various expressions related to {\ell}-adic sheaves.) In this higher-dimensional setting, the zeta function formalism is still present, but is much more difficult to use, in large part due to the much less tractable nature of divisors in higher dimensions (they are now combinations of codimension one subvarieties or subschemes, rather than combinations of points). To get around this difficulty, one has to change perspective yet again, from an algebraic or geometric perspective to an {\ell}-adic cohomological perspective. (I could imagine that once one is sufficiently expert in the subject, all these perspectives merge back together into a unified viewpoint, but I am certainly not yet at that stage of understanding.) In particular, the zeta function, while still present, plays a significantly less prominent role in the analysis (at least if one is willing to take Deligne’s theorems as a black box); the explicit formula is now obtained via a different route, namely the Grothendieck-Lefschetz fixed point formula. I have written some notes on this material below the fold (based in part on some lectures of Philippe Michel, as well as the text of Iwaniec-Kowalski and also this book of Katz), but I should caution that my understanding here is still rather sketchy and possibly inaccurate in places.

— 1. l-adic sheaves and the etale fundamental group —

From the point of view of applications to analytic number theory, one can view Deligne’s theorems as providing bounds of square root cancellation type for various sums of the form

\displaystyle  \sum_{x \in U( {\mathbb F} )} K(x)

where {U} is some (quasi-projective) curve (or possibly a higher dimensional variety), and {K} is a certain type of “structured” function on the set {U({\mathbb F})} of {{\mathbb F}}-rational points on {U}, such that {K} is not entirely degenerate (e.g. constant). In particular, Deligne’s results allow one to obtain square root cancellation bounds of the form

 \displaystyle  |\sum_{x \in {\mathbb F}}^* K(x)| \ll q^{1/2} \ \ \ \ \ (29)

for certain non-degenerate structured functions {K} defined on {{\mathbb F}} except at a few points where {K} is “singular”, and the restricted sum {\sum^*} denotes a sum over the non-singular points of {K}.

The class of functions that can be treated by Deligne’s machinery is very general. The Weil conjectures for curves (and twisted curves) already allows one to obtain bounds of the shape (29) for several useful classes of functions {K(x)}, such as phases

\displaystyle  e_p( \frac{P(x)}{Q(x)} )

of rational functions, multiplicative characters {\chi: {\mathbb F}^\times \rightarrow {\bf C}}, or products of the two. Deligne’s results enlarge this class of functions to include Fourier transforms of existing structured functions, such as

\displaystyle  K(x) = \frac{1}{\sqrt{q}} \sum_{y \in {\mathbb F}^\times} e_q( \frac{a}{y} + xy);

in fact the class of structured functions is closed under a large number of operations, such as addition, multiplication, convolution or pullback, making it an excellent class to use in analytic number theory applications. The situation here is not dissimilar to that of characters {\chi: g \mapsto \hbox{tr}(\rho(g))} of finite-dimensional representations {\rho: G \rightarrow GL(V)} of some group {G}, in that the class of characters is also closed under basic operations such as addition and multiplication (which correspond to tensor sum and tensor product of representations). Indeed, the formal definition of a structured function will involve such a finite-dimensional representation, but with two technical details: the vector space {V} is not exactly defined over the complex numbers, but instead defined over the {\ell}-adic numbers for some prime {\ell} coprime to {q}, and also the group {G} is going to be the étale fundamental group {\pi_1^{et}(U({\mathbb F}))} of {U( {\mathbb F} )}.

We first describe (in somewhat vague terms) what the étale fundamental group {\pi_1^{et}(U)} of a connected variety {U=U/k} (or more generally, a connected Noetherian scheme) defined over a field {k}. Crucially, we do not require the underlying field {k} to be algebraically closed, and in our applications {k} will in fact be the finite field {{\mathbb F}}. The étale fundamental group is the common generalisation of the (profinite completion of the) topological fundamental group {\pi_1(M)} (applied to, say, a smooth complex variety), and of the absolute Galois group {\hbox{Gal}(k_{sep}/k)} of a field {k}. (This point of view is nicely presented in this recent book of Szamuely.) To explain this, we first consider the topological fundamental group {\pi^{top}_1(M) = \pi^{top}_1(M,p)} of a smooth connected manifold {M} at some base point {p} (the choice of which is not too important if one is willing to view the fundamental group up to conjugacy). This group is conventionally defined in terms of loops in {M} based at {p}, but the notion of a loop does not make much sense in either Galois theory or algebraic geometry. Fortunately, as observed by Grothendieck, there is an alternate way to interpret this fundamental group as follows. Let {N} be any covering space of {M}, with covering map {\phi: N \rightarrow M}; then above the base point {p} there is a discrete fibre {\phi^{-1}(\{p\})}, and given any point {q} in this fibre, every loop based at {p} lifts by monodromy to a path starting at {q} and ending at another point in the fibre {\phi^{-1}(\{p\})}. The endpoint is not affected by homotopy of the path, so this leads to an action {g: q \mapsto gq} of the fundamental group {\pi^{top}_1(M)} (or more precisely, of the opposite group {\pi^{top}_1(M)^{op}} to this fundamental group, but never mind this annoying technicality) on the fibre {\phi^{-1}(\{p\})} above {p} of any covering space. For instance, if we take the {m}-fold cover {m: {\bf R}/{\bf Z} \rightarrow {\bf R}/{\bf Z}} of the unit circle by itself formed by multplying by a natural number {m}, then the fibre may be identified with {{\bf Z}/m{\bf Z}}, and the fundamental group {\pi_1({\bf R}/{\bf Z}) \equiv {\bf Z}} acts on this fibre by translation.

The actions of the fundamental group are natural in the following sense: given a morphism {f: N \rightarrow N'} between two covering spaces {\phi: N \rightarrow M}, {\phi': N' \rightarrow M} of {M} (so that {\phi' \circ f = \phi}), then the action of the fundamental group is intertwined by {f}, thus {f( g q) = g f(q)} for any {q \in \phi^{-1}(\{p\})} and {g \in \pi^{top}_1(M)}. Conversely, every collection of actions on fibres {\phi^{-1}(\{p\})} that is natural in the above sense arises from a unique element of the fundamental group {\pi^{top}_1(M)}; this can be easily seen by working with the universal cover {\tilde M} of {M}, of which all other (connected) covers are quotients, and on whose fibre the fundamental group acts freely and transitively. Thus, one could define the fundamental group {\pi^{top}_1(M)} as the group of all possible collections of isomorphisms on the fibres {\phi^{-1}(\{p\})} above {p} which are natural in the above sense. (In category-theoretic terms, {\pi^{top}_1(M)} is the group of natural isomorphisms of the fibre functor that maps covers {\phi: N \rightarrow M} to fibres {\phi^{-1}(\{p\})}.)

There is an analogous way to view the absolute Galois group {\hbox{Gal}(k^{sep}/k)} of a field {k}. For simplicity we shall only discuss the case of perfect fields (such as finite fields) here, in which case there is no distinction between the separable closure {k^{sep}} and the algebraic closure {\overline{k}}, although the discussion below can be extended to the general case (and from a scheme-theoretic viewpoint it is in fact natural to not restrict oneself to the perfect case). The analogue of the covering spaces {N} of the manifold {M} are the finite extensions {L} of {k}. Here, one encounters a “contravariance” problem in pursuing this analogy: for covering spaces, we have a map {\pi: N \rightarrow M} from the covering space {N} to the base space {M}; but for field extensions, one instead has an inclusion {\iota: k \rightarrow L} from the base field to the extension field. To make the analogy more accurate, one has to dualise, with the role of the covering space {N} being played not by the extension {L}, but rather by the set {\hbox{Hom}(L \rightarrow \overline{k})} of all field embeddings of {L} into the algebraic closure {\overline{k}} (cf. the Yoneda lemma). (There is nothing too special about the algebraic closure {\overline{k}} here; any field which is in some sense “large enough” to support lots of embeddings of {L} would suffice.) This set projects down to {\hbox{Hom}(k \rightarrow \overline{k})}, which has a canonical point {p}, namely the standard embedding of {k} in {\overline{k}}; the fibre of {\hbox{Hom}(L \rightarrow \overline{k})} at {p} is then the set {\hbox{Hom}_k(L \rightarrow \overline{k})} of field embeddings of {L} to {\overline{k}} that fix {k}. It is a basic fact of Galois theory that if {L} is an extension of {k} of degree {n}, then this fibre {\hbox{Hom}_k(L \rightarrow \overline{k})} is a finite set of cardinality {n} (for perfect fields, this can be easily deduced from the primitive element theorem). The Galois group {\hbox{Gal}(\overline{k}/k)} then acts on these fibres {\hbox{Hom}_k(L \rightarrow \overline{k})} by left-composition, and one can verify that the action of a given Galois group element {g} is natural in the same category-theoretic sense as considered previously. Conversely, because the algebraic closure {\overline{k}} of {k} can be viewed as the direct limit of finite extensions of {k}, one can show that every natural isomorphism of these fibres (or more precisely, the fibre functor from {L} (or {\hbox{Hom}(L \rightarrow \overline{k})}) to {\hbox{Hom}_k(L \rightarrow \overline{k})}) comes from exactly one element of the Galois group {\hbox{Gal}(\overline{k}/k)}.

Now we can define the étale fundamental group {\pi^{top}_1(U)} of a general (connected, Noetherian) scheme {U} with a specified base point {p}. (Actually, one minor advantages of schemes is that they come with a canonical point to pick here, namely the generic point, although the dependence on the base point is not a major issue here in any event.) The analogue of covering spaces or finite extensions are now the finite étale covers {\phi: W \rightarrow U} of {U}: morphisms from another scheme {W} to {U} that are étale (which, roughly, is like saying {\phi} is a local diffeomorphism) and finite (roughly, this means that {W} locally looks like the product of {U} with a finite set). Unlike the previous two contexts, in which a universal covering object was available, the category of finite étale covers of a given scheme {U} usually does not have a universal object. (One can already see this in the category of algebraic Riemann surfaces: the universal cover of the punctured plane {{\bf C}^\times} ought to be the complex plane {{\bf C}} with covering map given by the exponential map {\exp: {\bf C} \rightarrow {\bf C}^\times}; this is what happens in the topological setting, but it is not allowed in the algebraic geometry setting because the exponential map is not algebraic.) Nevertheless, one can still define the étale fundamental group without recourse to a universal object, again by using actions on fibres {\phi^{-1}(\{p\})}. Namely, the étale fundamental group {\pi_1(U)} consists of all objects {g} which act by permutation on the fibres {\phi^{-1}(\{p\})} above {p} of every finite cover of {U}, in such a way that this action is natural in the category-theoretic sense. (To actually construct the étale fundamental group as a well-defined set requires a small amount of set-theoretic care, because strictly speaking the class of finite étale covers of {U} is only a class and not a set; but one can start with one representative from each equivalence class of finite étale covers first, with some designated morphisms between them with which to enforce the naturality conditions, and build the fundamental group from there by an inverse limit construction; see e.g. Szamuely’s book for details.) As the étale fundamental group is defined through its actions on finite sets rather than arbitrary discrete sets, it will automatically be a profinite group, and so differs slightly from the topological fundamental group in that regard. For instance, the punctured complex plane {{\bf C}^\times} has a topological fundamental group of {{\bf Z}}, but has an étale fundamental group of {\hat {\bf Z} = \lim_{\leftarrow} {\bf Z}/n{\bf Z}} – the profinite integers, rather than the rational integers. More generally, for complex varieties, the étale fundamental group is always the profinite completion of the topological fundamental group (this comes from a deep connection between complex geometry and algebraic geometry known as the Riemann existence theorem), but the situation can be more complicated in finite characteristic or in non-algebraically closed fields, due to the existence of étale finite covers that do not arise from classical topological covers. For instance, the étale fundamental group of a perfect field {k} (which one can view geometrically as a point over {k}) turns out to be the absolute Galois group of {k}.

The étale fundamental group is functorial: every morphism {\phi: U \rightarrow V} of schemes gives rise to a homomorphism {\phi_*: \pi^{et}_1(U) \rightarrow \pi^{et}_1(V)} of fundamental groups. Among other things, this gives rise to a short exact sequence

\displaystyle  1 \rightarrow \pi_1^{et}( U/\overline{k} ) \rightarrow \pi_1^{et}( U/k ) \rightarrow \hbox{Gal}(\overline{k}/k) \rightarrow 1

whenever {U} is a variety defined over a perfect field {k} (and hence also defined over its algebraic closure {\overline{k}}, via base change) which is geometrically connected (i.e. that {U/\overline{k}} is connected). (Again, it is more natural from a scheme-theoretic perspective to not restrict to the perfect case here, but we will do so here for sake of concreteness.) The groups {\pi_1^{et}( U/k )} and {\pi_1^{et}( U/\overline{k} )} are known as the arithmetic fundamental group and geometric fundamental group of {U} respectively. The latter should be viewed as a profinite analogue of the topological fundamental group of (a complex model of) {U}; this intuition is quite accurate in characteristic zero (due to the Riemann existence theorem mentioned earlier), but only partially accurate in positive characteristic {p>0} (basically, one has to work with the prime-to-{p} components of these groups in order to see the correspondence). (See for instance these notes of Milne for further discussion.)

The étale fundamental group {\pi_1^{et}(U/k)} can also be described “explicitly” in terms of Galois groups as follows. [Caution: I am not 100% confident in the accuracy of the assertions in this paragraph.] Let {K = k(U)} be the function field on {U}, and let {K^{sep}} be its separable closure, so that one can form the Galois group {\hbox{Gal}(K^{sep}/K)}. For any closed point {\overline{x} \in U_k} of {U} (basically, an orbit of {\hbox{Gal}(k^{sep}/k)}), we can form a local version {K_{\overline{x}}} of {K} (the Henselization of the discrete valuation ring associated to {\overline{x}}, which roughly speaking captures the formal power series around {\overline{x}}), and a local version {D_{\overline{x}} := \hbox{Gal}(K^{sep}_{\overline{x}}/K_{\overline{x}})} of the Galois group, known as the decomposition group at {\overline{x}}. As {K} embeds into {K_{\overline{x}}}, {D_{\overline{x}}} embeds into {\hbox{Gal}(K^{sep}/K)}. Inside {D_{\overline{x}}} is the inertia group {I_{\overline{x}}}, defined (I think) as the elements of {D_{\overline{x}}} which stabilise the residue field of {\overline{x}}. Informally, this group measures the amount of ramification present at {\overline{x}}. One can then identify the arithmetic fundamental group {\pi_1^{et}(U/k)} with the quotient of {\hbox{Gal}(K^{sep}/K)} by the normal subgroup {\langle I_{\overline{x}} \rangle_{\overline{x} \in U_k}} generated (as a normal subgroup) by all the inertia groups of closed points; informally, the étale fundamental group describes the unramified Galois representations of {\hbox{Gal}(K^{sep}/K)}. The geometric fundamental group has a similar description, but with {\hbox{Gal}(K^{sep}/K)} replaced by the smaller group {\hbox{Gal}(K^{sep}/\overline{k} K)}.

(Note: as pointed out to me by Brian Conrad, one can also identify {\pi_1^{et}(U/k)} with {\hbox{Gal}(K'/K)} where {K'} is the maximal Galois extension of {K} which is unramified at the discrete valuations corresponding to all the closed points of {U}. A similar description can be given for higher-dimensional schemes {U}, if one replaces closed points with codimension-1 points, thanks to Abhyankar’s lemma.)

We have seen that fundamental groups (or absolute Galois groups) act on discrete sets, and specifically on the fibres of covering spaces (or field extensions, or étale finite covers). However, fundamental groups also naturally act on other spaces, and in particular on vector spaces {V} over various fields. For instance, suppose one starts with a connected complex manifold {M} and considers the holomorphic functions on this manifold. Typically, there are very few globally holomorphic functions on {M} (e.g. if {M} is compact, then Liouville’s theorem will force a globally holomorphic function to be constant), so one usually works instead with locally holomorphic functions, defined on some open subset {U} of {M}. These local holomorphic functions then form a sheaf {{\mathcal H}} over {M}, with a vector space {{\mathcal H}(U)} of holomorphic functions on {U} being attached to each open set {U}, as well as restriction maps from {{\mathcal H}(U)} to {{\mathcal H}(U')} for every open subset {U'} of {U} which obey a small number of axioms which I will not reproduce here (see e.g. the Wikipedia page on sheaves for the list of sheaf axioms).

Now let {p} be a base point in {M}. We can then associate a natural complex vector space {V} to {p}, namely the space of holomorphic germs at {p} (the direct limit of {H(U)} for neighbourhoods {U} of {p}). Any loop based at {p} then induces a map from {V} to itself, formed by starting with a germ at {p} and performing analytic continuation until one returns to {p}. As analytic continuation is a linear operation, this map from {V} to itself is linear; it is also invertible by reversing the loop, so it lies in the general linear group {GL(V)}. From the locally unique nature of analytic continuation, this map is not affected by homotopy of the loop, and this therefore gives a linear representation {\rho_{{\mathcal H}}: \pi_1(U) \rightarrow GL(V)} of the fundamental group. (Actually, to be pedantic, it gives a representation of the opposite group {\pi_1(U)^{op}}, due to the usual annoyance that composition of functions works in the reverse order from concatenation of paths, but let us ignore this minor technicality.)

Of course, one can perform the same sort of construction for other sheaves over {M} of holomorphic sections of various vector bundles (using the stalk of the sheaf at {p} for the space of germs), giving rise to other complex linear representations of the fundamental group. In analogy with this, we will define the notion of a certain type of sheaf over a variety {U/{\mathbb F}} defined over a finite field {{\mathbb F}} in terms of such representations. Due to the profinite nature of the étale fundamental group, we do not work with complex representations, but rather with {\ell}-adic representations, for some prime {\ell} that is invertible in {{\mathbb F}}. However, as the {\ell}-adics have characteristic zero, we can always choose an embedding {\iota: {\bf Q}_\ell \rightarrow {\bf C}} into the complex numbers, although it is not unique (and.

We can now give (a special case of) the definition of an {\ell}-adic sheaf:

Definition 1 (Lisse sheaf) Let {U/{\mathbb F}} be a non-empty affine curve defined over a finite field {{\mathbb F} = {\mathbb F}_q}, and let {\ell} be a prime not equal to to the characteristic {p} of {{\mathbb F}} (i.e. {\ell} is invertible in {{\mathbb F}}). An {\ell}-adic lisse sheaf {{\mathcal F}} over {U} (also known as an {\ell}-adic local system) is a continuous linear representation {\rho_{\mathcal F}: \pi_1^{et}/U({\mathbb F}) \rightarrow GL(V)} of the arithmetic fundamental group of {U}, where {V} is a finite-dimensional vector space over the {\ell}–adics {{\mathbb Q}_\ell}. (Here, the continuity is with respect to the profinite topology on {\pi_1^{et}/U({\mathbb F})} and the {\ell}-adic topology on {GL(V)}.) We refer to {V} as the fibre of the sheaf. The dimension of {V} is called the rank of the sheaf.

One can define more general {\ell}-adic sheaves (not necessarily lisse) over more general schemes, but the definitions are more complicated, and the lisse case already suffices for many analytic number theory applications. (However, even if one’s applications only involve lisse sheaves, it is natural to generalise to arbitrary sheaves when proving the key theorems about these sheaves, in particular Deligne’s theorems.)

As we see from the above definition, lisse sheaves are essentially just a linear representation of the arithmetic fundamental group (and hence also of the geometric fundamental group). As such, one can directly import many representation-theoretic concepts into the language of {\ell}-adic lisse sheaves. For instance, one can form the direct sum {{\mathcal F}_1 \oplus {\mathcal F}_2} and direct product {{\mathcal F}_1 \otimes {\mathcal F}_2} of lisse sheaves by using the representation-theoretic direct sum and direct product, and one can take the contragradient sheaf {\check {\mathcal F}} of a sheaf {{\mathcal F}} by replacing the representation {\rho_{\mathcal F}} with its inverse transpose. Morphisms {\phi: U \rightarrow U'} of the underlying curve give rise to a pullback operation {{\mathcal F}' \rightarrow \phi^* {\mathcal F}'} from sheaves {{\mathcal F'}} over {U'} to sheaves {\phi^* {\mathcal F}'} of {U}. (There is also an important pushforward operation, but it is much more difficult to define and study.) We also have the notion of a trivial sheaf (in which {\rho_{\mathcal F}} is the identity), an irreducible sheaf (which cannot be decomposed as an extension of a lower rank sheaf), a semisimple sheaf (a sheaf which factors as the direct sum of irreducibles), or an isotypic sheaf (the direct sum of isomorphic sheaves). By replacing the arithmetic fundamental group with its geometric counterpart, one also has geometric versions of many of the above concepts, thus for instance one can talk about geometrically irreducible sheaves, geometrically semisimple sheaves, etc.

Now let {\overline{x} \in U_{{\mathbb F}}} be a closed point in {U} of degree {n}, with its associated decomposition group {D_{\overline{x}}} and inertia group {I_{\overline{x}}}. The quotient {D_{\overline{x}}/I_{\overline{x}}} can be viewed as the arithmetic fundamental group of {\overline{x}}, or equivalently the absolute Galois group of the residue field at {\overline{x}}, and this quotient embeds into the arithmetic fundamental group {\pi_1^{et}(U({\mathbb F}))} of {U({\mathbb F})} since {\overline{x}} embeds into {U({\mathbb F})}. On the other hand, as {\overline{x}} has degree {n}, the residue field is isomorphic to the degree {n} extension {{\mathbb F}_n} of {{\mathbb F}}, so {D_{\overline{x}}/I_{\overline{x}}} is isomorphic to the absolute Galois group of {{\mathbb F}_n}. This latter group is generated (topologically) by the arithmetic Frobenius map {x \mapsto x^{q^n}}, and also by its inverse, known as the geometric Frobenius map. So the geometric Frobenius map is well defined (up to conjugacy) as an element of {D_{\overline{x}}/I_{\overline{x}}} and hence {\pi_1^{et}(U({\mathbb F}))}. By abuse of notation, we will refer to elements of this conjugacy class in {\pi_1^{et}(U({\mathbb F}))} as {\hbox{Frob}_{\overline{x}}}, bearing in mind that this object is only defined up to conjugacy. If we have a lisse {\ell}-adic sheaf {{\mathcal F}}, then {\rho_{{\mathcal F}}( \hbox{Frob}_{\overline{x}} )} is defined up to conjugation, which implies that the trace {\hbox{tr}( \rho_{{\mathcal F}}( \hbox{Frob}_{\overline{x}} ) | V )} on {V} is well-defined as an element of {{\bf Q}_\ell}. (Actually, a technical point: one should restrict the trace from {V} to the subspace {V^{I_{\overline{x}}}} fixed by the inertia group of {\overline{x}}, but as long as there is no ramification at {\overline{x}}, this is all of {V}; we will ignore this technicality, as in practice we will delete the points of {U} in which ramification occurs.) Fixing some embedding {\iota: {\bf Q}_\ell \rightarrow {\bf C}}, we can then form the trace function

\displaystyle  K_{\mathcal F}(\overline{x}) := \iota( \hbox{tr}( \rho_{{\mathcal F}}( \hbox{Frob}_{\overline{x}} ) | V )

which is a function from {U_{\mathbb F}} to {{\bf C}}, and in particular a function from {U({\mathbb F})} to {{\bf C}}. (A certain subclass of) these trace functions will serve as the “structured” functions mentioned at the start of this section.

Now an important definition. For each unramified closed point {\overline{x}}, {\rho_{{\mathcal F}}( \hbox{Frob}_{\overline{x}} )} is an invertible linear transformation on the vector space {V}, which has dimension {r} equal to the rank of {{\mathcal F}}. In particular, this transformation has {r} eigenvalues {\alpha_1,\ldots,\alpha_r} (depending on {\overline{x}}, of course) in {{\bf Q}_\ell}, and hence in {{\bf C}} after selecting an embedding {\iota: {\bf Q}_\ell \rightarrow {\bf C}}. We say that the sheaf {{\mathcal F}} is pure of weight {w} for some real number {w} if all of these eigenvalues (and their Galois conjugates) have magnitude exactly {|k_{\overline{x}}|^{w/2} = q^{w\hbox{deg}(\overline{x})/2}}, where {k_{\overline{x}}} is the residue field at {\overline{x}}. There is also the weaker concept of being mixed of weight {w}, in which the magnitude of the eigenvalues and their conjugates is merely assumed to be bounded above by {|k_{\overline{x}}|^{w/2} = q^{w\hbox{deg}(\overline{x})/2}}. (A technical remark: this upper bound are initially only assumed for unramified closed points, but a result of Deligne allows one to extend this upper bound to ramified points also.) If the sheaf is pure or mixed of weight {w}, then one clearly has the pointwise bound

\displaystyle  |K_{\mathcal F}(\overline{x})| \leq r q^{w\hbox{deg}(\overline{x})/2};

in particular, for sheaves of weight zero and bounded rank, the trace function is {O(1)}.

The trace functions {K_{{\mathcal F}}(\overline{x})} resemble characters of representations. For instance, taking the direct sum or product of two sheaves results in taking the sum or product of the two trace functions. If two sheaves have weight {w}, then their direct sum has weight {w} as well, while if two sheaves have weights {w_1,w_2}, then their direct product has weight {w_1+w_2}. Taking the contragradient {\check {\mathcal F}} of a sheaf of weight {w} results in a sheaf of weight {-w}; in the weight zero case, the trace function simply gets conjugated. Pulling back a sheaf by some morphism of curves preserves the weight of that sheaf, and pulls back the trace functions accordingly.

It is a convenient fact that pure sheaves automatically have a “geometric semisimplification” which is a pure sheaf of the same weight, and whose trace function is identical to that of the original sheaf. Furthermore, the geometrically irreducible components of the geometrically semisimple sheaf have the same weight as the original sheaf. Because of this, one can often reduce to the study of geometrically irreducible pure sheaves.

In practice, one can normalise the weight of a pure or mixed sheaf to zero by the following simple construction. Given any algebraic integer {\alpha} over the {\ell}-adics, we can define the Tate sheaf associated to {\alpha} to be the unique rank one continuous representation {\rho: \pi_1^{et}(U/{\mathbb F}) \rightarrow {\bf Q}_\ell^\times = GL_1({\bf Q}_\ell)} that acts trivially on the geometric fundamental group, and maps the Frobenius map {\hbox{Frob} \in \hbox{Gal}(\overline{\mathbb F}/{\mathbb F}) \subset \pi_1^{et}(U/{\mathbb F})} of {{\mathbb F}} to {\alpha}. If we set {\alpha = q^{w/2}}, then this is a pure sheaf of weight {w}, and the associated trace function {K(\overline{x})} is equal to {q^{w\hbox{deg}(\overline{x})/2}}; it is geometrically trivial but arithmetically non-trivial (if {\alpha \neq 1}), and conversely all geometrically trivial and geometrically semisimple sheaves come from direct sums of Tate sheaves. Given any other pure or mixed sheaf {{\mathcal F}} of some weight {w'}, one can then tensor with the Tate sheaf of weight {w} to create a pure or mixed sheaf {{\mathcal F}(w)} of weight {w'+w}, which at the level of trace functions amounts to multiplying the trace function by {q^{w\hbox{deg}(\overline{x})/2}}. In particular, setting {w=-w'} one can perform a “Tate twist” to normalise these sheaves to be of weight zero.

There is also the notion of the “complexity” of an {\ell}-adic sheaf, measured by a quantity known as the conductor of the sheaf. This quantity is a bit complicated to define here, but basically it incorporates the genus of the underlying curve {U}, the rank of the sheaf, the number of singularities of the sheaf, and something called the Swan conductor at each singularity of that sheaf; bounding the conductor then leads to a bound on all of these quantities. The conductor behaves well with respect to various sheaf-theoretic operations; for instance the direct sum or product of sheaves of bounded conductor will also be a sheaf of bounded conductor. I’ll use “bounded complexity” in place of “bounded conductor” in the text that follows.

The “structured functions” mentioned at the start of this section on a curve {C} are then precisely the trace functions {K_{{\mathcal F}}} associated to sheaves on open dense subsets {U} of this curve {C} formed by deleting a bounded number of points at most from {C}, which are pure of weight zero and of bounded complexity. As discussed above, this class of functions is closed under pointwise sum, pointwise product, complex conjugation, and pullback, and are also pointwise bounded. It is also a deep fact (essentially due to Deligne, Laumon, and Katz) that this class is closed under other important analytic operations, such as Fourier transform and convolution.

Now we give some basic examples of structured functions, which can be combined with each other using the various closure properties of such functions discussed above. First we show how any additive character {\psi: {\mathbb F} \rightarrow {\bf C}^\times} can be interpreted as an structured function on the affine line {{\mathbb A}^1}. The affine line {{\mathbb A}^1} is covered by itself via the finite étale covering map {\phi: {\mathbb A}^1 \rightarrow {\mathbb A}^1} defined by {\phi(x) := x - x^q}. The fibre of this map at {0} is just the field {{\mathbb F}}, and so the arithmetic fundamental group {\pi_1^{et}({\mathbb A}^1)} acts on {{\mathbb F}}. One can show that this action is a translation action (because of the translation symmetry of this covering space), and so we have a map from {\pi_1^{et}({\mathbb A}^1)} to {{\mathbb F}}, which on composition with {\psi} (and pulling back to {{\bf Q}_\ell^\times}) gives a rank one sheaf, called the Artin-Schrier sheaf associated to {\psi}. The trace function here is just {\psi}, and this is clearly a pure sheaf of weight {0}; it also has bounded complexity (indeed, the genus is zero and the only singularity is at {\infty}, and the Swan conductor there can be computed to be {1}).

In a similar vein, any multiplicative character {\chi: {\mathbb F}^\times \rightarrow {\bf C}^\times} can be viewed as an structured function on the multiplciative group {{\mathbb G}_m := {\mathbb A}^1 \backslash \{0\}} by a similar construction, using {x / x^q} instead of {x - x^q} (which is still an étale covering map, thanks to a baby case of Hilbert’s Theorem 90), giving rise to a rank one, bounded complexity pure sheaf of weight {0} on {{\mathbb G}_m} known as the Kummer sheaf, whose trace function is just {\chi}.

These two examples, combined with the closure operations defined previously, already give a large number of useful structured functions, such as the function {x \mapsto \psi( f(x) ) \chi(g(x))} on the affine line with boundedly many points removed, where {f, g} are on that affine line. But there is a deep and powerful additional closure property due to Deligne, Laumon, and Katz: if {{\mathcal F}} is a pure sheaf of weight zero on the affine line (with boundedly many points removed) of bounded complexity that does not contain any Artin-Schrier components, and {\psi: {\mathbb F} \rightarrow {\bf C}^\times} is a non-trivial additive character, then there is a “Fourier transform” {NFT_\psi({\mathcal F})} of {{\mathcal F}}, which is another pure sheaf of weight zero and bounded complexity with no Artin-Schrier components, with the property that the trace function of {NFT_\psi({\mathcal F})} is the Fourier transform of the trace function of {{\mathcal F}} with respect to {\psi} on {{\mathbb F}}:

\displaystyle  K_{NFT_\psi({\mathcal F})}(x) = \frac{1}{\sqrt{q}} \sum_{y \in {\mathbb F}} K( y ) \psi(xy).

See Theorem 8.2.3 of this book of Katz for details. (The construction here is analogous to that used in the Fourier-Mukai transform, although I do not know how tight this analogy is.) This closure under Fourier transforms also implies a closure property with respect to convolutions, by the usual intertwining relationship between convolution and multiplication provided by the Fourier transform. Using these additional closure properties, one can now add many new and interesting examples of structured functions, such as the normalised Kloosterman sums

\displaystyle  K(a) = \frac{1}{\sqrt{q}} \sum_{x,y \in {\mathbb F}: xy=a} \psi( x + y )

or more generally the hyper-Kloosterman sums

\displaystyle  K_k(a) = \frac{1}{q^{(k-1)/2}} \sum_{x_1,\ldots,x_k \in {\mathbb F}: x_1 \ldots x_k =a} \psi( x_1 + \ldots + x_k )

for some non-principal additive character {\psi}. These can then be combined with the previous examples of structured functions as before, for instance Kloosterman correlations {x \mapsto K_k(x) \overline{K_k(x+l)} \psi(x)} would now also be examples of structured functions.

We have now demonstrated that the class of structured functions {K(x)} contains many examples of interest to analytic number theory, but we have not yet done anything with this class, in particular we have not obtained any control on “exponential sums” such as

 \displaystyle  \sum_{x \in U({\mathbb F})} K(x) \ \ \ \ \ (30)

beyond the trivial bound of {O(q)} that comes from the pointwise bound on {K(x)} (and the elementary “Schwarz-Zippel” fact that a bounded complexity curve has at most {O(q)} points over {{\mathbb F}_q}). However, such control can be obtained through the important Grothendieck-Lefschetz fixed point formula

 \displaystyle  \sum_{x \in U({\mathbb F}_n)} K_{\mathcal F}(x) = \sum_{i=0}^2 (-1)^i \hbox{Tr}( \hbox{Frob}^n | H^i_c(U/\overline{\mathbb F}, {\mathcal F}) ) \ \ \ \ \ (31)

for any {n \geq 1}, where {H^i_c(U/\overline{\mathbb F}, {\mathcal F})} are the {\ell}-adic cohomology groups with compact support in {U/\overline{\mathbb F}} and coefficients in {{\mathcal F}}. These groups are defined through the general homological algebra machinery of derived categories (but can also be interpreted using either Galois cohomology or sheaf cohomology), and I do not yet have a sufficiently good understanding of these topics to say much more about these groups, other than that they turn out to be finite-dimensional vector spaces over {{\bf Q}_\ell}, and carry an action of the (geometric) Frobenius map of {{\mathbb F}}. (In fact they have a richer structure than this, being sheaves over {\hbox{Spec}({\mathbb F})}; this is useful when trying to iterate Deligne’s theorem to control higher-dimensional exponential sums, but we won’t directly use this structure here.) This formula is analogous to the explicit formulae (5), (10), (13), (17), (22), (26), (27) from the introduction, and in fact easily implies the latter four explicit formulae. Specialising to the case {n=1}, we see that the sum (30) takes the form

\displaystyle  \hbox{Tr}( \hbox{Frob} | H^0_c(U/\overline{\mathbb F}, {\mathcal F}) ) - \hbox{Tr}( \hbox{Frob} | H^1_c(U/\overline{\mathbb F}, {\mathcal F}) ) + \hbox{Tr}( \hbox{Frob} | H^2_c(U/\overline{\mathbb F}, {\mathcal F}) ).

To proceed further, we need to understand the eigenvalues of the Frobenius map on these cohomology groups (and we also need some control on the dimensions of these groups, i.e. on Betti numbers). This can be done by the following deep result of Deligne, essentially the main result in his second proof of the Weil conjectures:

Theorem 2 (Deligne’s Weil II) If {{\mathcal F}} is a lisse {\ell}-adic sheaf, pure of weight {w}, and {i \geq 0}, then any eigenvalue of Frobenius on {H^i_c(U/\overline{\mathbb F}, {\mathcal F})} has magnitude at most {q^{(w+i)/2}}, as does any of its Galois conjugates.

If we form the zeta function

\displaystyle  Z( {\mathcal F}, T ) = \exp( \sum_{n \geq 1} \frac{\sum_{x \in U({\mathbb F}_n)} K_{\mathcal F}(x)}{n} T^n )

in analogy with (2), (8), (12), (16), (21), or (25), Deligne’s theorem is equivalent to the Riemann hypothesis for {Z}, or at least to the “important” half of that hypothesis, namely that the zeroes have magnitude at most {\sqrt{q}}. In many cases, one can use Poincaré duality to derive a functional equation for {Z} which shows that the zeroes in fact have magnitude exactly {\sqrt{q}}, but for the purposes of upper bounds on exponential sums, it is only the upper bound on the zeroes which is relevant. Interestingly, once one has Deligne’s theorem, the zeta function {Z} plays very little role in applications; however, the zeta function is used to some extent in the proof of Deligne’s theorem. (In particular, my understanding is that Deligne establishes a preliminary zero-free region for this zeta function analogous to the classical zero-free region for the Riemann zeta function, which he then amplifies using a device similar to the amplification tricks mentioned previously.)

From Deligne’s theorem we conclude an important upper bound for (30) for structured functions:

\displaystyle  |\sum_{x \in U({\mathbb F})} K_{\mathcal F}(x)| \leq \hbox{dim}( H^0_c(U/\overline{\mathbb F}, {\mathcal F}) ) + q^{1/2} \hbox{dim}( H^1_c(U/\overline{\mathbb F}, {\mathcal F}) )

\displaystyle  + q \hbox{dim}( H^2_c(U/\overline{\mathbb F}, {\mathcal F}) ).

To use this bound, we need bounds on the dimensions of the the cohomology groups, and we need the {H^2_c} cohomology group to be trivial in order to get a non-trivial bound on the exponential sum {\sum_{x \in U({\mathbb F})} K_{\mathcal F}(x)}.

As is usual in cohomology, the extreme cohomology groups {H^0_c, H^2_c} are relatively easy to compute; for instance, if {U} is affine {H^0_c} can be shown to vanish, and {H^2_c} can be shown to vanish when {{\mathcal F}} is geometrically irreducible and non-trivial. In any case, when {{\mathcal F}} has bounded complexity, both of these groups have bounded dimension. This leads to the bound

\displaystyle  |\sum_{x \in U({\mathbb F})} K_{\mathcal F}(x)| \leq O(1) + q^{1/2} \hbox{dim}( H^1_c(U/\overline{\mathbb F}, {\mathcal F}) )

when {{\mathcal F}} is geometrically irreducible and non-trivial with bounded complexity. Finally, to control the dimension of {H^1_c}, one uses a variant of the Grothendieck-Lefschetz formula, namely the Euler-Poincaré formula

 \displaystyle  \sum_{i=0}^2 (-1)^i \hbox{dim}( H^i_c(U/\overline{\mathbb F}, {\mathcal F}) ) = r \chi(U) - \sum_{\overline{x} \in \hbox{Sing}({\mathcal F})} \hbox{Swan}_{\overline{x}}({\mathcal F}) \ \ \ \ \ (32)

where {r} is the rank of {{\mathcal F}}, {\chi(U)} is the geometric Euler characteristic of {U} (i.e. {2-2g-k}, where {k} is the number of points {U} omits from its projective closure {C = \overline{U}}), and {\hbox{Swan}_{\overline{x}}({\mathcal F})} are the Swan conductors. (A side note: this identity formally suggests that there is some extension of the Grothendieck-Lefschetz formula to the {n=0} case, with the right-hand side of (32) being interpretable as some sort of sum over the “field with one element“, whatever that means. I wonder if such an interpretation has been fleshed out further?) All the quantities on the right-hand side of (32) are bounded if {{\mathcal F}} has bounded complexity (basically by the definition of complexity), so we conclude that {H^1_c} has bounded dimension if {{\mathcal F}} has bounded complexity. (Here we are relying on the fact that {U} is one-dimensional, so that there is only one “difficult” Betti number to understand, which can then be recovered through the Euler characteristic; the situation is more complicated, though still reasonably well under control, in higher dimensions.) So we finally recover the square root cancellation bound

\displaystyle  |\sum_{x \in U({\mathbb F})} K_{\mathcal F}(x)| \ll q^{1/2}

whenever {{\mathcal F}} is geometrically irreducible and geometrically non-trivial. We thus obtain the more general bound

\displaystyle  \sum_{x \in U({\mathbb F})} K_{\mathcal F}(x) = cq + O(q^{1/2})

for any geometrically semisimple lisse sheaf {{\mathcal F}} of bounded complexity and pure of weight {0}, where {c} is an algebraic integer reflecting the Tate twists present in the geometrically trivial component of {{\mathcal F}} (in particular, {c=0} if there is no such component). Thus we have a strong “structure vs randomness” dichotomy for this class of functions: either {K} has a geometrically trivial component, or else {K} exhibits square root cancellation. Informally, we are guaranteed square root cancellation for structured functions unless there is a clear geometric reason why such cancellation is not available.

From Schur’s lemma, we then conclude the almost orthogonality relation

\displaystyle  \sum_{x \in U({\mathbb F})} K_{\mathcal F}(x) \overline{K_{{\mathcal F}'}(x)} = c \delta_{{\mathcal F} \equiv {\mathcal F}'} q + O( q^{1/2} )

for two geometrically irreducible sheaves {{\mathcal F}, {\mathcal F}'} of bounded complexity and weight {0}, where {\delta_{{\mathcal F} \equiv {\mathcal F}'}} is {1} when {{\mathcal F}, {\mathcal F}'} are geometrically isomorphic, and {0} otherwise, and {c} is an algebraic integer measuring the Tate twists in the geometric isomorphism between {{\mathcal F}} and {{\mathcal F}'} (in particular {c=1} when {{\mathcal F}={\mathcal F}'}). This gives a useful dichotomy: two irreducible structured functions on a curve with boundedly many points removed are either multiples of each other by a root of unity (after restricting to their common domain of definition), or else have an inner product of {O(q^{1/2})}. (Among other things, this provides polynomial bounds on the number of distinct structured functions of bounded complexity, by using the Kabatjanskii-Levenstein bound mentioned in the previous post.)

A typical application of this dichotomy is to correlations of the form {\sum_{x \in {\mathbb F}}^* K(x+l) \overline{K(x)}} of some structured function {K} (where the asterisk denotes a restriction to those {x} that avoid the singularities of {K}); if {q} is prime, one can show that this correlation is {O(q^{1/2})} for all non-zero {l}, unless {K} correlates with a linear phase (i.e. it has an Artin-Schrier component). In a similar fashion, {\sum_{x \in {\mathbb F}}^* K(x+l) \overline{K(x)} \psi(mx)} can be shown to be {O(q^{1/2})} for all non-zero {l} and any {m}, unless {K} correlates with a quadratic phase. These facts are reminiscent of the inverse theorems in the theory of the Gowers uniformity norms, but in the case of structured functions one gets extremely good bounds (either perfect correlation, or square root cancellation). A bit more generally, one can study correlations of the form {\sum_{x \in {\mathbb F}}^* K(x) K(\gamma x)}, where {\gamma} is a fractional linear transformation, leading to an analysis of the automorphy group of {K} with respect to the {SL_2} action; see this note of Fouvry, Kowalski and Michel for details.
