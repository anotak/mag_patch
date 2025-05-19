

use windows::core::s;
use windows::core::{PCSTR};
use windows::Win32::UI::WindowsAndMessaging;
use std::ffi::CString;

#[derive(Debug)]
pub struct MagError {
    msg : String,
}

impl std::error::Error for MagError {
}

impl std::fmt::Display for MagError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

pub fn panic_msg<S: Into<String>>(msg : S)
{
    let e = MagError {
        msg : msg.into(),
    };
    
    panic(Box::new(e));
}

pub fn panic(error : Box<dyn std::error::Error>)
{
    let backtrace = std::backtrace::Backtrace::force_capture();
    
    let msg = format!("mag_patch error:\n{}\n{}",error,backtrace);
    let msg = CString::new(msg).unwrap();
    
    unsafe {
        WindowsAndMessaging::MessageBoxA(None,
            PCSTR(msg.as_ptr()  as *const u8),
            s!("mag_patch error"),
            Default::default()
        );
    };
    
    panic!("{}", error.to_string());
}
