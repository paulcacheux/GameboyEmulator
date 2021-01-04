use std::{
    io::{stdout, Write},
    sync::{Arc, Mutex},
};

pub type SerialPtr = Arc<Mutex<Box<dyn SerialWrite + Send + Sync>>>;

pub trait SerialWrite {
    fn write_byte(&mut self, byte: u8);
}

pub struct StdoutSerialWrite;

impl SerialWrite for StdoutSerialWrite {
    fn write_byte(&mut self, byte: u8) {
        print!("{}", byte as char);
        let _ = stdout().flush();
    }
}
