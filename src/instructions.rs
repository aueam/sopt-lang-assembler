use std::fmt::{Display, Formatter};
use colored::Colorize;
use regex::Regex;
use crate::instructions::helpers::{make_instruction_number, replace_first, replace_last};

pub mod jumps;
pub mod reg_manipulation;
pub mod mem_manipulation;
pub mod set_mins;
pub mod teleport;
pub mod bomb;
pub mod helpers;

#[derive(Debug)]
pub enum ParseError {
    RegexDoesNotMatch,
    CannotWriteIntoReg0,
    MissingReg1,
    MissingReg2,
    MissingImm1,
    MissingImm2,
    UnsupportedReg1(String, u8, u8),
    UnsupportedReg2(String, u8, u8),
    UnsupportedImm1(String, u32),
    UnsupportedImm2(String, u32)
}

pub trait Instruction {
    type Instruction;
    fn parse(instruction: &str, regex: Regex, instruction_number: u8) -> Result<Self::Instruction, ParseError>;
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string: String = match self {
            ParseError::RegexDoesNotMatch => "regex does not match".to_owned(),
            ParseError::CannotWriteIntoReg0 => "can not write into reg0".to_owned(),
            ParseError::MissingReg1 => "missing reg1 argument".to_owned(),
            ParseError::MissingReg2 => "missing reg2 argument".to_owned(),
            ParseError::MissingImm1 => "missing imm1 argument".to_owned(),
            ParseError::MissingImm2 => "missing imm2 argument".to_owned(),
            ParseError::UnsupportedReg1(_, min, max) => format!("unsupported reg number (supported: {}-{})", min, max),
            ParseError::UnsupportedReg2(_, min, max) => format!("unsupported reg number (supported: {}-{})", min, max),
            ParseError::UnsupportedImm1(_, max) => format!("unsupported imm number (supported: 0-{})", max),
            ParseError::UnsupportedImm2(_, max) => format!("unsupported imm number (supported: 0-{})", max)
        };

        write!(f, "{}", string)
    }
}

pub fn build_error(err: ParseError, problem_line: String, instruction: &u8) -> String {
    let mut problem = String::from("in your program on line:\n\n");

    // println!("{:?}", err);
    problem.push_str(&match err {
        ParseError::CannotWriteIntoReg0 => {
            format!(
                "{}\n\nproblem: {}",
                replace_first(&problem_line, "reg0", &"reg0".red().to_string()),
                err.to_string().red().to_string()
            )
        }
        ParseError::RegexDoesNotMatch | ParseError::MissingReg1 | ParseError::MissingReg2 | ParseError::MissingImm1 | ParseError::MissingImm2 => {
            format!("{}\n\nproblem: {}", problem_line, err.to_string().red().to_string())
        }
        ParseError::UnsupportedReg1(ref invalid_reg, _, _) => {
            format!(
                "{}\n\nproblem: {}",
                replace_first(&problem_line, &invalid_reg, &invalid_reg.red().to_string()),
                err.to_string().red().to_string()
            )
        }
        ParseError::UnsupportedReg2(ref invalid_reg, _, _) => {
            format!(
                "{}\n\nproblem: {}",
                replace_last(&problem_line, &invalid_reg, &invalid_reg.red().to_string()),
                err.to_string().red().to_string()
            )
        }
        ParseError::UnsupportedImm1(ref invalid_imm, _) => {
            format!(
                "{}\n\nproblem: {}",
                replace_first(&problem_line, &invalid_imm, &invalid_imm.red().to_string()),
                err.to_string().red().to_string()
            )
        }
        ParseError::UnsupportedImm2(ref invalid_imm, _) => {
            format!(
                "{}\n\nproblem: {}",
                replace_last(&problem_line, &invalid_imm, &invalid_imm.red().to_string()),
                err.to_string().red().to_string()
            )
        }
    });
    problem.push_str(&format!("\ninstruction found: {}", make_instruction_number(*instruction).unwrap()));
    problem
}