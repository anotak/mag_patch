#![deny(unsafe_op_in_unsafe_fn)]
use crate::hook_helpers::get_cursor;
use byteorder::{ReadBytesExt};
use std::fmt;
use std::ffi::c_char;
use std::io::{Cursor, Seek};
use std::io;

/// the byte-strings the game uses for things like paths. 0 byte terminated. but also it has a fixed capacity, and can fill up that capacity with the 0 byte missing. Often the capacity is 64 bytes.
/// some other strings the game uses are Shift-JIS.
/// This struct is not owned by us, just points to the game's data.
pub struct GStr {
    /// a pointer to a `capacity`-length buffer.
    ptr : usize,
    /// maximum size of the string in bytes.
    capacity : usize,
}

pub type GChar = c_char;

impl GStr {
    /// get a pointer to `capacity` bytes from a cursor, advancing the cursor position.
    pub fn from_cursor(cursor : &mut Cursor<&'static [u8]>, capacity : usize) -> Option<Self> {
        let position = cursor.position() as usize;
        
        match cursor.seek(io::SeekFrom::Current(capacity as i64)) {
            Ok(_) => (),
            Err(_) => { return None; },
        }
        
        let slice = cursor.get_ref();
        
        let base_ptr = slice.as_ptr() as usize;
        let ptr = base_ptr + position;
        
        Some(Self {
            ptr,
            capacity,
        })
    }
    
    pub fn from_ptr(ptr : usize, capacity : usize) -> Self
    {
        Self {
            ptr,
            capacity,
        }
    }
    
    pub fn eq_ignore_ascii_case(&self, other: &Self) -> bool {
        if self.ptr == other.ptr && self.capacity == other.capacity {
            true
        } else {
            let mut is_match = true;
            
            for (lhs, rhs) in self.into_iter().zip(other.into_iter()) {
                let lhs = lhs as u8;
                let rhs = rhs as u8;
                
                if !lhs.eq_ignore_ascii_case(&rhs) {
                    is_match = false;
                    break;
                } else if lhs == 0x00 {
                    break;
                }
            }
            
            is_match
        }
    }
}

impl fmt::Display for GStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.into_iter() {
            write!(f, "{}", c)?;
        }
        
        Ok(())
    }
}


pub struct GStrIter {
    capacity : u64,
    cursor : Cursor<&'static [u8]>
}

impl Iterator for GStrIter {
    type Item = GChar;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.position() >= self.capacity {
            return None;
        }
        
        let output = self.cursor.read_u8();
        
        match output {
            Ok(0x00) => None,
            Ok(output) => Some(output as GChar),
            Err(_) => None,
        }
    }
}

impl IntoIterator for &GStr {
    type Item = GChar;
    type IntoIter = GStrIter;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            capacity : self.capacity as u64,
            cursor : unsafe { get_cursor(self.ptr, self.capacity) },
        }
    }
}