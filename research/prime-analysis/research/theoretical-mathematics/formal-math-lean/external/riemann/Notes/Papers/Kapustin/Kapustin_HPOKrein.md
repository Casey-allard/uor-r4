Here is the transcription of the paper into English with the mathematical expressions formatted in LaTeX.

***

**Siberian Mathematical Journal, 2024, Vol. 65, No. 1, pp. 72–75.**
© The Author(s), 2024. This article is an open access publication.
Russian Text © The Author(s), 2024, published in *Sibirskii Matematicheskii Zhurnal*, 2024, Vol. 65, No. 1, pp. 87–91.

# HILBERT–PÓLYA OPERATORS IN KREIN SPACES

**V. V. Kapustin**
**UDC 517.984**

**Abstract**—We construct some class of selfadjoint operators in the Krein spaces consisting of functions on the straight line $\{ \text{Re } s = \frac{1}{2} \}$. Each of these operators is a rank-one perturbation of a selfadjoint operator in the corresponding Hilbert space and has eigenvalues complex numbers of the form $\frac{1}{s(1-s)}$, where $s$ ranges over the set of nontrivial zeros of the Riemann zeta-function.

**DOI:** 10.1134/S0037446624010087
**Keywords:** Riemann zeta-function, eigenvalue, perturbation, selfadjoint operator

A Hilbert–Pólya operator is a hypothetically existing selfadjoint operator in a Hilbert space whose spectrum coincides with the set of all nontrivial zeros of the Riemann zeta-function rotated onto the real line. The idea of constructing a Hilbert–Pólya operator represents one of the approaches to proving the Riemann hypothesis about the zeros of the zeta-function. The author’s development of his previous works (see [1]) on the study of the de Branges space associated with the zeta-function led to constructing a family of selfadjoint operators with the desired spectrum. However, it turned out that the domains of these operators are Krein spaces with an indefinite metric rather than Hilbert spaces as required in connection with the Riemann hypothesis. Moreover, it turned out that the description of this construction can be presented independently and does not require connection with the de Branges space which was used in the original author’s construction. This note is devoted to the construction. We utilize a few well-known facts that can be found in books on the general theory of the zeta-function; for example, in [2].

Let $p$ be a real-valued measurable weight function on the straight line $\mathfrak{L} = \{ s : \text{Re } s = \frac{1}{2} \}$. The Krein space is defined by the inner product
$$ [f,g] = \int f(s)\overline{g(s)}p(s)i \, ds, $$
which is not assumed positive definite; integration (here and below) is carried out over the straight line $\mathfrak{L}$ from top down. The inner product in the corresponding Hilbert space has the form $\int f(s)\overline{g(s)}|p(s)|i \, ds$; the norm in the corresponding Krein space can be defined as the norm of this Hilbert space.

If $s \in \mathfrak{L}$ then $\bar{s} = 1 - s$. We assume additionally the symmetry condition $p(1-s) = p(s)$. In the Krein space, define the subspace $\mathscr{K}$ of the functions $f$ such that $f(1-s) = f(s)$.

We put
$$ u(s) = \frac{2\xi(s)}{s(s-1)} = \pi^{-s/2}\Gamma\left(\frac{s}{2}\right)\zeta(s), $$
where $\xi$ is the Riemann xi-function,
$$ \xi(s) = \frac{1}{2} s(s-1)\pi^{-s/2}\Gamma\left(\frac{s}{2}\right)\zeta(s). $$
It is well known that $\xi(1-s) = \xi(s)$ and the set of the zeros of $\xi$ coincides with the set of all nontrivial zeros of the zeta-function. Suppose that $u \in \mathscr{K}$ and consider the following operator on $\mathscr{K}$:
$$ f \mapsto \frac{f}{|s|^2} - [f, u]u. \quad (*) $$

The operator of division by $|s|^2 = s(1-s)$ is selfadjoint both in the Krein space and in the Hilbert space, whereas the rank-one operator perturbing it is selfadjoint only in the Krein space.

Let $\alpha > 0$ be a number such that
$$ |\zeta(s)| \leq \text{const} \cdot |s|^\alpha $$
for $s \in \mathfrak{L}$. The famous Lindelöf hypothesis is that this holds for all $\alpha > 0$. By the classical result of Hardy and Littlewood, this estimate holds for all $\alpha > \frac{1}{6}$. Later this estimate was improved many times; and by date, as far as we know, the inequality was proved by Bourgain for all $\alpha > \frac{13}{84}$. Take a number
$$ \beta > 2\alpha + \frac{1}{2}. $$

