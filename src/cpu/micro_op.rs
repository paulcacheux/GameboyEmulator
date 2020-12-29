use super::{JumpCondition, PrePostOperation, Register16, Register8};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Destination8Bits {
    Register(Register8),
    Indirect(Register16),
    Address(u16),
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source8bits {
    Register(Register8),
    Indirect(Register16),
    Address(u16),
    Literal(u8),
}

impl From<Register8> for Source8bits {
    fn from(reg: Register8) -> Self {
        Source8bits::Register(reg)
    }
}

impl From<u8> for Source8bits {
    fn from(literal: u8) -> Self {
        Source8bits::Literal(literal)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MicroOp {
    NOP,
    Move8Bits {
        destination: Destination8Bits,
        source: Source8bits,
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
    SubA {
        rhs: Source8bits,
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
    RotateRightThroughCarry {
        reg: Register8,
        set_zero: bool,
    },
    ShiftRightIntoCarry {
        reg: Register8,
    },
    CheckFlags {
        condition: JumpCondition,
        true_ops: Vec<MicroOp>,
        false_ops: Vec<MicroOp>,
    },
    RelativeJump(i8),
    EnableInterrupts,
    DisableInterrupts,
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
}
