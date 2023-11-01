mod instructions;

use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::process::exit;
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use regex::Regex;
use crate::instructions::Instruction;
use crate::instructions::build_error;
use crate::instructions::bomb::Bomb;
use crate::instructions::helpers::matches;
use crate::instructions::jumps::Jump;
use crate::instructions::mem_manipulation::MemManipulation;
use crate::instructions::reg_manipulation::RegManipulation;
use crate::instructions::set_imms::SetImm;
use crate::instructions::teleport::Teleport;

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
        return Err(anyhow!("output file must end with .tik\n:)"));
    }

    let mut sop_file = File::open(sop_raw_file).context("can not find input file")?;

    let mut input = String::new();
    sop_file.read_to_string(&mut input).context("cannot read input file to string")?;

    if File::open(tik_raw_file).is_ok() {
        return Err(anyhow!(format!("{} file already exists", tik_raw_file)));
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
    let rev_jump_regexes = vec![
        Regex::new(r"^if\((.*)==(.*)\)pc-=(.*)").context("can not create regex for REVJUMP1")?,
        Regex::new(r"^REVJUMP\((.*),(.*),(.*)\)").context("can not create regex for REVJUMP2")?,
        Regex::new(r"^REVJUMP(.*),(.*),(.*)").context("can not create regex for REVJUMP3")?,
    ];

    let lt_jump_regexes = vec![
        Regex::new(r"^if\((.*)<(.*)\)pc\+=(.*)").context("can not create regex for LTJUMP1")?,
        Regex::new(r"^LTJUMP\((.*),(.*),(.*)\)").context("can not create regex for LTJUMP2")?,
        Regex::new(r"^LTJUMP(.*),(.*),(.*)").context("can not create regex for LTJUMP3")?,
    ];
    let rev_lt_jump_regexes = vec![
        Regex::new(r"^if\((.*)<(.*)\)pc-=(.*)").context("can not create regex for REVLTJUMP1")?,
        Regex::new(r"^REVLTJUMP\((.*),(.*),(.*)\)").context("can not create regex for REVLTJUMP2")?,
        Regex::new(r"^REVLTJUMP(.*),(.*),(.*)").context("can not create regex for REVLTJUMP3")?,
    ];

    let neq_jump_regexes = vec![
        Regex::new(r"^if\((.*)!=(.*)\)pc\+=(.*)").context("can not create regex for NEQJUMP1")?,
        Regex::new(r"^NEQJUMP\((.*),(.*),(.*)\)").context("can not create regex for NEQJUMP2")?,
        Regex::new(r"^NEQJUMP(.*),(.*),(.*)").context("can not create regex for NEQJUMP3")?,
    ];
    let rev_neq_jump_regexes = vec![
        Regex::new(r"^if\((.*)!=(.*)\)pc-=(.*)").context("can not create regex for REVNEQJUMP1")?,
        Regex::new(r"^REVNEQJUMP\((.*),(.*),(.*)\)").context("can not create regex for REVNEQJUMP2")?,
        Regex::new(r"^REVNEQJUMP(.*),(.*),(.*)").context("can not create regex for REVNEQJUMP3")?,
    ];

    let set_imm_low_regexes = vec![
        Regex::new(r"(.*)\[low]=(.*)").context("can not create regex for SETIMMLOW1")?,
        Regex::new(r"^SETIMMLOW\((.*),(.*)\)").context("can not create regex for SETIMMLOW2")?,
        Regex::new(r"^SETIMMLOW(.*),(.*)").context("can not create regex for SETIMMLOW3")?
    ];
    let set_imm_high_regexes = vec![
        Regex::new(r"(.*)\[high]=(.*)").context("can not create regex for SETIMMHIGH1")?,
        Regex::new(r"^SETIMMHIGH\((.*),(.*)\)").context("can not create regex for SETIMMHIGH2")?,
        Regex::new(r"^SETIMMHIGH(.*),(.*)").context("can not create regex for SETIMMHIGH3")?
    ];

    let teleport_regexes = vec![
        Regex::new(r"^TELEPORT\((.*),(.*)\)").context("can not create regex for TELEPORT")?,
        Regex::new(r"^TELEPORT(.*),(.*)").context("can not create regex for TELEPORT")?,
    ];

    let bomb_regexes = vec![
        Regex::new(r"^BOMB\((.*)\)").context("can not create regex for BOMB2")?,
        Regex::new(r"^BOMB(.*)").context("can not create regex for BOMB1")?
    ];

    let instructions: Vec<&str> = input.split('\n').collect();
    let mut output = String::new();

    for (index, raw_instruction_and_comment) in instructions.iter().enumerate() {
        let instruction_and_comment = raw_instruction_and_comment.split_whitespace().collect::<String>();
        if instruction_and_comment.is_empty() {
            output.push('\n');
            continue;
        }

        let instruction = if let Some((instruction, _)) = instruction_and_comment.split_once(';') {
            instruction
        } else {
            &instruction_and_comment as &str
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

        if instruction.is_empty() {
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
        } else if let Some(matched_regex) = matches(instruction, &rev_jump_regexes) {
            process_jump_instruction!(instruction, matched_regex, 11, output, problem_line);
        } else if let Some(matched_regex) = matches(instruction, &lt_jump_regexes) {
            process_jump_instruction!(instruction, matched_regex, 12, output, problem_line);
        } else if let Some(matched_regex) = matches(instruction, &rev_lt_jump_regexes) {
            process_jump_instruction!(instruction, matched_regex, 13, output, problem_line);
        } else if let Some(matched_regex) = matches(instruction, &neq_jump_regexes) {
            process_jump_instruction!(instruction, matched_regex, 14, output, problem_line);
        } else if let Some(matched_regex) = matches(instruction, &rev_neq_jump_regexes) {
            process_jump_instruction!(instruction, matched_regex, 15, output, problem_line);
        } else if let Some(matched_regex) = matches(instruction, &set_imm_low_regexes) {
            match SetImm::parse(instruction, matched_regex, 20) {
                Ok(low) => output.push_str(&format!("{} ; {}\n", low, raw_instruction_and_comment)),
                Err(err) => return Err(anyhow!(build_error(err, problem_line, &20))),
            }
        } else if let Some(matched_regex) = matches(instruction, &set_imm_high_regexes) {
            match SetImm::parse(instruction, matched_regex, 21) {
                Ok(high) => output.push_str(&format!("{} ; {}\n", high, raw_instruction_and_comment)),
                Err(err) => return Err(anyhow!(build_error(err, problem_line, &21))),
            }
        } else if let Some(matched_regex) = matches(instruction, &bomb_regexes) {
            match Bomb::parse(instruction, matched_regex, 50) {
                Ok(bomb) => output.push_str(&format!("{} ; {}\n", bomb, raw_instruction_and_comment)),
                Err(err) => return Err(anyhow!(build_error(err, problem_line, &50))),
            }
        } else if let Some(matched_regex) = matches(instruction, &teleport_regexes) {
            match Teleport::parse(instruction, matched_regex, 42) {
                Ok(teleport) => output.push_str(&format!("{} ; {}\n", teleport, raw_instruction_and_comment)),
                Err(err) => return Err(anyhow!(build_error(err, problem_line, &42))),
            }
        } else {
            let problem = format!("in your program on line:\n\n{}\n\nproblem: {}", problem_line, "unknown instruction".red());
            return Err(anyhow!(problem));
        }
    }

    let mut tik_file = File::create(tik_raw_file).expect("can not create output file");
    tik_file.write_all(output.as_bytes()).context("failed to write program into .tik file")?;
    Ok(())
}