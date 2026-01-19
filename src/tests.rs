#![deny(unsafe_op_in_unsafe_fn)]
#![cfg(test)]

use crate::hook_helpers::*;
use crate::storage;
use crate::math::near_eq;

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


const TEST_CHARACTER_STRUCT_SIZE : usize = 0x10000;
fn test_execute_anmchr_command(ptr : usize, to_test_str : &str)
{
    let mut to_test = to_bytes(to_test_str);
    
    crate::execute_anmchr_command(ptr, to_test.as_mut_ptr() as usize);
}


extern "win64" fn replaced_fake_execute_anmchr_command(_executor_ptr : usize, _anmchr_command_ptr : usize) {}

fn get_register_bool(ptr : usize, register : usize) -> bool
{
    storage::with(
        ptr - 0x1348,
        |store| {
            store.get_bool(register as u8)
        }
    )
}

fn get_register_i32(ptr : usize, register : usize) -> i32
{
    storage::with(
        ptr - 0x1348,
        |store| {
            store.get_i32_register(register as u8)
        }
    )
}

fn get_register_f32(ptr : usize, register : usize) -> f32
{
    storage::with(
        ptr - 0x1348,
        |store| {
            store.get_f32_register(register as u8)
        }
    )
}

#[test]
fn test_commands() {
    crate::ExecuteAnmChrCommandFn::make_hook(replaced_fake_execute_anmchr_command as usize, crate::execute_anmchr_command).unwrap();
    
    
    let mut char_struct = [0; TEST_CHARACTER_STRUCT_SIZE];
    let ptr = char_struct.as_mut_ptr() as usize;
    
    let ptr = ptr + 0x4000;
    
    // register[0xff] = 1.7
    test_execute_anmchr_command(
        ptr,
        "66000000
        10000000
        000000FF
        9999d93F"
        );
    
    assert_eq!(get_register_i32(ptr, 0xff), 1);
    assert!(near_eq(get_register_f32(ptr, 0xff), 1.7));
    
    // register[0x1] = register[0x1] + 1 = 1
    test_execute_anmchr_command(
        ptr,
        "66000000
        11000000
        00000000
        01000001
        01000000"
        );
    
    assert_eq!(get_register_i32(ptr, 0x01), 1);
    
    // register[0x1] = register[0x1] + 1 = 2
    test_execute_anmchr_command(
        ptr,
        "66000000
        11000000
        00000000
        01000001
        01000000"
        );
    
    assert_eq!(get_register_i32(ptr, 0x01), 2);
    
    // register[0x2] = register[0x1] operation[0x1] ccc = 1998
    // should be multiply since register 01 contains 02
    // register[0x2] = register[0x1] * ccc = 1998
    test_execute_anmchr_command(
        ptr,
        "66000000
        11000000
        01FFFFFF
        01000002
        CC0C0000"
        );
    
    assert_eq!(get_register_i32(ptr, 0x02), 0x1998);
    
    // register[0x3] = register[0x2] / register[0x1] = ccc
    test_execute_anmchr_command(
        ptr,
        "66000000
        12000000
        03000000
        02010003"
        );
    
    assert_eq!(get_register_i32(ptr, 0x03), 0xCCC);
    
    // register[0x55] = isqrt(register[0x2]) = 80
    test_execute_anmchr_command(
        ptr,
        "66000000
        13000000
        10000000
        02000055"
        );
    
    assert_eq!(get_register_i32(ptr, 0x55), 80);
    
    // register[0x12] = bitwise not(0x12345678) = 0xEDCBA987
    test_execute_anmchr_command(
        ptr,
        "66000000
        14000000
        B0000000
        00000012
        78563412"
        );
    
    assert_eq!(get_register_i32(ptr, 0x12), 0xEDCBA987u32 as i32);
    
    // register[0x43] = condition register = 0xEDCBA987
    test_execute_anmchr_command(
        ptr,
        "66000000
        15000000
        00000043
        C0000000"
        );
    
    assert_eq!(get_register_i32(ptr, 0x43), 0xEDCBA987u32 as i32);
    
    // AirGroundStateFlags = register[0x55] = 80
    test_execute_anmchr_command(
        ptr,
        "66000000
        16000000
        00000055
        F0000000"
        );
    
    // register[0x56] = AirGroundStateFlags = 80
    test_execute_anmchr_command(
        ptr,
        "66000000
        15000000
        00000056
        F0000000"
        );
    
    assert_eq!(get_register_i32(ptr, 0x56), 80 as i32);
    
    // InvincibilityFlags = 17
    test_execute_anmchr_command(
        ptr,
        "66000000
        17000000
        00000000
        f2000000
        11000000"
        );
    
     // register[0x57] = InvincibilityFlags = 17
    test_execute_anmchr_command(
        ptr,
        "66000000
        15000000
        00000057
        F2000000"
        );
    
    assert_eq!(get_register_i32(ptr, 0x57), 17 as i32);
    
     // InvincibilityFlags = InvincibilityFlags << 5 = 544
    test_execute_anmchr_command(
        ptr,
        "66000000
        19000000
        b3000000
        00000000
        F2000000
        05000000"
        );
    
    // register[0x58] = InvincibilityFlags = 544
    test_execute_anmchr_command(
        ptr,
        "66000000
        15000000
        00000058
        F2000000"
        );
    
    assert_eq!(get_register_i32(ptr, 0x58), 544 as i32);
    
    
     // CharacterComboCounter = logical not(CharacterComboCounter) = 1
    test_execute_anmchr_command(
        ptr,
        "66000000
        1a000000
        c0000000
        00000000
        32000000"
        );
    
    // register[0x59] = CharacterComboCounter = 1
    test_execute_anmchr_command(
        ptr,
        "66000000
        15000000
        00000059
        32000000"
        );
    
    assert_eq!(get_register_i32(ptr, 0x59), 1 as i32);
    
     // XPosition = 1.0
    test_execute_anmchr_command(
        ptr,
        "66000000
        17000000
        00000000
        20000000
        0000803f"
        );
    
    // register[0x9b] = XPosition = 1.0
    test_execute_anmchr_command(
        ptr,
        "66000000
        15000000
        0000009b
        20000000"
        );
    
    assert_eq!(get_register_f32(ptr, 0x9b), 1.0);
    
    // XPosition = sqrt(XPosition)
    test_execute_anmchr_command(
        ptr,
        "66000000
        1a000000
        10000000
        00000000
        20000000"
        );
    
    // register[0x9c] = XPosition = 1.0
    test_execute_anmchr_command(
        ptr,
        "66000000
        15000000
        0000009c
        20000000"
        );
    assert_eq!(get_register_f32(ptr, 0x9c), 1.0);
    
    // XPosition = 64.0
    test_execute_anmchr_command(
        ptr,
        "66000000
        17000000
        00000000
        20000000
        00008042"
        );
    
    // register[0x71] = XPosition = 64
    test_execute_anmchr_command(
        ptr,
        "66000000
        15000000
        00000071
        20000000"
        );
    assert_eq!(get_register_i32(ptr, 0x71), 64);
    
    
    // XPosition = sqrt(XPosition)
    test_execute_anmchr_command(
        ptr,
        "66000000
        1a000000
        10000000
        00000000
        20000000"
        );
    
    // register[0x71] = XPosition = 8
    test_execute_anmchr_command(
        ptr,
        "66000000
        15000000
        00000071
        20000000"
        );
    assert_eq!(get_register_i32(ptr, 0x71), 8);
    
    // boolean[0x1] = boolean[0x1] + 1 = 1
    test_execute_anmchr_command(
        ptr,
        "66000000
        11000000
        00000000
        01000501
        01000000"
        );
    
    assert_eq!(get_register_bool(ptr, 0x01), true);
    // boolean[0x1] = boolean[0x1] + 1 = 1
    test_execute_anmchr_command(
        ptr,
        "66000000
        11000000
        00000000
        01000501
        01000000"
        );
    
    assert_eq!(get_register_bool(ptr, 0x01), true);
    
    // boolean[0x1] = boolean[0x1] ^ 1 = 0
    test_execute_anmchr_command(
        ptr,
        "66000000
        11000000
        b2000000
        01000501
        01000000"
        );
    
    assert_eq!(get_register_bool(ptr, 0x01), false);
    
    // boolean[0x1] = boolean[0x1] ^ 1 = 1
    test_execute_anmchr_command(
        ptr,
        "66000000
        11000000
        b2000000
        01000501
        01000000"
        );
    
    assert_eq!(get_register_bool(ptr, 0x01), true);
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
    test_reload_one(
        "01000000
        8B000000
        03000000
        00000000
        05000000
        06000000
        05000000
        32000000
        ffffffff
        01000000",
        "01000000
        8B000000
        03000000
        00000000
        05000000
        06000000
        05000000
        32000000
        00000000
        01000000",
    );
    test_reload_one(
        "01000000
        A4000000
        01000000
        00000000
        0C000000
        ffffffff
        ffffffff
        ffffffff",
        "01000000
        A4000000
        01000000
        00000000
        0C000000
        00000000
        00000000
        00000000",
    );
    test_reload_one(
        "01000000
        A5000000
        03000000
        00000000
        06000000
        06000000
        06000000
        ffffffff
        ffffffff
        ffffffff",
        "01000000
        A5000000
        03000000
        00000000
        06000000
        06000000
        06000000
        00000000
        00000000
        00000000",
    );
    
    test_reload_one(
        "01000000
        A6000000
        02000000
        00000000
        06000000
        06000000
        ffffffff
        ffffffff",
        "01000000
        A6000000
        02000000
        00000000
        06000000
        06000000
        00000000
        00000000",
    );
    
    test_reload_one(
        "01000000
        A7000000
        01000000
        00000000
        06000000
        ffffffff",
        "01000000
        A7000000
        01000000
        00000000
        06000000
        00000000",
    );
    
    test_reload_one(
        "01000000
        A8000000
        01000000
        00000000
        06000000
        ffffffff",
        "01000000
        A8000000
        01000000
        00000000
        06000000
        00000000",
    );
    
    test_reload_one(
        "01000000
        A9000000
        01000000
        00000000
        06000000
        ffffffff",
        "01000000
        A9000000
        01000000
        00000000
        06000000
        00000000",
    );
    
    test_reload_one(
        "01000000
        AA000000
        02000000
        00000000
        06000000
        06000000
        ffffffff
        ffffffff",
        "01000000
        AA000000
        02000000
        00000000
        06000000
        06000000
        00000000
        00000000",
    );
    
    test_reload_one(
        "01000000
        AB000000
        01000000
        00000000
        06000000
        ffffffff",
        "01000000
        AB000000
        01000000
        00000000
        06000000
        00000000",
    );
    
    test_reload_one(
        "01000000
        AC000000
        01000000
        00000000
        06000000
        ffffffff",
        "01000000
        AC000000
        01000000
        00000000
        06000000
        00000000",
    );
    
    test_reload_one(
        "01000000
        AD000000
        01000000
        00000000
        06000000
        ffffffff",
        "01000000
        AD000000
        01000000
        00000000
        06000000
        00000000",
    );
    
    test_reload_one(
        "01000000
        AE000000
        02000000
        00000000
        06000000
        06000000
        ffffffff
        ffffffff",
        "01000000
        AE000000
        02000000
        00000000
        06000000
        06000000
        00000000
        00000000",
    );
    
    test_reload_one(
        "01000000
        AF000000
        02000000
        00000000
        06000000
        06000000
        ffffffff
        ffffffff",
        "01000000
        AF000000
        02000000
        00000000
        06000000
        06000000
        00000000
        00000000",
    );
    
    test_reload_one(
        "01000000
        B0000000
        01000000
        00000000
        06000000
        ffffffff",
        "01000000
        B0000000
        01000000
        00000000
        06000000
        00000000",
    );
    
    test_reload_one(
        "01000000
        B1000000
        03000000
        00000000
        06000000
        06000000
        06000000
        ffffffff
        ffffffff
        ffffffff",
        "01000000
        B1000000
        03000000
        00000000
        06000000
        06000000
        06000000
        00000000
        00000000
        00000000",
    );
    
    test_reload_one(
        "01000000
        B3000000
        02000000
        00000000
        06000000
        06000000
        ffffffff
        ffffffff",
        "01000000
        B3000000
        02000000
        00000000
        06000000
        06000000
        00000000
        00000000",
    );
    
    test_reload_one(
        "03000000
        31000000
        0B000000
        00000000
        07000000
        0F000000
        0F000000
        05000000
        0C000000
        05000000
        0C000000
        0F000000
        0F000000
        05000000
        0F000000
        42756C6C
        65744461
        6E636500
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
        01000000
        01000000
        FFFFFFFF
        ffffffff
        ffffffff
        ffffffff
        FFFFFFFF
        ffffffff
        ffffffff
        ffffffff
        00000000
        01000000
        FFFFFFFF
        0A000000",
        "03000000
        31000000
        0B000000
        00000000
        07000000
        0F000000
        0F000000
        05000000
        0C000000
        05000000
        0C000000
        0F000000
        0F000000
        05000000
        0F000000
        42756C6C
        65744461
        6E636500
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
        01000000
        01000000
        FFFFFFFF
        00000000
        00000000
        00000000
        FFFFFFFF
        00000000
        00000000
        00000000
        00000000
        01000000
        FFFFFFFF
        0A000000",
    );
}

