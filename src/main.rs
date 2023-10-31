mod instructions;

use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::process::exit;
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use regex::Regex;
use crate::instructions::{Instruction, ParseError};
use crate::instructions::helpers::make_instruction_number;
use crate::instructions::jumps::Jump;
use crate::instructions::mem_manipulation::MemManipulation;
use crate::instructions::reg_manipulation::RegManipulation;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let usage = "Usage:\n\t./sopt-lang <path to input.sop> <path to output.tik>".bright_green();

    if args.len() != 3 {
        println!("{}", usage);
        exit(0);
    }

    let sop_raw_file = args.get(1).context("can not find input file argument")?;
    let tik_raw_file = args.get(2).context("can not find input file argument")?;

    if !sop_raw_file.ends_with(".sop") {
        return Err(anyhow!("input file must end with .sop\n:)"));
    }

    if !tik_raw_file.ends_with(".tik") {
        return Err(anyhow!("input file must end with .tik\n:)"));
    }

    let mut sop_file = File::open(sop_raw_file).context("can not find input file")?;

    let mut input = String::new();
    sop_file.read_to_string(&mut input).context("cannot read input file to string")?;

    if let Ok(_) = File::open(tik_raw_file) {
        return Err(anyhow!(".tik file already exists"));
    }

    let add_regexes = vec![
        Regex::new(r"(.*)\+=(.*)\+(.*)").context("can not create regex for ADD1")?,
        Regex::new(r"^ADD\((.*),(.*),(.*)\)").context("can not create regex for ADD2")?,
        Regex::new(r"^ADD(.*),(.*),(.*)").context("can not create regex for ADD3")?,
    ];
    let sub_regexes = vec![
        Regex::new(r"(.*)-=(.*)\+(.*)").context("can not create regex for SUB1")?,
        Regex::new(r"^SUB\((.*),(.*),(.*)\)").context("can not create regex for SUB2")?,
        Regex::new(r"^SUB(.*),(.*),(.*)").context("can not create regex for SUB3")?,
    ];
    let mul_regexes = vec![
        Regex::new(r"(.*)\*=(.*)\+(.*)").context("can not create regex for MUL1")?,
        Regex::new(r"^MUL\((.*),(.*),(.*)\)").context("can not create regex for MUL2")?,
        Regex::new(r"^MUL(.*),(.*),(.*)").context("can not create regex for MUL3")?,
    ];

    let load_regexes = vec![
        // Regex::new(r"(.*)=mem\[(.*)\+(.*)]").context("can not create regex for LOAD1")?, // TODO: Fix regexes?
        Regex::new(r"^LOAD\((.*),(.*),(.*)\)").context("can not create regex for LOAD2")?,
        Regex::new(r"^LOAD(.*),(.*),(.*)").context("can not create regex for LOAD3")?,
    ];
    let store_regexes = vec![
        // Regex::new(r"mem\[(.*)\+(.*)]=(.*)").context("can not create regex for STORE1")?, // TODO: Fix regexes?
        Regex::new(r"^STORE\((.*),(.*),(.*)\)").context("can not create regex for STORE2")?,
        Regex::new(r"^STORE(.*),(.*),(.*)").context("can not create regex for STORE3")?,
    ];

    let mov_regexes = vec![
        Regex::new(r"([^*+-]+)=(.*)\+([^=]+)").context("can not create regex for MOV1")?, // TODO: Is this correct?
        Regex::new(r"^MOV\((.*),(.*),(.*)\)").context("can not create regex for MOV2")?,
        Regex::new(r"^MOV(.*),(.*),(.*)").context("can not create regex for MOV3")?,
    ];

    let jump_regexes = vec![
        Regex::new(r"^if\((.*)==(.*)\)pc\+=(.*)").context("can not create regex for JUMP1")?,
        Regex::new(r"^JUMP\((.*),(.*),(.*)\)").context("can not create regex for JUMP2")?,
        Regex::new(r"^JUMP(.*),(.*),(.*)").context("can not create regex for JUMP3")?,
    ];
    let revjump_regexes = vec![
        Regex::new(r"^if\((.*)==(.*)\)pc-=(.*)").context("can not create regex for REVJUMP1")?,
        Regex::new(r"^REVJUMP\((.*),(.*),(.*)\)").context("can not create regex for REVJUMP2")?,
        Regex::new(r"^REVJUMP(.*),(.*),(.*)").context("can not create regex for REVJUMP3")?,
    ];

    let ltjump_regexes = vec![
        Regex::new(r"^if\((.*)<(.*)\)pc\+=(.*)").context("can not create regex for LTJUMP1")?,
        Regex::new(r"^LTJUMP\((.*),(.*),(.*)\)").context("can not create regex for LTJUMP2")?,
        Regex::new(r"^LTJUMP(.*),(.*),(.*)").context("can not create regex for LTJUMP3")?,
    ];
    let revltjump_regexes = vec![
        Regex::new(r"^if\((.*)<(.*)\)pc-=(.*)").context("can not create regex for REVLTJUMP1")?,
        Regex::new(r"^REVLTJUMP\((.*),(.*),(.*)\)").context("can not create regex for REVLTJUMP2")?,
        Regex::new(r"^REVLTJUMP(.*),(.*),(.*)").context("can not create regex for REVLTJUMP3")?,
    ];

    let neqjump_regexes = vec![
        Regex::new(r"^if\((.*)!=(.*)\)pc\+=(.*)").context("can not create regex for NEQJUMP1")?,
        Regex::new(r"^NEQJUMP\((.*),(.*),(.*)\)").context("can not create regex for NEQJUMP2")?,
        Regex::new(r"^NEQJUMP(.*),(.*),(.*)").context("can not create regex for NEQJUMP3")?,
    ];
    let revneqjump_regexes = vec![
        Regex::new(r"^if\((.*)!=(.*)\)pc-=(.*)").context("can not create regex for REVNEQJUMP1")?,
        Regex::new(r"^REVNEQJUMP\((.*),(.*),(.*)\)").context("can not create regex for REVNEQJUMP2")?,
        Regex::new(r"^REVNEQJUMP(.*),(.*),(.*)").context("can not create regex for REVNEQJUMP3")?,
    ];

    let setimmlow = Regex::new(r"(.*)\[low]=(.*)").context("can not create regex for SETIMMLOW")?;
    let setimmhigh = Regex::new(r"(.*)\[high]=(.*)").context("can not create regex for SETIMMHIGH")?;

    let teleport = Regex::new(r"teleport(.*),(.*)").context("can not create regex for TELEPORT")?;
    let bomb = Regex::new(r"bomb(.*)").context("can not create regex for BOMB")?;

    let instructions: Vec<&str> = input.split("\n").collect();
    let mut output = String::new();

    for (index, raw_instruction_and_comment) in instructions.iter().enumerate() {
        let instruction_and_comment = raw_instruction_and_comment.trim().split_whitespace().collect::<String>();
        if instruction_and_comment == "" { continue; }
        let (instruction, comment): (&str, Option<&str>) = match instruction_and_comment.split_once(";") {
            None => (&instruction_and_comment, None),
            Some((instruction, comment)) => (instruction, Some(comment))
        };

        let problem_line = format!("{}. {raw_instruction_and_comment}", index + 1);

        macro_rules! process_reg_instruction {
            ($instruction:expr, $matched_regex:expr, $param:expr, $output:expr, $problem_line:expr) => {
                match RegManipulation::parse($instruction, $matched_regex, $param) {
                    Ok(instruction) => $output.push_str(&format!("{} ; {}\n", instruction, raw_instruction_and_comment)),
                    Err(err) => return Err(anyhow!(build_error(err, $problem_line, &$param))),
                }
            };
        }

        macro_rules! process_jump_instruction {
            ($instruction:expr, $matched_regex:expr, $param:expr, $output:expr, $problem_line:expr) => {
                match Jump::parse($instruction, $matched_regex, $param) {
                    Ok(jump) => $output.push_str(&format!("{} ; {}\n", jump, raw_instruction_and_comment)),
                    Err(err) => return Err(anyhow!(build_error(err, $problem_line, &$param))),
                }
            };
        }

        if instruction == "" {
            output.push_str(&format!("{}\n", raw_instruction_and_comment));
        } else if instruction == "NOP" {
            output.push_str(&format!("69 00 00 00 ; {raw_instruction_and_comment}\n"));
        } else if let Some(matched_regex) = matches(instruction, &add_regexes) {
            process_reg_instruction!(instruction, matched_regex, 1, output, problem_line);
        } else if let Some(matched_regex) = matches(instruction, &sub_regexes) {
            process_reg_instruction!(instruction, matched_regex, 2, output, problem_line);
        } else if let Some(matched_regex) = matches(instruction, &mul_regexes) {
            process_reg_instruction!(instruction, matched_regex, 3, output, problem_line);
        } else if let Some(matched_regex) = matches(instruction, &mov_regexes) {
            process_reg_instruction!(instruction, matched_regex, 7, output, problem_line);
        } else if let Some(matched_regex) = matches(instruction, &load_regexes) {
            match MemManipulation::parse(instruction, matched_regex, 5, true) {
                Ok(load) => output.push_str(&format!("{} ; {}\n", load, raw_instruction_and_comment)),
                Err(err) => return Err(anyhow!(build_error(err, problem_line, &5))),
            }
        } else if let Some(matched_regex) = matches(instruction, &store_regexes) {
            match MemManipulation::parse(instruction, matched_regex, 6, false) {
                Ok(store) => output.push_str(&format!("{} ; {}\n", store, raw_instruction_and_comment)),
                Err(err) => return Err(anyhow!(build_error(err, problem_line, &6))),
            }
        } else if let Some(matched_regex) = matches(instruction, &jump_regexes) {
            process_jump_instruction!(instruction, matched_regex, 10, output, problem_line);
        } else if let Some(matched_regex) = matches(instruction, &revjump_regexes) {
            process_jump_instruction!(instruction, matched_regex, 11, output, problem_line);
        } else if let Some(matched_regex) = matches(instruction, &ltjump_regexes) {
            process_jump_instruction!(instruction, matched_regex, 12, output, problem_line);
        } else if let Some(matched_regex) = matches(instruction, &revltjump_regexes) {
            process_jump_instruction!(instruction, matched_regex, 13, output, problem_line);
        } else if let Some(matched_regex) = matches(instruction, &neqjump_regexes) {
            process_jump_instruction!(instruction, matched_regex, 14, output, problem_line);
        } else if let Some(matched_regex) = matches(instruction, &revneqjump_regexes) {
            process_jump_instruction!(instruction, matched_regex, 15, output, problem_line);
        } else {
            println!("{}", instruction);
            panic!("unknown instruction");
        }
    }

    let mut tik_file = File::create(tik_raw_file).expect("can not create output file");

    tik_file.write_all(output.as_bytes()).context("failed to write program into .tik file")?;

    Ok(())
}

fn matches(instruction: &str, regexes: &Vec<Regex>) -> Option<Regex> {
    for regex in regexes {
        if regex.is_match(instruction) {
            return Some(regex.clone());
        }
    }
    None
}

fn build_error(err: ParseError, problem_line: String, instruction: &u8) -> String {
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
    problem.push_str(&format!("\ninstruction: {}", make_instruction_number(*instruction).unwrap()));
    problem.clone()
}

fn replace_first(input: &str, from: &str, to: &str) -> String {
    if let Some(index) = input.find(from) {
        let (before, after) = input.split_at(index);
        format!("{}{}{}", before, to, &after[from.len()..])
    } else {
        input.to_string()
    }
}

fn replace_last(input: &str, from: &str, to: &str) -> String {
    if let Some(index) = input.rfind(from) {
        let (before, after) = input.split_at(index);
        format!("{}{}{}", before, to, &after[from.len()..])
    } else {
        input.to_string()
    }
}