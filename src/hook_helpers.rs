// we're going to be doing a lot of unsafe stuff so yeah
#![deny(unsafe_op_in_unsafe_fn)]

use std::sync::{Arc};
use std::ffi::CString;
use std::io::Cursor;

use windows::core::s;
use windows::core::{PCSTR};
use windows::Win32::UI::WindowsAndMessaging;

use retour::{GenericDetour};

use crate::error::*;

/// base address of umvc3.exe
pub const EXE_BASE : usize = 0x140000000;

pub trait Hook {
    fn make_hook(replaced_ptr : usize, replacer : Self) -> Result<(), Box<dyn std::error::Error>> where Self: retour::Function;
    
    /// used in cases where the optimizer might replace our many replacer functions with one function (character ticks)
    fn get_original_from_original_addr(original : usize)
        -> Arc<GenericDetour<Self>>
        where Self: retour::Function;
    
    fn get_original(replacer : Self)
        -> Arc<GenericDetour<Self>>
        where Self: retour::Function;
}

macro_rules! typed_hooks {
    ($hooked_func_type:ty, $statics_mod:ident) => {
        // type associated statics are not allowed otherwise we'd put this in the impl
        mod $statics_mod {
            use std::collections::HashMap;
            use std::sync::{LazyLock, Mutex, Arc};
            use retour::{GenericDetour};
            
            pub static HOOKS : LazyLock<Mutex<HashMap<usize, Arc<GenericDetour<$hooked_func_type>>>>> = LazyLock::new(|| {
                Mutex::new(HashMap::new())
            });
            
            // if you have the address of the replace get the address of the replaced from here
            pub static HOOKS_REPLACER : LazyLock<Mutex<HashMap<usize, usize>>> = LazyLock::new(|| {
                Mutex::new(HashMap::new())
            });
        }
        
        impl Hook for $hooked_func_type {
            fn make_hook(replaced_ptr : usize, replacer : Self) -> Result<(), Box<dyn std::error::Error>> where Self: retour::Function
            {
                {
                    let mut hooks_replacer = $statics_mod::HOOKS_REPLACER.lock()?;
                    let replacer_ptr = replacer as usize;
                    
                    hooks_replacer.insert(replacer_ptr, replaced_ptr);
                }
                
                {
                    let mut hooks = $statics_mod::HOOKS.lock()?;
                    
                    if hooks.contains_key(&(replaced_ptr as usize))
                    {
                        let addr = replaced_ptr as usize;
                        
                        panic_msg(format!("major mag_patch error: to hook address {:#X} with duplicate ptr", addr));
                    }
                    
                    let replaced = unsafe { std::mem::transmute::<usize, $hooked_func_type>(replaced_ptr) };
                    
                    let hook = unsafe { GenericDetour::new( replaced, replacer)? };
                    
                    unsafe { hook.enable()? };
                    
                    hooks.insert(replaced_ptr, Arc::new(hook));
                }
                
                Ok(())
            }
            
            fn get_original_from_original_addr(original : usize)
                -> Arc<GenericDetour<Self>>
                where Self: retour::Function
            {
                let hooks = $statics_mod::HOOKS.lock().unwrap();
                
                let hook = hooks.get(&(original as usize)).unwrap();
                
                hook.clone()
            }
            
            fn get_original(replacer : Self)
                -> Arc<GenericDetour<Self>>
                where Self: retour::Function
            {
                let ptr = replacer as usize;
                
                let original = {
                    let hooks_replacer = $statics_mod::HOOKS_REPLACER.lock().unwrap();
                    
                    hooks_replacer.get(&ptr).unwrap().clone()
                };
                
                let hooks = $statics_mod::HOOKS.lock().unwrap();
                
                let hook = hooks.get(&original).unwrap();
                
                hook.clone()
            }
        }
    }
}

typed_hooks!(crate::character_tick::TickFn, __tick_hooks);
typed_hooks!(crate::ExecuteAnmChrCommandFn, __execute_anmchr_command);


/// call in your hooked functions to get the original function
#[macro_export]
macro_rules! get_original_func {
    ($original:ident) => {
        {
            // i'd love to have real error handling instead of just .unwrapping
            // but also i dont know how i'd even begin to do that in this environ
            // of a hooked function
            
            let hooks = HOOKS.lock().unwrap();
            
            let hook = hooks.get(&($original as usize)).unwrap();
            
            let trampoline = hook.trampoline();
            
            unsafe { std::mem::transmute(trampoline) }
        }
    }
}

pub unsafe fn get_cursor(addr : usize, len : usize) -> Cursor<&'static [u8]>
{
    let ptr : *const u8 = addr as *const u8;
    
    Cursor::new(unsafe {
        std::slice::from_raw_parts(ptr, len)
    })
}

pub unsafe fn get_mut_cursor(addr : usize, len : usize) -> Cursor<&'static mut [u8]>
{
    let ptr : *mut u8 = addr as *mut u8;
    
    Cursor::new(unsafe {
        std::slice::from_raw_parts_mut(ptr, len)
    })
}

pub unsafe fn read_ptr_no_check<T>(addr : usize) -> T
    where T : Copy
{
    let ptr : *const T = addr as *const T;
    
    unsafe { std::ptr::read_unaligned(ptr) }
}

const ADDR_MIN : usize = 0x00010000;

#[cfg(not(test))]
const ADDR_MAX : usize = 0x80000000;

// while testing we're not in the address space of the game's exe so the norm ADDR_MAX doesnt really work right
#[cfg(test)]
const ADDR_MAX : usize = 0xFFFFFFFFFFF;

pub unsafe fn read_ptr<T>(addr : usize) -> Option<T>
    where T : Copy
{
    // check nullness among other things
    
    if addr < ADDR_MIN || addr >= ADDR_MAX {
        None
    } else {
        Some(unsafe { read_ptr_no_check(addr) })
    }
}

pub unsafe fn write_ptr<T>(addr : usize, value : T)
    where T : Copy
{
    let ptr : *mut T = addr as *mut T;
    
    unsafe { std::ptr::write_unaligned(ptr, value) }
}

pub unsafe fn read_usize(addr : usize) -> usize
{
    let ptr : *const usize = addr as *const usize;
    
    unsafe { std::ptr::read_unaligned(ptr) }
}

pub fn debug_msg<S: Into<String>>(msg : S)
{
    let msg = CString::new(msg.into()).unwrap();
    
    unsafe {
        WindowsAndMessaging::MessageBoxA::<PCSTR,PCSTR>(
            None,
            PCSTR(msg.as_ptr()  as *const u8),
            s!("mag_patch debug message"),
            Default::default()
        );
    };
}
