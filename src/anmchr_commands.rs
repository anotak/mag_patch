// we're going to be doing a lot of unsafe stuff so yeah
#![deny(unsafe_op_in_unsafe_fn)]

use std::io::{Seek, SeekFrom};
use std::mem::size_of;

use num_derive::FromPrimitive;
use byteorder::{LittleEndian, ReadBytesExt};

use crate::hook_helpers::*;
use crate::game_data::*;
use crate::storage;
use crate::storage::{RegisterType, RegisterFlags};
use crate::character_extensions;
use crate::var_rw;
use crate::binary_operators::{BinaryOp,BinaryOpHandler};
use crate::unary_operators::{UnaryOp,UnaryOpHandler};
use crate::math::*;

/// This is the number after the 0x66
/// All commands that start with 0x66 should be ones added by me (anotak). if you want to add commands you should reserve another starting value to prevent conflicts. (game uses commands 0 through 7 inclusive)
#[derive(FromPrimitive)]
#[repr(u32)]
pub enum AnoCmd
{
    RelativeTeleportX = 0x00,
    RelativeTeleportY = 0x01,
    
    LoadImmediateIntoRegister = 0x10,
    BinaryOperationRegisterImmediate = 0x11,
    BinaryOperationRegisterRegister = 0x12,
    UnaryOperationRegister = 0x13,
    UnaryOperationImmediate = 0x14,
    LoadVarIntoRegister = 0x15,
    StoreVarFromRegister = 0x16,
    StoreVarFromImmediate = 0x17,
    BinaryOperationVarRegister = 0x18,
    BinaryOperationVarImmediate = 0x19,
    UnaryOperationVar = 0x1a,
    CheckCharacterName = 0x1b,
    ConditionalBinaryOperation = 0x1c,
    
    GetProjectile = 0x30,
    NextProjectile = 0x31,
    
    LoadProjectileVarIntoRegister = 0x35,
    StoreProjectileVarFromRegister = 0x36,
    
    
    SuckX = 0x50,
    
    
}

