use gbemu::{ppu::PIXEL_COUNT, Memory};
use image::RgbaImage;

mod common;

fn read_img_file(path: &str) -> image::RgbaImage {
    let img = image::open(path).unwrap();
    let img = img.to_rgba8();
    img
}

#[test]
fn test_acid2() {
    let rom_path = "./test_roms/acid2/dmg-acid2.gb";
    let mut emu = common::setup_rom(rom_path);

    while emu.memory.read_memory(emu.cpu.pc) != 0x40 || !emu.cpu.is_pipeline_empty() {
        // breakpoint at LD B, B
        emu.cpu.step();
        emu.ppu.step();
    }

    let mut fb = vec![0; PIXEL_COUNT * 4];
    emu.display.lock().unwrap().draw_into_fb(&mut fb);

    let res_img = RgbaImage::from_raw(160, 144, fb).unwrap();

    let expected_img = read_img_file("./test_roms/acid2/reference-dmg.png");
    assert_eq!(res_img, expected_img);
}
