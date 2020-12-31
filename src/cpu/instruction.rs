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
    Move16Bits {
        dest: Register16,
        src: Register16,
    },
    LoadLiteralIntoReg8 {
        reg: Register8,
        literal: u8,
    },
    LoadLiteralIntoReg16 {
        reg: Register16,
        literal: u16,
    },
    LoadAddressOffsetIntoReg16 {
        dest: Register16,
        base: Register16,
        offset: i8,
    },
    AndAWithReg8 {
        reg: Register8,
    },
    AndAWithLiteral {
        literal: u8,
    },
    AndAWithIndirect {
        addr: Register16,
    },
    OrAWithReg8 {
        reg: Register8,
    },
    OrAWithIndirect {
        addr: Register16,
    },
    OrAWithLiteral {
        literal: u8,
    },
    XorAWithReg8 {
        reg: Register8,
    },
    XorAWithIndirect {
        addr: Register16,
    },
    XorAWithLiteral {
        literal: u8,
    },
    AddAWithReg8 {
        reg: Register8,
    },
    AddAWithLiteral {
        literal: u8,
    },
    AddAWithIndirect {
        addr: Register16,
    },
    AddHLWithReg {
        reg: Register16,
    },
    AddOffsetToReg16 {
        reg: Register16,
        offset: i8,
    },
    AdcAWithReg8 {
        reg: Register8,
    },
    AdcAWithLiteral {
        literal: u8,
    },
    AdcAWithIndirect {
        addr: Register16,
    },
    SubAWithReg8 {
        reg: Register8,
    },
    SubAWithLiteral {
        literal: u8,
    },
    SubAWithIndirect {
        addr: Register16,
    },
    SbcAWithReg8 {
        reg: Register8,
    },
    SbcAWithLiteral {
        literal: u8,
    },
    SbcAWithIndirect {
        addr: Register16,
    },
    DAA,
    ComplementA,
    WriteReg8ValueAtAddress {
        addr: u16,
        reg: Register8,
    },
    WriteReg16ValueAtAddress {
        addr: u16,
        reg: Register16,
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
    ReadZeroPageOffsetReg8ToReg8 {
        offset: Register8,
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
    BitTestIndirect {
        addr: Register16,
        bit: u8,
    },
    ResetBit {
        reg: Register8,
        bit: u8,
    },
    ResetBitIndirect {
        addr: Register16,
        bit: u8,
    },
    SetBit {
        reg: Register8,
        bit: u8,
    },
    SetBitIndirect {
        addr: Register16,
        bit: u8,
    },
    CallAddr {
        condition: Option<JumpCondition>,
        addr: u16,
    },
    Return {
        condition: Option<JumpCondition>,
    },
    ReturnInterrupt,
    JumpRelative {
        condition: Option<JumpCondition>,
        offset: i8,
    },
    JumpAbsolute {
        condition: Option<JumpCondition>,
        addr: u16,
    },
    JumpRegister16 {
        reg: Register16,
    },
    Reset {
        offset: u16,
    },
    IncReg16 {
        reg: Register16,
    },
    IncReg8 {
        reg: Register8,
    },
    IncIndirect {
        addr: Register16,
    },
    DecReg16 {
        reg: Register16,
    },
    DecReg8 {
        reg: Register8,
    },
    DecIndirect {
        addr: Register16,
    },
    CompareAWithLiteral {
        literal: u8,
    },
    CompareAWithReg {
        reg: Register8,
    },
    CompareAWithIndirect {
        addr: Register16,
    },
    RotateLeftThroughCarryA,
    RotateLeftThroughCarry {
        // different instruction because of flags
        reg: Register8,
    },
    RotateLeftThroughCarryWithIndirect {
        addr: Register16,
    },
    RotateRightThroughCarryA,
    RotateRightThroughCarry {
        reg: Register8,
    },
    RotateRightThroughCarryWithIndirect {
        addr: Register16,
    },
    RotateLeftA,
    RotateLeft {
        // different instruction because of flags
        reg: Register8,
    },
    RotateLeftWithIndirect {
        // different instruction because of flags
        addr: Register16,
    },
    RotateRightA,
    RotateRight {
        reg: Register8,
    },
    RotateRightWithIndirect {
        // different instruction because of flags
        addr: Register16,
    },
    ShiftLeftIntoCarry {
        reg: Register8,
    },
    ShiftLeftIntoCarryWithIndirect {
        addr: Register16,
    },
    ShiftRightWithZeroIntoCarry {
        reg: Register8,
    },
    ShiftRightWithZeroIntoCarryWithIndirect {
        addr: Register16,
    },
    ShiftRightWithSignIntoCarry {
        reg: Register8,
    },
    ShiftRightWithSignIntoCarryWithIndirect {
        addr: Register16,
    },
    SwapReg8 {
        reg: Register8,
    },
    SwapIndirect {
        addr: Register16,
    },
    SetCarryFlag,
    ComplementCarryFlag,
    EnableInterrupts,
    DisableInterrupts,
    Halt,
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
            Instruction::Move16Bits { dest, src } => {
                write!(f, "LD {}, {}", dest, src)
            }
            Instruction::LoadLiteralIntoReg8 { reg, literal } => {
                write!(f, "LD {}, ${:02x}", reg, literal)
            }
            Instruction::LoadLiteralIntoReg16 { reg, literal } => {
                write!(f, "LD {}, ${:04x}", reg, literal)
            }
            Instruction::LoadAddressOffsetIntoReg16 { dest, base, offset } => {
                write!(f, "LD {}, {}+{}", dest, base, offset)
            }
            Instruction::AndAWithReg8 { reg } => {
                write!(f, "AND A, {}", reg)
            }
            Instruction::AndAWithLiteral { literal } => {
                write!(f, "AND A, ${:02x}", literal)
            }
            Instruction::AndAWithIndirect { addr } => {
                write!(f, "AND A, ({})", addr)
            }
            Instruction::OrAWithReg8 { reg } => {
                write!(f, "OR A, {}", reg)
            }
            Instruction::OrAWithIndirect { addr } => {
                write!(f, "OR A, ({})", addr)
            }
            Instruction::OrAWithLiteral { literal } => {
                write!(f, "OR A, ${:02x}", literal)
            }
            Instruction::XorAWithReg8 { reg } => {
                write!(f, "XOR A, {}", reg)
            }
            Instruction::XorAWithIndirect { addr } => {
                write!(f, "XOR A, ({})", addr)
            }
            Instruction::XorAWithLiteral { literal } => {
                write!(f, "XOR A, ${:02x}", literal)
            }
            Instruction::AddAWithReg8 { reg } => {
                write!(f, "ADD A, {}", reg)
            }
            Instruction::AddAWithIndirect { addr } => {
                write!(f, "ADD A, ({})", addr)
            }
            Instruction::AddAWithLiteral { literal } => {
                write!(f, "ADD A, ${:02x}", literal)
            }
            Instruction::AddHLWithReg { reg } => {
                write!(f, "ADD HL, {}", reg)
            }
            Instruction::AddOffsetToReg16 { reg, offset } => {
                write!(f, "ADD {}, {}", reg, offset) // check format for offset
            }
            Instruction::AdcAWithReg8 { reg } => {
                write!(f, "ADC A, {}", reg)
            }
            Instruction::AdcAWithLiteral { literal } => {
                write!(f, "ADC A, ${:02x}", literal)
            }
            Instruction::AdcAWithIndirect { addr } => {
                write!(f, "ADC A, ({})", addr)
            }
            Instruction::SubAWithReg8 { reg } => {
                write!(f, "SUB A, {}", reg)
            }
            Instruction::SubAWithLiteral { literal } => {
                write!(f, "SUB A, ${:02x}", literal)
            }
            Instruction::SubAWithIndirect { addr } => {
                write!(f, "SUB A, ({})", addr)
            }
            Instruction::SbcAWithReg8 { reg } => {
                write!(f, "SBC A, {}", reg)
            }
            Instruction::SbcAWithLiteral { literal } => {
                write!(f, "SBC A, ${:02x}", literal)
            }
            Instruction::SbcAWithIndirect { addr } => {
                write!(f, "SBC A, ({})", addr)
            }
            Instruction::DAA => write!(f, "DAA"),
            Instruction::ComplementA => write!(f, "CPL"),
            Instruction::WriteReg8ValueAtAddress { addr, reg } => {
                write!(f, "LD (${:04x}), {}", addr, reg)
            }
            Instruction::WriteReg16ValueAtAddress { addr, reg } => {
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
            Instruction::ReadZeroPageOffsetReg8ToReg8 { offset, reg } => {
                write!(f, "LD {}, ($FF00 + {})", reg, offset)
            }
            Instruction::PushReg16 { reg } => {
                write!(f, "PUSH {}", reg)
            }
            Instruction::PopReg16 { reg } => {
                write!(f, "POP {}", reg)
            }
            Instruction::BitTest { reg, bit } => write!(f, "BIT {}, {}", bit, reg),
            Instruction::ResetBit { reg, bit } => write!(f, "RES {}, {}", bit, reg),
            Instruction::SetBit { reg, bit } => write!(f, "SET {}, {}", bit, reg),
            Instruction::BitTestIndirect { addr, bit } => write!(f, "BIT {}, ({})", bit, addr),
            Instruction::ResetBitIndirect { addr, bit } => write!(f, "RES {}, ({})", bit, addr),
            Instruction::SetBitIndirect { addr, bit } => write!(f, "SET {}, ({})", bit, addr),
            Instruction::CallAddr { condition, addr } => {
                if let Some(cond) = condition {
                    write!(f, "CALL {}, ${:04x}", cond, addr)
                } else {
                    write!(f, "CALL ${:04x}", addr)
                }
            }
            Instruction::Return { condition } => {
                if let Some(cond) = condition {
                    write!(f, "RET {}", cond)
                } else {
                    write!(f, "RET")
                }
            }
            Instruction::ReturnInterrupt => {
                write!(f, "RETI")
            }
            Instruction::JumpRelative { condition, offset } => {
                if let Some(cond) = condition {
                    write!(f, "JR {}, {}", cond, offset) // TODO: change the offset format
                } else {
                    write!(f, "JR {}", offset) // TODO: change the offset format
                }
            }
            Instruction::JumpAbsolute { condition, addr } => {
                if let Some(cond) = condition {
                    write!(f, "JP {}, ${:04x}", cond, addr)
                } else {
                    write!(f, "JP ${:04x}", addr)
                }
            }
            Instruction::JumpRegister16 { reg } => {
                write!(f, "JP {}", reg)
            }
            Instruction::Reset { offset } => {
                write!(f, "RST ${:02x}", offset)
            }
            Instruction::IncReg16 { reg } => {
                write!(f, "INC {}", reg)
            }
            Instruction::IncReg8 { reg } => {
                write!(f, "INC {}", reg)
            }
            Instruction::IncIndirect { addr } => {
                write!(f, "INC ({})", addr)
            }
            Instruction::DecReg16 { reg } => {
                write!(f, "DEC {}", reg)
            }
            Instruction::DecReg8 { reg } => {
                write!(f, "DEC {}", reg)
            }
            Instruction::DecIndirect { addr } => {
                write!(f, "DEC ({})", addr)
            }
            Instruction::CompareAWithLiteral { literal } => {
                write!(f, "CP ${:02x}", literal)
            }
            Instruction::CompareAWithReg { reg } => {
                write!(f, "CP {}", reg)
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
            Instruction::RotateLeftThroughCarryWithIndirect { addr } => {
                write!(f, "RL ({})", addr)
            }
            Instruction::RotateRightThroughCarryA => {
                write!(f, "RRA")
            }
            Instruction::RotateRightThroughCarry { reg } => {
                write!(f, "RR {}", reg)
            }
            Instruction::RotateRightThroughCarryWithIndirect { addr } => {
                write!(f, "RR ({})", addr)
            }
            Instruction::RotateLeftA => {
                write!(f, "RLCA")
            }
            Instruction::RotateLeft { reg } => {
                write!(f, "RLC {}", reg)
            }
            Instruction::RotateLeftWithIndirect { addr } => {
                write!(f, "RLC ({})", addr)
            }
            Instruction::RotateRightA => {
                write!(f, "RRCA")
            }
            Instruction::RotateRight { reg } => {
                write!(f, "RRC {}", reg)
            }
            Instruction::ShiftLeftIntoCarry { reg } => {
                write!(f, "SLA {}", reg)
            }
            Instruction::ShiftRightWithZeroIntoCarry { reg } => {
                write!(f, "SRL {}", reg)
            }
            Instruction::ShiftRightWithSignIntoCarry { reg } => {
                write!(f, "SRA {}", reg)
            }

            Instruction::RotateRightWithIndirect { addr } => {
                write!(f, "RRC ({})", addr)
            }
            Instruction::ShiftLeftIntoCarryWithIndirect { addr } => {
                write!(f, "SLA ({})", addr)
            }
            Instruction::ShiftRightWithZeroIntoCarryWithIndirect { addr } => {
                write!(f, "SRL ({})", addr)
            }
            Instruction::ShiftRightWithSignIntoCarryWithIndirect { addr } => {
                write!(f, "SRA ({})", addr)
            }
            Instruction::SwapReg8 { reg } => {
                write!(f, "SWAP {}", reg)
            }
            Instruction::SwapIndirect { addr } => {
                write!(f, "SWAP ({})", addr)
            }
            Instruction::SetCarryFlag => write!(f, "SCF"),
            Instruction::ComplementCarryFlag => write!(f, "CCF"),
            Instruction::EnableInterrupts => write!(f, "EI"),
            Instruction::DisableInterrupts => write!(f, "DI"),
            Instruction::Halt => write!(f, "HALT"),
        }
    }
}

impl Instruction {
    pub fn to_micro_ops(self) -> Vec<MicroOp> {
        match self {
            Instruction::NOP => vec![MicroOp::NOP],
            Instruction::Move { dest, src } => vec![simpl::move_micro_op(dest, src)],
            Instruction::Move16Bits { dest, src } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::Move16Bits {
                        destination: dest,
                        source: src,
                    },
                ]
            }
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
            Instruction::LoadAddressOffsetIntoReg16 { dest, base, offset } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::AddOffsetToReg16IntoReg16 {
                        dest,
                        rhs: base,
                        offset,
                        update_flags: true,
                    },
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
            Instruction::AndAWithIndirect { addr } => {
                vec![MicroOp::NOP, MicroOp::AndA { rhs: addr.into() }]
            }
            Instruction::OrAWithReg8 { reg } => vec![MicroOp::OrA { rhs: reg.into() }],
            Instruction::OrAWithIndirect { addr } => {
                vec![MicroOp::NOP, MicroOp::OrA { rhs: addr.into() }]
            }
            Instruction::OrAWithLiteral { literal } => vec![
                MicroOp::NOP,
                MicroOp::OrA {
                    rhs: literal.into(),
                },
            ],
            Instruction::XorAWithReg8 { reg } => {
                vec![MicroOp::XorA { rhs: reg.into() }]
            }
            Instruction::XorAWithIndirect { addr } => {
                vec![MicroOp::NOP, MicroOp::XorA { rhs: addr.into() }]
            }
            Instruction::XorAWithLiteral { literal } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::XorA {
                        rhs: literal.into(),
                    },
                ]
            }
            Instruction::AddAWithReg8 { reg } => vec![MicroOp::AddA { rhs: reg.into() }],
            Instruction::AddAWithIndirect { addr } => {
                vec![MicroOp::NOP, MicroOp::AddA { rhs: addr.into() }]
            }
            Instruction::AddAWithLiteral { literal } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::AddA {
                        rhs: literal.into(),
                    },
                ]
            }
            Instruction::AddHLWithReg { reg } => {
                vec![MicroOp::NOP, MicroOp::AddHL { rhs: reg }]
            }
            Instruction::AddOffsetToReg16 { reg, offset } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::AddOffsetToReg16IntoReg16 {
                        dest: reg,
                        rhs: reg,
                        offset,
                        update_flags: true,
                    },
                ]
            }
            Instruction::AdcAWithReg8 { reg } => {
                vec![MicroOp::AdcA { rhs: reg.into() }]
            }
            Instruction::AdcAWithLiteral { literal } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::AdcA {
                        rhs: literal.into(),
                    },
                ]
            }
            Instruction::AdcAWithIndirect { addr } => {
                vec![MicroOp::NOP, MicroOp::AdcA { rhs: addr.into() }]
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
            Instruction::SubAWithIndirect { addr } => {
                vec![MicroOp::NOP, MicroOp::SubA { rhs: addr.into() }]
            }
            Instruction::SbcAWithReg8 { reg } => {
                vec![MicroOp::SbcA { rhs: reg.into() }]
            }
            Instruction::SbcAWithLiteral { literal } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::SbcA {
                        rhs: literal.into(),
                    },
                ]
            }
            Instruction::SbcAWithIndirect { addr } => {
                vec![MicroOp::NOP, MicroOp::SbcA { rhs: addr.into() }]
            }
            Instruction::DAA => vec![MicroOp::DAA],
            Instruction::ComplementA => vec![MicroOp::ComplementA],
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
            Instruction::WriteReg16ValueAtAddress { addr, reg } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::Move8Bits {
                        destination: Destination8Bits::Address(addr),
                        source: reg.lower_half().into(),
                    },
                    MicroOp::Move8Bits {
                        destination: Destination8Bits::Address(addr + 1),
                        source: reg.higher_half().into(),
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
            Instruction::ReadZeroPageOffsetReg8ToReg8 { offset, reg } => vec![
                MicroOp::NOP,
                MicroOp::Move8Bits {
                    destination: Destination8Bits::Register(reg),
                    source: Source8bits::ZeroPageOffsetReg8(offset),
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
            Instruction::BitTest { reg, bit } => vec![
                MicroOp::NOP,
                MicroOp::BitTest {
                    reg: reg.into(),
                    bit,
                },
            ],
            Instruction::ResetBit { reg, bit } => vec![
                MicroOp::NOP,
                MicroOp::ResetBit {
                    reg: reg.into(),
                    bit,
                },
            ],
            Instruction::SetBit { reg, bit } => vec![
                MicroOp::NOP,
                MicroOp::SetBit {
                    reg: reg.into(),
                    bit,
                },
            ],
            Instruction::BitTestIndirect { addr, bit } => vec![
                MicroOp::NOP,
                MicroOp::NOP,
                MicroOp::BitTest {
                    reg: addr.into(),
                    bit,
                },
            ],
            Instruction::ResetBitIndirect { addr, bit } => vec![
                MicroOp::NOP,
                MicroOp::NOP,
                MicroOp::NOP,
                MicroOp::ResetBit {
                    reg: addr.into(),
                    bit,
                },
            ],
            Instruction::SetBitIndirect { addr, bit } => vec![
                MicroOp::NOP,
                MicroOp::NOP,
                MicroOp::NOP,
                MicroOp::SetBit {
                    reg: addr.into(),
                    bit,
                },
            ],
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
            Instruction::Return { condition } => {
                if let Some(cond) = condition {
                    vec![
                        MicroOp::NOP,
                        MicroOp::CheckFlags {
                            condition: cond,
                            true_ops: vec![
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
                            false_ops: vec![],
                        },
                    ]
                } else {
                    vec![
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
                    ]
                }
            }
            Instruction::ReturnInterrupt => {
                vec![
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
                    MicroOp::EnableInterrupts,
                ]
            }
            Instruction::JumpRelative { condition, offset } => {
                if let Some(cond) = condition {
                    vec![
                        MicroOp::NOP,
                        MicroOp::CheckFlags {
                            condition: cond,
                            true_ops: vec![simpl::jump_relative(offset)],
                            false_ops: vec![],
                        },
                    ]
                } else {
                    vec![MicroOp::NOP, simpl::jump_relative(offset)]
                }
            }
            Instruction::JumpAbsolute { condition, addr } => {
                if let Some(cond) = condition {
                    vec![
                        MicroOp::NOP,
                        MicroOp::CheckFlags {
                            condition: cond,
                            true_ops: vec![
                                simpl::load_literal_into_reg8(addr as u8, Register8::PCLow),
                                simpl::load_literal_into_reg8((addr >> 8) as u8, Register8::PCHigh),
                            ],
                            false_ops: vec![MicroOp::NOP],
                        },
                    ]
                } else {
                    vec![
                        MicroOp::NOP,
                        MicroOp::NOP,
                        simpl::load_literal_into_reg8(addr as u8, Register8::PCLow),
                        simpl::load_literal_into_reg8((addr >> 8) as u8, Register8::PCHigh),
                    ]
                }
            }
            Instruction::JumpRegister16 { reg } => vec![MicroOp::Move16Bits {
                destination: Register16::PC,
                source: reg,
            }],
            Instruction::Reset { offset } => vec![
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
                    literal: offset,
                },
            ],
            Instruction::IncReg16 { reg } => vec![MicroOp::NOP, MicroOp::IncReg16 { reg }],
            Instruction::IncReg8 { reg } => vec![MicroOp::Inc { reg: reg.into() }],
            Instruction::IncIndirect { addr } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::Inc { reg: addr.into() },
                ]
            }
            Instruction::DecReg16 { reg } => vec![MicroOp::NOP, MicroOp::DecReg16 { reg }],
            Instruction::DecReg8 { reg } => vec![MicroOp::Dec { reg: reg.into() }],
            Instruction::DecIndirect { addr } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::Dec { reg: addr.into() },
                ]
            }
            Instruction::CompareAWithLiteral { literal } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::CompareA {
                        rhs: literal.into(),
                    },
                ]
            }
            Instruction::CompareAWithReg { reg } => vec![MicroOp::CompareA { rhs: reg.into() }],
            Instruction::CompareAWithIndirect { addr } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::CompareA {
                        rhs: Source8bits::Indirect(addr),
                    },
                ]
            }
            Instruction::RotateLeftThroughCarryA => vec![MicroOp::RotateLeftThroughCarry {
                reg: Register8::A.into(),
                set_zero: false,
            }],
            Instruction::RotateLeftThroughCarry { reg } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::RotateLeftThroughCarry {
                        reg: reg.into(),
                        set_zero: true,
                    },
                ]
            }
            Instruction::RotateLeftThroughCarryWithIndirect { addr } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::RotateLeftThroughCarry {
                        reg: addr.into(),
                        set_zero: true,
                    },
                ]
            }
            Instruction::RotateRightThroughCarryA => vec![MicroOp::RotateRightThroughCarry {
                reg: Register8::A.into(),
                set_zero: false,
            }],
            Instruction::RotateRightThroughCarry { reg } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::RotateRightThroughCarry {
                        reg: reg.into(),
                        set_zero: true,
                    },
                ]
            }
            Instruction::RotateRightThroughCarryWithIndirect { addr } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::RotateRightThroughCarry {
                        reg: addr.into(),
                        set_zero: true,
                    },
                ]
            }
            Instruction::RotateLeftA => vec![MicroOp::RotateLeft {
                reg: Register8::A.into(),
                set_zero: false,
            }],
            Instruction::RotateLeft { reg } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::RotateLeft {
                        reg: reg.into(),
                        set_zero: true,
                    },
                ]
            }
            Instruction::RotateLeftWithIndirect { addr } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::RotateLeft {
                        reg: addr.into(),
                        set_zero: true,
                    },
                ]
            }
            Instruction::RotateRightA => vec![MicroOp::RotateRight {
                reg: Register8::A.into(),
                set_zero: false,
            }],
            Instruction::RotateRight { reg } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::RotateRight {
                        reg: reg.into(),
                        set_zero: true,
                    },
                ]
            }
            Instruction::RotateRightWithIndirect { addr } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::RotateRight {
                        reg: addr.into(),
                        set_zero: true,
                    },
                ]
            }
            Instruction::ShiftLeftIntoCarry { reg } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::ShiftLeftIntoCarry { reg: reg.into() },
                ]
            }
            Instruction::ShiftLeftIntoCarryWithIndirect { addr } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::ShiftLeftIntoCarry { reg: addr.into() },
                ]
            }
            Instruction::ShiftRightWithZeroIntoCarry { reg } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::ShiftRightWithZeroIntoCarry { reg: reg.into() },
                ]
            }
            Instruction::ShiftRightWithZeroIntoCarryWithIndirect { addr } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::ShiftRightWithZeroIntoCarry { reg: addr.into() },
                ]
            }
            Instruction::ShiftRightWithSignIntoCarry { reg } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::ShiftRightWithSignIntoCarry { reg: reg.into() },
                ]
            }
            Instruction::ShiftRightWithSignIntoCarryWithIndirect { addr } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::ShiftRightWithSignIntoCarry { reg: addr.into() },
                ]
            }
            Instruction::SwapReg8 { reg } => {
                vec![MicroOp::NOP, MicroOp::SwapReg8 { reg: reg.into() }]
            }
            Instruction::SwapIndirect { addr } => {
                vec![
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::NOP,
                    MicroOp::SwapReg8 { reg: addr.into() },
                ]
            }
            Instruction::SetCarryFlag => vec![MicroOp::SetCarryFlag],
            Instruction::ComplementCarryFlag => vec![MicroOp::ComplementCarryFlag],
            Instruction::EnableInterrupts => vec![MicroOp::EnableInterrupts],
            Instruction::DisableInterrupts => vec![MicroOp::DisableInterrupts],
            Instruction::Halt => vec![MicroOp::Halt],
        }
    }
}
