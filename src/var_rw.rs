//! gameplay variables to be read and written by anmchr commands

use crate::match_state;
use crate::game_data::Char;

/// used internally here inside the var_rw! macro to convey type, then whatever cast is needed is done
enum Number {
    F32(f32),
    I32(i32)
}

macro_rules! var_rw {
    
    {
        { $type_name:ident };
        
        $( ( $id:literal, $name:ident, $read:expr, $write:expr $(,)*) ),+
        $(,)*
    } => {
        use num_derive::FromPrimitive;
        #[derive(FromPrimitive)]
        #[repr(u32)]
        pub enum $type_name
        {
            $(
                $name = $id,
            )+
        }
        
        impl $type_name {
            
            fn load(owner_ptr : usize, var : u32) -> Number
            {
                let var = num::FromPrimitive::from_u32(var);
                
                if let Some(var) = var {
                    let func : fn(usize) -> Number = match var {
                        $(
                        $type_name::$name => $read,
                        )+
                    };
                    
                    func(owner_ptr)
                } else {
                    Number::I32(0)
                }
            }
            
            pub fn load_f32(owner_ptr : usize, var : u32) -> f32
            {
                use crate::var_rw::Number;
                
                match $type_name::load(owner_ptr, var)
                {
                    Number::I32(i) => i as f32,
                    Number::F32(f) => {
                        if f.is_finite() {
                            f
                        } else {
                            0.0
                        }
                    },
                }
            }
            
            pub fn store_f32(owner_ptr : usize, var : u32, new_value : f32)
            {
                let var = num::FromPrimitive::from_u32(var);
                
                if let Some(var) = var {
                    let func : fn(usize, f32) = match var {
                        $(
                        $type_name::$name => $write,
                        )+
                    };
                    
                    func(owner_ptr, new_value);
                }
            }
            
            pub fn load_i32(owner_ptr : usize, var : u32) -> i32
            {
                use crate::var_rw::Number;
                
                match $type_name::load(owner_ptr, var)
                {
                    Number::I32(i) => i,
                    Number::F32(f) => {
                        if f.is_finite() {
                            f as i32
                        } else {
                            0
                        }
                    },
                }
            }
            
            pub fn store_i32(owner_ptr : usize, var : u32, new_value : i32)
            {
                let var = num::FromPrimitive::from_u32(var);
                
                
                if let Some(var) = var {
                    let func : fn(usize, i32) = match var {
                        $(
                        $type_name::$name => $write,
                        )+
                    };
                    
                    func(owner_ptr, new_value);
                }
            }
        }
    }
}

var_rw! {
    { MatchState };
    
    (
        0x00,
        Timer,
        |_| {  Number::F32(match_state::get_match_game_time()) },
        |_, new_value| { match_state::set_match_game_time(new_value as f32); }
    ),
    (
        0x01,
        FrameTimerReadOnly,
        |_| {  Number::F32(match_state::get_match_frame_time()) },
        |_, _| { () },
    ),
    (
        0x02,
        MatchStateReadOnly,
        |_| {  Number::I32(match_state::get_match_state() as i32) },
        |_, _| { () },
    ),
    (
        0x10,
        Health,
        |ptr| {
            Number::F32(Char::if_valid(ptr, 0.0, |c| {
                c.get_health()
            }))
        },
        |ptr, new_value| {
            Char::if_valid(ptr, (), |mut c| {
                c.set_health(new_value as f32)
            })
        },
    ),
    (
        0x20,
        XPosition,
        |ptr| {
            Number::F32(Char::if_valid(ptr, 0.0, |c| {
                c.get_x_pos()
            }))
        },
        |ptr, new_value| {
            Char::if_valid(ptr, (), |c| {
                c.set_x_pos(new_value as f32)
            })
        },
    ),
    (
        0x21,
        YPosition,
        |ptr| {
            Number::F32(Char::if_valid(ptr, 0.0, |c| {
                c.get_y_pos()
            }))
        },
        |ptr, new_value| {
            Char::if_valid(ptr, (), |c| {
                c.set_y_pos(new_value as f32)
            })
        },
    ),
    (
        0x30,
        SpecialAirActionCounter,
        |ptr| {
            Number::I32(Char::if_valid(ptr, 0, |c| {
                c.get_special_air_action_counter()
            }))
        },
        |ptr, new_value| {
            Char::if_valid(ptr, (), |c| {
                c.set_special_air_action_counter(new_value as i32)
            })
        },
    ),
    (
        0x31,
        NormalAirActionCounter,
        |ptr| {
            Number::I32(Char::if_valid(ptr, 0, |c| {
                c.get_normal_air_action_counter()
            }))
        },
        |ptr, new_value| {
            Char::if_valid(ptr, (), |c| {
                c.set_normal_air_action_counter(new_value as i32)
            })
        },
    ),
    (
        0x32,
        CharacterComboCounter,
        |ptr| {
            Number::I32(Char::if_valid(ptr, 0, |c| {
                c.get_character_combo_counter()
            }))
        },
        |ptr, new_value| {
            Char::if_valid(ptr, (), |c| {
                c.set_character_combo_counter(new_value as i32)
            })
        },
    ),
    (
        0x40,
        Meter,
        |ptr| {
            Number::F32(Char::if_valid(ptr, 0.0, |c| {
                if let Some(player) = c.player() {
                    player.get_meter()
                } else {
                    0.0
                }
            }))
        },
        |ptr, new_value| {
            Char::if_valid(ptr, (), |c| {
                if let Some(mut player) = c.player() {
                    player.set_meter(new_value as f32)
                }
            })
        },
    ),
    (
        0x41,
        TeamComboCounter,
        |ptr| {
            Number::I32(Char::if_valid(ptr, 0, |c| {
                if let Some(player) = c.player() {
                    player.get_team_combo_counter()
                } else {
                    0
                }
            }))
        },
        |_, _| {
            // don't set
        },
    ),
    (
        0xC0,
        ConditionRegister,
        |ptr| {
            Number::I32(Char::if_valid(ptr, 0, |c| {
                c.get_condition_register()
            }))
        },
        |ptr, new_value| {
            Char::if_valid(ptr, (), |c| {
                c.set_condition_register(new_value as i32)
            })
        },
    ),
    
    
    // TODO - special air action counter, normal air action counter, combo counter, camera position
}