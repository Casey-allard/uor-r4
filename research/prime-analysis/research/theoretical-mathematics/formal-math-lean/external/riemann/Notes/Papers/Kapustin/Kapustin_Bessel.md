Here is the full translation of the paper into English, with mathematical expressions formatted in LaTeX.

***

**Zap. Nauchn. Sem. POMI, 2022, Volume 512, 88–94**

**V. V. Kapustin**

**THE MELLIN TRANSFORM, DE BRANGES SPACES, AND BESSEL FUNCTIONS**

**Abstract:** The Mellin transform is defined for functions on the positive semi-axis by the formula
$$ (\mathcal{M}f)(s) = \frac{1}{\sqrt{2\pi}} \int_0^{+\infty} f(t)t^{s-1} \, dt. $$
A classical fact is that it isometrically maps $L^2(0, +\infty)$ onto $L^2(\{\text{Re } s = \frac{1}{2}\})$. Let us denote by $\Omega$ the Mellin transform considered as an operator from the Hardy space $H^2$, defined here and hereinafter in the right half-plane $\{\text{Re } z > 0\}$, into the weighted space $L^2_W$ on the line $\{\text{Re } s = \frac{1}{2}\}$, where the weight with respect to the Lebesgue measure on this line has the form
$$ W(s) = \exp(\pi \text{Im } s) + \exp(-\pi \text{Im } s), $$
and the norm of a function $u$ in $L^2_W$ is given by the formula $\|u\|^2 = \int |u(s)|^2 W(s) \, \frac{ds}{i}$, where the integral is taken along the line $\{\text{Re } s = \frac{1}{2}\}$ in the upward direction. For taking the Mellin transform of functions from $H^2$ that are analytic in the right half-plane, their traces on the positive semi-axis are used. It is known that $\Omega$ acts isometrically and its image is the entire space $L^2_W$, see below.

The change of variable $s = \frac{1-ix}{2}$ transfers functions given on the line $\{\text{Re } s = \frac{1}{2}\}$ to the real line. In this process, the space $L^2_W$ is mapped isometrically onto the weighted space $L^2_w$ with the weight
$$ w(x) = \frac{1}{2}\left(e^{\frac{\pi x}{2}} + e^{-\frac{\pi x}{2}}\right). $$

**Keywords:** de Branges spaces, modified Bessel functions, Mellin transform.

---

**Page 89**

on the real line. We define the operator $V$ as an isometry from $H^2$ to $L^2_w$, representing the superposition of the Mellin transform and the transfer to the real line:
$$ (Vf)(x) = \Omega\left(\frac{1-ix}{2}\right) = \frac{1}{\sqrt{2\pi}} \int_0^{+\infty} f(t) t^{-\frac{1+ix}{2}} \, dt. $$
The main result of this note is the description of the images of subspaces of the form $\exp\left(-a\left(z+\frac{1}{z}\right)\right) \cdot H^2$ under this mapping. The definition of de Branges spaces will be given below. The notation $K_s$ is used for the modified Bessel function, defined in the standard way.

**Theorem.** *The operator $V$ isometrically maps subspaces of the form $\exp\left(-a\left(z+\frac{1}{z}\right)\right) \cdot H^2(\{\text{Re } z > 0\})$, where $a > 0$, onto de Branges spaces with structural functions $\mathcal{E}(z) = 2\sqrt{\frac{a}{\pi}} K_s(2a)$ with $s = s(z) = \frac{1-iz}{2}$.*

In particular, this contains the assertion that the mentioned de Branges spaces are isometrically embedded in the space $L^2_w$.

Interest in subspaces of the indicated type is connected with the fact that for $a = \pi$, the de Branges space with the structural function $K_s(2\pi)$ contains functions of the form $\frac{\xi(\frac{1}{2}-iz)}{p(z)}$, where $\xi$ is the xi-function representing the symmetrization of the Riemann zeta-function $\zeta$:
$$ \xi(s) = \frac{1}{2} s(s-1) \pi^{-s/2} \Gamma\left(\frac{s}{2}\right) \zeta(s), $$
and $p$ is a polynomial of degree 3 or higher, whose zeros are zeros of the function in the numerator; see [1].

