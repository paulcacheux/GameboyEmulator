use crate::memory::Memory;

#[derive(Debug, Clone)]
pub struct Fetcher {
    map_addr: u16,
    addressing_mode: AddressingMode,
    tile_x: u8,
    tile_y: u8,
    sub_y: u8,
}

impl Fetcher {
    pub fn new(
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
        }
    }

    pub fn fetch_pixels<M: Memory>(&mut self, memory: &mut M) -> [Pixel; 8] {
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

        let tile_addr = 0x8000 + real_tile_id * 16;

        let row_addr = tile_addr + (self.sub_y as u16) * 2;

        let byte1 = memory.read_memory(row_addr);
        let byte2 = memory.read_memory(row_addr + 1);

        let pixels = byte_pair_to_pixels(byte1, byte2, PixelSource::BackgroundWindow);

        self.tile_x = (self.tile_x + 1) % 32;
        pixels
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Pixel {
    pub color: u8,
    pub source: PixelSource,
}

#[derive(Debug, Clone, Copy)]
pub enum PixelSource {
    BackgroundWindow,
}

#[derive(Debug, Clone, Copy)]
pub enum AddressingMode {
    From8000,
    From8800,
}

pub fn byte_pair_to_pixels(low: u8, high: u8, source: PixelSource) -> [Pixel; 8] {
    let mut pixels = [Pixel { color: 0, source }; 8];

    for (index, bit) in (0..8).rev().enumerate() {
        let bit_low_value = (low >> bit) & 0x1;
        let bit_high_value = (high >> bit) & 0x1;

        let color_value = (bit_high_value << 1) | bit_low_value;
        pixels[index].color = color_value;
    }
    pixels
}
