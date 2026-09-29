//! Embeds Udon affine points and binds them to the Pasta parameter containers.

use alloc::vec::Vec;

use lazy_static::lazy_static;
use udon::{
    curve::{AffinePoint, PallasAffine, PastaCurve, VestaAffine},
    cycle::Generators,
};

use super::{DEFAULT_EP_K, DEFAULT_EQ_K, PastaParams};

bento::embed_array! {
    static PALLAS: [PallasAffine; (1 << DEFAULT_EP_K) + 1] =
        concat!(env!("OUT_DIR"), "/pallas-generators-v1-", udon::stored_form!(), ".bin");
}
bento::embed_array! {
    static VESTA: [VestaAffine; (1 << DEFAULT_EQ_K) + 1] =
        concat!(env!("OUT_DIR"), "/vesta-generators-v1-", udon::stored_form!(), ".bin");
}

fn generators<C: PastaCurve>(points: &'static [AffinePoint<C>]) -> Generators<C> {
    let (h, g) = points
        .split_last()
        .expect("the artifact includes a blinding generator");
    // Udon's parameter containers take identity-capable Point slices. Adapt
    // the embedded nonidentity points once, without coordinate decoding or
    // another curve-validation pass.
    let g: Vec<_> = g.iter().map(AffinePoint::to_point).collect();
    Generators::new(g.leak(), h.to_point())
}

lazy_static! {
    static ref PASTA_PARAMETERS: PastaParams =
        PastaParams::new(generators(PALLAS), generators(VESTA));
}

/// Returns Ragu's fixed generators bound to Udon's Pasta cycle.
///
/// The embedded points require no decoding. Parameter slices are assembled
/// once and retained for the lifetime of the program.
pub fn baked() -> &'static PastaParams {
    &PASTA_PARAMETERS
}
