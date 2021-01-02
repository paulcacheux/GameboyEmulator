use std::collections::VecDeque;

use crate::memory::Memory;

use super::{
    fetcher::{Fetcher, Pixel},
    LCD_SCROLL_X_ADDR, LCD_SCROLL_Y_ADDR, LCD_WINDOW_X_POSITION_ADDR, LCD_WINDOW_Y_POSITION_ADDR,
};
use super::{
    fetcher::{FetcherKind, PixelSource},
    ControlReg, LCD_CONTROL_REG_ADDR,
};

#[derive(Debug, Clone)]
pub struct PixelFIFO<M: Memory> {
    background_window_fetcher: Option<Fetcher<M>>,
    fifo: VecDeque<Pixel>,
    memory: M,
    window_scan_line: Option<u8>,
    current_scan_line: u8,
    current_x: u8,
}

impl<M: Memory> PixelFIFO<M> {
    pub fn new(memory: M) -> Self {
        PixelFIFO {
            background_window_fetcher: None,
            fifo: VecDeque::new(),
            memory,
            window_scan_line: None,
            current_scan_line: 0,
            current_x: 0,
        }
    }

    fn control_reg(&self) -> ControlReg {
        ControlReg::from_bits_truncate(self.memory.read_memory(LCD_CONTROL_REG_ADDR))
    }

    fn current_requested_mode(&self) -> Option<FetcherKind> {
        let lcdc = self.control_reg();
        if lcdc.contains(ControlReg::BG_WINDOW_DISPLAY_PRIORITY) {
            if lcdc.contains(ControlReg::WINDOW_DISPLAY_ENABLE) {
                let window_y_pos = self.memory.read_memory(LCD_WINDOW_Y_POSITION_ADDR);
                let window_x_pos = self.memory.read_memory(LCD_WINDOW_X_POSITION_ADDR);
                if window_x_pos < 7 || window_x_pos == 166 {
                    unimplemented!("Unimplemented window x pos {}", window_x_pos)
                }
                let window_x_pos = window_x_pos - 7;

                if self.current_scan_line >= window_y_pos && self.current_x >= window_x_pos {
                    Some(FetcherKind::Window)
                } else {
                    Some(FetcherKind::Background)
                }
            } else {
                Some(FetcherKind::Background)
            }
        } else {
            None
        }
    }

    fn match_fetcher_mode(&mut self) {
        let requested_state = self.current_requested_mode();
        if requested_state != self.background_window_fetcher.as_ref().map(|f| f.kind) {
            let lcdc = self.control_reg();
            let addressing_mode = lcdc.addressing_mode();

            self.background_window_fetcher = match requested_state {
                Some(FetcherKind::Background) => Some(Fetcher::new_background(
                    lcdc.background_tile_map_addr(),
                    addressing_mode,
                    self.memory.read_memory(LCD_SCROLL_X_ADDR),
                    self.memory.read_memory(LCD_SCROLL_Y_ADDR),
                    self.current_scan_line,
                )),
                Some(FetcherKind::Window) => {
                    let scan_line = self.window_scan_line.unwrap_or(0);
                    self.window_scan_line = Some(scan_line + 1);

                    Some(Fetcher::new_window(
                        lcdc.window_tile_map_addr(),
                        addressing_mode,
                        scan_line,
                    ))
                }
                None => None,
            };

            self.fifo.clear();
        }
    }

    pub fn begin_of_line(&mut self, scan_line: u8) {
        self.fifo.clear();
        self.current_x = 0;
        self.current_scan_line = scan_line;

        self.match_fetcher_mode();
        self.fill_fifo_if_needed();

        if self.background_window_fetcher.as_ref().map(|f| f.kind) == Some(FetcherKind::Background)
        {
            self.pop_first_pixels(self.memory.read_memory(LCD_SCROLL_X_ADDR));
        }
    }

    fn pop_first_pixels(&mut self, scroll_x: u8) {
        for _ in 0..(scroll_x % 8) {
            self.fifo.pop_front();
        }
    }

    pub fn next_pixel(&mut self) -> Pixel {
        self.match_fetcher_mode();

        let pixel = if self.background_window_fetcher.is_some() {
            self.fill_fifo_if_needed();
            self.fifo.pop_front().unwrap()
        } else {
            Pixel {
                color: 0x00,
                source: PixelSource::BackgroundWindow,
            }
        };
        self.current_x += 1;
        pixel
    }

    pub fn end_of_line(&mut self) {
        self.background_window_fetcher = None;
    }

    pub fn end_of_frame(&mut self) {
        self.window_scan_line = None;
    }

    fn fill_fifo_if_needed(&mut self) {
        if let Some(fetcher) = self.background_window_fetcher.as_mut() {
            if self.fifo.len() < 8 {
                self.fifo.extend(&fetcher.fetch_pixels(&self.memory));
            }
        }
    }
}
