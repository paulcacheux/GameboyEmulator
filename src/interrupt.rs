use std::{rc::Rc, sync::Mutex};

use bitflags::bitflags;
use log::warn;

pub type InterruptControllerPtr = Rc<Mutex<InterruptController>>;

bitflags! {
    pub struct IntKind: u8 {
        const VBLANK   = 1 << 0;
        const LCD_STAT = 1 << 1;
        const TIMER    = 1 << 2;
        const SERIAL   = 1 << 3;
        const JOYPAD   = 1 << 4;

        const DUMMY    = 0b11100000;
    }
}

#[derive(Debug)]
pub struct InterruptController {
    pub master_enable: bool,
    pub interrupt_enable: IntKind,
    pub interrupt_flag: IntKind,

    pub divider_register: u8,
    divider_counter: u8,

    pub timer_counter: u8,
    pub timer_modulo: u8,
    timer_sub_counter: u32,

    pub timer_control: u8,

    pub should_redraw: bool,
}

impl InterruptController {
    pub fn new() -> Self {
        InterruptController {
            master_enable: false, // should implement delay, 1 instruction
            interrupt_enable: IntKind::empty(),
            interrupt_flag: IntKind::DUMMY,

            divider_register: 0,
            divider_counter: 0,

            timer_counter: 0,
            timer_modulo: 0,
            timer_sub_counter: 0,

            timer_control: 0,

            should_redraw: false,
        }
    }

    pub fn timer_step(&mut self) {
        // divider (increase at 1/256 the frequency of the CPU)
        let (new_counter, trigger) = self.divider_counter.overflowing_add(1);
        self.divider_counter = new_counter;
        if trigger {
            self.divider_register = self.divider_register.wrapping_add(1);
        }

        // timer
        if self.is_timer_enabled() {
            self.timer_sub_counter += 1;
            if self.timer_sub_counter >= self.timer_divider() {
                self.timer_sub_counter = 0;
                let (new_timer, carry) = self.timer_counter.overflowing_add(1);
                self.timer_counter = if carry {
                    self.trigger_timer_int();
                    self.timer_modulo
                } else {
                    new_timer
                }
            }
        } else {
            self.timer_counter = 0;
            self.timer_sub_counter = 0;
        }
    }

    fn timer_divider(&self) -> u32 {
        let control = self.timer_control & 0b11;
        match control {
            00 => 1024,
            01 => 16,
            10 => 64,
            11 => 256,
            _ => unreachable!(),
        }
    }

    fn is_timer_enabled(&self) -> bool {
        self.timer_control & 0b100 == 0b100
    }

    pub fn trigger_vblank_int(&mut self) {
        self.interrupt_flag |= IntKind::VBLANK;
        self.should_redraw = true;
    }

    pub fn trigger_timer_int(&mut self) {
        self.interrupt_flag |= IntKind::TIMER;
    }

    pub fn is_interrupt_waiting(&self) -> Option<IntKind> {
        if !self.master_enable {
            return None;
        }

        let acceptables = self.interrupt_flag & self.interrupt_enable & !IntKind::DUMMY;
        for &kind in &[
            IntKind::VBLANK,
            IntKind::LCD_STAT,
            IntKind::TIMER,
            IntKind::SERIAL,
            IntKind::JOYPAD,
        ] {
            if acceptables.contains(kind) {
                return Some(kind);
            }
        }
        None
    }
}
