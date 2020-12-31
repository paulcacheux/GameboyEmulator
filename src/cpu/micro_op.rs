use super::{JumpCondition, PrePostOperation, Register16, Register8};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Destination8Bits {
    Register(Register8),
    Indirect(Register16),
    Address(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source8bits {
    Register(Register8),
    Indirect(Register16),
    Address(u16),
    Literal(u8),
    ZeroPageOffsetReg8(Register8),
}

impl From<Register8> for Source8bits {
    fn from(reg: Register8) -> Self {
        Source8bits::Register(reg)
    }
}

impl From<Register16> for Source8bits {
    fn from(reg: Register16) -> Self {
        Source8bits::Indirect(reg)
    }
}

impl From<u8> for Source8bits {
    fn from(literal: u8) -> Self {
        Source8bits::Literal(literal)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reg8OrIndirect {
    Reg8(Register8),
    Indirect(Register16),
}

impl From<Register8> for Reg8OrIndirect {
    fn from(reg: Register8) -> Self {
        Reg8OrIndirect::Reg8(reg)
    }
}

impl From<Register16> for Reg8OrIndirect {
    fn from(reg: Register16) -> Self {
        Reg8OrIndirect::Indirect(reg)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MicroOp {
    NOP,
    Move8Bits {
        destination: Destination8Bits,
        source: Source8bits,
    },
    Move16Bits {
        destination: Register16,
        source: Register16,
    },
    LoadReg16Lit {
        reg: Register16,
        literal: u16,
    },
    AndA {
        rhs: Source8bits,
    },
    OrA {
        rhs: Source8bits,
    },
    XorA {
        rhs: Source8bits,
    },
    AddA {
        rhs: Source8bits,
    },
    AddHL {
        rhs: Register16,
    },
    AddOffsetToReg16IntoReg16 {
        dest: Register16,
        rhs: Register16,
        offset: i8,
        update_flags: bool,
    },
    AdcA {
        rhs: Source8bits,
    },
    SubA {
        rhs: Source8bits,
    },
    SbcA {
        rhs: Source8bits,
    },
    DAA,
    ComplementA,
    WriteMem {
        addr: Register16,
        reg: Register8,
        pre_op: Option<PrePostOperation>,
        post_op: Option<PrePostOperation>,
    },
    WriteMemZeroPage {
        reg_offset: Register8,
        reg: Register8,
    },
    ReadMem {
        reg: Register8,
        addr: Register16,
        post_op: Option<PrePostOperation>,
    },
    BitTest {
        reg: Reg8OrIndirect,
        bit: u8,
    },
    ResetBit {
        reg: Reg8OrIndirect,
        bit: u8,
    },
    SetBit {
        reg: Reg8OrIndirect,
        bit: u8,
    },
    IncReg16 {
        reg: Register16,
    },
    Inc {
        reg: Reg8OrIndirect,
    },
    DecReg16 {
        reg: Register16,
    },
    Dec {
        reg: Reg8OrIndirect,
    },
    CompareA {
        rhs: Source8bits,
    },
    RotateLeftThroughCarry {
        reg: Reg8OrIndirect,
        set_zero: bool,
    },
    RotateRightThroughCarry {
        reg: Reg8OrIndirect,
        set_zero: bool,
    },
    RotateLeft {
        reg: Reg8OrIndirect,
        set_zero: bool,
    },
    RotateRight {
        reg: Reg8OrIndirect,
        set_zero: bool,
    },
    ShiftLeftIntoCarry {
        reg: Reg8OrIndirect,
    },
    ShiftRightWithZeroIntoCarry {
        reg: Reg8OrIndirect,
    },
    ShiftRightWithSignIntoCarry {
        reg: Reg8OrIndirect,
    },
    SwapReg8 {
        reg: Reg8OrIndirect,
    },
    CheckFlags {
        condition: JumpCondition,
        true_ops: Vec<MicroOp>,
        false_ops: Vec<MicroOp>,
    },
    SetCarryFlag,
    ComplementCarryFlag,
    EnableInterrupts,
    DisableInterrupts,
    Halt,
}

pub mod simpl {
    use super::*;

    pub fn load_literal_into_reg8(literal: u8, reg: Register8) -> MicroOp {
        MicroOp::Move8Bits {
            destination: Destination8Bits::Register(reg),
            source: Source8bits::Literal(literal),
        }
    }

    pub fn move_micro_op(destination: Register8, src: Register8) -> MicroOp {
        MicroOp::Move8Bits {
            destination: Destination8Bits::Register(destination),
            source: Source8bits::Register(src),
        }
    }

    pub fn jump_relative(offset: i8) -> MicroOp {
        MicroOp::AddOffsetToReg16IntoReg16 {
            dest: Register16::PC,
            rhs: Register16::PC,
            offset,
            update_flags: false,
        }
    }
}
