use std::collections::VecDeque;

use crate::memory::Memory;

use super::ControlReg;
use super::{
    fetcher::{AddressingMode, Fetcher, Pixel},
    LCD_SCROLL_X_ADDR, LCD_SCROLL_Y_ADDR,
};

#[derive(Debug, Clone)]
pub struct PixelFIFO<M: Memory> {
    fetcher: Option<Fetcher<M>>,
    fifo: VecDeque<Pixel>,
}

impl<M: Memory> PixelFIFO<M> {
    pub fn new() -> Self {
        PixelFIFO {
            fetcher: None,
            fifo: VecDeque::new(),
        }
    }

    pub fn begin_of_line(&mut self, lcdc: ControlReg, memory: &M, scan_line: u8) {
        self.fifo.clear();

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

        self.fetcher = Some(fetcher);
        self.fill_fifo_if_needed(memory);
    }

    pub fn pop_first_pixels(&mut self, scroll_x: u8) {
        for _ in 0..(scroll_x % 8) {
            self.fifo.pop_front();
        }
    }

    pub fn next_pixel(&mut self, memory: &M) -> Pixel {
        self.fill_fifo_if_needed(memory);
        self.fifo.pop_front().unwrap()
    }

    pub fn end_of_line(&mut self) {
        self.fetcher = None;
    }

    fn fill_fifo_if_needed(&mut self, memory: &M) {
        if self.fifo.len() < 8 {
            let fetcher = self.fetcher.as_mut().unwrap();
            self.fifo.extend(&fetcher.fetch_pixels(memory));
        }
    }
}
