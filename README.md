# Sopt lang & assembler

This is unofficial implementation of assembler for Sopt lang.

Sopt lang is custom language made for the [FIKS competition](https://fiks.fit.cvut.cz/).

# How to build

1. download [Rust](https://www.rust-lang.org/)
2. download [repo](https://github.com/aueam/sopt-lang-assembler) with `git clone https://github.com/aueam/sopt-lang-assembler`
3. go to `cd sopt-lang-assembler`
4. build assembler with `cargo build --release`
5. you can copy binary `cp target/release/sopt-lang-assembler .`

# How to use

1. write your [code](#examples) into `program.sop`
2. use the tool with `./sopt-lang-assembler program.sop output.tik`
3. submit `output.tik`
4. earn 10 points :)

# Available instructions

- [x] NOP
- [x] ADD
- [x] SUB
- [x] MUL
- [x] LOAD
- [x] STORE
- [x] MOV
- [x] JUMP
- [x] REVJUMP
- [x] LTJUMP
- [x] REVLTJUMP
- [x] NEQJUMP
- [x] REVNEQJUMP
- [ ] SETIMMLOW
- [ ] SETIMMHIGH
- [x] TELEPORT
- [x] BOMB

_(all possible instruction formats can be found [here](input.example))_

# Examples

So what can you do with this tool?

After you write code:
```
NOP
reg1 += reg0 + 1
if (reg0 == reg0) pc -= 2
```
_(all possible instruction formats can be found [here](input.example))_

you can generate byte code:

```
69 00 00 00 ; NOP
01 10 00 01 ; reg1 += reg0 + 1
11 00 00 02 ; if (reg0 == reg0) pc -= 2
```