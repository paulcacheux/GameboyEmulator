use std::collections::VecDeque;

use crate::memory::Memory;
use bitflags::bitflags;

mod fetcher;
use fetcher::*;

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

#[derive(Debug, Clone)]
pub struct PPU<M: Memory> {
    memory: M,
    current_dot_in_line: u32,
    frame: [u8; PIXEL_COUNT],
    fetcher: FetcherState,
    fifo: VecDeque<Pixel>,
}

const SCREEN_WIDTH: u8 = 160;
const SCREEN_HEIGHT: u8 = 144;
const PIXEL_COUNT: usize = (SCREEN_WIDTH as usize) * (SCREEN_HEIGHT as usize);

const SCAN_LINE_COUNT: u8 = SCREEN_HEIGHT + 10;
const DOT_PER_LINE_COUNT: u32 = 80 + 172 + 204;
const HBLANK_START: u32 = 172 + 80;

const LCD_CONTROL_REG_ADDR: u16 = 0xFF40;
const LCD_STATUS_REG_ADDR: u16 = 0xFF41;
const LCD_SCROLL_Y_ADDR: u16 = 0xFF42;
const LCD_SCROLL_X_ADDR: u16 = 0xFF43;
const LCD_LY_ADDR: u16 = 0xFF44;
const LCD_LYC_ADDR: u16 = 0xFF45;
const BG_PALETTE_DATA_ADDR: u16 = 0xFF47;

impl<M: Memory> PPU<M> {
    pub fn new(memory: M) -> Self {
        PPU {
            memory,
            current_dot_in_line: 0,
            frame: [0; PIXEL_COUNT],
            fetcher: FetcherState::Waiting,
            fifo: VecDeque::new(),
        }
    }

    fn control_reg(&self) -> ControlReg {
        ControlReg::from_bits(self.memory.read_memory(LCD_CONTROL_REG_ADDR))
            .expect("Failed to read control_reg")
    }

    fn current_line(&self) -> u8 {
        self.memory.read_memory(LCD_LY_ADDR)
    }

    fn current_mode(&self) -> Mode {
        let current_line = self.current_line();
        if current_line >= SCREEN_HEIGHT {
            Mode::VBlank
        } else if self.current_dot_in_line < 80 {
            Mode::OAMSearch
        } else if self.current_dot_in_line < HBLANK_START {
            Mode::LCDTransfer
        } else {
            Mode::HBlank
        }
    }

    fn update_status_reg(&mut self) {
        let coincidence = self.current_line() == self.memory.read_memory(LCD_LYC_ADDR);

        let updated_part = ((coincidence as u8) << 3) | (self.current_mode() as u8);
        let old_reg = self.memory.read_memory(LCD_STATUS_REG_ADDR);
        self.memory
            .write_memory(LCD_STATUS_REG_ADDR, (old_reg & 0xF0) | updated_part);
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
    }

    fn prepare_frame_and_fetcher(&mut self) {
        for pixel in &mut self.frame {
            *pixel = 0;
        }

        self.fifo.clear();
        self.next_fetcher_state();
    }

    fn next_fetcher_state(&mut self) {
        let lcdc = self.control_reg();
        let scan_line = self.current_line();
        self.fetcher.next_update(lcdc, &mut self.memory, scan_line);
    }

    fn cycle(&mut self) {
        self.update_status_reg();

        let current_mode = self.current_mode();
        match current_mode {
            Mode::HBlank => {}
            Mode::VBlank => {}
            Mode::OAMSearch => {}
            Mode::LCDTransfer => {
                if self.current_dot_in_line == 80 {
                    self.prepare_frame_and_fetcher();
                } else if self.current_dot_in_line > 80 {
                    self.next_fetcher_state();
                    let scan_line = self.current_line();

                    match &mut self.fetcher {
                        FetcherState::Waiting => {}
                        FetcherState::Ready(fetcher) => {
                            self.fifo.extend(&fetcher.fetch_pixels(&mut self.memory));

                            let scroll_x = self.memory.read_memory(LCD_SCROLL_X_ADDR);
                            for _ in 0..(scroll_x % 8) {
                                self.fifo.pop_front();
                            }
                        }
                        FetcherState::Working(fetcher, x) => {
                            if self.fifo.len() < 8 {
                                self.fifo.extend(&fetcher.fetch_pixels(&mut self.memory));
                            }

                            let pixel = self.fifo.pop_front().unwrap();
                            let offset =
                                (scan_line as usize) * (SCREEN_WIDTH as usize) + (*x as usize);

                            let bg_palette = self.memory.read_memory(BG_PALETTE_DATA_ADDR);
                            let actual_color = bg_palette >> (pixel.color * 2) & 0b11;

                            self.frame[offset] = actual_color;
                        }
                    }
                }
            }
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
enum Mode {
    HBlank = 0,
    VBlank = 1,
    OAMSearch = 2,
    LCDTransfer = 3,
}

#[derive(Debug, Clone)]
enum FetcherState {
    Waiting,
    Ready(Fetcher),
    Working(Fetcher, u8),
}

impl Default for FetcherState {
    fn default() -> Self {
        FetcherState::Waiting
    }
}

impl FetcherState {
    fn next(self, lcdc: ControlReg, memory: &mut dyn Memory, scan_line: u8) -> Self {
        match self {
            FetcherState::Waiting => {
                let fetcher = Fetcher::new(
                    if lcdc.contains(ControlReg::BG_TILE_MAP_DISPLAY_SELECT) {
                        0x9C00
                    } else {
                        0x9800
                    },
                    if lcdc.contains(ControlReg::BG_WINDOW_TILE_DATA_SELECT) {
                        AddressingMode::From8000
                    } else {
                        AddressingMode::From8800
                    },
                    memory.read_memory(LCD_SCROLL_X_ADDR),
                    memory.read_memory(LCD_SCROLL_Y_ADDR),
                    scan_line,
                );
                FetcherState::Ready(fetcher)
            }
            FetcherState::Ready(fetcher) => FetcherState::Working(fetcher, 0),
            FetcherState::Working(fetcher, x) => {
                if x + 1 == SCREEN_WIDTH {
                    FetcherState::Waiting
                } else {
                    FetcherState::Working(fetcher, x + 1)
                }
            }
        }
    }

    fn next_update(&mut self, lcdc: ControlReg, memory: &mut dyn Memory, scan_line: u8) {
        let current = std::mem::take(self);
        let next = current.next(lcdc, memory, scan_line);
        let _ = std::mem::replace(self, next);
    }
}
