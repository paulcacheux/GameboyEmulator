use std::sync::{Arc, RwLock};

use log::{debug, error, warn};

mod dma;
pub mod mbc1;
pub mod simple;
use dma::DMAInfo;

use crate::{
    interrupt::{IntKind, InterruptControllerPtr},
    serial::SerialPtr,
};

pub type BoxMBC = Box<dyn MBC + Send + Sync>;

const VRAM_BANK_COUNT: usize = 2;
const WRAM_BANK_COUNT: usize = 8;

pub struct MMU {
    bootstrap_rom: Box<[u8; 0x100]>,
    mbc: BoxMBC,
    vram: Box<[[u8; 0x2000]; VRAM_BANK_COUNT]>,
    vram_bank_index: u8,
    wram: Box<[[u8; 0x2000]; WRAM_BANK_COUNT]>,
    wram_second_bank_index: u8,
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

const VRAM_BANK_CONTROL_ADDR: u16 = 0xFF4F;
const WRAM_BANK_CONTROL_ADDR: u16 = 0xFF70;
const CGB_MODE_KEY1_ADDR: u16 = 0xFF4D;

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
            vram: Box::new([[0; 0x2000]; VRAM_BANK_COUNT]),
            vram_bank_index: 0,
            wram: Box::new([[0; 0x2000]; WRAM_BANK_COUNT]),
            wram_second_bank_index: 0,
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

    pub fn switch_vram_bank(&mut self, new_bank_index: u8) {
        debug_assert!((new_bank_index as usize) < VRAM_BANK_COUNT);

        if !self.interrupt_controller.lock().unwrap().cgb_mode {
            // TODO: should we really be checking that and not another bool ??
            error!("Tried to switch VRAM bank without being in CGB Mode");
        } else {
            self.vram_bank_index = new_bank_index;
        }
    }

    pub fn switch_wram_bank(&mut self, new_bank_index: u8) {
        debug_assert!((new_bank_index as usize) < WRAM_BANK_COUNT);
        debug_assert_ne!(new_bank_index, 0);

        if !self.interrupt_controller.lock().unwrap().cgb_mode {
            // TODO: should we really be checking that and not another bool ??
            error!("Tried to switch WRAM bank without being in CGB Mode");
        } else {
            self.wram_second_bank_index = new_bank_index;
        }
    }

    pub fn read_io_reg(&self, addr: u16) -> u8 {
        match addr {
            JOYPAD_STATUS_ADDR => self.interrupt_controller.lock().unwrap().read_joypad_reg(),
            DIVIDER_REGISTER_ADDR => self.interrupt_controller.lock().unwrap().divider_register,
            TIMER_COUNTER_ADDR => self.interrupt_controller.lock().unwrap().timer_counter,
            TIMER_MODULO_ADDR => self.interrupt_controller.lock().unwrap().timer_modulo,
            TIMER_CONTROL_ADDR => self.interrupt_controller.lock().unwrap().timer_control,
            INTERRUPT_FLAG_ADDR => self
                .interrupt_controller
                .lock()
                .unwrap()
                .interrupt_flag
                .bits(),
            VRAM_BANK_CONTROL_ADDR => {
                debug_assert!((self.vram_bank_index as usize) < VRAM_BANK_COUNT);
                (!0b1) | self.vram_bank_index // bit-0 to index, all other bits to 1
            }
            WRAM_BANK_CONTROL_ADDR => {
                debug_assert!((self.wram_second_bank_index as usize) < WRAM_BANK_COUNT);
                debug_assert_ne!(self.wram_second_bank_index, 0);
                (!0b11) | self.wram_second_bank_index
            }
            CGB_MODE_KEY1_ADDR => {
                let controller = self.interrupt_controller.lock().unwrap();
                let mode = controller.cgb_mode as u8;
                let prepare = controller.requested_new_mode.is_some() as u8;
                mode << 7 | prepare
            }
            _ => self.io_regs[addr as usize - 0xFF00],
        }
    }