/// handle commands starting in 66
pub fn handle_ano_command(command : AnoCmd, exe_char : Char, command_ptr : usize)
{
    match command {
        AnoCmd::RelativeTeleportX => {
            //debug_msg(format!("exe_char = {}\np1c1 = {:#X}\np1c2 = {:#X}\np1c3 = {:#X}\np2c1 = {:#X}\np2c2 = {:#X}\np2c3 = {:#X}", exe_char, get_p1_char1_ptr(), get_p1_char2_ptr(), get_p1_char3_ptr(), get_p2_char1_ptr(), get_p2_char2_ptr(), get_p2_char3_ptr()));
            
            let my_team = exe_char.identify_team();
            //debug_msg(format!("my_team = {:?}", my_team));
            let op_team = my_team.opposite();
            //debug_msg(format!("op_team = {:?}", op_team));
            
            let offset : f32 = {
                let offset = storage::with(
                    exe_char.get_ptr(),
                    |store| {
                        store.read_f32_with_replacement(command_ptr)
                    }
                );
                
                if offset.is_finite() {
                    offset
                } else {
                    0.0
                }
            };
            
            
            // we put this into a local function in order to not duplicate p1/p2 code
            let get_destination = |exe_char : &Char, op_char  : &Char| -> f32
            {
                let op_pos = op_char.get_x_pos();
                let facing = exe_char.get_facing();
                
                let offset = if facing == Facing::Right { -offset } else { offset };
                
                op_pos + offset
            };
            
            
            let x_pos = match op_team {
                Team::Player1 => get_destination(&exe_char, &Char::get_p1_point()),
                Team::Player2 => get_destination(&exe_char, &Char::get_p2_point()),
                Team::Unknown => {
                    let p1 = Char::get_p1_point();
                    let p2 = Char::get_p2_point();
                    
                    // this is basically an error. if we can't identify which team this character
                    // belongs to, then we just teleport them into the average between the two
                    // point chars since that seems like the safest and fairest choice
                    // other options might be: don't move the character?
                    (p1.get_x_pos() + p2.get_x_pos()) * 0.5
                },
            };
        
            exe_char.set_x_pos(x_pos);
        },
        AnoCmd::RelativeTeleportY => {
            let my_team = exe_char.identify_team();
            let op_team = my_team.opposite();
            
            let offset : f32 = {
                let offset = storage::with(
                    exe_char.get_ptr(),
                    |store| {
                        store.read_f32_with_replacement(command_ptr)
                    }
                );
                
                if offset.is_finite() {
                    offset
                } else {
                    0.0
                }
            };
            
            
            
            // we put this into a local function in order to not duplicate p1/p2 code
            let get_destination = |op_char  : &Char| -> f32
            {
                let op_pos = op_char.get_y_pos();
                let hitstun_state = op_char.get_hitstun_non_knockdown();
                
                // TODO - investigate more and see if this holds true all the time
                if op_pos > 0.0 && hitstun_state == HitstunFlagA::HitstunAirStandCrouch {
                    // seems like enemies in juggleable states are offset by 96
                    op_pos + offset - 96.0
                } else {
                    op_pos + offset
                }
            };
            
            
            let y_pos = match op_team {
                Team::Player1 => get_destination(&Char::get_p1_point()),
                Team::Player2 => get_destination(&Char::get_p2_point()),
                Team::Unknown => {
                    let p1 = Char::get_p1_point();
                    let p2 = Char::get_p2_point();
                    
                    // this is basically an error. if we can't identify which team this character
                    // belongs to, then we just teleport them into the average between the two
                    // point chars since that seems like the safest and fairest choice
                    // other options might be: don't move the character?
                    (p1.get_y_pos() + p2.get_y_pos()) * 0.5 + offset
                },
            };
        
            exe_char.set_y_pos(y_pos);
        },
        AnoCmd::LoadImmediateIntoRegister => {
            let mut cursor = unsafe { get_cursor(command_ptr, const { size_of::<u32>() * 2 }) };
            
            cursor.seek(SeekFrom::Current(2)).unwrap();
            let register_flags = RegisterFlags::read(&mut cursor);
            let destination = cursor.read_u8().unwrap();
            
            storage::with(
                exe_char.get_ptr(),
                |store| {
                    let destination = store.resolve_indirect_register(destination, register_flags.is_destination_indirect());
                    
                    store.read_into_register(destination, &mut cursor, register_flags)
                }
            );
        },
        AnoCmd::BinaryOperationRegisterImmediate => {
            let mut cursor = unsafe { get_cursor(command_ptr, const { size_of::<u32>() * 3 }) };
            
            let operation = storage::with(
                    exe_char.get_ptr(),
                    |store| {
                        store.cursor_read_u32_with_replacement(&mut cursor)
                    }
                );
            let operation = num::FromPrimitive::from_u32(operation);
            
            let lhs = cursor.read_u8().unwrap();
            cursor.seek(SeekFrom::Current(1)).unwrap();
            let register_flags = RegisterFlags::read(&mut cursor);
            let destination = cursor.read_u8().unwrap();
            
            
            if let Some(operation) = operation {
                storage::with(
                    exe_char.get_ptr(),
                    |store| {
                        let lhs = store.resolve_indirect_register(lhs, register_flags.is_lhs_indirect());
                        let destination = store.resolve_indirect_register(destination, register_flags.is_destination_indirect());
                        
                        let op_type = RegisterType::identify(destination);
                        
                        match op_type {
                            RegisterType::F32 => {
                                let rhs = store.cursor_read_f32_with_replacement(&mut cursor);
                                
                                store.register_imm_operation_f32(lhs, rhs, destination, operation, register_flags);
                            },
                            RegisterType::I32 | RegisterType::Bool => {
                                let rhs = cursor.read_i32::<LittleEndian>().unwrap();
                                
                                store.register_imm_operation_i32(lhs, rhs, destination, operation, register_flags);
                            },
                        };
                    }
                );
            }
        },
        AnoCmd::BinaryOperationRegisterRegister => {
            let mut cursor = unsafe { get_cursor(command_ptr, const { size_of::<u32>() * 2 }) };
            
            let operation = storage::with(
                    exe_char.get_ptr(),
                    |store| {
                        store.cursor_read_u32_with_replacement(&mut cursor)
                    }
                );
            let operation = num::FromPrimitive::from_u32(operation);
            
            let lhs = cursor.read_u8().unwrap();
            let rhs = cursor.read_u8().unwrap();
            let register_flags = RegisterFlags::read(&mut cursor);
            let destination = cursor.read_u8().unwrap();
            
            if let Some(operation) = operation {
                storage::with(
                    exe_char.get_ptr(),
                    |store| {
                        let lhs = store.resolve_indirect_register(lhs, register_flags.is_lhs_indirect());
                        let rhs = store.resolve_indirect_register(rhs, register_flags.is_rhs_indirect());
                        let destination = store.resolve_indirect_register(destination, register_flags.is_destination_indirect());
                        
                        store.register_register_operation(lhs, rhs, destination, operation, register_flags);
                    }
                );
            }
        },
        AnoCmd::UnaryOperationRegister => {
            
            let mut cursor = unsafe { get_cursor(command_ptr, const { size_of::<u32>() * 2 }) };
            
            let operation = storage::with(
                    exe_char.get_ptr(),
                    |store| {
                        store.cursor_read_u32_with_replacement(&mut cursor)
                    }
                );
            let operation = num::FromPrimitive::from_u32(operation);
            
            let reg = cursor.read_u8().unwrap();
            cursor.seek(SeekFrom::Current(1)).unwrap();
            let register_flags = RegisterFlags::read(&mut cursor);
            let destination = cursor.read_u8().unwrap();
            
            if let Some(operation) = operation {
                storage::with(
                    exe_char.get_ptr(),
                    |store| {
                        let reg = store.resolve_indirect_register(reg, register_flags.is_lhs_indirect());
                        let destination = store.resolve_indirect_register(destination, register_flags.is_destination_indirect());
                        
                        store.register_unary_operation(reg, destination, operation, register_flags);
                    }
                );
            }
        }, 
        AnoCmd::UnaryOperationImmediate => {
            let mut cursor = unsafe { get_cursor(command_ptr, const { size_of::<u32>() * 3 }) };
            
            let operation = storage::with(
                    exe_char.get_ptr(),
                    |store| {
                        store.cursor_read_u32_with_replacement(&mut cursor)
                    }
                );
            let operation = num::FromPrimitive::from_u32(operation);
            
            cursor.seek(SeekFrom::Current(2)).unwrap();
            let register_flags = RegisterFlags::read(&mut cursor);
            let destination = cursor.read_u8().unwrap();
            
            
            if let Some(operation) = operation {
                storage::with(
                    exe_char.get_ptr(),
                    |store| {
                        let destination = store.resolve_indirect_register(destination, register_flags.is_destination_indirect());
                        
                        let op_type = if register_flags.is_lhs_bool() | register_flags.is_destination_bool()
                            {
                                RegisterType::Bool
                            } else {
                                RegisterType::identify(destination)
                            };
                        
                        match op_type {
                            RegisterType::F32 => {
                                let immediate = store.cursor_read_f32_with_replacement(&mut cursor);
                                
                                store.immediate_unary_operation_f32(immediate, destination, operation, register_flags);
                            },
                            RegisterType::I32 => {
                                let immediate = cursor.read_i32::<LittleEndian>().unwrap();
                                
                                store.immediate_unary_operation_i32(immediate, destination, operation, register_flags);
                            },
                            RegisterType::Bool => {
                                let immediate = cursor.read_i32::<LittleEndian>().unwrap();
                                
                                store.immediate_unary_operation_bool(immediate, destination, operation, register_flags);
                            },
                        };
                    }
                );
            }
        },
        AnoCmd::LoadVarIntoRegister => {
            load_var_into_register(exe_char, command_ptr,
                |variable_character, destination_type, var| {
                    match destination_type {
                        RegisterType::F32 => {
                            var_rw::MatchState::load_f32(variable_character.get_ptr(), var).into_number()
                        },
                        RegisterType::I32 => {
                            var_rw::MatchState::load_i32(variable_character.get_ptr(), var).into_number()
                        },
                        RegisterType::Bool => {
                            var_rw::MatchState::load_i32(variable_character.get_ptr(), var).into_number()
                        },
                    }
                }
            )
        }, 
        AnoCmd::StoreVarFromRegister => {
            store_var_from_register(exe_char, command_ptr, |variable_character, var, source_type, source_value| {
                match source_type {
                    RegisterType::F32 => {
                        var_rw::MatchState::store_f32(variable_character.get_ptr(), var, source_value.into_float());
                    },
                    RegisterType::I32 => {
                        var_rw::MatchState::store_i32(variable_character.get_ptr(), var, source_value.into_int());
                    },
                    RegisterType::Bool => {
                        var_rw::MatchState::store_i32(variable_character.get_ptr(), var, source_value.into_int());
                    },
                }
            })
        },
        AnoCmd::StoreVarFromImmediate => {
            store_var_from_immediate(exe_char, command_ptr)
        },
        AnoCmd::BinaryOperationVarRegister => {
            binary_operation_var_register(exe_char, command_ptr)
        },
        AnoCmd::BinaryOperationVarImmediate => {
            binary_operation_var_immediate(exe_char, command_ptr)
        },
        AnoCmd::UnaryOperationVar => {
            unary_operation_var(exe_char, command_ptr)
        },
        AnoCmd::CheckCharacterName => {
            check_character_name(exe_char, command_ptr)
        },
        AnoCmd::SuckX => {
            use character_extensions::SuckOpponent;
            
            let mut cursor = unsafe { get_cursor(command_ptr, const { size_of::<f32>() * 2 }) };
            let (magnitude, delta) = storage::with(
                exe_char.get_ptr(),
                |store| {
                    let magnitude = store.cursor_read_f32_with_replacement(&mut cursor);
                    let delta = store.cursor_read_f32_with_replacement(&mut cursor);
                    
                    (magnitude, delta)
                }
            );
            SuckOpponent::apply_suck(exe_char, magnitude, delta);
        },
        AnoCmd::ConditionalBinaryOperation => {
            conditional_binary_operation(exe_char, command_ptr)
        },
        AnoCmd::GetProjectile => {
            get_projectile(exe_char, command_ptr)
        },
        AnoCmd::NextProjectile => {
            next_projectile(exe_char, command_ptr)
        },
        AnoCmd::LoadProjectileVarIntoRegister => {
            load_var_into_register(exe_char, command_ptr,
                |variable_character, destination_type, var| {
                    match destination_type {
                        RegisterType::F32 => {
                            var_rw::ProjectileState::load_f32(variable_character.get_ptr(), var).into_number()
                        },
                        RegisterType::I32 => {
                            var_rw::ProjectileState::load_i32(variable_character.get_ptr(), var).into_number()
                        },
                        RegisterType::Bool => {
                            var_rw::ProjectileState::load_i32(variable_character.get_ptr(), var).into_number()
                        },
                    }
                }
            )
        },
        
        AnoCmd::StoreProjectileVarFromRegister => {
            store_var_from_register(exe_char, command_ptr, |variable_character, var, source_type, source_value| {
                match source_type {
                    RegisterType::F32 => {
                        var_rw::ProjectileState::store_f32(variable_character.get_ptr(), var, source_value.into_float());
                    },
                    RegisterType::I32 => {
                        var_rw::ProjectileState::store_i32(variable_character.get_ptr(), var, source_value.into_int());
                    },
                    RegisterType::Bool => {
                        var_rw::ProjectileState::store_i32(variable_character.get_ptr(), var, source_value.into_int());
                    },
                }
            })
        },
    }
}


