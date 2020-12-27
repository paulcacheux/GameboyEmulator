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
        let tile_x = scroll_x / 8;
        let tile_y = ((scan_line + scroll_y) & 0xFF) / 8;
        let sub_y = (scan_line + scroll_y) % 8;

        Fetcher {
            map_addr,
            addressing_mode,
            tile_x,
            tile_y,
            sub_y,
        }
    }

    pub fn fetch_pixels<M: Memory>(&mut self, memory: &mut M) -> [Pixel; 8] {
        let offset = self.tile_y * 32 + self.tile_x;
        let tile_id = memory.read_memory(self.map_addr) + offset;

        let tile_addr = match self.addressing_mode {
            AddressingMode::From8000 => 0x8000 + (tile_id as u16) * 16,
            AddressingMode::From8800 => {
                let tile_id = tile_id as i8;
                if tile_id >= 0 {
                    0x9000 + (tile_id as u16) * 16
                } else {
                    0x9000 - (-tile_id as u16) * 16
                }
            }
        };

        let row_addr = tile_addr + (self.sub_y as u16) * 2;

        let byte1 = memory.read_memory(row_addr);
        let byte2 = memory.read_memory(row_addr + 1);

        let mut pixels = [Pixel {
            color: 0,
            source: PixelSource::BackgroundWindow,
        }; 8];

        for (index, bit) in (0..8).rev().enumerate() {
            let bit_low_value = (byte1 >> bit) & 0x1;
            let bit_high_value = (byte2 >> bit) & 0x1;

            let color_value = (bit_high_value << 1) | bit_low_value;
            pixels[index].color = color_value;
        }

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