    pub fn write_io_reg(&mut self, addr: u16, value: u8) {
        match addr {
            JOYPAD_STATUS_ADDR => self
                .interrupt_controller
                .lock()
                .unwrap()
                .write_joypad_reg(value),
            DIVIDER_REGISTER_ADDR => self.interrupt_controller.lock().unwrap().divider_register = 0,
            TIMER_COUNTER_ADDR => self.interrupt_controller.lock().unwrap().timer_counter = value,
            TIMER_MODULO_ADDR => self.interrupt_controller.lock().unwrap().timer_modulo = value,
            TIMER_CONTROL_ADDR => self.interrupt_controller.lock().unwrap().timer_control = value,
            INTERRUPT_FLAG_ADDR => {
                self.interrupt_controller.lock().unwrap().interrupt_flag =
                    IntKind::from_bits_truncate(value)
            }
            VRAM_BANK_CONTROL_ADDR => {
                let index = 0x1 & value;
                self.switch_vram_bank(index);
            }
            WRAM_BANK_CONTROL_ADDR => {
                let index = 0b11 & value;
                self.switch_wram_bank(if index == 0 { 1 } else { index });
            }
            CGB_MODE_KEY1_ADDR => {
                let new_cgb_mode = (value & 0b1) == 0b1;
                self.interrupt_controller.lock().unwrap().requested_new_mode = Some(new_cgb_mode);
            }
            _ => {
                if addr == LCD_OAM_DMA_ADDR {
                    if self.waiting_dma.is_some() {
                        warn!("New DMA while another one was running");
                    } else {
                        self.waiting_dma = Some(DMAInfo::new(value));
                    }
                }
                self.io_regs[addr as usize - 0xFF00] = value;
            }
        }
    }
}

impl Memory for MMU {
    fn read_memory(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x00FF => self.read_mounted_rom(addr),
            0x0100..=0x7FFF => self.mbc.read_memory(addr),
            0x8000..=0x9FFF => self.vram[self.vram_bank_index as usize][addr as usize - 0x8000],
            0xA000..=0xBFFF => self.mbc.read_memory(addr),
            0xC000..=0xCFFF => self.wram[0][addr as usize - 0xC000],
            0xD000..=0xDFFF => {
                self.wram[self.wram_second_bank_index as usize][addr as usize - 0xC000]
            }
            0xE000..=0xFDFF => self.read_memory(addr - 0xE000),
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
            self.serial.write_byte(byte);
        }

        match addr {
            0x0000..=0x00FF => self.write_mounted_rom(addr, value),
            0x0100..=0x7FFF => self.mbc.write_memory(addr, value),
            0x8000..=0x9FFF => {
                self.vram[self.vram_bank_index as usize][addr as usize - 0x8000] = value
            }
            0xA000..=0xBFFF => self.mbc.write_memory(addr, value),
            0xC000..=0xCFFF => self.wram[0][addr as usize - 0xC000] = value,
            0xD000..=0xDFFF => {
                self.wram[self.wram_second_bank_index as usize][addr as usize - 0xC000] = value
            }
            0xE000..=0xFDFF => self.write_memory(addr - 0xE000, value),
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

    fn read_vram(&self, addr: u16, bank: u8) -> u8 {
        debug_assert!(0x8000 <= addr);
        debug_assert!(addr <= 0x9FFF);
        debug_assert!((bank as usize) < VRAM_BANK_COUNT);

        self.vram[bank as usize][addr as usize - 0x8000]
    }

    fn write_vram(&mut self, addr: u16, bank: u8, value: u8) {
        debug_assert!(0x8000 <= addr);
        debug_assert!(addr <= 0x9FFF);
        debug_assert!((bank as usize) < VRAM_BANK_COUNT);

        self.vram[bank as usize][addr as usize - 0x8000] = value;
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

    fn read_vram(&self, addr: u16, bank: u8) -> u8;
    fn write_vram(&mut self, addr: u16, bank: u8, value: u8);

    fn tick(&mut self);
}

impl<M: Memory> Memory for Arc<RwLock<M>> {
    fn read_memory(&self, addr: u16) -> u8 {
        self.read().unwrap().read_memory(addr)
    }

    fn write_memory(&mut self, addr: u16, value: u8) {
        self.write().unwrap().write_memory(addr, value);
    }

    fn read_vram(&self, addr: u16, bank: u8) -> u8 {
        self.read().unwrap().read_vram(addr, bank)
    }

    fn write_vram(&mut self, addr: u16, bank: u8, value: u8) {
        self.write().unwrap().write_vram(addr, bank, value);
    }

    fn tick(&mut self) {
        self.write().unwrap().tick();
    }
}

pub trait MBC {
    fn read_memory(&self, addr: u16) -> u8;
    fn write_memory(&mut self, addr: u16, value: u8);
}
