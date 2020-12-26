use std::{rc::Rc, sync::RwLock};

use ppu::PPU;
use simple_logger::SimpleLogger;

mod cpu;
mod memory;
mod ppu;
mod utils;

use cpu::CPU;
use memory::Memory;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new().init()?;

    let rom_path = std::env::args().nth(1).expect("Failed to get rom path");
    let rom_content = std::fs::read(&rom_path)?;

    let mut mmu = memory::MMU::new();
    if rom_path.contains("DMG_ROM") {
        mmu.write_slice(&rom_content, 0x0);
    } else {
        mmu.write_slice(&rom_content, 0x100);
    }

    let memory = Rc::new(RwLock::new(mmu));

    let mut cpu = CPU::new(memory.clone());
    let mut ppu = PPU::new(memory);

    loop {
        cpu.step();
        ppu.step();
    }
}