**Theorem.** Let $\phi$ be an analytic function in the half-plane $\{ s : \text{Re } s > \frac{1}{2} \}$ such that $\overline{\phi(\bar{s})} = \phi(s)$, $\phi(1) = 1$, and $|\phi(s)| \leq \frac{\text{const}}{|s|^\beta}$ for $\text{Re } s > \frac{1}{2}$. For $s \in \mathfrak{L}$, use the boundary values of $\phi$ to define
$$ q(s) = \frac{(2\pi)^{s-2}}{\Gamma(s)} \phi(s), \quad p(s) = \frac{q(s) + q(1-s)}{2} = \text{Re } q(s), $$
and construct the corresponding Krein space from $p$. Then operator $(*)$ is well defined therein, and if $\xi(\sigma) = 0$ then $\lambda = \frac{1}{\sigma(1-\sigma)}$ is its eigenvalue.

The existence of a selfadjoint operator in a Hilbert space with the above eigenvalues would mean the validity of the Riemann hypothesis by which all nontrivial zeros of the zeta-function lie on the straight line $\mathfrak{L}$. A selfadjoint operator whose spectrum points are of the form $\frac{1}{\sigma(1-\sigma)}$, where $\xi(\sigma)=0$, is a linear combination of the two resolvents of the Hilbert–Pólya operator formulas for which are obtained from the decomposition $\frac{1}{\sigma(1-\sigma)} = \frac{1}{\sigma} + \frac{1}{1-\sigma}$. As $\phi$ in the theorem, one can take the functions of the form $\frac{1}{s^\beta}$ for $\beta > 2\frac{13}{84} + \frac{1}{2} = \frac{17}{21}$; in particular, we can take $\phi(s) = \frac{1}{s}$.

**Proof.** For $u \in \mathscr{K}$ to hold, it suffices that the integral
$$ \int |u(s)|^2 |q(s)| i \, ds = \int |\pi^{-s}| \left| \Gamma\left(\frac{s}{2}\right) \right|^2 |\zeta(s)|^2 \frac{|(2\pi)^{s-2}|}{|\Gamma(s)|} |\phi(s)| i \, ds $$
$$ \leq \text{const} \cdot \int \left| \frac{\left( \Gamma\left(\frac{s}{2}\right) \right)^2}{\Gamma(s)} \right| |s|^{2\alpha - \beta} i \, ds $$
be finite. Estimate the factor in the integrand by using the classical doubling formula for the gamma-function
$$ \Gamma\left(\frac{s}{2}\right) \Gamma\left(\frac{s}{2} + \frac{1}{2}\right) = 2^{1-s}\sqrt{\pi}\Gamma(s). $$
We infer
$$ \frac{\left(\Gamma\left(\frac{s}{2}\right)\right)^2}{\Gamma(s)} = \frac{\Gamma\left(\frac{s}{2}\right)}{\Gamma\left(\frac{s}{2} + \frac{1}{2}\right)} \frac{\Gamma\left(\frac{s}{2}\right)\Gamma\left(\frac{s}{2} + \frac{1}{2}\right)}{\Gamma(s)} = \frac{\Gamma\left(\frac{s}{2}\right)}{\Gamma\left(\frac{s}{2} + \frac{1}{2}\right)} 2^{1-s} \sqrt{\pi}, $$
and since $\frac{\Gamma(s/2)}{\Gamma(s/2 + 1/2)}$ behaves like $(s/2)^{-1/2}$, the modulus of this expression is estimated by $\frac{\text{const}}{\sqrt{|s|}}$. From this it is clear that if $\beta > 2\alpha + \frac{1}{2}$ then the integral is finite and so $u \in \mathscr{K}$.

The equation for the eigenvectors
$$ \frac{f(s)}{s(1-s)} - [f, u]u(s) = \lambda f(s) $$
implies that the eigenfunction $f$ is of the form
$$ f(s) = u(s) \frac{s(1-s)}{1 - \lambda s(1-s)} $$
and must satisfy the condition
$$ [f, u] = 1, $$
which must be checked for proving the theorem when $\lambda = \frac{1}{\sigma(1-\sigma)}$ for $\sigma$ such that $\xi(\sigma) = 0$. We have
$$ 2[f, u] = \int f(s)\overline{u(s)}q(s) i \, ds + \int f(s)\overline{u(s)}q(1-s) i \, ds. $$
For the first summand on the right-hand side, applying the doubling formula for the gamma-function, we infer
$$ \int f(s)\overline{u(s)}q(s) i \, ds = \int \frac{s(1-s)\pi^{-s/2}\Gamma\left(\frac{s}{2}\right)\zeta(s)}{1 - \lambda s(1-s)} \pi^{-s/2}\Gamma\left(\frac{s}{2}\right)\zeta(s) \frac{(2\pi)^{s-2}}{\Gamma(s)}\phi(s) i \, ds $$
$$ = \frac{1}{\sqrt{\pi}} \frac{1}{2\pi i} \int G(s) \frac{\zeta(s)}{s} \, ds, $$
where
$$ G(s) = \frac{s^2(s-1)\zeta(s)\phi(s)}{1 - \lambda s(1-s)} \frac{\Gamma\left(\frac{s}{2}\right)}{\Gamma\left(\frac{s}{2} + \frac{1}{2}\right)}. $$
The function $G$ is analytic in the half-plane $\{ s : \text{Re } s > \frac{1}{2} \}$, and the modulus of $G$ is estimated from above by $\text{const} \cdot |s|^{1+\alpha - \beta} |s|^{-1/2} = \text{const} \cdot |s|^{1/2 + \alpha - \beta}$.