fn load_var_into_register<F>(storage_character : Char, command_ptr : usize, loading_fn : F)
    where F : FnOnce(&Char, RegisterType, u32) -> Number
{
    let mut cursor = unsafe { get_cursor(command_ptr, const { size_of::<u32>() * 2 }) };
    
    cursor.seek(SeekFrom::Current(1)).unwrap();
    let character_relation = CharacterRelation::decode(cursor.read_u8().unwrap());
    let variable_character = storage_character.related_character(character_relation);
    let register_flags = RegisterFlags::read(&mut cursor);
    let destination = cursor.read_u8().unwrap();
    let var = cursor.read_u32::<LittleEndian>().unwrap();
    
    let destination = storage::with(
        storage_character.get_ptr(),
        |store| {
            store.resolve_indirect_register(destination, register_flags.is_destination_indirect())
        });
    
    
    let destination_type = if register_flags.is_destination_bool()
        {
            RegisterType::Bool
        } else {
            RegisterType::identify(destination)
        };
    
    match destination_type {
        RegisterType::F32 => {
            let result = match variable_character {
                Some(ref variable_character) => loading_fn(variable_character, RegisterType::F32, var).into_float(),
                None => 0.0,
            };
            storage::with(
                storage_character.get_ptr(),
                |store| {
                    store.set_f32_register(destination, result);
                }
            );
            
            variable_character.map(|c| c.set_condition_register(result as i32));
        },
        RegisterType::I32 => {
            let result = match variable_character {
                Some(ref variable_character) => loading_fn(variable_character, RegisterType::I32, var).into_int(),
                None => 0,
            };
            
            storage::with(
                storage_character.get_ptr(),
                |store| {
                    store.set_i32_register(destination, result);
                }
            );
            
            variable_character.map(|c| c.set_condition_register(result));
        },
        RegisterType::Bool => {
            let result = match variable_character {
                Some(ref variable_character) => loading_fn(variable_character, RegisterType::I32, var).into_int(),
                None => 0,
            };
            
            storage::with(
                storage_character.get_ptr(),
                |store| {
                    store.set_bool(destination, result.is_true());
                }
            );
            
            variable_character.map(|c| c.set_condition_register(result));
        },
    };
}

