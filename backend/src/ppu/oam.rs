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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OAMSize {
    _8x8,
    _8x16,
}

impl OAMSize {
    fn height(self) -> u8 {
        match self {
            OAMSize::_8x16 => 16,
            OAMSize::_8x8 => 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Oam {
    pub y_pos: u8,
    pub x_pos: u8,
    pub tile_id: u8,
    pub flags: OAMFlags,
}

impl Oam {
    pub fn read_from_memory(memory: &dyn Memory, addr: u16) -> Self {
        let y_pos = memory.read_memory(addr);
        let x_pos = memory.read_memory(addr + 1);
        let tile_id = memory.read_memory(addr + 2);
        let flags = OAMFlags::from_bits_truncate(memory.read_memory(addr + 3));

        Oam {
            y_pos,
            x_pos,
            tile_id,
            flags,
        }
    }

    pub fn is_y_hitting(&self, scan_line: u8, oam_size: OAMSize) -> bool {
        self.y_pos <= (scan_line + 16) && (scan_line + 16) < (self.y_pos + oam_size.height())
    }

    pub fn get_pixels(&self, memory: &dyn Memory, in_oam_y: u8, oam_size: OAMSize) -> [Pixel; 8] {
        let in_tile_y = if self.flags.contains(OAMFlags::Y_FLIP) {
            oam_size.height() - 1 - in_oam_y
        } else {
            in_oam_y
        };

        let palette = self.flags.contains(OAMFlags::PALETTE_NUMBER) as u8;

        let (real_tile_id, in_tile_y) = match oam_size {
            OAMSize::_8x8 => (self.tile_id, in_tile_y),
            OAMSize::_8x16 if in_tile_y < 8 => (self.tile_id & 0xFE, in_tile_y),
            OAMSize::_8x16 if (8..16).contains(&in_tile_y)  => {
                (self.tile_id | 0x01, in_tile_y - 8)
            }
            _ => unreachable!(),
        };

        let mut pixels = read_tile_pixels(
            memory,
            real_tile_id as u16,
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
