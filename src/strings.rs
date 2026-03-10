#![deny(unsafe_op_in_unsafe_fn)]
use crate::hook_helpers::{get_cursor, read_ptr};

use byteorder::{ReadBytesExt};
use std::fmt;
use std::ffi::c_char;
use std::io::{Cursor, Seek};
use std::io;

/// the byte-strings the game uses for things like paths. 0 byte terminated. but also it has a fixed capacity, and can fill up that capacity with the 0 byte missing. Often the capacity is 64 bytes.
/// some other strings the game uses are Shift-JIS.
/// This struct is not owned by us, just points to the game's data.
#[derive(Clone)]
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
    
    pub fn compare<F>(&self, other: &Self, comparator : F) -> bool
        where F : Fn(GChar, GChar) -> bool
    {
        if self.ptr == other.ptr && self.capacity == other.capacity {
            true
        } else {
            let mut is_match = true;
            let mut ended_before_capacity = false;
            
            for (lhs, rhs) in self.into_iter().zip(other.into_iter()) {
                if !comparator(lhs, rhs) {
                    is_match = false;
                    ended_before_capacity = true;
                    break;
                } else if lhs == 0x00 {
                    // equality implied by previous branch
                    ended_before_capacity = true;
                    break;
                }
            }
            
            if !ended_before_capacity {
                let self_len = self.len();
                let other_len = other.len();
                
                let ptr_to_last = if self_len > other_len {
                    self.ptr + other_len
                } else if other_len > self_len {
                    other.ptr + self_len
                } else {
                    // should be unreachable but would rather not unreachable!()
                    return is_match;
                };
                
                let last_char = unsafe { read_ptr::<u8>(ptr_to_last) };
                
                last_char.map_or(true, |c| c == 0x00)
            } else {
                is_match
            }
        }
    }
    
    pub fn eq_ignore_ascii_case(&self, other: &Self) -> bool {
        self.compare(other, |lhs, rhs| {
            let lhs = lhs as u8;
            let rhs = rhs as u8;
            
            lhs.eq_ignore_ascii_case(&rhs)
        })
    }
    
    pub fn eq_ignore_ascii_case_and_path_separators(&self, other : &Self) -> bool {
        self.compare(&other, |lhs, rhs| {
            if is_path_separator(lhs) && is_path_separator(rhs) {
                true
            } else {
                let lhs = lhs as u8;
                let rhs = rhs as u8;
            
                if lhs.eq_ignore_ascii_case(&rhs) {
                    true
                } else {
                false
                }
            }
        })
    }
    
    /// compare two paths, where self can be equal to other or it can be a sub-path at the end of other.
    /// so for example self = "b/c" would match other = "a\b\c" but not other = "a/b/c/d"
    pub fn path_suffix_compare(&self, other : &Self) -> bool {
        // this all could be faster but i think this is probably okay
        let self_slash_count = self.into_iter().filter(|&c| is_path_separator(c)).count();
        let other_slash_count = other.into_iter().filter(|&c| is_path_separator(c)).count();
        
        if other_slash_count < self_slash_count {
            false
        } else if other_slash_count == self_slash_count {
            self.eq_ignore_ascii_case_and_path_separators(other)
        } else {
            // always positive due to preconditions established above
            let slash_count_diff = other_slash_count - self_slash_count;
            
            // due to preconditions start_pos will always be in the middle of the array
            let start_pos = {
                let mut start_pos = 0;
                let mut slashes_needed = slash_count_diff;
                
                // this loop should always break
                for c in other.into_iter() {
                    // want to return the 1 _after_ the one we are looking at
                    start_pos += 1;
                    
                    if is_path_separator(c) {
                        slashes_needed -= 1;
                        
                        if slashes_needed <= 0 {
                            break;
                        }
                    }
                }
                
                start_pos
            };
            
            let other = GStr {
                ptr : other.ptr + start_pos,
                capacity : other.capacity - start_pos
            };
            
            self.eq_ignore_ascii_case_and_path_separators(&other)
        }
    }
    
    pub fn len(&self) -> usize {
        self.into_iter().count()
    }
}

fn is_path_separator(c : GChar) -> bool {
    let c = c as u8;
    
    c == 0x5c || c == 0x2f
}

impl PartialEq for GStr {
    fn eq(&self, other: &Self) -> bool {
        if self.ptr == other.ptr && self.capacity == other.capacity {
            true
        } else {
            let mut is_match = true;
            
            for (lhs, rhs) in self.into_iter().zip(other.into_iter()) {
                let lhs = lhs as u8;
                let rhs = rhs as u8;
                
                if lhs != rhs {
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

impl Eq for GStr {}

impl fmt::Display for GStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.into_iter() {
            write!(f, "{}", char::from(c as u8))?;
        }
        
        Ok(())
    }
}

impl fmt::Debug for GStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GStr")
         .field("ptr", &self.ptr)
         .field("capacity", &self.capacity)
         .field("contents", &self.to_string())
         .finish()
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