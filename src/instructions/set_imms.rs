use std::fmt::{Display, Formatter};
use regex::Regex;
use crate::imm_dec_to_hex;
use crate::instructions::{Instruction, ParseError};
use crate::instructions::helpers::make_instruction_number;
use crate::instructions::ParseError::{CannotWriteIntoReg0, MissingImm1, MissingReg1, RegexDoesNotMatch, UnsupportedImm1, UnsupportedReg1};

pub struct SetImm {
    instruction_number: u8,
    reg1: u8,
    imm1: u32,
}

impl Instruction for SetImm {
    type Instruction = Self;
    fn parse(instruction: &str, regex: Regex, instruction_number: u8) -> Result<Self::Instruction, ParseError> {
        return if let Some(captures) = regex.captures(instruction) {
            let reg1_number = if let Some(reg1) = captures.get(1) {
                let reg1 = reg1.as_str().to_owned();
                if reg1 == "reg0" { return Err(CannotWriteIntoReg0); }
                if reg1.is_empty() { return Err(MissingReg1); }
                let reg1_number = reg1.trim_start_matches("reg").parse::<u8>().map_err(|_| UnsupportedReg1(reg1.clone(), 1, 5))?;
                if reg1_number > 5 { return Err(UnsupportedReg1(reg1, 1, 5)); }
                reg1_number
            } else { return Err(MissingReg1); };

            let imm1_number = if let Some(imm1) = captures.get(2) {
                let imm1 = imm1.as_str().to_owned();
                if imm1.is_empty() { return Err(MissingImm1); }
                let imm1_number = imm1.parse::<u32>().map_err(|_| UnsupportedImm1(imm1.clone(), 65536))?;
                if imm1_number > 65535 { return Err(UnsupportedImm1(imm1, 65536)); }
                imm1_number
            } else { return Err(MissingImm1); };

            Ok(Self {instruction_number, reg1: reg1_number, imm1: imm1_number})
        } else {
            Err(RegexDoesNotMatch)
        };
    }
}

impl Display for SetImm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}0 {}", make_instruction_number(self.instruction_number).unwrap(), self.reg1, imm_dec_to_hex!(self.imm1))
    }
}
