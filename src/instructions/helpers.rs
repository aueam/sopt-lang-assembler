use regex::Regex;

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

pub fn matches(instruction: &str, regexes: &Vec<Regex>) -> Option<Regex> {
    for regex in regexes {
        if regex.is_match(instruction) {
            return Some(regex.clone());
        }
    }
    None
}

pub fn replace_first(input: &str, from: &str, to: &str) -> String {
    if let Some(index) = input.find(from) {
        let (before, after) = input.split_at(index);
        format!("{}{}{}", before, to, &after[from.len()..])
    } else {
        input.to_string()
    }
}

pub fn replace_last(input: &str, from: &str, to: &str) -> String {
    if let Some(index) = input.rfind(from) {
        let (before, after) = input.split_at(index);
        format!("{}{}{}", before, to, &after[from.len()..])
    } else {
        input.to_string()
    }
}
