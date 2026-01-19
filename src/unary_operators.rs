//! Operators with 1 parameter for use by anmchr commands and so on.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::math::*;

macro_rules! unary_operators {
    
    {
        $( ( $(#[$($attrss1:tt)*])* $id:literal, $(#[$($attrss2:tt)*])* $name:ident,  $float_func:expr, $int_func:expr, $bool_func:expr $(,)* ) ),+
        $(,)*
    } => {
        use num_derive::FromPrimitive;
        #[derive(Debug, Copy, Clone, FromPrimitive)]
        #[repr(u32)]
        pub enum UnaryOp
        {
            $(
                $(#[$($attrss1)*])*
                $(#[$($attrss2)*])*
                $name = $id,
            )+
        }
        
        pub trait UnaryOpHandler<T> {
            fn operate(self, value : T) -> T;
        }
        
        impl UnaryOpHandler<i32> for UnaryOp {
            fn operate(self, value : i32) -> i32
            {
                operation_i32(value, self)
            }
        }
        
        impl UnaryOpHandler<f32> for UnaryOp {
            fn operate(self, value : f32) -> f32
            {
                operation_f32(value, self)
            }
        }
        
        pub fn operation_f32(value : f32, op : UnaryOp) -> f32
        {
            let func = match op {
                $(
                UnaryOp::$name => $float_func,
                )+
            };
            
            let float = func(value);
            
            if float.is_finite() {
                float
            } else {
                0.0
            }
        }
        
        pub fn operation_i32(value : i32, op : UnaryOp) -> i32
        {
            let func = match op {
                $(
                UnaryOp::$name => $int_func,
                )+
            };
            
            func(value)
        }
        
        pub fn operation_bool(value : bool, op : UnaryOp) -> bool
        {
            let func = match op {
                $(
                UnaryOp::$name => $bool_func,
                )+
            };
            
            func(value)
        }
    }
}


unary_operators! {
    (
        /// chop the fraction part off a number, so  4.0 or 4.1 or 4.5 or 4.9 will all become 4
        /// (does nothing for integers)
        0x00, Floor,
        |value : f32| { value.floor() },
        |value : i32| { value },
        |value : bool| { value },
    ),
    (
        /// if there's a fraction part on a number, chop that off and get the next highest number so 5.0 or 4.1 or 4.5 or 4.9 will all become 5
        /// (does nothing for integers)
        0x01, Ceil,
        |value : f32| { value.ceil() },
        |value : i32| { value },
        |value : bool| { value },
    ),
    (
        /// rounds to the nearest integer so 4.0 and 4.1 will become 4.0. and 4.5 or 4.9 will become 5
        /// (does nothing for integers)
        0x02, Round,
        |value : f32| { value.round() },
        |value : i32| { value },
        |value : bool| { value },
    ),
    (
        /// keeps only the fraction part of a number. 0.1, 1.1, 1234.1 will all become 0.1
        /// (returns 0 for integers)
        0x03, Fract,
        |value : f32| { value.fract() },
        |_value : i32| { 0 },
        |value : bool| { value },
    ),
    
    (
        /// takes the square root of the number, so 64 will give 8.
        /// if the number is negative, then takes the absolute value and multiplies it by -1. so -64 will give -8
        0x10, SqrtWithNegative,
        |value : f32| {
            if value < 0.0 {
                value.abs().sqrt() * -1.0
            } else {
                value.sqrt()
            }
        },
        |value : i32| {
            if value < 0 {
                value.abs().isqrt() * -1
            } else {
                value.isqrt()
            }
        },
        |value : bool| { value },
    ),
    (
        /// approximate sine of the number. approximation is used to prevent this from working differently on different cpus / compiler versions.
        /// the approximation is here https://github.com/anotak/mag_patch/blob/main/src/math.rs#L58
        0x11, Sin,
        |value : f32| {
            approx_sin(value)
        },
        |value : i32| {
            // convert from degrees from i32 because idk what would even be useful here
            let value = (value as f32) * DEGREES_TO_RADIANS;
            
            (approx_sin(value)*16384.0) as i32
        },
        |value : bool| { value },
    ),
    (
        /// approximate cosine of the number. approximation is used to prevent this from working differently on different cpus / compiler versions.
        /// the approximation is here https://github.com/anotak/mag_patch/blob/main/src/math.rs#L58
        0x12, Cos,
        |value : f32| {
            approx_cos(value)
        },
        |value : i32| {
            // convert from degrees from i32 because idk what would even be useful here
            let value = (value as f32) * DEGREES_TO_RADIANS;
            
            (approx_cos(value)*16384.0) as i32
        },
        |value : bool| { value },
    ),
    (
        /// the number times itself
        0x13, Square,
        |value : f32| {
            value * value
        },
        |value : i32| {
            value.wrapping_mul(value)
        },
        |value : bool| { value },
    ),
    
    
    (
        /// absolute value
        0x20, Abs,
        |value : f32| { value.abs() },
        |value : i32| { value.saturating_abs() },
        |value : bool| { value },
    ),
    (
        /// the sign of the number.
        /// if it's negative, it gives -1.
        /// positive gives 1.
        /// 0 gives 0.
        0x21, Signum,
        |value : f32| { value.signum() },
        |value : i32| { value.signum() },
        |value : bool| { value },
    ),
    (
        /// the number multiplied by -1
        0x22, Negate,
        |value : f32| { value * -1.0 },
        |value : i32| { value.saturating_neg() },
        |value : bool| { !value },
    ),
    
    (
        /// 1 if the value is positive
        /// 0 if 0 or negative
        0x30, IsPositive,
        |value : f32| { (value > 0.0).from_bool() },
        |value : i32| { (value.is_positive()).from_bool() },
        |value : bool| { value },
    ),
    (
        /// 1 if the value is positive
        /// 0 if 0 or negative
        0x31, IsNegative,
        |value : f32| { (value < 0.0).from_bool() },
        |value : i32| { (value.is_negative()).from_bool() },
        |value : bool| { value },
    ),
    
    (
        /// flips all the bits 0 to 1 and 1 to 0. see also https://en.wikipedia.org/wiki/Bitwise_operation#NOT
        /// note that doing this on floating point numbers won't be very useful as mag_patch resets any NaN/infinity floats to 0.0
        0xB0, BitwiseNot,
        |value : f32| {
            f32::from_bits(!value.to_bits())
        },
        |value : i32| { !value },
        |value : bool| { !value },
    ),
    
    (
        /// if nonzero then returns 0.0
        /// if 0 then returns 1.0
        0xC0, LogicalNot,
        |value : f32| {
            (!value.is_true()).from_bool()
        },
        |value : i32| {
            (!value.is_true()).from_bool()
        },
        |value : bool| { !value },
    ),
}