use crate::{interrupt::InterruptControllerPtr, memory::Memory};
use bitflags::bitflags;

mod fetcher;
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

pub const SCREEN_WIDTH: u8 = 160;
pub const SCREEN_HEIGHT: u8 = 144;
const PIXEL_COUNT: usize = (SCREEN_WIDTH as usize) * (SCREEN_HEIGHT as usize);

const SCAN_LINE_COUNT: u8 = SCREEN_HEIGHT + 10;
const DOT_PER_LINE_COUNT: u32 = 80 + 172 + 204;

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

    scan_line: u8,
    dot_in_line: u32,
    state: PPUState,
    int_cond_met: bool,

    pub previous_frame: [u8; PIXEL_COUNT],
    pub frame: [u8; PIXEL_COUNT],

    pixel_fifo: PixelFIFO<M>,
}

impl<M: Memory> PPU<M> {
    pub fn new(memory: M, interrupt_controller: InterruptControllerPtr) -> Self {
        PPU {
            memory,
            interrupt_controller,

            scan_line: 0,
            dot_in_line: 0,
            state: PPUState::OAMSearchInit,
            int_cond_met: false,

            previous_frame: [0; PIXEL_COUNT],
            frame: [0; PIXEL_COUNT],

            pixel_fifo: PixelFIFO::new(),
        }
    }

    pub fn draw_into_fb(&self, fb: &mut [u8]) {
        assert_eq!(PIXEL_COUNT * 4, fb.len());

        for (i, pixel) in fb.chunks_exact_mut(4).enumerate() {
            let color = self.previous_frame[i];
            pixel.copy_from_slice(&pixel_color_to_screen_color(color));
        }
    }

    pub fn draw_tiles_into_fb(&self, fb: &mut [u8]) {
        let addresses: Vec<u16> = (0x8000..0x9800).collect();
        for (tile_id, tile) in addresses.chunks_exact(16).enumerate() {
            let tile_y = tile_id / 20;
            let tile_x = tile_id % 20;

            for (y, byte_addresses) in tile.chunks_exact(2).enumerate() {
                let low = self.memory.read_memory(byte_addresses[0]);
                let high = self.memory.read_memory(byte_addresses[1]);
                let pixels = fetcher::byte_pair_to_pixels(low, high, PixelSource::BackgroundWindow);

                for (x, pixel) in pixels.iter().enumerate() {
                    let screen_color = pixel_color_to_screen_color(pixel.color);

                    let final_y = tile_y * 8 + y;
                    let final_x = tile_x * 8 + x;
                    let offset = (final_y * (20 * 8) + final_x) * 4;

                    fb[offset..(offset + 4)].copy_from_slice(&screen_color);
                }
            }
        }
    }

    fn control_reg(&self) -> ControlReg {
        ControlReg::from_bits(self.memory.read_memory(LCD_CONTROL_REG_ADDR))
            .expect("Failed to read control_reg")
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
        for (pixel, previous_pixel) in self.frame.iter_mut().zip(self.previous_frame.iter_mut()) {
            *previous_pixel = *pixel;
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
            PPUState::OAMSearchInit => {}
            PPUState::OAMSearch => {}
            PPUState::TransferInit => {
                self.pixel_fifo
                    .begin_of_line(self.control_reg(), &self.memory, self.scan_line);

                let scroll_x = self.memory.read_memory(LCD_SCROLL_X_ADDR);
                self.pixel_fifo.pop_first_pixels(scroll_x);
            }
            PPUState::Transfer { x } => {
                assert!(x < 160);

                let pixel = self.pixel_fifo.next_pixel(&self.memory);

                let offset = (self.scan_line as usize) * (SCREEN_WIDTH as usize) + (x as usize);

                let bg_palette = self.memory.read_memory(BG_PALETTE_DATA_ADDR);
                let actual_color = (bg_palette >> (pixel.color * 2)) & 0b11;

                self.frame[offset] = actual_color;
            }
            PPUState::PostTransfer => {}
            PPUState::HBlankInit => {
                self.pixel_fifo.end_of_line();
            }
            PPUState::HBlank => {}
            PPUState::VBlankInit => self
                .interrupt_controller
                .lock()
                .unwrap()
                .trigger_vblank_int(),
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
    OAMSearchInit,
    OAMSearch,
    TransferInit,
    Transfer { x: u8 },
    PostTransfer,
    HBlankInit,
    HBlank,
    VBlankInit,
    VBlank,
}

impl Default for PPUState {
    fn default() -> Self {
        PPUState::OAMSearchInit
    }
}

impl PPUState {
    fn current_state(dot: u32, scan_line: u8) -> Self {
        assert!(scan_line < SCAN_LINE_COUNT);
        assert!(dot < 456);

        if scan_line < SCREEN_HEIGHT {
            match dot {
                0 => PPUState::OAMSearchInit,
                1..=79 => PPUState::OAMSearch,
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
            PPUState::OAMSearchInit => Mode::OAMSearch,
            PPUState::OAMSearch => Mode::OAMSearch,
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

fn pixel_color_to_screen_color(color: u8) -> [u8; 4] {
    /*
    // green
    match color {
        0 => [150, 182, 15, 255],
        1 => [135, 167, 15, 255],
        2 => [46, 95, 46, 255],
        3 => [15, 54, 15, 255],
        _ => panic!("Out of range color"),
    }
    */

    // gray
    match color {
        0 => [255, 255, 255, 255],
        1 => [169, 169, 169, 255],
        2 => [84, 84, 84, 255],
        3 => [0, 0, 0, 255],
        _ => panic!("Out of range color"),
    }
}
