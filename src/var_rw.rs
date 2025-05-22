//! gameplay variables to be read and written by anmchr commands

#![deny(unsafe_op_in_unsafe_fn)]

use crate::match_state;
use crate::game_data::Char;
use crate::math::*;

/// used internally here inside the var_rw! macro to convey type, then whatever cast is needed is done
enum Number {
    F32(f32),
    I32(i32)
}

macro_rules! var_rw {
    
    {
        { $type_name:ident };
        
        $( ( $id:literal, $(#[$($attrss:tt)*])* $name:ident, $read:expr, $write:expr $(,)*) ),+
        $(,)*
    } => {
        use num_derive::FromPrimitive;
        #[derive(FromPrimitive)]
        #[repr(u32)]
        pub enum $type_name
        {
            $(
                $(#[$($attrss)*])*
                $name = $id,
            )+
        }
        
        #[expect(clippy::unnecessary_cast)]
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
        /// The game timer, as in 0-99. -1 if the timer is infinite.
        Timer,
        |_| {  Number::F32(match_state::get_match_game_time()) },
        |_, new_value| { match_state::set_match_game_time(new_value as f32); }
    ),
    (
        0x01,
        /// Counts up from the start of the current match state once per frame
        FrameTimerReadOnly,
        |_| {  Number::F32(match_state::get_match_frame_time()) },
        |_, _| { },
    ),
    (
        0x02,
        MatchStateReadOnly,
        |_| {  Number::I32(match_state::get_match_state() as i32) },
        |_, _| { },
    ),
    (
        0x10,
        Health,
        |ptr| {
            Number::F32(Char::if_valid_ancestor(ptr, 0.0, |c| {
                c.get_health()
            }))
        },
        |ptr, new_value| {
            Char::if_valid_ancestor(ptr, (), |mut c| {
                c.set_health(new_value as f32)
            })
        },
    ),
    (
        0x11,
        RedHealth,
        |ptr| {
            Number::F32(Char::if_valid_ancestor(ptr, 0.0, |c| {
                c.get_red_health()
            }))
        },
        |ptr, new_value| {
            Char::if_valid_ancestor(ptr, (), |mut c| {
                c.set_red_health(new_value as f32)
            })
        },
    ),
    (
        0x12,
        MaxHealth,
        |ptr| {
            Number::I32(Char::if_valid_ancestor(ptr, 0, |c| {
                c.get_max_health()
            }))
        },
        |ptr, new_value| {
            Char::if_valid_ancestor(ptr, (), |mut c| {
                c.set_max_health(new_value as i32)
            })
        },
    ),
    (
        0x13,
        /// Currently executing anmchr index
        AnmChrIdReadonly,
        |ptr| {
            Number::I32(Char::if_valid(ptr, 0, |c| {
                c.get_anmchr_id()
            }))
        },
        |_, _| {
        },
    ),
    (
        0x20,
        /// 0.0 is the middle of the stage
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
        /// Floor is 0.0, upward is positive
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
        /// counts up from 0 every time you do a special in the air. is used to limit specials to 3 normally
        SpecialAirActionCounter,
        |ptr| {
            Number::I32(Char::if_valid_ancestor(ptr, 0, |c| {
                c.get_special_air_action_counter()
            }))
        },
        |ptr, new_value| {
            Char::if_valid_ancestor(ptr, (), |c| {
                c.set_special_air_action_counter(new_value as i32)
            })
        },
    ),
    (
        0x31,
        /// counts up from 0 for every time you switch button strength in an air chain. is used to limit how much you can chain in normal jump mode
        NormalAirActionCounter,
        |ptr| {
            Number::I32(Char::if_valid_ancestor(ptr, 0, |c| {
                c.get_normal_air_action_counter()
            }))
        },
        |ptr, new_value| {
            Char::if_valid_ancestor(ptr, (), |c| {
                c.set_normal_air_action_counter(new_value as i32)
            })
        },
    ),
    (
        0x32,
        /// combo counter for just this current character
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
        0x33,
        /// the extra cooldown off assist this specific character has
        AssistCooldown,
        |ptr| {
            Number::F32(Char::if_valid_ancestor(ptr, 0.0, |c| {
                c.get_assist_cooldown()
            }))
        },
        |ptr, new_value| {
            Char::if_valid_ancestor(ptr, (), |mut c| {
                c.set_assist_cooldown(new_value as f32)
            })
        },
    ),
    (
        0x40,
        /// super meter. 50000.0 is the max. 10000.0 is one bar
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
        /// the combo counter for the whole team. read only because this is derived from the character specific combo counters.
        TeamComboCounterReadOnly,
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
        0x42,
        /// Inputs as flags:
        /// 1 = fwd, 2 = bwd, 4 = up, 8 = down,
        /// 0x10 = l, 0x20 = m, 0x40 = h, 0x80 = s,
        /// 0x100 = a1, 0x200 = a2,
        /// 0x100000 = taunt
        InputsReadOnly,
        |ptr| {
            Number::I32(Char::if_valid_point(ptr, 0, |c| {
                c.get_inputs_raw()
            }))
        },
        |_, _| {
            // don't set
        },
    ),
    (
        0x43,
        /// controller forward / backward as a number, +1 is holding forward, -1 is holding backward, 0 is neutral
        InputsForwardBackwardReadOnly,
        |ptr| {
            Number::I32(Char::if_valid_point(ptr, 0, |c| {
                c.get_input_axis_forward_backward()
            }))
        },
        |_, _| {
            // don't set
        },
    ),
    (
        0x44,
        /// controller up / down as a number, +1 is holding up, -1 is holding down, 0 is neutral
        InputsUpDownReadOnly,
        |ptr| {
            Number::I32(Char::if_valid_point(ptr, 0, |c| {
                c.get_input_axis_up_down()
            }))
        },
        |_, _| {
            // don't set
        },
    ),
    
    (
        0xB0,
        /// 1 if facing left, 0 if facing right
        FacingReadOnly,
        |ptr| {
            Number::I32(Char::if_valid(ptr, 0, |c| {
                i32_bool(c.get_facing() == crate::game_data::Facing::Left)
            }))
        },
        |_, _| {
            // don't set
        },
    ),
    (
        0xB1,
        /// current position on the team. 0 = point character. 1 = assist 1. 2 = assist 2.
        CharOrderReadOnly,
        |ptr| {
            Number::I32(Char::if_valid_ancestor(ptr, 0, |c| {
                c.get_char_order_raw()
            }))
        },
        |_, _| {
            // don't set
        },
    ),
    (
        0xB2,
        /// current assist type. 0 = alpha, 1 = beta, 2 = gamma
        AssistType,
        |ptr| {
            Number::I32(Char::if_valid_ancestor(ptr, 0, |c| {
                c.get_assist_type()
            }))
        },
        |ptr, new_value| {
            Char::if_valid_ancestor(ptr, (), |mut c| {
                c.set_assist_type(new_value as i32)
            })
        },
    ),
    
    (
        0xC0,
        /// the condition register used by commands like 0_46 (rng) or 1_92 (check dhc)
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
    
    (
        0xF0,
        /// the flags that are affected by the 1_2f - 1_34 commands
        AirGroundStateFlags,
        |ptr| {
            Number::I32(Char::if_valid(ptr, 0, |c| {
                c.get_air_ground_state_flags()
            }))
        },
        |ptr, new_value| {
            Char::if_valid(ptr, (), |c| {
                c.set_air_ground_state_flags(new_value as i32)
            })
        },
    ),
    
    
    (
        0x1000,
        /// the flying screen install flag, which is normally set by launching the opponent. if it's 1 then a jump S will cause flying screen and a hard kd
        FlyingScreenInstallFlag,
        |ptr| {
            Number::I32(Char::if_valid_ancestor(ptr, 0, |c| {
                i32_bool(c.get_flying_screen_install())
            }))
        },
        |ptr, new_value| {
            Char::if_valid_ancestor(ptr, (), |c| {
                c.set_flying_screen_install(is_i32_true(new_value as i32))
            })
        },
    ),
}