The result from the theorem is closely related to the work [2], where subspaces of the space $H^2$ consisting of functions invariant under the substitution $z \leftrightarrow \frac{1}{z}$ are considered. Considering instead subspaces of functions invariant under the isometric involution $f \mapsto \frac{1}{z}f\left(\frac{1}{z}\right)$ on $H^2$ (see the proof of the theorem) allows one to obtain a direct proof of the isometry of the operator from the theorem in article [2] using the isometry of operator $V$ and results of articles [1, 2].

---

**Page 90**

### §1. OPERATOR $\Omega$

As is known, the inverse Mellin transform acts according to the formula
$$ u \mapsto \frac{1}{\sqrt{2\pi}} \int z^{-s} u(s) \, \frac{ds}{i}. $$
Applying it, i.e., the operator $\Omega^{-1}$, to a function from the space $L^2_W$, we obtain an analytic function in the region $\{\text{Re } z > 0\} = \{|\arg z| < \frac{\pi}{2}\}$, because for $s = \frac{1}{2} + it$ we have the estimate
$$ |z^{-s}| = \exp\left(-\text{Re}(s \log z)\right) = \exp\left(-\frac{1}{2}\log|z| - t \arg z\right) = \frac{1}{\sqrt{|z|}} \cdot e^{-t \arg z}, $$
which ensures the convergence of the integral.

Apparently, the statement that the operator $\Omega$ isometrically maps $H^2$ onto the space $L^2_W$ is known, however, the author of the present note was unable to find a reference to the original source. This statement will be proven below.

**Proof.** For $f(z) = \frac{1}{z+\lambda}$, where $\text{Re } \lambda > 0$, we have $f \in H^2$ and
$$ (\Omega f)(s) = \frac{1}{\sqrt{2\pi}} \int_0^{+\infty} \frac{t^{s-1} \, dt}{t+\lambda} = \frac{\lambda^{s-1}}{\sqrt{2\pi}} \int_0^{+\infty} \frac{y^{s-1} \, dy}{y+1} $$
$$ = \frac{\lambda^{s-1}}{\sqrt{2\pi}} \cdot \frac{\pi}{\sin(\pi s)} = \sqrt{2\pi} \cdot \frac{\lambda^{s-1}}{W(s)}, $$
since for $\text{Re } s = \frac{1}{2}$ we have
$$ \sin(\pi s) = \frac{e^{i\pi s} - e^{-i\pi s}}{2i} = \frac{ie^{-\pi \text{Im } s} + ie^{\pi \text{Im } s}}{2i} = \frac{1}{2} W(s). $$
The closed linear span of functions $\frac{1}{z+\lambda}$, where $\text{Re } \lambda > 0$, is dense in $H^2$: the value of a function from $H^2$ at the point $\bar{\lambda}$ is calculated via the scalar product with the function $\frac{1}{z+\lambda}$, so a function from $H^2$ orthogonal to all such functions turns out to be identically zero. On the other hand, the closed linear span of functions of the form $\frac{\lambda^{s-1}}{W(s)}$ is dense in $L^2_W$. Indeed, this statement is equivalent to the fact that the closed linear span of functions $\lambda^{s-1}$ is dense in $L^2_{1/W}$, and this is proven by standard means related to the ordinary Mellin transform, if one takes such functions only for $\lambda > 0$.

---

**Page 91**

Thus, for the operator $\Omega$, a dense set in $H^2$ corresponds to a dense set in $L^2_W$, and thus it suffices to establish the isometry of the operator $\Omega^{-1}$ for these dense sets.

