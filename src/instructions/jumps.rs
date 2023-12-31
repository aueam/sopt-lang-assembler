use std::fmt::{Display, Formatter};
use regex::Regex;
use crate::imm_dec_to_hex;
use crate::instructions::{Instruction, ParseError};
use crate::instructions::helpers::make_instruction_number;
use crate::instructions::ParseError::{MissingImm1, MissingReg1, MissingReg2, RegexDoesNotMatch, UnsupportedImm1, UnsupportedReg1, UnsupportedReg2};

pub struct Jump {
    instruction_number: u8,
    reg1: u8,
    reg2: u8,
    imm1: u32,
}

impl Instruction for Jump {
    type Instruction = Self;
    fn parse(instruction: &str, regex: Regex, instruction_number: u8) -> Result<Self::Instruction, ParseError> {
        return if let Some(captures) = regex.captures(instruction) {
            let reg1_number = if let Some(reg1) = captures.get(1) {
                let reg1 = reg1.as_str().to_owned();
                if reg1.is_empty() { return Err(MissingReg1); }
                let reg1_number = reg1.trim_start_matches("reg").parse::<u8>().map_err(|_| UnsupportedReg1(reg1.clone(), 0, 5))?;
                if reg1_number > 5 { return Err(UnsupportedReg1(reg1, 0, 5)); }
                reg1_number
            } else { return Err(MissingReg1); };

            let reg2_number = if let Some(reg2) = captures.get(2) {
                let reg2 = reg2.as_str().to_owned();
                if reg2.is_empty() { return Err(MissingReg2); }
                let reg2_number = reg2.trim_start_matches("reg").parse::<u8>().map_err(|_| UnsupportedReg2(reg2.clone(), 0, 5))?;
                if reg2_number > 5 { return Err(UnsupportedReg2(reg2, 0, 5)); }
                reg2_number
            } else { return Err(MissingReg2); };

            let imm1_number = if let Some(imm1) = captures.get(3) {
                let imm1 = imm1.as_str().to_owned();
                if imm1.is_empty() { return Err(MissingImm1); }
                let imm1_number = imm1.parse::<u32>().map_err(|_| UnsupportedImm1(imm1.clone(), 65536))?;
                if imm1_number > 65535 { return Err(UnsupportedImm1(imm1, 65536)); }
                imm1_number
            } else { return Err(MissingImm1); };

            Ok(Self {instruction_number, reg1: reg1_number, reg2: reg2_number, imm1: imm1_number})
        } else {
            Err(RegexDoesNotMatch)
        };
    }
}

impl Display for Jump {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}{} {}", make_instruction_number(self.instruction_number).unwrap(), self.reg1, self.reg2, imm_dec_to_hex!(self.imm1))
    }
}
