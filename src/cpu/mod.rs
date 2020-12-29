use crate::{memory::Memory, utils::combine};
use bitflags::bitflags;
use std::collections::VecDeque;

mod instruction;
mod micro_op;
mod register;

use instruction::{Instruction, JumpCondition};
use log::{debug, info};
use micro_op::{MicroOp, Move8BitsDestination, Move8BitsSource};
use register::{Register16, Register8};

use self::instruction::PrePostOperation;

bitflags! {
    struct Flags: u8 {
        const ZERO       = 1 << 7;
        const NEGATIVE    = 1 << 6;
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
            Register8::Flags => {
                self.flags = Flags::from_bits(value).expect("Failed to parse flags from bits")
            }
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
                self.flags =
                    Flags::from_bits(value as u8).expect("Failed to parse flags from bits");
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
            0x28 => Instruction::JumpRelative {
                condition: Some(JumpCondition::Zero),
                offset: self.fetch_and_advance() as i8,
            },
            0x2A => Instruction::ReadIndirectToReg8 {
                reg: Register8::A,
                addr: Register16::HL,
                post_op: Some(PrePostOperation::Inc),
            },
            0x2E => Instruction::LoadLiteralIntoReg8 {
                reg: Register8::L,
                literal: self.fetch_and_advance(),
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
            0x36 => Instruction::WriteLiteralAtIndirect {
                addr: Register16::HL,
                literal: self.fetch_and_advance(),
            },
            0x3D => Instruction::DecReg8 { reg: Register8::A },
            0x3E => Instruction::LoadLiteralIntoReg8 {
                reg: Register8::A,
                literal: self.fetch_and_advance(),
            },
            0x47 => Instruction::Move {
                dest: Register8::B,
                src: Register8::A,
            },
            0x4F => Instruction::Move {
                dest: Register8::C,
                src: Register8::A,
            },
            0x57 => Instruction::Move {
                dest: Register8::D,
                src: Register8::A,
            },
            0x67 => Instruction::Move {
                dest: Register8::H,
                src: Register8::A,
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
            0xAF => Instruction::XorAWithReg8 { reg: Register8::A },
            0xB1 => Instruction::OrAWithReg8 { reg: Register8::C },
            0xBE => Instruction::CompareAWithIndirect {
                addr: Register16::HL,
            },
            0xC1 => Instruction::PopReg16 {
                reg: Register16::BC,
            },
            0xC3 => Instruction::JumpAbsolute {
                addr: self.fetch_and_advance_u16(),
            },
            0xC5 => Instruction::PushReg16 {
                reg: Register16::BC,
            },
            0xC9 => Instruction::Return,
            0xCB => {
                // prefix 0xCB:
                match self.fetch_and_advance() {
                    0x11 => Instruction::RotateLeftThroughCarry { reg: Register8::C },
                    0x7C => Instruction::BitTest {
                        reg: Register8::H,
                        bit: 7,
                    },
                    other => panic!("Unknown sub-opcode (prefix 0xCB) {:#x}", other),
                }
            }
            0xCD => Instruction::CallAddr {
                addr: self.fetch_and_advance_u16(),
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
            0xEA => Instruction::WriteReg8ValueAtAddress {
                addr: self.fetch_and_advance_u16(),
                reg: Register8::A,
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
                    let value = match source {
                        Move8BitsSource::Register(reg) => self.load_reg8(reg),
                        Move8BitsSource::Literal(lit) => lit,
                        Move8BitsSource::Indirect(addr) => {
                            self.memory.read_memory(self.load_reg16(addr))
                        }
                        Move8BitsSource::Address(addr) => self.memory.read_memory(addr),
                    };

                    match destination {
                        Move8BitsDestination::Register(reg) => {
                            self.store_reg8(reg, value);
                        }
                        Move8BitsDestination::Indirect(addr) => {
                            self.memory.write_memory(self.load_reg16(addr), value);
                        }
                        Move8BitsDestination::Address(addr) => {
                            self.memory.write_memory(addr, value);
                        }
                    }
                }
                MicroOp::LoadReg16Lit { reg, literal } => {
                    self.store_reg16(reg, literal);
                }
                MicroOp::OrAReg { reg } => {
                    self.reg_a |= self.load_reg8(reg);
                    self.flags = if self.reg_a == 0 {
                        Flags::ZERO
                    } else {
                        Flags::empty()
                    };
                }
                MicroOp::XorAReg { reg } => {
                    self.reg_a ^= self.load_reg8(reg);
                    self.flags = if self.reg_a == 0 {
                        Flags::ZERO
                    } else {
                        Flags::empty()
                    };
                }
                MicroOp::AddAIndirect { addr } => {
                    let a_value = self.reg_a;
                    let rhs_value = self.memory.read_memory(self.load_reg16(addr));

                    let (res, carry) = a_value.overflowing_add(rhs_value);
                    let half_carry = check_half_carry(a_value, rhs_value);

                    self.reg_a = res;
                    self.update_flags_arith(res, false, carry, half_carry);
                }
                MicroOp::SubAReg { reg } => {
                    let a_value = self.reg_a;
                    let rhs_value = self.load_reg8(reg);

                    let (res, carry) = a_value.overflowing_sub(rhs_value);
                    let half_carry = check_half_carry_sub(a_value, rhs_value);

                    self.reg_a = res;
                    self.update_flags_arith(res, true, carry, half_carry);
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
                MicroOp::CheckFlags(condition) => {
                    let cond_true = match condition {
                        instruction::JumpCondition::NonZero => !self.flags.contains(Flags::ZERO),
                        instruction::JumpCondition::Zero => self.flags.contains(Flags::ZERO),
                        instruction::JumpCondition::NonCarry => !self.flags.contains(Flags::CARRY),
                        instruction::JumpCondition::Carry => self.flags.contains(Flags::CARRY),
                    };

                    if !cond_true {
                        self.pipeline.pop_front();
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
                    let mut flags = Flags::NEGATIVE;
                    if new_value == 0 {
                        flags |= Flags::ZERO;
                    }
                    if half_carry {
                        flags |= Flags::HALF_CARRY;
                    }
                    flags |= self.flags & Flags::CARRY;
                    self.flags = flags;
                }
                MicroOp::CompareALit { literal } => {
                    self.compare_a(literal);
                }
                MicroOp::CompareAIndirect { addr } => {
                    self.compare_a(self.memory.read_memory(self.load_reg16(addr)));
                }
                MicroOp::RotateLeftThroughCarry { reg, set_zero } => {
                    let value = self.load_reg8(reg);
                    let new_carry = (value >> 7) == 1;
                    let new_value = (value << 1) | (self.flags.contains(Flags::CARRY) as u8);
                    self.store_reg8(reg, new_value);

                    let mut flags = Flags::empty();
                    if new_carry {
                        flags |= Flags::CARRY;
                    }
                    if set_zero && new_value == 0 {
                        flags |= Flags::ZERO;
                    }
                    self.flags = flags;
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

fn check_half_carry_sub(a: u8, b: u8) -> bool {
    let neg_b = u8::MAX.wrapping_sub(b).wrapping_add(1);
    check_half_carry(a, neg_b)
}
