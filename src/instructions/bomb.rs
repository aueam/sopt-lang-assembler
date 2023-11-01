use std::fmt::{Display, Formatter};
use regex::Regex;
use crate::imm_dec_to_hex;
use crate::instructions::{Instruction, ParseError};
use crate::instructions::helpers::make_instruction_number;
use crate::instructions::ParseError::{MissingImm1, RegexDoesNotMatch, UnsupportedImm1};

pub struct Bomb {
    instruction_number: u8,
    imm1: u32,
}

impl Instruction for Bomb {
    type Instruction = Self;
    fn parse(instruction: &str, regex: Regex, instruction_number: u8) -> Result<Self::Instruction, ParseError> {
        return if let Some(captures) = regex.captures(instruction) {
            let imm1_number = if let Some(imm1) = captures.get(1) {
                let imm1 = imm1.as_str().to_owned();
                if imm1.is_empty() { return Err(MissingImm1); }
                let imm1_number = imm1.parse::<u32>().map_err(|_| UnsupportedImm1(imm1.clone(), 65536))?;
                if imm1_number > 65535 { return Err(UnsupportedImm1(imm1, 65536)); }
                imm1_number
            } else { return Err(MissingImm1); };

            Ok(Self {instruction_number, imm1: imm1_number})
        } else {
            Err(RegexDoesNotMatch)
        };
    }
}

impl Display for Bomb {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} 00", make_instruction_number(self.instruction_number).unwrap(), imm_dec_to_hex!(self.imm1))
    }
}
