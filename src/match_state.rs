//! Stuff like the timer that's not tied to a specific player

#![deny(unsafe_op_in_unsafe_fn)]

use crate::hook_helpers::*;

use num_derive::FromPrimitive;

pub const MATCH_ACTION_OFFSET : usize = 0xd47e68;

#[derive(Copy,Clone,PartialEq, Eq, Debug, FromPrimitive)]
#[repr(i32)]
// at address [EXE_BASE + 0xd47e68] + 0xD8
pub enum MatchState {
    /// Char select other menus etc
    NotPlaying = 0,
    /// Intro hasnt started playing yet, but done with loading screen
    PreIntro = 1,
    /// Intro sequence
    Intro = 2,
    /// Situation where players can move around
    Starting = 3,
    /// After you training mode reset, the fade in
    RestartingFadeIn = 4,
    /// After "Fight!"
    Fighting = 5,
    /// Character just got last hit is screaming but hasn't fallen to deadbody state yet
    Death = 6,
    /// Dead character laying there, winner can move
    DeadBody = 7,
    /// Brief moment before outro
    PreOutro = 8,
    /// the outro is playing
    Outro = 9,
    /// I wonder if this is the versus mode victory screen, or maybe the arcade mode loss screen
    Unknown10 = 10, 
    /// Victory screen, after you win, at least in arcade mode? is this true in versus mode? I can't test without a second controller
    PostOutro = 11,
    /// When you training mode reset
    RestartingFadeOut = 12,
    
    /// This is our own representation of an erroneous state in interpreting that game's value
    Error = -1,
}

pub fn get_match_state() -> MatchState
{
    let match_state = unsafe { read_ptr_no_check(read_usize(EXE_BASE + MATCH_ACTION_OFFSET) + 0xD8) };
    let match_state = num::FromPrimitive::from_i32(match_state);
    
    if let Some(match_state) = match_state {
        match_state
    }
    else
    {
        MatchState::Error
    }
}

pub fn get_match_frame_time() -> f32
{
    unsafe { read_ptr_no_check(read_usize(EXE_BASE + MATCH_ACTION_OFFSET) + 0xE4) }
}

/// the timer internally is 180 real seconds for 99 marvel seconds so we just do some math so our modders dont have to worry about that
const GET_TIMER_RATIO : f32 = 99.0 / 180.0;
/// The timer that shows 99 at round start normally. -1 is infinite
pub fn get_match_game_time() -> f32
{
    let timer = unsafe { read_ptr_no_check(read_usize(EXE_BASE + MATCH_ACTION_OFFSET) + 0xf8) };
    
    if timer > 0.0 {
        timer * GET_TIMER_RATIO
    } else {
        // if -1 or whatever return that
        timer
    }
}

/// the timer internally is 180 real seconds for 99 marvel seconds so we just do some math so our modders dont have to worry about that
const SET_TIMER_RATIO : f32 = 180.0 / 99.0;
/// Te timer that shows 99 normally. -1 is infinite
pub fn set_match_game_time(new_time : f32)
{
    if new_time.is_finite()
    {
        let new_time = if new_time > 0.0 { new_time } else { 0.0 };
        
        let old_time = get_match_game_time();
        
        if old_time > 0.0 {
            let new_timer = new_time * SET_TIMER_RATIO;
            
            let ptr = unsafe { read_usize(EXE_BASE + MATCH_ACTION_OFFSET) } + 0xf8;
            
            unsafe { write_ptr(ptr, new_timer) };
        }
    }
}

