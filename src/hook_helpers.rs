// we're going to be doing a lot of unsafe stuff so yeah
#![deny(unsafe_op_in_unsafe_fn)]

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::ffi::CString;
use std::io::Cursor;

use windows::core::s;
use windows::core::{PCSTR};
use windows::Win32::UI::WindowsAndMessaging;

use retour::RawDetour;

/// base address of umvc3.exe
pub const EXE_BASE : usize = 0x140000000;

/// we keep our hooked functions around in this so we can call the originals and so on
pub static HOOKS : LazyLock<Mutex<HashMap<usize, RawDetour>>> = LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

pub fn make_hook(replaced_ptr : usize, replacer_ptr : usize) -> Result<(), Box<dyn std::error::Error>>
{
    let hook = unsafe { RawDetour::new( replaced_ptr as *const (), replacer_ptr as *const ())? };
    
    unsafe { hook.enable()? };
    
    let mut hooks = HOOKS.lock()?;
    
    hooks.insert(replacer_ptr as usize, hook);
    
    Ok(())
}

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

pub fn handle_error(_error : Box<dyn std::error::Error>)
{
    //let msg = format!("mag_patch error:\n{}",error.to_string());
    unsafe {
        WindowsAndMessaging::MessageBoxA(None,
            s!("mag_patch error happened sorry for no more info"), //msg.into(),
            s!("mag_patch error"),
            Default::default()
        );
    };
}
