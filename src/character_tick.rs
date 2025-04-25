/// the base game's char specific update functions are all in here, we're just hooking all of them to override all of them

use crate::game_data::*;
use crate::storage;
use std::sync::{LazyLock, Mutex};

#[deny(unsafe_op_in_unsafe_fn)]

use crate::match_state::*;

#[derive(Copy,Clone,PartialEq, Eq, Debug)]
enum RestartState {
    Awaiting,
    JustRestarted,
}

static RESTART_STATE : LazyLock<Mutex<RestartState>> = LazyLock::new(|| {
    Mutex::new(RestartState::Awaiting)
});

const RESTART_TIME : f32 = 1.01;

fn generic_character_tick(owner : Char) {
    let match_state = get_match_state();
    let timer = get_match_frame_time();
    let mut restart_state = RESTART_STATE.lock().unwrap();
    
    match *restart_state {
        RestartState::Awaiting => {
            match match_state {
                // I'm uncertain if PreIntro can run without an Intro and vice-versa
                // but since this is all cleanup, it's fine to duplicate it at that stage probably
                MatchState::RestartingFadeIn | MatchState::PreIntro | MatchState::Intro =>
                {
                    if timer <= RESTART_TIME {
                        *restart_state = RestartState::JustRestarted;
                        
                        storage::reset_all();
                        
                        //debug_msg(format!("char addr? = {:#X}", owner));
                        //debug_msg(format!("is restarting! {}", timer));
                    }
                },
                _ => (),
            }
        },
        RestartState::JustRestarted => {
            if timer > RESTART_TIME {
                //debug_msg(format!("reset restarter! {}", timer));
                
                *restart_state = RestartState::Awaiting;
            }
        },
    }
    
    storage::with_no_make(owner.get_ptr(), |store|
        {
            store.suck_opponent.handle_suck(owner)
        }
    );
}


macro_rules! character_ticks {
    
    {
        $( ( $addr:expr, $name:ident ) ),+
        $(,)*
    } => {
        pub mod tick_hooks_macro_generated {
            use crate::hook_helpers::*;
            
            pub fn hook_character_ticks() -> Result<(), Box<dyn std::error::Error>> {
                $(
                make_hook($addr, $name as usize)?;
                )+
                
                Ok(())
            }
            
            $(
            fn $name(owner : *const ())
            {
                let original: fn(*const ()) -> () = {
                    // i'd love to have real error handling instead of just .unwrapping
                    // but also i dont know how i'd even begin to do that in this environ
                    // of a hooked function
                    
                    let hooks = HOOKS.lock().unwrap();
                    let hook = hooks.get(&($name as usize)).unwrap();
                    let trampoline = hook.trampoline();
                    unsafe { std::mem::transmute(trampoline) }
                };
                
                crate::character_tick::generic_character_tick(crate::game_data::Char::new(owner as usize));
                
                original(owner);
            }
            )+
        }
    }
}


