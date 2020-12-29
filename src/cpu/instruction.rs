use std::fmt;

use super::micro_op::simpl;
use super::{Destination8Bits, MicroOp, Register16, Register8, Source8bits};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    NOP,
    Move {
        dest: Register8,
        src: Register8,
    },
    LoadLiteralIntoReg8 {
        reg: Register8,
        literal: u8,
    },
    LoadLiteralIntoReg16 {
        reg: Register16,
        literal: u16,
    },
    AndAWithReg8 {
        reg: Register8,
    },
    AndAWithLiteral {
        literal: u8,
    },
    OrAWithReg8 {
        reg: Register8,
    },
    XorAWithReg8 {
        reg: Register8,
    },
    XorAWithIndirect {
        addr: Register16,
    },
    AddAWithLiteral {
        literal: u8,
    },
    AddAWithIndirect {
        addr: Register16,
    },
    SubAWithReg8 {
        reg: Register8,
    },
    SubAWithLiteral {
        literal: u8,
    },
    WriteReg8ValueAtAddress {
        addr: u16,
        reg: Register8,
    },
    WriteLiteralAtIndirect {
        addr: Register16,
        literal: u8,
    },
    WriteReg8ValueAtIndirect {
        addr: Register16,
        reg: Register8,
        post_op: Option<PrePostOperation>,
    },
    WriteReg8ValueAtZeroPageOffsetReg8 {
        reg_offset: Register8,
        reg: Register8,
    },
    WriteReg8ValueAtZeroPageOffsetLiteral {
        lit_offset: u8,
        reg: Register8,
    },
    ReadIndirectToReg8 {
        reg: Register8,
        addr: Register16,
        post_op: Option<PrePostOperation>,
    },
    ReadAtAddressToReg8 {
        reg: Register8,
        addr: u16,
    },
    ReadZeroPageOffsetLiteralToReg8 {
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
        condition: Option<JumpCondition>,
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
    CompareAWithLiteral {
        literal: u8,
    },
    CompareAWithIndirect {
        addr: Register16,
    },
    RotateLeftThroughCarryA,
    RotateLeftThroughCarry {
        // different instruction because of flags
        reg: Register8,
    },
    EnableInterrupts,
    DisableInterrupts,
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
            Instruction::LoadLiteralIntoReg8 { reg, literal } => {
                write!(f, "LD {}, ${:02x}", reg, literal)
            }
            Instruction::LoadLiteralIntoReg16 { reg, literal } => {
                write!(f, "LD {}, ${:04x}", reg, literal)
            }
            Instruction::AndAWithReg8 { reg } => {
                write!(f, "AND A, {}", reg)
            }
            Instruction::AndAWithLiteral { literal } => {
                write!(f, "AND A, ${:02x}", literal)
            }
            Instruction::OrAWithReg8 { reg } => {
                write!(f, "OR A, {}", reg)
            }
            Instruction::XorAWithReg8 { reg } => {
                write!(f, "XOR A, {}", reg)
            }
            Instruction::XorAWithIndirect { addr } => {
                write!(f, "XOR A, ({})", addr)
            }
            Instruction::AddAWithIndirect { addr } => {
                write!(f, "ADD A, ({})", addr)
            }
            Instruction::AddAWithLiteral { literal } => {
                write!(f, "ADD A, ${:02x}", literal)
            }
            Instruction::SubAWithReg8 { reg } => {
                write!(f, "SUB A, {}", reg)
            }
            Instruction::SubAWithLiteral { literal } => {
                write!(f, "SUB A, ${:02x}", literal)
            }
            Instruction::WriteReg8ValueAtAddress { addr, reg } => {
                write!(f, "LD (${:04x}), {}", addr, reg)
            }
            Instruction::WriteLiteralAtIndirect { addr, literal } => {
                write!(f, "LD ({}), ${:04x}", addr, literal)
            }
            Instruction::WriteReg8ValueAtIndirect { addr, reg, post_op } => match post_op {
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
            Instruction::WriteReg8ValueAtZeroPageOffsetReg8 { reg_offset, reg } => {
                write!(f, "LD ($FF00 + {}), {}", reg_offset, reg)
            }
            Instruction::WriteReg8ValueAtZeroPageOffsetLiteral { lit_offset, reg } => {
                write!(f, "LD ($FF00 + ${:02x}), {}", lit_offset, reg)
            }
            Instruction::ReadIndirectToReg8 { reg, addr, post_op } => match post_op {
                Some(PrePostOperation::Dec) => {
                    write!(f, "LD {}, ({}-)", reg, addr)
                }
                Some(PrePostOperation::Inc) => {
                    write!(f, "LD {}, ({}+)", reg, addr)
                }
                None => {
                    write!(f, "LD {}, ({})", reg, addr)
                }
            },
            Instruction::ReadAtAddressToReg8 { reg, addr } => {
                write!(f, "LD {}, (${:04x})", reg, addr)
            }
            Instruction::ReadZeroPageOffsetLiteralToReg8 { lit_offset, reg } => {
                write!(f, "LD {}, ($FF00 + ${:02x})", reg, lit_offset)
            }
            Instruction::PushReg16 { reg } => {
                write!(f, "PUSH {}", reg)
            }
            Instruction::PopReg16 { reg } => {
                write!(f, "POP {}", reg)
            }
            Instruction::BitTest { reg, bit } => write!(f, "BIT {}, {}", bit, reg),
            Instruction::CallAddr { condition, addr } => {
                if let Some(cond) = condition {
                    write!(f, "CALL {}, ${:04x}", cond, addr)
                } else {
                    write!(f, "CALL ${:04x}", addr)
                }
            }
            Instruction::Return => write!(f, "RET"),
            Instruction::JumpRelative { condition, offset } => {
                if let Some(cond) = condition {
                    write!(f, "JR {}, {}", cond, offset) // TODO: change the offset format
                } else {
                    write!(f, "JR {}", offset) // TODO: change the offset format
                }
            }
            Instruction::JumpAbsolute { addr } => {
                write!(f, "JP ${:04x}", addr)
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
            Instruction::CompareAWithLiteral { literal } => {
                write!(f, "CP ${:02x}", literal)
            }
            Instruction::CompareAWithIndirect { addr } => {
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
            Instruction::Move { dest, src } => vec![simpl::move_micro_op(dest, src)],
            Instruction::LoadLiteralIntoReg8 { reg, literal } => {
                vec![MicroOp::NOP, simpl::load_literal_into_reg8(literal, reg)]
            }
            Instruction::LoadLiteralIntoReg16 { reg, literal } => {
                vec![
                    MicroOp::NOP,
                    simpl::load_literal_into_reg8(literal as u8, reg.lower_half()),
                    simpl::load_literal_into_reg8((literal >> 8) as u8, reg.higher_half()),
                ]
            }
            Instruction::AndAWithReg8 { reg } => vec![MicroOp::AndA { rhs: reg.into() }],
            Instruction::AndAWithLiteral { literal } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::AndA {
                        rhs: literal.into(),
                    },
                ]
            }
            Instruction::OrAWithReg8 { reg } => vec![MicroOp::OrA { rhs: reg.into() }],
            Instruction::XorAWithReg8 { reg } => {
                vec![MicroOp::XorA { rhs: reg.into() }]
            }
            Instruction::XorAWithIndirect { addr } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::XorA {
                        rhs: Source8bits::Indirect(addr),
                    },
                ]
            }
            Instruction::AddAWithIndirect { addr } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::AddA {
                        rhs: Source8bits::Indirect(addr),
                    },
                ]
            }
            Instruction::AddAWithLiteral { literal } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::AddA {
                        rhs: literal.into(),
                    },
                ]
            }
            Instruction::SubAWithReg8 { reg } => {
                vec![MicroOp::SubA { rhs: reg.into() }]
            }
            Instruction::SubAWithLiteral { literal } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::SubA {
                        rhs: literal.into(),
                    },
                ]
            }
            Instruction::WriteReg8ValueAtAddress { addr, reg } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::Move8Bits {
                        destination: Destination8Bits::Address(addr),
                        source: reg.into(),
                    },
                ]
            }
            Instruction::WriteLiteralAtIndirect { addr, literal } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::Move8Bits {
                        destination: Destination8Bits::Indirect(addr),
                        source: literal.into(),
                    },
                ]
            }
            Instruction::WriteReg8ValueAtIndirect { addr, reg, post_op } => {
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
            Instruction::WriteReg8ValueAtZeroPageOffsetReg8 { reg_offset, reg } => {
                vec![MicroOp::NOP, MicroOp::WriteMemZeroPage { reg_offset, reg }]
            }

            Instruction::WriteReg8ValueAtZeroPageOffsetLiteral { lit_offset, reg } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::Move8Bits {
                        destination: Destination8Bits::Address(0xFF00 + lit_offset as u16),
                        source: reg.into(),
                    },
                ]
            }
            Instruction::ReadZeroPageOffsetLiteralToReg8 { lit_offset, reg } => vec![
                MicroOp::NOP,
                MicroOp::NOP,
                MicroOp::Move8Bits {
                    destination: Destination8Bits::Register(reg),
                    source: Source8bits::Address(0xFF00 + lit_offset as u16),
                },
            ],
            Instruction::ReadAtAddressToReg8 { reg, addr } => vec![
                MicroOp::NOP,
                MicroOp::Move8Bits {
                    destination: Destination8Bits::Register(reg),
                    source: Source8bits::Address(addr),
                },
            ],

            Instruction::ReadIndirectToReg8 { reg, addr, post_op } => {
                vec![MicroOp::NOP, MicroOp::ReadMem { reg, addr, post_op }]
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
            Instruction::CallAddr { condition, addr } => {
                if let Some(cond) = condition {
                    //
                    vec![
                        // TODO: check if this is really the correct order
                        MicroOp::NOP,
                        MicroOp::NOP,
                        MicroOp::CheckFlags {
                            condition: cond,
                            true_ops: vec![
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
                            false_ops: vec![],
                        },
                    ]
                } else {
                    vec![
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
                    ]
                }
            }

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
            Instruction::JumpRelative { condition, offset } => {
                if let Some(cond) = condition {
                    vec![
                        MicroOp::NOP,
                        MicroOp::CheckFlags {
                            condition: cond,
                            true_ops: vec![MicroOp::RelativeJump(offset)],
                            false_ops: vec![],
                        },
                    ]
                } else {
                    vec![MicroOp::NOP, MicroOp::RelativeJump(offset)]
                }
            }
            Instruction::JumpAbsolute { addr } => vec![
                MicroOp::NOP,
                MicroOp::NOP,
                simpl::load_literal_into_reg8(addr as u8, Register8::PCLow),
                simpl::load_literal_into_reg8((addr >> 8) as u8, Register8::PCHigh),
            ],
            Instruction::IncReg16 { reg } => vec![MicroOp::NOP, MicroOp::IncReg16 { reg }],
            Instruction::IncReg8 { reg } => vec![MicroOp::IncReg { reg }],
            Instruction::DecReg8 { reg } => vec![MicroOp::DecReg { reg }],
            Instruction::CompareAWithLiteral { literal } => {
                vec![MicroOp::NOP, MicroOp::CompareALit { literal }]
            }
            Instruction::CompareAWithIndirect { addr } => {
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
