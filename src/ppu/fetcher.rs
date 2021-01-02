use std::marker::PhantomData;

use crate::memory::Memory;

use super::pixel::{read_tile_pixels, Pixel, PixelSource};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetcherKind {
    Background,
    Window,
}

#[derive(Debug, Clone)]
pub struct Fetcher<M: Memory> {
    map_addr: u16,
    addressing_mode: AddressingMode,
    tile_x: u8,
    tile_y: u8,
    sub_y: u8,
    pub kind: FetcherKind,
    phantom_data: PhantomData<M>,
}

impl<M: Memory> Fetcher<M> {
    pub fn new_window(
        map_addr: u16,
        addressing_mode: AddressingMode,
        window_scan_line: u8,
    ) -> Self {
        let tile_y = window_scan_line / 8;
        let sub_y = window_scan_line % 8;

        Fetcher {
            map_addr,
            addressing_mode,
            tile_x: 0,
            tile_y,
            sub_y,
            kind: FetcherKind::Window,
            phantom_data: PhantomData,
        }
    }

    pub fn new_background(
        map_addr: u16,
        addressing_mode: AddressingMode,
        scroll_x: u8,
        scroll_y: u8,
        scan_line: u8,
    ) -> Self {
        let total_y_scroll = scan_line.wrapping_add(scroll_y);
        let tile_x = scroll_x / 8;
        let tile_y = total_y_scroll / 8;
        let sub_y = total_y_scroll % 8;
        /* println!(
            "Init fetcher: tile_x = {}, tile_y = {}, sub_y = {}",
            tile_x, tile_y, sub_y
        ); */

        Fetcher {
            map_addr,
            addressing_mode,
            tile_x,
            tile_y,
            sub_y,
            kind: FetcherKind::Background,
            phantom_data: PhantomData,
        }
    }

    pub fn fetch_pixels(&mut self, memory: &M) -> [Pixel; 8] {
        let offset = (self.tile_y as u16) * 32 + (self.tile_x as u16);
        let tile_id = memory.read_memory(self.map_addr + offset);

        let real_tile_id = match self.addressing_mode {
            AddressingMode::From8000 => tile_id as u16,
            AddressingMode::From8800 => {
                if tile_id < 128 {
                    tile_id as u16 + 256
                } else {
                    tile_id as u16
                }
            }
        };

        self.tile_x = (self.tile_x + 1) % 32;

        read_tile_pixels(
            memory,
            real_tile_id,
            self.sub_y,
            PixelSource::BackgroundWindow,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AddressingMode {
    From8000,
    From8800,
}
