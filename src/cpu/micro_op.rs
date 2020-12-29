use super::{JumpCondition, PrePostOperation, Register16, Register8};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move8BitsDestination {
    Register(Register8),
    Indirect(Register16),
    Address(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move8BitsSource {
    Register(Register8),
    Indirect(Register16),
    Address(u16),
    Literal(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicroOp {
    NOP,
    Move8Bits {
        destination: Move8BitsDestination,
        source: Move8BitsSource,
    },
    LoadReg16Lit {
        reg: Register16,
        literal: u16,
    },
    XorAReg {
        reg: Register8,
    },
    AddAIndirect {
        addr: Register16,
    },
    SubAReg {
        reg: Register8,
    },
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
        reg: Register8,
        bit: u8,
    },
    IncReg16 {
        reg: Register16,
    },
    IncReg {
        reg: Register8,
    },
    DecReg {
        reg: Register8,
    },
    CompareALit {
        literal: u8,
    },
    CompareAIndirect {
        addr: Register16,
    },
    RotateLeftThroughCarry {
        reg: Register8,
        set_zero: bool,
    },
    CheckFlags(JumpCondition),
    RelativeJump(i8),
    EnableInterrupts,
    DisableInterrupts,
}

pub mod simpl {
    use super::*;

    pub fn load_literal_into_reg8(literal: u8, reg: Register8) -> MicroOp {
        MicroOp::Move8Bits {
            destination: Move8BitsDestination::Register(reg),
            source: Move8BitsSource::Literal(literal),
        }
    }

    pub fn move_micro_op(destination: Register8, src: Register8) -> MicroOp {
        MicroOp::Move8Bits {
            destination: Move8BitsDestination::Register(destination),
            source: Move8BitsSource::Register(src),
        }
    }
}
