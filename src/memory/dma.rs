pub struct DMAInfo {
    pub high_byte_addr: u8,
    timer: u8,
}

impl DMAInfo {
    pub fn new(high_byte_addr: u8) -> Self {
        DMAInfo {
            high_byte_addr,
            timer: 0xA0,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.timer -= 1;
        self.timer == 0
    }
}
