
//! Variables used and set by things like anmchr commands

#![deny(unsafe_op_in_unsafe_fn)]


use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::io::{Cursor, Seek, SeekFrom};

use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};


use crate::character_extensions;
use crate::binary_operators;
use crate::binary_operators::BinaryOpHandler;
use crate::unary_operators;
use crate::game_data::{Char};
use crate::reload::Reload;
use crate::hook_helpers::read_ptr_no_check;
use crate::math::Number;

/// usize is usually pointer to owning object
pub static CHAR_STORAGE : LazyLock<Mutex<HashMap<usize, CharStore>>> = LazyLock::new(|| {
    Mutex::new(HashMap::with_capacity(64))
});

/// called when the round starts
pub fn reset_all() {
    let mut storage = CHAR_STORAGE.lock().unwrap();
    
    storage.clear();
    
    if storage.capacity() > 512 {
        storage.shrink_to_fit();
        storage.reserve(64);
    }
}

/// call to retrieve a new storage
pub fn with<F, T>(key : usize, function : F) -> T
    where F : FnOnce(&mut CharStore) -> T
{
    let mut storage = CHAR_STORAGE.lock().unwrap();
    
    if !storage.contains_key(&key) {
        storage.insert(key, CharStore::new(key));
    }
    
    let store = storage.get_mut(&key).unwrap();
    
    let result = function(store);
    
    result
}

pub fn with_no_make<F>(key : usize, function : F)
    where F : FnOnce(&mut CharStore)
{
    let mut storage = CHAR_STORAGE.lock().unwrap();
    
    if !storage.contains_key(&key) {
        return;
    }
    
    let store = storage.get_mut(&key).unwrap();
    
    let result = function(store);
    
    result
}


const REGISTER_COUNT : usize = 128;
const DEFAULT_REGISTER_F32 : f32 = 0.0;
const DEFAULT_REGISTER_I32 : i32 = 0;

pub struct CharStore
{
    character : Char,
    
    floats : Option<Box<[f32; REGISTER_COUNT]>>,
    ints : Option<Box<[i32; REGISTER_COUNT]>>,
    
    pub suck_opponent : character_extensions::SuckOpponent,
}

impl CharStore {
    fn new(ptr : usize) -> Self {
        Self {
            character : Char::new(ptr),
            floats : None,
            ints : None,
            suck_opponent : character_extensions::SuckOpponent {
                magnitude : 0.0,
                delta : 0.0,
            },
        }
    }
    
    pub fn set_f32_register(&mut self, index : u8, value : f32) {
        let index = index & F32_REGISTER_UNMASK;
        
        if self.floats == None {
            self.floats = Some(Box::new([DEFAULT_REGISTER_F32; REGISTER_COUNT]));
        }
        
        if let Some(list) = &mut self.floats {
            list[index as usize] = value;
        }
    }
    
    pub fn set_i32_register(&mut self, index : u8, value : i32) {
        let index = index & F32_REGISTER_UNMASK;
        
        if self.ints == None {
            self.ints = Some(Box::new([DEFAULT_REGISTER_I32; REGISTER_COUNT]));
        }
        
        if let Some(list) = &mut self.ints {
            list[index as usize] = value;
        }
    }
    
    pub fn set_number_register(&mut self, index : u8, value : Number) {
        match RegisterType::identify(index)
        {
            RegisterType::F32 => self.set_f32_register(index, value.into_float()),
            RegisterType::I32 => self.set_i32_register(index, value.into_int()),
        }
    }
    
    pub fn get_f32_register(&mut self, index : u8) -> f32 {
        if index & F32_REGISTER_MASK == F32_REGISTER_MASK {
            let index = index & F32_REGISTER_UNMASK;
            
            match &self.floats {
                Some(list) => list[index as usize],
                None => DEFAULT_REGISTER_F32,
            }
        } else {
            let index = index & F32_REGISTER_UNMASK;
            
            match &self.ints {
                Some(list) => list[index as usize] as f32,
                None => const { DEFAULT_REGISTER_I32 as f32 },
            }
        }
    }
    
    pub fn get_i32_register(&mut self, index : u8) -> i32 {
        if index & F32_REGISTER_MASK == F32_REGISTER_MASK {
            let index = index & F32_REGISTER_UNMASK;
            
            match &self.floats {
                Some(list) => list[index as usize] as i32,
                None => const { DEFAULT_REGISTER_F32 as i32 },
            }
        } else {
            match &self.ints {
                Some(list) => list[index as usize],
                None => DEFAULT_REGISTER_I32,
            }
        }
    }
    
    
    pub fn get_number_register(&mut self, index : u8) -> Number {
        match RegisterType::identify(index)
        {
            RegisterType::F32 => Number::F32(self.get_f32_register(index)),
            RegisterType::I32 => Number::I32(self.get_i32_register(index)),
        }
    }
    
