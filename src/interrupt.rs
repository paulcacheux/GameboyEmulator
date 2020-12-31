use std::{rc::Rc, sync::Mutex};

use bitflags::bitflags;

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

bitflags! {
    pub struct JoypadBits: u8 {
        const P15_SELECT_BUTTON_KEYS    = 1 << 5;
        const P14_SELECT_DIRECTION_KEYS = 1 << 4;
        const P13_INPUT_DOWN_OR_START   = 1 << 3;
        const P12_INPUT_UP_OR_SELECT    = 1 << 2;
        const P11_INPUT_LEFT_OR_B       = 1 << 1;
        const P10_INPUT_RIGHT_OR_A      = 1 << 0;
    }
}

#[repr(usize)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Keys {
    Up = 0,
    Down,
    Left,
    Right,
    A,
    B,
    Start,
    Select,
    KeysMax,
}

#[derive(Debug)]
pub struct InterruptController {
    pub master_enable: bool,
    pub interrupt_enable: IntKind,
    pub interrupt_flag: IntKind,

    pub divider_register: u8,
    divider_counter: u32,

    pub timer_counter: u8,
    pub timer_modulo: u8,
    timer_sub_counter: u32,

    pub timer_control: u8,

    pub should_redraw: bool,
    new_int_waiting: bool,

    keys_state: [bool; Keys::KeysMax as usize],
    select_buttons: bool,
    select_directions: bool,
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
            new_int_waiting: false,

            keys_state: [false; Keys::KeysMax as usize],
            select_buttons: false,
            select_directions: false,
        }
    }

    pub fn timer_step(&mut self, ticks: u32) {
        // divider (increase at 1/256 the frequency of the CPU)
        self.divider_counter = self.divider_counter.wrapping_add(ticks);
        while self.divider_counter >= 256 {
            self.divider_register = self.divider_register.wrapping_add(1);
            self.divider_counter -= 256;
        }

        // timer
        if self.is_timer_enabled() {
            let divider = self.timer_divider();

            self.timer_sub_counter += ticks;
            while self.timer_sub_counter >= divider {
                self.timer_sub_counter;
                let (new_timer, carry) = self.timer_counter.overflowing_add(1);
                self.timer_counter = if carry {
                    self.trigger_timer_int();
                    self.timer_modulo
                } else {
                    new_timer
                };

                self.timer_sub_counter -= divider;
            }
        } else {
            self.timer_counter = 0;
            self.timer_sub_counter = 0;
        }
    }

    fn timer_divider(&self) -> u32 {
        let control = self.timer_control & 0b11;
        match control {
            0b00 => 1024,
            0b01 => 16,
            0b10 => 64,
            0b11 => 256,
            _ => unreachable!(),
        }
    }

    fn is_timer_enabled(&self) -> bool {
        self.timer_control & 0b100 == 0b100
    }

    pub fn trigger_vblank_int(&mut self) {
        self.interrupt_flag |= IntKind::VBLANK;
        self.new_int_waiting = true;
        self.should_redraw = true;
    }

    pub fn trigger_timer_int(&mut self) {
        self.interrupt_flag |= IntKind::TIMER;
        self.new_int_waiting = true;
    }

    pub fn trigger_lcd_stat_int(&mut self) {
        self.interrupt_flag |= IntKind::LCD_STAT;
        self.new_int_waiting = true;
    }

    pub fn handle_new_interrupt(&mut self) -> bool {
        let res = self.new_int_waiting;
        self.new_int_waiting = false;
        res
    }

    pub fn is_interrupt_waiting(&self) -> Option<IntKind> {
        if !self.master_enable {
            return None;
        }

        let requested = self.interrupt_flag & self.interrupt_enable & !IntKind::DUMMY;
        for &kind in &[
            IntKind::VBLANK,
            IntKind::LCD_STAT,
            IntKind::TIMER,
            IntKind::SERIAL,
            IntKind::JOYPAD,
        ] {
            if requested.contains(kind) {
                return Some(kind);
            }
        }
        None
    }

    pub fn change_key_state(&mut self, key: Keys, pressed: bool) {
        self.keys_state[key as usize] = pressed;
        todo!("Trigger interruption")
    }

    pub fn write_joypad_reg(&mut self, reg_value: u8) {
        // 0 is selected
        let flags = JoypadBits::from_bits_truncate(!reg_value);

        self.select_directions = flags.contains(JoypadBits::P14_SELECT_DIRECTION_KEYS);
        self.select_buttons = flags.contains(JoypadBits::P15_SELECT_BUTTON_KEYS);
    }

    pub fn read_joypad_reg(&mut self) -> u8 {
        let mut flags = JoypadBits::empty();

        if self.select_directions {
            flags |= JoypadBits::P14_SELECT_DIRECTION_KEYS;
            if self.keys_state[Keys::Down as usize] {
                flags |= JoypadBits::P13_INPUT_DOWN_OR_START;
            }
            if self.keys_state[Keys::Up as usize] {
                flags |= JoypadBits::P12_INPUT_UP_OR_SELECT;
            }
            if self.keys_state[Keys::Left as usize] {
                flags |= JoypadBits::P11_INPUT_LEFT_OR_B;
            }
            if self.keys_state[Keys::Right as usize] {
                flags |= JoypadBits::P10_INPUT_RIGHT_OR_A;
            }
        }

        if self.select_buttons {
            flags |= JoypadBits::P15_SELECT_BUTTON_KEYS;
            if self.keys_state[Keys::Start as usize] {
                flags |= JoypadBits::P13_INPUT_DOWN_OR_START;
            }
            if self.keys_state[Keys::Select as usize] {
                flags |= JoypadBits::P12_INPUT_UP_OR_SELECT;
            }
            if self.keys_state[Keys::B as usize] {
                flags |= JoypadBits::P11_INPUT_LEFT_OR_B;
            }
            if self.keys_state[Keys::A as usize] {
                flags |= JoypadBits::P10_INPUT_RIGHT_OR_A;
            }
        }

        !flags.bits()
    }
}
