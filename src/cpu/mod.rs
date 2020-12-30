use crate::{memory::Memory, utils::combine};
use bitflags::bitflags;
use std::collections::VecDeque;

mod instruction;
mod micro_op;
mod register;

use instruction::{Instruction, JumpCondition};
use log::{debug, info};
use micro_op::{Destination8Bits, MicroOp, Source8bits};
use register::{Register16, Register8};

use self::instruction::PrePostOperation;

bitflags! {
    struct Flags: u8 {
        const ZERO       = 1 << 7;
        const NEGATIVE   = 1 << 6;
        const HALF_CARRY = 1 << 5;
        const CARRY      = 1 << 4;
    }
}

#[derive(Debug, Clone)]
pub struct CPU<M: Memory> {
    memory: M,

    reg_a: u8,
    reg_b: u8,
    reg_c: u8,
    reg_d: u8,
    reg_e: u8,
    reg_h: u8,
    reg_l: u8,

    flags: Flags,

    sp: u16,
    pub pc: u16,

    pipeline: VecDeque<MicroOp>,
}

impl<M: Memory> CPU<M> {
    pub fn new(memory: M) -> Self {
        CPU {
            memory,
            reg_a: 0,
            reg_b: 0,
            reg_c: 0,
            reg_d: 0,
            reg_e: 0,
            reg_h: 0,
            reg_l: 0,
            sp: 0,
            pc: 0,
            flags: Flags::empty(),
            pipeline: VecDeque::new(),
        }
    }

    pub fn load_reg8(&self, reg: Register8) -> u8 {
        match reg {
            Register8::A => self.reg_a,
            Register8::B => self.reg_b,
            Register8::C => self.reg_c,
            Register8::D => self.reg_d,
            Register8::E => self.reg_e,
            Register8::H => self.reg_h,
            Register8::L => self.reg_l,
            Register8::SPHigh => (self.sp >> 8) as u8,
            Register8::SPLow => self.sp as u8,
            Register8::PCHigh => (self.pc >> 8) as u8,
            Register8::PCLow => self.pc as u8,
            Register8::Flags => self.flags.bits(),
        }
    }

    pub fn store_reg8(&mut self, reg: Register8, value: u8) {
        match reg {
            Register8::A => self.reg_a = value,
            Register8::B => self.reg_b = value,
            Register8::C => self.reg_c = value,
            Register8::D => self.reg_d = value,
            Register8::E => self.reg_e = value,
            Register8::H => self.reg_h = value,
            Register8::L => self.reg_l = value,
            Register8::SPHigh => {
                self.sp = (self.sp & 0x00FF) | ((value as u16) << 8);
            }
            Register8::SPLow => {
                self.sp = (self.sp & 0xFF00) | (value as u16);
            }
            Register8::PCHigh => {
                self.pc = (self.pc & 0x00FF) | ((value as u16) << 8);
            }
            Register8::PCLow => {
                self.pc = (self.pc & 0xFF00) | (value as u16);
            }
            Register8::Flags => self.flags = Flags::from_bits_truncate(value),
        }
    }

    pub fn load_reg16(&self, reg: Register16) -> u16 {
        match reg {
            Register16::AF => combine(self.reg_a, self.flags.bits()),
            Register16::BC => combine(self.reg_b, self.reg_c),
            Register16::DE => combine(self.reg_d, self.reg_e),
            Register16::HL => combine(self.reg_h, self.reg_l),
            Register16::SP => self.sp,
            Register16::PC => self.pc,
        }
    }

    pub fn store_reg16(&mut self, reg: Register16, value: u16) {
        match reg {
            Register16::AF => {
                self.reg_a = (value >> 8) as u8;
                self.flags = Flags::from_bits_truncate(value as u8);
            }
            Register16::BC => {
                self.reg_b = (value >> 8) as u8;
                self.reg_c = value as u8;
            }
            Register16::DE => {
                self.reg_d = (value >> 8) as u8;
                self.reg_e = value as u8;
            }
            Register16::HL => {
                self.reg_h = (value >> 8) as u8;
                self.reg_l = value as u8;
            }
            Register16::SP => self.sp = value,
            Register16::PC => self.pc = value,
        }
    }