fn store_var_from_register<F>(storage_character : Char, command_ptr : usize, storing_fn : F)
    where F : FnOnce(&Char, u32, RegisterType, Number)
{
    let mut cursor = unsafe { get_cursor(command_ptr, const { size_of::<u32>() * 2 }) };
    
    cursor.seek(SeekFrom::Current(1)).unwrap();
    let character_relation = CharacterRelation::decode(cursor.read_u8().unwrap());
    
    let variable_character = {
        match storage_character.related_character(character_relation) {
            Some(variable_character) => variable_character,
            // just early out if we def cant figure out what character we're doing this to
            None => return,
        }
    };
    
    let register_flags = RegisterFlags::read(&mut cursor);
    let source = cursor.read_u8().unwrap();
    let var = cursor.read_u32::<LittleEndian>().unwrap();
    
    let (source_type, source_value) = storage::with(
            storage_character.get_ptr(),
            |store| {
                let source = store.resolve_indirect_register(source, register_flags.is_lhs_indirect());
                
                let source_type = if register_flags.is_lhs_bool()
                {
                    RegisterType::Bool
                } else {
                    RegisterType::identify(source)
                };
                
                let source_value = match source_type {
                    RegisterType::F32 => {
                        Number::F32(store.get_f32_register(source))
                    },
                    RegisterType::I32 => {
                        Number::I32(store.get_i32_register(source))
                    },
                    RegisterType::Bool => {
                        Number::I32(store.get_bool(source).from_bool())
                    },
                };
                
                (source_type, source_value)
            }
        );
    
    storing_fn(&variable_character, var, source_type, source_value);
    
    variable_character.set_condition_register(source_value.into_int());
}


