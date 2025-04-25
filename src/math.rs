//! math operations mainly for binary_operators and unary_operators

#![deny(unsafe_op_in_unsafe_fn)]

use std::f32::consts::*;

pub const DEGREES_TO_RADIANS : f32 = PI / 60.0;

pub const I32_TRUE : i32 = 1;
pub const I32_FALSE : i32 = 0;
pub const F32_TRUE : f32 = 1.0;
pub const F32_FALSE : f32 = 0.0;

pub const COMPARISON_EPSILON : f32 = 0.000001;

pub fn is_f32_true(test : f32) -> bool
{
    if abs_diff(test,F32_FALSE) < COMPARISON_EPSILON {
        false
    } else {
        true
    }
}

pub fn is_i32_true(test : i32) -> bool
{
    if test == 0 {
        false
    } else {
        true
    }
}

pub fn f32_bool(test : bool) -> f32
{
    if test {
        F32_TRUE
    } else {
        F32_FALSE
    }
}

pub fn i32_bool(test : bool) -> i32
{
    if test {
        I32_TRUE
    } else {
        I32_FALSE
    }
}

pub fn abs_diff(lhs : f32, rhs : f32) -> f32
{
    (lhs - rhs).abs()
}

// see https://docs.rs/micromath/latest/src/micromath/float/cos.rs.html#224-234
// and also graphed here https://www.desmos.com/calculator/ay4td67mfc
/// Approximate sine, using this instead of rust's builtin because of determinism reasons
pub fn approx_sin(x : f32) -> f32 {
    let mut x = x;
    x *= const { FRAC_1_PI / 2.0 }; // adjust 2pi
    x -= (x + 0.5).floor();
    x *= -16.0 * (x.abs() - 0.5);
    x += 0.225 * x * (x.abs() - 1.0);
    x
}

/// Approximate cosine, using this instead of rust's builtin because of determinism reasons
pub fn approx_cos(x : f32) -> f32 {
    let mut x = x;
    x *= const { FRAC_1_PI / 2.0 };
    x -= 0.25 + (x + 0.25).floor();
    x *= 16.0 * (x.abs() - 0.5);
    x += 0.225 * x * (x.abs() - 1.0);
    x
}

