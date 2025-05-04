// we're going to be doing a lot of unsafe stuff so yeah
#![deny(unsafe_op_in_unsafe_fn)]

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::ffi::CString;
use std::io::Cursor;

use windows::core::s;
use windows::core::{PCSTR};
use windows::Win32::UI::WindowsAndMessaging;

use retour::{RawDetour,GenericDetour};

use crate::error::*;

/// base address of umvc3.exe
pub const EXE_BASE : usize = 0x140000000;

/// we keep our hooked functions around in this so we can call the originals and so on
pub static HOOKS : LazyLock<Mutex<HashMap<usize, RawDetour>>> = LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

pub fn make_hook(replaced_ptr : usize, replacer_ptr : usize) -> Result<(), Box<dyn std::error::Error>>
{
    let key = replacer_ptr;
    
    let mut hooks = HOOKS.lock()?;
    
    if hooks.contains_key(&key)
    {
        panic_msg(format!("major mag_patch error: to hook address {:#X} with duplicate ptr", key));
    }
    
    let hook = unsafe { RawDetour::new( replaced_ptr as *const (), replacer_ptr as *const ())? };
    
    unsafe { hook.enable()? };
    
    hooks.insert(key as usize, hook);
    
    Ok(())
}

pub trait Hook {
    fn make_hook(replaced_ptr : usize, replacer : Self) -> Result<(), Box<dyn std::error::Error>> where Self: retour::Function;
    fn with_original<F, T>(original : usize, func : F) -> T
        where Self: retour::Function,
                F : FnOnce(&GenericDetour<Self>) -> T;
}

macro_rules! typed_hooks {
    ($hooked_func_type:ty, $statics_mod:ident) => {
        // type associated statics are not allowed otherwise we'd put this in the impl
        mod $statics_mod {
            use std::collections::HashMap;
            use std::sync::{LazyLock, Mutex};
            use retour::{GenericDetour};
            
            pub static HOOKS : LazyLock<Mutex<HashMap<usize, GenericDetour<$hooked_func_type>>>> = LazyLock::new(|| {
                Mutex::new(HashMap::new())
            });
        }
        
        impl Hook for $hooked_func_type {
            fn make_hook(replaced_ptr : usize, replacer : Self) -> Result<(), Box<dyn std::error::Error>> where Self: retour::Function
            {
                let mut hooks = $statics_mod::HOOKS.lock()?;
                
                if hooks.contains_key(&(replaced_ptr as usize))
                {
                    let addr = replaced_ptr as usize;
                    
                    panic_msg(format!("major mag_patch error: to hook address {:#X} with duplicate ptr", addr));
                }
                
                let replaced_ptr = unsafe { std::mem::transmute(replaced_ptr) };
                
                let hook = unsafe { GenericDetour::new( replaced_ptr, replacer)? };
                
                unsafe { hook.enable()? };
                
                hooks.insert(replaced_ptr as usize, hook);
                
                Ok(())
            }
            
            fn with_original<F,T>(original : usize, function : F) -> T
                where Self: retour::Function,
                F : FnOnce(&GenericDetour<Self>) -> T
            {
                // i'd love to have real error handling instead of just .unwrapping
                // but also i dont know how i'd even begin to do that in this environ
                // of a hooked function
                
                let hooks = $statics_mod::HOOKS.lock().unwrap();
                
                let hook = hooks.get(&(original as usize)).unwrap();
                
                function(hook)
            }
        }
    }
}

typed_hooks!(crate::character_tick::TickFn, __tick_hooks);


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

pub unsafe fn read_ptr<T>(addr : usize) -> Option<T>
    where T : Copy
{
    // check nullness among other things
    if addr < 0x00010000 || addr >= 0x80000000 {
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