fn store_var_from_immediate(storage_character : Char, command_ptr : usize)
{
    let mut cursor = unsafe { get_cursor(command_ptr, const { size_of::<u32>() * 3 }) };
    
    cursor.seek(SeekFrom::Current(1)).unwrap();
    let character_relation = CharacterRelation::decode(cursor.read_u8().unwrap());
    
    let variable_character = {
        match storage_character.related_character(character_relation) {
            Some(variable_character) => variable_character,
            // just early out if we def cant figure out what character we're doing this to
            None => return,
        }
    };
    
    let _register_flags = RegisterFlags::read(&mut cursor);
    cursor.seek(SeekFrom::Current(1)).unwrap();
    
    let var = cursor.read_u32::<LittleEndian>().unwrap();
    
    let variable_type = var_rw::MatchState::get_number_type(var);
    
    match variable_type {
        Some(RegisterType::F32) => {
            let immediate = storage::with(
                    storage_character.get_ptr(),
                    |store| {
                        store.cursor_read_f32_with_replacement(&mut cursor)
                    }
                );
            
            var_rw::MatchState::store_f32(variable_character.get_ptr(), var, immediate);
        },
        Some(RegisterType::I32 | RegisterType::Bool) => {
            let immediate = cursor.read_i32::<LittleEndian>().unwrap();
            
            var_rw::MatchState::store_i32(variable_character.get_ptr(), var, immediate);
        },
        None => {},
    };
}


