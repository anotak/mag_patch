
// we're going to be doing a lot of unsafe stuff so yeah
#![deny(unsafe_op_in_unsafe_fn)]

mod hook_helpers;
mod game_data;
mod anmchr_commands;
mod character_tick;
mod character_extensions;
mod match_state;
mod storage;
mod unary_operators;
mod binary_operators;
mod var_rw;
mod math;

use windows::Win32::System::SystemServices;
use windows::Win32::Foundation::HINSTANCE;

use crate::hook_helpers::*;


#[unsafe(no_mangle)]
pub extern "system" fn DllMain(
        _dll_module: HINSTANCE,
        call_reason: u32,
        _: *mut ()
    ) -> bool
{
    let result = match call_reason {
        SystemServices::DLL_PROCESS_ATTACH => attach(),
        SystemServices::DLL_PROCESS_DETACH => Ok(()),
        _ => Ok(())
    };
    
    match result {
        Err(e) => {handle_error(e); false},
        Ok(()) => true,
    }
}

#[derive(Debug)]
pub struct MpError {
    msg : String,
}

impl std::error::Error for MpError {}

impl std::fmt::Display for MpError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

fn attach() -> Result<(), Box<dyn std::error::Error>> {
    //make_hook(EXE_BASE + 0xc84f0, ryu_character_tick as usize)?;
    
    crate::character_tick::hook_character_ticks()?;
    
    make_hook(EXE_BASE + 0xFB7A0, execute_anmchr_command as usize)?;
    
    
    debug_msg("mag_patch hooking success!\nthis is a hella beta version please dont share it");
    
    Ok(())
}

/// original is at EXE_BASE + 0xFB7A0,
fn execute_anmchr_command(executor_ptr : usize, anmchr_command_ptr : usize)
{
    use crate::game_data::Char;
    
    let original : fn(usize, usize) -> () = get_original_func!(execute_anmchr_command);
    
    /*
    let original: fn(usize, usize) -> () = {
        // i'd love to have real error handling instead of just .unwrapping
        // but also i dont know how i'd even begin to do that in this environ
        // of a hooked function
        
        let hooks = HOOKS.lock().unwrap();
            
        let hook = hooks.get(&(execute_anmchr_command as usize)).unwrap();
        
        let trampoline = hook.trampoline();
        
        unsafe { std::mem::transmute(trampoline) }
    };
    */
    
    
    let command_type_group = unsafe { read_ptr::<u32>(anmchr_command_ptr as usize) };
    
    // (game uses commands 0 through 7 inclusive)
    // 0x66 commands are ones added by anotak
    if command_type_group == Some(0x66)
    {
        let command = unsafe { read_ptr::<u32>(anmchr_command_ptr + 4) };
        if let Some(command) = command {
            let command = num::FromPrimitive::from_u32(command);
            
            if let Some(command) = command {
                // i am not certain this is the correct way to do this
                // but it seems like it is working thus far
                let exe_char_ptr = executor_ptr - 0x1348;
                let exe_char = Char::new(exe_char_ptr);
                
                crate::anmchr_commands::handle_ano_command(command, exe_char, anmchr_command_ptr + 8);
            }
        }
    }
    
    original(executor_ptr, anmchr_command_ptr);
}
