use std::sync::{Arc, Mutex};

use crate::{display::Display, interrupt::InterruptControllerPtr, memory::Memory};
use bitflags::bitflags;

mod fetcher;
mod oam;
pub mod pixel;
mod pixel_fifo;
use fetcher::*;
use pixel_fifo::PixelFIFO;

bitflags! {
    pub struct ControlReg: u8 {
        const DISPLAY_ENABLE = 1 << 7;
        const WINDOW_TILE_MAP_DISPLAY_SELECT = 1 << 6;
        const WINDOW_DISPLAY_ENABLE = 1 << 5;
        const BG_WINDOW_TILE_DATA_SELECT = 1 << 4;
        const BG_TILE_MAP_DISPLAY_SELECT = 1 << 3;
        const OBJ_SIZE = 1 << 2;
        const OBJ_DISPLAY_ENABLE = 1 << 1;
        const BG_WINDOW_DISPLAY_PRIORITY = 1 << 0;
    }
}

impl ControlReg {
    pub fn background_tile_map_addr(&self) -> u16 {
        if self.contains(ControlReg::BG_TILE_MAP_DISPLAY_SELECT) {
            0x9C00
        } else {
            0x9800
        }
    }

    pub fn window_tile_map_addr(&self) -> u16 {
        if self.contains(ControlReg::WINDOW_TILE_MAP_DISPLAY_SELECT) {
            0x9C00
        } else {
            0x9800
        }
    }

    pub fn addressing_mode(&self) -> AddressingMode {
        if self.contains(ControlReg::BG_WINDOW_TILE_DATA_SELECT) {
            AddressingMode::From8000
        } else {
            AddressingMode::From8800
        }
    }
}

pub const SCREEN_WIDTH: u8 = 160;
pub const SCREEN_HEIGHT: u8 = 144;
pub const PIXEL_COUNT: usize = (SCREEN_WIDTH as usize) * (SCREEN_HEIGHT as usize);

const SCAN_LINE_COUNT: u8 = SCREEN_HEIGHT + 10;
const DOT_PER_LINE_COUNT: u32 = 80 + 172 + 204;

const LCD_CONTROL_REG_ADDR: u16 = 0xFF40;
const LCD_STATUS_REG_ADDR: u16 = 0xFF41;
const LCD_SCROLL_Y_ADDR: u16 = 0xFF42;
const LCD_SCROLL_X_ADDR: u16 = 0xFF43;
const LCD_LY_ADDR: u16 = 0xFF44;
const LCD_LYC_ADDR: u16 = 0xFF45;

const BG_PALETTE_DATA_ADDR: u16 = 0xFF47;
const OAM0_PALETTE_DATA_ADDR: u16 = 0xFF48;
const OAM1_PALETTE_DATA_ADDR: u16 = 0xFF49;

const LCD_WINDOW_Y_POSITION_ADDR: u16 = 0xFF4A;
const LCD_WINDOW_X_POSITION_ADDR: u16 = 0xFF4B;

#[derive(Debug, Clone)]
pub struct PPU<M: Memory> {
    memory: M,
    interrupt_controller: InterruptControllerPtr,

    scan_line: u8,
    dot_in_line: u32,
    state: PPUState,
    int_cond_met: bool,

    display: Arc<Mutex<Display>>,
    pub frame: [u8; PIXEL_COUNT],

    pixel_fifo: PixelFIFO<M>,
}

impl<M: Memory + Clone> PPU<M> {
    pub fn new(
        memory: M,
        interrupt_controller: InterruptControllerPtr,
        display: Arc<Mutex<Display>>,
    ) -> Self {
        PPU {
            memory: memory.clone(),
            interrupt_controller,

            scan_line: 0,
            dot_in_line: 0,
            state: PPUState::OAMSearchBegin,
            int_cond_met: false,

            display,
            frame: [0; PIXEL_COUNT],

            pixel_fifo: PixelFIFO::new(memory),
        }
    }

    fn update_registers(&mut self) {
        // status reg
        let coincidence = self.scan_line == self.memory.read_memory(LCD_LYC_ADDR);

        let updated_part = ((coincidence as u8) << 2) | (self.state.mode() as u8);
        let old_reg = self.memory.read_memory(LCD_STATUS_REG_ADDR);
        self.memory
            .write_memory(LCD_STATUS_REG_ADDR, (old_reg & 0b11111000) | updated_part);

        // LY reg

        self.memory.write_memory(LCD_LY_ADDR, self.scan_line);
    }

