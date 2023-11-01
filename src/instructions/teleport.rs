use std::fmt::{Display, Formatter};
use regex::Regex;
use crate::imm_dec_to_hex;
use crate::instructions::{Instruction, ParseError};
use crate::instructions::helpers::make_instruction_number;
use crate::instructions::ParseError::{MissingImm1, MissingImm2, RegexDoesNotMatch, UnsupportedImm1, UnsupportedImm2};

pub struct Teleport {
    instruction_number: u8,
    imm1: u32,
    imm2: u32,
}

impl Instruction for Teleport {
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

            let imm2_number = if let Some(imm2) = captures.get(2) {
                let imm2 = imm2.as_str().to_owned();
                if imm2.is_empty() { return Err(MissingImm2); }
                let imm2_number = imm2.parse::<u32>().map_err(|_| UnsupportedImm2(imm2.clone(), 65536))?;
                if imm2_number > 255 { return Err(UnsupportedImm2(imm2, 255)); }
                imm2_number
            } else { return Err(MissingImm1); };

            Ok(Self {instruction_number, imm1: imm1_number, imm2: imm2_number})
        } else {
            Err(RegexDoesNotMatch)
        };
    }
}

impl Display for Teleport {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {:02X}", make_instruction_number(self.instruction_number).unwrap(), imm_dec_to_hex!(self.imm1), self.imm2)
    }
}