fn binary_operation_var_register(storage_character : Char, command_ptr : usize)
{
    let mut cursor = unsafe { get_cursor(command_ptr, const { size_of::<u32>() * 3 }) };
    
    let operation = storage::with(
            storage_character.get_ptr(),
            |store| {
                store.cursor_read_u32_with_replacement(&mut cursor)
            }
        );
    let operation : Option<BinaryOp> = num::FromPrimitive::from_u32(operation);
    
    let rhs = cursor.read_u8().unwrap();
    
    let character_relation = CharacterRelation::decode(cursor.read_u8().unwrap());
    
    let variable_character = {
        match storage_character.related_character(character_relation) {
            Some(variable_character) => variable_character,
            // just early out if we def cant figure out what character we're doing this to
            None => return,
        }
    };
    
    
    let register_flags = RegisterFlags::read(&mut cursor);
    
    cursor.seek(SeekFrom::Current(1)).unwrap();
    
    let var = cursor.read_u32::<LittleEndian>().unwrap();
    
    let variable_type = var_rw::MatchState::get_number_type(var);
    
    let variable_type = if variable_type == None {
            None
        } else if register_flags.is_rhs_bool() {
            Some(RegisterType::Bool)
        } else {
            var_rw::MatchState::get_number_type(var)
        };
    
    let lhs = var_rw::MatchState::load_number(variable_character.get_ptr(), var);
    
    // these two branches look fairly identical but the type has to be carried through
    match (variable_type, operation) {
        (Some(RegisterType::F32), Some(operation)) => {
            let rhs = storage::with(
                storage_character.get_ptr(),
                |store| {
                    let rhs = store.resolve_indirect_register(rhs, register_flags.is_rhs_indirect());
                    
                    store.get_number_register(rhs)
                }
            );
            
            let result = operation.operate(lhs, rhs);
            
            var_rw::MatchState::store_f32(variable_character.get_ptr(), var, result);
        },
        (Some(RegisterType::I32), Some(operation)) => {
            let rhs = storage::with(
                storage_character.get_ptr(),
                |store| {
                    let rhs = store.resolve_indirect_register(rhs, register_flags.is_rhs_indirect());
                    
                    store.get_number_register(rhs)
                }
            );
            
            let result = operation.operate(lhs, rhs);
            
            var_rw::MatchState::store_i32(variable_character.get_ptr(), var, result);
        },
        (Some(RegisterType::Bool), Some(operation)) => {
            let rhs = storage::with(
                storage_character.get_ptr(),
                |store| {
                    let rhs = store.resolve_indirect_register(rhs, register_flags.is_rhs_indirect());
                    
                    store.get_bool(rhs)
                }
            );
            
            let result = operation.operate(lhs, rhs);
            
            var_rw::MatchState::store_i32(variable_character.get_ptr(), var, result);
        },
        _ => {},
    };
}

fn binary_operation_var_immediate(storage_character : Char, command_ptr : usize)
{
    let mut cursor = unsafe { get_cursor(command_ptr, const { size_of::<u32>() * 4 }) };
    
    let operation = storage::with(
            storage_character.get_ptr(),
            |store| {
                store.cursor_read_u32_with_replacement(&mut cursor)
            }
        );
    let operation : Option<BinaryOp> = num::FromPrimitive::from_u32(operation);
    
    cursor.seek(SeekFrom::Current(1)).unwrap();
    
    let character_relation = CharacterRelation::decode(cursor.read_u8().unwrap());
    
    let variable_character = {
        match storage_character.related_character(character_relation) {
            Some(variable_character) => variable_character,
            // just early out if we def cant figure out what character we're doing this to
            None => return,
        }
    };
    
    cursor.seek(SeekFrom::Current(2)).unwrap();
    
    let var = cursor.read_u32::<LittleEndian>().unwrap();
    
    let variable_type = var_rw::MatchState::get_number_type(var);
    
    let lhs = var_rw::MatchState::load_number(variable_character.get_ptr(), var);
    
    // these two branches look fairly identical but the type has to be carried through
    match (variable_type, operation) {
        (Some(RegisterType::F32), Some(operation)) => {
            let rhs = storage::with(
                storage_character.get_ptr(),
                |store| {
                    store.cursor_read_f32_with_replacement(&mut cursor)
                }
            );
            
            let result = operation.operate(lhs, rhs);
            
            var_rw::MatchState::store_f32(variable_character.get_ptr(), var, result);
        },
        (Some(RegisterType::I32), Some(operation)) => {
            let rhs = cursor.read_i32::<LittleEndian>().unwrap();
            
            let result = operation.operate(lhs, rhs);
            
            var_rw::MatchState::store_i32(variable_character.get_ptr(), var, result);
        },
        _ => {},
    };
}

