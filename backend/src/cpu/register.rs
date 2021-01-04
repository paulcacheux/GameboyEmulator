use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

impl Register16 {
    pub fn higher_half(self) -> Register8 {
        match self {
            Register16::AF => Register8::A,
            Register16::BC => Register8::B,
            Register16::DE => Register8::D,
            Register16::HL => Register8::H,
            Register16::SP => Register8::SPHigh,
            Register16::PC => {
                unimplemented!()
            }
        }
    }

    pub fn lower_half(self) -> Register8 {
        match self {
            Register16::AF => Register8::Flags,
            Register16::BC => Register8::C,
            Register16::DE => Register8::E,
            Register16::HL => Register8::L,
            Register16::SP => Register8::SPLow,
            Register16::PC => {
                unimplemented!()
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    SPHigh,
    SPLow,
    PCHigh,
    PCLow,
    Flags,
}

impl fmt::Display for Register16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Register16::AF => {
                write!(f, "AF")
            }
            Register16::BC => {
                write!(f, "BC")
            }
            Register16::DE => {
                write!(f, "DE")
            }
            Register16::HL => {
                write!(f, "HL")
            }
            Register16::SP => {
                write!(f, "SP")
            }
            Register16::PC => {
                write!(f, "PC")
            }
        }
    }
}

impl fmt::Display for Register8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Register8::B => {
                write!(f, "B")
            }
            Register8::A => {
                write!(f, "A")
            }
            Register8::C => {
                write!(f, "C")
            }
            Register8::D => {
                write!(f, "D")
            }
            Register8::E => {
                write!(f, "E")
            }
            Register8::H => {
                write!(f, "H")
            }
            Register8::L => {
                write!(f, "L")
            }
            Register8::SPHigh => {
                write!(f, "SP[high]")
            }
            Register8::SPLow => {
                write!(f, "SP[low]")
            }
            Register8::PCHigh => {
                write!(f, "PC[high]")
            }
            Register8::PCLow => {
                write!(f, "PC[low]")
            }
            Register8::Flags => {
                write!(f, "F")
            }
        }
    }
}