character_ticks! {
    (EXE_BASE + 0x600b0, amaterasu_character_tick),
    (EXE_BASE + 0x66c20, captain_america_character_tick),
    (EXE_BASE + 0x69310, chris_character_tick),
    (EXE_BASE + 0x6b7c0, chun_li_character_tick),
    (EXE_BASE + 0x6d670, crimson_viper_character_tick),
    (EXE_BASE + 0x700a0, dante_character_tick),
    (EXE_BASE + 0x77670, dormammu_character_tick),
    (EXE_BASE + 0x79eb0, doctor_doom_character_tick),
    (EXE_BASE + 0x7ad00, dr_strange_character_tick),
    (EXE_BASE + 0x7d990, dr_strange_sh_character_tick),
    (EXE_BASE + 0x81310, felicia_for_c_character_tick),
    (EXE_BASE + 0x89400, ghost_rider_character_tick),
    (EXE_BASE + 0x8c3c0, akuma_character_tick),
    (EXE_BASE + 0x8e370, haggar_character_tick),
    (EXE_BASE + 0x90810, hawkeye_character_tick),
    (EXE_BASE + 0x92b60, strider_character_tick),
    (EXE_BASE + 0x95b40, hulk_character_tick),
    (EXE_BASE + 0x97740, iron_fist_character_tick),
    (EXE_BASE + 0x99e30, iron_man_character_tick),
    (EXE_BASE + 0x9ceb0, jill_character_tick),
    (EXE_BASE + 0xa1410, hsien_ko_character_tick),
    (EXE_BASE + 0xa66e0, magneto_character_tick),
    (EXE_BASE + 0xa8830, maya_character_tick),
    (EXE_BASE + 0xaaeb0, modok_character_tick),
    (EXE_BASE + 0xac6e0, morrigan_character_tick),
    (EXE_BASE + 0xae4a0, morrigan_sh_character_tick),
    (EXE_BASE + 0xb5010, nemesis_character_tick),
    (EXE_BASE + 0xb8a00, nova_character_tick),
    (EXE_BASE + 0xbbba0, phoenix_character_tick),
    (EXE_BASE + 0xbe8f0, firebrand_character_tick),
    (EXE_BASE + 0xc84f0, ryu_character_tick),
    (EXE_BASE + 0xca240, sentinel_character_tick),
    (EXE_BASE + 0xcdc90, shuma_gorath_character_tick),
    (EXE_BASE + 0xd4b30, spider_man_character_tick),
    (EXE_BASE + 0xd76d0, storm_character_tick),
    (EXE_BASE + 0xdabd0, super_skrull_character_tick),
    (EXE_BASE + 0xdd840, taskmaster_character_tick),
    (EXE_BASE + 0xdff60, thor_character_tick),
    (EXE_BASE + 0xebbb0, viewtiful_joe_character_tick),
    (EXE_BASE + 0xedf80, wesker_character_tick),
    (EXE_BASE + 0xf0d70, wolverine_character_tick),
    (EXE_BASE + 0xf2eb0, x_23_character_tick),
    (EXE_BASE + 0xf6850, zero_character_tick),
}

pub fn hook_character_ticks() -> Result<(), Box<dyn std::error::Error>> {
    tick_hooks_macro_generated::hook_character_ticks()
}




/*
// originally at 0x1400c84f0
fn ryu_character_tick(unknown_param : *const ())
{
    let original: fn(*const ()) -> () = {
        // i'd love to have real error handling instead of just .unwrapping
        // but also i dont know how i'd even begin to do that in this environ
        // of a hooked function
        
        let hooks = HOOKS.lock().unwrap();
            
        let hook = hooks.get(&(ryu_character_tick as usize)).unwrap();
        
        let trampoline = hook.trampoline();
        
        unsafe { std::mem::transmute(trampoline) }
    };
    
    //let outer_ptr = unsafe { read_usize(EXE_BASE + CHAR_NODES_BASE) };
    
    //debug_msg(format!("outer_ptr = {}", outer_ptr));
    
    //let character_tree_ptr = unsafe { read_usize(read_usize(EXE_BASE + CHAR_NODES_BASE) + 0x58) };
    
    //debug_msg(format!("character_tree_ptr = {}", character_tree_ptr));
    
    let p1_char1_ptr = unsafe { read_usize(read_usize(read_usize(EXE_BASE + CHAR_NODES_BASE) + 0x58) + 0x8) };
    
    //debug_msg(format!("p1_char1_ptr = {}", p1_char1_ptr));
    
    let x_pos : f32 = unsafe { read_ptr_no_check(p1_char1_ptr + 0x50) };
    
    //debug_msg(format!("resulting_data = {}", resulting_data));
    
    let x_pos = x_pos + 0.9;
    
    unsafe { write_ptr(p1_char1_ptr + 0x50, x_pos) };
    
    original(unknown_param);
    
    
    // 1400fb7a is the function that reads anmchr commands
}
*/