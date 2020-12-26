use std::fmt;

use super::{Register16, Register8};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    NOP,
    Move {
        dest: Register8,
        src: Register8,
    },
    LoadRegLit8bits {
        reg: Register8,
        literal: u8,
    },
    LoadRegLit16bits {
        reg: Register16,
        literal: u16,
    },
    XorAReg8 {
        reg: Register8,
    },
    WriteMem {
        addr: Register16,
        reg: Register8,
        post_op: Option<PrePostOperation>,
    },
    WriteMemZeroPage {
        reg_offset: Register8,
        reg: Register8,
    },
    WriteMemZeroPageLit {
        lit_offset: u8,
        reg: Register8,
    },
    ReadMem {
        reg: Register8,
        addr: Register16,
    },
    PushReg16 {
        reg: Register16,
    },
    PopReg16 {
        reg: Register16,
    },
    BitTest {
        reg: Register8,
        bit: u8,
    },
    CallAddr {
        addr: u16,
    },
    Return,
    JumpRelative {
        condition: JumpCondition,
        offset: i8,
    },
    IncReg16 {
        reg: Register16,
    },
    IncReg8 {
        reg: Register8,
    },
    DecReg8 {
        reg: Register8,
    },
    RotateLeftThroughCarryA,
    RotateLeftThroughCarry {
        reg: Register8,
    },
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpCondition {
    NonZero,
    Zero,
    NonCarry,
    Carry,
}

impl fmt::Display for JumpCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JumpCondition::NonZero => {
                write!(f, "NZ")
            }
            JumpCondition::Zero => {
                write!(f, "Z")
            }
            JumpCondition::NonCarry => {
                write!(f, "NC")
            }
            JumpCondition::Carry => {
                write!(f, "C")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrePostOperation {
    Dec,
    Inc,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::NOP => {
                write!(f, "NOP")
            }
            Instruction::Move { dest, src } => {
                write!(f, "LD {}, {}", dest, src)
            }
            Instruction::LoadRegLit8bits { reg, literal } => {
                write!(f, "LD {}, ${:02x}", reg, literal)
            }
            Instruction::LoadRegLit16bits { reg, literal } => {
                write!(f, "LD {}, ${:04x}", reg, literal)
            }
            Instruction::XorAReg8 { reg } => {
                write!(f, "XOR A, {}", reg)
            }
            Instruction::WriteMem { addr, reg, post_op } => match post_op {
                Some(PrePostOperation::Dec) => {
                    write!(f, "LD ({}-), {}", addr, reg)
                }
                Some(PrePostOperation::Inc) => {
                    write!(f, "LD ({}+), {}", addr, reg)
                }
                None => {
                    write!(f, "LD ({}), {}", addr, reg)
                }
            },
            Instruction::WriteMemZeroPage { reg_offset, reg } => {
                write!(f, "LD ($FF00 + {}), {}", reg_offset, reg)
            }
            Instruction::WriteMemZeroPageLit { lit_offset, reg } => {
                write!(f, "LD ($FF00 + ${:02x}), {}", lit_offset, reg)
            }
            Instruction::ReadMem { reg, addr } => {
                write!(f, "LD {}, ({})", reg, addr)
            }
            Instruction::PushReg16 { reg } => {
                write!(f, "PUSH {}", reg)
            }
            Instruction::PopReg16 { reg } => {
                write!(f, "POP {}", reg)
            }
            Instruction::BitTest { reg, bit } => write!(f, "BIT {}, {}", bit, reg),
            Instruction::CallAddr { addr } => write!(f, "CALL ${:04x}", addr),
            Instruction::Return => write!(f, "RET"),
            Instruction::JumpRelative { condition, offset } => {
                write!(f, "JR {}, {}", condition, offset) // TODO: change the offset format
            }
            Instruction::IncReg16 { reg } => {
                write!(f, "INC {}", reg)
            }
            Instruction::IncReg8 { reg } => {
                write!(f, "INC {}", reg)
            }
            Instruction::DecReg8 { reg } => {
                write!(f, "DEC {}", reg)
            }
            Instruction::RotateLeftThroughCarryA => {
                write!(f, "RLA")
            }
            Instruction::RotateLeftThroughCarry { reg } => {
                write!(f, "RL {}", reg)
            }
        }
    }
}

