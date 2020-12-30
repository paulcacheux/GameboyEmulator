use log::{error, warn};

use super::MBC;

const BANK_SIZE: usize = 0x4000;

pub struct MBC1 {
    bank_count: usize,
    bank_index: usize,
    ram_index: usize,
    ram_enabled: bool,
    banks: Vec<u8>,
    rams: Vec<u8>,
}

impl MBC1 {
    pub fn new(content: &[u8], rom_size: usize, ram_size_class: u8) -> Self {
        assert_eq!(content.len() % BANK_SIZE, 0);

        let mut banks: Vec<u8> = content.iter().copied().collect();
        banks.resize(rom_size as usize, 0);

        let rams = match ram_size_class {
            0x00 => Vec::new(),
            0x01 => vec![0; 1 << 11],
            0x02 => vec![0; 1 << 13],
            0x03 => vec![0; 1 << 15],
            0x04 => vec![0; 1 << 17],
            0x05 => vec![0; 1 << 16],
            _ => panic!("Unknown RAM Size"),
        };

        warn!("{:#x} {:#x}", banks.len(), rams.len());

        MBC1 {
            bank_count: content.len() / BANK_SIZE,
            bank_index: 1,
            ram_index: 0,
            ram_enabled: false,
            banks,
            rams,
        }
    }
}

impl MBC for MBC1 {
    fn read_memory(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.banks[addr as usize],
            0x4000..=0x7FFF => self.banks[self.bank_index * BANK_SIZE + (addr as usize - 0x4000)],
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    self.rams[self.ram_index * BANK_SIZE + (addr as usize - 0xA000)]
                } else {
                    error!("Read from ram with ram disabled");
                    0xFF
                }
            }
            _ => panic!("Access MBC in non managed space"),
        }
    }

    fn write_memory(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => {
                if value == 0x0A {
                    self.ram_enabled = true;
                } else {
                    self.ram_enabled = false;
                }
            }
            0x2000..=0x3FFF => {
                let value = value & 0x20;
                let mut bank_index = value as usize % self.bank_count;
                if bank_index & 0xF == 0 {
                    bank_index += 1;
                }
                self.bank_index = bank_index;
            }
            0x4000..=0x5FFF => {
                let value = value & 0b11;
                self.ram_index = value as usize;
            }
            0x6000..=0x7FFF => {
                if value != 0 {
                    unimplemented!()
                }
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    self.rams[self.ram_index * BANK_SIZE + (addr as usize - 0xA000)] = value;
                } else {
                    error!("Write to ram with ram disabled");
                }
            }
            _ => panic!("Access MBC in non managed space"),
        }
    }
}
