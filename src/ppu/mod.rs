use std::thread::current;

use crate::memory::Memory;
use bitflags::bitflags;
use log::debug;

bitflags! {
    pub struct ControlReg: u8 {
        const DISPLAY_ENABLE = 1 << 7;
        const WINDOW_TILE_MAP_DISPLAY_SELECT = 1 << 6;
        const WINDOW_DISPLAY_ENABLE = 1 << 5;
        const BG_WINDOW_TILE_DATA_SELECT = 1 << 4;
        const BG_TILE_MAP_DISPLAY_SELECT = 1 << 3;
        const OBJ_SIZE = 1 << 2;
        const OBJ_DISPLAY_ENABLE = 1 << 1;
        const BG_WINDOW_DISPLAY_PRIORITY = 1 << 1;
    }
}

#[derive(Debug, Clone)]
pub struct PPU<M: Memory> {
    memory: M,
    current_dot_in_line: u32,
}

const SCREEN_HEIGHT: u8 = 144;
const SCAN_LINE_COUNT: u8 = SCREEN_HEIGHT + 10;
const DOT_PER_LINE_COUNT: u32 = 80 + 172 + 204;
const HBLANK_START: u32 = 172 + 80;

const LCD_CONTROL_REG_ADDR: u16 = 0xFF40;
const LCD_STATUS_REG_ADDR: u16 = 0xFF41;
const LCD_LY_ADDR: u16 = 0xFF44;
const LCD_LYC_ADDR: u16 = 0xFF45;

impl<M: Memory> PPU<M> {
    pub fn new(memory: M) -> Self {
        PPU {
            memory,
            current_dot_in_line: 0,
        }
    }

    fn control_reg(&self) -> ControlReg {
        ControlReg::from_bits(self.memory.read_memory(LCD_CONTROL_REG_ADDR))
            .expect("Failed to read control_reg")
    }

    fn current_line(&self) -> u8 {
        self.memory.read_memory(LCD_LY_ADDR)
    }

    fn next_dot(&mut self) {
        let mut current_line = self.current_line();
        self.current_dot_in_line += 1;

        if self.current_dot_in_line == DOT_PER_LINE_COUNT {
            self.current_dot_in_line = 0;
            current_line += 1;
            if current_line == SCAN_LINE_COUNT {
                current_line = 0;
            }
            self.memory.write_memory(LCD_LY_ADDR, current_line);
        }

        // update status reg
        let coincidence = self.current_line() == self.memory.read_memory(LCD_LYC_ADDR);

        let mode = if current_line >= SCREEN_HEIGHT {
            1 // V-blank
        } else if self.current_dot_in_line < 80 {
            2 // Search OAM
        } else if self.current_dot_in_line < HBLANK_START {
            3 // Transfer data to LCD
        } else {
            0 // H-blank
        };

        let updated_part = ((coincidence as u8) << 3) | mode;
        let old_reg = self.memory.read_memory(LCD_STATUS_REG_ADDR);
        self.memory
            .write_memory(LCD_STATUS_REG_ADDR, (old_reg & 0xF0) | updated_part);
    }

    fn cycle(&mut self) {
        if self.current_line() >= SCREEN_HEIGHT {
            debug!("V-Blank");
        } else if self.current_dot_in_line == 0 {
            debug!("Start OAM search");
        } else if self.current_dot_in_line == 80 {
            debug!("Start updating frame");
        } else if self.current_dot_in_line == HBLANK_START {
            debug!("Start H-Blank")
        }

        self.next_dot();
    }

    pub fn step(&mut self) {
        for _ in 0..4 {
            self.cycle();
        }
    }
}