In the classical representation for the zeta-function
$$ \frac{\zeta(s)}{s} = \frac{1}{s-1} - \int_1^{+\infty} \frac{\{x\}}{x^{s+1}} \, dx, \quad \text{Re } s > 0, $$
where $\{x\}$ stands for the fractional part of a real number $x$, after the change of variables $x = \frac{1}{y}$, we obtain the formula
$$ \frac{\zeta(s)}{s} = \frac{1}{s-1} - \int_0^1 \left\{ \frac{1}{y} \right\} y^{s-1} \, dy, $$
in which the integral is the Mellin transform of the bounded function $\{ \frac{1}{y} \}$ on the finite interval $(0, 1)$. Since the Mellin transform maps $L^2(0, 1)$ onto the Hardy space $H^2$ in the half-plane $\{ s : \text{Re } s > \frac{1}{2} \}$, the integral defines some function from $H^2$ in this domain, and in view of the estimates of the modulus of the zeta-function, its modulus is estimated from above by $\text{const} \cdot |s|^{\alpha-1}$. It follows that
$$ \int G(s) \left( \frac{\zeta(s)}{s} - \frac{1}{s-1} \right) \, ds = 0. $$
Indeed, this is a consequence of the fact that the integrand is an analytic function in $\{ s : \text{Re } s > \frac{1}{2} \}$ whose modulus is at most $\text{const} \cdot |s|^{1/2 + \alpha - \beta} |s|^{\alpha-1} = \frac{\text{const}}{|s|^{1+\epsilon}}$, where by the assumption $\epsilon = \beta - 2\alpha - \frac{1}{2} > 0$. For each of such functions, the integral under consideration is zero.

Thus, in our calculation, $\frac{\zeta(s)}{s}$ can be replaced by $\frac{1}{s-1}$ which yields the relation
$$ \int f(s)\overline{u(s)}q(s) i \, ds = \frac{1}{\sqrt{\pi}} \frac{1}{2\pi i} \int \frac{G(s)}{s-1} \, ds; $$
and, by the Cauchy formula, we obtain
$$ \int f(s)\overline{u(s)}q(s) i \, ds = \frac{1}{\sqrt{\pi}} G(1) = \frac{1}{\sqrt{\pi}} \phi(1) \frac{\Gamma\left(\frac{1}{2}\right)}{\Gamma(1)} = 1. $$
Replacing $\lambda$ and $s$ by $\overline{\lambda}$ and $\overline{s}$ and passing to the complex-conjugate quantities, we have
$$ \int f(s)\overline{u(s)}q(1-s) i \, ds = 1. $$
Finally, summing up the last two equalities, we get the desired relation $[f, u] = 1$. $\square$

**FUNDING**
This work was supported by ongoing institutional funding. No additional grants to carry out or direct this particular research were obtained.

**CONFLICT OF INTEREST**
As author of this work, I declare that I have no conflicts of interest.

**OPEN ACCESS**
This article is licensed under a Creative Commons Attribution 4.0 International License, which permits use, sharing, adaptation, distribution and reproduction in any medium or format, as long as you give appropriate credit to the original author(s) and the source, provide a link to the Creative Commons license, and indicate if changes were made. The images or other third party material in this article are included in the article’s Creative Commons license, unless indicated otherwise in a credit line to the material. If material is not included in the article’s Creative Commons license and your intended use is not permitted by statutory regulation or exceeds the permitted use, you will need to obtain permission directly from the copyright holder. To view a copy of this license, visit http://creativecommons.org/licenses/by/4.0/.

**References**

1. Kapustin V.V., “The set of zeros of the Riemann zeta function as the point spectrum of an operator,” *St. Petersburg Math. J.*, vol. 33, no. 4, 661–673 (2022).
2. Titchmarsh E.C., *The Zeta-Function of Riemann*, Cambridge University, Cambridge (1930).

**Publisher’s Note.** Pleiades Publishing remains neutral with regard to jurisdictional claims in published maps and institutional affiliations.

V. V. Kapustin
St. Petersburg Department of the Steklov Mathematical Institute
St. Petersburg, Russia
*E-mail address:* kapustin@pdmi.ras.ru
