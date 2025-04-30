#![deny(unsafe_op_in_unsafe_fn)]
#![cfg(test)]

/// interpret a string that looks like "12abce53" to a series of hex bytes
fn to_bytes(input : &str) -> Vec<u8>
{
    let mut output : Vec<u8> = Vec::with_capacity(input.len() / 2);
    
    let mut chars = input.chars();
    
    loop {
        let hi_nibble = match chars.next()
        {
            Some(c) => c,
            _ => break,
        };
        
        if hi_nibble.is_whitespace() {
            continue;
        }
        
        let lo_nibble = match chars.next()
        {
            Some(c) => c,
            _ => break,
        };
        
        if lo_nibble.is_whitespace() {
            continue;
        }
        
        let byte : [char; 2] = [hi_nibble, lo_nibble];
        
        let byte : String = byte.iter().collect();
        
        let byte = u8::from_str_radix(byte.as_str(), 16).unwrap();
        
        output.push(byte);
    }
    
    output
}

fn to_hex_string(input : Vec<u8>) -> String
{
    let mut output = String::new();
    
    for (index, value) in input.iter().enumerate() {
        if index != 0 && index % 4 == 0 {
            output.push_str("\n\t");
        }
        output.push_str(format!("{:02X}",value).as_str());
    }
    
    output
}

