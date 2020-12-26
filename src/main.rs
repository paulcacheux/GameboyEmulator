use simple_logger::SimpleLogger;

mod cpu;
mod memory;
mod utils;

use cpu::CPU;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new().init()?;

    let rom_path = std::env::args().nth(1).expect("Failed to get rom path");
    let rom_content = std::fs::read(&rom_path)?;

    let mut mmu = memory::MMU::new();
    mmu.write_slice(&rom_content, 0x0);

    let mut cpu = CPU::new(mmu);
    loop {
        cpu.step();
    }
}
