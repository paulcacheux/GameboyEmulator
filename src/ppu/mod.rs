use std::{collections::VecDeque, io::BufWriter, writeln};

use crate::{interrupt::InterruptControllerPtr, memory::Memory};
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

pub const SCREEN_WIDTH: u8 = 160;
pub const SCREEN_HEIGHT: u8 = 144;
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

#[derive(Debug, Clone)]
pub struct PPU<M: Memory> {
    memory: M,
    interrupt_controller: InterruptControllerPtr,
    current_dot_in_line: u32,
    pub previous_frame: [u8; PIXEL_COUNT],
    pub frame: [u8; PIXEL_COUNT],
    fetcher: FetcherState,
    fifo: VecDeque<Pixel>,
}

impl<M: Memory> PPU<M> {
    pub fn new(memory: M, interrupt_controller: InterruptControllerPtr) -> Self {
        PPU {
            memory,
            interrupt_controller,
            current_dot_in_line: 0,
            previous_frame: [0; PIXEL_COUNT],
            frame: [0; PIXEL_COUNT],
            fetcher: FetcherState::Waiting,
            fifo: VecDeque::new(),
        }
    }

    pub fn draw_into_fb(&self, fb: &mut [u8]) {
        assert_eq!(PIXEL_COUNT * 4, fb.len());

        for (i, pixel) in fb.chunks_exact_mut(4).enumerate() {
            let color = self.previous_frame[i];
            pixel.copy_from_slice(&pixel_color_to_screen_color(color));
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
        for (i, pixel) in self.frame.iter_mut().enumerate() {
            self.previous_frame[i] = *pixel;
            *pixel = 0;
        }

        self.fifo.clear();
        self.fetcher = FetcherState::Waiting;
    }

    fn next_fetcher_state(&mut self) {
        let lcdc = self.control_reg();
        let scan_line = self.current_line();
        self.fetcher.next_update(lcdc, &mut self.memory, scan_line);
    }

    fn cycle(&mut self) {
        self.update_status_reg();

        let current_mode = self.current_mode();
        self.interrupt_controller
            .lock()
            .unwrap()
            .ppu_mode_update(current_mode);

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
                        FetcherState::Waiting | FetcherState::Finished => {}
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
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
    Finished,
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
                    FetcherState::Finished
                } else {
                    FetcherState::Working(fetcher, x + 1)
                }
            }
            FetcherState::Finished => FetcherState::Finished,
        }
    }

    fn next_update(&mut self, lcdc: ControlReg, memory: &mut dyn Memory, scan_line: u8) {
        let current = std::mem::take(self);
        let next = current.next(lcdc, memory, scan_line);
        let _ = std::mem::replace(self, next);
    }
}

fn pixel_color_to_screen_color(color: u8) -> [u8; 4] {
    match color {
        0 => [150, 182, 15, 255],
        1 => [135, 167, 15, 255],
        2 => [46, 95, 46, 255],
        3 => [15, 54, 15, 255],
        _ => panic!("Out of range color"),
    }
}

#[allow(dead_code)]
pub fn dump_tiles_to_file(memory: &dyn Memory, path: &str) -> Result<(), std::io::Error> {
    use std::io::Write;

    let file = std::fs::File::create(path)?;
    let mut writer = BufWriter::new(file);

    let addresses: Vec<u16> = (0x8000..0x9800).collect();

    writeln!(&mut writer, "P3")?;
    writeln!(&mut writer, "8 {}", addresses.len() / 2)?;

    for tile in addresses.chunks_exact(16) {
        for byte_addresses in tile.chunks_exact(2) {
            let low = memory.read_memory(byte_addresses[0]);
            let high = memory.read_memory(byte_addresses[1]);
            let pixels = fetcher::bytes_to_pixels(low, high, PixelSource::BackgroundWindow);

            for pixel in &pixels {
                let screen_color = pixel_color_to_screen_color(pixel.color);
                for color_part in &screen_color[..3] {
                    write!(&mut writer, "{} ", color_part)?;
                }
            }
            writeln!(&mut writer)?;
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub fn dump_frame_to_file(frame: &[u8], path: &str) -> Result<(), std::io::Error> {
    use std::io::Write;

    assert_eq!(frame.len(), PIXEL_COUNT);

    let file = std::fs::File::create(path)?;
    let mut writer = BufWriter::new(file);

    writeln!(&mut writer, "P3")?;
    writeln!(&mut writer, "{} {}", SCREEN_WIDTH, SCREEN_HEIGHT)?;

    for y in 0..SCREEN_HEIGHT {
        for x in 0..SCREEN_WIDTH {
            let offset = (y as usize) * (SCREEN_WIDTH as usize) + (x as usize);
            let screen_color = pixel_color_to_screen_color(frame[offset]);
            for color_part in &screen_color[..3] {
                write!(&mut writer, "{} ", color_part)?;
            }
            writeln!(&mut writer)?;
        }
    }

    Ok(())
}
