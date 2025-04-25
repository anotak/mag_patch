
#![deny(unsafe_op_in_unsafe_fn)]

//! New structs to track new character functionality

use crate::game_data::{Char, Facing};
use crate::storage;


const SUCK_MAX : f32 = 128.0;
const SUCK_EPSILON : f32 = 0.001;

pub struct SuckOpponent
{
    pub delta : f32,
    pub magnitude : f32,
}

impl SuckOpponent
{
    pub fn apply_suck(exe_char : Char, magnitude : f32, delta : f32)
    {
        let magnitude = match exe_char.get_facing() {
            Facing::Left => -magnitude,
            Facing::Right => magnitude,
        };
        
        storage::with(
            exe_char.get_ptr(),
            |store| {
                store.suck_opponent = SuckOpponent {
                    magnitude,
                    delta,
                };
            }
        );
    }
    
    /// called once per character per tick to apply physics to opponent
    pub fn handle_suck(&mut self, owner : Char)
    {
        if self.magnitude.abs() < SUCK_EPSILON {
            self.magnitude = 0.0;
            
            // early out - no need to apply motion
            return;
        }
        
        self.magnitude = 
        {
            let mut magnitude = self.magnitude;
            let delta = self.delta;
            
            if magnitude > 0.0 {
                if magnitude > SUCK_MAX {
                    magnitude = SUCK_MAX;
                }
                
                magnitude -= delta;
                
                if magnitude < SUCK_EPSILON {
                    self.magnitude = 0.0;
                    // early out - no need to apply motion
                    return;
                }
            } else {
                if magnitude < -SUCK_MAX {
                    magnitude = -SUCK_MAX;
                }
                
                magnitude += delta;
                
                if magnitude > -SUCK_EPSILON {
                    self.magnitude = 0.0;
                    // early out - no need to apply motion
                    return;
                }
            }
            
            magnitude
        };
        
        if let Some(op) = owner.get_opponent_point_char() {
            let destination_x_pos = op.get_x_pos() + self.magnitude;
            
            op.set_x_pos(destination_x_pos);
        }
    }
}