For functions $u$ from the dense set in $L^2_W$, their inverse Mellin transforms belong to $H^2$, which allows working with the boundary values of the latter on the imaginary axis. Let us take $z = ix$ with $x > 0$. Then $z^{-s} = x^{-s} \cdot \exp\left(-\frac{i\pi s}{2}\right)$, and we obtain the ordinary inverse Mellin transform of the function $\exp\left(-\frac{i\pi s}{2}\right) \cdot u(s)$. Using its isometry, we have
$$ \int_0^\infty |(\Omega^{-1}u)(ix)|^2 \, dx = \int \left|\exp\left(-\frac{i\pi s}{2}\right) \cdot u(s)\right|^2 \frac{ds}{i} = \int |u(s)|^2 e^{\pi \text{Im } s} \frac{ds}{i}, $$
where the integration along the line $\{\text{Re } s = \frac{1}{2}\}$ is performed upwards. Similarly, for $z = -ix$ with $x > 0$, we obtain $z^{-s} = x^{-s} \cdot \exp\left(\frac{i\pi s}{2}\right)$ and
$$ \int_0^\infty |(\Omega^{-1}u)(-ix)|^2 \, dx = \int \left|\exp\left(\frac{i\pi s}{2}\right) \cdot u(s)\right|^2 \frac{ds}{i} = \int |u(s)|^2 e^{-\pi \text{Im } s} \frac{ds}{i}. $$
The sum of the two obtained relations reflects the fact that $\Omega^{-1}$ is an isometry. $\square$

### §2. PROOF OF THE THEOREM

Let $\mathcal{E}$ be an entire function in the complex plane from the Hermite-Biehler class, i.e., $|\mathcal{E}(\bar{z})| < |\mathcal{E}(z)|$ for all $z$ in the upper half-plane $\{\text{Im } z > 0\}$. Then $\mathcal{E}$ is the structural function of the de Branges space $\mathcal{H}_{\mathcal{E}}$, defined as the space of entire functions $F$ in the complex plane for which the functions $\frac{F(z)}{\mathcal{E}(z)}$ and $\frac{F(\bar{z})}{\mathcal{E}(z)}$ belong to the Hardy class $H^2$ in the upper half-plane $\{\text{Im } z > 0\}$; the norms of these functions in the Hardy space are equal and they determine the norm of the function $F$ in $\mathcal{H}_{\mathcal{E}}$.

Fix $a > 0$ and set $\mathcal{E}(z) = 2\sqrt{\frac{a}{\pi}} K_s(2a)$, where $s = \frac{1-iz}{2}$. The function $\mathcal{E}$ belongs to the Hermite-Biehler class; the proof can be found, for example, in article [1], see also [2]; let us consider the corresponding de Branges space $\mathcal{H}_{\mathcal{E}}$.

---

**Page 92**

**Proof.** For the proof of the theorem, we will use the fact that the operators
$$ u \mapsto \frac{1}{2\pi} \int_{2a}^\infty \left(K_s(t) + K_{s-1}(t)\right) u(t) \, dt, $$
\begin{equation} u \mapsto \frac{1}{2\pi} \int_{2a}^\infty \left(K_s(t) - K_{s-1}(t)\right) u(t) \, dt, \tag{1} \end{equation}
where $s = s(z) = \frac{1-iz}{2}$, isometrically map the space $L^2(2a, \infty)$ onto the even and odd subspaces of the de Branges space $\mathcal{H}_{\mathcal{E}}$ respectively. This fact is proven in article [1], where it is obtained as a result of considering the canonical system corresponding to the de Branges space under study; see also [2]. The even and odd subspaces are mutually orthogonal and their direct sum gives the entire space $\mathcal{H}_{\mathcal{E}}$.

