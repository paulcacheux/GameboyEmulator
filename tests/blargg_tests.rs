use std::sync::{Arc, Mutex};

use gbemu::serial::{SerialPtr, SerialWrite};

mod common;

#[derive(Debug)]
struct SerialDebug {
    pub content: Arc<Mutex<String>>,
}
impl SerialDebug {
    fn new(content_ptr: Arc<Mutex<String>>) -> Self {
        SerialDebug {
            content: content_ptr,
        }
    }
}

impl SerialWrite for SerialDebug {
    fn write_byte(&mut self, byte: u8) {
        self.content.lock().unwrap().push(byte as char);
    }
}

const EXPECTED_OUTPUT: &str = "cpu_instrs\n\n01:ok  02:ok  03:ok  04:ok  05:ok  06:ok  07:ok  08:ok  09:ok  10:ok  11:ok  \n\nPassed all tests";

#[test]
fn test_blargg_cpu_instrs() {
    let rom_path = "./test_roms/blargg/cpu_instrs.gb";
    let output_content = Arc::new(Mutex::new(String::new()));
    let serial: SerialPtr = Arc::new(Mutex::new(Box::new(SerialDebug::new(
        output_content.clone(),
    ))));
    let mut emu = common::setup_rom(rom_path, Some(serial));

    let start_time = std::time::Instant::now();

    while start_time.elapsed() < std::time::Duration::from_secs(10 * 60)
        && output_content.lock().unwrap().len() < EXPECTED_OUTPUT.len()
    {
        emu.cpu.step();
        emu.ppu.step();
    }

    assert_eq!(output_content.lock().unwrap().as_str(), EXPECTED_OUTPUT);
}
