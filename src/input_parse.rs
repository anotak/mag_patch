#![deny(unsafe_op_in_unsafe_fn)]

use crate::hook_helpers::*;

pub type InputParseFn = unsafe extern "win64" fn(u32, u32, u32) -> u32;


pub extern "win64" fn input_parse(param1 : u32, param2 : u32, param3 : u32) -> u32 {
    
    let hook = InputParseFn::get_original(input_parse);
    let output = unsafe { hook.call(param1, param2, param3) };
    
    if (output & 0x0C) == 0x0C {
        // clean up+down SOCD to prevent cheating and serious bugs
        output & 0xFFFFFFF7
    } else {
        output
    }
}


