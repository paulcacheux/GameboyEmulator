use std::collections::VecDeque;

use crate::memory::Memory;

use super::{
    fetcher::Fetcher,
    oam::{OAMSize, Oam},
    pixel::{Pixel, PixelSource},
    LCD_SCROLL_X_ADDR, LCD_SCROLL_Y_ADDR, LCD_WINDOW_X_POSITION_ADDR, LCD_WINDOW_Y_POSITION_ADDR,
};
use super::{fetcher::FetcherKind, ControlReg, LCD_CONTROL_REG_ADDR};

#[derive(Debug, Clone)]
pub struct PixelFIFO<M: Memory> {
    background_window_fetcher: Option<Fetcher<M>>,
    objects: Vec<Oam>,
    oam_size: OAMSize,

    background_fifo: VecDeque<Pixel>,
    oam_fifo: VecDeque<Pixel>,

    memory: M,
    window_scan_line: Option<u8>,
    current_scan_line: u8,
    current_x: u8,
}

impl<M: Memory> PixelFIFO<M> {
    pub fn new(memory: M) -> Self {
        PixelFIFO {
            background_window_fetcher: None,
            objects: Vec::new(),
            oam_size: OAMSize::_8x8,

            background_fifo: VecDeque::new(),
            oam_fifo: VecDeque::new(),

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

            self.background_fifo.clear();
        }
    }

    fn find_oams(&mut self) {
        self.oam_size = if self.control_reg().contains(ControlReg::OBJ_SIZE) {
            OAMSize::_8x16
        } else {
            OAMSize::_8x8
        };

        for oam_addr in (0xFE00..0xFEA0).step_by(4) {
            let oam = Oam::read_from_memory(&self.memory, oam_addr);
            if oam.is_y_hitting(self.current_scan_line, self.oam_size) {
                self.objects.push(oam);
            }

            if self.objects.len() >= 10 {
                break;
            }
        }
    }

    pub fn begin_of_line(&mut self, scan_line: u8) {
        self.current_scan_line = scan_line;
        self.background_fifo.clear();
        self.oam_fifo.clear();
    }

    pub fn end_of_oam_search(&mut self) {
        self.find_oams();
    }

    pub fn begin_lcd_transfer(&mut self) {
        self.current_x = 0;

        self.match_fetcher_mode();
        self.fill_background_fifo_if_needed();

        if self.background_window_fetcher.as_ref().map(|f| f.kind) == Some(FetcherKind::Background)
        {
            self.pop_first_pixels(self.memory.read_memory(LCD_SCROLL_X_ADDR));
        }
    }

    fn pop_first_pixels(&mut self, scroll_x: u8) {
        for _ in 0..(scroll_x % 8) {
            self.background_fifo.pop_front();
        }
    }

    pub fn next_pixel(&mut self) -> Pixel {
        self.match_fetcher_mode();

        let background_pixel = if self.background_window_fetcher.is_some() {
            self.fill_background_fifo_if_needed();
            self.background_fifo.pop_front().unwrap()
        } else {
            Pixel {
                color: 0x00,
                source: PixelSource::BackgroundWindow,
            }
        };

        let oam_pixel = {
            self.fill_oam_fifo_if_needed();
            self.oam_fifo.pop_front().unwrap()
        };

        self.current_x += 1;
        choose_pixel(
            background_pixel,
            oam_pixel,
            self.control_reg().contains(ControlReg::OBJ_DISPLAY_ENABLE),
        )
    }

    pub fn end_of_line(&mut self) {
        self.background_window_fetcher = None;
        self.objects.clear();
        self.background_fifo.clear();
        self.oam_fifo.clear();
    }

    pub fn end_of_frame(&mut self) {
        self.window_scan_line = None;
    }

    fn fill_background_fifo_if_needed(&mut self) {
        if let Some(fetcher) = self.background_window_fetcher.as_mut() {
            if self.background_fifo.len() < 8 {
                self.background_fifo
                    .extend(&fetcher.fetch_pixels(&self.memory));
            }
        }
    }

    fn fill_oam_fifo_if_needed(&mut self) {
        if self.oam_fifo.len() < 8 {
            self.oam_fifo.resize_with(8, || Pixel {
                color: 0,
                source: PixelSource::OAM {
                    palette: 0,
                    bg_priority: true,
                },
            });
        }

        for oam in &self.objects {
            if self.current_x + 8 == oam.x_pos {
                let in_oam_y = self.current_scan_line + 16 - oam.y_pos;
                let pixels = oam.get_pixels(&self.memory, in_oam_y, self.oam_size);

                for (i, &pixel) in pixels.into_iter().enumerate() {
                    if self.oam_fifo[i].color == 0 {
                        self.oam_fifo[i] = pixel;
                    }
                }
            }
        }
    }
}

fn choose_pixel(bg_pixel: Pixel, oam_pixel: Pixel, obj_enable: bool) -> Pixel {
    match (bg_pixel, oam_pixel) {
        (
            Pixel {
                color: bg_color,
                source: PixelSource::BackgroundWindow,
            },
            Pixel {
                color: oam_color,
                source: PixelSource::OAM { bg_priority, .. },
            },
        ) => {
            if !obj_enable || oam_color == 0 {
                bg_pixel
            } else if !bg_priority {
                oam_pixel
            } else if bg_color != 0 {
                // && bg_priority
                bg_pixel
            } else {
                oam_pixel
            }
        }
        _ => panic!("Bad mixing between pixels"),
    }
}
