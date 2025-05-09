// we're going to be doing a lot of unsafe stuff so yeah
#![deny(unsafe_op_in_unsafe_fn)]

use crate::hook_helpers::*;
use std::fmt;

const CHAR_NODES_BASE : usize = 0xD44A70;
const TEAMS_BASE : usize = 0xD47E68;

/// We set up getters and setters for basic offsetted values inside structs like so. Setting them up this way reduces code duplication / chance mistakes.
macro_rules! offset_getter_and_setter {
    ($getter:ident, $setter:ident, $ty:ty, $offset:expr) => {
        #[allow(dead_code)]
        pub fn $getter(&self) -> $ty
        {
            let offset = ($offset) as usize;
            unsafe { read_ptr_no_check::<$ty>(self.ptr + offset) }
        }
        
        #[allow(dead_code)]
        pub fn $setter(&self, value : $ty)
        {
            let offset = ($offset) as usize;
            unsafe { write_ptr::<$ty>(self.ptr + offset, value) }
        }
    }
}

/// We set up getters and setters for basic offsetted values inside structs like so. Setting them up this way reduces code duplication / chance mistakes.
macro_rules! offset_getter_and_setter_flag {
    ($getter:ident, $setter:ident, $ty:ty, $offset:expr, $flag:expr) => {
        #[allow(dead_code)]
        pub fn $getter(&self) -> bool
        {
            let offset = ($offset) as usize;
            let value = unsafe { read_ptr_no_check::<$ty>(self.ptr + offset) };
            
            if (value & const { $flag }) == const { $flag } {
                true
            } else {
                false
            }
        }
        
        #[allow(dead_code)]
        pub fn $setter(&self, new_value : bool)
        {
            let offset = ($offset) as usize;
            
            let value = unsafe { read_ptr_no_check::<$ty>(self.ptr + offset) };
            let value = if new_value {
                value | const { $flag }
            } else {
                value & const { !$flag }
            };
            
            unsafe { write_ptr::<$ty>(self.ptr + offset, value) }
        }
    }
}

pub fn get_p1_char1_ptr() -> usize
{
    unsafe {
        read_usize(read_usize(read_usize(EXE_BASE + CHAR_NODES_BASE) + 0x58) + 0x8)
    }
}

pub fn get_p1_char2_ptr() -> usize
{
    unsafe {
        read_usize(read_usize(read_usize(read_usize(EXE_BASE + CHAR_NODES_BASE) + 0x58) + 0x10) + 0x8)
    }
}

pub fn get_p1_char3_ptr() -> usize
{
    unsafe {
        read_usize(read_usize(read_usize(read_usize(read_usize(EXE_BASE + CHAR_NODES_BASE) + 0x58) + 0x10) + 0x8) + 0x18)
    }
}

pub fn get_p2_char1_ptr() -> usize
{
    unsafe {
        read_usize(read_usize(read_usize(EXE_BASE + CHAR_NODES_BASE) + 0x328) + 0x8)
    }
}

pub fn get_p2_char2_ptr() -> usize
{
    unsafe {
        read_usize(read_usize(read_usize(read_usize(EXE_BASE + CHAR_NODES_BASE) + 0x328) + 0x10) + 0x8)
    }
}

pub fn get_p2_char3_ptr() -> usize
{
    unsafe {
        read_usize(read_usize(read_usize(read_usize(read_usize(EXE_BASE + CHAR_NODES_BASE) + 0x328) + 0x10) + 0x8) + 0x18)
    }
}

pub fn get_p1_point_char_ptr() -> usize
{
    unsafe {
        read_usize(read_usize(EXE_BASE + TEAMS_BASE) + 0x350 + 0x48)
    }
}

