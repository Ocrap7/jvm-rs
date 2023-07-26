// pub trait Instruction {
//     const OPCODE: u8;
//     const OPERANDS: u8;
// }

// macro_rules! impl_op {
//     ($ty:ty, $op:expr, $ops:expr) => {
//         impl $ty {
//             pub const OPCODE: u8 = $op;
//             pub const OPERANDS: u8 = $ops;
//         }
//     };
// }

use std::fmt::Display;

use crate::byte_stream::{ByteStream, ReaderContext};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Nop = 0,
    BiPush = 0x10,
    SiPush = 0x11,

    // Integer operations
    IConstM1 = 0x2,
    IConst0 = 0x3,
    IConst1 = 0x4,
    IConst2 = 0x5,
    IConst3 = 0x6,
    IConst4 = 0x7,
    IConst5 = 0x8,

    ILoad = 0x15,
    ILoad0 = 0x1a,
    ILoad1 = 0x1b,
    ILoad2 = 0x1c,
    ILoad3 = 0x1d,

    IStore = 0x36,
    IStore0 = 0x3b,
    IStore1 = 0x3c,
    IStore2 = 0x3d,
    IStore3 = 0x3e,

    IAdd = 0x60,
    ISub = 0x64,
    IMul = 0x68,
    IDiv = 0x6C,
    IRem = 0x70,
    INeg = 0x74,
    IShl = 0x78,
    IShr = 0x7a,
    IUShr = 0x7c,
    IAnd = 0x7e,
    IOr = 0x80,
    IXOr = 0x82,
    IInc = 0x84,

    I2L = 0x85,
    I2F = 0x86,
    I2D = 0x87,
    I2B = 0x91,
    I2C = 0x92,
    I2S = 0x93,

    LConst0 = 0x9,
    LConst1 = 0xa,

    LLoad = 0x16,
    LLoad0 = 0x1e,
    LLoad1 = 0x1f,
    LLoad2 = 0x20,
    LLoad3 = 0x21,

    LStore = 0x37,
    LStore0 = 0x3f,
    LStore1 = 0x40,
    LStore2 = 0x41,
    LStore3 = 0x42,

    LAdd = 0x61,
    LSub = 0x65,
    LMul = 0x69,
    LDiv = 0x6d,
    LRem = 0x71,
    LNeg = 0x75,
    LShl = 0x79,
    LShr = 0x7b,
    LUShr = 0x7d,
    LAnd = 0x7f,
    LOr = 0x81,
    LXOr = 0x83,

    L2I = 0x88,
    L2F = 0x89,
    L2D = 0x8a,

    FConst0 = 0xb,
    FConst1 = 0xc,
    FConst2 = 0xd,

    FLoad = 0x17,
    FLoad0 = 0x22,
    FLoad1 = 0x23,
    FLoad2 = 0x24,
    FLoad3 = 0x25,

    FStore = 0x38,
    FStore0 = 0x43,
    FStore1 = 0x44,
    FStore2 = 0x45,
    FStore3 = 0x46,

    FAdd = 0x62,
    FSub = 0x66,
    FMul = 0x6a,
    FDiv = 0x6e,
    FRem = 0x72,
    FNeg = 0x76,

    F2L = 0x8b,
    F2I = 0x8c,
    F2D = 0x8d,

    DConst0 = 0xe,
    DConst1 = 0xf,

    DLoad = 0x18,
    DLoad0 = 0x26,
    DLoad1 = 0x27,
    DLoad2 = 0x28,
    DLoad3 = 0x29,

    DStore = 0x39,
    DStore0 = 0x47,
    DStore1 = 0x48,
    DStore2 = 0x49,
    DStore3 = 0x4a,

    DAdd = 0x63,
    DSub = 0x67,
    DMul = 0x6b,
    DDiv = 0x6f,
    DRem = 0x73,
    DNeg = 0x77,

    D2L = 0x8f,
    D2I = 0x8e,
    D2F = 0x90,

    Dup = 0x59,
    DupX1 = 0x5a,
    DupX2 = 0x5b,
    Dup2 = 0x5c,
    Dup2X1 = 0x5d,
    Dup2X2 = 0x5e,

    Goto = 0xa7,
    GotoW = 0xc8,

    ICmpEq = 0x9f,
    ICmpNe = 0xa0,
    ICmpLt = 0xa1,
    ICmpGe = 0xa2,
    ICmpGt = 0xa3,
    ICmpLe = 0xa4,

    LCmp = 0x94,

    FCmpl = 0x95,
    FCmpg = 0x96,
    DCmpl = 0x97,
    DCmpg = 0x98,

    IEq = 0x99,
    INe = 0x9a,
    ILt = 0x9b,
    IGe = 0x9c,
    IGt = 0x9d,
    ILe = 0x9e,

    IfNull = 0xc6,
    IfNotNull = 0xc7,

    Return = 0xb1,
    IReturn = 0xac,
    LReturn = 0xad,
    FReturn = 0xae,
    DReturn = 0xaf,

    Pop = 0x57,
    Pop2 = 0x58,
    Swap = 0x5f,

    // Array Methods
    ALoad = 0x19,
    ALoad0 = 0x2a,
    ALoad1 = 0x2b,
    ALoad2 = 0x2c,
    ALoad3 = 0x2d,

    AStore = 0x3a,
    AStore0 = 0x4b,
    AStore1 = 0x4c,
    AStore2 = 0x4d,
    AStore3 = 0x4e,

    CALoad = 0x34,
    BALoad = 0x33,
    SALoad = 0x35,
    IALoad = 0x2e,
    LALoad = 0x2f,
    FALoad = 0x30,
    DALoad = 0x31,

    CAStore = 0x55,
    BAStore = 0x54,
    SAStore = 0x56,
    IAStore = 0x4f,
    LAStore = 0x50,
    FAStore = 0x51,
    DAStore = 0x52,

    NewArray = 0xbc,
    ArrayLength = 0xbe,

    // Object methods
    GetStatic = 0xb2,
    PutStatic = 0xb3,
    InvokeStatic = 0xb8,
    InvokeSpecial = 0xb7,
}

