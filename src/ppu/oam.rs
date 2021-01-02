use bitflags::bitflags;

use crate::memory::Memory;

use super::pixel::Pixel;
use super::pixel::{read_tile_pixels, PixelSource};

bitflags! {
    pub struct OAMFlags: u8 {
        const OBJ_TO_BG_PRIORITY = 1 << 7;
        const Y_FLIP = 1 << 6;
        const X_FLIP = 1 << 5;
        const PALETTE_NUMBER = 1 << 4;
    }
}

#[derive(Debug, Clone)]
pub struct OAM {
    pub y_pos: u8,
    pub x_pos: u8,
    pub tile_id: u8,
    pub flags: OAMFlags,
}

impl OAM {
    pub fn read_from_memory(memory: &dyn Memory, addr: u16) -> Self {
        let y_pos = memory.read_memory(addr);
        let x_pos = memory.read_memory(addr + 1);
        let tile_id = memory.read_memory(addr + 2);
        let flags = OAMFlags::from_bits_truncate(memory.read_memory(addr + 3));

        OAM {
            y_pos,
            x_pos,
            tile_id,
            flags,
        }
    }

    pub fn is_y_hitting(&self, scan_line: u8) -> bool {
        self.y_pos <= (scan_line + 16) && (scan_line + 16) < (self.y_pos + 8)
    }

    pub fn get_pixels(&self, memory: &dyn Memory, in_oam_y: u8) -> [Pixel; 8] {
        let in_tile_y = if self.flags.contains(OAMFlags::Y_FLIP) {
            7 - in_oam_y
        } else {
            in_oam_y
        };

        let palette = self.flags.contains(OAMFlags::PALETTE_NUMBER) as u8;

        let mut pixels = read_tile_pixels(
            memory,
            self.tile_id as u16,
            in_tile_y,
            PixelSource::OAM {
                palette,
                bg_priority: self.flags.contains(OAMFlags::OBJ_TO_BG_PRIORITY),
            },
        );

        if self.flags.contains(OAMFlags::X_FLIP) {
            pixels.reverse();
        }
        pixels
    }
}
