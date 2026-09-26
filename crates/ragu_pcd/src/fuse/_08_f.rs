//! Construct and commit to $f(X)$.
//!
//! This computes the multi-quotient polynomial $f(X)$ that acts as a witness
//! for the claimed evaluations in the `query` stage. This also constructs the
//! bridge for committing to the polynomial.
//!
//! Each `factor_iter` call below produces the coefficients of $(p\_i(X) - v\_i)
//! / (X - x\_i)$ for a single query. The static query prefix is defined by
//! [`STATIC_F_QUERIES`] and consumed by both this prover path and the
//! `compute_v` circuit.

use alloc::vec::Vec;

use ragu_circuits::{
    polynomials::{Rank, sparse},
    staging::StageExt,
};
use ragu_core::{Cycle, Result, drivers::Driver, maybe::Maybe};
use ragu_primitives::Element;
use rand::CryptoRng;
use udon::field::Field;

use super::{NativeF, NativeSPrime, RegistryWy};
use crate::{
    Application, Proof,
    internal::{
        native,
        native::{RxComponent, RxIndex, STATIC_F_QUERIES, StaticFQuery},
        nested,
    },
    proof::ProofBuilder,
};

impl<C: Cycle, R: Rank, const HEADER_SIZE: usize, B: crate::SelectableBackend>
    Application<'_, C, R, HEADER_SIZE, B>
{
    pub(super) fn compute_f<'dr, D, RNG: CryptoRng>(
        &self,
        rng: &mut RNG,
        w: &Element<'dr, D>,
        y: &Element<'dr, D>,
        z: &Element<'dr, D>,
        x: &Element<'dr, D>,
        alpha: &Element<'dr, D>,
        s_prime: &NativeSPrime<C, R>,
        registry_wy: &RegistryWy<C, R>,
        builder: &mut ProofBuilder<'_, C, R, B>,
        left: &Proof<C, R>,
        right: &Proof<C, R>,
    ) -> Result<NativeF<C, R>>
    where
        D: Driver<'dr, F = C::CircuitField>,
    {
        let native = self.compute_native_f(
            w,
            y,
            z,
            x,
            alpha,
            s_prime,
            registry_wy,
            builder,
            left,
            right,
        )?;
        self.compute_bridge_f(rng, &native, builder)?;
        Ok(native)
    }

    /// Manually commits the bridge for $f$, rather than having the
    /// [`ProofBuilder`] retain the native copy that derives it, since the `f`
    /// polynomial is not retained after the fuse step and so does not appear in
    /// the proof.
    fn compute_bridge_f<RNG: CryptoRng>(
        &self,
        rng: &mut RNG,
        native: &NativeF<C, R>,
        builder: &mut ProofBuilder<'_, C, R, B>,
    ) -> Result<()> {
        let bridge_rx = nested::stages::f::Stage::<C::HostCurve, R>::rx(
            C::ScalarField::random(|bytes| rng.fill_bytes(bytes)),
            &nested::stages::f::Witness {
                native_f: native.commitment,
            },
        )?;
        let bridge_commitment =
            B::sparse_commit_to_affine(&bridge_rx, C::nested_generators(self.params));
        builder.set_bridge_f_rx(bridge_rx, bridge_commitment);
        Ok(())
    }

    fn compute_native_f<'dr, D>(
        &self,
        w: &Element<'dr, D>,
        y: &Element<'dr, D>,
        z: &Element<'dr, D>,
        x: &Element<'dr, D>,
        alpha: &Element<'dr, D>,
        s_prime: &NativeSPrime<C, R>,
        registry_wy: &RegistryWy<C, R>,
        builder: &ProofBuilder<'_, C, R, B>,
        left: &Proof<C, R>,
        right: &Proof<C, R>,
    ) -> Result<NativeF<C, R>>
    where
        D: Driver<'dr, F = C::CircuitField>,
    {
        use udon::polynomial::divide_linear_rev;

        let w = *w.value().take();
        let y = *y.value().take();
        let z = *z.value().take();
        let x = *x.value().take();
        let xz = x * z;
        let alpha = *alpha.value().take();

        let omega_j = |idx: native::InternalCircuitIndex| -> C::CircuitField {
            idx.circuit_index().omega_j()
        };

        let mut iters: Vec<_> = STATIC_F_QUERIES
            .into_iter()
            .map(|query| match query {
                StaticFQuery::LeftP => {
                    divide_linear_rev(left.native_p_poly().iter_coeffs(), left.u())
                }
                StaticFQuery::RightP => {
                    divide_linear_rev(right.native_p_poly().iter_coeffs(), right.u())
                }
                StaticFQuery::LeftRegistryXyAtW => {
                    divide_linear_rev(left.native_registry_xy_poly().iter_coeffs(), w)
                }
                StaticFQuery::RightRegistryXyAtW => {
                    divide_linear_rev(right.native_registry_xy_poly().iter_coeffs(), w)
                }
                StaticFQuery::RegistryWx0AtLeftY => {
                    divide_linear_rev(s_prime.registry_wx0_poly.iter_coeffs(), left.y())
                }
                StaticFQuery::RegistryWx1AtRightY => {
                    divide_linear_rev(s_prime.registry_wx1_poly.iter_coeffs(), right.y())
                }
                StaticFQuery::RegistryWx0AtY => {
                    divide_linear_rev(s_prime.registry_wx0_poly.iter_coeffs(), y)
                }
                StaticFQuery::RegistryWx1AtY => {
                    divide_linear_rev(s_prime.registry_wx1_poly.iter_coeffs(), y)
                }
                StaticFQuery::RegistryWyAtLeftX => {
                    divide_linear_rev(registry_wy.poly.iter_coeffs(), left.x())
                }
                StaticFQuery::RegistryWyAtRightX => {
                    divide_linear_rev(registry_wy.poly.iter_coeffs(), right.x())
                }
                StaticFQuery::RegistryWyAtX => divide_linear_rev(registry_wy.poly.iter_coeffs(), x),
                StaticFQuery::RegistryXyAtW => {
                    divide_linear_rev(builder.native_registry_xy_poly().iter_coeffs(), w)
                }
                StaticFQuery::RegistryXyAtLeftCircuitId => divide_linear_rev(
                    builder.native_registry_xy_poly().iter_coeffs(),
                    left.circuit_id().omega_j(),
                ),
                StaticFQuery::RegistryXyAtRightCircuitId => divide_linear_rev(
                    builder.native_registry_xy_poly().iter_coeffs(),
                    right.circuit_id().omega_j(),
                ),
                StaticFQuery::LeftAbAAtXz => {
                    divide_linear_rev(left[RxComponent::AbA].iter_coeffs(), xz)
                }
                StaticFQuery::LeftAbBAtX => {
                    divide_linear_rev(left[RxComponent::AbB].iter_coeffs(), x)
                }
                StaticFQuery::RightAbAAtXz => {
                    divide_linear_rev(right[RxComponent::AbA].iter_coeffs(), xz)
                }
                StaticFQuery::RightAbBAtX => {
                    divide_linear_rev(right[RxComponent::AbB].iter_coeffs(), x)
                }
                StaticFQuery::CurrentAAtXz => {
                    divide_linear_rev(builder.native_a_poly().iter_coeffs(), xz)
                }
                StaticFQuery::CurrentBAtX => {
                    divide_linear_rev(builder.native_b_poly().iter_coeffs(), x)
                }
            })
            .collect();
        // Per-rx evaluations at xz only. The same r_i(xz) values feed
        // into both A(xz) (undilated) and B(x) (Z-dilated).
        for proof in [left, right] {
            for &id in &RxIndex::ALL {
                iters.push(divide_linear_rev(proof[id].iter_coeffs(), xz));
            }
        }

        // m(\omega^j, x, y) evaluations for each internal index j
        for &id in &native::InternalCircuitIndex::ALL {
            iters.push(divide_linear_rev(
                builder.native_registry_xy_poly().iter_coeffs(),
                omega_j(id),
            ));
        }

        let mut coeffs = Vec::with_capacity(R::num_coeffs());
        let (first, rest) = iters.split_first_mut().unwrap();
        for val in first.by_ref() {
            let c = rest
                .iter_mut()
                .fold(val, |acc, iter| acc.mul_add(&alpha, &iter.next().unwrap()));
            coeffs.push(c);
        }
        coeffs.reverse();

        let poly = sparse::Polynomial::from_coeffs(coeffs);
        let commitment = B::sparse_commit_to_affine(&poly, C::host_generators(self.params));

        Ok(NativeF { poly, commitment })
    }
}
