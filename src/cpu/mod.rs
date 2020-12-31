use crate::{
    interrupt::{IntKind, InterruptControllerPtr},
    memory::Memory,
    utils::combine,
};
use bitflags::bitflags;
use std::collections::VecDeque;

mod decode;
mod instruction;
mod micro_op;
mod register;

use instruction::{Instruction, JumpCondition};
use log::{debug, warn};
use micro_op::{Destination8Bits, MicroOp, Reg8OrIndirect, Source8bits};
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
    interrupt_controller: InterruptControllerPtr,
    halted: bool,
    stoped: bool,
}

impl<M: Memory> CPU<M> {
    pub fn new(memory: M, interrupt_controller: InterruptControllerPtr) -> Self {
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
            interrupt_controller,
            halted: false,
            stoped: false,
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
        decode::decode_instruction(self)
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
            Source8bits::ZeroPageOffsetReg8(offset) => self
                .memory
                .read_memory(0xFF00 + self.load_reg8(offset) as u16),
        }
    }

    fn load_reg8_or_indirect(&self, op: Reg8OrIndirect) -> u8 {
        match op {
            Reg8OrIndirect::Reg8(reg) => self.load_reg8(reg),
            Reg8OrIndirect::Indirect(addr) => self.memory.read_memory(self.load_reg16(addr)),
        }
    }

    fn store_reg8_or_indirect(&mut self, op: Reg8OrIndirect, value: u8) {
        match op {
            Reg8OrIndirect::Reg8(reg) => self.store_reg8(reg, value),
            Reg8OrIndirect::Indirect(addr) => {
                self.memory.write_memory(self.load_reg16(addr), value)
            }
        }
    }

    fn handle_interrupts(&mut self) {
        let mut controller = self.interrupt_controller.lock().unwrap();
        if controller.handle_new_interrupt() {
            self.halted = false;
            self.stoped = false;
        }

        if let Some(kind) = controller.is_interrupt_waiting() {
            controller.interrupt_flag.remove(kind);
            controller.master_enable = false;

            let addr = match kind {
                IntKind::VBLANK => 0x40,
                IntKind::LCD_STAT => 0x48,
                IntKind::TIMER => 0x50,
                IntKind::SERIAL => 0x58,
                IntKind::JOYPAD => 0x60,
                _ => panic!("Failed to get interrupt handler address"),
            };

            let micro_ops = vec![
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
            ];
            self.pipeline.extend(micro_ops);
        }
    }

    fn decode_next_instruction(&mut self) {
        let instruction = self.fetch_and_decode();
        debug!("{:#06x}: {}", self.pc, instruction);
        self.pipeline.extend(instruction.to_micro_ops());
    }

    pub fn step(&mut self) {
        if !self.stoped {
            self.interrupt_controller.lock().unwrap().timer_step(4);
        }

        if self.pipeline.is_empty() {
            self.handle_interrupts();
        }

        if self.pipeline.is_empty() && !self.halted && !self.stoped {
            self.decode_next_instruction();
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
                    let half_carry = check_half_carry_16bits_high(hl_value, rhs_value);

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
                    self.sub_a(self.source_8bits_to_value(rhs), false, true);
                }
                MicroOp::SbcA { rhs } => {
                    self.sub_a(self.source_8bits_to_value(rhs), true, true);
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
                    let is_set = (self.load_reg8_or_indirect(reg) >> bit) & 1 == 1;
                    let rest = Flags::HALF_CARRY | (self.flags & Flags::CARRY);
                    self.flags = if is_set { rest } else { Flags::ZERO | rest };
                }
                MicroOp::ResetBit { reg, bit } => {
                    let value = self.load_reg8_or_indirect(reg);
                    let res = value & !(1 << bit);
                    self.store_reg8_or_indirect(reg, res);
                }
                MicroOp::SetBit { reg, bit } => {
                    let value = self.load_reg8_or_indirect(reg);
                    let res = value | (1 << bit);
                    self.store_reg8_or_indirect(reg, res);
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
                MicroOp::AddOffsetToReg16IntoReg16 {
                    dest,
                    rhs,
                    offset,
                    update_flags,
                } => {
                    let value = self.load_reg16(rhs);
                    let (res, carry, half_carry) = if offset < 0 {
                        let neg_offset = (-offset) as u16;
                        (
                            value.wrapping_sub(neg_offset),
                            check_half_carry_sub_16bits_mid(value, neg_offset),
                            check_half_carry_sub_16bits_low(value, neg_offset),
                        )
                    } else {
                        let offset = offset as u16;
                        (
                            value.wrapping_add(offset),
                            check_half_carry_16bits_mid(value, offset),
                            check_half_carry_16bits_low(value, offset),
                        )
                    };
                    self.store_reg16(dest, res);

                    if update_flags {
                        self.flags = Flags::empty();

                        if carry {
                            self.flags |= Flags::CARRY;
                        }

                        if half_carry {
                            self.flags |= Flags::HALF_CARRY;
                        }
                    }
                }
                MicroOp::IncReg16 { reg } => {
                    // No flags change for this micro op
                    self.store_reg16(reg, self.load_reg16(reg).wrapping_add(1));
                }
                MicroOp::Inc { reg } => {
                    let reg_value = self.load_reg8_or_indirect(reg);
                    let half_carry = check_half_carry(reg_value, 1);
                    let new_value = reg_value.wrapping_add(1);
                    self.store_reg8_or_indirect(reg, new_value);
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
                MicroOp::DecReg16 { reg } => {
                    // No flags change for this micro op
                    self.store_reg16(reg, self.load_reg16(reg).wrapping_sub(1));
                }
                MicroOp::Dec { reg } => {
                    let reg_value = self.load_reg8_or_indirect(reg);
                    let half_carry = check_half_carry_sub(reg_value, 1);
                    let new_value = reg_value.wrapping_sub(1);
                    self.store_reg8_or_indirect(reg, new_value);

                    self.update_flags_arith(
                        new_value,
                        true,
                        self.flags.contains(Flags::CARRY),
                        !half_carry,
                    );
                }
                MicroOp::CompareA { rhs } => {
                    let rhs = self.source_8bits_to_value(rhs);
                    self.sub_a(rhs, false, false);
                }
                MicroOp::RotateLeftThroughCarry { reg, set_zero } => {
                    let value = self.load_reg8_or_indirect(reg);
                    let new_carry = (value >> 7) == 1;
                    let new_value = (value << 1) | (self.flags.contains(Flags::CARRY) as u8);
                    self.store_reg8_or_indirect(reg, new_value);

                    self.flags = Flags::empty();
                    if new_carry {
                        self.flags |= Flags::CARRY;
                    }
                    if set_zero && new_value == 0 {
                        self.flags |= Flags::ZERO;
                    }
                }
                MicroOp::RotateRightThroughCarry { reg, set_zero } => {
                    let value = self.load_reg8_or_indirect(reg);
                    let new_carry = (value & 0x1) == 1;
                    let new_value = ((self.flags.contains(Flags::CARRY) as u8) << 7) | (value >> 1);
                    self.store_reg8_or_indirect(reg, new_value);

                    self.flags = Flags::empty();
                    if new_carry {
                        self.flags |= Flags::CARRY;
                    }
                    if set_zero && new_value == 0 {
                        self.flags |= Flags::ZERO;
                    }
                }
                MicroOp::RotateLeft { reg, set_zero } => {
                    let value = self.load_reg8_or_indirect(reg);
                    let new_carry = (value >> 7) == 1;
                    let new_value = value.rotate_left(1);
                    self.store_reg8_or_indirect(reg, new_value);

                    self.flags = Flags::empty();
                    if new_carry {
                        self.flags |= Flags::CARRY;
                    }
                    if set_zero && new_value == 0 {
                        self.flags |= Flags::ZERO;
                    }
                }
                MicroOp::RotateRight { reg, set_zero } => {
                    let value = self.load_reg8_or_indirect(reg);
                    let new_carry = (value & 1) == 1;
                    let new_value = value.rotate_right(1);
                    self.store_reg8_or_indirect(reg, new_value);

                    self.flags = Flags::empty();
                    if new_carry {
                        self.flags |= Flags::CARRY;
                    }
                    if set_zero && new_value == 0 {
                        self.flags |= Flags::ZERO;
                    }
                }
                MicroOp::ShiftLeftIntoCarry { reg } => {
                    let value = self.load_reg8_or_indirect(reg);
                    let carry = (value >> 7) == 1;
                    let new_value = value << 1;
                    self.store_reg8_or_indirect(reg, new_value);

                    self.flags = Flags::empty();
                    if carry {
                        self.flags |= Flags::CARRY;
                    }
                    if new_value == 0 {
                        self.flags |= Flags::ZERO;
                    }
                }
                MicroOp::ShiftRightWithZeroIntoCarry { reg } => {
                    let value = self.load_reg8_or_indirect(reg);
                    let carry = (value & 0x1) == 1;
                    let new_value = value >> 1;
                    self.store_reg8_or_indirect(reg, new_value);

                    self.flags = Flags::empty();
                    if carry {
                        self.flags |= Flags::CARRY;
                    }
                    if new_value == 0 {
                        self.flags |= Flags::ZERO;
                    }
                }
                MicroOp::ShiftRightWithSignIntoCarry { reg } => {
                    let value = self.load_reg8_or_indirect(reg);
                    let carry = (value & 0x1) == 1;
                    let new_value = ((value as i8) >> 1) as u8;
                    self.store_reg8_or_indirect(reg, new_value);

                    self.flags = Flags::empty();
                    if carry {
                        self.flags |= Flags::CARRY;
                    }
                    if new_value == 0 {
                        self.flags |= Flags::ZERO;
                    }
                }
                MicroOp::SwapReg8 { reg } => {
                    let value = self.load_reg8_or_indirect(reg);
                    let high = (value & 0xF0) >> 4;
                    let low = value & 0x0F;
                    let res = (low << 4) | high;
                    self.store_reg8_or_indirect(reg, res);
                    self.flags = Flags::empty();
                    if res == 0 {
                        self.flags |= Flags::ZERO;
                    }
                }
                MicroOp::SetCarryFlag => {
                    self.flags = (self.flags & Flags::ZERO) | Flags::CARRY;
                }
                MicroOp::ComplementCarryFlag => {
                    self.flags.remove(Flags::NEGATIVE | Flags::HALF_CARRY);
                    self.flags.toggle(Flags::CARRY);
                }
                MicroOp::EnableInterrupts => {
                    self.interrupt_controller.lock().unwrap().master_enable = true;
                }
                MicroOp::DisableInterrupts => {
                    self.interrupt_controller.lock().unwrap().master_enable = false;
                }
                MicroOp::Halt => {
                    self.halted = true;
                }
                MicroOp::Stop => {
                    self.stoped = true;
                    warn!("CPU stopped pc={:#x}", self.pc);
                }
            }
        }
    }

    fn sub_a(&mut self, b: u8, use_carry: bool, apply_res: bool) {
        let c = if use_carry && self.flags.contains(Flags::CARRY) {
            1
        } else {
            0
        };

        let a = self.reg_a;
        let res = a.wrapping_sub(b).wrapping_sub(c);

        self.update_flags_arith(
            res,
            true,
            (a as u16) < (b as u16) + (c as u16),
            (a & 0x0F) < (b & 0x0F) + c,
        );

        if apply_res {
            self.reg_a = res;
        }
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

fn check_half_carry_16bits_high(a: u16, b: u16) -> bool {
    (((a & 0xFFF) + (b & 0xFFF)) & 0x1000) == 0x1000
}

fn check_half_carry_16bits_mid(a: u16, b: u16) -> bool {
    (((a & 0xFF) + (b & 0xFF)) & 0x100) == 0x100
}

fn check_half_carry_16bits_low(a: u16, b: u16) -> bool {
    check_half_carry(a as u8, b as u8)
}

fn check_half_carry_sub(a: u8, b: u8) -> bool {
    let neg_b = u8::MAX.wrapping_sub(b).wrapping_add(1);
    check_half_carry(a, neg_b)
}

fn check_half_carry_sub_16bits_low(a: u16, b: u16) -> bool {
    check_half_carry_sub(a as u8, b as u8)
}

fn check_half_carry_sub_16bits_mid(a: u16, b: u16) -> bool {
    let neg_b = u16::MAX.wrapping_sub(b).wrapping_add(1);
    check_half_carry_16bits_mid(a, neg_b)
}
