use crate::{memory::MMU, utils::combine};
use bitflags::bitflags;
use std::collections::VecDeque;

mod instruction;
mod register;
use instruction::{Instruction, JumpCondition, MicroOp};
use log::debug;
use register::{Register16, Register8};

use self::instruction::PostOperation;

bitflags! {
    struct Flags: u8 {
        const ZERO       = 1 << 7;
        const ADD_SUB    = 1 << 6;
        const HALF_CARRY = 1 << 5;
        const CARRY      = 1 << 4;
    }
}

#[derive(Debug, Clone)]
pub struct CPU {
    mmu: MMU,

    reg_a: u8,
    reg_b: u8,
    reg_c: u8,
    reg_d: u8,
    reg_e: u8,
    reg_h: u8,
    reg_l: u8,

    flags: Flags,

    sp: u16,
    pc: u16,

    pipeline: VecDeque<MicroOp>,
}

impl CPU {
    pub fn new(mmu: MMU) -> Self {
        CPU {
            mmu,
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
        }
    }

    pub fn load_reg16(&self, reg: Register16) -> u16 {
        match reg {
            Register16::BC => combine(self.reg_b, self.reg_c),
            Register16::DE => combine(self.reg_d, self.reg_e),
            Register16::HL => combine(self.reg_h, self.reg_l),
            Register16::SP => self.sp,
            Register16::PC => self.pc,
        }
    }

    pub fn store_reg16(&mut self, reg: Register16, value: u16) {
        match reg {
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
        let byte = self.mmu.read_memory(self.pc);
        self.pc += 1;
        byte
    }

    pub fn fetch_and_advance_u16(&mut self) -> u16 {
        let low = self.fetch_and_advance();
        let high = self.fetch_and_advance();
        ((high as u16) << 8) | (low as u16)
    }

    pub fn fetch_and_decode(&mut self) -> Instruction {
        let opcode = self.fetch_and_advance();

        match opcode {
            0x00 => Instruction::NOP,
            0x0C => Instruction::IncReg8 { reg: Register8::C },
            0x0E => Instruction::LoadRegLit8bits {
                reg: Register8::C,
                literal: self.fetch_and_advance(),
            },
            0x11 => Instruction::LoadRegLit16bits {
                reg: Register16::DE,
                literal: self.fetch_and_advance_u16(),
            },
            0x20 => Instruction::JumpRelative {
                condition: JumpCondition::NonZero,
                offset: self.fetch_and_advance() as i8,
            },
            0x21 => Instruction::LoadRegLit16bits {
                reg: Register16::HL,
                literal: self.fetch_and_advance_u16(),
            },
            0x31 => Instruction::LoadRegLit16bits {
                reg: Register16::SP,
                literal: self.fetch_and_advance_u16(),
            },
            0x32 => Instruction::StoreMem {
                addr: Register16::HL,
                reg: Register8::A,
                post_op: Some(PostOperation::Dec),
            },
            0x3E => Instruction::LoadRegLit8bits {
                reg: Register8::A,
                literal: self.fetch_and_advance(),
            },
            0x77 => Instruction::StoreMem {
                addr: Register16::HL,
                reg: Register8::A,
                post_op: None,
            },
            0xA8 => Instruction::XorAReg8 { reg: Register8::B },
            0xA9 => Instruction::XorAReg8 { reg: Register8::C },
            0xAA => Instruction::XorAReg8 { reg: Register8::D },
            0xAB => Instruction::XorAReg8 { reg: Register8::E },
            0xAC => Instruction::XorAReg8 { reg: Register8::H },
            0xAD => Instruction::XorAReg8 { reg: Register8::L },
            0xAF => Instruction::XorAReg8 { reg: Register8::A },
            0xCB => {
                // prefix 0xCB:
                match self.fetch_and_advance() {
                    0x7C => Instruction::BitTest {
                        reg: Register8::H,
                        bit: 7,
                    },
                    other => panic!("Unknown sub-opcode (prefix 0xCB) {:#x}", other),
                }
            }
            0xE0 => Instruction::StoreMemZeroPageLit {
                lit_offset: self.fetch_and_advance(),
                reg: Register8::A,
            },
            0xE2 => Instruction::StoreMemZeroPage {
                reg_offset: Register8::C,
                reg: Register8::A,
            },
            _ => panic!("Unknown opcode {:#x}", opcode),
        }
    }

    pub fn step(&mut self) {
        if self.pipeline.is_empty() {
            let pc = self.pc;
            let instruction = self.fetch_and_decode();
            debug!("{:#06x}: {}", pc, instruction);
            self.pipeline.extend(instruction.to_micro_ops());
        }

        if let Some(micro_op) = self.pipeline.pop_front() {
            match micro_op {
                MicroOp::NOP => {}
                MicroOp::LoadRegLit { reg, literal } => {
                    self.store_reg8(reg, literal);
                }
                MicroOp::XorAReg { reg } => {
                    self.reg_a ^= self.load_reg8(reg);
                    self.flags = if self.reg_a == 0 {
                        Flags::ZERO
                    } else {
                        Flags::empty()
                    };
                }
                MicroOp::StoreMem { addr, reg, post_op } => {
                    let addr_value = self.load_reg16(addr);
                    self.mmu.write_memory(addr_value, self.load_reg8(reg));
                    match post_op {
                        Some(PostOperation::Dec) => {
                            self.store_reg16(addr, self.load_reg16(addr).wrapping_sub(1))
                        }
                        Some(PostOperation::Inc) => {
                            self.store_reg16(addr, self.load_reg16(addr).wrapping_add(1))
                        }
                        None => {}
                    }
                }
                MicroOp::StoreMemZeroPage { reg_offset, reg } => {
                    let addr_value = 0xFF00 + self.load_reg8(reg_offset) as u16;
                    self.mmu.write_memory(addr_value, self.load_reg8(reg));
                }
                MicroOp::StoreMemZeroPageLit { lit_offset, reg } => {
                    let addr_value = 0xFF00 + lit_offset as u16;
                    self.mmu.write_memory(addr_value, self.load_reg8(reg));
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
                MicroOp::IncReg { reg } => {
                    self.store_reg8(reg, self.load_reg8(reg).wrapping_add(1));
                }
            }
        }
    }
}
