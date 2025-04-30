#![deny(unsafe_op_in_unsafe_fn)]
#![cfg(test)]

/// interpret a string that looks like "12abce53" to a series of hex bytes
fn to_bytes(input : &str) -> Vec<u8>
{
    let mut output : Vec<u8> = Vec::with_capacity(input.len() / 2);
    
    let mut chars = input.chars();
    
    loop {
        let (hi_nibble,lo_nibble) = match (chars.next(), chars.next())
        {
            (Some(a), Some(b)) => (a,b),
            _ => break,
        };
        
        let byte : [char; 2] = [hi_nibble, lo_nibble];
        
        let byte : String = byte.iter().collect();
        
        let byte = u8::from_str_radix(byte.as_str(), 16).unwrap();
        
        output.push(byte);
    }
    
    output
}

fn test_reload_one(to_test : &str, expected : &str)
{
    const STORAGE_KEY : usize = 1111;
    
    let mut to_test = to_bytes(to_test);
    let original = to_test.clone();
    
    let command_type_group = Some(to_test[0] as u32);
    let command = Some(to_test[4] as u32);
    
    let anmchr_command_ptr = to_test.as_mut_ptr() as usize;
    
    let reloads = crate::reload::save_anmchr_command(STORAGE_KEY, anmchr_command_ptr + 8, command_type_group, command);
    
    let expected = to_bytes(expected);
    
    for (a,b) in to_test.clone().into_iter().zip(expected.into_iter()) {
        assert_eq!(a, b);
    }
    
    reloads.restore();
    
    for (a,b) in to_test.into_iter().zip(original.into_iter()) {
        assert_eq!(a, b);
    }
}

#[test]
fn test_reload_all() {
    test_reload_one(
        "0000000021000000040000000000000003000000030000000600000003000000010000002D000000FFFFFFFF03000000",
        "0000000021000000040000000000000003000000030000000600000003000000010000002D0000000000000003000000",
    );
    
    
    test_reload_one(
        "0000000022000000050000000000000003000000030000000600000006000000030000000100000002000000FFFFFFFFFFFFFFFF03000000",
        "0000000022000000050000000000000003000000030000000600000006000000030000000100000002000000000000000000000003000000",
    );
    
}