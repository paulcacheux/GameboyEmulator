use core::panic;

use crate::memory::Memory;

use super::{BG_PALETTE_DATA_ADDR, OAM0_PALETTE_DATA_ADDR, OAM1_PALETTE_DATA_ADDR};

#[derive(Debug, Clone, Copy)]
pub struct Pixel {
    pub color: u8,
    pub source: PixelSource,
}

impl Pixel {
    pub fn through_palette(&self, memory: &dyn Memory) -> u8 {
        let palette_addr = match self.source {
            PixelSource::BackgroundWindow => BG_PALETTE_DATA_ADDR,
            PixelSource::OAM { palette: 0, .. } => OAM0_PALETTE_DATA_ADDR,
            PixelSource::OAM { palette: 1, .. } => OAM1_PALETTE_DATA_ADDR,
            _ => panic!("Out of range oam palette"),
        };
        let palette = memory.read_memory(palette_addr);

        (palette >> (self.color * 2)) & 0b11
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PixelSource {
    BackgroundWindow,
    OAM { palette: u8, bg_priority: bool },
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

pub fn read_tile_pixels(
    memory: &dyn Memory,
    real_tile_id: u16,
    in_tile_y: u8,
    source: PixelSource,
) -> [Pixel; 8] {
    let tile_addr = 0x8000 + real_tile_id * 16;
    let row_addr = tile_addr + (in_tile_y as u16) * 2;

    let byte1 = memory.read_memory(row_addr);
    let byte2 = memory.read_memory(row_addr + 1);

    let pixels = byte_pair_to_pixels(byte1, byte2, source);
    pixels
}
