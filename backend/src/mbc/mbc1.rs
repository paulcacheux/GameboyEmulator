use log::error;

use super::MBC;

const BANK_SIZE: usize = 0x4000;

pub struct MBC1 {
    bank_count: usize,
    bank_index: usize,
    ram_index: usize,
    ram_enabled: bool,
    rom: Vec<u8>,
    ram: Vec<u8>,
}

impl MBC1 {
    pub fn new(content: &[u8], rom_size: usize, ram_size: usize) -> Self {
        assert_eq!(content.len() % BANK_SIZE, 0);

        let mut rom: Vec<u8> = content.to_vec();
        rom.resize(rom_size, 0);

        let ram = vec![0; ram_size];

        MBC1 {
            bank_count: content.len() / BANK_SIZE,
            bank_index: 1,
            ram_index: 0,
            ram_enabled: false,
            rom,
            ram,
        }
    }
}

impl MBC for MBC1 {
    fn read_memory(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.rom[addr as usize],
            0x4000..=0x7FFF => self.rom[self.bank_index * BANK_SIZE + (addr as usize - 0x4000)],
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    self.ram[self.ram_index * BANK_SIZE + (addr as usize - 0xA000)]
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
                self.ram_enabled = value == 0x0A;
            }
            0x2000..=0x3FFF => {
                let value = value & 0b11111;
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
                    self.ram[self.ram_index * BANK_SIZE + (addr as usize - 0xA000)] = value;
                } else {
                    error!("Write to ram with ram disabled");
                }
            }
            _ => panic!("Access MBC in non managed space"),
        }
    }
}
