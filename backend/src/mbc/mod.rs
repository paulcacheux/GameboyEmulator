use log::info;

pub mod mbc1;
pub mod simple;

pub use mbc1::MBC1;
pub use simple::Simple as SimpleMBC;

pub type BoxMBC = Box<dyn MBC + Send + Sync>;

pub trait MBC {
    fn read_memory(&self, addr: u16) -> u8;
    fn write_memory(&mut self, addr: u16, value: u8);
}

const CARTRIDGE_TYPE_ADDR: usize = 0x0147;
const CARTRIDGE_ROM_SIZE_ADDR: usize = 0x0148;
const CARTRIDGE_RAM_SIZE_ADDR: usize = 0x0149;
const CARTRIDGE_CGB_FLAG_ADDR: usize = 0x0143;

pub fn read_cartridge(content: &[u8]) -> (BoxMBC, CGBFlag) {
    let rom_size_tag = content[CARTRIDGE_ROM_SIZE_ADDR];
    if rom_size_tag > 0x08 {
        unimplemented!()
    }

    let rom_size = (1 << 15) << rom_size_tag;
    assert_eq!(rom_size, content.len());

    let ram_size = match content[CARTRIDGE_RAM_SIZE_ADDR] {
        0x00 => 0,
        0x01 => 1 << 11,
        0x02 => 1 << 13,
        0x03 => 1 << 15,
        0x04 => 1 << 17,
        0x05 => 1 << 16,
        _ => panic!("Unknown RAM Size"),
    };

    let cgb_flag_bits = content[CARTRIDGE_CGB_FLAG_ADDR];
    info!("CGB flag: {:x}", cgb_flag_bits);
    let cgb_flag = match cgb_flag_bits {
        0x80 => CGBFlag::CGBFeatures,
        0xC0 => CGBFlag::CGBOnly,
        _ => CGBFlag::DMG,
    };

    let mbc: BoxMBC = match content[CARTRIDGE_TYPE_ADDR] {
        0x00 => Box::new(SimpleMBC::new(content)),
        0x01 => Box::new(MBC1::new(content, rom_size, 0)),
        0x02 | 0x03 => Box::new(MBC1::new(content, rom_size, ram_size)),
        other_tag => {
            unimplemented!("MBG tag {:#x} is not implemented", other_tag)
        }
    };

    (mbc, cgb_flag)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CGBFlag {
    DMG,
    CGBFeatures,
    CGBOnly,
}