fn unary_operation_var(storage_character : Char, command_ptr : usize)
{
    let mut cursor = unsafe { get_cursor(command_ptr, const { size_of::<u32>() * 4 }) };
    
    let operation = storage::with(
            storage_character.get_ptr(),
            |store| {
                store.cursor_read_u32_with_replacement(&mut cursor)
            }
        );
    let operation : Option<UnaryOp> = num::FromPrimitive::from_u32(operation);
    
    cursor.seek(SeekFrom::Current(1)).unwrap();
    
    let character_relation = CharacterRelation::decode(cursor.read_u8().unwrap());
    
    let variable_character = {
        match storage_character.related_character(character_relation) {
            Some(variable_character) => variable_character,
            // just early out if we def cant figure out what character we're doing this to
            None => return,
        }
    };
    
    cursor.seek(SeekFrom::Current(2)).unwrap();
    
    let var = cursor.read_u32::<LittleEndian>().unwrap();
    
    let variable_type = var_rw::MatchState::get_number_type(var);
    let input = var_rw::MatchState::load_number(variable_character.get_ptr(), var);
    
    // these two branches look fairly identical but the type has to be carried through
    match (variable_type, operation) {
        (Some(RegisterType::F32), Some(operation)) => {
            let result = operation.operate(input.into_float());
            
            var_rw::MatchState::store_f32(variable_character.get_ptr(), var, result);
        },
        (Some(RegisterType::I32), Some(operation)) => {
            let result = operation.operate(input.into_int());
            
            var_rw::MatchState::store_i32(variable_character.get_ptr(), var, result);
        },
        _ => {},
    };
}

fn conditional_binary_operation(storage_character : Char, command_ptr : usize)
{
    let mut cursor = unsafe { get_cursor(command_ptr, const { size_of::<u32>() * 6 }) };
    
    let (comparator_operation, result_operation) = storage::with(
            storage_character.get_ptr(),
            |store| {
                (store.cursor_read_u32_with_replacement(&mut cursor),
                store.cursor_read_u32_with_replacement(&mut cursor))
            }
        );
    let comparator_operation : Option<BinaryOp> = num::FromPrimitive::from_u32(comparator_operation);
    let result_operation = num::FromPrimitive::from_u32(result_operation);
    
    
    
    let lhs = cursor.read_u8().unwrap();
    cursor.seek(SeekFrom::Current(1)).unwrap();
    let register_flags = RegisterFlags::read(&mut cursor);
    let destination = cursor.read_u8().unwrap();
    
    
    if let Some(comparator_operation) = comparator_operation 
        && let Some(result_operation) = result_operation {
        storage::with(
            storage_character.get_ptr(),
            |store| {
                let lhs = store.resolve_indirect_register(lhs, register_flags.is_lhs_indirect());
                let destination = store.resolve_indirect_register(destination, register_flags.is_destination_indirect());
                
                let op_type = if register_flags.is_destination_bool()
                    {
                        RegisterType::Bool
                    } else {
                        RegisterType::identify(destination)
                    };
                
                let lhs_type = if register_flags.is_lhs_bool()
                    {
                        RegisterType::Bool
                    } else {
                        RegisterType::identify(lhs)
                    };
                
                let is_condition_passed = {
                    match lhs_type {
                        RegisterType::I32 => {
                            let rhs_comparator = cursor.read_i32::<LittleEndian>().unwrap();
                            let comparison = comparator_operation.operate(store.get_number_register(lhs), rhs_comparator);
                            
                            comparison.is_true()
                        },
                        RegisterType::F32 => {
                            let rhs_comparator = store.cursor_read_f32_with_replacement(&mut cursor);
                            let comparison = comparator_operation.operate(store.get_number_register(lhs), rhs_comparator);
                            
                            comparison.is_true()
                        },
                        RegisterType::Bool => {
                            let rhs_comparator = cursor.read_i32::<LittleEndian>().unwrap();
                            let comparison = comparator_operation.operate(store.get_bool(lhs), rhs_comparator.is_true());
                            
                            comparison.is_true()
                        }
                    }
                };
                    
                if is_condition_passed {
                    let result_register_flags = register_flags.set_lhs_bool(false);
                    
                    match op_type {
                        RegisterType::F32 => {
                            let rhs_for_op = store.cursor_read_f32_with_replacement(&mut cursor);
                            
                            store.register_imm_operation_f32(destination, rhs_for_op, destination, result_operation, result_register_flags);
                        },
                        RegisterType::I32 | RegisterType::Bool => {
                            let rhs_for_op = cursor.read_i32::<LittleEndian>().unwrap();
                            
                            store.register_imm_operation_i32(destination, rhs_for_op, destination, result_operation, result_register_flags);
                        },
                    };
                    
                    match lhs_type {
                        RegisterType::I32 => {
                            let immediate = cursor.read_i32::<LittleEndian>().unwrap();
                            
                            store.set_i32_register(lhs, immediate);
                        },
                        RegisterType::F32 => {
                            let immediate = store.cursor_read_f32_with_replacement(&mut cursor);
                            
                            store.set_f32_register(lhs, immediate);
                        },
                        RegisterType::Bool => {
                            let immediate = cursor.read_i32::<LittleEndian>().unwrap().is_true();
                            
                            store.set_bool(destination, immediate);
                        }
                    }
                }
            }
        );
    }
}

