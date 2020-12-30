use super::{
    instruction::{Instruction, JumpCondition, PrePostOperation},
    micro_op::Reg8OrIndirect,
    register::{Register16, Register8},
    CPU,
};
use crate::memory::Memory;

pub fn decode_instruction<M: Memory>(cpu: &mut CPU<M>) -> Instruction {
    let pc = cpu.pc;
    let opcode = cpu.fetch_and_advance();

    match opcode {
        0x00 => Instruction::NOP,
        0x01 => Instruction::LoadLiteralIntoReg16 {
            reg: Register16::BC,
            literal: cpu.fetch_and_advance_u16(),
        },
        0x02 => Instruction::WriteReg8ValueAtIndirect {
            reg: Register8::A,
            addr: Register16::BC,
            post_op: None,
        },
        0x03 => Instruction::IncReg16 {
            reg: Register16::BC,
        },
        0x04 => Instruction::IncReg8 { reg: Register8::B },
        0x05 => Instruction::DecReg8 { reg: Register8::B },
        0x06 => Instruction::LoadLiteralIntoReg8 {
            reg: Register8::B,
            literal: cpu.fetch_and_advance(),
        },
        0x07 => Instruction::RotateLeftA,
        0x08 => Instruction::WriteReg16ValueAtAddress {
            addr: cpu.fetch_and_advance_u16(),
            reg: Register16::SP,
        },
        0x09 => Instruction::AddHLWithReg {
            reg: Register16::BC,
        },
        0x0A => Instruction::ReadIndirectToReg8 {
            addr: Register16::BC,
            reg: Register8::A,
            post_op: None,
        },
        0x0B => Instruction::DecReg16 {
            reg: Register16::BC,
        },
        0x0C => Instruction::IncReg8 { reg: Register8::C },
        0x0D => Instruction::DecReg8 { reg: Register8::C },
        0x0E => Instruction::LoadLiteralIntoReg8 {
            reg: Register8::C,
            literal: cpu.fetch_and_advance(),
        },
        0x0F => Instruction::RotateRightA,
        0x11 => Instruction::LoadLiteralIntoReg16 {
            reg: Register16::DE,
            literal: cpu.fetch_and_advance_u16(),
        },
        0x12 => Instruction::WriteReg8ValueAtIndirect {
            addr: Register16::DE,
            reg: Register8::A,
            post_op: None,
        },
        0x13 => Instruction::IncReg16 {
            reg: Register16::DE,
        },
        0x14 => Instruction::IncReg8 { reg: Register8::D },
        0x15 => Instruction::DecReg8 { reg: Register8::D },
        0x16 => Instruction::LoadLiteralIntoReg8 {
            reg: Register8::D,
            literal: cpu.fetch_and_advance(),
        },
        0x17 => Instruction::RotateLeftThroughCarryA,
        0x18 => Instruction::JumpRelative {
            condition: None,
            offset: cpu.fetch_and_advance() as i8,
        },
        0x19 => Instruction::AddHLWithReg {
            reg: Register16::DE,
        },
        0x1A => Instruction::ReadIndirectToReg8 {
            addr: Register16::DE,
            reg: Register8::A,
            post_op: None,
        },
        0x1B => Instruction::DecReg16 {
            reg: Register16::DE,
        },
        0x1C => Instruction::IncReg8 { reg: Register8::E },
        0x1D => Instruction::DecReg8 { reg: Register8::E },
        0x1E => Instruction::LoadLiteralIntoReg8 {
            reg: Register8::E,
            literal: cpu.fetch_and_advance(),
        },
        0x1F => Instruction::RotateRightThroughCarryA,
        0x20 => Instruction::JumpRelative {
            condition: Some(JumpCondition::NonZero),
            offset: cpu.fetch_and_advance() as i8,
        },
        0x21 => Instruction::LoadLiteralIntoReg16 {
            reg: Register16::HL,
            literal: cpu.fetch_and_advance_u16(),
        },
        0x22 => Instruction::WriteReg8ValueAtIndirect {
            addr: Register16::HL,
            reg: Register8::A,
            post_op: Some(PrePostOperation::Inc),
        },
        0x23 => Instruction::IncReg16 {
            reg: Register16::HL,
        },
        0x24 => Instruction::IncReg8 { reg: Register8::H },
        0x25 => Instruction::DecReg8 { reg: Register8::H },
        0x26 => Instruction::LoadLiteralIntoReg8 {
            reg: Register8::H,
            literal: cpu.fetch_and_advance(),
        },
        0x27 => Instruction::DAA,
        0x28 => Instruction::JumpRelative {
            condition: Some(JumpCondition::Zero),
            offset: cpu.fetch_and_advance() as i8,
        },
        0x29 => Instruction::AddHLWithReg {
            reg: Register16::HL,
        },
        0x2A => Instruction::ReadIndirectToReg8 {
            reg: Register8::A,
            addr: Register16::HL,
            post_op: Some(PrePostOperation::Inc),
        },
        0x2B => Instruction::DecReg16 {
            reg: Register16::HL,
        },
        0x2C => Instruction::IncReg8 { reg: Register8::L },
        0x2D => Instruction::DecReg8 { reg: Register8::L },
        0x2E => Instruction::LoadLiteralIntoReg8 {
            reg: Register8::L,
            literal: cpu.fetch_and_advance(),
        },
        0x2F => Instruction::ComplementA,
        0x30 => Instruction::JumpRelative {
            condition: Some(JumpCondition::NonCarry),
            offset: cpu.fetch_and_advance() as i8,
        },
        0x31 => Instruction::LoadLiteralIntoReg16 {
            reg: Register16::SP,
            literal: cpu.fetch_and_advance_u16(),
        },
        0x32 => Instruction::WriteReg8ValueAtIndirect {
            addr: Register16::HL,
            reg: Register8::A,
            post_op: Some(PrePostOperation::Dec),
        },
        0x33 => Instruction::IncReg16 {
            reg: Register16::SP,
        },
        0x34 => Instruction::IncIndirect {
            addr: Register16::HL,
        },
        0x35 => Instruction::DecIndirect {
            addr: Register16::HL,
        },
        0x36 => Instruction::WriteLiteralAtIndirect {
            addr: Register16::HL,
            literal: cpu.fetch_and_advance(),
        },
        0x37 => Instruction::SetCarryFlag,
        0x38 => Instruction::JumpRelative {
            condition: Some(JumpCondition::Carry),
            offset: cpu.fetch_and_advance() as i8,
        },
        0x39 => Instruction::AddHLWithReg {
            reg: Register16::SP,
        },
        0x3A => Instruction::ReadIndirectToReg8 {
            addr: Register16::HL,
            reg: Register8::A,
            post_op: Some(PrePostOperation::Dec),
        },
        0x3B => Instruction::DecReg16 {
            reg: Register16::SP,
        },
        0x3C => Instruction::IncReg8 { reg: Register8::A },
        0x3D => Instruction::DecReg8 { reg: Register8::A },
        0x3E => Instruction::LoadLiteralIntoReg8 {
            reg: Register8::A,
            literal: cpu.fetch_and_advance(),
        },
        0x3F => Instruction::ComplementCarryFlag,
        0x40 => Instruction::Move {
            dest: Register8::B,
            src: Register8::B,
        },
        0x41 => Instruction::Move {
            dest: Register8::B,
            src: Register8::C,
        },
        0x42 => Instruction::Move {
            dest: Register8::B,
            src: Register8::D,
        },
        0x43 => Instruction::Move {
            dest: Register8::B,
            src: Register8::E,
        },
        0x44 => Instruction::Move {
            dest: Register8::B,
            src: Register8::H,
        },
        0x45 => Instruction::Move {
            dest: Register8::B,
            src: Register8::L,
        },
        0x46 => Instruction::ReadIndirectToReg8 {
            reg: Register8::B,
            addr: Register16::HL,
            post_op: None,
        },
        0x47 => Instruction::Move {
            dest: Register8::B,
            src: Register8::A,
        },
        0x48 => Instruction::Move {
            dest: Register8::C,
            src: Register8::B,
        },
        0x49 => Instruction::Move {
            dest: Register8::C,
            src: Register8::C,
        },
        0x4A => Instruction::Move {
            dest: Register8::C,
            src: Register8::D,
        },
        0x4B => Instruction::Move {
            dest: Register8::C,
            src: Register8::E,
        },
        0x4C => Instruction::Move {
            dest: Register8::C,
            src: Register8::H,
        },
        0x4D => Instruction::Move {
            dest: Register8::C,
            src: Register8::L,
        },
        0x4E => Instruction::ReadIndirectToReg8 {
            reg: Register8::C,
            addr: Register16::HL,
            post_op: None,
        },
        0x4F => Instruction::Move {
            dest: Register8::C,
            src: Register8::A,
        },
        0x50 => Instruction::Move {
            dest: Register8::D,
            src: Register8::B,
        },
        0x51 => Instruction::Move {
            dest: Register8::D,
            src: Register8::C,
        },
        0x52 => Instruction::Move {
            dest: Register8::D,
            src: Register8::D,
        },
        0x53 => Instruction::Move {
            dest: Register8::D,
            src: Register8::E,
        },
        0x54 => Instruction::Move {
            dest: Register8::D,
            src: Register8::H,
        },
        0x55 => Instruction::Move {
            dest: Register8::D,
            src: Register8::L,
        },
        0x56 => Instruction::ReadIndirectToReg8 {
            reg: Register8::D,
            addr: Register16::HL,
            post_op: None,
        },
        0x57 => Instruction::Move {
            dest: Register8::D,
            src: Register8::A,
        },
        0x58 => Instruction::Move {
            dest: Register8::E,
            src: Register8::B,
        },
        0x59 => Instruction::Move {
            dest: Register8::E,
            src: Register8::C,
        },
        0x5A => Instruction::Move {
            dest: Register8::E,
            src: Register8::D,
        },
        0x5B => Instruction::Move {
            dest: Register8::E,
            src: Register8::E,
        },
        0x5C => Instruction::Move {
            dest: Register8::E,
            src: Register8::H,
        },
        0x5D => Instruction::Move {
            dest: Register8::E,
            src: Register8::L,
        },
        0x5E => Instruction::ReadIndirectToReg8 {
            reg: Register8::E,
            addr: Register16::HL,
            post_op: None,
        },
        0x5F => Instruction::Move {
            dest: Register8::E,
            src: Register8::A,
        },
        0x60 => Instruction::Move {
            dest: Register8::H,
            src: Register8::B,
        },
        0x61 => Instruction::Move {
            dest: Register8::H,
            src: Register8::C,
        },
        0x62 => Instruction::Move {
            dest: Register8::H,
            src: Register8::D,
        },
        0x63 => Instruction::Move {
            dest: Register8::H,
            src: Register8::E,
        },
        0x64 => Instruction::Move {
            dest: Register8::H,
            src: Register8::H,
        },
        0x65 => Instruction::Move {
            dest: Register8::H,
            src: Register8::L,
        },
        0x66 => Instruction::ReadIndirectToReg8 {
            addr: Register16::HL,
            reg: Register8::H,
            post_op: None,
        },
        0x67 => Instruction::Move {
            dest: Register8::H,
            src: Register8::A,
        },
        0x68 => Instruction::Move {
            dest: Register8::L,
            src: Register8::B,
        },
        0x69 => Instruction::Move {
            dest: Register8::L,
            src: Register8::C,
        },
        0x6A => Instruction::Move {
            dest: Register8::L,
            src: Register8::D,
        },
        0x6B => Instruction::Move {
            dest: Register8::L,
            src: Register8::E,
        },
        0x6C => Instruction::Move {
            dest: Register8::L,
            src: Register8::H,
        },
        0x6D => Instruction::Move {
            dest: Register8::L,
            src: Register8::L,
        },
        0x6E => Instruction::ReadIndirectToReg8 {
            addr: Register16::HL,
            reg: Register8::L,
            post_op: None,
        },
        0x6F => Instruction::Move {
            dest: Register8::L,
            src: Register8::A,
        },
        0x70 => Instruction::WriteReg8ValueAtIndirect {
            addr: Register16::HL,
            reg: Register8::B,
            post_op: None,
        },
        0x71 => Instruction::WriteReg8ValueAtIndirect {
            addr: Register16::HL,
            reg: Register8::C,
            post_op: None,
        },
        0x72 => Instruction::WriteReg8ValueAtIndirect {
            addr: Register16::HL,
            reg: Register8::D,
            post_op: None,
        },
        0x73 => Instruction::WriteReg8ValueAtIndirect {
            addr: Register16::HL,
            reg: Register8::E,
            post_op: None,
        },
        0x74 => Instruction::WriteReg8ValueAtIndirect {
            addr: Register16::HL,
            reg: Register8::H,
            post_op: None,
        },
        0x75 => Instruction::WriteReg8ValueAtIndirect {
            addr: Register16::HL,
            reg: Register8::L,
            post_op: None,
        },
        0x77 => Instruction::WriteReg8ValueAtIndirect {
            addr: Register16::HL,
            reg: Register8::A,
            post_op: None,
        },
        0x78 => Instruction::Move {
            dest: Register8::A,
            src: Register8::B,
        },
        0x79 => Instruction::Move {
            dest: Register8::A,
            src: Register8::C,
        },
        0x7A => Instruction::Move {
            dest: Register8::A,
            src: Register8::D,
        },
        0x7B => Instruction::Move {
            dest: Register8::A,
            src: Register8::E,
        },
        0x7C => Instruction::Move {
            dest: Register8::A,
            src: Register8::H,
        },
        0x7D => Instruction::Move {
            dest: Register8::A,
            src: Register8::L,
        },
        0x7E => Instruction::ReadIndirectToReg8 {
            reg: Register8::A,
            addr: Register16::HL,
            post_op: None,
        },
        0x7F => Instruction::Move {
            dest: Register8::A,
            src: Register8::A,
        },
        0x80 => Instruction::AddAWithReg8 { reg: Register8::B },
        0x81 => Instruction::AddAWithReg8 { reg: Register8::C },
        0x82 => Instruction::AddAWithReg8 { reg: Register8::D },
        0x83 => Instruction::AddAWithReg8 { reg: Register8::E },
        0x84 => Instruction::AddAWithReg8 { reg: Register8::H },
        0x85 => Instruction::AddAWithReg8 { reg: Register8::L },
        0x86 => Instruction::AddAWithIndirect {
            addr: Register16::HL,
        },
        0x87 => Instruction::AddAWithReg8 { reg: Register8::A },
        0x88 => Instruction::AdcAWithReg8 { reg: Register8::B },
        0x89 => Instruction::AdcAWithReg8 { reg: Register8::C },
        0x8A => Instruction::AdcAWithReg8 { reg: Register8::D },
        0x8B => Instruction::AdcAWithReg8 { reg: Register8::E },
        0x8C => Instruction::AdcAWithReg8 { reg: Register8::H },
        0x8D => Instruction::AdcAWithReg8 { reg: Register8::L },
        0x8E => Instruction::AdcAWithIndirect {
            addr: Register16::HL,
        },
        0x8F => Instruction::AdcAWithReg8 { reg: Register8::A },
        0x90 => Instruction::SubAWithReg8 { reg: Register8::B },
        0x91 => Instruction::SubAWithReg8 { reg: Register8::C },
        0x92 => Instruction::SubAWithReg8 { reg: Register8::D },
        0x93 => Instruction::SubAWithReg8 { reg: Register8::E },
        0x94 => Instruction::SubAWithReg8 { reg: Register8::H },
        0x95 => Instruction::SubAWithReg8 { reg: Register8::L },
        0x96 => Instruction::SubAWithIndirect {
            addr: Register16::HL,
        },
        0x97 => Instruction::SubAWithReg8 { reg: Register8::A },

        0x98 => Instruction::SbcAWithReg8 { reg: Register8::B },
        0x99 => Instruction::SbcAWithReg8 { reg: Register8::C },
        0x9A => Instruction::SbcAWithReg8 { reg: Register8::D },
        0x9B => Instruction::SbcAWithReg8 { reg: Register8::E },
        0x9C => Instruction::SbcAWithReg8 { reg: Register8::H },
        0x9D => Instruction::SbcAWithReg8 { reg: Register8::L },
        0x9E => Instruction::SbcAWithIndirect {
            addr: Register16::HL,
        },
        0x9F => Instruction::SbcAWithReg8 { reg: Register8::A },

        0xA0 => Instruction::AndAWithReg8 { reg: Register8::B },
        0xA1 => Instruction::AndAWithReg8 { reg: Register8::C },
        0xA2 => Instruction::AndAWithReg8 { reg: Register8::D },
        0xA3 => Instruction::AndAWithReg8 { reg: Register8::E },
        0xA4 => Instruction::AndAWithReg8 { reg: Register8::H },
        0xA5 => Instruction::AndAWithReg8 { reg: Register8::L },
        0xA6 => Instruction::AndAWithIndirect {
            addr: Register16::HL,
        },
        0xA7 => Instruction::AndAWithReg8 { reg: Register8::A },
        0xA8 => Instruction::XorAWithReg8 { reg: Register8::B },
        0xA9 => Instruction::XorAWithReg8 { reg: Register8::C },
        0xAA => Instruction::XorAWithReg8 { reg: Register8::D },
        0xAB => Instruction::XorAWithReg8 { reg: Register8::E },
        0xAC => Instruction::XorAWithReg8 { reg: Register8::H },
        0xAD => Instruction::XorAWithReg8 { reg: Register8::L },
        0xAE => Instruction::XorAWithIndirect {
            addr: Register16::HL,
        },
        0xAF => Instruction::XorAWithReg8 { reg: Register8::A },
        0xB0 => Instruction::OrAWithReg8 { reg: Register8::B },
        0xB1 => Instruction::OrAWithReg8 { reg: Register8::C },
        0xB2 => Instruction::OrAWithReg8 { reg: Register8::D },
        0xB3 => Instruction::OrAWithReg8 { reg: Register8::E },
        0xB4 => Instruction::OrAWithReg8 { reg: Register8::H },
        0xB5 => Instruction::OrAWithReg8 { reg: Register8::L },
        0xB6 => Instruction::OrAWithIndirect {
            addr: Register16::HL,
        },
        0xB7 => Instruction::OrAWithReg8 { reg: Register8::A },
        0xB8 => Instruction::CompareAWithReg { reg: Register8::B },
        0xB9 => Instruction::CompareAWithReg { reg: Register8::C },
        0xBA => Instruction::CompareAWithReg { reg: Register8::D },
        0xBB => Instruction::CompareAWithReg { reg: Register8::E },
        0xBC => Instruction::CompareAWithReg { reg: Register8::H },
        0xBD => Instruction::CompareAWithReg { reg: Register8::L },
        0xBE => Instruction::CompareAWithIndirect {
            addr: Register16::HL,
        },
        0xBF => Instruction::CompareAWithReg { reg: Register8::A },
        0xC0 => Instruction::Return {
            condition: Some(JumpCondition::NonZero),
        },
        0xC1 => Instruction::PopReg16 {
            reg: Register16::BC,
        },
        0xC2 => Instruction::JumpAbsolute {
            condition: Some(JumpCondition::NonZero),
            addr: cpu.fetch_and_advance_u16(),
        },
        0xC3 => Instruction::JumpAbsolute {
            condition: None,
            addr: cpu.fetch_and_advance_u16(),
        },
        0xC4 => Instruction::CallAddr {
            condition: Some(JumpCondition::NonZero),
            addr: cpu.fetch_and_advance_u16(),
        },
        0xC5 => Instruction::PushReg16 {
            reg: Register16::BC,
        },
        0xC6 => Instruction::AddAWithLiteral {
            literal: cpu.fetch_and_advance(),
        },
        0xC7 => Instruction::Reset { offset: 0x00 },
        0xC8 => Instruction::Return {
            condition: Some(JumpCondition::Zero),
        },
        0xC9 => Instruction::Return { condition: None },
        0xCA => Instruction::JumpAbsolute {
            condition: Some(JumpCondition::Zero),
            addr: cpu.fetch_and_advance_u16(),
        },
        0xCB => {
            // prefix 0xCB:
            match cpu.fetch_and_advance() {
                0x00 => Instruction::RotateLeft { reg: Register8::B },
                0x01 => Instruction::RotateLeft { reg: Register8::C },
                0x02 => Instruction::RotateLeft { reg: Register8::D },
                0x03 => Instruction::RotateLeft { reg: Register8::E },
                0x04 => Instruction::RotateLeft { reg: Register8::H },
                0x05 => Instruction::RotateLeft { reg: Register8::L },
                0x07 => Instruction::RotateLeft { reg: Register8::A },

                0x08 => Instruction::RotateRight { reg: Register8::B },
                0x09 => Instruction::RotateRight { reg: Register8::C },
                0x0A => Instruction::RotateRight { reg: Register8::D },
                0x0B => Instruction::RotateRight { reg: Register8::E },
                0x0C => Instruction::RotateRight { reg: Register8::H },
                0x0D => Instruction::RotateRight { reg: Register8::L },
                0x0F => Instruction::RotateRight { reg: Register8::A },

                0x10 => Instruction::RotateLeftThroughCarry { reg: Register8::B },
                0x11 => Instruction::RotateLeftThroughCarry { reg: Register8::C },
                0x12 => Instruction::RotateLeftThroughCarry { reg: Register8::D },
                0x13 => Instruction::RotateLeftThroughCarry { reg: Register8::E },
                0x14 => Instruction::RotateLeftThroughCarry { reg: Register8::H },
                0x15 => Instruction::RotateLeftThroughCarry { reg: Register8::L },

                0x17 => Instruction::RotateLeftThroughCarry { reg: Register8::A },
                0x18 => Instruction::RotateRightThroughCarry { reg: Register8::B },
                0x19 => Instruction::RotateRightThroughCarry { reg: Register8::C },
                0x1A => Instruction::RotateRightThroughCarry { reg: Register8::D },
                0x1B => Instruction::RotateRightThroughCarry { reg: Register8::E },
                0x1C => Instruction::RotateRightThroughCarry { reg: Register8::H },
                0x1D => Instruction::RotateRightThroughCarry { reg: Register8::L },
                0x1F => Instruction::RotateRightThroughCarry { reg: Register8::A },

                0x20 => Instruction::ShiftLeftIntoCarry { reg: Register8::B },
                0x21 => Instruction::ShiftLeftIntoCarry { reg: Register8::C },
                0x22 => Instruction::ShiftLeftIntoCarry { reg: Register8::D },
                0x23 => Instruction::ShiftLeftIntoCarry { reg: Register8::E },
                0x24 => Instruction::ShiftLeftIntoCarry { reg: Register8::H },
                0x25 => Instruction::ShiftLeftIntoCarry { reg: Register8::L },
                0x27 => Instruction::ShiftLeftIntoCarry { reg: Register8::A },

                0x28 => Instruction::ShiftRightWithSignIntoCarry { reg: Register8::B },
                0x29 => Instruction::ShiftRightWithSignIntoCarry { reg: Register8::C },
                0x2A => Instruction::ShiftRightWithSignIntoCarry { reg: Register8::D },
                0x2B => Instruction::ShiftRightWithSignIntoCarry { reg: Register8::E },
                0x2C => Instruction::ShiftRightWithSignIntoCarry { reg: Register8::H },
                0x2D => Instruction::ShiftRightWithSignIntoCarry { reg: Register8::L },
                0x2F => Instruction::ShiftRightWithSignIntoCarry { reg: Register8::A },

                0x30 => Instruction::SwapReg8 { reg: Register8::B },
                0x31 => Instruction::SwapReg8 { reg: Register8::C },
                0x32 => Instruction::SwapReg8 { reg: Register8::D },
                0x33 => Instruction::SwapReg8 { reg: Register8::E },
                0x34 => Instruction::SwapReg8 { reg: Register8::H },
                0x35 => Instruction::SwapReg8 { reg: Register8::L },
                0x37 => Instruction::SwapReg8 { reg: Register8::A },

                0x38 => Instruction::ShiftRightWithZeroIntoCarry { reg: Register8::B },
                0x39 => Instruction::ShiftRightWithZeroIntoCarry { reg: Register8::C },
                0x3A => Instruction::ShiftRightWithZeroIntoCarry { reg: Register8::D },
                0x3B => Instruction::ShiftRightWithZeroIntoCarry { reg: Register8::E },
                0x3C => Instruction::ShiftRightWithZeroIntoCarry { reg: Register8::H },
                0x3D => Instruction::ShiftRightWithZeroIntoCarry { reg: Register8::L },
                0x3F => Instruction::ShiftRightWithZeroIntoCarry { reg: Register8::A },

                opcode @ 0x40..=0x7F => decode_bit_test(opcode),
                opcode @ 0x80..=0xBF => decode_reset_bit(opcode),
                opcode @ 0xC0..=0xFF => decode_set_bit(opcode),

                other => panic!("Unknown sub-opcode (prefix 0xCB) {:#x}", other),
            }
        }
        0xCC => Instruction::CallAddr {
            condition: Some(JumpCondition::Zero),
            addr: cpu.fetch_and_advance_u16(),
        },
        0xCD => Instruction::CallAddr {
            condition: None,
            addr: cpu.fetch_and_advance_u16(),
        },
        0xCE => Instruction::AdcAWithLiteral {
            literal: cpu.fetch_and_advance(),
        },
        0xCF => Instruction::Reset { offset: 0x08 },
        0xD0 => Instruction::Return {
            condition: Some(JumpCondition::NonCarry),
        },
        0xD1 => Instruction::PopReg16 {
            reg: Register16::DE,
        },
        0xD2 => Instruction::JumpAbsolute {
            condition: Some(JumpCondition::NonCarry),
            addr: cpu.fetch_and_advance_u16(),
        },
        // 0xD3 => nothing
        0xD4 => Instruction::CallAddr {
            condition: Some(JumpCondition::NonCarry),
            addr: cpu.fetch_and_advance_u16(),
        },
        0xD5 => Instruction::PushReg16 {
            reg: Register16::DE,
        },
        0xD6 => Instruction::SubAWithLiteral {
            literal: cpu.fetch_and_advance(),
        },
        0xD7 => Instruction::Reset { offset: 0x10 },
        0xD8 => Instruction::Return {
            condition: Some(JumpCondition::Carry),
        },
        0xD9 => Instruction::ReturnInterrupt,
        0xDA => Instruction::JumpAbsolute {
            condition: Some(JumpCondition::Carry),
            addr: cpu.fetch_and_advance_u16(),
        },
        // 0xDB => nothing
        0xDC => Instruction::CallAddr {
            condition: Some(JumpCondition::Carry),
            addr: cpu.fetch_and_advance_u16(),
        },
        // 0xDD => nothing
        0xDE => Instruction::SbcAWithLiteral {
            literal: cpu.fetch_and_advance(),
        },
        0xDF => Instruction::Reset { offset: 0x18 },
        0xE0 => Instruction::WriteReg8ValueAtZeroPageOffsetLiteral {
            lit_offset: cpu.fetch_and_advance(),
            reg: Register8::A,
        },
        0xE1 => Instruction::PopReg16 {
            reg: Register16::HL,
        },
        0xE2 => Instruction::WriteReg8ValueAtZeroPageOffsetReg8 {
            reg_offset: Register8::C,
            reg: Register8::A,
        },
        // 0xE3 => nothing
        // 0xE4 => nothing
        0xE5 => Instruction::PushReg16 {
            reg: Register16::HL,
        },
        0xE6 => Instruction::AndAWithLiteral {
            literal: cpu.fetch_and_advance(),
        },
        0xE7 => Instruction::Reset { offset: 0x20 },
        0xE8 => Instruction::AddOffsetToReg16 {
            reg: Register16::SP,
            offset: cpu.fetch_and_advance() as i8,
        },
        0xE9 => Instruction::JumpRegister16 {
            reg: Register16::HL,
        },
        0xEA => Instruction::WriteReg8ValueAtAddress {
            addr: cpu.fetch_and_advance_u16(),
            reg: Register8::A,
        },
        // 0xEB => nothing
        // 0xEC => nothing
        // 0xED => nothing
        0xEE => Instruction::XorAWithLiteral {
            literal: cpu.fetch_and_advance(),
        },
        0xEF => Instruction::Reset { offset: 0x28 },
        0xF0 => Instruction::ReadZeroPageOffsetLiteralToReg8 {
            reg: Register8::A,
            lit_offset: cpu.fetch_and_advance(),
        },
        0xF1 => Instruction::PopReg16 {
            reg: Register16::AF,
        },
        0xF2 => Instruction::ReadZeroPageOffsetReg8ToReg8 {
            offset: Register8::C,
            reg: Register8::A,
        },
        0xF3 => Instruction::DisableInterrupts,
        // 0xF4 => nothing
        0xF5 => Instruction::PushReg16 {
            reg: Register16::AF,
        },
        0xF6 => Instruction::OrAWithLiteral {
            literal: cpu.fetch_and_advance(),
        },
        0xF7 => Instruction::Reset { offset: 0x30 },
        0xF8 => Instruction::LoadAddressOffsetIntoReg16 {
            dest: Register16::HL,
            base: Register16::SP,
            offset: cpu.fetch_and_advance() as i8,
        },
        0xF9 => Instruction::Move16Bits {
            dest: Register16::SP,
            src: Register16::HL,
        },
        0xFA => Instruction::ReadAtAddressToReg8 {
            addr: cpu.fetch_and_advance_u16(),
            reg: Register8::A,
        },
        0xFB => Instruction::EnableInterrupts,
        // 0xFC => nothing
        // 0xFD => nothing
        0xFE => Instruction::CompareAWithLiteral {
            literal: cpu.fetch_and_advance(),
        },
        0xFF => Instruction::Reset { offset: 0x38 },
        _ => panic!("Unknown opcode {:#x} at {:#x}", opcode, pc),
    }
}