    fn maybe_trigger_stat_int(&mut self) {
        let mut new_int_cond_met = false;
        let stat_value = self.memory.read_memory(LCD_STATUS_REG_ADDR);

        if (stat_value & (1 << 6) != 0) && (stat_value & (1 << 2) != 0) {
            new_int_cond_met = true;
        }

        if (stat_value & (1 << 5) != 0) && (stat_value & 0b11 == 2) {
            new_int_cond_met = true;
        }

        if (stat_value & (1 << 4) != 0) && (stat_value & 0b11 == 1) {
            new_int_cond_met = true;
        }

        if (stat_value & (1 << 3) != 0) && (stat_value & 0b11 == 0) {
            new_int_cond_met = true;
        }

        if !self.int_cond_met && new_int_cond_met {
            self.interrupt_controller
                .lock()
                .unwrap()
                .trigger_lcd_stat_int();
        }
        self.int_cond_met = new_int_cond_met
    }

    fn next_dot(&mut self) {
        self.dot_in_line += 1;

        if self.dot_in_line == DOT_PER_LINE_COUNT {
            self.dot_in_line = 0;
            self.scan_line += 1;
            if self.scan_line == SCAN_LINE_COUNT {
                self.scan_line = 0;
            }
            self.memory.write_memory(LCD_LY_ADDR, self.scan_line);
        }

        self.state = PPUState::current_state(self.dot_in_line, self.scan_line);
    }

    fn clear_frame(&mut self) {
        self.display.lock().unwrap().push_frame(&self.frame);

        for pixel in self.frame.iter_mut() {
            *pixel = 0;
        }
    }

    fn cycle(&mut self) {
        self.update_registers();
        self.maybe_trigger_stat_int();

        if self.dot_in_line == 0 && self.scan_line == 0 {
            self.clear_frame();
        }

        match self.state {
            PPUState::OAMSearchBegin => {
                self.pixel_fifo.begin_of_line(self.scan_line);
            }
            PPUState::OAMSearch => {}
            PPUState::OAMSearchEnd => {
                self.pixel_fifo.end_of_oam_search();
            }
            PPUState::TransferInit => {
                self.pixel_fifo.begin_lcd_transfer();
            }
            PPUState::Transfer { x } => {
                assert!(x < 160);

                let pixel = self.pixel_fifo.next_pixel();

                let offset = (self.scan_line as usize) * (SCREEN_WIDTH as usize) + (x as usize);

                let actual_color = pixel.through_palette(&self.memory);

                self.frame[offset] = actual_color;
            }
            PPUState::PostTransfer => {}
            PPUState::HBlankInit => {
                self.pixel_fifo.end_of_line();
            }
            PPUState::HBlank => {}
            PPUState::VBlankInit => {
                self.interrupt_controller
                    .lock()
                    .unwrap()
                    .trigger_vblank_int();

                self.pixel_fifo.end_of_frame();
            }
            PPUState::VBlank => {}
        }

        self.next_dot();
    }

    pub fn step(&mut self) {
        for _ in 0..4 {
            self.cycle();
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    HBlank = 0,
    VBlank = 1,
    OAMSearch = 2,
    LCDTransfer = 3,
}

#[derive(Debug, Clone)]
enum PPUState {
    OAMSearchBegin,
    OAMSearch,
    OAMSearchEnd,
    TransferInit,
    Transfer { x: u8 },
    PostTransfer,
    HBlankInit,
    HBlank,
    VBlankInit,
    VBlank,
}

impl PPUState {
    fn current_state(dot: u32, scan_line: u8) -> Self {
        assert!(scan_line < SCAN_LINE_COUNT);
        assert!(dot < 456);

        if scan_line < SCREEN_HEIGHT {
            match dot {
                0 => PPUState::OAMSearchBegin,
                1..=78 => PPUState::OAMSearch,
                79 => PPUState::OAMSearchEnd,
                80 => PPUState::TransferInit,
                81..=240 => PPUState::Transfer { x: dot as u8 - 81 },
                241..=251 => PPUState::PostTransfer,
                252 => PPUState::HBlankInit,
                253..=455 => PPUState::HBlank,
                _ => unreachable!(),
            }
        } else if scan_line == SCREEN_HEIGHT && dot == 0 {
            PPUState::VBlankInit
        } else {
            PPUState::VBlank
        }
    }

    fn mode(&self) -> Mode {
        match self {
            PPUState::OAMSearchBegin => Mode::OAMSearch,
            PPUState::OAMSearch => Mode::OAMSearch,
            PPUState::OAMSearchEnd => Mode::OAMSearch,
            PPUState::TransferInit => Mode::LCDTransfer,
            PPUState::Transfer { .. } => Mode::LCDTransfer,
            PPUState::PostTransfer => Mode::LCDTransfer,
            PPUState::HBlankInit => Mode::HBlank,
            PPUState::HBlank => Mode::HBlank,
            PPUState::VBlankInit => Mode::VBlank,
            PPUState::VBlank => Mode::VBlank,
        }
    }
}