impl Instruction {
    pub fn to_micro_ops(self) -> Vec<MicroOp> {
        match self {
            Instruction::NOP => vec![MicroOp::NOP],
            Instruction::Move { dest, src } => vec![MicroOp::Move { dest, src }],
            Instruction::LoadRegLit8bits { reg, literal } => {
                vec![MicroOp::NOP, MicroOp::LoadRegLit { reg, literal }]
            }
            Instruction::LoadRegLit16bits { reg, literal } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::LoadRegLit {
                        reg: reg.lower_half(),
                        literal: literal as u8,
                    },
                    MicroOp::LoadRegLit {
                        reg: reg.higher_half(),
                        literal: (literal >> 8) as u8,
                    },
                ]
            }
            Instruction::XorAReg8 { reg } => {
                vec![MicroOp::XorAReg { reg }]
            }
            Instruction::WriteMem { addr, reg, post_op } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::WriteMem {
                        addr,
                        reg,
                        pre_op: None,
                        post_op,
                    },
                ]
            }
            Instruction::WriteMemZeroPage { reg_offset, reg } => {
                vec![MicroOp::NOP, MicroOp::WriteMemZeroPage { reg_offset, reg }]
            }

            Instruction::WriteMemZeroPageLit { lit_offset, reg } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::WriteMemZeroPageLit { lit_offset, reg },
                ]
            }
            Instruction::ReadMem { reg, addr } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::ReadMem {
                        reg,
                        addr,
                        post_op: None,
                    },
                ]
            }
            Instruction::PushReg16 { reg } => vec![
                MicroOp::NOP,
                MicroOp::NOP,
                MicroOp::WriteMem {
                    addr: Register16::SP,
                    reg: reg.higher_half(),
                    pre_op: Some(PrePostOperation::Dec),
                    post_op: None,
                },
                MicroOp::WriteMem {
                    addr: Register16::SP,
                    reg: reg.lower_half(),
                    pre_op: Some(PrePostOperation::Dec),
                    post_op: None,
                },
            ],
            Instruction::PopReg16 { reg } => vec![
                MicroOp::NOP,
                MicroOp::ReadMem {
                    reg: reg.lower_half(),
                    addr: Register16::SP,
                    post_op: Some(PrePostOperation::Inc),
                },
                MicroOp::ReadMem {
                    reg: reg.higher_half(),
                    addr: Register16::SP,
                    post_op: Some(PrePostOperation::Inc),
                },
            ],
            Instruction::BitTest { reg, bit } => vec![MicroOp::NOP, MicroOp::BitTest { reg, bit }],
            Instruction::CallAddr { addr } => vec![
                // TODO: check if this is really the correct order
                MicroOp::NOP,
                MicroOp::NOP,
                MicroOp::NOP,
                MicroOp::WriteMem {
                    addr: Register16::SP,
                    reg: Register8::PCHigh,
                    pre_op: Some(PrePostOperation::Dec),
                    post_op: None,
                },
                MicroOp::WriteMem {
                    addr: Register16::SP,
                    reg: Register8::PCLow,
                    pre_op: Some(PrePostOperation::Dec),
                    post_op: None,
                },
                MicroOp::LoadReg16Lit {
                    reg: Register16::PC,
                    literal: addr,
                },
            ],
            Instruction::Return => vec![
                MicroOp::NOP,
                MicroOp::NOP,
                MicroOp::ReadMem {
                    reg: Register8::PCLow,
                    addr: Register16::SP,
                    post_op: Some(PrePostOperation::Inc),
                },
                MicroOp::ReadMem {
                    reg: Register8::PCHigh,
                    addr: Register16::SP,
                    post_op: Some(PrePostOperation::Inc),
                },
            ],
            Instruction::JumpRelative { condition, offset } => vec![
                MicroOp::NOP,
                MicroOp::CheckFlags(condition),
                MicroOp::RelativeJump(offset),
            ],
            Instruction::IncReg16 { reg } => vec![MicroOp::NOP, MicroOp::IncReg16 { reg }],
            Instruction::IncReg8 { reg } => vec![MicroOp::IncReg { reg }],
            Instruction::DecReg8 { reg } => vec![MicroOp::DecReg { reg }],
            Instruction::RotateLeftThroughCarryA => vec![MicroOp::RotateLeftThroughCarry {
                reg: Register8::A,
                set_zero: false,
            }],
            Instruction::RotateLeftThroughCarry { reg } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::RotateLeftThroughCarry {
                        reg,
                        set_zero: true,
                    },
                ]
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicroOp {
    NOP,
    Move {
        dest: Register8,
        src: Register8,
    },
    LoadRegLit {
        reg: Register8,
        literal: u8,
    },
    LoadReg16Lit {
        reg: Register16,
        literal: u16,
    },
    XorAReg {
        reg: Register8,
    },
    WriteMem {
        addr: Register16,
        reg: Register8,
        pre_op: Option<PrePostOperation>,
        post_op: Option<PrePostOperation>,
    },
    WriteMemZeroPage {
        reg_offset: Register8,
        reg: Register8,
    },
    WriteMemZeroPageLit {
        lit_offset: u8,
        reg: Register8,
    },
    ReadMem {
        reg: Register8,
        addr: Register16,
        post_op: Option<PrePostOperation>,
    },
    BitTest {
        reg: Register8,
        bit: u8,
    },
    IncReg16 {
        reg: Register16,
    },
    IncReg {
        reg: Register8,
    },
    DecReg {
        reg: Register8,
    },
    RotateLeftThroughCarry {
        reg: Register8,
        set_zero: bool,
    },
    CheckFlags(JumpCondition),
    RelativeJump(i8),
}
