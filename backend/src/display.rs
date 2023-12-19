use crate::{memory::Memory, ppu::pixel::PixelSource};

use super::ppu::pixel::byte_pair_to_pixels;
use super::ppu::PIXEL_COUNT;

#[derive(Debug)]
pub struct Display {
    frame: [u8; PIXEL_COUNT],
}

impl Default for Display {
    fn default() -> Self {
        Display {
            frame: [0; PIXEL_COUNT],
        }
    }
}

impl Display {
    pub fn push_frame(&mut self, frame: &[u8]) {
        assert_eq!(frame.len(), self.frame.len());
        self.frame.copy_from_slice(frame);
    }

    pub fn draw_into_fb(&self, fb: &mut [u8]) {
        assert_eq!(PIXEL_COUNT * 4, fb.len());

        for (i, pixel) in fb.chunks_exact_mut(4).enumerate() {
            let color = self.frame[i];
            pixel.copy_from_slice(&pixel_color_to_screen_color(color));
        }
    }

    pub fn draw_tiles_into_fb(memory: &dyn Memory, fb: &mut [u8]) {
        let addresses: Vec<u16> = (0x8000..0x9800).collect();
        for (tile_id, tile) in addresses.chunks_exact(16).enumerate() {
            let tile_y = tile_id / 20;
            let tile_x = tile_id % 20;

            for (y, byte_addresses) in tile.chunks_exact(2).enumerate() {
                let low = memory.read_memory(byte_addresses[0]);
                let high = memory.read_memory(byte_addresses[1]);
                let pixels = byte_pair_to_pixels(low, high, PixelSource::BackgroundWindow);

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
        1 => [170, 170, 170, 255],
        2 => [85, 85, 85, 255],
        3 => [0, 0, 0, 255],
        _ => panic!("Out of range color"),
    }
}
