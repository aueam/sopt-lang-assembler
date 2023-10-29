use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::process::exit;
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use regex::Regex;

// let mut code: String = "01 ".to_owned();
// if let Some(captures) = add.captures(s) {
//     if let Some(reg1) = captures.get(1) {
//         if reg1.as_str() == "" { return Err(anyhow!(format!("{problem}missing \"reg1\" argument").red())); }
//         let arg = reg1.as_str().trim_start_matches("reg");
//         let reg = arg.parse::<i8>().map_err(|_| anyhow!(format!("{problem}reg must contain number (1-5)").red()))?;
//         if reg == 0 {
//             return Err(anyhow!(format!("{problem}you can not write into reg0").red()));
//         } else if reg < 0 || reg > 5 {
//             return Err(anyhow!(format!("{problem}invalid reg number, reg must contain number (1-5)").red()));
//         }
//         code.push_str(arg);
//     }
//     if let Some(reg2) = captures.get(2) {
//         if reg2.as_str() == "" { return Err(anyhow!(format!("{problem}missing \"reg2\" argument").red())); }
//         let arg = reg2.as_str().trim_start_matches("reg");
//         let reg = arg.parse::<i8>().map_err(|_| anyhow!(format!("{problem}reg must contain number (0-5)").red()))?;
//         if reg < 0 || reg > 5 {
//             return Err(anyhow!(format!("{problem}invalid reg number, reg must contain number (1-5)").red()));
//         }
//         code.push_str(arg);
//         code.push(' ');
//     }
//     if let Some(imm) = captures.get(3) {
//         if imm.as_str() == "" { return Err(anyhow!(format!("{problem}missing \"imm\" argument").red())); }
//         let imm = imm.as_str().parse::<i32>().map_err(|_| anyhow!(format!("{problem}reg must contain number (0-5)").red()))?;
//         if imm < 0 || imm > 65535 {
//             return Err(anyhow!(format!("{problem}invalid imm, supported numbers: 0-65535").red()));
//         }
//         code.push_str(&format!("{:02X} {:02X}", (imm >> 8) & 0xFF, imm & 0xFF));
//     }
// } else { return Err(anyhow!(format!("{problem}instruction should be ADD but isn't it?").red())); }
// output.push_str(&format!("{code} ; {raw_instruction_and_comment}\n"))

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

    let nop = Regex::new(r"NOP").context("can not create regex for NOP")?;

    let add = Regex::new(r"(.*)\+=(.*)\+(.*)").context("can not create regex for ADD")?;
    let sub = Regex::new(r"(.*)-=(.*)\+(.*)").context("can not create regex for SUB")?;
    let mul = Regex::new(r"(.*)\*=(.*)\+(.*)").context("can not create regex for MUL")?;

    let load = Regex::new(r"(.*)=mem\[(.*)\+(.*)]").context("can not create regex for LOAD")?;
    let store = Regex::new(r"mem\[(.*)\+(.*)]=(.*)").context("can not create regex for STORE")?;

    let mov = Regex::new(r"(.*)(!.*[+\-*])=(.*)\+(.*)").context("can not create regex for MOV")?;

    let jump = Regex::new(r"if\((.*)==(.*)\)pc+=(.*)").context("can not create regex for JUMP")?;
    let revjump = Regex::new(r"if\((.*)==(.*)\)pc-=(.*)").context("can not create regex for REVJUMP")?;

    let ltjump = Regex::new(r"if\((.*)<(.*)\)pc+=(.*)").context("can not create regex for LTJUMP")?;
    let revltjump = Regex::new(r"if\((.*)<(.*)\)pc-=(.*)").context("can not create regex for REVLTJUMP")?;

    let neqjump = Regex::new(r"if\((.*)<(.*)\)pc+=(.*)").context("can not create regex for NEQJUMP")?;
    let revneqjump = Regex::new(r"if\((.*)<(.*)\)pc-=(.*)").context("can not create regex for REVNEQJUMP")?;

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

        let problem = format!("\nyou have problem in your program on line:\n\n{index}. {raw_instruction_and_comment}\n\nproblem: ");

        macro_rules! check_arg {
            ($arg:expr, $min:expr, $max:expr, &mut $code:expr, $msg:expr) => {
                if $arg == "" { return Err(anyhow!(format!("{problem}missing \"reg1\" argument").red())); }
                let arg = $arg.trim_start_matches("reg");
                let reg = arg.parse::<i8>()
                    .map_err(|_| anyhow!(format!("{problem}reg must contain number ({}-{})", $min, $max).red()))?;
                if reg < $min || reg > $max {
                    return Err(anyhow!(format!("{problem}{}", $msg).red()));
                }
                $code.push_str(arg);
            };
        }

        match instruction {
            s if nop.is_match(s) => output.push_str(&format!("69 00 00 00 ; {raw_instruction_and_comment}\n")),
            s if add.is_match(s) => {
                let mut code = String::from("01 ");

                if let Some(captures) = add.captures(s) {
                    check_arg!(captures.get(1).map(|r| r.as_str()).unwrap_or(""), 1, 5, &mut code, "invalid reg number, reg must contain number (1-5)");
                    check_arg!(captures.get(2).map(|r| r.as_str()).unwrap_or(""), 0, 5, &mut code, "invalid reg number, reg must contain number (0-5)");
                    code.push(' ');

                    if let Some(imm) = captures.get(3) {
                        let imm = imm.as_str().parse::<i32>()
                            .map_err(|_| anyhow!(format!("{problem}reg must contain number (0-5)").red()))?;
                        if imm < 0 || imm > 65535 {
                            return Err(anyhow!(format!("{problem}invalid imm, supported numbers: 0-65535").red()));
                        }
                        code.push_str(&format!("{:02X} {:02X}", (imm >> 8) & 0xFF, imm & 0xFF));
                    }
                } else {
                    return Err(anyhow!(format!("{problem}instruction should be ADD but isn't it?").red()));
                }

                output.push_str(&format!("{code} ; {raw_instruction_and_comment}\n"));
            }
            s if sub.is_match(s) => unimplemented!(),
            s if mul.is_match(s) => unimplemented!(),
            s if load.is_match(s) => unimplemented!(),
            s if store.is_match(s) => unimplemented!(),
            s if mov.is_match(s) => unimplemented!(),
            s if jump.is_match(s) => unimplemented!(),
            s if revjump.is_match(s) => unimplemented!(),
            s if ltjump.is_match(s) => unimplemented!(),
            s if revltjump.is_match(s) => unimplemented!(),
            s if neqjump.is_match(s) => unimplemented!(),
            s if revneqjump.is_match(s) => unimplemented!(),
            s if setimmlow.is_match(s) => unimplemented!(),
            s if setimmhigh.is_match(s) => unimplemented!(),
            s if teleport.is_match(s) => unimplemented!(),
            s if bomb.is_match(s) => unimplemented!(),
            s if s == "" => output.push_str(&format!("{raw_instruction_and_comment}\n")),
            _ => return Err(anyhow!(format!("{problem}unknown instruciton").red())),
        }
    }

    let mut tik_file = File::create(tik_raw_file).expect("can not create output file");

    tik_file.write_all(output.as_bytes()).context("failed to write program into .tik file")?;

    Ok(())
}
