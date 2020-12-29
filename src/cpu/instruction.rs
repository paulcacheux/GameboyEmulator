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
    AddAIndirect {
        addr: Register16,
    },
    SubAReg8 {
        reg: Register8,
    },
    WriteMemLit {
        addr: u16,
        reg: Register8,
    },
    WriteLitAt {
        addr: Register16,
        literal: u8,
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
    ReadMemZeroPageLit {
        lit_offset: u8,
        reg: Register8,
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
        condition: Option<JumpCondition>,
        offset: i8,
    },
    JumpAbsolute {
        addr: u16,
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
    CompareLit {
        literal: u8,
    },
    CompareIndirectAddr {
        addr: Register16,
    },
    RotateLeftThroughCarryA,
    RotateLeftThroughCarry {
        reg: Register8,
    },
    EnableInterrupts,
    DisableInterrupts,
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
            Instruction::AddAIndirect { addr } => {
                write!(f, "ADD A, ({})", addr)
            }
            Instruction::SubAReg8 { reg } => {
                write!(f, "SUB A, {}", reg)
            }
            Instruction::WriteMemLit { addr, reg } => {
                write!(f, "LD (${:04x}), {}", addr, reg)
            }
            Instruction::WriteLitAt { addr, literal } => {
                write!(f, "LD ({}), ${:04x}", addr, literal)
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
            Instruction::ReadMemZeroPageLit { lit_offset, reg } => {
                write!(f, "LD {}, ($FF00 + ${:02x})", reg, lit_offset)
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
                if let Some(cond) = condition {
                    write!(f, "JR {}, {}", cond, offset) // TODO: change the offset format
                } else {
                    write!(f, "JR {}", offset) // TODO: change the offset format
                }
            }
            Instruction::JumpAbsolute { addr } => {
                write!(f, "JP {:06x}", addr)
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
            Instruction::CompareLit { literal } => {
                write!(f, "CP ${:02x}", literal)
            }
            Instruction::CompareIndirectAddr { addr } => {
                write!(f, "CP ({})", addr)
            }
            Instruction::RotateLeftThroughCarryA => {
                write!(f, "RLA")
            }
            Instruction::RotateLeftThroughCarry { reg } => {
                write!(f, "RL {}", reg)
            }
            Instruction::EnableInterrupts => write!(f, "EI"),
            Instruction::DisableInterrupts => write!(f, "DI"),
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
            Instruction::AddAIndirect { addr } => {
                vec![MicroOp::NOP, MicroOp::AddAIndirect { addr }]
            }
            Instruction::SubAReg8 { reg } => {
                vec![MicroOp::SubAReg { reg }]
            }
            Instruction::WriteMemLit { addr, reg } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::WriteMemLit { addr, reg },
                ]
            }
            Instruction::WriteLitAt { addr, literal } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::WriteLitAt { addr, literal },
                ]
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
                    MicroOp::WriteMemLit {
                        addr: 0xFF00 + lit_offset as u16,
                        reg,
                    },
                ]
            }
            Instruction::ReadMemZeroPageLit { lit_offset, reg } => vec![
                MicroOp::NOP,
                MicroOp::NOP,
                MicroOp::ReadMemLit {
                    reg,
                    addr: 0xFF00 + lit_offset as u16,
                },
            ],
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
                if let Some(cond) = condition {
                    MicroOp::CheckFlags(cond)
                } else {
                    MicroOp::NOP
                },
                MicroOp::RelativeJump(offset),
            ],
            Instruction::JumpAbsolute { addr } => vec![
                MicroOp::NOP,
                MicroOp::NOP,
                MicroOp::LoadRegLit {
                    reg: Register8::PCLow,
                    literal: addr as u8,
                },
                MicroOp::LoadRegLit {
                    reg: Register8::PCHigh,
                    literal: (addr >> 8) as u8,
                },
            ],
            Instruction::IncReg16 { reg } => vec![MicroOp::NOP, MicroOp::IncReg16 { reg }],
            Instruction::IncReg8 { reg } => vec![MicroOp::IncReg { reg }],
            Instruction::DecReg8 { reg } => vec![MicroOp::DecReg { reg }],
            Instruction::CompareLit { literal } => {
                vec![MicroOp::NOP, MicroOp::CompareALit { literal }]
            }
            Instruction::CompareIndirectAddr { addr } => {
                vec![MicroOp::NOP, MicroOp::CompareAIndirect { addr }]
            }
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
            Instruction::EnableInterrupts => vec![MicroOp::EnableInterrupts],
            Instruction::DisableInterrupts => vec![MicroOp::DisableInterrupts],
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
    AddAIndirect {
        addr: Register16,
    },
    SubAReg {
        reg: Register8,
    },
    WriteMemLit {
        addr: u16,
        reg: Register8,
    },
    WriteMem {
        addr: Register16,
        reg: Register8,
        pre_op: Option<PrePostOperation>,
        post_op: Option<PrePostOperation>,
    },
    WriteLitAt {
        addr: Register16,
        literal: u8,
    },
    WriteMemZeroPage {
        reg_offset: Register8,
        reg: Register8,
    },
    ReadMemLit {
        reg: Register8,
        addr: u16,
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
    CompareALit {
        literal: u8,
    },
    CompareAIndirect {
        addr: Register16,
    },
    RotateLeftThroughCarry {
        reg: Register8,
        set_zero: bool,
    },
    CheckFlags(JumpCondition),
    RelativeJump(i8),
    EnableInterrupts,
    DisableInterrupts,
}
