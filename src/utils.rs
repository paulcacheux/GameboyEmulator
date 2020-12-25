pub fn combine(a: u8, b: u8) -> u16 {
    ((a as u16) << 8) | (b as u16)
}