    pub fn read_into_register(&mut self, destination : u8, cursor : &mut Cursor<&'static [u8]>)
    {
        match RegisterType::identify(destination)
        {
            RegisterType::F32 => {
                let immediate = self.cursor_read_f32_with_replacement(cursor);
                self.set_f32_register(destination, immediate);
                self.character.set_condition_register(immediate as i32);
            },
            RegisterType::I32 => {
                let immediate = cursor.read_i32::<LittleEndian>().unwrap();
                self.set_i32_register(destination, immediate);
                self.character.set_condition_register(immediate);
            },
        }
    }
    
    pub fn register_unary_operation(&mut self, source : u8, destination : u8, operation : unary_operators::UnaryOp)
    {
        let ltype = RegisterType::identify(source);
        let rtype = RegisterType::identify(destination);
        
        match ltype
        {
            RegisterType::F32 => {
                let result = unary_operators::operation_f32(
                    self.get_f32_register(source),
                    operation
                );
                
                match rtype {
                    RegisterType::F32 => self.set_f32_register(destination, result),
                    RegisterType::I32 => self.set_i32_register(destination, result as i32),
                }
                
                self.character.set_condition_register(result as i32);
            },
            RegisterType::I32 => {
                let result = unary_operators::operation_i32(
                    self.get_i32_register(source),
                    operation
                );
                
                match rtype {
                    RegisterType::F32 => self.set_f32_register(destination, result as f32),
                    RegisterType::I32 => self.set_i32_register(destination, result),
                }
                
                self.character.set_condition_register(result);
            },
        };
    }
    
    pub fn immediate_unary_operation_f32(&mut self, immediate : f32, destination : u8, operation : unary_operators::UnaryOp)
    {
        let result = unary_operators::operation_f32(
                    immediate,
                    operation
                );
        
        let ltype = RegisterType::identify(destination);
        
        match ltype
        {
            RegisterType::F32 => self.set_f32_register(destination, result),
            RegisterType::I32 => self.set_i32_register(destination, result as i32),
        };
        
        self.character.set_condition_register(result as i32);
    }
    
    pub fn immediate_unary_operation_i32(&mut self, immediate : i32, destination : u8, operation : unary_operators::UnaryOp)
    {
        let result = unary_operators::operation_i32(
                    immediate,
                    operation
                );
        
        let ltype = RegisterType::identify(destination);
        
        match ltype
        {
            RegisterType::F32 => self.set_f32_register(destination, result as f32),
            RegisterType::I32 => self.set_i32_register(destination, result),
        };
        
        self.character.set_condition_register(result);
    }
    
    pub fn register_imm_operation_i32(&mut self, lhs : u8, rhs_imm : i32, destination : u8, operation : binary_operators::BinaryOp)
    {
        let ltype = RegisterType::identify(lhs);
        
        match ltype
        {
            RegisterType::F32 => {
                let result = operation.operate(
                    self.get_f32_register(lhs),
                    rhs_imm
                );
                self.set_i32_register(destination, result);
                self.character.set_condition_register(result as i32);
            },
            RegisterType::I32 => {
                let result = operation.operate(
                    self.get_i32_register(lhs),
                    rhs_imm
                );
                self.set_i32_register(destination, result);
                self.character.set_condition_register(result);
            },
        };
    }
    
    pub fn register_imm_operation_f32(&mut self, lhs : u8, rhs_imm : f32, destination : u8, operation : binary_operators::BinaryOp)
    {
        let ltype = RegisterType::identify(lhs);
        
        match ltype
        {
            RegisterType::F32 => {
                let result = operation.operate(
                    self.get_f32_register(lhs),
                    rhs_imm
                );
                self.set_f32_register(destination, result);
                self.character.set_condition_register(result as i32);
            },
            RegisterType::I32 => {
                let result = operation.operate(
                    self.get_i32_register(lhs),
                    rhs_imm
                );
                self.set_f32_register(destination, result);
                self.character.set_condition_register(result as i32);
            },
        };
    }
    
    pub fn register_register_operation(&mut self, lhs : u8, rhs : u8, destination : u8, operation : binary_operators::BinaryOp)
    {
        let ltype = RegisterType::identify(lhs);
        let rtype = RegisterType::identify(rhs);
        let dtype = RegisterType::identify(destination);
        
        match (ltype, rtype, dtype)
        {
            (RegisterType::F32, RegisterType::F32, RegisterType::F32) => {
                let result = operation.operate(
                    self.get_f32_register(lhs),
                    self.get_f32_register(rhs)
                );
                self.set_f32_register(destination, result);
                self.character.set_condition_register(result as i32);
            },
            (RegisterType::I32, RegisterType::F32, RegisterType::F32) => {
                let result = operation.operate(
                    self.get_i32_register(lhs),
                    self.get_f32_register(rhs)
                );
                self.set_f32_register(destination, result);
                self.character.set_condition_register(result as i32);
            },
            (RegisterType::F32, RegisterType::I32, RegisterType::F32) => {
                let result = operation.operate(
                    self.get_f32_register(lhs),
                    self.get_i32_register(rhs)
                );
                self.set_f32_register(destination, result);
                self.character.set_condition_register(result as i32);
            },
            (RegisterType::I32, RegisterType::I32, RegisterType::F32) => {
                let result = operation.operate(
                    self.get_i32_register(lhs),
                    self.get_i32_register(rhs)
                );
                self.set_f32_register(destination, result);
                self.character.set_condition_register(result as i32);
            },
            (RegisterType::F32, RegisterType::F32, RegisterType::I32) => {
                let result = operation.operate(
                    self.get_f32_register(lhs),
                    self.get_f32_register(rhs)
                );
                self.set_i32_register(destination, result);
                self.character.set_condition_register(result);
            },
            (RegisterType::I32, RegisterType::F32, RegisterType::I32) => {
                let result = operation.operate(
                    self.get_i32_register(lhs),
                    self.get_f32_register(rhs)
                );
                self.set_i32_register(destination, result);
                self.character.set_condition_register(result);
            },
            (RegisterType::F32, RegisterType::I32, RegisterType::I32) => {
                let result = operation.operate(
                    self.get_f32_register(lhs),
                    self.get_i32_register(rhs)
                );
                self.set_i32_register(destination, result);
                self.character.set_condition_register(result);
            },
            (RegisterType::I32, RegisterType::I32, RegisterType::I32) => {
                let result = operation.operate(
                    self.get_i32_register(lhs),
                    self.get_i32_register(rhs)
                );
                self.set_i32_register(destination, result);
                self.character.set_condition_register(result);
            },
        };
    }
}

pub enum RegisterType {
    I32,
    F32,
}

const F32_REGISTER_MASK : u8 = 0x80;
const F32_REGISTER_UNMASK : u8 = !F32_REGISTER_MASK;

impl RegisterType
{
    pub fn identify(index : u8) -> RegisterType
    {
        if index & F32_REGISTER_MASK == F32_REGISTER_MASK {
            RegisterType::F32
        } else {
            RegisterType::I32
        }
    }
}



impl CharStore
{
    pub const F32_RELOAD_MASK : u32 = 0xFFFFFF00;
    
