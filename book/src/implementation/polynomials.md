# Polynomial Management

Ragu's prover works primarily with polynomials: constructing them from
circuit descriptions, multiplying them via FFTs, and decomposing their
products into forms the verifier can check. This chapter covers the
wiring polynomial that encodes an arithmetic circuit, the synthesis
process that builds it incrementally, and the low-level polynomial
utilities in [`udon`] that support these operations.

## Wiring Polynomials

Individual arithmetic circuits are defined by the
[structured vector](../protocol/prelim/structured_vectors.md)
$\v{s} \in \F^{4n}$ that describes the
[constraints](../protocol/core/arithmetization.md#constraints)
enforced over the witness, given a concrete choice of random challenge $y$.
This vector is the coefficient vector of a special polynomial

$$
s(X, Y) = \sum\limits_{j=0}^{4n - 1} Y^j \Big(
      \sum_{i = 0}^{n - 1} (\v{u})_{i,j} X^{2n - 1 - i}
    + \sum_{i = 0}^{n - 1} (\v{v})_{i,j} X^{2n + i}
    + \sum_{i = 0}^{n - 1} (\v{w})_{i,j} X^{4n - 1 - i}
\Big)
$$

at the restriction $Y = y$. This is known as the "wiring polynomial."

## Synthesis {#synthesis}

Ragu will directly synthesize circuit code into (partial) evaluations of
the reduced wiring polynomial. There are two operations that influence this
polynomial:

* `enforce_zero` creates a
  [constraint](../protocol/core/arithmetization.md#constraints)
  that enforces that a linear combination of wires must equal zero. This
  produces a new term in $Y^j$ for some unused $j$.
* `mul` creates new wires $(a, b, c)$ that must satisfy a
  [gate]
  $ab = c$. This allocates (or assigns) the corresponding powers
  $(X^{2n + i}, X^{2n - 1 - i}, X^i)$ for some unused $i$.

**Importantly, this synthesis process is procedural.** Any contiguous
sequence of `enforce_zero` and `mul` operations is defined by the
polynomials $g, h \in \F[X, Y]$ and transforms $s(X, Y)$ into $s'(X, Y)$
where for some $i, j$

$$
s'(X, Y) = s(X, Y) + Y^j (X^i g(X, Y) + h(X, Y)).
$$

Here, only $h(X, Y)$ varies depending on wires not allocated within that
sequence of operations. In many cases, $h$ is either extremely sparse (and
so trivial to compute as necessary) or is used in multiple repeated
sequences. Any repeated sequence produces the same $g$ polynomial by
definition, and so its evaluation can be fully memoized for future
invocations of an identical sequence of operations by simply scaling by
$X^i Y^j$.

[gate]: ../protocol/core/arithmetization.md#gates

## Polynomial Arithmetic

The synthesis machinery above relies on standard polynomial operations
provided by the [`udon`] crate. These operate on coefficient
vectors in ascending
degree order: the slice $[c_0, c_1, \ldots, c_n]$ represents the
polynomial
$c_0 + c_1 X + \cdots + c_n X^n$.

### Evaluation and Inner Products

[`evaluate`] evaluates a polynomial at a point using Horner's method.
[`dot`] computes the inner product $\langle \v{a}, \v{b} \rangle$ of
two equal-length coefficient slices; [`dot_iter`] accepts iterators for reversed
or noncontiguous inputs. These helpers provide the scalar operations underlying
polynomial evaluation and inner-product checks.

### Polynomial Multiplication

[`multiply`] computes the coefficient convolution of two polynomials,
using schoolbook multiplication or the field's FFT when the supplied scratch
is sufficient and the transform is cheaper. Pasta fields dispatch to Udon's
optimized transforms through `FftField`.

Given polynomials $a(X)$ of degree $d_a$ and $b(X)$ of degree $d_b$, it produces
$c(X) = a(X) \cdot b(X)$ of degree
$d_a + d_b$. When Pasta uses transforms, Udon expands each coefficient
prefix into a power-of-two evaluation domain, multiplies pointwise, and
interpolates the product. The intermediate evaluations stay in bit-reversed
order to avoid extra permutations.

Output and scratch are caller-supplied `&mut [F]` buffers, allowing their
storage to be reused across multiplications.

[`udon`]: https://github.com/tachyon-zcash/udon/tree/dea665b207bbf11a3eb1b0d0d7ab5367b7d583bb/crates/udon
[`evaluate`]: https://github.com/tachyon-zcash/udon/blob/dea665b207bbf11a3eb1b0d0d7ab5367b7d583bb/crates/udon/src/polynomial/evaluation.rs
[`dot`]: https://github.com/tachyon-zcash/udon/blob/dea665b207bbf11a3eb1b0d0d7ab5367b7d583bb/crates/udon/src/field/products.rs
[`dot_iter`]: https://github.com/tachyon-zcash/udon/blob/dea665b207bbf11a3eb1b0d0d7ab5367b7d583bb/crates/udon/src/field/products.rs
[`multiply`]: https://github.com/tachyon-zcash/udon/blob/dea665b207bbf11a3eb1b0d0d7ab5367b7d583bb/crates/udon/src/polynomial/multiplication.rs