pub fn get_p2_point_char_ptr() -> usize
{
    unsafe {
        read_usize(read_usize(EXE_BASE + TEAMS_BASE) + 0x610 + 0x48)
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Team
{
    Player1,
    Player2,
    // this value is just in case our process for identifying the team fails
    // we need this instead of defaulting to player1 or player2 in order to
    // prevent errors from benefiting a certain player slot
    Unknown
}

impl Team {
    pub fn opposite(&self) -> Team
    {
        match *self {
            Team::Player1 => Team::Player2,
            Team::Player2 => Team::Player1,
            Team::Unknown => Team::Unknown,
        }
    }
    
    pub fn player(&self) -> Option<Player>
    {
        match *self {
            Team::Player1 => Some(Player::player1()),
            Team::Player2 => Some(Player::player2()),
            Team::Unknown => None,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Facing
{
    Right,
    Left
}

#[derive(PartialEq, Eq, Debug)]
/// at address Char+0x06EC, might be a flags
pub enum HitstunFlagA
{
    /// is in hitstun, but not knocked down. 0x04
    HitstunAirStandCrouch,
    /// is knocked down or not in hitstun
    KnockdownOrNonHitstun
}

#[derive(PartialEq, Eq, Debug)]
#[expect(dead_code)]
/// at address Char+0x14F6
/// this is unreliable and persists across training mode resets and when snapped out so i dont recommend using it
pub enum HitstunB
{
    /// is in hitstun, but not knocked down. 0x02. after being snapped out persists  till tagging in or assist call.
    Hitstun,
    /// is not in hitstun. 0x00 or maybe 0x06 or 0x08 when getting up
    NonHitstun,
}



#[derive(PartialEq, Eq, Debug, Clone)]
#[repr(C)]
pub struct Char { ptr : usize }

impl fmt::Display for Char {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "char({:#X})", self.ptr)
    }
}

/// minimum health as enforced by the game, apparently
pub const MIN_HEALTH : f32 = 2.0;

impl Char {
    pub fn new(ptr : usize) -> Self {
        Self { ptr }
    }
    
    pub fn get_p1_point() -> Self {
        Self {
            ptr : get_p1_point_char_ptr()
        }
    }
    
    pub fn get_p2_point() -> Self {
        Self {
            ptr : get_p2_point_char_ptr()
        }
    }
    
    pub fn identify_team(&self) -> Team
    {
        // FIXME - traverse the player character / team nodes properly
        if self.ptr == get_p1_char1_ptr() {
            Team::Player1
        } else if self.ptr == get_p1_char2_ptr() {
            Team::Player1
        } else if self.ptr == get_p1_char3_ptr() {
            Team::Player1
        } else if self.ptr == get_p2_char1_ptr() {
            Team::Player2
        } else if self.ptr == get_p2_char2_ptr() {
            Team::Player2
        } else if self.ptr == get_p2_char3_ptr() {
            Team::Player2
        } else {
            Team::Unknown
        }
    }
    
    
    pub fn player(&self) -> Option<Player>
    {
        self.identify_team().player()
    }
    
    pub fn if_valid<F, T>(addr : usize, default : T, function : F) -> T
        where F : FnOnce(Char) -> T
    {
        let character = Self { ptr : addr };
        
        if character.identify_team() == Team::Unknown {
            default
        } else {
            function(character)
        }
    }
    
    pub fn get_opponent_point_char(&self) -> Option<Char>
    {
        let op_team = self.identify_team().opposite();
        
        match op_team {
            Team::Player1 => Some(Char::get_p1_point()),
            Team::Player2 => Some(Char::get_p2_point()),
            Team::Unknown => {
                None
            },
        }
    }
    
    pub fn get_ptr(&self) -> usize {
        self.ptr
    }
    
    pub fn get_facing(&self) -> Facing
    {
        let read_value = unsafe { read_ptr_no_check::<u8>(self.ptr + 0x14FA) } & 0x20 == 0x20;
        
        if read_value
        {
            Facing::Left
        } else {
            Facing::Right
        }
    }
    
    
    pub fn get_hitstun_non_knockdown(&self) -> HitstunFlagA
    {
        let read_value = unsafe { read_ptr_no_check::<u8>(self.ptr + 0x06EC) } & 0x04 == 0x04;
        
        if read_value
        {
            HitstunFlagA::KnockdownOrNonHitstun
        } else {
            HitstunFlagA::HitstunAirStandCrouch
        }
    }
    
    pub fn set_health(&mut self, value : f32)
    {
        if !value.is_finite()
        {
            return;
        }
        
        if self.get_health() <= 0.1 {
            return;
        }
        
        // FIXME - clamp this appropriately for the player's max health
        if value < MIN_HEALTH {
            self.set_health_raw(MIN_HEALTH);
        } else {
            self.set_health_raw(value);
        }
    }
    
    offset_getter_and_setter!(get_x_pos, set_x_pos, f32, 0x50);
    offset_getter_and_setter!(get_y_pos, set_y_pos, f32, 0x54);
    offset_getter_and_setter!(get_health, set_health_raw, f32, 0x1550);
    offset_getter_and_setter!(get_condition_register, set_condition_register, i32, 0x13C4);
    offset_getter_and_setter!(get_character_combo_counter, set_character_combo_counter, i32, 0x4164);
    offset_getter_and_setter!(get_special_air_action_counter, set_special_air_action_counter, i32, 0x41a0);
    offset_getter_and_setter!(get_normal_air_action_counter, set_normal_air_action_counter, i32, 0x4190);
    
    
    offset_getter_and_setter_flag!(get_flying_screen_install, set_flying_screen_install, u8, 0x1509, 0x04);
}

pub fn get_p1_ptr() -> usize
{
    unsafe {
        read_usize(EXE_BASE + TEAMS_BASE) + 0x350 
    }
}

pub fn get_p2_ptr() -> usize
{
    unsafe {
        read_usize(EXE_BASE + TEAMS_BASE) + 0x610 
    }
}

#[derive(PartialEq, Eq, Debug)]
#[repr(C)]
pub struct Player { ptr : usize }

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Player({:#X})", self.ptr)
    }
}

pub const METER_MAX : f32 = 50000.0;
impl Player {
    pub fn new(ptr : usize) -> Self {
        Self { ptr }
    }
    
    pub fn player1() -> Self {
        Self::new(get_p1_ptr())
    }
    
    pub fn player2() -> Self {
        Self::new(get_p2_ptr())
    }
    
    pub fn point_char(&self) -> Char {
        let ptr = unsafe { read_usize(self.ptr + 0x48) };
        
        Char::new(ptr)
    }
    
    pub fn set_meter(&mut self, value : f32)
    {
        if !value.is_finite()
        {
            self.set_meter_raw(0.0);
        }
        else if value > METER_MAX
        {
            self.set_meter_raw(METER_MAX);
        }
        else if value <= 0.0
        {
            self.set_meter_raw(0.0);
        }
        else
        {
            self.set_meter_raw(value);
        }
    }
    
    offset_getter_and_setter!(get_meter, set_meter_raw, f32, 0x78);
    offset_getter_and_setter!(get_team_combo_counter, _set_team_combo_counter_dont_use, i32, 0x90);
    // FIXME - clamp this properly
    //offset_getter_and_setter!(get_xfactor_timer, set_xfactor_timer_raw, f32, 0xC0);
}
