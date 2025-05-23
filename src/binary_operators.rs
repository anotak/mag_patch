//! Operators with 2 parameters for use by anmchr commands and so on.

#![deny(unsafe_op_in_unsafe_fn)]
use crate::math::*;

macro_rules! binary_operators {
    
    {
        $( ( $(#[$($attrss1:tt)*])* $id:literal, $(#[$($attrss2:tt)*])* $name:ident,  $float_func:expr, $int_func:expr $(,)* ) ),+
        $(,)*
    } => {
        use num_derive::FromPrimitive;
        #[derive(Debug, Copy, Clone, FromPrimitive)]
        #[repr(u32)]
        pub enum BinaryOp
        {
            $(
                $(#[$($attrss1)*])*
                $(#[$($attrss2)*])*
                $name = $id,
            )+
        }
        
        // this is all a bit of a mess but we can't use generics for this stuff easily
        // because there's not really a generic that covers all the potential operations you'd use
        // so we end up using this macro soup since it just seems like a better solution for this
        pub fn handle_binary_operation_f32_f32_f32(lhs : f32, rhs : f32, op : BinaryOp) -> f32
        {
            let func = match op {
                $(
                BinaryOp::$name => $float_func,
                )+
            };
            
            let float = func(lhs, rhs);
            
            if float.is_finite() {
                float
            } else {
                0.0
            }
        }
        
        pub fn handle_binary_operation_f32_i32_f32(lhs : f32, rhs : i32, op : BinaryOp) -> f32
        {
            let rhs = rhs as f32;
            let func = match op {
                $(
                BinaryOp::$name => $float_func,
                )+
            };
            
            let float = func(lhs, rhs);
            
            if float.is_finite() {
                float
            } else {
                0.0
            }
        }
        
        pub fn handle_binary_operation_i32_f32_f32(lhs : i32, rhs : f32, op : BinaryOp) -> f32
        {
            let lhs = lhs as f32;
            let func = match op {
                $(
                BinaryOp::$name => $float_func,
                )+
            };
            
            let float = func(lhs, rhs);
            
            if float.is_finite() {
                float
            } else {
                0.0
            }
        }
        
        pub fn handle_binary_operation_i32_i32_f32(lhs : i32, rhs : i32, op : BinaryOp) -> f32
        {
            let func = match op {
                $(
                BinaryOp::$name => $int_func,
                )+
            };
            
            func(lhs, rhs) as f32
        }
        
        pub fn handle_binary_operation_f32_f32_i32(lhs : f32, rhs : f32, op : BinaryOp) -> i32
        {
            let func = match op {
                $(
                BinaryOp::$name => $float_func,
                )+
            };
            
            let float = func(lhs, rhs);
            
            if float.is_finite() {
                float as i32
            } else {
                0
            }
        }
        
        pub fn handle_binary_operation_f32_i32_i32(lhs : f32, rhs : i32, op : BinaryOp) -> i32
        {
            let rhs = rhs as f32;
            let func = match op {
                $(
                BinaryOp::$name => $float_func,
                )+
            };
            
            let float = func(lhs, rhs);
            
            if float.is_finite() {
                float as i32
            } else {
                0
            }
        }
        
        pub fn handle_binary_operation_i32_f32_i32(lhs : i32, rhs : f32, op : BinaryOp) -> i32
        {
            let lhs = lhs as f32;
            let func = match op {
                $(
                BinaryOp::$name => $float_func,
                )+
            };
            
            let float = func(lhs, rhs);
            
            if float.is_finite() {
                float as i32
            } else {
                0
            }
        }
        
        pub fn handle_binary_operation_i32_i32_i32(lhs : i32, rhs : i32, op : BinaryOp) -> i32
        {
            let func = match op {
                $(
                BinaryOp::$name => $int_func,
                )+
            };
            
            func(lhs, rhs)
        }
    }
}




// rhs means right hand side
// lhs means left hand side
binary_operators! {
    (
        /// addition
        0x00, Add,
        |lhs, rhs| { lhs + rhs },
        |lhs : i32, rhs : i32| { lhs.wrapping_add(rhs) }
    ),
    (
        /// subtraction
        0x01, Sub,
        |lhs, rhs| { lhs - rhs },
        |lhs : i32, rhs : i32| { lhs.wrapping_sub(rhs) }
    ),
    (
        /// multiplication
        0x02, Mul,
        |lhs, rhs| { lhs * rhs },
        |lhs : i32, rhs : i32| { lhs.wrapping_mul(rhs) }
    ),
    (
        /// division
        /// note: if the rhs is 0, the result will be 0
        0x03, Div,
        |lhs, rhs| {
            if rhs != 0.0 {
                lhs / rhs
            } else {
                0.0
            }
        },
        |lhs : i32, rhs : i32| {
            if rhs != 0 {
                lhs.wrapping_div(rhs)
            } else {
                0
            }
        }
    ),
    (
        /// takes the remainder of division
        0x04, Remainder,
        |lhs, rhs| {
            if rhs != 0.0 {
                lhs % rhs
            } else {
                0.0
            }
        },
        |lhs : i32, rhs : i32| {
            if rhs != 0 {
                lhs.wrapping_rem_euclid(rhs)
            } else {
                0
            }
        },
    ),
    
    
    (
        /// Minimum, the smaller of the two numbers
        0x10, Min,
        |lhs : f32, rhs| { lhs.min(rhs) },
        |lhs : i32, rhs : i32| { lhs.min(rhs) }
    ),
    (
        /// Maximum, the larger of the two numbers
        0x11, Max,
        |lhs : f32, rhs| { lhs.max(rhs) },
        |lhs : i32, rhs : i32| { lhs.max(rhs) },
    ),
    
    (
        /// absolute difference. absolute value of (left hand side - right hand side)
        0x20, AbsDiff,
        |lhs : f32, rhs : f32| {
            abs_diff(lhs,rhs)
        },
        |lhs : i32, rhs : i32| {
            let result = lhs.abs_diff(rhs).try_into();
            
            match result {
                Ok(result) => result,
                Err(_) => i32::MAX,
            }
        },
    ),
    (
        /// copies the sign from the right hand side. if right hand side is positive, then result is the left hand side. if the right hand side is negative, then the result is -left hand side.
        0x21, CopySign,
        |lhs : f32, rhs : f32| {
            lhs.copysign(rhs)
        },
        |lhs : i32, rhs : i32| {
            let lhs = lhs.abs();
            
            if rhs < 0 {
                -lhs
            } else {
                lhs
            }
        },
    ),
    
    
    
    
    
    
    // bitwise operations
    (
        /// [bitwise and](https://en.wikipedia.org/wiki/Bitwise_operation#AND)
        /// note that doing this on floating point numbers won't be very useful as mag_patch resets any NaN/infinity floats to 0.0
        0xB0, BitwiseAnd,
        |lhs : f32, rhs : f32| {
            f32::from_bits(lhs.to_bits() & rhs.to_bits())
        },
        |lhs : i32, rhs : i32| { lhs & rhs }
    ),
    (
        /// [bitwise or](https://en.wikipedia.org/wiki/Bitwise_operation#OR)
        /// note that doing this on floating point numbers won't be very useful as mag_patch resets any NaN/infinity floats to 0.0
        0xB1, BitwiseOr,
        |lhs : f32, rhs : f32| {
            f32::from_bits(lhs.to_bits() | rhs.to_bits())
        },
        |lhs : i32, rhs : i32| { lhs | rhs }
    ),
    (
        /// [bitwise or](https://en.wikipedia.org/wiki/Bitwise_operation#XOR)
        /// note that doing this on floating point numbers won't be very useful as mag_patch resets any NaN/infinity floats to 0.0
        0xB2, BitwiseXor,
        |lhs : f32, rhs : f32| {
            f32::from_bits(lhs.to_bits() ^ rhs.to_bits())
        },
        |lhs : i32, rhs : i32| { lhs ^ rhs }
    ),
    (
        /// [sign-extending arithmetic shift left](https://en.wikipedia.org/wiki/Arithmetic_shift)
        /// note that doing this on floating point numbers won't be very useful as mag_patch resets any NaN/infinity floats to 0.0
        0xB3, ShiftLeftSignExtend,
        |lhs : f32, rhs : f32| {
            f32::from_bits((lhs.to_bits() as i32).wrapping_shl(rhs as u32) as u32)
        },
        |lhs : i32, rhs : i32| { lhs.wrapping_shl(rhs as u32) }
    ),
    (
        /// [sign-extending arithmetic shift right](https://en.wikipedia.org/wiki/Arithmetic_shift)
        /// note that doing this on floating point numbers won't be very useful as mag_patch resets any NaN/infinity floats to 0.0
        0xb4, ShiftRightSignExtend,
        |lhs : f32, rhs : f32| {
            f32::from_bits((lhs.to_bits() as i32).wrapping_shr(rhs as u32) as u32)
        },
        |lhs : i32, rhs : i32| { lhs.wrapping_shr(rhs as u32) }
    ),
    (
        /// [rotate left](https://en.wikipedia.org/wiki/Circular_shift)
        /// note that doing this on floating point numbers won't be very useful as mag_patch resets any NaN/infinity floats to 0.0
        0xb5, RotateLeft,
        |lhs : f32, rhs : f32| {
            f32::from_bits(lhs.to_bits().rotate_left(rhs as u32))
        },
        |lhs : i32, rhs : i32| { lhs.rotate_left(rhs as u32)}
    ),
    (
        /// [rotate right](https://en.wikipedia.org/wiki/Circular_shift)
        /// note that doing this on floating point numbers won't be very useful as mag_patch resets any NaN/infinity floats to 0.0
        0xb6, RotateRight,
        |lhs : f32, rhs : f32| {
            f32::from_bits(lhs.to_bits().rotate_right(rhs as u32))
        },
        |lhs : i32, rhs : i32| { lhs.rotate_right(rhs as u32) }
    ),
    (
        /// [zero-extending logical shift left](https://en.wikipedia.org/wiki/Logical_shift)
        /// note that doing this on floating point numbers won't be very useful as mag_patch resets any NaN/infinity floats to 0.0
        0xB7, ShiftLeftZeroExtend,
        |lhs : f32, rhs : f32| {
            f32::from_bits(lhs.to_bits().wrapping_shl(rhs as u32))
        },
        |lhs : i32, rhs : i32| { (lhs as u32).wrapping_shl(rhs as u32) as i32 }
    ),
    (
        /// [zero-extending logical shift right](https://en.wikipedia.org/wiki/Logical_shift)
        /// note that doing this on floating point numbers won't be very useful as mag_patch resets any NaN/infinity floats to 0.0
        0xb8, ShiftRightZeroExtend,
        |lhs : f32, rhs : f32| {
            f32::from_bits(lhs.to_bits().wrapping_shr(rhs as u32))
        },
        |lhs : i32, rhs : i32| { (lhs as u32).wrapping_shr(rhs as u32) as i32 }
    ),
    
    (
        /// equality operator. approximate for floating point numbers within 0.000001 due to [floating point imprecision](https://en.wikipedia.org/wiki/Floating-point_arithmetic#Accuracy_problems)
        /// true results are 1 and false are 0
        0xC0, EqualityApproximate,
        |lhs : f32, rhs : f32| {
            f32_bool(near_eq(lhs, rhs))
        },
        |lhs : i32, rhs : i32| {
            i32_bool(lhs == rhs)
        }
    ),
    (
        /// left hand side < right hand side
        /// true results are 1 and false are 0
        0xC1, LessThan,
        |lhs : f32, rhs : f32| {
            f32_bool(lhs < rhs)
        },
        |lhs : i32, rhs : i32| {
            i32_bool(lhs < rhs)
        }
    ),
    (
        /// left hand side <= right hand side
        /// true results are 1 and false are 0
        0xC2, LessThanEqual,
        |lhs : f32, rhs : f32| {
            f32_bool(lhs <= rhs)
        },
        |lhs : i32, rhs : i32| {
            i32_bool(lhs <= rhs)
        }
    ),
    (
        /// left hand side > right hand side
        /// true results are 1 and false are 0
        0xC3, GreaterThan,
        |lhs : f32, rhs : f32| {
            f32_bool(lhs > rhs)
        },
        |lhs : i32, rhs : i32| {
            i32_bool(lhs > rhs)
        }
    ),
    (
        /// left hand side >= right hand side
        /// true results are 1 and false are 0
        0xC4, GreaterThanEqual,
        |lhs : f32, rhs : f32| {
            f32_bool(lhs >= rhs)
        },
        |lhs : i32, rhs : i32| {
            i32_bool(lhs >= rhs)
        }
    ),
}