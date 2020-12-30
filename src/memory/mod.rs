use std::{rc::Rc, sync::RwLock};

use log::debug;

mod mbc1;
mod simple;
use mbc1::MBC1;
use simple::Simple as SimpleMBC;

pub struct MMU {
    bootstrap_rom: Box<[u8; 0x100]>,
    mbc: Box<dyn MBC>,
    vram: Box<[u8; 0x2000]>,
    wram: Box<[u8; 0x2000]>,
    oam: Box<[u8; 0xA0]>,
    io_regs: Box<[u8; 0x80]>,
    hram: Box<[u8; 0x7F]>,
    ie_reg: u8,
}

impl MMU {
    pub fn new(mbc: Box<dyn MBC>) -> Self {
        MMU {
            bootstrap_rom: Box::new([0; 0x100]),
            mbc,
            vram: Box::new([0; 0x2000]),
            wram: Box::new([0; 0x2000]),
            oam: Box::new([0; 0xA0]),
            io_regs: Box::new([0; 0x80]),
            hram: Box::new([0; 0x7F]),
            ie_reg: 0,
        }
    }

    pub fn write_bootstrap_rom(&mut self, slice: &[u8]) {
        self.bootstrap_rom[..slice.len()].copy_from_slice(slice);
    }

    pub fn read_mounted_rom(&self, addr: u16) -> u8 {
        if self.read_memory(BOOTSTRAP_ROM_MOUNT_CONTROL_ADDR) != 0 {
            self.mbc.read_memory(addr)
        } else {
            self.bootstrap_rom[addr as usize]
        }
    }

    pub fn write_mounted_rom(&mut self, addr: u16, value: u8) {
        if self.read_memory(BOOTSTRAP_ROM_MOUNT_CONTROL_ADDR) != 0 {
            self.mbc.write_memory(addr, value);
        } else {
            self.bootstrap_rom[addr as usize] = value;
        }
    }

    pub fn unmount_bootstrap_rom(&mut self) {
        self.write_memory(BOOTSTRAP_ROM_MOUNT_CONTROL_ADDR, 1);
    }
}

const SERIAL_TRANSFER_DATA_ADDR: u16 = 0xFF01;
const SERIAL_TRANSFER_CONTROL_ADDR: u16 = 0xFF02;
const BOOTSTRAP_ROM_MOUNT_CONTROL_ADDR: u16 = 0xFF50;

impl Memory for MMU {
    fn read_memory(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x00FF => self.read_mounted_rom(addr),
            0x0100..=0x7FFF => self.mbc.read_memory(addr),
            0x8000..=0x9FFF => self.vram[addr as usize - 0x8000],
            0xA000..=0xBFFF => self.mbc.read_memory(addr),
            0xC000..=0xDFFF => self.wram[addr as usize - 0xC000],
            0xE000..=0xFDFF => self.wram[addr as usize - 0xE000],
            0xFE00..=0xFE9F => self.oam[addr as usize - 0xFE00],
            0xFEA0..=0xFEFF => {
                debug!("Unusable space {:#x}", addr);
                0xFF
            }
            0xFF00..=0xFF7F => self.io_regs[addr as usize - 0xFF00],
            0xFF80..=0xFFFE => self.hram[addr as usize - 0xFF80],
            0xFFFF => self.ie_reg,
        }
    }

    fn write_memory(&mut self, addr: u16, value: u8) {
        // Used for test roms output
        if addr == SERIAL_TRANSFER_CONTROL_ADDR && value == 0x81 {
            print!("{}", self.read_memory(SERIAL_TRANSFER_DATA_ADDR) as char);
        }

        match addr {
            0x0000..=0x00FF => self.write_mounted_rom(addr, value),
            0x0100..=0x7FFF => self.mbc.write_memory(addr, value),
            0x8000..=0x9FFF => self.vram[addr as usize - 0x8000] = value,
            0xA000..=0xBFFF => self.mbc.write_memory(addr, value),
            0xC000..=0xDFFF => self.wram[addr as usize - 0xC000] = value,
            0xE000..=0xFDFF => self.wram[addr as usize - 0xE000] = value,
            0xFE00..=0xFE9F => self.oam[addr as usize - 0xFE00] = value,
            0xFEA0..=0xFEFF => {
                debug!("Write to unusable space {:#x}", addr)
            }
            0xFF00..=0xFF7F => self.io_regs[addr as usize - 0xFF00] = value,
            0xFF80..=0xFFFE => self.hram[addr as usize - 0xFF80] = value,
            0xFFFF => self.ie_reg = value,
        }
    }
}

pub trait Memory {
    fn read_memory(&self, addr: u16) -> u8;
    fn write_memory(&mut self, addr: u16, value: u8);
}

impl<M: Memory> Memory for Rc<RwLock<M>> {
    fn read_memory(&self, addr: u16) -> u8 {
        self.read().unwrap().read_memory(addr)
    }

    fn write_memory(&mut self, addr: u16, value: u8) {
        self.write().unwrap().write_memory(addr, value);
    }
}

pub trait MBC {
    fn read_memory(&self, addr: u16) -> u8;
    fn write_memory(&mut self, addr: u16, value: u8);
}

pub fn build_mbc(content: &[u8]) -> Box<dyn MBC> {
    const CARTRIDGE_TYPE_ADDR: usize = 0x0147;
    const CARTRIDGE_ROM_SIZE_ADDR: usize = 0x0148;
    const CARTRIDGE_RAM_SIZE_ADDR: usize = 0x0149;

    let rom_size = (1 << 15) << content[CARTRIDGE_ROM_SIZE_ADDR];
    assert_eq!(rom_size, content.len());

    let ram_size = match content[CARTRIDGE_RAM_SIZE_ADDR] {
        0x00 => 0,
        0x01 => 11,
        0x02 => 13,
        0x03 => 15,
        0x04 => 17,
        0x05 => 16,
        _ => panic!("Unknown RAM Size"),
    };

    match content[CARTRIDGE_TYPE_ADDR] {
        0x00 => Box::new(SimpleMBC::new(content)),
        0x01 => Box::new(MBC1::new(content, rom_size, 0)),
        0x02 | 0x03 => Box::new(MBC1::new(content, rom_size, ram_size)),
        _ => unimplemented!(),
    }
}
