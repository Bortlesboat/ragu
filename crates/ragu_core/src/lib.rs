//! # `ragu_core`
//!
//! This crate contains the fundamental traits and types for writing protocols
//! and arithmetic circuits for the Ragu project. This API is re-exported (as
//! necessary) in other crates and so this crate is only intended to be used
//! internally by Ragu.
//!
//! This crate reexports Udon's cycle and Poseidon traits and exposes its Pasta
//! types through [`pasta`]. Ragu-specific generator derivation and loading
//! live in `ragu_pcd::pasta`.
//!
//! ## Cycles of Elliptic Curves
//!
//! Ragu is parameterized by a cycle of elliptic curves defined over large prime
//! fields, described by an implementation of the [`Cycle`] trait. The only
//! implementation is the [Pasta cycle](pasta), whose fields and curves — and
//! the vocabulary generic code names: fields, curves, evaluation domains,
//! polynomial utilities — are `udon`'s.
//!
//! ## Algebraic Hashes
//!
//! Ragu leans on [Poseidon](https://eprint.iacr.org/2019/458); a [`Cycle`]
//! provides the permutation's parameters over each field through
//! [`PoseidonPermutation`].

#![no_std]
#![allow(clippy::type_complexity)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(missing_docs)]
#![doc(html_favicon_url = "https://tachyon.z.cash/assets/ragu/v1/favicon-32x32.png")]
#![doc(html_logo_url = "https://tachyon.z.cash/assets/ragu/v1/rustdoc-128x128.png")]

#[cfg(not(feature = "alloc"))]
compile_error!("`ragu_core` requires the `alloc` feature to be enabled.");
extern crate alloc;

pub mod convert;
pub mod drivers;
mod errors;
pub mod gadgets;
pub mod maybe;
pub mod routines;

pub use drivers::Coeff;
pub use errors::{Error, Result};
pub use udon::{
    cycle::{Cycle, FixedGenerators},
    poseidon::PoseidonPermutation,
};

/// Udon's Pasta cycle, fields, curves, and Poseidon instances.
pub mod pasta {
    pub use udon::{
        cycle::{PallasGenerators, Pasta, PastaParams, VestaGenerators},
        poseidon::{PoseidonFp, PoseidonFq},
    };

    /// The Pallas base field, in Udon's loose representation.
    pub type Fp = udon::field::Fp;
    /// The Pallas scalar field, in Udon's loose representation.
    pub type Fq = udon::field::Fq;

    /// Pallas in projective coordinates.
    pub type Ep = udon::curve::PallasProjective;
    /// Pallas in affine coordinates, including identity.
    pub type EpAffine = udon::curve::PallasPoint;
    /// Vesta in projective coordinates.
    pub type Eq = udon::curve::VestaProjective;
    /// Vesta in affine coordinates, including identity.
    pub type EqAffine = udon::curve::VestaPoint;
}
