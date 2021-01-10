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

const CPU_INSTRS_EXPECTED_OUTPUT: &str = "cpu_instrs\n\n01:ok  02:ok  03:ok  04:ok  05:ok  06:ok  07:ok  08:ok  09:ok  10:ok  11:ok  \n\nPassed all tests";
const INSTR_TIMING_EXPECTED_OUTPUT: &str = "instr_timing\n\n\nPassed";

fn blargg_test(rom_path: &str, timemout: std::time::Duration, expected_output: &str) {
    let output_content = Arc::new(Mutex::new(String::new()));
    let serial: SerialPtr = Box::new(SerialDebug::new(output_content.clone()));
    let mut emu = common::setup_rom(rom_path, Some(serial));

    let start_time = std::time::Instant::now();

    while start_time.elapsed() < timemout
        && output_content.lock().unwrap().len() < expected_output.len()
    {
        emu.cpu.step();
    }

    assert_eq!(output_content.lock().unwrap().as_str(), expected_output);
}

#[test]
fn test_blargg_cpu_instrs() {
    blargg_test(
        "./test_roms/blargg/cpu_instrs.gb",
        std::time::Duration::from_secs(10 * 60),
        CPU_INSTRS_EXPECTED_OUTPUT,
    );
}

#[test]
fn test_blargg_instr_timing() {
    blargg_test(
        "./test_roms/blargg/instr_timing.gb",
        std::time::Duration::from_secs(2 * 60),
        INSTR_TIMING_EXPECTED_OUTPUT,
    );
}
