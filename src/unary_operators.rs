//! Operators with 1 parameter for use by anmchr commands and so on.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::math::*;

macro_rules! unary_operators {
    
    {
        $( ( $id:literal, $name:ident,  $float_func:expr, $int_func:expr $(,)* ) ),+
        $(,)*
    } => {
        use num_derive::FromPrimitive;
        #[derive(Debug, Copy, Clone, FromPrimitive)]
        #[repr(u32)]
        pub enum UnaryOp
        {
            $(
                $name = $id,
            )+
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
    }
}


unary_operators! {
    
    (
        0x00, Floor,
        |value : f32| { value.floor() },
        |value : i32| { value }
    ),
    (
        0x01, Ceil,
        |value : f32| { value.ceil() },
        |value : i32| { value }
    ),
    (
        0x02, Round,
        |value : f32| { value.round() },
        |value : i32| { value }
    ),
    (
        0x03, Fract,
        |value : f32| { value.fract() },
        |_value : i32| { 0 }
    ),
    
    (
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
        }
    ),
    (
        0x11, Sin,
        |value : f32| {
            approx_sin(value)
        },
        |value : i32| {
            // convert from degrees from i32 because idk what would even be useful here
            let value = (value as f32) * DEGREES_TO_RADIANS;
            
            (approx_sin(value)*16384.0) as i32
        }
    ),
    (
        0x12, Cos,
        |value : f32| {
            approx_cos(value)
        },
        |value : i32| {
            // convert from degrees from i32 because idk what would even be useful here
            let value = (value as f32) * DEGREES_TO_RADIANS;
            
            (approx_cos(value)*16384.0) as i32
        }
    ),
    
    
    (
        0x20, Abs,
        |value : f32| { value.abs() },
        |value : i32| { value.abs() }
    ),
    (
        0x21, Signum,
        |value : f32| { value.signum() },
        |value : i32| { value.signum() }
    ),
    
    (
        0x30, IsPositive,
        |value : f32| { f32_bool(value > 0.0) },
        |value : i32| { i32_bool(value.is_positive()) }
    ),
    
    (
        0xB0, BitwiseNot,
        |value : f32| {
            f32::from_bits(!value.to_bits())
        },
        |value : i32| {!value}
    ),
    
    (
        0xC0, LogicalNot,
        |value : f32| {
            f32_bool(!is_f32_true(value))
        },
        |value : i32| {
            i32_bool(!is_i32_true(value))
        }
    ),
}