Further, calculations in article [2] show that the functions obtained under the mappings (1) are the Mellin transforms of the functions
$$ \frac{1}{2}\left(1+\frac{1}{x}\right) \cdot v\left(\frac{1}{2}\left(x+\frac{1}{x}\right)\right), \quad \frac{1}{2}\left(1-\frac{1}{x}\right) \cdot v\left(\frac{1}{2}\left(x+\frac{1}{x}\right)\right) $$
respectively, where $v$ is the image of the function $u$ under the Laplace transform:
$$ v(z) = \frac{1}{\sqrt{2\pi}} \int_{2a}^{+\infty} u(t) e^{-tz} \, dt. $$
Thus, the isometric mappings (1) represent superpositions of the Laplace transform, one of the transformations
$$ v \mapsto \frac{1}{2}\left(1+\frac{1}{z}\right) \cdot v\left(\frac{1}{2}\left(z+\frac{1}{z}\right)\right), \quad v \mapsto \frac{1}{2}\left(1-\frac{1}{z}\right) \cdot v\left(\frac{1}{2}\left(z+\frac{1}{z}\right)\right), \tag{2} $$
and the Mellin transform. The Laplace transform isometrically maps the space $L^2(2a, \infty)$ onto $e^{-2az} \cdot H^2(\{\text{Re } z > 0\})$; the Mellin transform isometrically maps $H^2(\{\text{Re } z > 0\})$ onto the space $L^2_W$, which is isometrically equivalent to the space $L^2_w$. Therefore, it remains to prove the following assertion: the mappings (2), acting in $H^2(\{\text{Re } z > 0\})$, are isometric operators,

---

**Page 93**

and the images of the subspace $e^{-2az} \cdot H^2$ under these mappings are mutually orthogonal and their direct sum coincides with the subspace $\exp\left(-a\left(z+\frac{1}{z}\right)\right) \cdot H^2$ from the statement of the theorem.

Consider an orthonormal basis in $H^2$ consisting of the functions
$$ \frac{1}{\sqrt{2\pi}} \frac{(z-1)^n}{(z+1)^{n+1}}, \quad n = 0, 1, 2, \dots $$
(Note that multiplication by the function $\frac{z-1}{z+1}$, the modulus of which is 1 on the imaginary axis, acts as a shift on this basis). For a basis function $v(z) = \frac{1}{\sqrt{2\pi}} \frac{(z-1)^n}{(z+1)^{n+1}}$ with index $n$, we have
$$ v\left(\frac{1}{2}\left(z+\frac{1}{z}\right)\right) = \frac{1}{\sqrt{2\pi}} \frac{\left(\frac{1}{2}\left(z+\frac{1}{z}\right)-1\right)^n}{\left(\frac{1}{2}\left(z+\frac{1}{z}\right)+1\right)^{n+1}} = \frac{2z}{\sqrt{2\pi}} \cdot \frac{(z-1)^{2n}}{(z+1)^{2n+2}}. $$
Consequently, under the mappings (2), basis elements with numbers $2n$ and $2n+1$ are obtained respectively. Thus, under the first mapping, from the basis elements one obtains elements of the very same basis with even numbers, generating the subspace of functions from $H^2$ invariant under the involution $f \mapsto \frac{1}{z}f\left(\frac{1}{z}\right)$. Under the second mapping, basis elements with odd numbers are obtained, generating the subspace of functions possessing the property $\frac{1}{z}f\left(\frac{1}{z}\right) = -f(z)$.

By multiplying by the considered exponential factors, the result obtained for the entire space $H^2$ is transferred to the required subspaces. $\square$

For a direct proof of the isometry of the operators from the theorem of article [2], one can now use the fact that the multiplication operator $f \mapsto \sqrt{2} \cdot \frac{z f(z)}{1+z}$ in $H^2$ isometrically maps the subspace of functions invariant under the involution $f \mapsto \frac{1}{z}f\left(\frac{1}{z}\right)$ onto the subspace of functions invariant under the substitution $z \leftrightarrow \frac{1}{z}$. Thus, the statement reduces to the isometry of the operator $\Omega$.

**REFERENCES**

1.  V. V. Kapustin, "The set of zeros of the Riemann zeta function as the point spectrum of an operator", *Algebra i Analiz*, **33**, No. 4 (2021), 107–124.
2.  V. V. Kapustin, "Five models in Hilbert spaces related to the Riemann zeta function", *Zap. Nauchn. Sem. POMI*, **503** (2021), 84–96.

**Kapustin V. V.** The Mellin transform, de Branges spaces, and Bessel functions.

An explicit description is obtained for the subspaces of the Hardy space on the right half-plane whose images under the Mellin transform yield a chain of de Branges spaces related to the Riemann zeta function.

St. Petersburg Department of Steklov Mathematical Institute
E-mail: kapustin@pdmi.ras.ru
Received July 8, 2022
