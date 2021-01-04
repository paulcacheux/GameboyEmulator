use std::sync::{Arc, RwLock};

use log::{debug, error};

mod dma;
mod mbc1;
mod simple;
use dma::DMAInfo;
use mbc1::MBC1;
use simple::Simple as SimpleMBC;

use crate::{
    interrupt::{IntKind, InterruptControllerPtr},
    serial::SerialPtr,
};

pub type BoxMBC = Box<dyn MBC + Send + Sync>;

pub struct MMU {
    bootstrap_rom: Box<[u8; 0x100]>,
    mbc: BoxMBC,
    vram: Box<[u8; 0x2000]>,
    wram: Box<[u8; 0x2000]>,
    oam: Box<[u8; 0xA0]>,
    io_regs: Box<[u8; 0x80]>,
    hram: Box<[u8; 0x7F]>,
    serial: SerialPtr,
    interrupt_controller: InterruptControllerPtr,
    waiting_dma: Option<DMAInfo>,
}

const JOYPAD_STATUS_ADDR: u16 = 0xFF00;

const SERIAL_TRANSFER_DATA_ADDR: u16 = 0xFF01;
const SERIAL_TRANSFER_CONTROL_ADDR: u16 = 0xFF02;

const LCD_OAM_DMA_ADDR: u16 = 0xFF46;

const BOOTSTRAP_ROM_MOUNT_CONTROL_ADDR: u16 = 0xFF50;

const DIVIDER_REGISTER_ADDR: u16 = 0xFF04;
const TIMER_COUNTER_ADDR: u16 = 0xFF05;
const TIMER_MODULO_ADDR: u16 = 0xFF06;
const TIMER_CONTROL_ADDR: u16 = 0xFF07;

const INTERRUPT_FLAG_ADDR: u16 = 0xFF0F;

impl MMU {
    pub fn new(mbc: BoxMBC, int_controller: InterruptControllerPtr, serial: SerialPtr) -> Self {
        let mut mmu = MMU {
            bootstrap_rom: Box::new([0; 0x100]),
            mbc,
            vram: Box::new([0; 0x2000]),
            wram: Box::new([0; 0x2000]),
            oam: Box::new([0; 0xA0]),
            io_regs: Box::new([0; 0x80]),
            hram: Box::new([0; 0x7F]),
            serial,
            interrupt_controller: int_controller,
            waiting_dma: None,
        };
        mmu.init_default_values();
        mmu
    }

