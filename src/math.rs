//! math operations mainly for binary_operators and unary_operators

#![deny(unsafe_op_in_unsafe_fn)]

use std::f32::consts::*;

pub const DEGREES_TO_RADIANS : f32 = PI / 60.0;

pub const COMPARISON_EPSILON : f32 = 0.000001;

pub trait Truthful {
    const TRUE: Self;
    const FALSE: Self;
    
    fn is_true(self) -> bool;
}

// want this to be distinct from regular rust From / Into infrastructure because it's not quite the same thing
pub trait NumFromBool<T> where T : Truthful {
    fn from_bool(self) -> T;
}

pub trait BoolRoundtrip {
    fn bool_roundtrip(self : &Self) -> Self;
}

impl Truthful for i32 {
    const TRUE : Self = 1;
    const FALSE : Self = 0;
    

    #[inline]
    fn is_true(self) -> bool
    {
        self != Self::FALSE
    }
}

impl NumFromBool<i32> for bool {
    #[inline]
    fn from_bool(self) -> i32
    {
        if self {
            i32::TRUE
        } else {
            i32::FALSE
        }
    }
}

impl Truthful for f32 {
    const TRUE : Self = 1.0;
    const FALSE : Self = 0.0;
    

    #[inline]
    fn is_true(self) -> bool
    {
        if abs_diff(self,Self::FALSE) < COMPARISON_EPSILON {
            false
        } else {
            true
        }
    }
}

impl NumFromBool<f32> for bool {
    #[inline]
    fn from_bool(self) -> f32
    {
        if self {
            f32::TRUE
        } else {
            f32::FALSE
        }
    }
}

impl BoolRoundtrip for i32 {
    fn bool_roundtrip(self : &Self) -> Self {
        self.is_true().from_bool()
    }
}

impl BoolRoundtrip for f32 {
    fn bool_roundtrip(self : &Self) -> Self {
        self.is_true().from_bool()
    }
}

pub fn bool_to_i32(b : bool) -> i32
{
    b.from_bool()
}

pub fn bool_to_f32(b : bool) -> f32
{
    b.from_bool()
}



pub fn abs_diff(lhs : f32, rhs : f32) -> f32
{
    (lhs - rhs).abs()
}

pub fn near_eq(lhs : f32, rhs : f32) -> bool
{
    abs_diff(lhs,rhs) < COMPARISON_EPSILON
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


pub fn clean_float(float : f32) -> f32 {
    if float.is_finite() {
        float
    } else {
        0.0
    }
}

/// used inside the var_rw! macro to convey type, then whatever cast is needed is done.
/// also used for registers, similarly
#[derive(Clone, Copy)]
pub enum Number {
    F32(f32),
    I32(i32)
}

impl Number {
    pub fn into_float(self) -> f32 {
        match self {
            Number::F32(f) => f,
            Number::I32(i) => i as f32,
        }
    }
    
    pub fn into_int(self) -> i32 {
        match self {
            Number::F32(f) => f as i32,
            Number::I32(i) => i,
        }
    }
}

impl Truthful for Number {
    const TRUE : Self = Number::I32(i32::TRUE);
    const FALSE : Self = Number::I32(i32::FALSE);
    
    #[inline]
    fn is_true(self) -> bool
    {
        match self {
            Number::F32(f) => f.is_true(),
            Number::I32(i) => i.is_true(),
        }
    }
}

impl NumFromBool<Number> for bool {
    #[inline]
    fn from_bool(self) -> Number
    {
        match self {
            true => Number::TRUE,
            false => Number::FALSE,
        }
    }
}


impl BoolRoundtrip for Number {
    fn bool_roundtrip(self : &Self) -> Self {
        self.is_true().from_bool()
    }
}