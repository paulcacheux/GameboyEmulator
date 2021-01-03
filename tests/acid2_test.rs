use std::sync::{Arc, Mutex, RwLock};

use gbemu::{
    display::Display, interrupt::InterruptController, memory, ppu::PIXEL_COUNT, Memory, CPU, PPU,
};
use image::RgbaImage;

fn read_img_file(path: &str) -> image::RgbaImage {
    let img = image::open(path).unwrap();
    let img = img.to_rgba8();
    img
}

#[test]
fn test_acid2() {
    let rom_path = "./test_roms/dmg-acid2.gb";
    let rom = std::fs::read(rom_path).unwrap();

    let interrupt_controller = Arc::new(Mutex::new(InterruptController::new()));

    let mbc = memory::build_mbc(&rom);
    let mut mmu = memory::MMU::new(mbc, interrupt_controller.clone());
    mmu.unmount_bootstrap_rom();

    let memory = Arc::new(RwLock::new(mmu));
    let display = Arc::new(Mutex::new(Display::new()));

    let mut cpu = CPU::new(memory.clone(), interrupt_controller.clone());
    cpu.pc = 0x100;

    let mut ppu = PPU::new(
        memory.clone(),
        interrupt_controller.clone(),
        display.clone(),
    );

    while memory.read_memory(cpu.pc) != 0x40 || !cpu.is_pipeline_empty() {
        // breakpoint at LD B, B
        cpu.step();
        ppu.step();
    }

    let mut fb = vec![0; PIXEL_COUNT * 4];
    display.lock().unwrap().draw_into_fb(&mut fb);

    let res_img = RgbaImage::from_raw(160, 144, fb).unwrap();

    let expected_img = read_img_file("./test_roms/reference-dmg.png");
    assert_eq!(res_img, expected_img);
}