impl Instruction {
    /// Returns size of instruction's operands in bytes
    pub fn operands_size(&self) -> u8 {
        match self {
            Instruction::BiPush => 1,
            Instruction::SiPush => 1,

            Instruction::IInc => 2,
            Instruction::IStore => 1,
            Instruction::ILoad => 1,
            Instruction::LStore => 1,
            Instruction::LLoad => 1,
            Instruction::FStore => 1,
            Instruction::FLoad => 1,
            Instruction::DStore => 1,
            Instruction::DLoad => 1,

            Instruction::Goto => 2,
            Instruction::GotoW => 4,

            Instruction::ICmpEq => 2,
            Instruction::ICmpNe => 2,
            Instruction::ICmpLt => 2,
            Instruction::ICmpGe => 2,
            Instruction::ICmpGt => 2,
            Instruction::ICmpLe => 2,

            Instruction::IEq => 2,
            Instruction::INe => 2,
            Instruction::ILt => 2,
            Instruction::IGe => 2,
            Instruction::IGt => 2,
            Instruction::ILe => 2,

            Instruction::IfNotNull => 2,
            Instruction::IfNull => 2,

            Instruction::ALoad => 1,
            Instruction::AStore => 1,
            Instruction::NewArray => 1,

            Instruction::InvokeStatic => 2,
            Instruction::InvokeSpecial => 2,
            Instruction::PutStatic => 2,
            Instruction::GetStatic => 2,
            _ => 0,
        }
    }
}

impl From<u8> for Instruction {
    fn from(value: u8) -> Self {
        let ptr: *const u8 = &value;
        unsafe { *(ptr as *const Instruction) }
    }
}

pub struct Format<'a>(&'a [u8]);

impl<'a> From<&'a [u8]> for Format<'a> {
    fn from(value: &'a [u8]) -> Self {
        Format(value)
    }
}

impl Display for Format<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_instructions(self.0, f)
    }
}

fn format_instructions(data: &[u8], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let ctx = ReaderContext {
        constant_pool: Vec::new(),
    };
    let mut stream = ByteStream::new(data);

    write!(f, "Instructions:\n")?;

    while stream.has_next() {
        let op: u8 = stream.read(&ctx);
        let instr = Instruction::from(op);

        write!(f, "    {op:02x} {instr:?}")?;
        let operands = instr.operands_size();

        if operands > 0 {
            write!(f, "(")?;
            for i in 0..operands {
                let operand = stream.read::<u8>(&ctx);
                if i == operands - 1 {
                    write!(f, "0x{:x}", operand)?;
                } else {
                    write!(f, "0x{:x}, ", operand)?;
                }
            }
            write!(f, ")")?;
        }
        writeln!(f)?;
    }

    Ok(())
}

// pub struct Add {}

// impl_op!(Add, 0x89, 2);

// pub struct BiPush {}

// impl_op!(BiPush, 0x10, 1);
