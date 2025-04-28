//! altering and reloading values from storage registers in anmchr commands
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(unreachable_patterns)]

use crate::storage;
use crate::hook_helpers::get_mut_cursor;
use byteorder::{WriteBytesExt, LittleEndian};
use std::io::{Seek, SeekFrom};

pub const RELOAD_SIZE : usize = const { size_of::<u32>() * 128 };

// values that have been replaced and need to be restored
pub struct Reload
{
    // position and value. the floats can be treated like u32s
    pub original_values : Option<Vec<(u64, u32)>>,
    target_ptr : usize,
}

impl Reload
{
    pub fn new(target_ptr : usize) -> Self {
        Self {
            target_ptr,
            original_values : None,
        }
    }
    
    pub fn restore(self)
    {
        if let Some(original_values) = self.original_values {
            let mut cursor = unsafe { get_mut_cursor(self.target_ptr, RELOAD_SIZE) };
            
            for (pos, original_value) in original_values {
                cursor.seek(SeekFrom::Start(pos)).unwrap();
                cursor.write_u32::<LittleEndian>(original_value).unwrap();
            }
        }
    }
}

pub fn save_anmchr_command(exe_ptr : usize, command_ptr : usize, command_type_group : Option<u32>, command : Option<u32>) -> Reload
{
    let mut reload = Reload::new(command_ptr);
    
    let command_type_group = match command_type_group {
        Some(c) => c,
        None => return reload,
    };
    
    let command = match command {
        Some(c) => c,
        None => return reload,
    };
    
    let mut cursor = unsafe { get_mut_cursor(command_ptr, RELOAD_SIZE) };
    
    const SIZE_U32 : u64 = size_of::<u32>() as u64;
    
    match (command_type_group, command) {
        (0,0x21) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 8 });
                }
            );
        },
        (0,0x22) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 9 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 10 });
                }
            );
        },
        (0,0x23) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 10 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 11 });
                }
            );
        },
        (0,0x24) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 11 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 12 });
                }
            );
        },
        (0,0x26) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 4 });
                }
            );
        },
        (0,0x31) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 11 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 15 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 16 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 17 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 19 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 20 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 21 });
                }
            );
        },
        (0,0x32) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 9 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 13 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 14 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 15 });
                }
            );
        },
        (0,0x37) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 11 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 15 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 16 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 17 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 19 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 20 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 21 });
                }
            );
        },
        (0,0x38) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 9 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 13 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 14 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 15 });
                }
            );
        },
        (0,0x45) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 3 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 4 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 5 });
                }
            );
        },
        (1,0x0C) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 7 });
                }
            );
        },
        (1,0x79) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 3 });
                }
            );
        },
        (1,0x7C) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 3 });
                }
            );
        },
        (1,0x7E) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 12 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 13 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 14 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 16 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 17 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 18 });
                }
            );
        },
        (1,0x7F) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 30 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 31 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 32 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 34 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 35 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 36 });
                }
            );
        },
        (1,0x89) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 3 });
                }
            );
        },
        (1,0x8a) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 3 });
                }
            );
        },
        (1,0x8B) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 6 });
                }
            );
        },
        (1,0xA4) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 3 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 4 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 5 });
                }
            );
        },
        (1,0xA5) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 5 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 7 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 8 });
                }
            );
        },
        (1,0xA6) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 4 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 5 });
                }
            );
        },
        (1,0xA7) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 3 });
                }
            );
        },
        (1,0xA8) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 3 });
                }
            );
        },
        (1,0xA9) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 3 });
                }
            );
        },
        (1,0xAA) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 4 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 5 });
                }
            );
        },
        (1,0xAB) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 3 });
                }
            );
        },
        (1,0xAC) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 3 });
                }
            );
        },
        (1,0xAD) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 3 });
                }
            );
        },
        (1,0xAE) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 4 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 5 });
                }
            );
        },
        (1,0xAF) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 4 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 5 });
                }
            );
        },
        (1,0xB0) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 3 });
                }
            );
        },
        (1,0xB1) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 5 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 6 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 7 });
                }
            );
        },
        (1,0xB3) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 4 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 5 });
                }
            );
        },
        (1,0xB4) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 3 });
                }
            );
        },
        (1,0xB6)  => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 3 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 4 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 5 });
                }
            );
        },
        (1,0xB7) | (1,0xB8) | (1,0xB9) | (1,0xBA) | (1,0xBB) | (1,0xBC) | (1,0xBD) | (1,0xBE) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 3 });
                }
            );
        },
        (1,0xBF)  => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 4 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 5 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 6 });
                }
            );
        },
        (1,0xC1)  => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 4 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 5 });
                }
            );
        },
        (1,0xC3)  => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 7 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 8 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 9 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 10 });
                }
            );
        },
        (1,0xCA)  => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 5 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 6 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 7 });
                }
            );
        },
        (1,0xCC) | (1,0xCD) | (1,0xCE) | (1,0xCF) | (1,0xD0) | (1,0xD1) => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 9 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 10 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 11 });
                }
            );
        },
        (1,0xD2)  => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 5 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 6 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 7 });
                }
            );
        },
        (1,0xDC)  => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 9 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 10 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 12 });
                }
            );
        },
        (1,0xDD)  => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 3 });
                }
            );
        },
        (1,0xE7) | (1,0xE8) | (1,0xE9) | (1,0xEA) | (1,0xEB) | (1,0xEC) | (1,0xED) | (1,0xEE) | (1,0xEF)  => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 5 });
                }
            );
        },
        (1,0xF0)  => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 6 });
                }
            );
        },
        (1,0xF7)  => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 9 });
                }
            );
        },
        (1,0xF8)  => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 5 });
                }
            );
        },
        (1,0xF9)  => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 5 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 6 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 7 });
                }
            );
        },
        (1,0xFB)  => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 5 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 6 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 7 });
                }
            );
        },
        (1,0xFD)  => {
            storage::with(
                exe_ptr,
                |store| {
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 5 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 6 });
                    store.store_f32_for_reload(&mut reload, &mut cursor, const { SIZE_U32 * 7 });
                }
            );
        },
        (_,_) => (),
    }
    
    reload
}