fn decode_reg_bit_ops(reg_opcode: u8) -> (Reg8OrIndirect, u8) {
    match reg_opcode {
        0x0 => (Register8::B.into(), 0),
        0x1 => (Register8::C.into(), 0),
        0x2 => (Register8::D.into(), 0),
        0x3 => (Register8::E.into(), 0),
        0x4 => (Register8::H.into(), 0),
        0x5 => (Register8::L.into(), 0),
        0x6 => (Register16::HL.into(), 0),
        0x7 => (Register8::A.into(), 0),

        0x8 => (Register8::B.into(), 1),
        0x9 => (Register8::C.into(), 1),
        0xA => (Register8::D.into(), 1),
        0xB => (Register8::E.into(), 1),
        0xC => (Register8::H.into(), 1),
        0xD => (Register8::L.into(), 1),
        0xE => (Register16::HL.into(), 1),
        0xF => (Register8::A.into(), 1),
        _ => unreachable!(),
    }
}

fn decode_bit_test(opcode: u8) -> Instruction {
    let bit_opcode = opcode >> 4;
    let reg_opcode = opcode & 0xF;

    let (reg, offset) = decode_reg_bit_ops(reg_opcode);
    let bit = match bit_opcode {
        0x4 => 0,
        0x5 => 2,
        0x6 => 4,
        0x7 => 6,
        _ => unimplemented!(),
    } + offset;

    match reg {
        Reg8OrIndirect::Reg8(reg) => Instruction::BitTest { reg, bit },
        Reg8OrIndirect::Indirect(_) => {
            unimplemented!()
        }
    }
}

fn decode_reset_bit(opcode: u8) -> Instruction {
    let bit_opcode = opcode >> 4;
    let reg_opcode = opcode & 0xF;

    let (reg, offset) = decode_reg_bit_ops(reg_opcode);
    let bit = match bit_opcode {
        0x8 => 0,
        0x9 => 2,
        0xA => 4,
        0xB => 6,
        _ => unimplemented!(),
    } + offset;

    match reg {
        Reg8OrIndirect::Reg8(reg) => Instruction::ResetBit { reg, bit },
        Reg8OrIndirect::Indirect(_) => {
            unimplemented!()
        }
    }
}

fn decode_set_bit(opcode: u8) -> Instruction {
    let bit_opcode = opcode >> 4;
    let reg_opcode = opcode & 0xF;

    let (reg, offset) = decode_reg_bit_ops(reg_opcode);
    let bit = match bit_opcode {
        0xC => 0,
        0xD => 2,
        0xE => 4,
        0xF => 6,
        _ => unimplemented!(),
    } + offset;

    match reg {
        Reg8OrIndirect::Reg8(reg) => Instruction::SetBit { reg, bit },
        Reg8OrIndirect::Indirect(_) => {
            unimplemented!()
        }
    }
}