fn test_reload_one(to_test_str : &str, expected_str : &str)
{
    const STORAGE_KEY : usize = 1111;
    
    let mut to_test = to_bytes(to_test_str);
    let original = to_test.clone();
    
    let command_type_group = Some(to_test[0] as u32);
    let command = Some(to_test[4] as u32);
    
    let anmchr_command_ptr = to_test.as_mut_ptr() as usize;
    
    let reloads = crate::reload::save_anmchr_command(STORAGE_KEY, anmchr_command_ptr + 8, command_type_group, command);
    
    let expected = to_bytes(expected_str);
    
    for (index, (a,b)) in to_test.clone().into_iter().zip(expected.into_iter()).enumerate() {
        assert_eq!(a, b, "in test of \n\t{}\n\tvs\n\t{}\n\tvs\n\t{},\n\nto_test {:#X} != {:#X} expected at index {} bytes (div by 4 = {})", to_test_str, expected_str, to_hex_string(to_test), a, b, index, index/4);
    }
    
    reloads.restore();
    
    for (index, (a,b)) in to_test.clone().into_iter().zip(original.into_iter()).enumerate() {
        assert_eq!(a, b, "in test of \n\t{}\n\tvs\n\t{},\n\nto_test {:#X} != {:#X} original at index {} bytes (div by 4 = {})", to_test_str, expected_str, a, b, index, index/4);
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
    
    
    test_reload_one(
        "00000000230000000600000000000000030000000300000006000000060000000E0000000300000001000000A3000000FFFFFFFFFFFFFFFF0000000003000000",
        "00000000230000000600000000000000030000000300000006000000060000000E0000000300000001000000A300000000000000000000000000000003000000",
    );
    
    test_reload_one(
        "000000002400000006000000000000000F00000003000000050000000600000006000000030000000300000000000000E7000000ffffffffffffffff03000000",
        "000000002400000006000000000000000F00000003000000050000000600000006000000030000000300000000000000E7000000000000000000000003000000",
    );
    
    test_reload_one(
        "00000000260000000200000000000000060000000F000000ffffffff00000000",
        "00000000260000000200000000000000060000000F0000000000000000000000",
    );
    
    
    test_reload_one(
        "00000000260000000200000000000000060000000F000000ffffffff00000000",
        "00000000260000000200000000000000060000000F0000000000000000000000",
    );
    
    test_reload_one(
        "00000000
        31000000
        08000000
        00000000
        0F000000
        06000000
        0F000000
        05000000
        05000000
        0C000000
        05000000
        0C000000
        01000000
        ffffffff
        05000000
        00000000
        01000000
        ffffffff
        ffffffff
        ffffffff
        01000000
        ffffffff
        ffffffff
        ffffffff",
        "00000000
        31000000
        08000000
        00000000
        0F000000
        06000000
        0F000000
        05000000
        05000000
        0C000000
        05000000
        0C000000
        01000000
        00000000
        05000000
        00000000
        01000000
        00000000
        00000000
        00000000
        01000000
        00000000
        00000000
        00000000",
    );
    
    test_reload_one(
        "00000000
        32000000
        06000000
        00000000
        0F000000
        06000000
        0F000000
        05000000
        05000000
        0C000000
        01000000
        ffffffff
        05000000
        00000000
        FFFFFFFF
        ffffffff
        ffffffff
        ffffffff",
        "00000000
        32000000
        06000000
        00000000
        0F000000
        06000000
        0F000000
        05000000
        05000000
        0C000000
        01000000
        00000000
        05000000
        00000000
        FFFFFFFF
        00000000
        00000000
        00000000",
    );
    
    test_reload_one(
        "00000000
        37000000
        08000000
        00000000
        0F000000
        06000000
        0F000000
        05000000
        05000000
        0C000000
        05000000
        0C000000
        01000000
        ffffffff
        06000000
        00000000
        01000000
        ffffffff
        ffffffff
        ffffffff
        01000000
        ffffffff
        ffffffff
        ffffffff",
        "00000000
        37000000
        08000000
        00000000
        0F000000
        06000000
        0F000000
        05000000
        05000000
        0C000000
        05000000
        0C000000
        01000000
        00000000
        06000000
        00000000
        01000000
        00000000
        00000000
        00000000
        01000000
        00000000
        00000000
        00000000",
    );
    
    test_reload_one(
        "00000000
        38000000
        06000000
        00000000
        0F000000
        06000000
        0F000000
        05000000
        05000000
        0C000000
        01000000
        ffffffff
        05000000
        00000000
        FFFFFFFF
        ffffffff
        ffffffff
        ffffffff",
        "00000000
        38000000
        06000000
        00000000
        0F000000
        06000000
        0F000000
        05000000
        05000000
        0C000000
        01000000
        00000000
        05000000
        00000000
        FFFFFFFF
        00000000
        00000000
        00000000",
    );
    test_reload_one(
        "00000000
        45000000
        01000000
        00000000
        0C000000
        ffffffff
        ffffffff
        ffffffff",
        "00000000
        45000000
        01000000
        00000000
        0C000000
        00000000
        00000000
        00000000",
    );
    test_reload_one(
        "01000000
        0C000000
        03000000
        00000000
        05000000
        05000000
        06000000
        00000000
        28000000
        ffffffff",
        "01000000
        0C000000
        03000000
        00000000
        05000000
        05000000
        06000000
        00000000
        28000000
        00000000",
    );
    test_reload_one(
        "01000000
        79000000
        01000000
        00000000
        06000000
        ffffffff",
        "01000000
        79000000
        01000000
        00000000
        06000000
        00000000",
    );
    test_reload_one(
        "01000000
        7C000000
        01000000
        00000000
        06000000
        ffffffff",
        "01000000
        7C000000
        01000000
        00000000
        06000000
        00000000",
    );
    test_reload_one(
        "01000000
        7E000000
        07000000
        00000000
        05000000
        05000000
        05000000
        0C000000
        05000000
        0C000000
        0E000000
        07000000
        BE000000
        00000000
        ffffffff
        ffffffff
        ffffffff
        FFFFFFFF
        ffffffff
        ffffffff
        ffffffff
        00000000",
        "01000000
        7E000000
        07000000
        00000000
        05000000
        05000000
        05000000
        0C000000
        05000000
        0C000000
        0E000000
        07000000
        BE000000
        00000000
        00000000
        00000000
        00000000
        FFFFFFFF
        00000000
        00000000
        00000000
        00000000",
    );
    test_reload_one(
        "01000000
        7F000000
        08000000
        00000000
        10000000
        05000000
        05000000
        05000000
        0C000000
        05000000
        0C000000
        0E000000
        75536875
        6D610000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        FFFFFFFF
        D0000000
        FFFFFFFF
        ffffffff
        ffffffff
        ffffffff
        FFFFFFFF
        ffffffff
        ffffffff
        ffffffff
        00000000",
        "01000000
        7F000000
        08000000
        00000000
        10000000
        05000000
        05000000
        05000000
        0C000000
        05000000
        0C000000
        0E000000
        75536875
        6D610000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        FFFFFFFF
        D0000000
        FFFFFFFF
        00000000
        00000000
        00000000
        FFFFFFFF
        00000000
        00000000
        00000000
        00000000",
    );
    test_reload_one(
        "01000000
        89000000
        01000000
        00000000
        06000000
        ffffffff",
        "01000000
        89000000
        01000000
        00000000
        06000000
        00000000",
    );
    test_reload_one(
        "01000000
        8A000000
        01000000
        00000000
        06000000
        ffffffff",
        "01000000
        8A000000
        01000000
        00000000
        06000000
        00000000",
    );
    
}