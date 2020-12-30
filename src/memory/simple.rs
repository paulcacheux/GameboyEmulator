use log::{debug, error};

use super::MBC;

pub struct Simple {
    rom: Box<[u8; 0x8000]>,
}

impl Simple {
    pub fn new(content: &[u8]) -> Self {
        assert!(content.len() <= 0x8000);
        let mut mbc = Simple {
            rom: Box::new([0; 0x8000]),
        };
        mbc.rom[..content.len()].copy_from_slice(content);
        mbc
    }
}

impl MBC for Simple {
    fn read_memory(&self, addr: u16) -> u8 {
        match addr {
            0x0100..=0x7FFF => self.rom[addr as usize],
            _ => {
                debug!("Read from uncontrolled MBC space");
                0xFF
            }
        }
    }

    fn write_memory(&mut self, _addr: u16, _value: u8) {
        error!("Write to ROM error")
    }
}
