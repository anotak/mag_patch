//! altering and reloading values from storage registers in anmchr commands
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(unreachable_patterns)]

use crate::storage;
use crate::hook_helpers::get_mut_cursor;
use byteorder::{WriteBytesExt, ReadBytesExt, LittleEndian};
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
    
    #[cfg_attr(not(test), expect(unused_variables))]
    let command = match command {
        Some(c) => c,
        None => return reload,
    };
    
    let mut cursor = unsafe { get_mut_cursor(command_ptr, RELOAD_SIZE) };
    
    const SIZE_U32 : u64 = size_of::<u32>() as u64;
    
    match command_type_group {
        0 | 1 | 3 => {
            storage::with(
                exe_ptr,
                |store| {
                    let len = cursor.read_u32::<LittleEndian>().unwrap();
                    
                    let mut target_ptr = SIZE_U32 * (len + 2) as u64;
                    
                    // read in table of types
                    for table_index  in 0..len {
                        let seek_offset = SIZE_U32 * (2 + table_index as u64);
                        cursor.seek(SeekFrom::Start(seek_offset)).unwrap();
                        
                        let value_type = cursor.read_u32::<LittleEndian>().unwrap();
                        match value_type {
                            1 => {
                                // 1 byte integer
                                target_ptr += 1;
                            },
                            3 | 5 | 0xE | 0xF => {
                                // various 4 byte integers
                                target_ptr += SIZE_U32;
                            },
                            6 => {
                                store.store_f32_for_reload(&mut reload, &mut cursor, target_ptr);
                                target_ptr += size_of::<f32>() as u64;
                            },
                            0xC => {
                                // vector of 3
                                store.store_f32_for_reload(&mut reload, &mut cursor, target_ptr);
                                target_ptr += size_of::<f32>() as u64;
                                store.store_f32_for_reload(&mut reload, &mut cursor, target_ptr);
                                target_ptr += size_of::<f32>() as u64;
                                store.store_f32_for_reload(&mut reload, &mut cursor, target_ptr);
                                target_ptr += size_of::<f32>() as u64;
                            },
                            0xD => {
                                // vector of 4
                                store.store_f32_for_reload(&mut reload, &mut cursor, target_ptr);
                                target_ptr += size_of::<f32>() as u64;
                                store.store_f32_for_reload(&mut reload, &mut cursor, target_ptr);
                                target_ptr += size_of::<f32>() as u64;
                                store.store_f32_for_reload(&mut reload, &mut cursor, target_ptr);
                                target_ptr += size_of::<f32>() as u64;
                                store.store_f32_for_reload(&mut reload, &mut cursor, target_ptr);
                                target_ptr += size_of::<f32>() as u64;
                            },
                            0x10 | 0x07 => {
                                // string of 64 bytes fixed
                                target_ptr += 64;
                            },
                            // don't support anything else
                            _ => 
                            {
                                #[cfg(test)]
                                {
                                    println!("unknown arg type {:02X} in command {:02X}_{:02X}", value_type, command_type_group, command);
                                }
                                
                                break;
                            },
                        }
                    }
                }
            );
        },
        _ => (),
    }
    
    reload
}
