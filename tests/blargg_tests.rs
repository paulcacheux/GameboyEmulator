mod common;

#[test]
fn test_blargg_cpu_instrs() {
    let rom_path = "./test_roms/blargg/cpu_instrs.gb";
    let mut emu = common::setup_rom(rom_path);

    let start_time = std::time::Instant::now();
}
