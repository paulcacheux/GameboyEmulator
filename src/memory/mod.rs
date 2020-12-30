use std::{rc::Rc, sync::RwLock};

#[derive(Debug, Clone)]
pub struct MMU {
    bootstrap_rom: Box<[u8; 0x8000]>,
    rom: Box<[u8; 0x8000]>,
    vram: Box<[u8; 0x2000]>,
    eram: Box<[u8; 0x2000]>,
    wram: Box<[u8; 0x2000]>,
    oam: Box<[u8; 0xA0]>,
    io_regs: Box<[u8; 0x80]>,
    hram: Box<[u8; 0x7F]>,
    ie_reg: u8,
}

impl MMU {
    pub fn new() -> Self {
        MMU {
            bootstrap_rom: Box::new([0; 0x8000]),
            rom: Box::new([0; 0x8000]),
            vram: Box::new([0; 0x2000]),
            eram: Box::new([0; 0x2000]),
            wram: Box::new([0; 0x2000]),
            oam: Box::new([0; 0xA0]),
            io_regs: Box::new([0; 0x80]),
            hram: Box::new([0; 0x7F]),
            ie_reg: 0,
        }
    }

    pub fn write_rom(&mut self, slice: &[u8]) {
        self.rom[..slice.len()].copy_from_slice(slice);
    }

    pub fn write_bootstrap_rom(&mut self, slice: &[u8]) {
        self.bootstrap_rom[..slice.len()].copy_from_slice(slice);
    }

    pub fn mounted_rom(&self) -> &[u8] {
        if self.read_memory(BOOTSTRAP_ROM_MOUNT_CONTROL_ADDR) != 0 {
            self.rom.as_ref()
        } else {
            self.bootstrap_rom.as_ref()
        }
    }

    pub fn mounted_rom_mut(&mut self) -> &mut [u8] {
        if self.read_memory(BOOTSTRAP_ROM_MOUNT_CONTROL_ADDR) != 0 {
            self.rom.as_mut()
        } else {
            self.bootstrap_rom.as_mut()
        }
    }
}

const SERIAL_TRANSFER_DATA_ADDR: u16 = 0xFF01;
const SERIAL_TRANSFER_CONTROL_ADDR: u16 = 0xFF02;
const BOOTSTRAP_ROM_MOUNT_CONTROL_ADDR: u16 = 0xFF50;

impl Memory for MMU {
    fn read_memory(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x00FF => self.mounted_rom()[addr as usize],
            0x0100..=0x7FFF => self.rom[addr as usize],
            0x8000..=0x9FFF => self.vram[addr as usize - 0x8000],
            0xA000..=0xBFFF => self.eram[addr as usize - 0xA000],
            0xC000..=0xDFFF => self.wram[addr as usize - 0xC000],
            0xE000..=0xFDFF => self.wram[addr as usize - 0xE000],
            0xFE00..=0xFE9F => self.oam[addr as usize - 0xFE00],
            0xFEA0..=0xFEFF => panic!("Unusable space"),
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
            0x0000..=0x00FF => self.mounted_rom_mut()[addr as usize] = value,
            0x0100..=0x7FFF => self.rom[addr as usize] = value,
            0x8000..=0x9FFF => self.vram[addr as usize - 0x8000] = value,
            0xA000..=0xBFFF => self.eram[addr as usize - 0xA000] = value,
            0xC000..=0xDFFF => self.wram[addr as usize - 0xC000] = value,
            0xE000..=0xFDFF => self.wram[addr as usize - 0xE000] = value,
            0xFE00..=0xFE9F => self.oam[addr as usize - 0xFE00] = value,
            0xFEA0..=0xFEFF => panic!("Unusable space"),
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
