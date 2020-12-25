#[derive(Debug, Clone)]
pub struct MMU {
    mem: Box<[u8; 0x10000]>,
}

impl MMU {
    pub fn new() -> Self {
        MMU {
            mem: Box::new([0; 0x10000]),
        }
    }

    pub fn write_slice(&mut self, slice: &[u8], start_addr: u16) {
        for (i, value) in slice.iter().enumerate() {
            self.write_memory(start_addr + i as u16, *value);
        }
    }

    pub fn read_memory(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    pub fn write_memory(&mut self, addr: u16, value: u8) {
        self.mem[addr as usize] = value;
    }
}