    fn init_default_values(&mut self) {
        self.write_memory(0xFF4D, 0xFF);
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
            // do nothing here
        }
    }

    pub fn unmount_bootstrap_rom(&mut self) {
        self.write_memory(BOOTSTRAP_ROM_MOUNT_CONTROL_ADDR, 1);
    }

    pub fn read_io_reg(&self, addr: u16) -> u8 {
        if addr == JOYPAD_STATUS_ADDR {
            self.interrupt_controller.lock().unwrap().read_joypad_reg()
        } else if addr == DIVIDER_REGISTER_ADDR {
            self.interrupt_controller.lock().unwrap().divider_register
        } else if addr == TIMER_COUNTER_ADDR {
            self.interrupt_controller.lock().unwrap().timer_counter
        } else if addr == TIMER_MODULO_ADDR {
            self.interrupt_controller.lock().unwrap().timer_modulo
        } else if addr == TIMER_CONTROL_ADDR {
            self.interrupt_controller.lock().unwrap().timer_control
        } else if addr == INTERRUPT_FLAG_ADDR {
            self.interrupt_controller
                .lock()
                .unwrap()
                .interrupt_flag
                .bits()
        } else {
            self.io_regs[addr as usize - 0xFF00]
        }
    }

    pub fn write_io_reg(&mut self, addr: u16, value: u8) {
        if addr == JOYPAD_STATUS_ADDR {
            self.interrupt_controller
                .lock()
                .unwrap()
                .write_joypad_reg(value)
        } else if addr == DIVIDER_REGISTER_ADDR {
            self.interrupt_controller.lock().unwrap().divider_register = 0;
        } else if addr == TIMER_COUNTER_ADDR {
            self.interrupt_controller.lock().unwrap().timer_counter = value;
        } else if addr == TIMER_MODULO_ADDR {
            self.interrupt_controller.lock().unwrap().timer_modulo = value;
        } else if addr == TIMER_CONTROL_ADDR {
            self.interrupt_controller.lock().unwrap().timer_control = value;
        } else if addr == INTERRUPT_FLAG_ADDR {
            self.interrupt_controller.lock().unwrap().interrupt_flag =
                IntKind::from_bits_truncate(value);
        } else {
            if addr == LCD_OAM_DMA_ADDR {
                if self.waiting_dma.is_some() {
                    error!("New DMA while another one was running");
                } else {
                    self.waiting_dma = Some(DMAInfo::new(value));
                }
            }
            self.io_regs[addr as usize - 0xFF00] = value;
        }
    }
}

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
            0xFF00..=0xFF7F => self.read_io_reg(addr),
            0xFF80..=0xFFFE => self.hram[addr as usize - 0xFF80],
            0xFFFF => self
                .interrupt_controller
                .lock()
                .unwrap()
                .interrupt_enable
                .bits(),
        }
    }

    fn write_memory(&mut self, addr: u16, value: u8) {
        // Used for test roms output
        if addr == SERIAL_TRANSFER_CONTROL_ADDR && value == 0x81 {
            let byte = self.read_memory(SERIAL_TRANSFER_DATA_ADDR);
            self.serial.lock().unwrap().write_byte(byte);
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
            0xFF00..=0xFF7F => self.write_io_reg(addr, value),
            0xFF80..=0xFFFE => self.hram[addr as usize - 0xFF80] = value,
            0xFFFF => {
                self.interrupt_controller.lock().unwrap().interrupt_enable =
                    IntKind::from_bits_truncate(value)
            }
        }
    }

    fn tick(&mut self) {
        if let Some(dma_info) = self.waiting_dma.as_mut() {
            if dma_info.tick() {
                let start_addr = (dma_info.high_byte_addr as u16) << 8;
                for offset in 0x00..0xA0 {
                    let src_addr = start_addr + offset;
                    let dest_addr = 0xFE00 + offset;
                    self.write_memory(dest_addr, self.read_memory(src_addr));
                }
                self.waiting_dma = None;
            }
        }
    }
}

pub trait Memory {
    fn read_memory(&self, addr: u16) -> u8;
    fn write_memory(&mut self, addr: u16, value: u8);
    fn tick(&mut self);
}

impl<M: Memory> Memory for Arc<RwLock<M>> {
    fn read_memory(&self, addr: u16) -> u8 {
        self.read().unwrap().read_memory(addr)
    }

    fn write_memory(&mut self, addr: u16, value: u8) {
        self.write().unwrap().write_memory(addr, value);
    }

    fn tick(&mut self) {
        self.write().unwrap().tick();
    }
}

pub trait MBC {
    fn read_memory(&self, addr: u16) -> u8;
    fn write_memory(&mut self, addr: u16, value: u8);
}

pub fn build_mbc(content: &[u8]) -> BoxMBC {
    const CARTRIDGE_TYPE_ADDR: usize = 0x0147;
    const CARTRIDGE_ROM_SIZE_ADDR: usize = 0x0148;
    const CARTRIDGE_RAM_SIZE_ADDR: usize = 0x0149;

    let rom_size = (1 << 15) << content[CARTRIDGE_ROM_SIZE_ADDR];
    assert_eq!(rom_size, content.len());

    let ram_size = match content[CARTRIDGE_RAM_SIZE_ADDR] {
        0x00 => 0,
        0x01 => 1 << 11,
        0x02 => 1 << 13,
        0x03 => 1 << 15,
        0x04 => 1 << 17,
        0x05 => 1 << 16,
        _ => panic!("Unknown RAM Size"),
    };

    match content[CARTRIDGE_TYPE_ADDR] {
        0x00 => Box::new(SimpleMBC::new(content)),
        0x01 => Box::new(MBC1::new(content, rom_size, 0)),
        0x02 | 0x03 => Box::new(MBC1::new(content, rom_size, ram_size)),
        _ => unimplemented!(),
    }
}