fn check_character_name(storage_character : Char, command_ptr : usize)
{
    let cursor_size = const { size_of::<u32>() * 1 + size_of::<u8>() * 64 };
    let mut cursor = unsafe { get_cursor(command_ptr, cursor_size) };
    
    cursor.seek(SeekFrom::Current(1)).unwrap();
    
    let character_relation = CharacterRelation::decode(cursor.read_u8().unwrap());
    
    let variable_character = {
        match storage_character.related_character(character_relation) {
            Some(variable_character) => variable_character,
            // just early out if we def cant figure out what character we're doing this to
            None => return,
        }
    };
    
    let register_flags = RegisterFlags::read(&mut cursor);
    let destination = cursor.read_u8().unwrap();
    
    let get_character_name_ptr = external_fn!(EXE_BASE + 0x58F90, extern "win64" fn(i32) -> *const u8);
    
    // not ideal, but we're doing our own strlen style comparison here because none of the rust library functions quite match our use-case. if we have more string stuff then this should really be factored out into a separate function, but for now this is the only instance of this in the code
    let id = variable_character.get_char_id();
    
    let name_ptr = get_character_name_ptr(id) as usize;
    
    let mut name_cursor = unsafe { get_cursor(name_ptr, cursor_size) };
    
    let mut is_match = true;
    
    for _ in 0..64 {
        let expected_char = cursor.read_u8().unwrap();
        let actual_char = name_cursor.read_u8().unwrap();
        
        
        
        if expected_char != actual_char {
            is_match = false;
            break;
        } else if expected_char == 0x00 {
            break;
        }
    }
    
    let result = if is_match { 1 } else { 0 };
    
    
    storage::with(
        storage_character.get_ptr(),
        |store| {
            use crate::math::Number;
            
            let destination = store.resolve_indirect_register(destination, register_flags.is_destination_indirect());
            
            if register_flags.is_destination_bool()
            {
                store.set_bool(destination, is_match);
            } else {
                store.set_number_register(destination, Number::I32(result));
            };
            
            storage_character.set_condition_register(result);
        }
    );
}

fn get_projectile(storage_character : Char, command_ptr : usize)
{
    let mut cursor = unsafe { get_cursor(command_ptr, const { size_of::<u32>() * 2 }) };
    
    cursor.seek(SeekFrom::Current(1)).unwrap();
    
    let character_relation = CharacterRelation::decode(cursor.read_u8().unwrap());
    
    
    cursor.seek(SeekFrom::Current(2)).unwrap();
    
    let filter_flags = ProjectileFilterFlags::read(&mut cursor);
    
    let variable_character = {
        match storage_character.related_character(character_relation) {
            Some(variable_character) => variable_character,
            // just early out if we def cant figure out what character we're doing this to
            None => return,
        }
    };
    
    // note that we get the projectiles of variable_character
    // but we're storing on storage_character
    let projectile_filter = variable_character.get_projectiles(filter_flags);
    
    let result = storage::with(
        storage_character.get_ptr(),
        |store| {
            match projectile_filter {
                None => {
                    store.projectile_filter = None;
                    
                    i32::FALSE
                },
                Some(mut projectile_filter) => {
                    projectile_filter.step();
                    
                    match projectile_filter.projectile {
                        Some(_projectile) => {
                            store.projectile_filter = Some(projectile_filter);
                            
                            i32::TRUE
                        },
                        None => i32::FALSE,
                    }
                }
            }
        }
    );
    
    storage_character.set_condition_register(result);
}



fn next_projectile(storage_character : Char, _command_ptr : usize)
{
    let result = storage::with(
        storage_character.get_ptr(),
        |store| {
            match &mut store.projectile_filter {
                None => {
                    store.projectile_filter = None;
                    
                    i32::FALSE
                },
                Some(projectile_filter) => {
                    projectile_filter.step();
                    
                    match projectile_filter.projectile {
                        Some(_projectile) => {
                            store.projectile_filter = Some(projectile_filter.clone());
                            
                            i32::TRUE
                        },
                        None => {
                            store.projectile_filter = None;
                            
                            i32::FALSE
                        }
                    }
                }
            }
        }
    );
    
    storage_character.set_condition_register(result);
}