    pub fn fetch_and_advance(&mut self) -> u8 {
        let byte = self.memory.read_memory(self.pc);
        self.pc += 1;
        byte
    }

    pub fn fetch_and_advance_u16(&mut self) -> u16 {
        let low = self.fetch_and_advance();
        let high = self.fetch_and_advance();
        ((high as u16) << 8) | (low as u16)
    }

    pub fn fetch_and_decode(&mut self) -> Instruction {
        let pc = self.pc;
        let opcode = self.fetch_and_advance();

        match opcode {
            0x00 => Instruction::NOP,
            0x01 => Instruction::LoadLiteralIntoReg16 {
                reg: Register16::BC,
                literal: self.fetch_and_advance_u16(),
            },
            0x03 => Instruction::IncReg16 {
                reg: Register16::BC,
            },
            0x04 => Instruction::IncReg8 { reg: Register8::B },
            0x05 => Instruction::DecReg8 { reg: Register8::B },
            0x06 => Instruction::LoadLiteralIntoReg8 {
                reg: Register8::B,
                literal: self.fetch_and_advance(),
            },
            0x08 => Instruction::WriteReg16ValueAtAddress {
                addr: self.fetch_and_advance_u16(),
                reg: Register16::SP,
            },
            0x0C => Instruction::IncReg8 { reg: Register8::C },
            0x0D => Instruction::DecReg8 { reg: Register8::C },
            0x0E => Instruction::LoadLiteralIntoReg8 {
                reg: Register8::C,
                literal: self.fetch_and_advance(),
            },
            0x11 => Instruction::LoadLiteralIntoReg16 {
                reg: Register16::DE,
                literal: self.fetch_and_advance_u16(),
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
                literal: self.fetch_and_advance(),
            },
            0x17 => Instruction::RotateLeftThroughCarryA,
            0x18 => Instruction::JumpRelative {
                condition: None,
                offset: self.fetch_and_advance() as i8,
            },
            0x1A => Instruction::ReadIndirectToReg8 {
                addr: Register16::DE,
                reg: Register8::A,
                post_op: None,
            },
            0x1C => Instruction::IncReg8 { reg: Register8::E },
            0x1D => Instruction::DecReg8 { reg: Register8::E },
            0x1E => Instruction::LoadLiteralIntoReg8 {
                reg: Register8::E,
                literal: self.fetch_and_advance(),
            },
            0x1F => Instruction::RotateRightThroughCarryA,
            0x20 => Instruction::JumpRelative {
                condition: Some(JumpCondition::NonZero),
                offset: self.fetch_and_advance() as i8,
            },
            0x21 => Instruction::LoadLiteralIntoReg16 {
                reg: Register16::HL,
                literal: self.fetch_and_advance_u16(),
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
                literal: self.fetch_and_advance(),
            },
            0x27 => Instruction::DAA,
            0x28 => Instruction::JumpRelative {
                condition: Some(JumpCondition::Zero),
                offset: self.fetch_and_advance() as i8,
            },
            0x2A => Instruction::ReadIndirectToReg8 {
                reg: Register8::A,
                addr: Register16::HL,
                post_op: Some(PrePostOperation::Inc),
            },
            0x2C => Instruction::IncReg8 { reg: Register8::L },
            0x2D => Instruction::DecReg8 { reg: Register8::L },
            0x2E => Instruction::LoadLiteralIntoReg8 {
                reg: Register8::L,
                literal: self.fetch_and_advance(),
            },
            0x2F => Instruction::ComplementA,
            0x29 => Instruction::AddHLWithReg {
                reg: Register16::HL,
            },
            0x30 => Instruction::JumpRelative {
                condition: Some(JumpCondition::NonCarry),
                offset: self.fetch_and_advance() as i8,
            },
            0x31 => Instruction::LoadLiteralIntoReg16 {
                reg: Register16::SP,
                literal: self.fetch_and_advance_u16(),
            },
            0x32 => Instruction::WriteReg8ValueAtIndirect {
                addr: Register16::HL,
                reg: Register8::A,
                post_op: Some(PrePostOperation::Dec),
            },
            0x35 => Instruction::DecIndirect {
                addr: Register16::HL,
            },
            0x36 => Instruction::WriteLiteralAtIndirect {
                addr: Register16::HL,
                literal: self.fetch_and_advance(),
            },
            0x38 => Instruction::JumpRelative {
                condition: Some(JumpCondition::Carry),
                offset: self.fetch_and_advance() as i8,
            },
            0x3C => Instruction::IncReg8 { reg: Register8::A },
            0x3D => Instruction::DecReg8 { reg: Register8::A },
            0x3E => Instruction::LoadLiteralIntoReg8 {
                reg: Register8::A,
                literal: self.fetch_and_advance(),
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
            0x4E => Instruction::ReadIndirectToReg8 {
                reg: Register8::C,
                addr: Register16::HL,
                post_op: None,
            },
            0x4F => Instruction::Move {
                dest: Register8::C,
                src: Register8::A,
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
            0x66 => Instruction::ReadIndirectToReg8 {
                addr: Register16::HL,
                reg: Register8::H,
                post_op: None,
            },
            0x67 => Instruction::Move {
                dest: Register8::H,
                src: Register8::A,
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
            0x86 => Instruction::AddAWithIndirect {
                addr: Register16::HL,
            },
            0x90 => Instruction::SubAWithReg8 { reg: Register8::B },
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
            0xB1 => Instruction::OrAWithReg8 { reg: Register8::C },
            0xBB => Instruction::CompareAWithReg { reg: Register8::E },
            0xBE => Instruction::CompareAWithIndirect {
                addr: Register16::HL,
            },
            0xB6 => Instruction::OrAWithIndirect {
                addr: Register16::HL,
            },
            0xB7 => Instruction::OrAWithReg8 { reg: Register8::A },
            0xB8 => Instruction::CompareAWithReg { reg: Register8::B },
            0xB9 => Instruction::CompareAWithReg { reg: Register8::C },
            0xBA => Instruction::CompareAWithReg { reg: Register8::D },
            0xC1 => Instruction::PopReg16 {
                reg: Register16::BC,
            },
            0xC2 => Instruction::JumpAbsolute {
                condition: Some(JumpCondition::NonZero),
                addr: self.fetch_and_advance_u16(),
            },
            0xC3 => Instruction::JumpAbsolute {
                condition: None,
                addr: self.fetch_and_advance_u16(),
            },
            0xC4 => Instruction::CallAddr {
                condition: Some(JumpCondition::NonZero),
                addr: self.fetch_and_advance_u16(),
            },
            0xC5 => Instruction::PushReg16 {
                reg: Register16::BC,
            },
            0xC6 => Instruction::AddAWithLiteral {
                literal: self.fetch_and_advance(),
            },
            0xC8 => Instruction::Return {
                condition: Some(JumpCondition::Zero),
            },
            0xC9 => Instruction::Return { condition: None },
            0xCB => {
                // prefix 0xCB:
                match self.fetch_and_advance() {
                    0x11 => Instruction::RotateLeftThroughCarry { reg: Register8::C },
                    0x18 => Instruction::RotateRightThroughCarry { reg: Register8::B },
                    0x19 => Instruction::RotateRightThroughCarry { reg: Register8::C },
                    0x1A => Instruction::RotateRightThroughCarry { reg: Register8::D },
                    0x1B => Instruction::RotateRightThroughCarry { reg: Register8::E },
                    0x1C => Instruction::RotateRightThroughCarry { reg: Register8::H },
                    0x1D => Instruction::RotateRightThroughCarry { reg: Register8::L },
                    0x37 => Instruction::SwapReg8 { reg: Register8::A },
                    0x38 => Instruction::ShiftRightIntoCarry { reg: Register8::B },
                    0x7C => Instruction::BitTest {
                        reg: Register8::H,
                        bit: 7,
                    },
                    other => panic!("Unknown sub-opcode (prefix 0xCB) {:#x}", other),
                }
            }
            0xCD => Instruction::CallAddr {
                condition: None,
                addr: self.fetch_and_advance_u16(),
            },
            0xCE => Instruction::AdcAWithLiteral {
                literal: self.fetch_and_advance(),
            },
            0xD0 => Instruction::Return {
                condition: Some(JumpCondition::NonCarry),
            },
            0xD1 => Instruction::PopReg16 {
                reg: Register16::DE,
            },
            0xD5 => Instruction::PushReg16 {
                reg: Register16::DE,
            },
            0xD6 => Instruction::SubAWithLiteral {
                literal: self.fetch_and_advance(),
            },
            0xD8 => Instruction::Return {
                condition: Some(JumpCondition::Carry),
            },
            0xE0 => Instruction::WriteReg8ValueAtZeroPageOffsetLiteral {
                lit_offset: self.fetch_and_advance(),
                reg: Register8::A,
            },
            0xE1 => Instruction::PopReg16 {
                reg: Register16::HL,
            },
            0xE2 => Instruction::WriteReg8ValueAtZeroPageOffsetReg8 {
                reg_offset: Register8::C,
                reg: Register8::A,
            },
            0xE5 => Instruction::PushReg16 {
                reg: Register16::HL,
            },
            0xE6 => Instruction::AndAWithLiteral {
                literal: self.fetch_and_advance(),
            },
            0xE9 => Instruction::JumpRegister16 {
                reg: Register16::HL,
            },
            0xEA => Instruction::WriteReg8ValueAtAddress {
                addr: self.fetch_and_advance_u16(),
                reg: Register8::A,
            },
            0xEE => Instruction::XorAWithLiteral {
                literal: self.fetch_and_advance(),
            },
            0xF0 => Instruction::ReadZeroPageOffsetLiteralToReg8 {
                reg: Register8::A,
                lit_offset: self.fetch_and_advance(),
            },
            0xF1 => Instruction::PopReg16 {
                reg: Register16::AF,
            },
            0xF3 => Instruction::DisableInterrupts,
            0xF5 => Instruction::PushReg16 {
                reg: Register16::AF,
            },
            0xF9 => Instruction::Move16Bits {
                dest: Register16::SP,
                src: Register16::HL,
            },
            0xFA => Instruction::ReadAtAddressToReg8 {
                addr: self.fetch_and_advance_u16(),
                reg: Register8::A,
            },
            0xFB => Instruction::EnableInterrupts,
            0xFE => Instruction::CompareAWithLiteral {
                literal: self.fetch_and_advance(),
            },
            _ => panic!("Unknown opcode {:#x} at {:#x}", opcode, pc),
        }
    }

    fn run_pre_post_op(&mut self, reg16: Register16, op: Option<PrePostOperation>) {
        match op {
            Some(PrePostOperation::Dec) => {
                self.store_reg16(reg16, self.load_reg16(reg16).wrapping_sub(1))
            }
            Some(PrePostOperation::Inc) => {
                self.store_reg16(reg16, self.load_reg16(reg16).wrapping_add(1))
            }
            None => {}
        }
    }

    fn source_8bits_to_value(&self, src: Source8bits) -> u8 {
        match src {
            Source8bits::Register(reg) => self.load_reg8(reg),
            Source8bits::Literal(lit) => lit,
            Source8bits::Indirect(addr) => self.memory.read_memory(self.load_reg16(addr)),
            Source8bits::Address(addr) => self.memory.read_memory(addr),
        }
    }

    pub fn step(&mut self) {
        if self.pipeline.is_empty() {
            let instruction = self.fetch_and_decode();
            debug!("{:#06x}: {}", self.pc, instruction);
            self.pipeline.extend(instruction.to_micro_ops());
        }

        if let Some(micro_op) = self.pipeline.pop_front() {
            match micro_op {
                MicroOp::NOP => {}
                MicroOp::Move8Bits {
                    destination,
                    source,
                } => {
                    let value = self.source_8bits_to_value(source);

                    match destination {
                        Destination8Bits::Register(reg) => {
                            self.store_reg8(reg, value);
                        }
                        Destination8Bits::Indirect(addr) => {
                            self.memory.write_memory(self.load_reg16(addr), value);
                        }
                        Destination8Bits::Address(addr) => {
                            self.memory.write_memory(addr, value);
                        }
                    }
                }
                MicroOp::Move16Bits {
                    destination,
                    source,
                } => {
                    self.store_reg16(destination, self.load_reg16(source));
                }
                MicroOp::LoadReg16Lit { reg, literal } => {
                    self.store_reg16(reg, literal);
                }
                MicroOp::AndA { rhs } => {
                    self.reg_a &= self.source_8bits_to_value(rhs);
                    self.flags = Flags::HALF_CARRY;
                    if self.reg_a == 0 {
                        self.flags |= Flags::ZERO;
                    }
                }
                MicroOp::OrA { rhs } => {
                    self.reg_a |= self.source_8bits_to_value(rhs);
                    self.flags = if self.reg_a == 0 {
                        Flags::ZERO
                    } else {
                        Flags::empty()
                    };
                }
                MicroOp::XorA { rhs } => {
                    self.reg_a ^= self.source_8bits_to_value(rhs);
                    self.flags = if self.reg_a == 0 {
                        Flags::ZERO
                    } else {
                        Flags::empty()
                    };
                }
                MicroOp::AddA { rhs } => {
                    let a_value = self.reg_a;
                    let rhs_value = self.source_8bits_to_value(rhs);

                    let (res, carry) = a_value.overflowing_add(rhs_value);
                    let half_carry = check_half_carry(a_value, rhs_value);

                    self.reg_a = res;
                    self.update_flags_arith(res, false, carry, half_carry);
                }
                MicroOp::AddHL { rhs } => {
                    let hl_value = self.load_reg16(Register16::HL);
                    let rhs_value = self.load_reg16(rhs);

                    let (res, carry) = hl_value.overflowing_add(rhs_value);
                    let half_carry = check_half_carry_16bits(hl_value, rhs_value);

                    self.store_reg16(Register16::HL, res);

                    self.flags = self.flags & Flags::ZERO;
                    if half_carry {
                        self.flags |= Flags::HALF_CARRY;
                    }
                    if carry {
                        self.flags |= Flags::CARRY;
                    }
                }
                MicroOp::AdcA { rhs } => {
                    let a_value = self.reg_a;
                    let rhs_value = self.source_8bits_to_value(rhs);

                    let (mut res, mut carry) = a_value.overflowing_add(rhs_value);
                    let mut half_carry = check_half_carry(a_value, rhs_value);

                    if self.flags.contains(Flags::CARRY) {
                        let (res_carry, carry2) = res.overflowing_add(1);
                        let half_carry2 = check_half_carry(res, 1);

                        res = res_carry;
                        carry |= carry2;
                        half_carry |= half_carry2;
                    }

                    self.reg_a = res;
                    self.update_flags_arith(res, false, carry, half_carry);
                }
                MicroOp::SubA { rhs } => {
                    let a_value = self.reg_a;
                    let rhs_value = self.source_8bits_to_value(rhs);

                    let (res, carry) = a_value.overflowing_sub(rhs_value);
                    let half_carry = check_half_carry_sub(a_value, rhs_value);

                    self.reg_a = res;
                    self.update_flags_arith(res, true, carry, half_carry);
                }
                MicroOp::DAA => {
                    let mut a = self.reg_a as u32;

                    if !self.flags.contains(Flags::NEGATIVE) {
                        if self.flags.contains(Flags::HALF_CARRY) || (a & 0xF) > 9 {
                            a += 0x06;
                        }
                        if self.flags.contains(Flags::CARRY) || a > 0x9F {
                            a += 0x60;
                        }
                    } else {
                        if self.flags.contains(Flags::HALF_CARRY) {
                            a = (a - 6) & 0xFF;
                        }
                        if self.flags.contains(Flags::CARRY) {
                            a -= 0x60;
                        }
                    }

                    self.flags.remove(Flags::HALF_CARRY | Flags::ZERO);

                    if (a & 0x100) == 0x100 {
                        self.flags |= Flags::CARRY;
                    }

                    a &= 0xFF;
                    if a == 0 {
                        self.flags |= Flags::ZERO;
                    }
                    self.reg_a = a as u8;
                }
                MicroOp::ComplementA => {
                    self.reg_a = !self.reg_a;
                    self.flags |= Flags::NEGATIVE | Flags::HALF_CARRY;
                }
                MicroOp::WriteMem {
                    addr,
                    reg,
                    pre_op,
                    post_op,
                } => {
                    self.run_pre_post_op(addr, pre_op);
                    let addr_value = self.load_reg16(addr);
                    self.memory.write_memory(addr_value, self.load_reg8(reg));
                    self.run_pre_post_op(addr, post_op);
                }
                MicroOp::WriteMemZeroPage { reg_offset, reg } => {
                    let addr_value = 0xFF00 + self.load_reg8(reg_offset) as u16;
                    self.memory.write_memory(addr_value, self.load_reg8(reg));
                }
                MicroOp::ReadMem { reg, addr, post_op } => {
                    let addr_value = self.load_reg16(addr);
                    let mem_value = self.memory.read_memory(addr_value);
                    self.store_reg8(reg, mem_value);
                    self.run_pre_post_op(addr, post_op);
                }
                MicroOp::BitTest { reg, bit } => {
                    let is_set = (self.load_reg8(reg) >> bit) & 1 == 1;
                    let rest = Flags::HALF_CARRY | (self.flags & Flags::CARRY);
                    self.flags = if is_set { rest } else { Flags::ZERO | rest };
                }
                MicroOp::CheckFlags {
                    condition,
                    true_ops,
                    false_ops,
                } => {
                    let cond_true = match condition {
                        instruction::JumpCondition::NonZero => !self.flags.contains(Flags::ZERO),
                        instruction::JumpCondition::Zero => self.flags.contains(Flags::ZERO),
                        instruction::JumpCondition::NonCarry => !self.flags.contains(Flags::CARRY),
                        instruction::JumpCondition::Carry => self.flags.contains(Flags::CARRY),
                    };

                    let to_prepend_ops = if cond_true { true_ops } else { false_ops };
                    for op in to_prepend_ops.into_iter().rev() {
                        self.pipeline.push_front(op);
                    }
                }
                MicroOp::RelativeJump(offset) => {
                    if offset < 0 {
                        self.pc = self.pc.wrapping_sub((-offset) as u16);
                    } else {
                        self.pc = self.pc.wrapping_add(offset as u16);
                    }
                }
                MicroOp::IncReg16 { reg } => {
                    // No flags change for this micro op
                    self.store_reg16(reg, self.load_reg16(reg).wrapping_add(1));
                }
                MicroOp::IncReg { reg } => {
                    let reg_value = self.load_reg8(reg);
                    let half_carry = check_half_carry(reg_value, 1);
                    let new_value = reg_value.wrapping_add(1);
                    self.store_reg8(reg, new_value);
                    let mut flags = Flags::empty();
                    if new_value == 0 {
                        flags |= Flags::ZERO;
                    }
                    if half_carry {
                        flags |= Flags::HALF_CARRY;
                    }
                    flags |= self.flags & Flags::CARRY;
                    self.flags = flags;
                }
                MicroOp::DecReg { reg } => {
                    let reg_value = self.load_reg8(reg);
                    let half_carry = check_half_carry_sub(reg_value, 1);
                    let new_value = reg_value.wrapping_sub(1);
                    self.store_reg8(reg, new_value);

                    self.update_flags_arith(
                        new_value,
                        true,
                        self.flags.contains(Flags::CARRY),
                        half_carry,
                    );
                }
                MicroOp::DecIndirect { addr } => {
                    let addr = self.load_reg16(addr);
                    let start_value = self.memory.read_memory(addr);
                    let half_carry = check_half_carry_sub(start_value, 1);
                    let new_value = start_value.wrapping_sub(1);
                    self.memory.write_memory(addr, new_value);

                    self.update_flags_arith(
                        new_value,
                        true,
                        self.flags.contains(Flags::CARRY),
                        half_carry,
                    );
                }
                MicroOp::CompareA { rhs } => {
                    let value = self.source_8bits_to_value(rhs);
                    self.compare_a(value);
                }
                MicroOp::RotateLeftThroughCarry { reg, set_zero } => {
                    let value = self.load_reg8(reg);
                    let new_carry = (value >> 7) == 1;
                    let new_value = (value << 1) | (self.flags.contains(Flags::CARRY) as u8);
                    self.store_reg8(reg, new_value);

                    self.flags = Flags::empty();
                    if new_carry {
                        self.flags |= Flags::CARRY;
                    }
                    if set_zero && new_value == 0 {
                        self.flags |= Flags::ZERO;
                    }
                }
                MicroOp::RotateRightThroughCarry { reg, set_zero } => {
                    let value = self.load_reg8(reg);
                    let new_carry = (value & 0x1) == 1;
                    let new_value = ((self.flags.contains(Flags::CARRY) as u8) << 7) | (value >> 1);
                    self.store_reg8(reg, new_value);

                    self.flags = Flags::empty();
                    if new_carry {
                        self.flags |= Flags::CARRY;
                    }
                    if set_zero && new_value == 0 {
                        self.flags |= Flags::ZERO;
                    }
                }
                MicroOp::ShiftRightIntoCarry { reg } => {
                    let value = self.load_reg8(reg);
                    let carry = (value & 0x1) == 1;
                    let new_value = value >> 1;

                    self.store_reg8(reg, new_value);
                    self.flags = Flags::empty();
                    if carry {
                        self.flags |= Flags::CARRY;
                    }
                    if new_value == 0 {
                        self.flags |= Flags::ZERO;
                    }
                }
                MicroOp::SwapReg8 { reg } => {
                    let value = self.load_reg8(reg);
                    let high = (value & 0xF0) >> 4;
                    let low = value & 0x0F;
                    let res = (low << 4) | high;
                    self.store_reg8(reg, res);
                    self.flags = Flags::empty();
                    if res == 0 {
                        self.flags |= Flags::ZERO;
                    }
                }
                MicroOp::EnableInterrupts => {
                    info!("Enable interrupts")
                }
                MicroOp::DisableInterrupts => {
                    info!("Disable interrupts")
                }
            }
        }
    }

    fn compare_a(&mut self, with: u8) {
        let a_value = self.reg_a;
        let (res, carry) = a_value.overflowing_sub(with);

        let mut flags = Flags::NEGATIVE;
        if res == 0 {
            flags |= Flags::ZERO;
        }

        if check_half_carry_sub(a_value, with) {
            flags |= Flags::HALF_CARRY;
        }

        if carry {
            flags |= Flags::CARRY;
        }
        self.flags = flags;
    }

    fn update_flags_arith(&mut self, res: u8, negative: bool, carry: bool, half_carry: bool) {
        let mut flags = Flags::empty();
        if res == 0 {
            flags |= Flags::ZERO;
        }

        if negative {
            flags |= Flags::NEGATIVE;
        }

        if carry {
            flags |= Flags::CARRY;
        }

        if half_carry {
            flags |= Flags::HALF_CARRY;
        }

        self.flags = flags;
    }
}

fn check_half_carry(a: u8, b: u8) -> bool {
    (((a & 0xf) + (b & 0xf)) & 0x10) == 0x10
}

fn check_half_carry_16bits(a: u16, b: u16) -> bool {
    (((a & 0xFF) + (b & 0xFF)) & 0x100) == 0x100
}

fn check_half_carry_sub(a: u8, b: u8) -> bool {
    let neg_b = u8::MAX.wrapping_sub(b).wrapping_add(1);
    check_half_carry(a, neg_b)
}
