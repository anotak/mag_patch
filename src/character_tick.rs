//! the base game's char specific update functions are all in here, we're just hooking all of them to override all of them

#![deny(unsafe_op_in_unsafe_fn)]

use crate::game_data::*;
use crate::storage;
use std::sync::{LazyLock, Mutex};


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
                    }
                },
                _ => (),
            }
        },
        RestartState::JustRestarted => {
            if timer > RESTART_TIME {
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


pub type TickFn = unsafe extern "win64" fn(*const ());

macro_rules! character_ticks {
    
    {
        $( ( $addr:expr, $name:ident ) ),+
        $(,)*
    } => {
        pub mod tick_hooks_macro_generated {
            use crate::hook_helpers::*;
            type TickFn = crate::character_tick::TickFn;
            
            pub fn hook_character_ticks() -> Result<(), Box<dyn std::error::Error>> {
                $(
                    TickFn::make_hook($addr, $name)?;
                )+
                
                Ok(())
            }
            
            $(
                extern "win64" fn $name(owner : *const ())
                {
                    crate::character_tick::generic_character_tick(crate::game_data::Char::new(owner as usize));
                    
                    let hook = TickFn::get_original_from_original_addr($addr);
                    
                    unsafe { hook.call(owner) };
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
    (EXE_BASE + 0x74470, deadpool_character_tick),
    (EXE_BASE + 0x77670, dormammu_character_tick),
    (EXE_BASE + 0x79eb0, doctor_doom_character_tick),
    (EXE_BASE + 0x7ad00, dr_strange_character_tick),
    (EXE_BASE + 0x7d990, dr_strange_sh_character_tick),
    (EXE_BASE + 0x7F870, felicia_character_tick),
    
    (EXE_BASE + 0x804D0, felicia_c_or_f_character_tick),
    (EXE_BASE + 0x81310, felicia_f_or_c_character_tick),
    (EXE_BASE + 0x835E0, frank_west_character_tick),
    (EXE_BASE + 0x86F90, galactus_character_tick),
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
    (EXE_BASE + 0xBFD10, firebrand_sh_character_tick),
    
    (EXE_BASE + 0xC4C10, rocket_raccoon_character_tick),
    (EXE_BASE + 0xc84f0, ryu_character_tick),
    (EXE_BASE + 0xca240, sentinel_character_tick),
    (EXE_BASE + 0xCD200, she_hulk_character_tick),
    (EXE_BASE + 0xcdc90, shuma_gorath_character_tick),
    (EXE_BASE + 0xD1A00, spencer_character_tick),
    
    (EXE_BASE + 0xd4b30, spider_man_character_tick),
    (EXE_BASE + 0xd76d0, storm_character_tick),
    (EXE_BASE + 0xdabd0, super_skrull_character_tick),
    (EXE_BASE + 0xdd840, taskmaster_character_tick),
    (EXE_BASE + 0xdff60, thor_character_tick),
    
    (EXE_BASE + 0xE52D0, tron_bonne_character_tick),
    (EXE_BASE + 0xE81B0, vergil_joe_character_tick),
    (EXE_BASE + 0xebbb0, viewtiful_joe_character_tick),
    (EXE_BASE + 0xedf80, wesker_character_tick),
    
    (EXE_BASE + 0xf0d70, wolverine_character_tick),
    (EXE_BASE + 0xf2eb0, x_23_character_tick),
    (EXE_BASE + 0xf6850, zero_character_tick),
    (EXE_BASE + 0xf9e30, zero_sh_character_tick),
    (EXE_BASE + 0xfac60, zombie_character_tick),
}

pub fn hook_character_ticks() -> Result<(), Box<dyn std::error::Error>> {
    tick_hooks_macro_generated::hook_character_ticks()
}


