use std::{rc::Rc, sync::RwLock};

use ppu::PPU;

mod cpu;
mod memory;
mod ppu;
mod utils;

use cpu::CPU;
use memory::Memory;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let bootstrap_path = std::env::args()
        .nth(1)
        .expect("Failed to get bootstrap path");
    let rom_path = std::env::args().nth(2).expect("Failed to get rom path");

    let bootstrap_content = std::fs::read(&bootstrap_path)?;
    let rom_content = std::fs::read(&rom_path)?;

    let mut mmu = memory::MMU::new();
    mmu.write_slice(&rom_content, 0x0);
    mmu.write_slice(&bootstrap_content, 0x0);

    let memory = Rc::new(RwLock::new(mmu));

    let mut cpu = CPU::new(memory.clone());
    let mut ppu = PPU::new(memory);

    loop {
        cpu.step();
        ppu.step();
    }
}
