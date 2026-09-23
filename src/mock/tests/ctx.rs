use alloc::string::ToString as _;

use ragu_core::{
    Error,
    pasta::{Eq, Fp},
};

use super::super::hooks::FrameworkHooks;
use crate::StepCtx;

#[test]
fn enforce_poly_query_rejects_identity() {
    let mut hooks = FrameworkHooks::new();
    let mut ctx = StepCtx::new(&mut hooks);

    assert!(
        ctx.enforce_poly_query(Eq::GENERATOR, Fp::ONE, Fp::ONE)
            .is_ok()
    );

    let err = ctx
        .enforce_poly_query(Eq::IDENTITY, Fp::ONE, Fp::ONE)
        .expect_err("identity commitment must be rejected");
    assert!(matches!(err, Error::InvalidWitness(_)));
    assert!(err.to_string().contains("point at infinity"));
}

#[test]
fn derive_challenge_rejects_identity() {
    let mut hooks = FrameworkHooks::new();
    let mut ctx = StepCtx::new(&mut hooks);

    assert!(ctx.derive_challenge(&[Eq::GENERATOR]).is_ok());

    let err = ctx
        .derive_challenge(&[Eq::GENERATOR, Eq::IDENTITY])
        .expect_err("identity commitment must be rejected");
    assert!(matches!(err, Error::InvalidWitness(_)));
    assert!(err.to_string().contains("point at infinity"));
}
