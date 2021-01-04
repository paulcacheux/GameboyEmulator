use std::sync::{Arc, Mutex, RwLock};

use gbemu::{
    display::Display,
    interrupt::{InterruptController, InterruptControllerPtr},
    memory::{self, MMU},
    CPU, PPU,
};

type MMUPtr = Arc<RwLock<MMU>>;
type DisplayPtr = Arc<Mutex<Display>>;

pub struct EmuComponents {
    pub interrupt_controller: InterruptControllerPtr,
    pub memory: MMUPtr,
    pub cpu: CPU<MMUPtr>,
    pub ppu: PPU<MMUPtr>,
    pub display: DisplayPtr,
}

pub fn setup_rom(rom_path: &str) -> EmuComponents {
    let rom = std::fs::read(rom_path).unwrap();

    let interrupt_controller = Arc::new(Mutex::new(InterruptController::new()));

    let mbc = memory::build_mbc(&rom);
    let mut mmu = memory::MMU::new(mbc, interrupt_controller.clone());
    mmu.unmount_bootstrap_rom();

    let memory = Arc::new(RwLock::new(mmu));
    let display = Arc::new(Mutex::new(Display::new()));

    let mut cpu = CPU::new(memory.clone(), interrupt_controller.clone());
    cpu.pc = 0x100;

    let ppu = PPU::new(
        memory.clone(),
        interrupt_controller.clone(),
        display.clone(),
    );

    EmuComponents {
        interrupt_controller,
        memory,
        cpu,
        ppu,
        display,
    }
}