    pub fn read_f32_with_replacement(&mut self, ptr : usize) -> f32
    {
        let value = unsafe { read_ptr_no_check::<u32>(ptr) };
        
        if (value & Self::F32_RELOAD_MASK) != Self::F32_RELOAD_MASK {
            f32::from_bits(value)
        } else {
            let register_index : u8 = (value & 0xFF).try_into().unwrap();
            
            self.get_f32_register(register_index)
        }
    }
    
    pub fn cursor_read_f32_with_replacement(&mut self,  cursor : &mut Cursor<&'static [u8]>) -> f32
    {
        let value = cursor.read_u32::<LittleEndian>().unwrap();
        
        if (value & Self::F32_RELOAD_MASK) != Self::F32_RELOAD_MASK {
            f32::from_bits(value)
        } else {
            let register_index : u8 = (value & 0xFF).try_into().unwrap();
            
            self.get_f32_register(register_index)
        }
    }
    
    pub fn cursor_read_u32_with_replacement(&mut self,  cursor : &mut Cursor<&'static [u8]>) -> u32
    {
        let value = cursor.read_u32::<LittleEndian>().unwrap();
        
        if (value & Self::F32_RELOAD_MASK) != Self::F32_RELOAD_MASK {
            value
        } else {
            let register_index : u8 = (value & 0xFF).try_into().unwrap();
            
            self.get_i32_register(register_index) as u32
        }
    }
    
    pub fn store_f32_for_reload(&mut self, reload : &mut Reload, cursor : &mut Cursor<&'static mut [u8]>, offset : u64)
    {
        cursor.seek(SeekFrom::Start(offset)).unwrap();
        
        let saved = cursor.read_u32::<LittleEndian>().unwrap();
        
        if (saved & Self::F32_RELOAD_MASK) != Self::F32_RELOAD_MASK {
            // early out because nothing to replace
            return;
        }
        
        let to_save = (offset, saved);
        
        let register_index : u8 = (saved & 0xFF).try_into().unwrap();
        
        let replacement_value = self.get_f32_register(register_index).to_bits();
        
        cursor.seek(SeekFrom::Start(offset)).unwrap();
        cursor.write_u32::<LittleEndian>(replacement_value).unwrap();
        
        match &mut reload.original_values {
            None => {
                let mut original_values = Vec::with_capacity(16);
                
                original_values.push(to_save);
                
                reload.original_values = Some(original_values);
            }
            Some(original_values) => {
                original_values.push(to_save);
            },
        };
    }
}

