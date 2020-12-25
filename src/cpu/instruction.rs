use std::fmt;

use super::{Register16, Register8};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    NOP,
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
    StoreMem {
        addr: Register16,
        reg: Register8,
        post_op: Option<PostOperation>,
    },
    StoreMemZeroPage {
        reg_offset: Register8,
        reg: Register8,
    },
    StoreMemZeroPageLit {
        lit_offset: u8,
        reg: Register8,
    },
    BitTest {
        reg: Register8,
        bit: u8,
    },
    JumpRelative {
        condition: JumpCondition,
        offset: i8,
    },
    IncReg8 {
        reg: Register8,
    },
}

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
pub enum PostOperation {
    Dec,
    Inc,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::NOP => {
                write!(f, "NOP")
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
            Instruction::StoreMem { addr, reg, post_op } => match post_op {
                Some(PostOperation::Dec) => {
                    write!(f, "LD ({}-), {}", addr, reg)
                }
                Some(PostOperation::Inc) => {
                    write!(f, "LD ({}+), {}", addr, reg)
                }
                None => {
                    write!(f, "LD ({}), {}", addr, reg)
                }
            },
            Instruction::StoreMemZeroPage { reg_offset, reg } => {
                write!(f, "LD ($FF00 + {}), {}", reg_offset, reg)
            }
            Instruction::StoreMemZeroPageLit { lit_offset, reg } => {
                write!(f, "LD ($FF00 + ${:02x}), {}", lit_offset, reg)
            }
            Instruction::BitTest { reg, bit } => write!(f, "BIT {}, {}", bit, reg),
            Instruction::JumpRelative { condition, offset } => {
                write!(f, "JR {}, {}", condition, offset) // TODO: change the offset format
            }
            Instruction::IncReg8 { reg } => {
                write!(f, "INC {}", reg)
            }
        }
    }
}

impl Instruction {
    pub fn to_micro_ops(self) -> Vec<MicroOp> {
        match self {
            Instruction::NOP => vec![MicroOp::NOP],
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
            Instruction::StoreMem { addr, reg, post_op } => {
                vec![MicroOp::NOP, MicroOp::StoreMem { addr, reg, post_op }]
            }
            Instruction::StoreMemZeroPage { reg_offset, reg } => {
                vec![MicroOp::NOP, MicroOp::StoreMemZeroPage { reg_offset, reg }]
            }

            Instruction::StoreMemZeroPageLit { lit_offset, reg } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::StoreMemZeroPageLit { lit_offset, reg },
                ]
            }
            Instruction::BitTest { reg, bit } => vec![MicroOp::NOP, MicroOp::BitTest { reg, bit }],
            Instruction::JumpRelative { condition, offset } => vec![
                MicroOp::NOP,
                MicroOp::CheckFlags(condition),
                MicroOp::RelativeJump(offset),
            ],
            Instruction::IncReg8 { reg } => vec![MicroOp::IncReg { reg }],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicroOp {
    NOP,
    LoadRegLit {
        reg: Register8,
        literal: u8,
    },
    XorAReg {
        reg: Register8,
    },
    StoreMem {
        addr: Register16,
        reg: Register8,
        post_op: Option<PostOperation>,
    },
    StoreMemZeroPage {
        reg_offset: Register8,
        reg: Register8,
    },
    StoreMemZeroPageLit {
        lit_offset: u8,
        reg: Register8,
    },
    BitTest {
        reg: Register8,
        bit: u8,
    },
    IncReg {
        reg: Register8,
    },
    CheckFlags(JumpCondition),
    RelativeJump(i8),
}
