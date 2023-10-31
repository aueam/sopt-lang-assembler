pub fn make_instruction_number(number: u8) -> Option<String> {
    return if number < 10 {
        Some(format!("0{number}"))
    } else if number > 99 {
        None
    } else {
        Some(format!("{number}"))
    }
}

#[macro_export]
macro_rules! imm_dec_to_hex {
    ($imm_dec:expr) => {
        format!("{:02X} {:02X}", ($imm_dec >> 8) & 0xFF, $imm_dec & 0xFF)
    };
}